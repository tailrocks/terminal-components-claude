//! Bounded Showcase inventory and review bookkeeping.
//!
//! The registry tracks a declared syntax subset across 20 library crates.
//! Page/title leads do not prove behavior. The small `PROOFS` list records
//! only explicit current-head examples; all other items retain an SC roadmap
//! or pending review disposition. No full-public-API or full-behavior claim
//! is made by this module.

pub mod catalog;
pub mod declarations;
pub mod scan;
