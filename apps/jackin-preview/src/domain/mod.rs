//! Jackin domain: Workspaces, Roles, mounts, environments, Auth, instances,
//! sessions, accounts, Usage. Plain data; no rendering, no services.

// These public fixture records are intentionally data-shaped for integration
// tests; field-by-field rustdoc would add noise without documenting behavior.
#![expect(
    missing_docs,
    reason = "public deterministic fixture records are the integration-test contract"
)]

pub mod account;
pub mod agent;
pub mod fixtures;
pub mod instance;
pub mod onepassword;
pub mod usage;
pub mod workspace;
