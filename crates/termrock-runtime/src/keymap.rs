use std::any::TypeId;
use std::borrow::Cow;

use crate::action::ActionKey;
use crate::diagnostics::Diagnostic;
use crate::event::KeyModifiers;
use crate::event::{Chord, Key};
use crate::focus::ScopeId;
use crate::id::Id;
use crate::response::StateFlags;

pub use termrock_core::keys::KeyCode;
pub use termrock_core::keys::KeyPhase;
pub use termrock_core::platform::ClientPlatform;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BindingState {
    pub flags: StateFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding<C: 'static> {
    pub action: ActionKey,

    pub chord: Option<Chord>,

    pub cmd: C,

    pub label: &'static str,

    pub priority: u8,

    pub visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingExtensionError {
    MissingBase { owner: Id },
    DuplicateAction { owner: Id, action: ActionKey },
    ConflictingChord { owner: Id, chord: Chord },
}

impl<C: Copy + 'static> Binding<C> {
    pub fn command(table: &[Binding<C>], action: ActionKey) -> Option<C> {
        table.iter().find(|b| b.action == action).map(|b| b.cmd)
    }
}

pub trait Bindings {
    type Cmd: Copy + 'static;

    fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>];
}

pub fn binding_conflicts<C: Copy + 'static>(
    owner: Id,
    phase: KeyPhase,
    table: &[Binding<C>],
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (i, a) in table.iter().enumerate() {
        let Some(chord) = a.chord else {
            continue;
        };
        let dup = table
            .iter()
            .skip(i.saturating_add(1))
            .any(|b| b.chord.is_some_and(|other| chord.effective_eq(&other)));
        if dup {
            out.push(Diagnostic::BindingConflict {
                chord,
                phase,
                a: owner,
                b: owner,
            });
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    phase: KeyPhase,
    chord: Chord,
    key: ActionKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TypingCapture {
    owner: Id,
    chord: Chord,
    key: ActionKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ComponentEntry {
    owner: Id,
    action: ActionKey,
    chord: Option<Chord>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyMap {
    entries: Vec<Entry>,
    components: Vec<ComponentEntry>,
    typing: Vec<TypingCapture>,
}

impl KeyMap {
    pub const EMPTY: KeyMap = KeyMap {
        entries: Vec::new(),
        components: Vec::new(),
        typing: Vec::new(),
    };

    pub const EMPTY_REF: &'static KeyMap = &KeyMap::EMPTY;

    pub const OWNER: Id = Id::root("tui.keymap");

    pub const fn new() -> Self {
        Self::EMPTY
    }

    #[must_use]
    pub fn bind(mut self, phase: KeyPhase, chord: Chord, key: ActionKey) -> Self {
        self.add(phase, chord, key);
        self
    }

    pub fn add(&mut self, phase: KeyPhase, chord: Chord, key: ActionKey) {
        self.entries.push(Entry { phase, chord, key });
    }

    #[must_use]
    pub fn bind_before_typing(mut self, owner: Id, chord: Chord, command: ActionKey) -> Self {
        self.add_before_typing(owner, chord, command);
        self
    }

    pub fn add_before_typing(&mut self, owner: Id, chord: Chord, command: ActionKey) {
        self.typing.push(TypingCapture {
            owner,
            chord,
            key: command,
        });
    }

    pub fn remove_before_typing(&mut self, owner: Id, chord: Chord) {
        self.typing
            .retain(|entry| entry.owner != owner || !entry.chord.effective_eq(&chord));
    }

    pub(crate) fn before_typing(&self, owner: Id, key: &Key) -> Option<ActionKey> {
        self.typing
            .iter()
            .find(|entry| entry.owner == owner && entry.chord.matches(key))
            .map(|entry| entry.key)
    }

    pub fn remove(&mut self, phase: KeyPhase, chord: Chord) {
        self.entries
            .retain(|e| !(e.phase == phase && e.chord.effective_eq(&chord)));
    }

    pub fn remap(&mut self, phase: KeyPhase, from: Chord, to: Chord) {
        for e in &mut self.entries {
            if e.phase == phase && e.chord.effective_eq(&from) {
                e.chord = to;
            }
        }
    }

    #[must_use]
    pub fn bind_component(mut self, owner: Id, action: ActionKey, chord: Chord) -> Self {
        self.remap_component(owner, action, chord);
        self
    }

    pub fn remap_component(&mut self, owner: Id, action: ActionKey, chord: Chord) {
        self.set_component(owner, action, Some(chord));
    }

    pub fn remove_component(&mut self, owner: Id, action: ActionKey) {
        self.set_component(owner, action, None);
    }

    fn set_component(&mut self, owner: Id, action: ActionKey, chord: Option<Chord>) {
        if let Some(entry) = self
            .components
            .iter_mut()
            .find(|entry| entry.owner == owner && entry.action == action)
        {
            entry.chord = chord;
        } else {
            self.components.push(ComponentEntry {
                owner,
                action,
                chord,
            });
        }
    }

    pub(crate) fn component_chord(
        &self,
        owner: Id,
        action: ActionKey,
        default: Option<Chord>,
    ) -> Option<Chord> {
        self.components
            .iter()
            .rev()
            .find(|entry| entry.owner == owner && entry.action == action)
            .map_or(default, |entry| entry.chord)
    }

    pub(crate) fn component_binding(
        &self,
        owner: Id,
        table: &[BindingDescriptor],
        key: &Key,
    ) -> Option<(ActionKey, Chord)> {
        table.iter().find_map(|binding| {
            self.component_chord(owner, binding.action, binding.chord)
                .filter(|chord| chord.matches(key))
                .map(|chord| (binding.action, chord))
        })
    }

    pub(crate) fn component_conflicts(
        &self,
        owner: Id,
        table: &[BindingDescriptor],
    ) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (index, binding) in table.iter().enumerate() {
            let Some(chord) = self.component_chord(owner, binding.action, binding.chord) else {
                continue;
            };
            let duplicate = table.iter().skip(index.saturating_add(1)).any(|other| {
                self.component_chord(owner, other.action, other.chord)
                    .is_some_and(|candidate| chord.effective_eq(&candidate))
            });
            if duplicate {
                out.push(Diagnostic::BindingConflict {
                    chord,
                    phase: KeyPhase::Bubble,
                    a: owner,
                    b: owner,
                });
            }
        }
        out
    }

    pub fn lookup(&self, phase: KeyPhase, key: &Key, swallows_typing: bool) -> Option<ActionKey> {
        self.entries
            .iter()
            .filter(|e| e.phase == phase)
            .filter(|e| !(phase == KeyPhase::Capture && swallows_typing && e.chord.is_bare_char()))
            .find(|e| e.chord.matches(key))
            .map(|e| e.key)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.components.is_empty() && self.typing.is_empty()
    }

    pub fn conflicts(&self) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (i, a) in self.entries.iter().enumerate() {
            let dup = self
                .entries
                .iter()
                .skip(i.saturating_add(1))
                .any(|b| b.phase == a.phase && b.chord.effective_eq(&a.chord));
            if dup {
                out.push(Diagnostic::BindingConflict {
                    chord: a.chord,
                    phase: a.phase,
                    a: KeyMap::OWNER,
                    b: KeyMap::OWNER,
                });
            }
        }
        for (index, entry) in self.typing.iter().enumerate() {
            if self
                .typing
                .iter()
                .skip(index.saturating_add(1))
                .any(|other| other.owner == entry.owner && other.chord.effective_eq(&entry.chord))
            {
                out.push(Diagnostic::BindingConflict {
                    chord: entry.chord,
                    phase: KeyPhase::Capture,
                    a: entry.owner,
                    b: entry.owner,
                });
            }
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingTableId {
    Static {
        command_type: TypeId,

        address: usize,

        len: usize,
    },

    Dynamic {
        revision: u64,

        len: usize,
    },
}

impl BindingTableId {
    fn of<C: 'static>(table: &'static [Binding<C>]) -> Self {
        BindingTableId::Static {
            command_type: TypeId::of::<C>(),
            address: table.as_ptr() as usize,
            len: table.len(),
        }
    }

    fn dynamic(owner: Id, len: usize, revision: u64) -> Self {
        let _ = owner;
        BindingTableId::Dynamic { revision, len }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindingDescriptor {
    pub(crate) action: ActionKey,
    pub(crate) chord: Option<Chord>,
    pub(crate) label: &'static str,
    pub(crate) priority: u8,
    pub(crate) visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PublishedBindings {
    pub(crate) owner: Id,
    pub(crate) flags: StateFlags,
    pub(crate) layer: crate::layer::LayerId,
    pub(crate) table: BindingTableId,
    start: usize,
    len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopedBubbleBinding {
    pub(crate) scope: ScopeId,
    pub(crate) owner: Id,
    pub(crate) layer: crate::layer::LayerId,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingRegistry {
    tables: Vec<PublishedBindings>,
    descriptors: Vec<BindingDescriptor>,
    composite_revision: u64,
}

impl BindingRegistry {
    pub(crate) fn reset(&mut self) {
        self.tables.clear();
        self.descriptors.clear();
    }

    pub(crate) fn extend<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        layer: crate::layer::LayerId,
        table: &'static [Binding<C>],
        include: impl Fn(C) -> bool,
    ) -> Result<(), BindingExtensionError> {
        let prior_index = self
            .tables
            .iter()
            .rposition(|published| published.owner == owner);
        let Some(prior_index) = prior_index else {
            return Err(BindingExtensionError::MissingBase { owner });
        };
        let prior = self.tables[prior_index];
        let prior_end = prior.start.saturating_add(prior.len);
        let prior_range = self
            .descriptors
            .get(prior.start..prior_end)
            .map(|_| prior.start..prior_end);
        let prior_len = prior_range.as_ref().map_or(0, |range| range.len());
        let start = self.descriptors.len();

        self.descriptors
            .reserve(prior_len.saturating_add(table.len()));
        self.tables.reserve(1);
        let copied_start = self.descriptors.len();
        if let Some(range) = prior_range {
            self.descriptors.extend_from_within(range);
        }
        let additions_start = self.descriptors.len();
        for binding in table {
            if !include(binding.cmd) {
                continue;
            }
            let descriptor = BindingDescriptor {
                action: binding.action,
                chord: binding.chord,
                label: binding.label,
                priority: binding.priority,
                visible: binding.visible,
            };
            let prior = &self.descriptors[copied_start..additions_start];
            let additions = &self.descriptors[additions_start..];
            if prior
                .iter()
                .chain(additions)
                .any(|item| item.action == binding.action)
            {
                self.descriptors.truncate(start);
                return Err(BindingExtensionError::DuplicateAction {
                    owner,
                    action: binding.action,
                });
            }
            if let Some(chord) = descriptor.chord
                && prior
                    .iter()
                    .chain(additions)
                    .any(|item| item.chord.is_some_and(|other| chord.effective_eq(&other)))
            {
                self.descriptors.truncate(start);
                return Err(BindingExtensionError::ConflictingChord { owner, chord });
            }
            self.descriptors.push(descriptor);
        }
        let len = self.descriptors.len().saturating_sub(start);
        self.composite_revision = self.composite_revision.wrapping_add(1);
        self.tables.push(PublishedBindings {
            owner,
            flags,
            layer,
            table: BindingTableId::dynamic(owner, len, self.composite_revision),
            start,
            len,
        });
        Ok(())
    }

    pub(crate) fn publish_filtered<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        layer: crate::layer::LayerId,
        table: &'static [Binding<C>],
        include: impl Fn(C) -> bool,
    ) -> Option<ActionKey> {
        let start = self.descriptors.len();
        for binding in table.iter().filter(|binding| include(binding.cmd)) {
            if self
                .descriptors
                .get(start..)
                .is_some_and(|prior| prior.iter().any(|item| item.action == binding.action))
            {
                self.descriptors.truncate(start);
                return Some(binding.action);
            }
            self.descriptors.push(BindingDescriptor {
                action: binding.action,
                chord: binding.chord,
                label: binding.label,
                priority: binding.priority,
                visible: binding.visible,
            });
        }
        self.tables.push(PublishedBindings {
            owner,
            flags,
            layer,
            table: BindingTableId::of(table),
            start,
            len: self.descriptors.len().saturating_sub(start),
        });
        None
    }

    pub(crate) fn get(&self, owner: Id) -> Option<(PublishedBindings, &[BindingDescriptor])> {
        let published = self
            .tables
            .iter()
            .rev()
            .find(|table| table.owner == owner)?;
        let end = published.start.saturating_add(published.len);
        Some((*published, self.descriptors.get(published.start..end)?))
    }

    pub(crate) fn publish_dynamic(
        &mut self,
        owner: Id,
        flags: StateFlags,
        layer: crate::layer::LayerId,
        dynamic: &[BindingDescriptor],
        revision: u64,
    ) -> Option<ActionKey> {
        let existing = self.tables.iter().rposition(|table| table.owner == owner);
        let (prior_start, prior_len) = existing
            .and_then(|index| self.tables.get(index))
            .map_or((0, 0), |table| (table.start, table.len));
        let prior_end = prior_start.saturating_add(prior_len);
        for (index, binding) in dynamic.iter().enumerate() {
            if self
                .descriptors
                .get(prior_start..prior_end)
                .unwrap_or(&[])
                .iter()
                .any(|item| item.action == binding.action)
                || dynamic
                    .iter()
                    .skip(index.saturating_add(1))
                    .any(|item| item.action == binding.action)
            {
                return Some(binding.action);
            }
        }
        let contiguous = existing.is_some_and(|index| {
            self.tables.get(index).is_some_and(|table| {
                index == self.tables.len().saturating_sub(1)
                    && table.start.saturating_add(table.len) == self.descriptors.len()
            })
        });
        if contiguous {
            self.descriptors.extend_from_slice(dynamic);
            if let Some(table) = existing.and_then(|index| self.tables.get_mut(index)) {
                table.flags = flags;
                table.layer = layer;
                table.len = table.len.saturating_add(dynamic.len());
                table.table = BindingTableId::dynamic(owner, table.len, revision);
            }
        } else {
            let start = self.descriptors.len();
            self.descriptors
                .reserve(prior_len.saturating_add(dynamic.len()));
            for index in prior_start..prior_end {
                if let Some(descriptor) = self.descriptors.get(index).copied() {
                    self.descriptors.push(descriptor);
                }
            }
            self.descriptors.extend_from_slice(dynamic);
            let len = prior_len.saturating_add(dynamic.len());
            self.tables.push(PublishedBindings {
                owner,
                flags,
                layer,
                table: BindingTableId::dynamic(owner, len, revision),
                start,
                len,
            });
        }
        None
    }
}

#[derive(Debug, Default)]
pub(crate) struct DynamicBindingRegistry {
    current: Option<DynamicBindingSet>,
    next_revision: u64,
}

#[derive(Debug)]
struct DynamicBindingSet {
    owner: Id,
    base: Option<BindingTableId>,
    base_descriptors: Vec<BindingDescriptor>,
    descriptors: Vec<BindingDescriptor>,
    revision: u64,
}

impl DynamicBindingRegistry {
    pub(crate) fn update_descriptors<I>(
        &mut self,
        owner: Id,
        base: Option<BindingTableId>,
        base_descriptors: &[BindingDescriptor],
        bindings: I,
    ) -> (&[BindingDescriptor], u64)
    where
        I: Iterator<Item = BindingDescriptor> + Clone,
    {
        let mut owner_changed = false;
        if let Some(set) = self.current.as_mut() {
            if set.owner != owner {
                owner_changed = true;
                set.owner = owner;
                set.base = None;
                set.descriptors.clear();
            }
        } else {
            self.current = Some(DynamicBindingSet {
                owner,
                base: None,
                base_descriptors: Vec::new(),
                descriptors: Vec::new(),
                revision: 0,
            });
        }
        let Some(set) = self.current.as_mut() else {
            return (&[], 0);
        };
        let unchanged = !owner_changed
            && set.base == base
            && set.base_descriptors == base_descriptors
            && set
                .descriptors
                .iter()
                .zip(bindings.clone())
                .all(|(descriptor, next)| descriptor == &next)
            && set.descriptors.len() == bindings.clone().count();
        if !unchanged {
            set.base = base;
            set.base_descriptors.clear();
            set.base_descriptors.extend_from_slice(base_descriptors);
            set.descriptors.clear();
            set.descriptors.extend(bindings);
            self.next_revision = self.next_revision.wrapping_add(1);
            set.revision = self.next_revision;
        }
        (&set.descriptors, set.revision)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FocusedHintKey {
    pub(crate) focus: Id,
    pub(crate) flags: StateFlags,
    pub(crate) layer: crate::layer::LayerId,
    pub(crate) table: BindingTableId,
    pub(crate) keymap_revision: u64,
}

#[derive(Debug, Default)]
pub(crate) struct FocusedHints {
    key: Option<FocusedHintKey>,
    pub(crate) layer: HintLayer,
    descriptors: Vec<BindingDescriptor>,
}

impl FocusedHints {
    pub(crate) fn invalidate(&mut self) {
        self.key = None;
    }

    pub(crate) fn derive(
        &mut self,
        key: FocusedHintKey,
        table: &[BindingDescriptor],
        keymap: &KeyMap,
    ) {
        if self.key == Some(key) && self.descriptors == table {
            return;
        }
        self.key = Some(key);
        self.descriptors.clear();
        self.descriptors.extend_from_slice(table);
        self.layer.hints.clear();
        self.layer.badge = None;
        self.layer.status = None;
        self.layer.centered = false;
        for (index, binding) in table.iter().enumerate() {
            let Some(chord) = keymap.component_chord(key.focus, binding.action, binding.chord)
            else {
                continue;
            };
            let shadowed = table[..index]
                .iter()
                .filter_map(|prior| keymap.component_chord(key.focus, prior.action, prior.chord))
                .any(|prior| chord.effective_eq(&prior));
            if !binding.visible || shadowed {
                continue;
            }
            let hint = Hint {
                key: HintKey::Chord(chord),
                label: binding.label,
                priority: binding.priority,
            };
            let position = self
                .layer
                .hints
                .iter()
                .position(|prior| prior.priority < hint.priority)
                .unwrap_or(self.layer.hints.len());
            self.layer.hints.insert(position, hint);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChordCase {
    #[default]
    Preserve,

    UppercaseAscii,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKey {
    Chord(Chord),

    ChordWithCase { chord: Chord, case: ChordCase },

    Label(&'static str),
}

impl HintKey {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    pub key: HintKey,

    pub label: &'static str,

    pub priority: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct HintLayer {
    pub hints: Vec<Hint>,

    pub badge: Option<&'static str>,

    pub status: Option<Cow<'static, str>>,

    pub centered: bool,
}

impl HintLayer {
    pub const fn empty() -> Self {
        HintLayer {
            hints: Vec::new(),
            badge: None,
            status: None,
            centered: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
            && self.badge.is_none()
            && self.status.as_deref().is_none_or(str::is_empty)
    }

    pub fn from_bindings<C: Copy + 'static>(table: &[Binding<C>]) -> Self {
        let mut hints: Vec<Hint> = table
            .iter()
            .filter(|b| b.visible)
            .filter_map(|b| b.chord.map(|chord| (b, chord)))
            .map(|(b, chord)| Hint {
                key: HintKey::Chord(chord),
                label: b.label,
                priority: b.priority,
            })
            .collect();
        hints.sort_by_key(|h| core::cmp::Reverse(h.priority));
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: false,
        }
    }
}

/// Ported from PR #26 `KeyBinding::label_on` (src/core/keymap.rs:298,
/// commit `f4867e9b5`).
pub trait LabelOn {
    fn label_on(&self, platform: ClientPlatform) -> String;

    fn label(&self) -> String;
}

impl LabelOn for Chord {
    fn label_on(&self, platform: ClientPlatform) -> String {
        let mut out = String::new();
        let char_shift =
            self.mods.contains(KeyModifiers::SHIFT) && matches!(self.code, KeyCode::Char(_));
        if self.mods.contains(KeyModifiers::SUPER) {
            if platform.is_macos() {
                out.push_str(if self.mods.contains(KeyModifiers::SHIFT) {
                    "⇧⌘"
                } else {
                    "⌘"
                });
            } else {
                out.push_str("Super+");
            }
        }
        if self.mods.contains(KeyModifiers::CONTROL) {
            out.push_str("Ctrl+");
        }
        if self.mods.contains(KeyModifiers::ALT) {
            out.push_str("Alt+");
        }
        let ctrl_or_alt = self
            .mods
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER);
        let mac_cmd = self.mods.contains(KeyModifiers::SUPER) && platform.is_macos();

        let back_tab = self.code == KeyCode::BackTab;
        if self.mods.contains(KeyModifiers::SHIFT)
            && !mac_cmd
            && !back_tab
            && (!char_shift || ctrl_or_alt)
        {
            out.push_str("Shift+");
        }
        match self.code {
            KeyCode::Char(' ') => out.push_str("Space"),
            KeyCode::Char(c) if ctrl_or_alt => out.extend(c.to_uppercase()),
            KeyCode::Char(c) => out.push(c),
            KeyCode::Enter => out.push_str("Enter"),
            KeyCode::Esc => out.push_str("Esc"),
            KeyCode::Tab => out.push_str("Tab"),
            KeyCode::BackTab => out.push_str("Shift+Tab"),
            KeyCode::Backspace => out.push_str("Backspace"),
            KeyCode::Delete => out.push_str("Delete"),
            KeyCode::Insert => out.push_str("Ins"),
            KeyCode::Up => out.push('↑'),
            KeyCode::Down => out.push('↓'),
            KeyCode::Left => out.push('←'),
            KeyCode::Right => out.push('→'),
            KeyCode::Home => out.push_str("Home"),
            KeyCode::End => out.push_str("End"),
            KeyCode::PageUp => out.push_str("PgUp"),
            KeyCode::PageDown => out.push_str("PgDn"),
            KeyCode::F(n) => out.push_str(&format!("F{n}")),
            other => out.push_str(&format!("{other:?}")),
        }
        out
    }

    fn label(&self) -> String {
        self.label_on(ClientPlatform::current())
    }
}
