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
