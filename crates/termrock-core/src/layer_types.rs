use crate::action::ActionKey;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LayerId(pub u16);

impl LayerId {
    pub const PAGE: LayerId = LayerId(0);

    pub const fn index(self) -> u16 {
        self.0
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DismissReason {
    Esc,
    OutsideClick,
    FocusOut,
    Programmatic,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerEvent {
    Opened,
    Dismissed(DismissReason),
    Closed(ActionKey),
}
