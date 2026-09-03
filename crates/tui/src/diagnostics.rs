//! Runtime diagnostics (`COMPONENT_ARCHITECTURE.md` §17.0 A9, §21 item 30 P6).
//!
//! Never a panic: collisions, rejected cursor writes and undelivered
//! intents are recorded and surfaced through `Runtime::diagnostics()`.
//! At most [`Diagnostics::CAPACITY`] are retained per `handle`, plus a
//! dropped count.

use ratatui_core::layout::Rect;

use crate::event::Chord;
use crate::id::Id;
use crate::keymap::KeyPhase;
use crate::layer::LayerId;

/// One recorded condition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    /// Two `Control` regions registered the same id in one frame.
    DuplicateId {
        /// The id.
        id: Id,
        /// The first area.
        first: Rect,
        /// The second area.
        second: Rect,
    },
    /// A cursor write was dropped (§8.4).
    CursorRejected {
        /// The writer.
        owner: Id,
        /// The layer it wrote from.
        layer: LayerId,
    },
    /// An owner with a `Control`/`Part` region drained nothing (§3.3 step 9).
    UndeliveredIntent {
        /// The owner.
        owner: Id,
    },
    /// Two visible bindings share a chord in one phase.
    BindingConflict {
        /// The chord.
        chord: Chord,
        /// The phase.
        phase: KeyPhase,
        /// The first owner.
        a: Id,
        /// The second owner.
        b: Id,
    },
    /// A fifth focus pass was required (§21 item 11).
    FocusTransitionDidNotSettle {
        /// The last requested target.
        target: Option<Id>,
    },
    /// A test addressed an id that has no area (§21 item 17, F7).
    UnaddressableId {
        /// The id.
        id: Id,
    },
    /// `ui.layer` was called twice for one id in one frame (F10).
    DuplicateLayerDraw {
        /// The layer id.
        id: Id,
    },
}

/// Bounded diagnostic store.
#[derive(Debug, Default, Clone)]
pub(crate) struct Diagnostics {
    items: Vec<Diagnostic>,
    dropped: usize,
}

impl Diagnostics {
    /// Retained diagnostics per `handle`.
    pub(crate) const CAPACITY: usize = 64;

    pub(crate) fn push(&mut self, d: Diagnostic) {
        if self.items.len() < Self::CAPACITY {
            self.items.push(d);
        } else {
            self.dropped = self.dropped.saturating_add(1);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.dropped = 0;
    }

    pub(crate) fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    pub(crate) fn dropped(&self) -> usize {
        self.dropped
    }

    pub(crate) fn extend(&mut self, other: impl IntoIterator<Item = Diagnostic>) {
        for d in other {
            self.push(d);
        }
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
}
