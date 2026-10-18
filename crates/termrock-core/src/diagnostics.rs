#![doc = r" Runtime diagnostics (`COMPONENT_ARCHITECTURE.md` §17.0 A9, §21 item 30 P6)."]
#![doc = r""]
#![doc = r" Never a panic: collisions, rejected cursor writes and undelivered"]
#![doc = r" intents are recorded and surfaced through `Runtime::diagnostics()`."]
#![doc = r" At most the documented `CAPACITY` bound is retained per `handle`, plus a"]
#![doc = r" dropped count."]

use ratatui_core::layout::Rect;

use crate::action::ActionKey;
use crate::event::Chord;
use crate::id::Id;
use crate::keys::KeyPhase;
use crate::layer_types::LayerId;

#[doc = r" One recorded condition."]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    #[doc = r" Hover-dependent geometry did not converge within four presentations."]
    #[doc = r" Hover stays suppressed until the next admitted pointer move."]
    HoverLayoutDidNotSettle,
    #[doc = r" Two `Control` regions registered the same id in one frame."]
    DuplicateId {
        #[doc = r" The id."]
        id: Id,
        #[doc = r" The first area."]
        first: Rect,
        #[doc = r" The second area."]
        second: Rect,
    },
    #[doc = r" A cursor write was dropped (§8.4)."]
    CursorRejected {
        #[doc = r" The writer."]
        owner: Id,
        #[doc = r" The layer it wrote from."]
        layer: LayerId,
    },
    #[doc = r" More than one admissible fallback editor was declared on one layer."]
    TypingTargetConflict {
        #[doc = r" The first editor."]
        a: Id,
        #[doc = r" The other editor."]
        b: Id,
        #[doc = r" The active layer."]
        layer: LayerId,
    },
    #[doc = r" An owner with a `Control`/`Part` region drained nothing (§3.3 step 9)."]
    UndeliveredIntent {
        #[doc = r" The owner."]
        owner: Id,
    },
    #[doc = r" Two visible bindings share a chord in one phase."]
    BindingConflict {
        #[doc = r" The chord."]
        chord: Chord,
        #[doc = r" The phase."]
        phase: KeyPhase,
        #[doc = r" The first owner."]
        a: Id,
        #[doc = r" The second owner."]
        b: Id,
    },
    #[doc = r" A component published the same stable action more than once."]
    DuplicateBindingAction {
        #[doc = r" The component publishing the table."]
        owner: Id,
        #[doc = r" The repeated action."]
        action: ActionKey,
    },
    #[doc = r" A fifth focus pass was required (§21 item 11)."]
    FocusTransitionDidNotSettle {
        #[doc = r" The last requested target."]
        target: Option<Id>,
    },
    #[doc = r" A test addressed an id that has no area (§21 item 17, F7)."]
    UnaddressableId {
        #[doc = r" The id."]
        id: Id,
    },
    #[doc = r" `ui.layer` was called twice for one id in one frame (F10)."]
    DuplicateLayerDraw {
        #[doc = r" The layer id."]
        id: Id,
    },
}

#[doc = r" Bounded diagnostic store."]
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
    dropped: usize,
}

impl Diagnostics {
    #[doc = r" Retained diagnostics per `handle`."]
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
