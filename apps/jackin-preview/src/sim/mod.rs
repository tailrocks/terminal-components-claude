//! Deterministic in-memory services: the world, 1Password, provider
//! operations, and the launch pipeline.

/// Deterministic changed-file and diff fixture model.
pub mod changes;
/// Deterministic launch-pipeline state machine.
pub mod launch;
/// In-memory 1Password account and vault service.
pub mod onepassword;
/// Deterministic credential validation and usage refresh operations.
pub mod provider;
/// Capsule tabs, panes, transcripts, and daemon simulation.
pub mod pty;
/// Virtual clock, jobs, fixtures, and service state.
pub mod world;
