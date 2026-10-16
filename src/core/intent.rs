//! Pre-resolved input: what a component receives once the runtime has
//! resolved an [`Input`](super::event::Input) to an owner and a part.
//!
//! [`Intent`] is the one event shape of the shared input chain
//! (docs/architecture/shared-components-plan.md §1.3, §1.5). The
//! dispatcher builds it from `Input`, `PointerState` and the hit registry;
//! widgets never see raw ids or [`MouseKind`](super::event::MouseKind).
//! Step S0 adds the vocabulary only. Widgets still take `Key` and `Mouse`
//! through their `on_*` methods until the dispatcher (S6) and the
//! `Component` trait (S15) deliver intents; the names and shapes follow
//! `termrock-core` on `main` (plan §1.9) so the port moves code instead of
//! rewriting it.

use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Position;

use super::event::Key;
use super::hit::Axis;
use super::id::{PartRef, WidgetId};

/// What a component receives.
///
/// `main` names the same enum `Intent` (termrock-core `intent.rs`); this
/// branch adds `count` to [`Intent::Pointer`] (triple click) and `mods` to
/// [`Intent::Wheel`] (Shift+wheel scrolls sideways), plan §1.9.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent<'a> {
    /// A key press delivered to the focused owner.
    Key(Key),
    /// A declared component action resolved through its effective chord.
    Binding(ActionKey),
    /// Pasted text, delivered to the focused owner while it edits text.
    Paste(&'a str),
    /// A pointer phase over one of the owner's parts.
    Pointer {
        /// The phase.
        phase: Phase,
        /// The part under the pointer (or the captured part during a drag).
        part: PartRef,
        /// Absolute position.
        pos: Position,
        /// 1, 2 or 3 for a single, double or triple click (500 ms, 1-cell
        /// slop). There is no separate double-click phase.
        count: u8,
        /// Modifiers held.
        mods: KeyModifiers,
    },
    /// Wheel motion over a scroll region.
    Wheel {
        /// The axis.
        axis: Axis,
        /// Rows or columns to scroll, signed (negative is up or left),
        /// already multiplied by the configured wheel step.
        delta: i16,
        /// The scroll region's part.
        part: PartRef,
        /// Absolute position.
        pos: Position,
        /// Modifiers held.
        mods: KeyModifiers,
    },
    /// Focus arrived.
    FocusIn(FocusVia),
    /// Focus left, towards the new owner if any.
    FocusOut(Option<WidgetId>),
    /// A layer owned by this component changed lifecycle.
    Layer(LayerEvent),
    /// Esc reached this owner after layer dismissal.
    Cancel,
}

impl Intent<'_> {
    /// A primary-button intent of `phase` over `part`, one click, no
    /// modifiers. The common case in tests and adapters.
    pub const fn pointer(phase: Phase, part: PartRef, pos: Position) -> Self {
        Intent::Pointer {
            phase,
            part,
            pos,
            count: 1,
            mods: KeyModifiers::NONE,
        }
    }

    /// The part a pointer or wheel intent addresses.
    pub const fn part(&self) -> Option<PartRef> {
        match self {
            Intent::Pointer { part, .. } | Intent::Wheel { part, .. } => Some(*part),
            _ => None,
        }
    }

    /// The position of a pointer or wheel intent.
    pub const fn pos(&self) -> Option<Position> {
        match self {
            Intent::Pointer { pos, .. } | Intent::Wheel { pos, .. } => Some(*pos),
            _ => None,
        }
    }
}

/// Pointer phases.
///
/// `main` has a `DoubleClick` phase; here the click count travels in
/// [`Intent::Pointer`]'s `count`, which also expresses a triple click
/// (plan §1.9).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The pointer moved over the part with no button held.
    Move,
    /// Primary button down on the part.
    Press,
    /// Primary button up (anywhere, after a press on the owner).
    Release,
    /// Press and release on the same part.
    Click,
    /// Secondary button down on the part (a context action).
    Secondary,
    /// Secondary button up (press, drag, release chooses a menu row).
    SecondaryRelease,
    /// Middle button down (closes a tab).
    Middle,
    /// The first motion with the primary button held.
    DragStart,
    /// Motion with the primary button held.
    Drag,
    /// The button was released after a drag. Every drag target gets it.
    DragEnd,
}

/// How focus arrived.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusVia {
    /// Tab or Shift+Tab.
    Keyboard,
    /// A press focused the owner.
    Pointer,
    /// Requested by code, or focus reconciliation.
    Programmatic,
    /// Restored after a layer closed.
    Restore,
}

/// A typed action identity, `const`-constructible so binding tables and
/// dialog action rows can name it without allocation (`main`:
/// termrock-core `action.rs`). `0..=255` is the library range;
/// [`ActionKey::custom`] maps an app's name into `0x8000..=0xFFFF`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ActionKey(u16);

impl ActionKey {
    /// Confirm / OK.
    pub const CONFIRM: ActionKey = ActionKey(0);
    /// Cancel.
    pub const CANCEL: ActionKey = ActionKey(1);
    /// Close.
    pub const CLOSE: ActionKey = ActionKey(2);

    /// An app-defined action, folded from its name into the high range.
    pub const fn custom(name: &str) -> ActionKey {
        ActionKey(super::id::custom_hash16(name))
    }

    /// The raw number.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Why a layer was dismissed. Dismissing resolves to the layer's cancel
/// result (plan §1.7).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DismissReason {
    /// Esc.
    Esc,
    /// A click outside the layer.
    OutsideClick,
    /// Focus left the layer.
    FocusOut,
    /// Closed by code.
    Programmatic,
}

/// Lifecycle events of a layer, delivered to its owner.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerEvent {
    /// The layer opened.
    Opened,
    /// The layer was dismissed.
    Dismissed(DismissReason),
    /// The layer closed with an action.
    Closed(ActionKey),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::id::{ItemKey, Part};

    #[test]
    fn pointer_constructor_is_one_plain_click() {
        let part = PartRef::item(Part::ROW, ItemKey::index(3));
        let i = Intent::pointer(Phase::Click, part, Position::new(4, 2));
        assert_eq!(
            i,
            Intent::Pointer {
                phase: Phase::Click,
                part,
                pos: Position::new(4, 2),
                count: 1,
                mods: KeyModifiers::NONE,
            }
        );
        assert_eq!(i.part(), Some(part));
        assert_eq!(i.pos(), Some(Position::new(4, 2)));
    }

    #[test]
    fn non_pointer_intents_have_no_part() {
        assert_eq!(Intent::Cancel.part(), None);
        assert_eq!(Intent::Paste("x").pos(), None);
        let wheel = Intent::Wheel {
            axis: Axis::Horizontal,
            delta: -3,
            part: PartRef::of(Part::BODY),
            pos: Position::new(1, 1),
            mods: KeyModifiers::SHIFT,
        };
        assert_eq!(wheel.part(), Some(PartRef::of(Part::BODY)));
    }

    #[test]
    fn action_keys_keep_library_and_custom_ranges_apart() {
        assert_ne!(ActionKey::CONFIRM, ActionKey::CANCEL);
        let save = ActionKey::custom("save");
        assert!(save.raw() >= 0x8000, "custom keys land in the high range");
        assert_eq!(save, ActionKey::custom("save"), "stable across calls");
        assert_eq!(
            LayerEvent::Closed(ActionKey::CANCEL),
            LayerEvent::Closed(ActionKey::CANCEL)
        );
    }
}
