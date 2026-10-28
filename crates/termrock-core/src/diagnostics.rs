use ratatui_core::layout::Rect;

use crate::action::ActionKey;
use crate::event::Chord;
use crate::id::{Id, Part};
use crate::keys::KeyPhase;
use crate::layer_types::LayerId;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    HoverLayoutDidNotSettle,
    DuplicateId {
        id: Id,
        first: Rect,
        second: Rect,
    },
    CursorRejected {
        owner: Id,
        layer: LayerId,
    },
    TypingTargetConflict {
        a: Id,
        b: Id,
        layer: LayerId,
    },
    UndeliveredIntent {
        owner: Id,
    },
    BindingConflict {
        chord: Chord,
        phase: KeyPhase,
        a: Id,
        b: Id,
    },
    DuplicateBindingAction {
        owner: Id,
        action: ActionKey,
    },
    FocusTransitionDidNotSettle {
        target: Option<Id>,
    },

    FocusContinuationUnacknowledged {
        owner: Id,
    },
    UnaddressableId {
        id: Id,
    },
    DuplicateLayerDraw {
        id: Id,
    },

    InvalidComponentLayout {
        owner: Id,
        field: Option<Id>,
    },

    UnsupportedClippedComponentSlot {
        owner: Id,
        part: Part,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
    dropped: usize,
}

impl Diagnostics {
    pub(crate) const CAPACITY: usize = 64;

    pub fn push(&mut self, d: Diagnostic) {
        if self.items.len() < Self::CAPACITY {
            self.items.push(d);
        } else {
            self.dropped = self.dropped.saturating_add(1);
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.dropped = 0;
    }

    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    pub fn dropped(&self) -> usize {
        self.dropped
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = Diagnostic>) {
        for d in other {
            self.push(d);
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.extend(other.items.iter().cloned());
        self.dropped = self.dropped.saturating_add(other.dropped);
    }
}
