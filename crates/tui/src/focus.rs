//! Focus ring, scopes, traps and restoration (`COMPONENT_ARCHITECTURE.md` §8.1).
//!
//! The ring is rebuilt every frame in registration order, so Tab order is
//! reading order. Scopes nest; a `Trap` scope confines traversal to itself
//! and its descendants and wraps inside it. A modal layer's trap is armed
//! when the layer is pushed, not when it draws, so a modal that fails to
//! draw still traps. Restoration is runtime-owned (`restore: ScopeId → Id`).

use std::collections::HashMap;

use ratatui_core::layout::Rect;

use crate::id::Id;
use crate::layer::LayerId;

/// A focus scope's identity (usually the owning layer's or container's id).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ScopeId(Id);

impl ScopeId {
    /// The implicit root scope of the page.
    pub const ROOT: ScopeId = ScopeId(Id::root("tui.focus.root"));

    /// A scope named by an id.
    pub const fn new(id: Id) -> Self {
        ScopeId(id)
    }

    /// The id.
    pub const fn id(self) -> Id {
        self.0
    }
}

/// Whether a scope traps traversal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeMode {
    /// Traversal flows through.
    Normal,
    /// Tab / Shift+Tab are confined to the scope and wrap inside it.
    Trap,
}

/// How a control participates in focus.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focusability {
    /// In the ring and reachable.
    Focusable,
    /// In the ring and reachable; the runtime marks it `READ_ONLY`.
    FocusableReadOnly,
    /// Recorded in the ring with `disabled: true`, never reachable.
    Disabled,
    /// A hit target only; never in the ring.
    ClickOnly,
}

/// Focus as seen by one control.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FocusVis {
    /// Not focused.
    None,
    /// Focused, indicator hidden (the last input was a pointer).
    Focused,
    /// Focused, indicator shown (the last input was a key).
    FocusedVisible,
}

/// One ring entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FocusEntry {
    /// The control.
    pub id: Id,
    /// Its scope.
    pub scope: ScopeId,
    /// Registered but never reachable.
    pub disabled: bool,
    /// Its area this frame.
    pub area: Rect,
    /// The layer it was registered from.
    pub layer: LayerId,
    /// Bare `Char` chords are typing for this control (capture skips them).
    pub swallows_typing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ScopeRecord {
    id: ScopeId,
    parent: Option<ScopeId>,
    mode: ScopeMode,
    layer: LayerId,
}

/// The per-frame focus ring.
#[derive(Clone, Debug)]
pub struct FocusRing {
    entries: Vec<FocusEntry>,
    scopes: Vec<ScopeRecord>,
    /// The scope stack while drawing.
    open: Vec<ScopeId>,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusRing {
    /// An empty ring with the root scope.
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

    /// Reset for a new frame, keeping allocations.
    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.scopes.truncate(1);
        self.open.clear();
        self.open.push(ScopeId::ROOT);
    }

    /// Every entry, in registration order, disabled ones included.
    pub fn entries(&self) -> &[FocusEntry] {
        &self.entries
    }

    /// The scope currently open during draw.
    pub(crate) fn current_scope(&self) -> ScopeId {
        self.open.last().copied().unwrap_or(ScopeId::ROOT)
    }

    /// Open a nested scope during draw.
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

    /// Close the innermost open scope.
    pub(crate) fn pop_scope(&mut self) {
        if self.open.len() > 1 {
            self.open.pop();
        }
    }

    /// Arm a scope (a layer's trap) whether or not anything draws into it.
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

    /// Register a control in the current scope.
    pub(crate) fn register(&mut self, entry: FocusEntry) {
        self.entries.push(entry);
    }

    fn scope(&self, id: ScopeId) -> Option<&ScopeRecord> {
        self.scopes.iter().find(|r| r.id == id)
    }

    /// The innermost armed trap: the highest layer, then the latest record.
    pub fn active_trap(&self) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|r| r.mode == ScopeMode::Trap)
            .max_by(|a, b| a.layer.cmp(&b.layer))
            .map(|r| r.id)
    }

    /// The innermost active scope: the highest layer's latest scope.
    pub fn innermost_scope(&self) -> ScopeId {
        self.scopes
            .iter()
            .rev()
            .max_by(|a, b| a.layer.cmp(&b.layer))
            .map_or(ScopeId::ROOT, |r| r.id)
    }

    /// Whether `scope` is `ancestor` or nested inside it.
    pub fn within(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        let mut cur = Some(scope);
        // bounded by the scope count so a malformed parent chain cannot loop
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

    /// Enabled entries inside the innermost trap, in registration order.
    pub fn reachable(&self) -> impl Iterator<Item = &FocusEntry> + '_ {
        let trap = self.active_trap();
        self.entries
            .iter()
            .filter(move |e| self.is_reachable(e, trap))
    }

    /// Whether `id` is reachable.
    pub fn contains(&self, id: Id) -> bool {
        self.reachable().any(|e| e.id == id)
    }

    /// Whether `id` is registered at all (disabled included).
    pub fn is_registered(&self, id: Id) -> bool {
        self.entries.iter().any(|e| e.id == id)
    }

    /// The entry for `id`, if registered.
    pub fn entry(&self, id: Id) -> Option<&FocusEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// The reachable entry after `from`, wrapping; the first when `from` is
    /// `None` or not reachable.
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

    /// The reachable entry before `from`, wrapping; the last when `from` is
    /// `None` or not reachable.
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

    /// The first reachable entry in `scope` or its descendants.
    pub fn first_in(&self, scope: ScopeId) -> Option<Id> {
        self.reachable()
            .find(|e| self.within(e.scope, scope))
            .map(|e| e.id)
    }

    /// The §3.3 step 14 reconciliation rule, applied when `current` is absent
    /// from this ring (or `None`):
    /// (a) the nearest surviving entry of the same scope by previous index,
    /// (b) else that scope's first reachable entry,
    /// (c) else the innermost active scope's first reachable entry,
    /// (d) else `None`.
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
            // (a) nearest surviving entry in the same scope by previous index
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
            // (b) that scope's first enabled entry
            if let Some(id) = self.first_in(pe.scope) {
                return Some(id);
            }
        }
        // (c) the innermost active scope's first enabled entry, then (d) None
        self.first_in(self.innermost_scope())
            .or_else(|| self.reachable().next().map(|e| e.id))
    }
}

/// Runtime-owned focus state.
#[derive(Clone, Debug, Default)]
pub struct FocusState {
    current: Option<Id>,
    visible: bool,
    restore: HashMap<ScopeId, Id>,
}

impl FocusState {
    /// The focused control.
    pub const fn current(&self) -> Option<Id> {
        self.current
    }

    /// Focus-visible: true iff the last input was a key.
    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Focus as seen by `id`.
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

    /// Remember where focus should return when `scope` closes.
    pub(crate) fn save_restore(&mut self, scope: ScopeId, id: Id) {
        self.restore.insert(scope, id);
    }

    /// Take the restore target for `scope`.
    pub(crate) fn take_restore(&mut self, scope: ScopeId) -> Option<Id> {
        self.restore.remove(&scope)
    }

    /// Peek the restore target for `scope`.
    pub fn restore_target(&self, scope: ScopeId) -> Option<Id> {
        self.restore.get(&scope).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &'static str, scope: ScopeId, disabled: bool, layer: LayerId) -> FocusEntry {
        FocusEntry {
            id: Id::root(name),
            scope,
            disabled,
            area: Rect::new(0, 0, 1, 1),
            layer,
            swallows_typing: false,
        }
    }

    fn ring(names: &[&'static str]) -> FocusRing {
        let mut r = FocusRing::new();
        for n in names {
            r.register(entry(n, ScopeId::ROOT, false, LayerId::PAGE));
        }
        r
    }

    #[test]
    fn tab_cycles_forward_and_backward() {
        let r = ring(&["a", "b", "c"]);
        let (a, b, c) = (Id::root("a"), Id::root("b"), Id::root("c"));
        assert_eq!(r.next(None), Some(a));
        assert_eq!(r.next(Some(a)), Some(b));
        assert_eq!(r.next(Some(c)), Some(a));
        assert_eq!(r.prev(None), Some(c));
        assert_eq!(r.prev(Some(a)), Some(c));
        assert_eq!(r.prev(Some(b)), Some(a));
    }

    #[test]
    fn shift_tab_is_the_exact_reverse() {
        let r = ring(&["a", "b", "c", "d"]);
        for e in r.reachable() {
            let here = Some(e.id);
            assert_eq!(r.prev(r.next(here)), here);
            assert_eq!(r.next(r.prev(here)), here);
        }
        // a full forward walk read backwards is a full backward walk
        let mut cur = None;
        let fwd: Vec<Option<Id>> = (0..4)
            .map(|_| {
                cur = r.next(cur);
                cur
            })
            .collect();
        let mut cur = None;
        let mut back: Vec<Option<Id>> = (0..4)
            .map(|_| {
                cur = r.prev(cur);
                cur
            })
            .collect();
        back.reverse();
        assert_eq!(fwd, back);
    }

    #[test]
    fn disabled_entries_are_registered_but_skipped() {
        let mut r = FocusRing::new();
        r.register(entry("a", ScopeId::ROOT, false, LayerId::PAGE));
        r.register(entry("b", ScopeId::ROOT, true, LayerId::PAGE));
        r.register(entry("c", ScopeId::ROOT, false, LayerId::PAGE));
        assert_eq!(r.entries().len(), 3);
        assert!(r.is_registered(Id::root("b")));
        assert!(!r.contains(Id::root("b")));
        assert_eq!(r.next(Some(Id::root("a"))), Some(Id::root("c")));
    }

    #[test]
    fn read_only_entries_stay_in_the_ring() {
        // read-only is `Focusability::FocusableReadOnly`: `disabled: false`
        let mut r = FocusRing::new();
        let mut e = entry("ro", ScopeId::ROOT, false, LayerId::PAGE);
        e.swallows_typing = false;
        r.register(e);
        assert!(r.contains(Id::root("ro")));
    }

    #[test]
    fn click_only_entries_are_never_reachable() {
        // `Focusability::ClickOnly` never registers: the ring has no entry
        let r = ring(&["a"]);
        assert!(!r.is_registered(Id::root("clickonly")));
        assert!(!r.contains(Id::root("clickonly")));
    }

    fn trapped() -> FocusRing {
        let mut r = ring(&["a", "b"]);
        let dlg = ScopeId::new(Id::root("dlg"));
        r.push_scope(dlg, ScopeMode::Trap, LayerId(1));
        r.register(entry("ok", dlg, false, LayerId(1)));
        r.register(entry("cancel", dlg, false, LayerId(1)));
        r.pop_scope();
        r
    }

    #[test]
    fn trap_confines_traversal_to_the_scope() {
        let r = trapped();
        let ids: Vec<Id> = r.reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![Id::root("ok"), Id::root("cancel")]);
        assert!(!r.contains(Id::root("a")));
    }

    #[test]
    fn trap_wraps_inside_the_scope() {
        let r = trapped();
        assert_eq!(r.next(Some(Id::root("cancel"))), Some(Id::root("ok")));
        assert_eq!(r.prev(Some(Id::root("ok"))), Some(Id::root("cancel")));
    }

    #[test]
    fn nested_scopes_resolve_innermost_first() {
        let mut r = trapped();
        let inner = ScopeId::new(Id::root("picker"));
        let dlg = ScopeId::new(Id::root("dlg"));
        r.push_scope(dlg, ScopeMode::Trap, LayerId(1));
        r.push_scope(inner, ScopeMode::Trap, LayerId(2));
        r.register(entry("row", inner, false, LayerId(2)));
        r.pop_scope();
        r.pop_scope();
        assert_eq!(r.active_trap(), Some(inner));
        let ids: Vec<Id> = r.reachable().map(|e| e.id).collect();
        assert_eq!(ids, vec![Id::root("row")]);
        assert!(r.within(inner, dlg));
        assert!(!r.within(dlg, inner));
    }

    #[test]
    fn scope_restore_returns_focus_to_the_opener() {
        let mut f = FocusState::default();
        let dlg = ScopeId::new(Id::root("dlg"));
        f.set(Some(Id::root("open")));
        f.save_restore(dlg, Id::root("open"));
        f.set(Some(Id::root("ok")));
        assert_eq!(f.restore_target(dlg), Some(Id::root("open")));
        assert_eq!(f.take_restore(dlg), Some(Id::root("open")));
        assert_eq!(f.take_restore(dlg), None);
    }

    #[test]
    fn reconcile_prefers_nearest_surviving_entry_by_previous_index() {
        let prev = ring(&["a", "b", "c", "d", "e"]);
        // c vanished: forward neighbour d wins over backward neighbour b
        let now = ring(&["a", "b", "d", "e"]);
        assert_eq!(
            now.reconcile(&prev, Some(Id::root("c"))),
            Some(Id::root("d"))
        );
        // d also gone: b (distance 1 backward) beats e (distance 2 forward)
        let now = ring(&["a", "b", "e"]);
        assert_eq!(
            now.reconcile(&prev, Some(Id::root("c"))),
            Some(Id::root("b"))
        );
        // survivors keep focus
        assert_eq!(
            now.reconcile(&prev, Some(Id::root("a"))),
            Some(Id::root("a"))
        );
    }

    #[test]
    fn reconcile_falls_back_to_scope_first_enabled() {
        let prev = ring(&["x", "y"]);
        // nothing survives from the previous ring; new entries in the same scope
        let now = ring(&["n1", "n2"]);
        assert_eq!(
            now.reconcile(&prev, Some(Id::root("y"))),
            Some(Id::root("n1"))
        );
    }

    #[test]
    fn reconcile_falls_back_to_innermost_active_scope() {
        let prev = ring(&["page"]);
        let mut now = FocusRing::new();
        let dlg = ScopeId::new(Id::root("dlg"));
        now.push_scope(dlg, ScopeMode::Trap, LayerId(1));
        now.register(entry("ok", dlg, false, LayerId(1)));
        now.pop_scope();
        assert_eq!(
            now.reconcile(&prev, Some(Id::root("page"))),
            Some(Id::root("ok"))
        );
        assert_eq!(now.reconcile(&prev, None), Some(Id::root("ok")));
    }

    #[test]
    fn reconcile_yields_none_when_nothing_is_reachable() {
        let prev = ring(&["a"]);
        let mut now = FocusRing::new();
        now.register(entry("a", ScopeId::ROOT, true, LayerId::PAGE));
        assert_eq!(now.reconcile(&prev, Some(Id::root("a"))), None);
        assert_eq!(FocusRing::new().reconcile(&prev, None), None);
    }

    #[test]
    fn focus_visible_is_true_only_after_a_key() {
        let mut f = FocusState::default();
        f.set(Some(Id::root("a")));
        f.set_visible(false);
        assert_eq!(f.vis(Id::root("a")), FocusVis::Focused);
        f.set_visible(true);
        assert_eq!(f.vis(Id::root("a")), FocusVis::FocusedVisible);
        assert_eq!(f.vis(Id::root("b")), FocusVis::None);
    }

    #[test]
    fn trap_is_armed_when_the_layer_is_pushed_not_when_it_draws() {
        // the runtime arms the layer's scope with `ensure_scope` before any
        // draw; a layer that draws nothing still traps
        let mut r = ring(&["a", "b"]);
        r.ensure_scope(
            ScopeId::new(Id::root("modal")),
            ScopeMode::Trap,
            LayerId(1),
            Some(ScopeId::ROOT),
        );
        assert_eq!(r.reachable().count(), 0);
        assert_eq!(r.next(None), None);
    }

    #[test]
    fn restore_target_receives_keys_before_the_next_draw() {
        // FocusState::current is the restore target as soon as a layer closes,
        // even though it is absent from the last ring (§21 item 15)
        let mut f = FocusState::default();
        let dlg = ScopeId::new(Id::root("dlg"));
        f.save_restore(dlg, Id::root("opener"));
        f.set(Some(Id::root("ok")));
        let target = f.take_restore(dlg);
        f.set(target);
        assert_eq!(f.current(), Some(Id::root("opener")));
        let last = trapped();
        assert!(!last.contains(Id::root("opener")));
    }
}
