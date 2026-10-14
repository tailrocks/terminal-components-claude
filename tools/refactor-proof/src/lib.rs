//! Shared JSON schemas and canonical serialization for tc-proof tools.

#![allow(missing_docs)]
// Non-shipped proof CLI. Workspace denies encode the TUI library contract.
#![allow(
    unreachable_pub,
    missing_debug_implementations,
    clippy::pedantic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::undocumented_unsafe_blocks,
    clippy::arithmetic_side_effects,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

pub mod compare;
pub mod host;
pub mod json_util;
pub mod observer;

pub use json_util::{canonical_json, sha256_bytes, sha256_canonical};
