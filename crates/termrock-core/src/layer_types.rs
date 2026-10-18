#![doc = r" Core layer types: LayerId, DismissReason, LayerEvent."]

use crate::action::ActionKey;

#[doc = r" Unique identifier for an open layer."]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LayerId(pub u16);

impl LayerId {
    #[doc = r" The page."]
    pub const PAGE: LayerId = LayerId(0);

    #[doc = r" The stack position."]
    pub const fn index(self) -> u16 {
        self.0
    }

    #[doc = r" Return the raw integer id."]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[doc = r" Why a layer was dismissed."]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DismissReason {
    #[doc = r" Esc."]
    Esc,
    #[doc = r" A click outside."]
    OutsideClick,
    #[doc = r" Focus left."]
    FocusOut,
    #[doc = r" Programmatic close."]
    Programmatic,
}

#[doc = r" Events emitted during layer transitions."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerEvent {
    #[doc = r" The layer opened."]
    Opened,
    #[doc = r" The layer was dismissed."]
    Dismissed(DismissReason),
    #[doc = r" The layer was closed with an action."]
    Closed(ActionKey),
}
