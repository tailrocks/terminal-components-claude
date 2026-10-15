//! `jackin-preview-presentation`
//!
//! Shared screen prelude state and deterministic atmosphere rendering.

#![forbid(unsafe_code)]

/// Workspace creation prelude state shared by host and capsule hosts.
pub mod prelude;
/// Deterministic Jackin atmosphere and entry/exit timing.
pub mod rain;

pub use prelude::*;
pub use rain::*;
