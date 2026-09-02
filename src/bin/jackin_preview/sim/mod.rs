//! Deterministic in-memory services: the world (fixtures + job queue),
//! 1Password, provider operations, the launch pipeline, PTYs and agents.

pub mod changes;
pub mod launch;
pub mod onepassword;
pub mod provider;
pub mod pty;
pub mod world;
