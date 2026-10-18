#![doc = r" Captured workspace writes. Payloads stay in the deterministic World, never in a UI callback."]
use super::workspace::{Workspace, WorkspaceId};
use std::fmt;

#[doc = r" Identity of one admitted save and its reserved durable target."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveTicket {
    #[doc = r" World-lifetime operation identity."]
    pub operation: u64,
    #[doc = r" Existing or reserved workspace identifier."]
    pub workspace: WorkspaceId,
}

#[doc = r" Why a save could not be admitted."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    #[doc = r" No immutable preview has been reviewed."]
    NoReview,
    #[doc = r" Draft values changed after preview."]
    ChangedReview,
    #[doc = r" This editor already owns an admitted save."]
    Busy,
    #[doc = r" Persisted target differs from the loaded original."]
    TargetChanged,
    #[doc = r" A checked identity allocator reached its limit."]
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

#[doc = r" Result after validating and consuming one captured write."]
#[derive(Clone, PartialEq, Eq)]
pub enum SaveResult {
    #[doc = r" Unknown, early or already consumed completion."]
    Ignored,
    #[doc = r" Captured configuration was persisted atomically."]
    Saved {
        #[doc = r" Completed operation identity."]
        ticket: SaveTicket,
        #[doc = r" Exact persisted configuration; debug output excludes its environment values."]
        workspace: Box<Workspace>,
    },
    #[doc = r" Simulated write failed, with no durable mutation."]
    Failed(SaveTicket),
    #[doc = r" Target changed or disappeared before completion."]
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
    #[doc = r" Ticket for results belonging to an admitted write."]
    pub const fn ticket(&self) -> Option<SaveTicket> {
        match self {
            Self::Ignored => None,
            Self::Saved { ticket, .. } | Self::Failed(ticket) | Self::Stale(ticket) => {
                Some(*ticket)
            }
        }
    }
}

#[doc = r" Queued editor workspace write: what to save, when, and against which"]
#[doc = r" expected snapshot."]
#[derive(Clone)]
pub struct PendingWrite {
    #[doc = r" Ticket this write fulfills."]
    pub ticket: SaveTicket,
    #[doc = r" Virtual deadline in milliseconds."]
    pub due_ms: i64,
    #[doc = r" Snapshot the write was staged against, if any."]
    pub expected: Option<Box<Workspace>>,
    #[doc = r" Proposed workspace contents."]
    pub proposed: Box<Workspace>,
    #[doc = r" Whether the write applies cleanly."]
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
