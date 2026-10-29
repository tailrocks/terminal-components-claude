use core::fmt;

use super::agent::Agent;
use super::workspace::WorkspaceId;

pub type InstanceId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(u64);

impl RunId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in label.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        Self(hash)
    }

    pub fn short(self) -> String {
        format!("{:08x}", self.0 & u64::from(u32::MAX))
    }

    pub fn container_uid(self) -> String {
        format!("3f9c{}e21a", self.short())
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run-{}", self.short())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceStatus {
    Running,
    CleanExited,
    Crashed,
    PreservedDirty,
    PreservedUnpushed,
    RestoreAvailable,
    Superseded,
    Purged,
    FailedSetup,
}

impl InstanceStatus {
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

    pub fn hidden(self) -> bool {
        matches!(self, InstanceStatus::Superseded | InstanceStatus::Purged)
    }

    pub fn is_live(self) -> bool {
        self == InstanceStatus::Running
    }

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

    pub fn stoppable(self) -> bool {
        self == InstanceStatus::Running
    }

    pub fn dirty(self) -> bool {
        matches!(
            self,
            InstanceStatus::PreservedDirty | InstanceStatus::PreservedUnpushed
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub id: InstanceId,
    pub container: String,
    pub workspace: Option<WorkspaceId>,
    pub workdir: String,
    pub role: String,
    pub agent: Agent,
    pub status: InstanceStatus,
    pub created_secs: i64,
    pub last_seen_secs: i64,
    pub run_id: RunId,
    pub sessions: Result<Vec<SessionRecord>, ManifestError>,
    pub daemon: DaemonSnapshot,
    pub branch: Option<String>,
    pub pr: Option<(u32, String)>,
    pub default_branch: String,
    pub uncommitted: usize,
    pub unpushed: usize,
    pub accounts: Vec<super::account::AccountId>,
}

impl Instance {
    pub fn container_uid(&self) -> String {
        self.run_id.container_uid()
    }

    pub fn container_id(&self) -> String {
        format!("{}-{}", self.container, self.id.trim_start_matches("jk-"))
    }

    pub fn is_dirty(&self) -> bool {
        self.uncommitted > 0 || self.unpushed > 0
    }

    pub fn dirty_summary(&self) -> String {
        match (self.uncommitted, self.unpushed) {
            (0, 0) => "clean".into(),
            (u, 0) => format!("{u} uncommitted"),
            (0, p) => format!("{p} unpushed"),
            (u, p) => format!("{u} uncommitted · {p} unpushed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    ReadError,
}

impl ManifestError {
    pub fn label(self) -> &'static str {
        match self {
            ManifestError::ReadError => "Sessions unavailable (manifest read error)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub id: String,
    pub agent: Option<Agent>,
    pub label: String,
    pub status: SessionStatus,
    pub started_secs: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Exited(i32),
    Crashed,
}

impl SessionStatus {
    pub fn label(self) -> String {
        match self {
            SessionStatus::Active => "active".into(),
            SessionStatus::Exited(0) => "exited".into(),
            SessionStatus::Exited(code) => format!("exited {code}"),
            SessionStatus::Crashed => "crashed".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonSnapshot {
    Unavailable,
    NoTabs,
    Tabs(Vec<TabSnapshot>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSnapshot {
    pub label: String,
    pub active: bool,
    pub panes: Vec<PaneSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSnapshot {
    pub label: String,
    pub agent: Option<Agent>,
    pub state: AgentState,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentState {
    Idle,
    Working,
    Done,
    Blocked,
    Unknown,
}

impl AgentState {
    pub fn label(self) -> &'static str {
        match self {
            AgentState::Idle => "idle",
            AgentState::Working => "working",
            AgentState::Done => "done",
            AgentState::Blocked => "blocked",
            AgentState::Unknown => "unknown",
        }
    }

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
