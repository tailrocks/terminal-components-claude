//! Keymaps, bindings and hints (`COMPONENT_ARCHITECTURE.md` §13.1, §21 item 10).
//!
//! Components declare bindings from small `const` tables selected by state;
//! an application overrides and adds product chords through [`KeyMap`] in
//! two explicit phases. Hints are derived from the same tables.

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
    /// The chord.
    pub chord: Chord,
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
    /// Find the command bound to a key press in `table`.
    pub fn lookup(table: &[Binding<C>], key: &Key) -> Option<C> {
        table.iter().find(|b| b.chord.matches(key)).map(|b| b.cmd)
    }
}

/// A component's binding tables, one per state.
pub trait Bindings {
    /// The command type (`Next`, `Prev`, `Activate`, …).
    type Cmd: Copy + 'static;

    /// The table for `st`.
    fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>];
}

/// Visible bindings on the same chord within one table are a conflict.
pub fn binding_conflicts<C: Copy + 'static>(
    owner: Id,
    phase: KeyPhase,
    table: &[Binding<C>],
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (i, a) in table.iter().enumerate() {
        if !a.visible {
            continue;
        }
        let dup = table
            .iter()
            .skip(i.saturating_add(1))
            .any(|b| b.visible && b.chord == a.chord);
        if dup {
            out.push(Diagnostic::BindingConflict {
                chord: a.chord,
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

/// The application's chord layer: add, remove and remap per phase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyMap {
    entries: Vec<Entry>,
}

impl KeyMap {
    /// The empty map.
    pub const EMPTY: KeyMap = KeyMap {
        entries: Vec::new(),
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
        self.entries.is_empty()
    }

    /// A cheap `O(n)` structural fingerprint of the bindings, so a caller can
    /// tell whether the `O(n²)` [`KeyMap::conflicts`] scan needs re-running.
    pub(crate) fn fingerprint(&self) -> u64 {
        use core::hash::{Hash, Hasher};

        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.entries.len().hash(&mut h);
        for e in &self.entries {
            e.chord.hash(&mut h);
            e.phase.hash(&mut h);
            e.key.raw().hash(&mut h);
        }
        h.finish()
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

    /// Derive hints from a binding table: visible bindings only, sorted by
    /// priority descending, ties by declaration order.
    pub fn from_bindings<C: Copy + 'static>(table: &[Binding<C>]) -> Self {
        let mut hints: Vec<Hint> = table
            .iter()
            .filter(|b| b.visible)
            .map(|b| Hint {
                chord: b.chord,
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
                chord: Chord::key(KeyCode::Left),
                cmd: Cmd::A,
                label: "Prev",
                priority: 40,
                visible: true,
            },
            Binding {
                chord: Chord::key(KeyCode::Enter),
                cmd: Cmd::B,
                label: "Select",
                priority: 80,
                visible: true,
            },
            Binding {
                chord: Chord::key(KeyCode::Char(' ')),
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
            Binding::lookup(T, &k(KeyCode::Char(' '))),
            Some(Cmd::B)
        ));
        assert!(binding_conflicts(Id::root("x"), KeyPhase::Bubble, T).is_empty());
    }
}
