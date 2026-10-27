#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]

pub mod arbiter;
pub mod changes;
pub mod clock;
pub mod fixtures;
pub mod launch;
pub mod onepassword;
pub mod provider;
pub mod pty;
pub mod world;

pub use world::{GithubRepo, GlobalConfig, TrustRow, World, world_for};
