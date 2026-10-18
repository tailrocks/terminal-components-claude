#![doc = r" Durable instance records, persisted session records, and live daemon"]
#![doc = r" snapshots. The three are kept apart on purpose: a persisted record can"]
#![doc = r" outlive its daemon, and a daemon snapshot is never invented from records."]

use core::fmt;

use super::agent::Agent;
use super::workspace::WorkspaceId;

#[doc = r" Stable identifier for a durable instance record."]
pub type InstanceId = String;

#[doc = r" Stable numeric identity for one launch attempt."]
#[doc = r""]
#[doc = r" Keeping this separate from instance and workspace names prevents the"]
#[doc = r" capsule from accidentally treating arbitrary display text as a run id."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(u64);

impl RunId {
    #[doc = r" Construct a fixture or persisted run identity."]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[doc = r" The numeric identity."]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[doc = r" Construct a stable identity from arbitrary fixture text."]
    #[doc = r""]
    #[doc = r" The input is hashed instead of sliced or copied into a display token,"]
    #[doc = r" so short and malformed producer values remain total."]
    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in label.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        Self(hash)
    }

    #[doc = r" An eight-hex-digit display form. The full value remains in the typed"]
    #[doc = r" field; this compact projection is for rows and status text only."]
    pub fn short(self) -> String {
        format!("{:08x}", self.0 & u64::from(u32::MAX))
    }

    #[doc = r" The public short container identifier used by Capsule diagnostics."]
    pub fn container_uid(self) -> String {
        format!("3f9c{}e21a", self.short())
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run-{}", self.short())
    }
}

#[doc = r" Lifecycle state of a durable instance."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceStatus {
    #[doc = r" The container and daemon are running."]
    Running,
    #[doc = r" The final session exited successfully."]
    CleanExited,
    #[doc = r" The daemon stopped unexpectedly."]
    Crashed,
    #[doc = r" The stopped instance retains uncommitted changes."]
    PreservedDirty,
    #[doc = r" The stopped instance retains unpushed commits."]
    PreservedUnpushed,
    #[doc = r" The instance has a restorable Capsule layout."]
    RestoreAvailable,
    #[doc = r" A newer instance replaced this one."]
    Superseded,
    #[doc = r" The instance and its records were removed."]
    Purged,
    #[doc = r" Setup failed before the Capsule was ready."]
    FailedSetup,
}

impl InstanceStatus {
    #[doc = r" Compact lifecycle label shown in the tree row."]
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

    #[doc = r" Return the full operator-facing lifecycle description."]
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

    #[doc = r" Return whether this status is hidden from the normal tree."]
    #[doc = r" Hidden from the normal tree, as in current Jackin."]
    pub fn hidden(self) -> bool {
        matches!(self, InstanceStatus::Superseded | InstanceStatus::Purged)
    }

    #[doc = r" Return whether the instance is currently running."]
    pub fn is_live(self) -> bool {
        self == InstanceStatus::Running
    }

    #[doc = r" Return whether the instance can be reconnected to or restored."]
    #[doc = r" Reconnect/restore is offered for these."]
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

    #[doc = r" Return whether stopping the instance is offered."]
    #[doc = r" Stop is offered only for live instances."]
    pub fn stoppable(self) -> bool {
        self == InstanceStatus::Running
    }

    #[doc = r" Return whether the instance retains uncommitted or unpushed work."]
    pub fn dirty(self) -> bool {
        matches!(
            self,
            InstanceStatus::PreservedDirty | InstanceStatus::PreservedUnpushed
        )
    }
}

#[doc = r" Durable instance record and its persisted/live projections."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    #[doc = r" Stable instance identifier."]
    pub id: InstanceId,
    #[doc = r" Container base name; the container id appends the instance suffix."]
    pub container: String,
    #[doc = r" Workspace associated with the instance, when known."]
    pub workspace: Option<WorkspaceId>,
    #[doc = r" Working directory mounted for the instance."]
    pub workdir: String,
    #[doc = r" Role selected for the instance."]
    pub role: String,
    #[doc = r" Agent runtime launched in the instance."]
    pub agent: Agent,
    #[doc = r" Current durable lifecycle status."]
    pub status: InstanceStatus,
    #[doc = r" Creation time in fixture seconds."]
    pub created_secs: i64,
    #[doc = r" Most recent observation time in fixture seconds."]
    pub last_seen_secs: i64,
    #[doc = r" Stable identity for the launch attempt."]
    pub run_id: RunId,
    #[doc = r" Persisted session records (manifest)."]
    pub sessions: Result<Vec<SessionRecord>, ManifestError>,
    #[doc = r" Live daemon snapshot; independent from the manifest."]
    pub daemon: DaemonSnapshot,
    #[doc = r" Branch / PR context resolved for this instance."]
    pub branch: Option<String>,
    #[doc = r" Pull request number and title, when one is associated."]
    pub pr: Option<(u32, String)>,
    #[doc = r" Default branch of the associated repository."]
    pub default_branch: String,
    #[doc = r" Uncommitted / unpushed simulated git state."]
    pub uncommitted: usize,
    #[doc = r" Number of commits not pushed to the remote."]
    pub unpushed: usize,
    #[doc = r" The Workspace's effective account set at launch: every account the"]
    #[doc = r" container can hand to a session, not just the one that started it."]
    pub accounts: Vec<super::account::AccountId>,
}

impl Instance {
    #[doc = r" The public short container id shown by Capsule's debug information."]
    pub fn container_uid(&self) -> String {
        self.run_id.container_uid()
    }

    #[doc = r" Compute the full container identifier from its base name and instance id."]
    pub fn container_id(&self) -> String {
        format!("{}-{}", self.container, self.id.trim_start_matches("jk-"))
    }

    #[doc = r" Return whether the instance has uncommitted or unpushed work."]
    pub fn is_dirty(&self) -> bool {
        self.uncommitted > 0 || self.unpushed > 0
    }

    #[doc = r" Return a compact summary of the instance's git state."]
    pub fn dirty_summary(&self) -> String {
        match (self.uncommitted, self.unpushed) {
            (0, 0) => "clean".into(),
            (u, 0) => format!("{u} uncommitted"),
            (0, p) => format!("{p} unpushed"),
            (u, p) => format!("{u} uncommitted · {p} unpushed"),
        }
    }
}

#[doc = r" Error state for reading persisted session records."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    #[doc = r" The session manifest could not be read."]
    ReadError,
}

impl ManifestError {
    #[doc = r" Return the operator-facing error label."]
    pub fn label(self) -> &'static str {
        match self {
            ManifestError::ReadError => "Sessions unavailable (manifest read error)",
        }
    }
}

#[doc = r" One persisted session entry from an instance manifest."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    #[doc = r" Stable session identifier."]
    pub id: String,
    #[doc = r" Agent runtime in the session, when identified."]
    pub agent: Option<Agent>,
    #[doc = r" Operator-facing session label."]
    pub label: String,
    #[doc = r" Current persisted session status."]
    pub status: SessionStatus,
    #[doc = r" Session start time in fixture seconds."]
    pub started_secs: i64,
}

#[doc = r" Persisted outcome of a session."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    #[doc = r" The session is still active."]
    Active,
    #[doc = r" The session exited with the given process code."]
    Exited(i32),
    #[doc = r" The session ended without a normal exit code."]
    Crashed,
}

impl SessionStatus {
    #[doc = r" Return the operator-facing session status label."]
    pub fn label(self) -> String {
        match self {
            SessionStatus::Active => "active".into(),
            SessionStatus::Exited(0) => "exited".into(),
            SessionStatus::Exited(code) => format!("exited {code}"),
            SessionStatus::Crashed => "crashed".into(),
        }
    }
}

#[doc = r" What the daemon reports right now (never derived from the manifest)."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonSnapshot {
    #[doc = r" No daemon socket answered."]
    Unavailable,
    #[doc = r" Daemon reports no tabs."]
    NoTabs,
    #[doc = r" Daemon reports one or more live tabs."]
    Tabs(Vec<TabSnapshot>),
}

#[doc = r" Live daemon tab and its panes."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSnapshot {
    #[doc = r" Tab display label."]
    pub label: String,
    #[doc = r" Whether this is the daemon's active tab."]
    pub active: bool,
    #[doc = r" Panes currently reported in the tab."]
    pub panes: Vec<PaneSnapshot>,
}

#[doc = r" Live daemon pane and its agent attention state."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSnapshot {
    #[doc = r" Pane display label."]
    pub label: String,
    #[doc = r" Agent runtime in the pane, when identified."]
    pub agent: Option<Agent>,
    #[doc = r" Current attention state of the pane's agent."]
    pub state: AgentState,
    #[doc = r" Whether the pane currently has focus."]
    pub focused: bool,
}

#[doc = r" Public agent attention state, as the Capsule status bar glyphs encode it."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentState {
    #[doc = r" The agent is idle."]
    Idle,
    #[doc = r" The agent is working."]
    Working,
    #[doc = r" The agent completed its current work."]
    Done,
    #[doc = r" The agent is waiting on an issue or decision."]
    Blocked,
    #[doc = r" The agent state could not be determined."]
    Unknown,
}

impl AgentState {
    #[doc = r" Return the operator-facing state label."]
    pub fn label(self) -> &'static str {
        match self {
            AgentState::Idle => "idle",
            AgentState::Working => "working",
            AgentState::Done => "done",
            AgentState::Blocked => "blocked",
            AgentState::Unknown => "unknown",
        }
    }

    #[doc = r" Glyph priority for a tab that holds several panes."]
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
