//! Shared JSON schemas and canonical serialization for tc-proof tools.

#![allow(missing_docs)]

pub mod compare;
pub mod json_util;
pub mod observer;

pub use json_util::{canonical_json, sha256_bytes, sha256_canonical};
