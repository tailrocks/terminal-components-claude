//! Core layer types: LayerId, DismissReason, LayerEvent.

use crate::action::ActionKey;

/// Unique identifier for an open layer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LayerId(pub u16);

impl LayerId {
    /// The page.
    pub const PAGE: LayerId = LayerId(0);

    /// The stack position.
    pub const fn index(self) -> u16 {
        self.0
    }

    /// Return the raw integer id.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Why a layer was dismissed.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DismissReason {
    /// Esc.
    Esc,
    /// A click outside.
    OutsideClick,
    /// Focus left.
    FocusOut,
    /// Programmatic close.
    Programmatic,
}

/// Events emitted during layer transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerEvent {
    /// The layer opened.
    Opened,
    /// The layer was dismissed.
    Dismissed(DismissReason),
    /// The layer was closed with an action.
    Closed(ActionKey),
}
