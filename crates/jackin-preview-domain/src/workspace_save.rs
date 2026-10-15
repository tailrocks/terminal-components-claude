//! Captured workspace writes. Payloads stay in the deterministic World, never in a UI callback.
use super::workspace::{Workspace, WorkspaceId};
use std::fmt;

/// Identity of one admitted save and its reserved durable target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveTicket {
    /// World-lifetime operation identity.
    pub operation: u64,
    /// Existing or reserved workspace identifier.
    pub workspace: WorkspaceId,
}

/// Why a save could not be admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    /// No immutable preview has been reviewed.
    NoReview,
    /// Draft values changed after preview.
    ChangedReview,
    /// This editor already owns an admitted save.
    Busy,
    /// Persisted target differs from the loaded original.
    TargetChanged,
    /// A checked identity allocator reached its limit.
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

/// Result after validating and consuming one captured write.
#[derive(Clone, PartialEq, Eq)]
pub enum SaveResult {
    /// Unknown, early or already consumed completion.
    Ignored,
    /// Captured configuration was persisted atomically.
    Saved {
        /// Completed operation identity.
        ticket: SaveTicket,
        /// Exact persisted configuration; debug output excludes its environment values.
        workspace: Box<Workspace>,
    },
    /// Simulated write failed, with no durable mutation.
    Failed(SaveTicket),
    /// Target changed or disappeared before completion.
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
    /// Ticket for results belonging to an admitted write.
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
pub(crate) struct PendingWrite {
    pub(crate) ticket: SaveTicket,
    pub(crate) due_ms: i64,
    pub(crate) expected: Option<Box<Workspace>>,
    pub(crate) proposed: Box<Workspace>,
    pub(crate) ok: bool,
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
