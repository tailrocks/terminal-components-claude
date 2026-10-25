use std::collections::HashMap;

use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::layer::LayerId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ScopeId(Id);

impl ScopeId {
    pub const ROOT: ScopeId = ScopeId(Id::root("tui.focus.root"));

    pub const fn new(id: Id) -> Self {
        ScopeId(id)
    }

    pub const fn id(self) -> Id {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeMode {
    Normal,
    Trap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focusability {
    Focusable,
    FocusableReadOnly,
    Disabled,
    ClickOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusVis {
    None,
    Focused,
    FocusedVisible,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FocusEntry {
    pub id: Id,
    pub scope: ScopeId,
    pub disabled: bool,
    pub area: Rect,
    pub layer: LayerId,
    pub swallows_typing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ScopeRecord {
    id: ScopeId,
    parent: Option<ScopeId>,
    mode: ScopeMode,
    layer: LayerId,
}

#[derive(Clone, Debug)]
pub struct FocusRing {
    entries: Vec<FocusEntry>,
    scopes: Vec<ScopeRecord>,
    open: Vec<ScopeId>,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusRing {
    pub fn new() -> Self {
        FocusRing {
            entries: Vec::new(),
            scopes: vec![ScopeRecord {
                id: ScopeId::ROOT,
                parent: None,
                mode: ScopeMode::Normal,
                layer: LayerId::PAGE,
            }],
            open: vec![ScopeId::ROOT],
        }
    }

    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.scopes.truncate(1);
        self.open.clear();
        self.open.push(ScopeId::ROOT);
    }

    pub fn entries(&self) -> &[FocusEntry] {
        &self.entries
    }

    pub(crate) fn current_scope(&self) -> ScopeId {
        self.open.last().copied().unwrap_or(ScopeId::ROOT)
    }

    pub(crate) fn push_scope(&mut self, id: ScopeId, mode: ScopeMode, layer: LayerId) {
        let parent = Some(self.current_scope());
        if let Some(r) = self.scopes.iter_mut().find(|r| r.id == id) {
            r.mode = mode;
            r.layer = layer;
            if r.parent.is_none() && id != ScopeId::ROOT {
                r.parent = parent;
            }
        } else {
            self.scopes.push(ScopeRecord {
                id,
                parent,
                mode,
                layer,
            });
        }
        self.open.push(id);
    }

    pub(crate) fn pop_scope(&mut self) {
        if self.open.len() > 1 {
            self.open.pop();
        }
    }

    pub(crate) fn ensure_scope(
        &mut self,
        id: ScopeId,
        mode: ScopeMode,
        layer: LayerId,
        parent: Option<ScopeId>,
    ) {
        if let Some(r) = self.scopes.iter_mut().find(|r| r.id == id) {
            r.mode = mode;
            r.layer = layer;
            if r.parent.is_none() && id != ScopeId::ROOT {
                r.parent = parent;
            }
        } else {
            self.scopes.push(ScopeRecord {
                id,
                parent: parent.or(Some(ScopeId::ROOT)),
                mode,
                layer,
            });
        }
    }

    pub(crate) fn register(&mut self, entry: FocusEntry) {
        self.entries.push(entry);
    }

    fn scope(&self, id: ScopeId) -> Option<&ScopeRecord> {
        self.scopes.iter().find(|r| r.id == id)
    }

    pub(crate) fn scope_layer(&self, id: ScopeId) -> Option<LayerId> {
        self.scope(id).map(|scope| scope.layer)
    }

    pub(crate) fn scope_depth(&self, id: ScopeId) -> Option<usize> {
        let mut current = Some(id);
        let mut depth = 0usize;
        for _ in 0..=self.scopes.len() {
            let scope = current?;
            depth = depth.saturating_add(1);
            current = self.scope(scope)?.parent;
            if current.is_none() {
                return Some(depth);
            }
        }
        None
    }

    pub fn active_trap(&self) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|r| r.mode == ScopeMode::Trap)
            .max_by(|a, b| a.layer.cmp(&b.layer))
            .map(|r| r.id)
    }

    pub fn innermost_scope(&self) -> ScopeId {
        self.scopes
            .iter()
            .max_by(|a, b| a.layer.cmp(&b.layer))
            .map_or(ScopeId::ROOT, |r| r.id)
    }

    pub fn within(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        let mut cur = Some(scope);

        for _ in 0..=self.scopes.len() {
            match cur {
                Some(s) if s == ancestor => return true,
                Some(s) => cur = self.scope(s).and_then(|r| r.parent),
                None => return false,
            }
        }
        false
    }

    fn is_reachable(&self, e: &FocusEntry, trap: Option<ScopeId>) -> bool {
        !e.disabled && trap.is_none_or(|t| self.within(e.scope, t))
    }

    pub fn reachable(&self) -> impl Iterator<Item = &FocusEntry> + '_ {
        let trap = self.active_trap();
        self.entries
            .iter()
            .filter(move |e| self.is_reachable(e, trap))
    }

    pub fn contains(&self, id: Id) -> bool {
        self.reachable().any(|e| e.id == id)
    }

    pub fn is_registered(&self, id: Id) -> bool {
        self.entries.iter().any(|e| e.id == id)
    }

    pub fn entry(&self, id: Id) -> Option<&FocusEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn next(&self, from: Option<Id>) -> Option<Id> {
        let first = self.reachable().next()?.id;
        let Some(from) = from else {
            return Some(first);
        };
        let mut seen = false;
        for e in self.reachable() {
            if seen {
                return Some(e.id);
            }
            if e.id == from {
                seen = true;
            }
        }
        Some(first)
    }

    pub fn prev(&self, from: Option<Id>) -> Option<Id> {
        let last = self.reachable().last()?.id;
        let Some(from) = from else {
            return Some(last);
        };
        let mut prev: Option<Id> = None;
        for e in self.reachable() {
            if e.id == from {
                return Some(prev.unwrap_or(last));
            }
            prev = Some(e.id);
        }
        Some(last)
    }

    pub fn first_in(&self, scope: ScopeId) -> Option<Id> {
        self.reachable()
            .find(|e| self.within(e.scope, scope))
            .map(|e| e.id)
    }

    pub fn reconcile(&self, prev: &FocusRing, current: Option<Id>) -> Option<Id> {
        if let Some(cur) = current
            && self.contains(cur)
        {
            return Some(cur);
        }
        let prev_entry = current.and_then(|c| prev.entry(c).copied());
        if let Some(pe) = prev_entry {
            let prev_index = prev.entries.iter().position(|e| e.id == pe.id);
            let trap = self.active_trap();

            let mut best: Option<(usize, Id, bool)> = None;
            for e in self.entries.iter().filter(|e| e.scope == pe.scope) {
                if !self.is_reachable(e, trap) {
                    continue;
                }
                let Some(pi) = prev.entries.iter().position(|p| p.id == e.id) else {
                    continue;
                };
                let (dist, forward) = match prev_index {
                    Some(ci) if pi >= ci => (pi.saturating_sub(ci), true),
                    Some(ci) => (ci.saturating_sub(pi), false),
                    None => (pi, true),
                };
                let better = match best {
                    None => true,
                    Some((bd, _, bf)) => dist < bd || (dist == bd && forward && !bf),
                };
                if better {
                    best = Some((dist, e.id, forward));
                }
            }
            if let Some((_, id, _)) = best {
                return Some(id);
            }

            if let Some(id) = self.first_in(pe.scope) {
                return Some(id);
            }
        }

        self.first_in(self.innermost_scope())
            .or_else(|| self.reachable().next().map(|e| e.id))
    }
}

#[derive(Clone, Debug, Default)]
pub struct FocusState {
    current: Option<Id>,
    visible: bool,
    restore: HashMap<ScopeId, Id>,
}

impl FocusState {
    pub const fn current(&self) -> Option<Id> {
        self.current
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }

    pub fn vis(&self, id: Id) -> FocusVis {
        if self.current != Some(id) {
            FocusVis::None
        } else if self.visible {
            FocusVis::FocusedVisible
        } else {
            FocusVis::Focused
        }
    }

    pub(crate) fn set(&mut self, id: Option<Id>) {
        self.current = id;
    }

    pub(crate) fn set_visible(&mut self, yes: bool) {
        self.visible = yes;
    }

    pub(crate) fn save_restore(&mut self, scope: ScopeId, id: Id) {
        self.restore.insert(scope, id);
    }

    pub(crate) fn take_restore(&mut self, scope: ScopeId) -> Option<Id> {
        self.restore.remove(&scope)
    }

    pub fn restore_target(&self, scope: ScopeId) -> Option<Id> {
        self.restore.get(&scope).copied()
    }
}
