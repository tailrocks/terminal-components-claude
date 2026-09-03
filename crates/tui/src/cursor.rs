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
    /// The owner carried `FOCUSED` when the write was made. `Ui::set_cursor`
    /// keeps the best candidate by `(layer, focused)`, because §8.4 makes the
    /// *runtime* the filter and two same-layer writers are legitimate.
    pub(crate) focused: bool,
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
            focused: true,
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

    /// Two same-layer writers (two focusable text controls in one form) are
    /// legitimate: §8.4 makes filtering the runtime's job, so components write
    /// unconditionally. `Ui::set_cursor` must keep the **focused** owner's
    /// write, not the first one drawn — otherwise `cursor::resolve` drops the
    /// retained request and the frame ends with no cursor at all (BL-6).
    #[test]
    fn the_focused_owners_write_wins_on_the_same_layer() {
        use ratatui_core::buffer::Buffer;
        use ratatui_core::layout::Rect;

        use crate::theme::Theme;
        use crate::ui::Ui;
        use crate::ui::cx::LastFrame;
        use crate::ui::{FrameState, UiCore};

        const FIRST: Id = Id::root("form.first");
        const SECOND: Id = Id::root("form.second");

        let screen = Rect::new(0, 0, 20, 4);
        let theme = Theme::junie();
        for focused in [FIRST, SECOND] {
            let mut frame = FrameState::default();
            frame.reset(1, screen);
            let mut page = Buffer::empty(screen);
            let mut core = UiCore::default();
            let mut last = LastFrame::default();
            last.snapshot.focus = Some(focused);
            {
                let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
                // drawn in this order regardless of which one is focused
                ui.set_cursor(FIRST, Position::new(1, 1));
                ui.set_cursor(SECOND, Position::new(9, 2));
            }
            let req = frame.cursor.expect("a cursor request survives the frame");
            assert_eq!(req.owner, focused);
            assert!(req.focused);
            // and the runtime keeps it, because its owner is the focused one
            assert_eq!(
                resolve(req, LayerId::PAGE, Some(focused)),
                CursorDecision::Keep(req.pos)
            );
            // exactly one loser is diagnosed, and it is the other control
            let rejected: Vec<Id> = frame
                .diagnostics
                .iter()
                .filter_map(|d| match d {
                    Diagnostic::CursorRejected { owner, .. } => Some(*owner),
                    _ => None,
                })
                .collect();
            let other = if focused == FIRST { SECOND } else { FIRST };
            assert_eq!(rejected, vec![other]);
        }
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
