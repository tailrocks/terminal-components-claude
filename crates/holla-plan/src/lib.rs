//! Plan dependency rules, activity/output lifecycle, cancellation and target-bound safety decisions.
#![forbid(unsafe_code)]

pub mod activity;
pub mod plan;

pub use activity::*;
pub use plan::*;
