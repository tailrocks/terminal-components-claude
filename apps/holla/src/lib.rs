//! Holla's deterministic application model and simulation boundary.
//!
//! Domain operations modify explicit in-memory fixtures. This package has no
//! filesystem, network, process, or terminal dependency. Production screens and
//! the public executable are migrated separately onto the shared UI runtime.
#![forbid(unsafe_code)]

mod clock;
mod domain;
mod scenario;
mod sim;
