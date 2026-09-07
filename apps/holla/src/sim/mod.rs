//! In-memory simulation services. The launcher never spawns a real process:
//! mise, git, gh, docker, btm, pg_activity, ssh and the filesystem are
//! deterministic fixtures that evolve only on virtual-clock ticks.

pub mod catalogue;
pub mod plans;
pub mod world;
