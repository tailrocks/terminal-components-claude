use ratatui_core::layout::Position;

use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::id::Id;
use crate::layer::LayerId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorShape {
    Block,
    Bar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CursorRequest {
    pub(crate) layer: LayerId,
    pub(crate) owner: Id,
    pub(crate) pos: Position,
    pub(crate) inert: bool,
    pub(crate) typing_offer: bool,
    pub(crate) shape: CursorShape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CursorDecision {
    Keep(Position),
    Reject(Diagnostic),
    Silent,
}

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

pub(crate) fn resolve_requests(
    requests: &[CursorRequest],
    top: LayerId,
    owner: Option<Id>,
    typing_owner: Option<Id>,
    diagnostics: &mut Diagnostics,
    typing: &[crate::runtime::typing::TypingDeclaration],
) -> Option<(Position, CursorShape)> {
    let mut kept: Option<CursorRequest> = None;
    for request in requests {
        if request.inert {
            continue;
        }
        if request.typing_offer {
            let declared = typing.iter().any(|candidate| {
                candidate.owner == request.owner
                    && candidate.layer == request.layer
                    && candidate.cursor
            });
            if !declared {
                diagnostics.push(Diagnostic::CursorRejected {
                    owner: request.owner,
                    layer: request.layer,
                });
                continue;
            }
            if request.layer != top || typing_owner != Some(request.owner) {
                continue;
            }
        }
        match resolve(*request, top, owner) {
            CursorDecision::Keep(_) => {
                if let Some(previous) = kept.replace(*request) {
                    diagnostics.push(Diagnostic::CursorRejected {
                        owner: previous.owner,
                        layer: previous.layer,
                    });
                }
            }
            CursorDecision::Reject(diagnostic) => diagnostics.push(diagnostic),
            CursorDecision::Silent => {}
        }
    }
    kept.map(|request| (request.pos, request.shape))
}
