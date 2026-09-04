//! Deterministic in-memory services: the world, 1Password, provider
//! operations, and the launch pipeline.

// These public simulators expose their state so integration tests can assert
// deterministic transitions without depending on private implementation.
#![expect(
    missing_docs,
    reason = "public deterministic simulators are the integration-test contract"
)]

pub mod changes;
pub mod launch;
pub mod onepassword;
pub mod provider;
pub mod pty;
pub mod world;
