//! Effects: what a finished run changes in the fixture world. Nothing is
//! pretended — a restarted container heals, a cancelled backend frees its
//! waiters, a cleanup removes what it claimed — so the root experience is
//! truthful after an action, not only before it.

use crate::sim::world::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    DockerRestart(String),
    DockerStop(String),
    ServiceRestart(String),
    PgCancel(u32),
    PgTerminate(u32),
    /// Fast-forward the repository at this path.
    GitPull(String),
    /// Remove the Cargo target artifacts under this path.
    CargoClean(String),
    MiseInstall,
}

impl Effect {
    /// The effect an item's activity has when it succeeds, derived from the
    /// item id so the catalogue never has to spell it twice.
    pub fn for_item(id: &str, w: &World) -> Option<Effect> {
        if let Some(name) = id.strip_prefix("service.restart.") {
            return Some(Effect::ServiceRestart(name.to_owned()));
        }
        if let Some(name) = id.strip_prefix("docker.restart.") {
            return Some(Effect::DockerRestart(name.to_owned()));
        }
        if let Some(name) = id.strip_prefix("docker.stop.") {
            return Some(Effect::DockerStop(name.to_owned()));
        }
        match id {
            "pg.cancel" => w.pg_blocker().map(Effect::PgCancel),
            "pg.terminate" => w.pg_blocker().map(Effect::PgTerminate),
            "git.pull" => w.git_here().map(|g| Effect::GitPull(g.path.clone())),
            "cargo.clean" => Some(Effect::CargoClean(w.location.cwd.clone())),
            "mise.install" => Some(Effect::MiseInstall),
            _ => None,
        }
    }
}
