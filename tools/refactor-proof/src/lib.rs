//! Shared JSON schemas and canonical serialization for tc-proof tools.

pub mod compare;
pub mod json_util;

pub use json_util::{canonical_json, sha256_bytes, sha256_canonical};
