//! Frozen context index loading and binding checks for `verify`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::json_util::{parse_json_bytes_strict, require_str, sha256_bytes};

pub(super) const CONTEXT_INDEX_PATH_ENV: &str = "TC_PROOF_CONTEXT_INDEX";
pub(super) const CONTEXT_INDEX_SHA256_ENV: &str = "TC_PROOF_CONTEXT_INDEX_SHA256";

const CONTEXT_INDEX_SCHEMA: &str = "tc-proof-context-index/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ContextIndex {
    pub schema: ContextIndexSchema,
    pub run_id: String,
    pub task_id: String,
    pub tree: String,
    pub trust_sha256: String,
    pub members: Vec<ContextMember>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum ContextIndexSchema {
    #[serde(rename = "tc-proof-context-index/v1")]
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ContextMember {
    pub check_id: String,
    pub context_path: String,
    pub context_sha256: String,
    pub schema: String,
    pub operation: String,
    pub lane: Option<String>,
    pub namespace: Option<String>,
    pub required_ids: Vec<String>,
    pub output_id: String,
}

pub(super) fn load_bound_context_index(run_dir: &Path) -> Result<ContextIndex, &'static str> {
    let index_path = std::env::var(CONTEXT_INDEX_PATH_ENV).map_err(|_| "integrity")?;
    let expected_digest = std::env::var(CONTEXT_INDEX_SHA256_ENV).map_err(|_| "integrity")?;
    let index_path = PathBuf::from(index_path);
    let expected_run_path = run_dir.join("context-index.json");
    if index_path != expected_run_path {
        return Err("integrity");
    }
    let bytes = fs::read(&index_path).map_err(|_| "integrity")?;
    if sha256_bytes(&bytes) != expected_digest {
        return Err("integrity");
    }
    let value = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
    let index = parse_context_index(&value)?;
    validate_members(run_dir, &index)?;
    Ok(index)
}

fn parse_context_index(value: &Value) -> Result<ContextIndex, &'static str> {
    match value.get("schema").and_then(Value::as_str) {
        Some(CONTEXT_INDEX_SCHEMA) => {}
        _ => return Err("integrity"),
    }
    let run_id = require_str(value, "run_id").map_err(|_| "integrity")?;
    let task_id = require_str(value, "task_id").map_err(|_| "integrity")?;
    let tree = require_str(value, "tree").map_err(|_| "integrity")?;
    let trust_sha256 = require_str(value, "trust_sha256").map_err(|_| "integrity")?;
    let members_value = value
        .get("members")
        .and_then(Value::as_array)
        .ok_or("integrity")?;
    let mut members = Vec::with_capacity(members_value.len());
    for member in members_value {
        members.push(parse_context_member(member)?);
    }
    Ok(ContextIndex {
        schema: ContextIndexSchema::V1,
        run_id,
        task_id,
        tree,
        trust_sha256,
        members,
    })
}

fn parse_context_member(value: &Value) -> Result<ContextMember, &'static str> {
    Ok(ContextMember {
        check_id: require_str(value, "check_id").map_err(|_| "integrity")?,
        context_path: require_str(value, "context_path").map_err(|_| "integrity")?,
        context_sha256: require_str(value, "context_sha256").map_err(|_| "integrity")?,
        schema: require_str(value, "schema").map_err(|_| "integrity")?,
        operation: require_str(value, "operation").map_err(|_| "integrity")?,
        lane: optional_string(value, "lane")?,
        namespace: optional_string(value, "namespace")?,
        required_ids: value
            .get("required_ids")
            .and_then(Value::as_array)
            .ok_or("integrity")?
            .iter()
            .map(|item| item.as_str().map(str::to_owned).ok_or("integrity"))
            .collect::<Result<_, _>>()?,
        output_id: require_str(value, "output_id").map_err(|_| "integrity")?,
    })
}

fn optional_string(value: &Value, key: &str) -> Result<Option<String>, &'static str> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err("integrity"),
    }
}

fn validate_members(run_dir: &Path, index: &ContextIndex) -> Result<(), &'static str> {
    let contexts_dir = run_dir.join("contexts");
    if !contexts_dir.is_dir() {
        return Err("integrity");
    }
    let mut seen_paths = std::collections::HashSet::new();
    for member in &index.members {
        if !seen_paths.insert(member.context_path.clone()) {
            return Err("integrity");
        }
        let context_path = run_dir.join(&member.context_path);
        if !context_path.is_file() {
            return Err("integrity");
        }
        let bytes = fs::read(&context_path).map_err(|_| "integrity")?;
        if sha256_bytes(&bytes) != member.context_sha256 {
            return Err("integrity");
        }
        let child = parse_json_bytes_strict(&bytes).map_err(|_| "integrity")?;
        if require_str(&child, "schema").map_err(|_| "integrity")? != member.schema {
            return Err("integrity");
        }
        if require_str(&child, "run_id").map_err(|_| "integrity")? != index.run_id {
            return Err("integrity");
        }
        if require_str(&child, "task_id").map_err(|_| "integrity")? != index.task_id {
            return Err("integrity");
        }
        if require_str(&child, "tree").map_err(|_| "integrity")? != index.tree {
            return Err("integrity");
        }
        if require_str(&child, "check_id").map_err(|_| "integrity")? != member.check_id {
            return Err("integrity");
        }
        if require_str(&child, "operation").map_err(|_| "integrity")? != member.operation {
            return Err("integrity");
        }
        if optional_string(&child, "lane")? != member.lane {
            return Err("integrity");
        }
        if optional_string(&child, "namespace")? != member.namespace {
            return Err("integrity");
        }
    }
    let expected: std::collections::HashSet<_> = index
        .members
        .iter()
        .map(|member| member.context_path.clone())
        .collect();
    let actual: std::collections::HashSet<_> = fs::read_dir(&contexts_dir)
        .map_err(|_| "integrity")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| format!("contexts/{}", entry.file_name().to_string_lossy()))
        .collect();
    if expected != actual {
        return Err("integrity");
    }
    Ok(())
}
