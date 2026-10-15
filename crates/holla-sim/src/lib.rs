//! Deterministic simulation world and virtual filesystem matching oracle baselines.
#![forbid(unsafe_code)]

pub mod catalog;
pub mod cleanup;
pub mod fixtures;
pub mod fs;
pub mod outcomes;
pub mod parity;
pub mod world;

pub use fixtures::world_for;
pub use fs::Fs;
pub use world::World;

/// `2 min 14 s` from ticks.
pub fn ticks_label(ticks: u64) -> String {
    holla_domain::clock::format_duration(ticks * world::TICK_MS as u64 / 1000)
}
