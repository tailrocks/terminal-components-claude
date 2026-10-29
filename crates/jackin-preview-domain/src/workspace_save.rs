use super::workspace::{Workspace, WorkspaceId};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveTicket {
    pub operation: u64,
    pub workspace: WorkspaceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    NoReview,
    ChangedReview,
    Busy,
    TargetChanged,
    IdentityExhausted,
}
impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoReview => "Review the workspace before saving",
            Self::ChangedReview => "Draft changed · review again before saving",
            Self::Busy => "Workspace save already running",
            Self::TargetChanged => "Workspace changed · reload before saving",
            Self::IdentityExhausted => "Workspace save identity exhausted",
        })
    }
}
impl std::error::Error for SaveError {}

#[derive(Clone, PartialEq, Eq)]
pub enum SaveResult {
    Ignored,
    Saved {
        ticket: SaveTicket,
        workspace: Box<Workspace>,
    },
    Failed(SaveTicket),
    Stale(SaveTicket),
}
impl fmt::Debug for SaveResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ignored => f.write_str("Ignored"),
            Self::Saved { ticket, .. } => f.debug_tuple("Saved").field(ticket).finish(),
            Self::Failed(ticket) => f.debug_tuple("Failed").field(ticket).finish(),
            Self::Stale(ticket) => f.debug_tuple("Stale").field(ticket).finish(),
        }
    }
}
impl SaveResult {
    pub const fn ticket(&self) -> Option<SaveTicket> {
        match self {
            Self::Ignored => None,
            Self::Saved { ticket, .. } | Self::Failed(ticket) | Self::Stale(ticket) => {
                Some(*ticket)
            }
        }
    }
}

#[derive(Clone)]
pub struct PendingWrite {
    pub ticket: SaveTicket,
    pub due_ms: i64,
    pub expected: Option<Box<Workspace>>,
    pub proposed: Box<Workspace>,
    pub ok: bool,
}
impl fmt::Debug for PendingWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingWrite")
            .field("ticket", &self.ticket)
            .field("due_ms", &self.due_ms)
            .field("ok", &self.ok)
            .finish_non_exhaustive()
    }
}
