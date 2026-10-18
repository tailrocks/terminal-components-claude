//! Context identity captured before opening a product choice.
use crate::{
    domain::host::{Environment, HostKind},
    sim::world::World,
};
#[derive(PartialEq, Eq)]
pub(super) struct Context {
    host: String,
    kind: HostKind,
    env: Environment,
    cwd: String,
    revision: u64,
}
impl Context {
    pub(super) fn of(world: &World) -> Self {
        Self {
            host: world.host.name.clone(),
            kind: world.host.kind,
            env: world.host.env,
            cwd: world.cwd.clone(),
            revision: world.effect_revision,
        }
    }
}
