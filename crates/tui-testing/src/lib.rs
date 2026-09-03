//! Test support for `tui-next` (`COMPONENT_ARCHITECTURE.md` §16): the
//! deterministic [`Harness`], the headless digest [`Scene`], the shared
//! conformance driver and the counting allocator for the perf suite.
//!
//! Dev-only: `publish = false`, depended on with `[dev-dependencies]` only,
//! so nothing here reaches a shipped binary.
#![deny(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::arithmetic_side_effects,
    clippy::cast_lossless,
    clippy::many_single_char_names,
    clippy::field_reassign_with_default,
    clippy::struct_excessive_bools,
    clippy::type_complexity,
    clippy::print_stdout,
    clippy::format_push_string,
    reason = "a test-support crate: assertions and indexing are the product, not a hazard"
)]

pub mod conformance;
pub mod digest;
pub mod harness;
pub mod perf;

pub use conformance::{Caps, Conformance, Fixture, FixtureRow};
pub use digest::{Baseline, NoApp, Scene};
pub use harness::Harness;
