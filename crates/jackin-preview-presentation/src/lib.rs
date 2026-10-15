//! `jackin-preview-presentation`
//!
//! Shared screen prelude state and deterministic atmosphere rendering.

#![forbid(unsafe_code)]

#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

/// Workspace creation prelude state shared by host and capsule hosts.
pub mod prelude;
/// Deterministic Jackin atmosphere and entry/exit timing.
pub mod rain;

pub use prelude::*;
pub use rain::*;
