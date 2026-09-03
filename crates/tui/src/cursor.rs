//! Cursor ownership (`COMPONENT_ARCHITECTURE.md` §8.4, §21 item 15).
//!
//! `ui.set_cursor(owner, pos)` records `(layer, owner, pos)`. The runtime
//! keeps the write iff the layer is the top layer and the owner is focused;
//! otherwise it drops it and records `CursorRejected` — except for a write
//! from a suppressed (inert) layer, which is discarded silently.

use ratatui_core::layout::Position;

use crate::diagnostics::Diagnostic;
use crate::id::Id;
use crate::layer::LayerId;

/// A cursor request made during draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CursorRequest {
    pub(crate) layer: LayerId,
    pub(crate) owner: Id,
    pub(crate) pos: Position,
    /// The layer was inert (below an `inert_below` layer) when written.
    pub(crate) inert: bool,
}

/// The outcome of resolving the frame's cursor requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CursorDecision {
    Keep(Position),
    Reject(Diagnostic),
    Silent,
}

/// Resolve one request against the top layer and the focused owner.
pub(crate) fn resolve(req: CursorRequest, top: LayerId, focus: Option<Id>) -> CursorDecision {
    if req.inert {
        return CursorDecision::Silent;
    }
    if req.layer == top && focus == Some(req.owner) {
        CursorDecision::Keep(req.pos)
    } else {
        CursorDecision::Reject(Diagnostic::CursorRejected {
            owner: req.owner,
            layer: req.layer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: Id = Id::root("input");

    const fn req(layer: LayerId, inert: bool) -> CursorRequest {
        CursorRequest {
            layer,
            owner: OWNER,
            pos: Position::new(4, 2),
            inert,
        }
    }

    #[test]
    fn cursor_write_is_kept_for_the_focused_owner_on_the_top_layer() {
        assert_eq!(
            resolve(req(LayerId(1), false), LayerId(1), Some(OWNER)),
            CursorDecision::Keep(Position::new(4, 2))
        );
    }

    #[test]
    fn cursor_write_from_a_lower_layer_is_rejected() {
        assert!(matches!(
            resolve(req(LayerId::PAGE, false), LayerId(1), Some(OWNER)),
            CursorDecision::Reject(Diagnostic::CursorRejected {
                owner: OWNER,
                layer: LayerId::PAGE
            })
        ));
        // an inert lower layer is discarded silently (§21 item 15)
        assert_eq!(
            resolve(req(LayerId::PAGE, true), LayerId(1), Some(OWNER)),
            CursorDecision::Silent
        );
    }

    #[test]
    fn cursor_write_from_an_unfocused_owner_is_rejected() {
        assert!(matches!(
            resolve(
                req(LayerId::PAGE, false),
                LayerId::PAGE,
                Some(Id::root("other"))
            ),
            CursorDecision::Reject(_)
        ));
        assert!(matches!(
            resolve(req(LayerId::PAGE, false), LayerId::PAGE, None),
            CursorDecision::Reject(_)
        ));
    }

    #[test]
    fn rejection_records_a_diagnostic() {
        let CursorDecision::Reject(d) = resolve(req(LayerId::PAGE, false), LayerId(2), None) else {
            panic!("expected a rejection");
        };
        assert_eq!(
            d,
            Diagnostic::CursorRejected {
                owner: OWNER,
                layer: LayerId::PAGE
            }
        );
    }
}
