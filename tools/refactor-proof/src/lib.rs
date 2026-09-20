//! Shared JSON schemas and canonical serialization for tc-proof tools.

#![allow(missing_docs)]

pub mod compare;
pub mod json_util;
pub mod observer;
pub mod verifier;

pub use json_util::{canonical_json, sha256_bytes, sha256_canonical};
pub use verifier::validate_run;
