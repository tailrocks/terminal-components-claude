//! Jackin domain: Workspaces, Roles, mounts, environments, Auth, instances,
//! sessions, accounts, Usage. Plain data; no rendering, no services.

/// Account identities, credential sources, and registry rules.
pub mod account;
/// Agent runtimes, providers, and authentication modes.
pub mod agent;
/// Deterministic scenario data and precedence fixtures.
pub mod fixtures;
/// Persisted instance, session, daemon, and pane models.
pub mod instance;
/// Public 1Password reference metadata.
pub mod onepassword;
/// Usage windows, freshness, and roll-up summaries.
pub mod usage;
/// Workspace, role, mount, and environment models.
pub mod workspace;
