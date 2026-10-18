#![doc = r" Jackin domain: Workspaces, Roles, mounts, environments, Auth, instances,"]
#![doc = r" sessions, accounts, Usage. Plain data; no rendering, no services."]
#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

#[doc = r" Account identities, credential sources, and registry rules."]
pub mod account;
#[doc = r" Agent runtimes, providers, and authentication modes."]
pub mod agent;
#[doc = r" Persisted instance, session, daemon, and pane models."]
pub mod instance;
#[doc = r" Public 1Password reference metadata."]
pub mod onepassword;
#[doc = r" Deterministic scenario contract: fixture world, motion, capture frames."]
pub mod scenario;
#[doc = r" Usage windows, freshness, and roll-up summaries."]
pub mod usage;
#[doc = r" Workspace, role, mount, and environment models."]
pub mod workspace;

#[doc = r" Captured workspace save identities and results."]
pub mod workspace_save;

pub use account::{
    Account, AccountId, AccountIdentity, AccountRegistry, Confidence, CredentialSource,
    DetectedKind, IdentitySubject, IssueCode, Lifecycle, Provenance, Recoverability,
    RecoverableIssue, ValidationLevel, ValidationState,
};
pub use agent::{Agent, Provider, UsageSurface};
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
