//! `jackin-preview-domain`
//!
//! Connection, auth, service catalog, diagnostic types, pure state logic.

#![forbid(unsafe_code)]

pub mod account;
pub mod agent;
pub mod clock;
pub mod instance;
pub mod onepassword;
pub mod scenario;
pub mod usage;
pub mod workspace;
pub mod workspace_save;

pub use account::{
    Account, AccountId, AccountIdentity, AccountRegistry, Confidence, CredentialSource,
    DetectedKind, IdentitySubject, IssueCode, Lifecycle, Provenance, Recoverability,
    RecoverableIssue, ValidationLevel, ValidationState,
};
pub use agent::{Agent, Provider, UsageSurface};
pub use clock::{Clock, EPOCH_SECS};
pub use instance::{
    AgentState, DaemonSnapshot, Instance, InstanceId, InstanceStatus, PaneSnapshot, RunId,
    SessionRecord, SessionStatus, TabSnapshot,
};
pub use onepassword::OpReference;
pub use scenario::{Motion, Scenario};
pub use usage::{
    AccountUsage, ComparableRollup, Freshness, FreshnessInfo, HealthWord, NotComparableNote,
    OverallCounts, OverallSummary, QuotaStatus, QuotaWindow, WindowCategory, WindowUnit,
};
pub use workspace::{
    AllowedRoles, DirtyExitPolicy, EnvValue, EnvVar, Mount, MountScope, RoleEntry, RolePolicy,
    RoleSource, Workspace, WorkspaceId,
};
pub use workspace_save::{PendingWrite, SaveError, SaveResult, SaveTicket};
