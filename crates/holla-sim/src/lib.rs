//! In-memory simulation services. The launcher never spawns a real process:
//! mise, git, gh, docker, btm, `pg_activity`, ssh and the filesystem are
//! deterministic fixtures that evolve only on virtual-clock ticks.
//!
//! The fixture clock, the scenario contract and the per-scenario fixture
//! worlds (`fixtures`) live here so the world, the fixtures and the UI can
//! share one dependency direction: holla-ui depends on holla-sim.

pub mod catalogue;
pub mod clock;
pub mod fixtures;
pub mod pg;
pub mod plans;
pub mod scenario;
pub mod world;

pub use fixtures::world_for;
pub use scenario::{Motion, Scenario};
pub use world::World;

// Compatibility view; the simulation code keeps its historical
// `crate::domain` paths (plan and ranking live in their own crates).
pub mod domain {
    pub use crate::fixtures;
    pub use holla_catalog::ranking;
    pub use holla_domain::*;
    pub use holla_plan::plan;
}
