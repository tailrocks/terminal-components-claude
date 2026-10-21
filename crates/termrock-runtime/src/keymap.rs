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

    #[cfg(test)]
    pub(crate) fn publish<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        layer: crate::layer::LayerId,
        table: &'static [Binding<C>],
    ) -> Option<ActionKey> {
        self.publish_filtered(owner, flags, layer, table, |_| true)
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
        // Reserve enough for the copied base and every possible extension before
        // appending. This keeps the frame-hot path allocation-free and lets the
        // validation pass call `include` exactly once for each binding.
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
    #[cfg(test)]
    pub(crate) fn update<I>(
        &mut self,
        owner: Id,
        base: Option<BindingTableId>,
        base_descriptors: &[BindingDescriptor],
        bindings: I,
    ) -> (&[BindingDescriptor], u64)
    where
        I: Iterator<Item = (ActionKey, Option<Chord>)> + Clone,
    {
        self.update_descriptors(
            owner,
            base,
            base_descriptors,
            bindings.map(|(action, chord)| BindingDescriptor {
                action,
                chord,
                label: "",
                priority: 0,
                visible: false,
            }),
        )
    }

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
    #[cfg(test)]
    fn contains_chord(&self, chord: Chord) -> bool {
        self.layer.hints.iter().any(|hint| {
            hint.key
                .physical_chord()
                .is_some_and(|prior| chord.effective_eq(&prior))
        })
    }

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

impl HintKey {
    #[cfg(test)]
    const fn physical_chord(self) -> Option<Chord> {
        match self {
            Self::Chord(chord) | Self::ChordWithCase { chord, .. } => Some(chord),
            Self::Label(_) => None,
        }
    }
}

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

/// Human-readable chord labels, per client platform.
///
/// Ported from PR #26 `KeyBinding::label_on` (src/core/keymap.rs:298,
/// commit `f4867e9b5`). macOS names Super as `⌘`; everything else says
/// `Super+`. Hint layers can adopt this so a hint never advertises a key
/// the terminal cannot deliver (`termrock_core::keys::keyboard::
/// label_deliverable` answers the delivery half).
pub trait LabelOn {
    /// The label this chord carries under `platform`.
    fn label_on(&self, platform: ClientPlatform) -> String;

    /// The label under the detected client platform.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{KeyCode, KeyModifiers};

    const fn k(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn capture_skips_bare_chars_while_typing() {
        let m = KeyMap::new()
            .bind(
                KeyPhase::Capture,
                Chord::key(KeyCode::Char('q')),
                ActionKey::CLOSE,
            )
            .bind(
                KeyPhase::Capture,
                Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL),
                ActionKey::SAVE,
            );
        assert_eq!(
            m.lookup(KeyPhase::Capture, &k(KeyCode::Char('q')), false),
            Some(ActionKey::CLOSE)
        );
        assert_eq!(
            m.lookup(KeyPhase::Capture, &k(KeyCode::Char('q')), true),
            None
        );
        let ctrl_s = Key {
            code: KeyCode::Char('s'),
            mods: KeyModifiers::CONTROL,
        };
        assert_eq!(
            m.lookup(KeyPhase::Capture, &ctrl_s, true),
            Some(ActionKey::SAVE)
        );
        assert_eq!(m.lookup(KeyPhase::Bubble, &ctrl_s, true), None);
    }

    #[test]
    fn remove_remap_and_conflicts() {
        let mut m = KeyMap::new().bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::F(1)),
            ActionKey::RETRY,
        );
        m.remap(
            KeyPhase::Bubble,
            Chord::key(KeyCode::F(1)),
            Chord::key(KeyCode::F(2)),
        );
        assert_eq!(
            m.lookup(KeyPhase::Bubble, &k(KeyCode::F(2)), false),
            Some(ActionKey::RETRY)
        );
        m.add(
            KeyPhase::Bubble,
            Chord::key(KeyCode::F(2)),
            ActionKey::CLOSE,
        );
        assert_eq!(m.conflicts().len(), 1);
        m.remove(KeyPhase::Bubble, Chord::key(KeyCode::F(2)));
        assert!(m.is_empty());
        assert!(KeyMap::EMPTY_REF.is_empty());
    }

    #[test]
    fn global_keymap_mutations_use_effective_chord_identity() {
        let shifted_back_tab = Chord::with(KeyCode::BackTab, KeyModifiers::SHIFT);
        let bare_back_tab = Chord::key(KeyCode::BackTab);
        let key = Key {
            code: KeyCode::BackTab,
            mods: KeyModifiers::SHIFT,
        };

        let mut removed = KeyMap::new().bind(
            KeyPhase::Bubble,
            bare_back_tab,
            ActionKey::custom("keymap.back-tab"),
        );
        // A true modifier remains distinct from the intrinsic Shift on BackTab.
        removed.remove(
            KeyPhase::Bubble,
            Chord::with(KeyCode::BackTab, KeyModifiers::ALT),
        );
        assert_eq!(
            removed.lookup(KeyPhase::Bubble, &key, false),
            Some(ActionKey::custom("keymap.back-tab"))
        );
        removed.remove(KeyPhase::Bubble, shifted_back_tab);
        assert_eq!(removed.lookup(KeyPhase::Bubble, &key, false), None);

        let action = ActionKey::custom("keymap.back-tab-remap");
        let mut remapped = KeyMap::new().bind(KeyPhase::Bubble, bare_back_tab, action);
        remapped.remap(KeyPhase::Bubble, shifted_back_tab, Chord::key(KeyCode::Up));
        assert_eq!(
            remapped.lookup(KeyPhase::Bubble, &k(KeyCode::Up), false),
            Some(action)
        );
        assert_eq!(remapped.lookup(KeyPhase::Bubble, &key, false), None);

        const OWNER: Id = Id::root("keymap.back-tab-typing");
        let mut typing = KeyMap::new().bind_before_typing(OWNER, bare_back_tab, action);
        assert_eq!(typing.before_typing(OWNER, &key), Some(action));
        typing.remove_before_typing(OWNER, shifted_back_tab);
        assert_eq!(typing.before_typing(OWNER, &key), None);
    }

    #[test]
    fn hints_derive_from_visible_bindings_by_priority() {
        #[derive(Clone, Copy)]
        enum Cmd {
            A,
            B,
        }
        const T: &[Binding<Cmd>] = &[
            Binding {
                action: ActionKey::custom("test.previous"),
                chord: Some(Chord::key(KeyCode::Left)),
                cmd: Cmd::A,
                label: "Prev",
                priority: 40,
                visible: true,
            },
            Binding {
                action: ActionKey::custom("test.select.enter"),
                chord: Some(Chord::key(KeyCode::Enter)),
                cmd: Cmd::B,
                label: "Select",
                priority: 80,
                visible: true,
            },
            Binding {
                action: ActionKey::custom("test.select.space"),
                chord: Some(Chord::key(KeyCode::Char(' '))),
                cmd: Cmd::B,
                label: "Select",
                priority: 80,
                visible: false,
            },
        ];
        let h = HintLayer::from_bindings(T);
        assert_eq!(h.hints.len(), 2);
        assert_eq!(h.hints.first().map(|h| h.label), Some("Select"));
        assert!(matches!(
            Binding::command(T, ActionKey::custom("test.select.space")),
            Some(Cmd::B)
        ));
        assert!(binding_conflicts(Id::root("x"), KeyPhase::Bubble, T).is_empty());
    }

    #[test]
    fn duplicate_binding_action_is_rejected() {
        #[derive(Clone, Copy)]
        enum Cmd {
            A,
        }
        const ACTION: ActionKey = ActionKey::custom("test.duplicate-action");
        const TABLE: &[Binding<Cmd>] = &[
            Binding {
                action: ACTION,
                chord: Some(Chord::key(KeyCode::Left)),
                cmd: Cmd::A,
                label: "Left",
                priority: 1,
                visible: true,
            },
            Binding {
                action: ACTION,
                chord: Some(Chord::key(KeyCode::Right)),
                cmd: Cmd::A,
                label: "Right",
                priority: 1,
                visible: true,
            },
        ];
        let mut registry = BindingRegistry::default();
        assert_eq!(
            registry.publish(
                Id::root("duplicate"),
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                TABLE,
            ),
            Some(ACTION)
        );
        assert!(registry.get(Id::root("duplicate")).is_none());
    }

    #[test]
    fn explicit_extension_combines_different_typed_actions_and_resets_per_frame() {
        #[derive(Clone, Copy)]
        enum FieldCmd {
            Backspace,
            Commit,
        }
        #[derive(Clone, Copy)]
        enum NavCmd {
            Up,
            Enter,
        }
        const BACKSPACE: ActionKey = ActionKey::custom("extension.field.backspace");
        const COMMIT: ActionKey = ActionKey::custom("extension.field.commit");
        const UP: ActionKey = ActionKey::custom("extension.nav.up");
        const ENTER: ActionKey = ActionKey::custom("extension.nav.enter");
        const FIELD: &[Binding<FieldCmd>] = &[
            Binding {
                action: BACKSPACE,
                chord: Some(Chord::key(KeyCode::Backspace)),
                cmd: FieldCmd::Backspace,
                label: "Backspace",
                priority: 80,
                visible: true,
            },
            Binding {
                action: COMMIT,
                chord: Some(Chord::key(KeyCode::Enter)),
                cmd: FieldCmd::Commit,
                label: "Commit",
                priority: 70,
                visible: true,
            },
        ];
        const NAV: &[Binding<NavCmd>] = &[
            Binding {
                action: UP,
                chord: Some(Chord::key(KeyCode::Up)),
                cmd: NavCmd::Up,
                label: "Up",
                priority: 40,
                visible: true,
            },
            Binding {
                action: ENTER,
                chord: Some(Chord::key(KeyCode::Enter)),
                cmd: NavCmd::Enter,
                label: "Choose",
                priority: 60,
                visible: true,
            },
        ];
        let owner = Id::root("extension.owner");
        let mut registry = BindingRegistry::default();
        assert_eq!(
            registry.publish(
                owner,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                FIELD
            ),
            None
        );
        assert_eq!(
            registry.extend(
                owner,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                NAV,
                |command| matches!(command, NavCmd::Up),
            ),
            Ok(())
        );
        let (_, descriptors) = registry.get(owner).expect("extended table");
        assert_eq!(descriptors.len(), 3);
        assert!(
            descriptors
                .iter()
                .any(|binding| binding.action == BACKSPACE)
        );
        assert!(descriptors.iter().any(|binding| binding.action == COMMIT));
        assert!(descriptors.iter().any(|binding| binding.action == UP));
        assert!(!descriptors.iter().any(|binding| binding.action == ENTER));

        let mut keymap = KeyMap::new();
        keymap.remap_component(owner, UP, Chord::key(KeyCode::Backspace));
        assert_eq!(keymap.component_conflicts(owner, descriptors).len(), 1);

        registry.reset();
        assert!(registry.get(owner).is_none());
    }

    #[test]
    fn explicit_extension_rejects_action_and_default_chord_conflicts() {
        #[derive(Clone, Copy)]
        enum Command {
            Base,
            Extra,
        }
        const ACTION: ActionKey = ActionKey::custom("extension.conflict.action");
        const EXTRA: ActionKey = ActionKey::custom("extension.conflict.extra");
        const BASE: &[Binding<Command>] = &[Binding {
            action: ACTION,
            chord: Some(Chord::key(KeyCode::Esc)),
            cmd: Command::Base,
            label: "Cancel",
            priority: 1,
            visible: true,
        }];
        const SAME_ACTION: &[Binding<Command>] = &[Binding {
            action: ACTION,
            chord: Some(Chord::key(KeyCode::Enter)),
            cmd: Command::Extra,
            label: "Extra",
            priority: 1,
            visible: true,
        }];
        const SAME_CHORD: &[Binding<Command>] = &[Binding {
            action: EXTRA,
            chord: Some(Chord::key(KeyCode::Esc)),
            cmd: Command::Extra,
            label: "Extra",
            priority: 1,
            visible: true,
        }];
        let owner = Id::root("extension.conflict");
        let mut registry = BindingRegistry::default();
        registry.publish(
            owner,
            StateFlags::FOCUSED,
            crate::layer::LayerId::PAGE,
            BASE,
        );
        assert_eq!(
            registry.extend(
                owner,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                SAME_ACTION,
                |_| true,
            ),
            Err(BindingExtensionError::DuplicateAction {
                owner,
                action: ACTION
            })
        );
        assert_eq!(
            registry.extend(
                owner,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                SAME_CHORD,
                |_| true,
            ),
            Err(BindingExtensionError::ConflictingChord {
                owner,
                chord: Chord::key(KeyCode::Esc),
            })
        );
        assert_eq!(
            registry.tables.len(),
            1,
            "a rejected extension published a table"
        );
        assert_eq!(
            registry.descriptors.len(),
            1,
            "a rejected extension retained its provisional descriptors"
        );
        assert_eq!(registry.get(owner).expect("base retained").1.len(), 1);
    }

    #[test]
    fn effective_duplicate_chord_including_hidden_is_diagnosed() {
        const VISIBLE: ActionKey = ActionKey::custom("test.visible");
        const HIDDEN: ActionKey = ActionKey::custom("test.hidden");
        let table = [
            BindingDescriptor {
                action: VISIBLE,
                chord: Some(Chord::key(KeyCode::Left)),
                label: "Visible",
                priority: 80,
                visible: true,
            },
            BindingDescriptor {
                action: HIDDEN,
                chord: Some(Chord::key(KeyCode::Right)),
                label: "Hidden",
                priority: 40,
                visible: false,
            },
        ];
        let owner = Id::root("effective-conflict");
        let mut keymap = KeyMap::new();
        keymap.remap_component(owner, HIDDEN, Chord::key(KeyCode::Left));
        assert_eq!(keymap.component_conflicts(owner, &table).len(), 1);
    }

    #[test]
    fn dynamic_publication_is_contiguous_after_an_intervening_owner() {
        #[derive(Clone, Copy)]
        enum Cmd {
            Run,
        }
        const STATIC_ACTION: ActionKey = ActionKey::custom("dynamic.static");
        const DYNAMIC_ACTION: ActionKey = ActionKey::custom("dynamic.item");
        const TABLE: &[Binding<Cmd>] = &[Binding {
            action: STATIC_ACTION,
            chord: Some(Chord::key(KeyCode::Enter)),
            cmd: Cmd::Run,
            label: "Run",
            priority: 1,
            visible: true,
        }];
        let a = Id::root("dynamic.a");
        let b = Id::root("dynamic.b");
        let mut registry = BindingRegistry::default();
        assert_eq!(
            registry.publish(a, StateFlags::FOCUSED, crate::layer::LayerId::PAGE, TABLE),
            None
        );
        assert_eq!(
            registry.publish(b, StateFlags::FOCUSED, crate::layer::LayerId::PAGE, TABLE),
            None
        );
        let dynamic = [BindingDescriptor {
            action: DYNAMIC_ACTION,
            chord: Some(Chord::key(KeyCode::F(4))),
            label: "",
            priority: 0,
            visible: false,
        }];
        assert_eq!(
            registry.publish_dynamic(
                a,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                &dynamic,
                7,
            ),
            None
        );
        let (_, descriptors) = registry.get(a).expect("A table");
        assert_eq!(
            descriptors
                .iter()
                .map(|item| item.action)
                .collect::<Vec<_>>(),
            vec![STATIC_ACTION, DYNAMIC_ACTION]
        );
    }

    #[test]
    fn duplicate_dynamic_action_rejects_the_additive_table() {
        let owner = Id::root("dynamic.duplicate");
        let action = ActionKey::custom("dynamic.duplicate.action");
        let descriptors = [
            BindingDescriptor {
                action,
                chord: Some(Chord::key(KeyCode::F(1))),
                label: "",
                priority: 0,
                visible: false,
            },
            BindingDescriptor {
                action,
                chord: Some(Chord::key(KeyCode::F(2))),
                label: "",
                priority: 0,
                visible: false,
            },
        ];
        let mut registry = BindingRegistry::default();
        assert_eq!(
            registry.publish_dynamic(
                owner,
                StateFlags::FOCUSED,
                crate::layer::LayerId::PAGE,
                &descriptors,
                1,
            ),
            Some(action)
        );
        assert!(registry.get(owner).is_none());
    }

    #[test]
    fn dynamic_snapshot_is_bounded_structural_and_wrap_invalidates_identity() {
        let action = ActionKey::custom("dynamic.action");
        let mut snapshots = DynamicBindingRegistry::default();
        let (_, first) = snapshots.update(
            Id::root("first"),
            None,
            &[],
            core::iter::once((action, Some(Chord::key(KeyCode::F(1))))),
        );
        let (_, unchanged) = snapshots.update(
            Id::root("first"),
            None,
            &[],
            core::iter::once((action, Some(Chord::key(KeyCode::F(1))))),
        );
        assert_eq!(first, unchanged);
        let initial_capacity = snapshots
            .current
            .as_ref()
            .map_or(0, |set| set.descriptors.capacity());
        for index in 0..2_000usize {
            let _ = snapshots.update(
                Id::root("owner").index(index),
                None,
                &[],
                core::iter::once((action, Some(Chord::key(KeyCode::F(2))))),
            );
        }
        assert_eq!(
            snapshots.current.as_ref().map(|set| set.descriptors.len()),
            Some(1)
        );
        assert_eq!(
            snapshots
                .current
                .as_ref()
                .map_or(0, |set| set.descriptors.capacity()),
            initial_capacity
        );
        snapshots.next_revision = u64::MAX - 1;
        let (_, max) = snapshots.update(
            Id::root("wrap"),
            None,
            &[],
            core::iter::once((action, Some(Chord::key(KeyCode::F(3))))),
        );
        let (_, wrapped) = snapshots.update(
            Id::root("wrap"),
            None,
            &[],
            core::iter::once((action, Some(Chord::key(KeyCode::F(4))))),
        );
        assert_eq!(max, u64::MAX);
        assert_eq!(wrapped, 0);
        assert_ne!(
            BindingTableId::dynamic(Id::root("wrap"), 1, max),
            BindingTableId::dynamic(Id::root("wrap"), 1, wrapped)
        );
    }

    #[test]
    fn dynamic_snapshot_invalidates_when_selected_base_descriptors_change() {
        let owner = Id::root("selected-base");
        let dynamic = ActionKey::custom("selected-base.dynamic");
        let first = [BindingDescriptor {
            action: ActionKey::custom("selected-base.first"),
            chord: Some(Chord::key(KeyCode::Up)),
            label: "First",
            priority: 1,
            visible: true,
        }];
        let second = [BindingDescriptor {
            action: ActionKey::custom("selected-base.second"),
            chord: Some(Chord::key(KeyCode::Down)),
            label: "Second",
            priority: 1,
            visible: true,
        }];
        let binding = core::iter::once((dynamic, Some(Chord::key(KeyCode::Char('x')))));
        let mut snapshots = DynamicBindingRegistry::default();
        let (_, first_revision) = snapshots.update(
            owner,
            Some(BindingTableId::Static {
                command_type: TypeId::of::<()>(),
                address: 1,
                len: 2,
            }),
            &first,
            binding.clone(),
        );
        let (_, second_revision) = snapshots.update(
            owner,
            Some(BindingTableId::Static {
                command_type: TypeId::of::<()>(),
                address: 1,
                len: 2,
            }),
            &second,
            binding.clone(),
        );
        let (_, unchanged_revision) = snapshots.update(
            owner,
            Some(BindingTableId::Static {
                command_type: TypeId::of::<()>(),
                address: 1,
                len: 2,
            }),
            &second,
            binding,
        );

        assert_ne!(first_revision, second_revision);
        assert_eq!(second_revision, unchanged_revision);
    }

    #[test]
    fn focused_hints_refresh_when_selected_descriptors_change_under_same_table_id() {
        let owner = Id::root("filtered-hints");
        let key = FocusedHintKey {
            focus: owner,
            flags: StateFlags::FOCUSED,
            layer: crate::layer::LayerId::PAGE,
            table: BindingTableId::Static {
                command_type: TypeId::of::<()>(),
                address: 1,
                len: 2,
            },
            keymap_revision: 0,
        };
        let first = [BindingDescriptor {
            action: ActionKey::custom("filtered.first"),
            chord: Some(Chord::key(KeyCode::Up)),
            label: "First",
            priority: 1,
            visible: true,
        }];
        let second = [BindingDescriptor {
            action: ActionKey::custom("filtered.second"),
            chord: Some(Chord::key(KeyCode::Down)),
            label: "Second",
            priority: 1,
            visible: true,
        }];
        let mut focused = FocusedHints::default();
        focused.derive(key, &first, &KeyMap::new());
        assert_eq!(focused.layer.hints[0].label, "First");
        focused.derive(key, &second, &KeyMap::new());
        assert_eq!(focused.layer.hints[0].label, "Second");
    }

    #[test]
    fn focused_hints_refresh_when_keymap_remaps_a_stable_descriptor() {
        let owner = Id::root("remapped-hints");
        let action = ActionKey::custom("remapped-hints.action");
        let table = [BindingDescriptor {
            action,
            chord: Some(Chord::key(KeyCode::Up)),
            label: "Move",
            priority: 1,
            visible: true,
        }];
        let mut focused = FocusedHints::default();
        let key = |keymap_revision| FocusedHintKey {
            focus: owner,
            flags: StateFlags::FOCUSED,
            layer: crate::layer::LayerId::PAGE,
            table: BindingTableId::Static {
                command_type: TypeId::of::<()>(),
                address: 1,
                len: 1,
            },
            keymap_revision,
        };

        focused.derive(key(0), &table, &KeyMap::new());
        assert_eq!(
            focused.layer.hints[0].key,
            HintKey::Chord(Chord::key(KeyCode::Up))
        );
        let remapped = KeyMap::new().bind_component(owner, action, Chord::key(KeyCode::Down));
        focused.derive(key(1), &table, &remapped);
        assert_eq!(
            focused.layer.hints[0].key,
            HintKey::Chord(Chord::key(KeyCode::Down))
        );
    }
    #[test]
    fn focused_hint_dedup_uses_physical_chord_independent_of_display_case() {
        let chord = Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL);
        for key in [
            HintKey::Chord(chord),
            HintKey::ChordWithCase {
                chord,
                case: ChordCase::UppercaseAscii,
            },
        ] {
            let mut hints = FocusedHints::default();
            hints.layer.hints.push(Hint {
                key,
                label: "Scope",
                priority: 1,
            });
            assert!(hints.contains_chord(chord));
            assert!(!hints.contains_chord(Chord::with(KeyCode::Char('S'), KeyModifiers::CONTROL)));
        }
        let mut hints = FocusedHints::default();
        hints.layer.hints.push(Hint {
            key: HintKey::Label("Ctrl+S"),
            label: "Label",
            priority: 1,
        });
        assert!(!hints.contains_chord(chord));
    }

    #[test]
    fn table_conflicts_use_effective_chord_identity() {
        #[derive(Clone, Copy)]
        enum Cmd {
            A,
            B,
        }
        const fn binding(action: ActionKey, chord: Chord, cmd: Cmd) -> Binding<Cmd> {
            Binding {
                action,
                chord: Some(chord),
                cmd,
                label: "test",
                priority: 1,
                visible: true,
            }
        }
        const FIRST: ActionKey = ActionKey::custom("test.effective.first");
        const SECOND: ActionKey = ActionKey::custom("test.effective.second");
        let owner = Id::root("effective-table");
        for (first, second, conflicts) in [
            (
                Chord::key(KeyCode::Char('A')),
                Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
                1,
            ),
            (
                Chord::key(KeyCode::Char('a')),
                Chord::key(KeyCode::Char('A')),
                0,
            ),
            (
                Chord::key(KeyCode::Up),
                Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
                0,
            ),
            (
                Chord::key(KeyCode::BackTab),
                Chord::with(KeyCode::BackTab, KeyModifiers::SHIFT),
                1,
            ),
            (
                Chord::key(KeyCode::BackTab),
                Chord::with(KeyCode::BackTab, KeyModifiers::ALT),
                0,
            ),
            (
                Chord::key(KeyCode::Char('A')),
                Chord::key(KeyCode::Char('A')),
                1,
            ),
        ] {
            let table = [
                binding(FIRST, first, Cmd::A),
                binding(SECOND, second, Cmd::B),
            ];
            assert_eq!(
                binding_conflicts(owner, KeyPhase::Bubble, &table).len(),
                conflicts,
                "table {first:?} vs {second:?}"
            );
        }
        assert_ne!(
            Chord::key(KeyCode::Char('A')),
            Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
            "structural identity still distinguishes modifier forms"
        );
    }

    #[test]
    fn component_conflicts_use_effective_chord_identity() {
        const VISIBLE: ActionKey = ActionKey::custom("test.effective.visible");
        const HIDDEN: ActionKey = ActionKey::custom("test.effective.hidden");
        let owner = Id::root("effective-component");
        let keymap = KeyMap::new();
        for (first, second, conflicts) in [
            (
                Chord::key(KeyCode::Char('A')),
                Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
                1,
            ),
            (
                Chord::key(KeyCode::Char('a')),
                Chord::key(KeyCode::Char('A')),
                0,
            ),
            (
                Chord::key(KeyCode::Up),
                Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
                0,
            ),
        ] {
            let table = [
                BindingDescriptor {
                    action: VISIBLE,
                    chord: Some(first),
                    label: "Visible",
                    priority: 80,
                    visible: true,
                },
                BindingDescriptor {
                    action: HIDDEN,
                    chord: Some(second),
                    label: "Hidden",
                    priority: 40,
                    visible: false,
                },
            ];
            assert_eq!(
                keymap.component_conflicts(owner, &table).len(),
                conflicts,
                "component {first:?} vs {second:?}"
            );
        }
        let table = [
            BindingDescriptor {
                action: VISIBLE,
                chord: Some(Chord::key(KeyCode::Char('A'))),
                label: "Visible",
                priority: 80,
                visible: true,
            },
            BindingDescriptor {
                action: HIDDEN,
                chord: Some(Chord::key(KeyCode::F(1))),
                label: "Hidden",
                priority: 40,
                visible: false,
            },
        ];
        let mut remapped = KeyMap::new();
        remapped.remap_component(
            owner,
            HIDDEN,
            Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
        );
        assert_eq!(remapped.component_conflicts(owner, &table).len(), 1);
    }

    #[test]
    fn keymap_conflicts_use_effective_chord_identity() {
        let owner = Id::root("effective-scopes");
        for (first, second, conflicts) in [
            (
                Chord::key(KeyCode::Char('A')),
                Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
                1,
            ),
            (
                Chord::key(KeyCode::Char('a')),
                Chord::key(KeyCode::Char('A')),
                0,
            ),
            (
                Chord::key(KeyCode::Up),
                Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
                0,
            ),
        ] {
            let map = KeyMap::new()
                .bind(KeyPhase::Bubble, first, ActionKey::CLOSE)
                .bind(KeyPhase::Bubble, second, ActionKey::SAVE);
            assert_eq!(
                map.conflicts().len(),
                conflicts,
                "ordinary {first:?} vs {second:?}"
            );
            let typed = KeyMap::new()
                .bind_before_typing(owner, first, ActionKey::CLOSE)
                .bind_before_typing(owner, second, ActionKey::SAVE);
            assert_eq!(
                typed.conflicts().len(),
                conflicts,
                "typing {first:?} vs {second:?}"
            );
        }
    }

    #[test]
    fn focused_hints_deduplicate_effective_chords() {
        const FIRST: ActionKey = ActionKey::custom("test.effective.hint.first");
        const SECOND: ActionKey = ActionKey::custom("test.effective.hint.second");
        let owner = Id::root("effective-hints");
        let keymap = KeyMap::new();
        for (first, second, hints) in [
            (
                Chord::key(KeyCode::Char('A')),
                Chord::with(KeyCode::Char('A'), KeyModifiers::SHIFT),
                1,
            ),
            (
                Chord::key(KeyCode::Char('a')),
                Chord::key(KeyCode::Char('A')),
                2,
            ),
            (
                Chord::key(KeyCode::Up),
                Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
                2,
            ),
        ] {
            let table = [
                BindingDescriptor {
                    action: FIRST,
                    chord: Some(first),
                    label: "First",
                    priority: 80,
                    visible: true,
                },
                BindingDescriptor {
                    action: SECOND,
                    chord: Some(second),
                    label: "Second",
                    priority: 40,
                    visible: true,
                },
            ];
            let mut focused = FocusedHints::default();
            focused.derive(
                FocusedHintKey {
                    focus: owner,
                    flags: StateFlags::FOCUSED,
                    layer: crate::layer::LayerId::PAGE,
                    table: BindingTableId::dynamic(owner, table.len(), 1),
                    keymap_revision: 0,
                },
                &table,
                &keymap,
            );
            assert_eq!(
                focused.layer.hints.len(),
                hints,
                "hints {first:?} vs {second:?}"
            );
        }
    }

    #[test]
    fn hidden_bindings_still_shadow_later_visible_hints() {
        const HIDDEN: ActionKey = ActionKey::custom("test.hints.hidden");
        const VISIBLE: ActionKey = ActionKey::custom("test.hints.visible");
        const OTHER: ActionKey = ActionKey::custom("test.hints.other");
        let owner = Id::root("hidden-hint-owner");
        let table = [
            BindingDescriptor {
                action: HIDDEN,
                chord: Some(Chord::key(KeyCode::Tab)),
                label: "Child command",
                priority: 80,
                visible: false,
            },
            BindingDescriptor {
                action: VISIBLE,
                chord: Some(Chord::key(KeyCode::Tab)),
                label: "Show details",
                priority: 40,
                visible: true,
            },
            BindingDescriptor {
                action: OTHER,
                chord: Some(Chord::key(KeyCode::Esc)),
                label: "Back",
                priority: 40,
                visible: true,
            },
        ];
        let mut focused = FocusedHints::default();
        focused.derive(
            FocusedHintKey {
                focus: owner,
                flags: StateFlags::FOCUSED,
                layer: crate::layer::LayerId::PAGE,
                table: BindingTableId::dynamic(owner, table.len(), 1),
                keymap_revision: 0,
            },
            &table,
            &KeyMap::new(),
        );

        assert_eq!(focused.layer.hints.len(), 1);
        assert_eq!(focused.layer.hints[0].label, "Back");
    }
}

#[cfg(test)]
mod label_on_tests {
    use super::*;

    fn chord(code: KeyCode, mods: KeyModifiers) -> Chord {
        Chord { code, mods }
    }

    #[test]
    fn labels_follow_the_pr26_rules() {
        assert_eq!(
            chord(KeyCode::Char('s'), KeyModifiers::CONTROL).label_on(ClientPlatform::Other),
            "Ctrl+S"
        );
        assert_eq!(
            chord(KeyCode::Delete, KeyModifiers::NONE).label_on(ClientPlatform::Other),
            "Delete"
        );
        assert_eq!(
            chord(KeyCode::Char(' '), KeyModifiers::NONE).label_on(ClientPlatform::Other),
            "Space"
        );
        assert_eq!(
            chord(KeyCode::BackTab, KeyModifiers::SHIFT).label_on(ClientPlatform::Other),
            "Shift+Tab",
            "BackTab is named once"
        );
        assert_eq!(
            chord(KeyCode::Left, KeyModifiers::NONE).label_on(ClientPlatform::Other),
            "←"
        );
        assert_eq!(
            chord(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )
            .label_on(ClientPlatform::Other),
            "Ctrl+Shift+A"
        );
        assert_eq!(
            chord(KeyCode::F(2), KeyModifiers::NONE).label_on(ClientPlatform::Other),
            "F2"
        );
    }

    #[test]
    fn super_labels_per_platform() {
        let cmd_a = chord(KeyCode::Char('a'), KeyModifiers::SUPER);
        assert_eq!(cmd_a.label_on(ClientPlatform::MacOs), "⌘A");
        assert_eq!(cmd_a.label_on(ClientPlatform::Other), "Super+A");
        let shift_cmd = chord(
            KeyCode::Char('a'),
            KeyModifiers::SUPER | KeyModifiers::SHIFT,
        );
        assert_eq!(shift_cmd.label_on(ClientPlatform::MacOs), "⇧⌘A");
        assert_eq!(shift_cmd.label_on(ClientPlatform::Other), "Super+Shift+A");
        let alt_cmd = chord(KeyCode::Char('a'), KeyModifiers::SUPER | KeyModifiers::ALT);
        assert_eq!(
            alt_cmd.label_on(ClientPlatform::MacOs),
            "⌘Alt+A",
            "only Super is named with the symbol; Alt stays textual"
        );
    }

    #[test]
    fn shift_is_part_of_the_char_it_names() {
        assert_eq!(
            chord(KeyCode::Char('a'), KeyModifiers::SHIFT).label_on(ClientPlatform::Other),
            "a",
            "Shift+A carries the shifted character, no Shift+ prefix"
        );
        assert_eq!(
            chord(KeyCode::Up, KeyModifiers::SHIFT).label_on(ClientPlatform::Other),
            "Shift+↑"
        );
    }
}
