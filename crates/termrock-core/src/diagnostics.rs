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
    /// A settled focus continuation result was not acknowledged before its bounded retention expired.
    FocusContinuationUnacknowledged {
        owner: Id,
    },
    UnaddressableId {
        id: Id,
    },
    DuplicateLayerDraw {
        id: Id,
    },
    /// A component could not produce a valid layout from its current configuration.
    ///
    /// `field` identifies a field or child when the component can name one; it is
    /// absent for structural errors that apply to the component as a whole.
    InvalidComponentLayout {
        owner: Id,
        field: Option<Id>,
    },
    /// A legacy part painter cannot describe a partially clipped component part.
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

    /// Append another bounded diagnostic store without losing its overflow count.
    /// Existing items remain first; both retained items and dropped counts obey
    /// this store's capacity.
    pub fn append(&mut self, other: &Self) {
        self.extend(other.items.iter().cloned());
        self.dropped = self.dropped.saturating_add(other.dropped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_is_bounded_and_counts_drops() {
        let mut d = Diagnostics::default();
        for _ in 0..70 {
            d.push(Diagnostic::UnaddressableId { id: Id::root("x") });
        }
        assert_eq!(d.items().len(), Diagnostics::CAPACITY);
        assert_eq!(d.dropped(), 6);
        d.clear();
        assert!(d.items().is_empty());
        assert_eq!(d.dropped(), 0);
    }

    #[test]
    fn appending_bounded_stores_preserves_items_and_both_overflow_counts() {
        let mut first = Diagnostics::default();
        let mut second = Diagnostics::default();
        first.push(Diagnostic::UnaddressableId {
            id: Id::root("first"),
        });
        for _ in 0..70 {
            second.push(Diagnostic::UnaddressableId {
                id: Id::root("second"),
            });
        }

        first.append(&second);

        assert_eq!(first.items().len(), Diagnostics::CAPACITY);
        assert_eq!(first.dropped(), 7);
        assert!(matches!(
            first.items().first(),
            Some(Diagnostic::UnaddressableId { id }) if *id == Id::root("first")
        ));
    }

    #[test]
    fn clipped_slot_diagnostics_use_the_same_bounded_store() {
        let owner = Id::root("scroll-region");
        let mut d = Diagnostics::default();
        for _ in 0..Diagnostics::CAPACITY + 3 {
            d.push(Diagnostic::UnsupportedClippedComponentSlot {
                owner,
                part: Part::TRACK,
            });
        }
        assert_eq!(d.items().len(), Diagnostics::CAPACITY);
        assert_eq!(d.dropped(), 3);
        assert!(matches!(
            d.items().first(),
            Some(Diagnostic::UnsupportedClippedComponentSlot { owner: actual, part: Part::TRACK })
                if *actual == owner
        ));
    }
}
