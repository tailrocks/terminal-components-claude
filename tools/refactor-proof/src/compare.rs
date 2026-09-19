//! tc-proof compare implementation (Phase 1).

#![expect(
    clippy::print_stderr,
    reason = "comparator reports malformed proof input on stderr"
)]

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use tuisnap::Frame;

use crate::json_util::{
    as_object_map, duplicate_values, get_str, is_safe_relative_path, parse_json_bytes_strict,
    parse_json_strict, require_str, require_str_array, require_u64, sha256_bytes, sha256_canonical,
    values_equal,
};

const CONTEXT_SCHEMA: &str = "tc-proof-compare-context/v1";
const RUNNER_CONTEXT_SCHEMA: &str = "tc-proof-context/v1";
const COMPARISON_SCHEMA: &str = "tc-proof-comparison/v1";
const ARTIFACTS_SCHEMA: &str = "tc-proof-artifacts/v1";
const REQUIRED_SCHEMA: &str = "tc-proof-required/v1";
const PROVENANCE_SCHEMA: &str = "tc-proof-provenance/v1";
const STATE_SCHEMA: &str = "tc-proof-state/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureCode {
    UnsafePath,
    UnexpectedApproval,
    Integrity,
    RequiredSet,
    Provenance,
    FrameInvalid,
    FrameMismatch,
    StateInvalid,
    StateMismatch,
}

impl FailureCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UnsafePath => "UNSAFE_PATH",
            Self::UnexpectedApproval => "UNEXPECTED_APPROVAL",
            Self::Integrity => "INTEGRITY",
            Self::RequiredSet => "REQUIRED_SET",
            Self::Provenance => "PROVENANCE",
            Self::FrameInvalid => "FRAME_INVALID",
            Self::FrameMismatch => "FRAME_MISMATCH",
            Self::StateInvalid => "STATE_INVALID",
            Self::StateMismatch => "STATE_MISMATCH",
        }
    }
}

#[derive(Debug)]
struct CompareContext {
    raw_bytes: Vec<u8>,
    run_id: String,
    task_id: String,
    oracle_commit: String,
    candidate_source_tree: String,
    oracle_root: PathBuf,
    candidate_root: PathBuf,
    oracle_manifest_sha256: String,
    candidate_manifest_sha256: String,
    required_sha256: String,
    actions_sha256: String,
    required_ids: Vec<String>,
    required_count: u64,
    tool_sha256: String,
    oracle_adapter_sha256: String,
    candidate_adapter_sha256: String,
    report_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct ScenarioResult {
    id: String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct FailureRecord {
    id: Option<String>,
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize)]
struct ComparisonReport {
    schema: &'static str,
    run_id: String,
    task_id: String,
    context_sha256: String,
    required_count: u64,
    checked_count: u64,
    passed_count: u64,
    results: Vec<ScenarioResult>,
    failures: Vec<FailureRecord>,
}

struct ScenarioSpec {
    id: String,
    frame: String,
    state: String,
    provenance: String,
    state_keys: Vec<String>,
    lane: String,
    checkpoint: String,
}

struct ArtifactsManifest {
    files: Vec<(String, u64, String)>,
}

enum CompareOutcome {
    Success,
    Global(FailureCode),
    Scenario(Vec<(String, FailureCode)>),
}

/// Execute `tc-proof compare` for the context file at `context_path`.
///
/// # Errors
///
/// Returns an I/O error when the context cannot be read or the comparison
/// report cannot be written.
pub fn run_compare(context_path: &Path) -> io::Result<i32> {
    let raw_bytes = fs::read(context_path)?;
    let context = match parse_context(&raw_bytes) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("tc-proof: invalid context: {error}");
            return Ok(2);
        }
    };

    let outcome = compare_roots(&context);
    let (exit_code, report) = build_report(&context, outcome);
    write_report(&context.report_path, &report)?;
    Ok(exit_code)
}

fn parse_context(raw_bytes: &[u8]) -> Result<CompareContext, String> {
    let text = std::str::from_utf8(raw_bytes).map_err(|error| error.to_string())?;
    let value = parse_json_strict(text)?;
    match get_str(&value, "schema") {
        Some(CONTEXT_SCHEMA) => parse_compare_value(&value, raw_bytes, None),
        Some(RUNNER_CONTEXT_SCHEMA) => parse_runner_value(&value, raw_bytes),
        _ => Err(format!(
            "schema must be {CONTEXT_SCHEMA} or {RUNNER_CONTEXT_SCHEMA}"
        )),
    }
}

fn parse_compare_value(
    value: &Value,
    raw_bytes: &[u8],
    host_report_path: Option<PathBuf>,
) -> Result<CompareContext, String> {
    let oracle_root = qualify_input_root(&PathBuf::from(require_str(value, "oracle_root")?))?;
    let candidate_root = qualify_input_root(&PathBuf::from(require_str(value, "candidate_root")?))?;
    let report_path = match host_report_path {
        Some(path) => qualify_report_path(&path)?,
        None => qualify_report_path(&PathBuf::from(require_str(value, "report_path")?))?,
    };
    validate_report_path(&report_path, &oracle_root, &candidate_root, None)?;
    Ok(CompareContext {
        raw_bytes: raw_bytes.to_vec(),
        run_id: require_str(value, "run_id")?,
        task_id: require_str(value, "task_id")?,
        oracle_commit: require_str(value, "oracle_commit")?,
        candidate_source_tree: require_str(value, "candidate_source_tree")?,
        oracle_root,
        candidate_root,
        oracle_manifest_sha256: require_str(value, "oracle_manifest_sha256")?,
        candidate_manifest_sha256: require_str(value, "candidate_manifest_sha256")?,
        required_sha256: require_str(value, "required_sha256")?,
        actions_sha256: require_str(value, "actions_sha256")?,
        required_ids: require_str_array(value, "required_ids")?,
        required_count: require_u64(value, "required_count")?,
        tool_sha256: require_str(value, "tool_sha256")?,
        oracle_adapter_sha256: require_str(value, "oracle_adapter_sha256")?,
        candidate_adapter_sha256: require_str(value, "candidate_adapter_sha256")?,
        report_path,
    })
}

fn parse_runner_value(value: &Value, raw_bytes: &[u8]) -> Result<CompareContext, String> {
    let run_id = require_str(value, "run_id")?;
    let task_id = require_str(value, "task_id")?;
    let check_id = require_str(value, "check_id")?;
    let oracle_commit = require_str(value, "oracle_commit")?;
    let candidate_source_tree = require_str(value, "tree")?;
    let qualification = value
        .get("qualification")
        .and_then(Value::as_object)
        .ok_or_else(|| "runner context qualification is missing".to_string())?;
    let comparator = qualification
        .get("comparator")
        .and_then(Value::as_object)
        .ok_or_else(|| "runner context comparator binding is missing".to_string())?;
    if get_str(&Value::Object(comparator.clone()), "schema") != Some(CONTEXT_SCHEMA) {
        return Err("runner comparator schema mismatch".to_string());
    }
    let nested = comparator
        .get("context")
        .ok_or_else(|| "runner comparator context is missing".to_string())?;
    nested
        .as_object()
        .ok_or_else(|| "runner comparator context is not an object".to_string())?;
    if get_str(nested, "schema") != Some(CONTEXT_SCHEMA)
        || get_str(nested, "run_id") != Some(run_id.as_str())
        || get_str(nested, "task_id") != Some(task_id.as_str())
        || get_str(nested, "check_id") != Some(check_id.as_str())
        || get_str(nested, "oracle_commit") != Some(oracle_commit.as_str())
        || get_str(nested, "candidate_source_tree") != Some(candidate_source_tree.as_str())
    {
        return Err("runner comparator context identity mismatch".to_string());
    }
    let report_path = qualify_report_path(&PathBuf::from(
        comparator
            .get("report_path")
            .and_then(Value::as_str)
            .ok_or_else(|| "runner comparator report_path is missing".to_string())?,
    ))?;
    if qualify_report_path(&PathBuf::from(require_str(nested, "report_path")?))? != report_path {
        return Err("runner comparator report path mismatch".to_string());
    }
    let common = qualification
        .get("common")
        .and_then(Value::as_object)
        .ok_or_else(|| "runner context common binding is missing".to_string())?;
    let runtime = qualify_runtime_root(&PathBuf::from(
        common
            .get("outputs")
            .and_then(Value::as_object)
            .and_then(|outputs| outputs.get("runtime"))
            .and_then(Value::as_str)
            .ok_or_else(|| "runner runtime output binding is missing".to_string())?,
    ))?;
    let oracle_root = qualify_input_root(&PathBuf::from(require_str(nested, "oracle_root")?))?;
    let candidate_root =
        qualify_input_root(&PathBuf::from(require_str(nested, "candidate_root")?))?;
    validate_report_path(&report_path, &oracle_root, &candidate_root, Some(&runtime))?;
    let expected_report_name = format!("{check_id}.compare.json");
    if report_path.file_name().and_then(|name| name.to_str()) != Some(expected_report_name.as_str())
    {
        return Err("runner comparator report filename is not host-bound".to_string());
    }
    let mut context = parse_compare_value(nested, raw_bytes, Some(report_path))?;
    context.run_id = run_id;
    context.task_id = task_id;
    context.oracle_commit = oracle_commit;
    context.candidate_source_tree = candidate_source_tree;
    Ok(context)
}

fn validate_report_path(
    report_path: &Path,
    oracle_root: &Path,
    candidate_root: &Path,
    runtime_root: Option<&Path>,
) -> Result<(), String> {
    if !report_path.is_absolute()
        || report_path == oracle_root
        || report_path == candidate_root
        || report_path.starts_with(oracle_root)
        || report_path.starts_with(candidate_root)
    {
        return Err("report path is not an external absolute path".to_string());
    }
    if let Some(runtime_root) = runtime_root
        && (report_path.parent() != Some(runtime_root)
            || !runtime_root.is_absolute()
            || runtime_root == oracle_root
            || runtime_root == candidate_root
            || runtime_root.starts_with(oracle_root)
            || runtime_root.starts_with(candidate_root))
    {
        return Err("report path is outside the host-selected runtime output".to_string());
    }
    Ok(())
}

/// Resolve an existing report parent before applying containment checks.
///
/// macOS exposes temporary directories through `/var` symlink ancestry. The
/// qualified path is used only for the report parent; descendant oracle and
/// candidate symlinks remain rejected by the tree scans below.
fn qualify_report_path(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("report path is not an external absolute path".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "report path has no parent".to_string())?;
    let Some(file_name) = path.file_name() else {
        return Err("report path has no filename".to_string());
    };
    if parent.exists() {
        let qualified_parent = parent
            .canonicalize()
            .map_err(|error| format!("report parent realpath failed: {error}"))?;
        if !qualified_parent.is_dir() {
            return Err("report parent is not a directory".to_string());
        }
        return Ok(qualified_parent.join(file_name));
    }
    Ok(path.to_path_buf())
}

fn qualify_input_root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("comparison input root must be absolute".to_string());
    }
    if path.exists() {
        let qualified = path
            .canonicalize()
            .map_err(|error| format!("comparison input realpath failed: {error}"))?;
        if !qualified.is_dir() {
            return Err("comparison input root is not a directory".to_string());
        }
        return Ok(qualified);
    }
    Ok(path.to_path_buf())
}

fn qualify_runtime_root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("runtime output must be an absolute directory".to_string());
    }
    if path.exists() {
        let qualified = path
            .canonicalize()
            .map_err(|error| format!("runtime output realpath failed: {error}"))?;
        if !qualified.is_dir() {
            return Err("runtime output is not a directory".to_string());
        }
        return Ok(qualified);
    }
    Ok(path.to_path_buf())
}

fn compare_roots(context: &CompareContext) -> CompareOutcome {
    if context.required_ids.is_empty()
        || context.required_count as usize != context.required_ids.len()
        || !duplicate_values(&context.required_ids).is_empty()
    {
        return CompareOutcome::Global(FailureCode::Integrity);
    }

    if let Err(code) = check_unexpected_approval(&context.candidate_root) {
        return CompareOutcome::Global(code);
    }

    if let Err(code) = scan_tree_for_symlinks(&context.oracle_root) {
        return CompareOutcome::Global(code);
    }
    if let Err(code) = scan_tree_for_symlinks(&context.candidate_root) {
        return CompareOutcome::Global(code);
    }

    let oracle_manifest = match verify_manifest(
        &context.oracle_root,
        &context.oracle_manifest_sha256,
        &context.required_ids,
        false,
    ) {
        Ok(manifest) => manifest,
        Err(code) => return CompareOutcome::Global(code),
    };

    let candidate_manifest = match verify_manifest(
        &context.candidate_root,
        &context.candidate_manifest_sha256,
        &context.required_ids,
        true,
    ) {
        Ok(manifest) => manifest,
        Err(code) => return CompareOutcome::Global(code),
    };

    if let Err(code) = check_required_set(context, &oracle_manifest, &candidate_manifest) {
        return CompareOutcome::Global(code);
    }

    let scenarios = match load_required_scenarios(&context.oracle_root, &context.required_ids) {
        Ok(scenarios) => scenarios,
        Err(code) => return CompareOutcome::Global(code),
    };

    let mut scenario_failures = Vec::new();
    for scenario in &scenarios {
        if let Some(code) = compare_scenario(context, scenario) {
            scenario_failures.push((scenario.id.clone(), code));
        }
    }

    if scenario_failures.is_empty() {
        CompareOutcome::Success
    } else {
        CompareOutcome::Scenario(scenario_failures)
    }
}

fn check_unexpected_approval(root: &Path) -> Result<(), FailureCode> {
    let approved = root.join("approved");
    if approved.exists() {
        return Err(FailureCode::UnexpectedApproval);
    }
    Ok(())
}

fn scan_tree_for_symlinks(root: &Path) -> Result<(), FailureCode> {
    if !root.is_dir() {
        return Err(FailureCode::Integrity);
    }
    for entry in walk_files(root)? {
        if entry.is_symlink() {
            return Err(FailureCode::UnsafePath);
        }
    }
    Ok(())
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, FailureCode> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|_| FailureCode::Integrity)?;
        for entry in entries {
            let entry = entry.map_err(|_| FailureCode::Integrity)?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn verify_manifest(
    root: &Path,
    expected_hash: &str,
    required_ids: &[String],
    allow_extra_files: bool,
) -> Result<ArtifactsManifest, FailureCode> {
    let manifest_path = root.join("manifest.json");
    let bytes = fs::read(&manifest_path).map_err(|_| FailureCode::Integrity)?;
    if sha256_bytes(&bytes) != expected_hash {
        return Err(FailureCode::Integrity);
    }

    let text = std::str::from_utf8(&bytes).map_err(|_| FailureCode::Integrity)?;
    let value = parse_json_strict(text).map_err(|_| FailureCode::Integrity)?;
    if get_str(&value, "schema") != Some(ARTIFACTS_SCHEMA) {
        return Err(FailureCode::Integrity);
    }

    let scenario_ids = value
        .get("scenario_ids")
        .and_then(Value::as_array)
        .ok_or(FailureCode::Integrity)?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or(FailureCode::Integrity)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if !duplicate_values(&scenario_ids).is_empty() {
        return Err(FailureCode::RequiredSet);
    }
    if scenario_ids != required_ids {
        return Err(FailureCode::RequiredSet);
    }

    let files_array = value
        .get("files")
        .and_then(Value::as_array)
        .ok_or(FailureCode::Integrity)?;
    let mut files = Vec::new();
    let mut paths = Vec::new();
    for entry in files_array {
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or(FailureCode::Integrity)?;
        if !is_safe_relative_path(path) {
            return Err(FailureCode::UnsafePath);
        }
        let size = entry
            .get("size")
            .and_then(Value::as_u64)
            .ok_or(FailureCode::Integrity)?;
        let hash = entry
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or(FailureCode::Integrity)?;
        paths.push(path.to_owned());
        files.push((path.to_owned(), size, hash.to_owned()));
    }

    if !duplicate_values(&paths).is_empty() {
        return Err(FailureCode::RequiredSet);
    }

    for (path, size, hash) in &files {
        let full = root.join(path);
        if full.is_symlink() {
            return Err(FailureCode::UnsafePath);
        }
        let data = fs::read(&full).map_err(|_| FailureCode::Integrity)?;
        if data.len() as u64 != *size || sha256_bytes(&data) != *hash {
            return Err(FailureCode::Integrity);
        }
    }

    if allow_extra_files {
        let manifest_paths: HashSet<_> = paths.iter().collect();
        for file in walk_files(root)? {
            let rel = file
                .strip_prefix(root)
                .map_err(|_| FailureCode::Integrity)?
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "manifest.json" {
                continue;
            }
            if !manifest_paths.contains(&rel) {
                return Err(FailureCode::Integrity);
            }
        }
    }

    Ok(ArtifactsManifest { files })
}

fn check_required_set(
    context: &CompareContext,
    oracle_manifest: &ArtifactsManifest,
    candidate_manifest: &ArtifactsManifest,
) -> Result<(), FailureCode> {
    let required_path = context.oracle_root.join("required.json");
    let required_bytes = fs::read(&required_path).map_err(|_| FailureCode::Integrity)?;
    if sha256_canonical(
        &parse_json_bytes_strict(&required_bytes).map_err(|_| FailureCode::Integrity)?,
    ) != context.required_sha256
    {
        return Err(FailureCode::Integrity);
    }

    let required = parse_json_bytes_strict(&required_bytes).map_err(|_| FailureCode::Integrity)?;
    if get_str(&required, "schema") != Some(REQUIRED_SCHEMA) {
        return Err(FailureCode::Integrity);
    }

    let scenarios = required
        .get("scenarios")
        .and_then(Value::as_array)
        .ok_or(FailureCode::Integrity)?;

    if scenarios.len() != context.required_ids.len() {
        return Err(FailureCode::Integrity);
    }

    let mut ids = Vec::new();
    for scenario in scenarios {
        ids.push(require_str(scenario, "id").map_err(|_| FailureCode::Integrity)?);
    }
    if ids != context.required_ids {
        return Err(FailureCode::Integrity);
    }

    let expected_candidate_paths = expected_artifact_paths(&required);
    let candidate_paths: HashSet<_> = candidate_manifest
        .files
        .iter()
        .map(|(path, _, _)| path.clone())
        .collect();
    if candidate_paths != expected_candidate_paths {
        return Err(FailureCode::RequiredSet);
    }

    let oracle_paths: HashSet<_> = oracle_manifest
        .files
        .iter()
        .map(|(path, _, _)| path.clone())
        .collect();
    let expected_oracle_paths = expected_oracle_paths(&required);
    if oracle_paths != expected_oracle_paths {
        return Err(FailureCode::Integrity);
    }

    Ok(())
}

fn expected_artifact_paths(required: &Value) -> HashSet<String> {
    let mut paths = HashSet::from(["binary.bin".to_owned()]);
    if let Some(scenarios) = required.get("scenarios").and_then(Value::as_array) {
        for scenario in scenarios {
            for key in ["frame", "state", "provenance"] {
                if let Some(path) = scenario.get(key).and_then(Value::as_str) {
                    paths.insert(path.to_owned());
                }
            }
        }
    }
    paths
}

fn expected_oracle_paths(required: &Value) -> HashSet<String> {
    let mut paths = expected_artifact_paths(required);
    paths.insert("actions.json".to_owned());
    paths.insert("font.bin".to_owned());
    paths.insert("profile.json".to_owned());
    paths.insert("required.json".to_owned());
    paths
}

fn load_required_scenarios(
    oracle_root: &Path,
    required_ids: &[String],
) -> Result<Vec<ScenarioSpec>, FailureCode> {
    let required_bytes =
        fs::read(oracle_root.join("required.json")).map_err(|_| FailureCode::Integrity)?;
    let required = parse_json_bytes_strict(&required_bytes).map_err(|_| FailureCode::Integrity)?;
    let scenarios = required
        .get("scenarios")
        .and_then(Value::as_array)
        .ok_or(FailureCode::Integrity)?;

    let mut ordered = Vec::new();
    let by_id = scenarios
        .iter()
        .map(|scenario| {
            let id = require_str(scenario, "id").map_err(|_| FailureCode::Integrity)?;
            Ok((id, scenario))
        })
        .collect::<Result<HashMap<_, _>, FailureCode>>()?;

    for id in required_ids {
        let scenario = by_id.get(id).ok_or(FailureCode::Integrity)?;
        ordered.push(ScenarioSpec {
            id: id.clone(),
            frame: require_str(scenario, "frame").map_err(|_| FailureCode::Integrity)?,
            state: require_str(scenario, "state").map_err(|_| FailureCode::Integrity)?,
            provenance: require_str(scenario, "provenance").map_err(|_| FailureCode::Integrity)?,
            state_keys: scenario
                .get("state_keys")
                .and_then(Value::as_array)
                .ok_or(FailureCode::Integrity)?
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_owned)
                        .ok_or(FailureCode::Integrity)
                })
                .collect::<Result<_, _>>()?,
            lane: require_str(scenario, "lane").map_err(|_| FailureCode::Integrity)?,
            checkpoint: require_str(scenario, "checkpoint").map_err(|_| FailureCode::Integrity)?,
        });
    }
    Ok(ordered)
}

fn compare_scenario(context: &CompareContext, scenario: &ScenarioSpec) -> Option<FailureCode> {
    let oracle_frame_path = context.oracle_root.join(&scenario.frame);
    let candidate_frame_path = context.candidate_root.join(&scenario.frame);
    let oracle_state_path = context.oracle_root.join(&scenario.state);
    let candidate_state_path = context.candidate_root.join(&scenario.state);
    let oracle_prov_path = context.oracle_root.join(&scenario.provenance);
    let candidate_prov_path = context.candidate_root.join(&scenario.provenance);

    for path in [
        &oracle_frame_path,
        &candidate_frame_path,
        &oracle_state_path,
        &candidate_state_path,
        &oracle_prov_path,
        &candidate_prov_path,
    ] {
        if path.is_symlink() {
            return Some(FailureCode::UnsafePath);
        }
        if !path.is_file() {
            return Some(FailureCode::Integrity);
        }
    }

    if let Err(code) = check_provenance(context, scenario, &oracle_prov_path, true) {
        return Some(code);
    }
    if let Err(code) = check_provenance(context, scenario, &candidate_prov_path, false) {
        return Some(code);
    }

    match compare_frames(&oracle_frame_path, &candidate_frame_path) {
        Ok(true) => {}
        Ok(false) => return Some(FailureCode::FrameMismatch),
        Err(FailureCode::FrameInvalid) => return Some(FailureCode::FrameInvalid),
        Err(other) => return Some(other),
    }

    match compare_states(
        &oracle_state_path,
        &candidate_state_path,
        &scenario.state_keys,
    ) {
        Ok(true) => None,
        Ok(false) => Some(FailureCode::StateMismatch),
        Err(FailureCode::StateInvalid) => Some(FailureCode::StateInvalid),
        Err(other) => Some(other),
    }
}

fn check_provenance(
    context: &CompareContext,
    scenario: &ScenarioSpec,
    path: &Path,
    is_oracle: bool,
) -> Result<(), FailureCode> {
    let bytes = fs::read(path).map_err(|_| FailureCode::Integrity)?;
    let value = parse_json_bytes_strict(&bytes).map_err(|_| FailureCode::Integrity)?;
    if get_str(&value, "schema") != Some(PROVENANCE_SCHEMA) {
        return Err(FailureCode::Provenance);
    }

    let expected_source = if is_oracle {
        context.oracle_commit.clone()
    } else {
        context.candidate_source_tree.clone()
    };
    if require_str(&value, "source_tree").map_err(|_| FailureCode::Provenance)? != expected_source {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "scenario_id").map_err(|_| FailureCode::Provenance)? != scenario.id {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "lane").map_err(|_| FailureCode::Provenance)? != scenario.lane {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "checkpoint").map_err(|_| FailureCode::Provenance)?
        != scenario.checkpoint
    {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "binary_path").map_err(|_| FailureCode::Provenance)? != "binary.bin" {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "actions_sha256").map_err(|_| FailureCode::Provenance)?
        != context.actions_sha256
    {
        return Err(FailureCode::Provenance);
    }
    if require_str(&value, "tool_sha256").map_err(|_| FailureCode::Provenance)?
        != context.tool_sha256
    {
        return Err(FailureCode::Provenance);
    }

    let adapter = if is_oracle {
        &context.oracle_adapter_sha256
    } else {
        &context.candidate_adapter_sha256
    };
    if require_str(&value, "adapter_sha256").map_err(|_| FailureCode::Provenance)? != *adapter {
        return Err(FailureCode::Provenance);
    }

    if !is_oracle {
        if require_str(&value, "run_id").map_err(|_| FailureCode::Provenance)? != context.run_id {
            return Err(FailureCode::Provenance);
        }
        if require_str(&value, "task_id").map_err(|_| FailureCode::Provenance)? != context.task_id {
            return Err(FailureCode::Provenance);
        }
    }

    let root = if is_oracle {
        &context.oracle_root
    } else {
        &context.candidate_root
    };
    let binary = root.join("binary.bin");
    let binary_bytes = fs::read(&binary).map_err(|_| FailureCode::Provenance)?;
    if require_str(&value, "binary_sha256").map_err(|_| FailureCode::Provenance)?
        != sha256_bytes(&binary_bytes)
    {
        return Err(FailureCode::Provenance);
    }

    Ok(())
}

fn compare_frames(oracle_path: &Path, candidate_path: &Path) -> Result<bool, FailureCode> {
    let oracle_text = fs::read_to_string(oracle_path).map_err(|_| FailureCode::Integrity)?;
    let candidate_text = fs::read_to_string(candidate_path).map_err(|_| FailureCode::Integrity)?;

    if parse_json_strict(&candidate_text).is_err() {
        return Err(FailureCode::FrameInvalid);
    }
    if parse_json_strict(&oracle_text).is_err() {
        return Err(FailureCode::Integrity);
    }

    let oracle = Frame::from_json(&oracle_text).map_err(|_| FailureCode::FrameInvalid)?;
    let candidate = Frame::from_json(&candidate_text).map_err(|_| FailureCode::FrameInvalid)?;

    let diffs = oracle
        .diff_cells(&candidate)
        .map_err(|_| FailureCode::FrameInvalid)?;
    Ok(diffs.is_empty())
}

fn compare_states(
    oracle_path: &Path,
    candidate_path: &Path,
    state_keys: &[String],
) -> Result<bool, FailureCode> {
    let oracle_bytes = fs::read(oracle_path).map_err(|_| FailureCode::Integrity)?;
    let candidate_bytes = fs::read(candidate_path).map_err(|_| FailureCode::Integrity)?;
    let oracle = parse_json_bytes_strict(&oracle_bytes).map_err(|_| FailureCode::Integrity)?;
    let candidate =
        parse_json_bytes_strict(&candidate_bytes).map_err(|_| FailureCode::Integrity)?;

    if get_str(&oracle, "schema") != Some(STATE_SCHEMA)
        || get_str(&candidate, "schema") != Some(STATE_SCHEMA)
    {
        return Err(FailureCode::StateInvalid);
    }

    let allowed: HashSet<_> = state_keys.iter().cloned().collect();
    for value in [&oracle, &candidate] {
        let map = as_object_map(value).map_err(|_| FailureCode::StateInvalid)?;
        for key in map.keys() {
            if key == "schema" {
                continue;
            }
            if !allowed.contains(key) {
                return Err(FailureCode::StateInvalid);
            }
        }
        for key in state_keys {
            if !map.contains_key(key) {
                return Err(FailureCode::StateInvalid);
            }
        }
    }

    let mut oracle_payload = oracle.clone();
    let mut candidate_payload = candidate.clone();
    if let Some(map) = oracle_payload.as_object_mut() {
        map.remove("schema");
    }
    if let Some(map) = candidate_payload.as_object_mut() {
        map.remove("schema");
    }

    Ok(values_equal(&oracle_payload, &candidate_payload))
}

fn build_report(context: &CompareContext, outcome: CompareOutcome) -> (i32, ComparisonReport) {
    let required_count = context.required_count;
    match outcome {
        CompareOutcome::Success => (
            0,
            ComparisonReport {
                schema: COMPARISON_SCHEMA,
                run_id: context.run_id.clone(),
                task_id: context.task_id.clone(),
                context_sha256: sha256_bytes(&context.raw_bytes),
                required_count,
                checked_count: required_count,
                passed_count: required_count,
                results: context
                    .required_ids
                    .iter()
                    .map(|id| ScenarioResult {
                        id: id.clone(),
                        status: "passed",
                    })
                    .collect(),
                failures: Vec::new(),
            },
        ),
        CompareOutcome::Global(code) => (
            1,
            ComparisonReport {
                schema: COMPARISON_SCHEMA,
                run_id: context.run_id.clone(),
                task_id: context.task_id.clone(),
                context_sha256: sha256_bytes(&context.raw_bytes),
                required_count,
                checked_count: 0,
                passed_count: 0,
                results: Vec::new(),
                failures: vec![FailureRecord {
                    id: None,
                    code: code.as_str(),
                    detail: None,
                }],
            },
        ),
        CompareOutcome::Scenario(failures) => {
            let failed: HashMap<_, _> = failures
                .iter()
                .map(|(id, code)| (id.clone(), *code))
                .collect();
            let results = context
                .required_ids
                .iter()
                .map(|id| ScenarioResult {
                    id: id.clone(),
                    status: if failed.contains_key(id) {
                        "failed"
                    } else {
                        "passed"
                    },
                })
                .collect::<Vec<_>>();
            let failure_records = failures
                .into_iter()
                .map(|(id, code)| FailureRecord {
                    id: Some(id),
                    code: code.as_str(),
                    detail: None,
                })
                .collect::<Vec<_>>();
            let passed_count = results
                .iter()
                .filter(|result| result.status == "passed")
                .count() as u64;
            (
                1,
                ComparisonReport {
                    schema: COMPARISON_SCHEMA,
                    run_id: context.run_id.clone(),
                    task_id: context.task_id.clone(),
                    context_sha256: sha256_bytes(&context.raw_bytes),
                    required_count,
                    checked_count: required_count,
                    passed_count,
                    results,
                    failures: failure_records,
                },
            )
        }
    }
}

fn write_report(path: &Path, report: &ComparisonReport) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "report has no parent"))?;
    ensure_real_directory(parent)?;
    let json = serde_json::to_string(report).map_err(io::Error::other)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    file.write_all(format!("{json}\n").as_bytes())?;
    file.sync_all()
}

fn ensure_real_directory(path: &Path) -> io::Result<()> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "report parent must be absolute",
        ));
    }
    let mut current = PathBuf::from("/");
    for component in path.components() {
        if let std::path::Component::Normal(part) = component {
            current.push(part);
            let metadata = fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "report parent contains an unsafe path component",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[cfg(unix)]
    #[test]
    fn report_path_qualifies_symlinked_temporary_ancestry() -> Result<(), Box<dyn std::error::Error>>
    {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir()?;
        let real_root = temporary.path().join("real-run");
        fs::create_dir_all(real_root.join("outputs"))?;
        let aliased_root = temporary.path().join("var-link");
        symlink(&real_root, &aliased_root)?;

        let qualified = qualify_report_path(&aliased_root.join("outputs/report.json"))
            .map_err(io::Error::other)?;
        assert_eq!(
            qualified,
            real_root.canonicalize()?.join("outputs/report.json")
        );
        Ok(())
    }

    fn old_compare_context(report_path: &str) -> Value {
        json!({
            "schema": CONTEXT_SCHEMA,
            "run_id": "/run",
            "task_id": "TASK-001",
            "check_id": "CHK-001",
            "oracle_commit": "a".repeat(40),
            "candidate_source_tree": "b".repeat(40),
            "oracle_root": "/oracle",
            "candidate_root": "/candidate",
            "oracle_manifest_sha256": "c".repeat(64),
            "candidate_manifest_sha256": "d".repeat(64),
            "required_sha256": "e".repeat(64),
            "actions_sha256": "f".repeat(64),
            "required_ids": ["case-001"],
            "required_count": 1,
            "tool_sha256": "0".repeat(64),
            "oracle_adapter_sha256": "1".repeat(64),
            "candidate_adapter_sha256": "2".repeat(64),
            "report_path": report_path,
        })
    }

    #[test]
    fn runner_context_preserves_nested_comparator_schema_and_host_output()
    -> Result<(), Box<dyn std::error::Error>> {
        let nested = old_compare_context("/run/outputs/CHK-001.compare.json");
        let runner = json!({
            "schema": RUNNER_CONTEXT_SCHEMA,
            "run_id": "/run",
            "task_id": "TASK-001",
            "check_id": "CHK-001",
            "tree": "b".repeat(40),
            "oracle_commit": "a".repeat(40),
            "qualification": {
                "common": {"outputs": {"runtime": "/run/outputs"}},
                "comparator": {
                    "schema": CONTEXT_SCHEMA,
                    "context": nested,
                    "report_path": "/run/outputs/CHK-001.compare.json",
                },
            },
        });
        let raw = serde_json::to_vec(&runner)?;
        let parsed = parse_context(&raw).map_err(io::Error::other)?;
        assert_eq!(
            parsed.report_path,
            PathBuf::from("/run/outputs/CHK-001.compare.json")
        );
        assert_eq!(parsed.candidate_source_tree, "b".repeat(40));
        Ok(())
    }

    #[test]
    fn runner_context_rejects_comparator_report_outside_runtime_outputs()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut nested = old_compare_context("/run/outputs/CHK-001.compare.json");
        assert!(nested.is_object(), "nested context must be an object");
        let Some(nested_object) = nested.as_object_mut() else {
            return Ok(());
        };
        nested_object.insert(
            "report_path".to_string(),
            Value::String("/oracle/forged.json".to_string()),
        );
        let runner = json!({
            "schema": RUNNER_CONTEXT_SCHEMA,
            "run_id": "/run",
            "task_id": "TASK-001",
            "check_id": "CHK-001",
            "tree": "b".repeat(40),
            "oracle_commit": "a".repeat(40),
            "qualification": {
                "common": {"outputs": {"runtime": "/run/outputs"}},
                "comparator": {
                    "schema": CONTEXT_SCHEMA,
                    "context": nested,
                    "report_path": "/oracle/forged.json",
                },
            },
        });
        let raw = serde_json::to_vec(&runner)?;
        assert!(parse_context(&raw).is_err());
        Ok(())
    }
}
