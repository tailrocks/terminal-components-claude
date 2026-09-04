//! Keymaps, bindings and hints (`COMPONENT_ARCHITECTURE.md` §13.1, §21 item 10).
//!
//! Components declare bindings from small `const` tables selected by state;
//! an application overrides and adds product chords through [`KeyMap`] in
//! two explicit phases. Hints are derived from the same tables.

use std::any::TypeId;
use std::borrow::Cow;

use crate::action::ActionKey;
use crate::diagnostics::Diagnostic;
use crate::event::{Chord, Key};
use crate::id::Id;
use crate::response::StateFlags;

/// The phase in which an application chord is matched (§3.3 steps 2 and 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum KeyPhase {
    /// Before dispatch; a bare `Char` chord is skipped while the focused
    /// control swallows typing.
    Capture,
    /// After dispatch, for keys no component consumed.
    Bubble,
}

/// The state a binding table is selected for. `Copy`, so a table is chosen
/// by `match` in a `const fn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BindingState {
    /// The live flags.
    pub flags: StateFlags,
}

/// One chord → command binding in a component's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding<C: 'static> {
    /// Stable identity used by component-scoped overrides and dispatch.
    pub action: ActionKey,
    /// The default chord. `None` declares a latent action that can be bound by
    /// the application's [`KeyMap`].
    pub chord: Option<Chord>,
    /// The const-constructible command; `update` maps it to the emitted
    /// action with the live key.
    pub cmd: C,
    /// The hint label.
    pub label: &'static str,
    /// Hint priority; higher survives width pressure longer.
    pub priority: u8,
    /// Whether the hint bar shows it.
    pub visible: bool,
}

impl<C: Copy + 'static> Binding<C> {
    /// Find the command identified by `action` in `table`.
    pub fn command(table: &[Binding<C>], action: ActionKey) -> Option<C> {
        table.iter().find(|b| b.action == action).map(|b| b.cmd)
    }
}

/// A component's binding tables, one per state.
pub trait Bindings {
    /// The command type (`Next`, `Prev`, `Activate`, …).
    type Cmd: Copy + 'static;

    /// The table for `st`.
    fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>];
}

/// Bindings on the same effective chord within one table are a conflict.
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
            .any(|b| b.chord == Some(chord));
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
struct ComponentEntry {
    owner: Id,
    action: ActionKey,
    chord: Option<Chord>,
}

/// The application's chord layer: add, remove and remap per phase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyMap {
    entries: Vec<Entry>,
    components: Vec<ComponentEntry>,
}

impl KeyMap {
    /// The empty map.
    pub const EMPTY: KeyMap = KeyMap {
        entries: Vec::new(),
        components: Vec::new(),
    };

    /// A `'static` reference to the empty map, for `App::keymap` defaults.
    pub const EMPTY_REF: &'static KeyMap = &KeyMap::EMPTY;

    /// The id reported as the owner of a key map binding in diagnostics.
    pub const OWNER: Id = Id::root("tui.keymap");

    /// An empty, growable map.
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Bind `chord` to `key` in `phase`.
    #[must_use]
    pub fn bind(mut self, phase: KeyPhase, chord: Chord, key: ActionKey) -> Self {
        self.add(phase, chord, key);
        self
    }

    /// Bind in place.
    pub fn add(&mut self, phase: KeyPhase, chord: Chord, key: ActionKey) {
        self.entries.push(Entry { phase, chord, key });
    }

    /// Remove every binding of `chord` in `phase`.
    pub fn remove(&mut self, phase: KeyPhase, chord: Chord) {
        self.entries
            .retain(|e| !(e.phase == phase && e.chord == chord));
    }

    /// Move every binding of `from` in `phase` onto `to`.
    pub fn remap(&mut self, phase: KeyPhase, from: Chord, to: Chord) {
        for e in &mut self.entries {
            if e.phase == phase && e.chord == from {
                e.chord = to;
            }
        }
    }

    /// Bind a component action, including an action whose default chord is
    /// latent.
    #[must_use]
    pub fn bind_component(mut self, owner: Id, action: ActionKey, chord: Chord) -> Self {
        self.remap_component(owner, action, chord);
        self
    }

    /// Replace the effective chord for one component action.
    pub fn remap_component(&mut self, owner: Id, action: ActionKey, chord: Chord) {
        self.set_component(owner, action, Some(chord));
    }

    /// Suppress one component action's default chord and hint.
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
            let duplicate = table
                .iter()
                .skip(index.saturating_add(1))
                .any(|other| self.component_chord(owner, other.action, other.chord) == Some(chord));
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

    /// The action bound to `key` in `phase`, if any. In the capture phase a
    /// bare `Char` chord is skipped while `swallows_typing` (§3.3 step 2).
    pub fn lookup(&self, phase: KeyPhase, key: &Key, swallows_typing: bool) -> Option<ActionKey> {
        self.entries
            .iter()
            .filter(|e| e.phase == phase)
            .filter(|e| !(phase == KeyPhase::Capture && swallows_typing && e.chord.is_bare_char()))
            .find(|e| e.chord.matches(key))
            .map(|e| e.key)
    }

    /// Whether the map has no bindings.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.components.is_empty()
    }

    /// Two bindings of the same chord in the same phase.
    pub fn conflicts(&self) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (i, a) in self.entries.iter().enumerate() {
            let dup = self
                .entries
                .iter()
                .skip(i.saturating_add(1))
                .any(|b| b.phase == a.phase && b.chord == a.chord);
            if dup {
                out.push(Diagnostic::BindingConflict {
                    chord: a.chord,
                    phase: a.phase,
                    a: KeyMap::OWNER,
                    b: KeyMap::OWNER,
                });
            }
        }
        out
    }
}

/// Stable identity of one monomorphic component binding table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingTableId {
    /// A monomorphic static component table.
    Static {
        /// Concrete command type.
        command_type: TypeId,
        /// Static slice address.
        address: usize,
        /// Static descriptor count.
        len: usize,
    },
    /// A structurally compared dynamic descriptor snapshot.
    Dynamic {
        /// Monotonic structural revision.
        revision: u64,
        /// Combined static and dynamic descriptor count.
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

/// Reusable frame publication of the focused component's erased bindings.
#[derive(Clone, Debug, Default)]
pub(crate) struct BindingRegistry {
    tables: Vec<PublishedBindings>,
    descriptors: Vec<BindingDescriptor>,
}

impl BindingRegistry {
    pub(crate) fn reset(&mut self) {
        self.tables.clear();
        self.descriptors.clear();
    }

    pub(crate) fn publish<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        layer: crate::layer::LayerId,
        table: &'static [Binding<C>],
    ) -> Option<ActionKey> {
        let start = self.descriptors.len();
        for binding in table {
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
            len: table.len(),
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
    descriptors: Vec<BindingDescriptor>,
    revision: u64,
}

impl DynamicBindingRegistry {
    pub(crate) fn update<I>(
        &mut self,
        owner: Id,
        base: Option<BindingTableId>,
        bindings: I,
    ) -> (&[BindingDescriptor], u64)
    where
        I: Iterator<Item = (ActionKey, Option<Chord>)> + Clone,
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
                descriptors: Vec::new(),
                revision: 0,
            });
        }
        let Some(set) = self.current.as_mut() else {
            return (&[], 0);
        };
        let unchanged =
            !owner_changed
                && set.base == base
                && set.descriptors.iter().zip(bindings.clone()).all(
                    |(descriptor, (action, chord))| {
                        descriptor.action == action && descriptor.chord == chord
                    },
                )
                && set.descriptors.len() == bindings.clone().count();
        if !unchanged {
            set.base = base;
            set.descriptors.clear();
            set.descriptors
                .extend(bindings.map(|(action, chord)| BindingDescriptor {
                    action,
                    chord,
                    label: "",
                    priority: 0,
                    visible: false,
                }));
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
        if self.key == Some(key) {
            return;
        }
        self.key = Some(key);
        self.layer.hints.clear();
        self.layer.badge = None;
        self.layer.status = None;
        self.layer.centered = false;
        for binding in table {
            let Some(chord) = binding
                .visible
                .then(|| keymap.component_chord(key.focus, binding.action, binding.chord))
                .flatten()
            else {
                continue;
            };
            if self.layer.hints.iter().any(|hint| hint.chord == chord) {
                continue;
            }
            let hint = Hint {
                chord,
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

/// One hint in the hint bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    /// The chord shown.
    pub chord: Chord,
    /// The label shown.
    pub label: &'static str,
    /// Higher survives width pressure longer.
    pub priority: u8,
}

/// A layer of hints contributed by a screen, a layer or a component.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct HintLayer {
    /// The hints, in display order.
    pub hints: Vec<Hint>,
    /// An optional badge at the start.
    pub badge: Option<&'static str>,
    /// An optional status message.
    pub status: Option<Cow<'static, str>>,
    /// Centre the row instead of left-aligning it.
    pub centered: bool,
}

impl HintLayer {
    /// No hints.
    pub const fn empty() -> Self {
        HintLayer {
            hints: Vec::new(),
            badge: None,
            status: None,
            centered: false,
        }
    }

    /// Whether this layer contributes no visible content.
    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
            && self.badge.is_none()
            && self.status.as_deref().is_none_or(str::is_empty)
    }

    /// Derive hints from a binding table: visible bindings only, sorted by
    /// priority descending, ties by declaration order.
    pub fn from_bindings<C: Copy + 'static>(table: &[Binding<C>]) -> Self {
        let mut hints: Vec<Hint> = table
            .iter()
            .filter(|b| b.visible)
            .filter_map(|b| b.chord.map(|chord| (b, chord)))
            .map(|(b, chord)| Hint {
                chord,
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
            core::iter::once((action, Some(Chord::key(KeyCode::F(1))))),
        );
        let (_, unchanged) = snapshots.update(
            Id::root("first"),
            None,
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
            core::iter::once((action, Some(Chord::key(KeyCode::F(3))))),
        );
        let (_, wrapped) = snapshots.update(
            Id::root("wrap"),
            None,
            core::iter::once((action, Some(Chord::key(KeyCode::F(4))))),
        );
        assert_eq!(max, u64::MAX);
        assert_eq!(wrapped, 0);
        assert_ne!(
            BindingTableId::dynamic(Id::root("wrap"), 1, max),
            BindingTableId::dynamic(Id::root("wrap"), 1, wrapped)
        );
    }
}
