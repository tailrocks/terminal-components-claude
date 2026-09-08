//! In-memory simulation services. The launcher never spawns a real process:
//! mise, git, gh, docker, btm, `pg_activity`, ssh and the filesystem are
//! deterministic fixtures that evolve only on virtual-clock ticks.

pub(crate) mod catalogue;
pub(crate) mod plans;
pub(crate) mod world;
