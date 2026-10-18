use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Position;

use super::event::Key;
use super::hit::Axis;
use super::id::{PartRef, WidgetId};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent<'a> {
    Key(Key),

    Binding(ActionKey),

    Paste(&'a str),

    Pointer {
        phase: Phase,

        part: PartRef,

        pos: Position,

        count: u8,

        mods: KeyModifiers,
    },

    Wheel {
        axis: Axis,

        delta: i16,

        part: PartRef,

        pos: Position,

        mods: KeyModifiers,
    },

    FocusIn(FocusVia),

    FocusOut(Option<WidgetId>),

    Layer(LayerEvent),

    Cancel,
}

impl Intent<'_> {
    pub const fn pointer(phase: Phase, part: PartRef, pos: Position) -> Self {
        Intent::Pointer {
            phase,
            part,
            pos,
            count: 1,
            mods: KeyModifiers::NONE,
        }
    }

    pub const fn part(&self) -> Option<PartRef> {
        match self {
            Intent::Pointer { part, .. } | Intent::Wheel { part, .. } => Some(*part),
            _ => None,
        }
    }

    pub const fn pos(&self) -> Option<Position> {
        match self {
            Intent::Pointer { pos, .. } | Intent::Wheel { pos, .. } => Some(*pos),
            _ => None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Move,

    Press,

    Release,

    Click,

    Secondary,

    SecondaryRelease,

    Middle,

    DragStart,

    Drag,

    DragEnd,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusVia {
    Keyboard,

    Pointer,

    Programmatic,

    Restore,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ActionKey(u16);

impl ActionKey {
    pub const CONFIRM: ActionKey = ActionKey(0);

    pub const CANCEL: ActionKey = ActionKey(1);

    pub const CLOSE: ActionKey = ActionKey(2);

    pub const fn custom(name: &str) -> ActionKey {
        ActionKey(super::id::custom_hash16(name))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DismissReason {
    Esc,

    CloseButton,

    OutsideClick,

    FocusOut,

    Programmatic,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerEvent {
    Opened,

    Dismissed(DismissReason),

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
