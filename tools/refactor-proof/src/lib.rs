//! Shared JSON schemas and canonical serialization for tc-proof tools.

pub mod host;
pub mod observer;

use serde_json::{Map, Value};

/// Serialize JSON with sorted object keys and compact separators (no trailing newline).
pub fn canonical_json(value: &Value) -> String {
    let sorted = sort_value(value);
    serde_json::to_string(&sorted).expect("canonical JSON serialization")
}

fn sort_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), sort_value(&map[key]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
        _ => value.clone(),
    }
}

/// SHA-256 hex digest of canonical JSON bytes.
pub fn context_sha256(value: &Value) -> String {
    use sha2::{Digest, Sha256};
    let bytes = canonical_json(value);
    format!("{:x}", Sha256::digest(bytes.as_bytes()))
}
