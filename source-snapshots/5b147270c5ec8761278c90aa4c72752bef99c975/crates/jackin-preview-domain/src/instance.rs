//! Durable instance records, persisted session records, and live daemon
//! snapshots. The three are kept apart on purpose: a persisted record can
//! outlive its daemon, and a daemon snapshot is never invented from records.

use core::fmt;

use super::agent::Agent;
use super::workspace::WorkspaceId;

/// Stable identifier for a durable instance record.
pub type InstanceId = String;

/// Stable numeric identity for one launch attempt.
///
/// Keeping this separate from instance and workspace names prevents the
/// capsule from accidentally treating arbitrary display text as a run id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(u64);

impl RunId {
    /// Construct a fixture or persisted run identity.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The numeric identity.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Construct a stable identity from arbitrary fixture text.
    ///
    /// The input is hashed instead of sliced or copied into a display token,
    /// so short and malformed producer values remain total.
    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in label.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        Self(hash)
    }

    /// An eight-hex-digit display form. The full value remains in the typed
    /// field; this compact projection is for rows and status text only.
    pub fn short(self) -> String {
        format!("{:08x}", self.0 & u64::from(u32::MAX))
    }

    /// The public short container identifier used by Capsule diagnostics.
    pub fn container_uid(self) -> String {
        format!("3f9c{}e21a", self.short())
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run-{}", self.short())
    }
}

/// Lifecycle state of a durable instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceStatus {
    /// The container and daemon are running.
    Running,
    /// The final session exited successfully.
    CleanExited,
    /// The daemon stopped unexpectedly.
    Crashed,
    /// The stopped instance retains uncommitted changes.
    PreservedDirty,
    /// The stopped instance retains unpushed commits.
    PreservedUnpushed,
    /// The instance has a restorable Capsule layout.
    RestoreAvailable,
    /// A newer instance replaced this one.
    Superseded,
    /// The instance and its records were removed.
    Purged,
    /// Setup failed before the Capsule was ready.
    FailedSetup,
}

impl InstanceStatus {
    /// Compact lifecycle label shown in the tree row.
    pub fn label(self) -> &'static str {
        match self {
            InstanceStatus::Running => "running",
            InstanceStatus::CleanExited => "exited",
            InstanceStatus::Crashed => "crashed",
            InstanceStatus::PreservedDirty => "preserved · dirty",
            InstanceStatus::PreservedUnpushed => "preserved · unpushed",
            InstanceStatus::RestoreAvailable => "restore available",
            InstanceStatus::Superseded => "superseded",
            InstanceStatus::Purged => "purged",
            InstanceStatus::FailedSetup => "failed setup",
        }
    }

    /// Return the full operator-facing lifecycle description.
    pub fn description(self) -> &'static str {
        match self {
            InstanceStatus::Running => "Container and Capsule daemon are live.",
            InstanceStatus::CleanExited => {
                "The final session ended cleanly; the container is stopped."
            }
            InstanceStatus::Crashed => {
                "The Capsule daemon stopped unexpectedly; the container is kept for inspection."
            }
            InstanceStatus::PreservedDirty => {
                "Stopped with uncommitted changes in the Workspace; kept until you decide."
            }
            InstanceStatus::PreservedUnpushed => {
                "Stopped with commits that were never pushed; kept until you decide."
            }
            InstanceStatus::RestoreAvailable => "Stopped cleanly with a restorable Capsule layout.",
            InstanceStatus::Superseded => "Replaced by a newer instance for the same Workspace.",
            InstanceStatus::Purged => "Container and records were removed.",
            InstanceStatus::FailedSetup => {
                "Launch failed before the Capsule was ready; nothing to attach to."
            }
        }
    }

    /// Return whether this status is hidden from the normal tree.
    /// Hidden from the normal tree, as in current Jackin.
    pub fn hidden(self) -> bool {
        matches!(self, InstanceStatus::Superseded | InstanceStatus::Purged)
    }

    /// Return whether the instance is currently running.
    pub fn is_live(self) -> bool {
        self == InstanceStatus::Running
    }

    /// Return whether the instance can be reconnected to or restored.
    /// Reconnect/restore is offered for these.
    pub fn reconnectable(self) -> bool {
        matches!(
            self,
            InstanceStatus::Running
                | InstanceStatus::RestoreAvailable
                | InstanceStatus::PreservedDirty
                | InstanceStatus::PreservedUnpushed
                | InstanceStatus::Crashed
        )
    }

    /// Return whether stopping the instance is offered.
    /// Stop is offered only for live instances.
    pub fn stoppable(self) -> bool {
        self == InstanceStatus::Running
    }

    /// Return whether the instance retains uncommitted or unpushed work.
    pub fn dirty(self) -> bool {
        matches!(
            self,
            InstanceStatus::PreservedDirty | InstanceStatus::PreservedUnpushed
        )
    }
}

/// Durable instance record and its persisted/live projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// Stable instance identifier.
    pub id: InstanceId,
    /// Container base name; the container id appends the instance suffix.
    pub container: String,
    /// Workspace associated with the instance, when known.
    pub workspace: Option<WorkspaceId>,
    /// Working directory mounted for the instance.
    pub workdir: String,
    /// Role selected for the instance.
    pub role: String,
    /// Agent runtime launched in the instance.
    pub agent: Agent,
    /// Current durable lifecycle status.
    pub status: InstanceStatus,
    /// Creation time in fixture seconds.
    pub created_secs: i64,
    /// Most recent observation time in fixture seconds.
    pub last_seen_secs: i64,
    /// Stable identity for the launch attempt.
    pub run_id: RunId,
    /// Persisted session records (manifest).
    pub sessions: Result<Vec<SessionRecord>, ManifestError>,
    /// Live daemon snapshot; independent from the manifest.
    pub daemon: DaemonSnapshot,
    /// Branch / PR context resolved for this instance.
    pub branch: Option<String>,
    /// Pull request number and title, when one is associated.
    pub pr: Option<(u32, String)>,
    /// Default branch of the associated repository.
    pub default_branch: String,
    /// Uncommitted / unpushed simulated git state.
    pub uncommitted: usize,
    /// Number of commits not pushed to the remote.
    pub unpushed: usize,
    /// The Workspace's effective account set at launch: every account the
    /// container can hand to a session, not just the one that started it.
    pub accounts: Vec<super::account::AccountId>,
}

impl Instance {
    /// The public short container id shown by Capsule's debug information.
    pub fn container_uid(&self) -> String {
        self.run_id.container_uid()
    }

    /// Compute the full container identifier from its base name and instance id.
    pub fn container_id(&self) -> String {
        format!("{}-{}", self.container, self.id.trim_start_matches("jk-"))
    }

    /// Return whether the instance has uncommitted or unpushed work.
    pub fn is_dirty(&self) -> bool {
        self.uncommitted > 0 || self.unpushed > 0
    }

    /// Return a compact summary of the instance's git state.
    pub fn dirty_summary(&self) -> String {
        match (self.uncommitted, self.unpushed) {
            (0, 0) => "clean".into(),
            (u, 0) => format!("{u} uncommitted"),
            (0, p) => format!("{p} unpushed"),
            (u, p) => format!("{u} uncommitted · {p} unpushed"),
        }
    }
}

/// Error state for reading persisted session records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    /// The session manifest could not be read.
    ReadError,
}

impl ManifestError {
    /// Return the operator-facing error label.
    pub fn label(self) -> &'static str {
        match self {
            ManifestError::ReadError => "Sessions unavailable (manifest read error)",
        }
    }
}

/// One persisted session entry from an instance manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    /// Stable session identifier.
    pub id: String,
    /// Agent runtime in the session, when identified.
    pub agent: Option<Agent>,
    /// Operator-facing session label.
    pub label: String,
    /// Current persisted session status.
    pub status: SessionStatus,
    /// Session start time in fixture seconds.
    pub started_secs: i64,
}

/// Persisted outcome of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// The session is still active.
    Active,
    /// The session exited with the given process code.
    Exited(i32),
    /// The session ended without a normal exit code.
    Crashed,
}

impl SessionStatus {
    /// Return the operator-facing session status label.
    pub fn label(self) -> String {
        match self {
            SessionStatus::Active => "active".into(),
            SessionStatus::Exited(0) => "exited".into(),
            SessionStatus::Exited(code) => format!("exited {code}"),
            SessionStatus::Crashed => "crashed".into(),
        }
    }
}

/// What the daemon reports right now (never derived from the manifest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonSnapshot {
    /// No daemon socket answered.
    Unavailable,
    /// Daemon reports no tabs.
    NoTabs,
    /// Daemon reports one or more live tabs.
    Tabs(Vec<TabSnapshot>),
}

/// Live daemon tab and its panes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSnapshot {
    /// Tab display label.
    pub label: String,
    /// Whether this is the daemon's active tab.
    pub active: bool,
    /// Panes currently reported in the tab.
    pub panes: Vec<PaneSnapshot>,
}

/// Live daemon pane and its agent attention state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSnapshot {
    /// Pane display label.
    pub label: String,
    /// Agent runtime in the pane, when identified.
    pub agent: Option<Agent>,
    /// Current attention state of the pane's agent.
    pub state: AgentState,
    /// Whether the pane currently has focus.
    pub focused: bool,
}

/// Public agent attention state, as the Capsule status bar glyphs encode it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentState {
    /// The agent is idle.
    Idle,
    /// The agent is working.
    Working,
    /// The agent completed its current work.
    Done,
    /// The agent is waiting on an issue or decision.
    Blocked,
    /// The agent state could not be determined.
    Unknown,
}

impl AgentState {
    /// Return the operator-facing state label.
    pub fn label(self) -> &'static str {
        match self {
            AgentState::Idle => "idle",
            AgentState::Working => "working",
            AgentState::Done => "done",
            AgentState::Blocked => "blocked",
            AgentState::Unknown => "unknown",
        }
    }

    /// Glyph priority for a tab that holds several panes.
    pub fn rank(self) -> u8 {
        match self {
            AgentState::Blocked => 4,
            AgentState::Done => 3,
            AgentState::Working => 2,
            AgentState::Idle => 1,
            AgentState::Unknown => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_statuses_and_actions() {
        assert!(InstanceStatus::Purged.hidden());
        assert!(InstanceStatus::Superseded.hidden());
        assert!(!InstanceStatus::Crashed.hidden());
        assert!(InstanceStatus::Running.stoppable());
        assert!(!InstanceStatus::CleanExited.stoppable());
        assert!(InstanceStatus::PreservedUnpushed.reconnectable());
        assert!(!InstanceStatus::FailedSetup.reconnectable());
        assert_eq!(AgentState::Blocked.rank(), 4);
    }
}
