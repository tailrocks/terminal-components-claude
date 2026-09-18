//! Bounded native verifier preparation and child launcher.
//!
//! This module owns only verifier-run state.  It does not create worktrees,
//! mutate refs, invoke taskfmt lifecycle commands, or write campaign state.

#![expect(
    unsafe_code,
    reason = "native inherited-pipe transport wraps POSIX descriptors"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::json_util::{canonical_json, parse_json_bytes_strict, sha256_bytes, sha256_canonical};

/// The ledger-owned preparation ABI.  Runner-specific fields live below this
/// envelope; the index, contexts, preparation results, and observer capability
/// all use this same binding.
const CONTEXT_SCHEMA: &str = "tc-proof-context/v1";
const INDEX_SCHEMA: &str = "tc-proof-context-index/v1";
const RESULT_SCHEMA: &str = "tc-proof-runner-result/v1";
const PREPARATION_RESULT_SCHEMA: &str = "tc-proof-preparation-result/v1";
const OBSERVER_SCHEMA: &str = "tc-proof-observer-capability/v1";
const EXPECTED_ORACLE_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const MAX_LAUNCH_TIMEOUT_MS: u64 = 600_000;
const MAX_CHILD_OUTPUT_BYTES: usize = 32 * 1024 * 1024;

/// Native preparation inputs supplied by the verifier subagent.
#[derive(Debug, Clone)]
pub struct PrepareOptions {
    /// Task package directory.
    pub task_dir: PathBuf,
    /// Empty external verifier run directory.
    pub run_dir: PathBuf,
    /// Committed candidate worktree.
    pub worktree: PathBuf,
    /// Explicit full scope-base commit.
    pub scope_base: String,
    /// Explicit tag ref containing the frozen oracle.
    pub oracle_tag: String,
    /// Expected peeled oracle commit.
    pub oracle_commit: String,
    /// Worker path used by task checks.
    pub tool: PathBuf,
    /// Native comparator path.
    pub comparator: PathBuf,
    /// Qualified standalone taskfmt path, when used by the task.
    pub taskfmt: Option<PathBuf>,
    /// Accepted dependency receipt paths.
    pub dependency_receipts: Vec<PathBuf>,
    /// Stable run identity, or a random value when absent.
    pub run_id: Option<String>,
    /// Observer nonce, or a random value when absent.
    pub observer_nonce: Option<String>,
    /// External observer Unix socket path.
    pub observer_socket: Option<PathBuf>,
}

/// Bounded child launch inputs.
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    /// External verifier run directory.
    pub run_dir: PathBuf,
    /// Check selected from context-index.json.
    pub check_id: String,
    /// Retained only to reject the retired socket transport explicitly.
    pub observer_socket: Option<PathBuf>,
    /// Maximum child lifetime.
    pub timeout: Duration,
    /// Program to launch.
    pub program: PathBuf,
    /// Program arguments.
    pub args: Vec<String>,
}

/// Summary emitted after a successful child launch and result validation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LaunchRecord {
    /// Result schema.
    pub schema: &'static str,
    /// Run identity.
    pub run_id: String,
    /// Check identity.
    pub check_id: String,
    /// Child exit code.
    pub exit: i32,
    /// Captured stdout digest.
    pub stdout_sha256: String,
    /// Captured stderr digest.
    pub stderr_sha256: String,
    /// Result artifact digest.
    pub result_sha256: String,
}

/// Native verifier failure.  All failures are intended to be fail-closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierError(String);

impl VerifierError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for VerifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for VerifierError {}

impl From<io::Error> for VerifierError {
    fn from(error: io::Error) -> Self {
        Self::new(error.to_string())
    }
}

type Result<T> = std::result::Result<T, VerifierError>;

#[derive(Debug, Deserialize)]
struct VerifyFile {
    schema: String,
    task_id: String,
    #[serde(default)]
    checks: Vec<VerifyCheck>,
}

#[derive(Debug, Deserialize)]
struct VerifyCheck {
    id: String,
    phase: String,
    #[serde(default)]
    shell: Option<String>,
    #[serde(default)]
    argv: Option<Vec<String>>,
    #[serde(default)]
    requirements: Vec<String>,
    #[serde(default)]
    acceptance: Vec<String>,
}

#[derive(Debug, Clone)]
struct CheckSpec {
    id: String,
    phase: String,
    operation: String,
    context_ref: Option<String>,
    lane: String,
    namespace: String,
    requirements: Vec<String>,
    acceptance: Vec<String>,
    command: Value,
}

#[derive(Debug, Clone)]
struct TemplateInfo {
    path: PathBuf,
    sha256: String,
    value: Value,
}

struct ContextInputs<'a> {
    common: &'a Value,
    trust_manifest: &'a Value,
    trust_sha256: &'a str,
    candidate_tree: &'a str,
    oracle: &'a OracleIdentity,
    dependencies: &'a [Value],
}

#[derive(Debug, Clone)]
struct PreparedMember {
    check_id: String,
    operation: String,
    context_path: PathBuf,
    context_sha256: String,
    preparation_result_path: PathBuf,
    result_path: PathBuf,
    comparator_report_path: Option<PathBuf>,
}

/// Prepared run identity returned by validation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PreparedRun {
    /// External run directory.
    pub run_dir: PathBuf,
    /// Run identity.
    pub run_id: String,
    /// Task identity.
    pub task_id: String,
    /// Candidate tree.
    pub candidate_tree: String,
    /// Context index digest.
    pub index_sha256: String,
    /// Trust manifest digest.
    pub trust_sha256: String,
    #[serde(skip)]
    members: Vec<PreparedMember>,
}

/// Prepare the exact immutable context/index set for one task.
pub fn prepare(options: &PrepareOptions) -> Result<PreparedRun> {
    let task_dir = canonical_input_dir(&options.task_dir, "task directory")?;
    let worktree = canonical_input_dir(&options.worktree, "candidate worktree")?;
    let run_dir = canonical_existing_dir(&options.run_dir, "run directory")?;
    if worktree == run_dir || run_dir.starts_with(&worktree) {
        return Err(VerifierError::new(
            "run directory must be external to worktree",
        ));
    }
    ensure_empty_dir(&run_dir, "run directory")?;
    let run_id = path_string(&run_dir);
    if let Some(requested) = options.run_id.as_deref()
        && requested != run_id
    {
        return Err(VerifierError::new(
            "run_id must be the canonical external run-directory path",
        ));
    }
    let observer_nonce = options
        .observer_nonce
        .clone()
        .unwrap_or_else(|| random_hex(24));
    require_safe_id(&observer_nonce, "observer nonce")?;

    let verify_path = regular_file(&task_dir.join("verify.toml"), "verify.toml")?;
    let verify_raw = fs::read(&verify_path)?;
    let verify_hash = sha256_bytes(&verify_raw);
    let verify: VerifyFile = toml::from_str(
        std::str::from_utf8(&verify_raw)
            .map_err(|error| VerifierError::new(format!("verify.toml is not UTF-8: {error}")))?,
    )
    .map_err(|error| VerifierError::new(format!("invalid verify.toml: {error}")))?;
    if verify.schema != "verify/v2" {
        return Err(VerifierError::new("verify.toml must use schema verify/v2"));
    }
    if verify.task_id != task_id_from_dir(&task_dir)? {
        return Err(VerifierError::new(
            "verify.toml task_id does not match task directory",
        ));
    }
    let checks = discover_checks(&verify)?;
    if checks.is_empty() {
        return Err(VerifierError::new("verify.toml has no checks"));
    }

    let readme_path = regular_file(&task_dir.join("README.md"), "task README")?;
    let readme_hash = hash_file(&readme_path)?;
    let task_meta = task_dir.join("task.toml");
    let task_meta_hash = if task_meta.exists() {
        Some((
            task_meta.clone(),
            hash_file(&regular_file(&task_meta, "task.toml")?)?,
        ))
    } else {
        None
    };

    let (candidate_commit, candidate_tree) = git_identity(&worktree)?;
    let scope_base = validate_scope_base(&worktree, &options.scope_base, &candidate_commit)?;
    require_clean_candidate(&worktree)?;
    let oracle = oracle_identity(&worktree, &options.oracle_tag, &options.oracle_commit)?;

    let tool = executable_identity(&options.tool, "proof tool")?;
    let comparator = executable_identity(&options.comparator, "native comparator")?;
    let taskfmt = options
        .taskfmt
        .as_ref()
        .map(|path| executable_identity(path, "taskfmt"))
        .transpose()?;
    let dependencies = dependency_receipts(
        &task_dir,
        &task_meta,
        &options.dependency_receipts,
        &worktree,
    )?;

    let template_map = discover_templates(&task_dir, &checks)?;
    if options.observer_socket.is_some() {
        return Err(VerifierError::new(
            "Unix-socket observer transport is retired; use inherited pipes",
        ));
    }

    let file_bindings = json!({
        "task_readme": {"path": path_string(&readme_path), "sha256": readme_hash},
        "verify_toml": {"path": path_string(&verify_path), "sha256": verify_hash},
        "task_toml": task_meta_hash.as_ref().map(|(path, hash)| json!({"path": path_string(path), "sha256": hash})),
        "templates": template_map.iter().map(|(id, template)| (id.clone(), json!({
            "path": path_string(&template.path), "sha256": template.sha256
        }))).collect::<Map<String, Value>>(),
    });
    let oracle_value = json!({
        "tag": options.oracle_tag,
        "tag_ref_sha256": oracle.tag_ref_sha256,
        "commit": oracle.commit,
        "tree": oracle.tree,
    });
    let dependency_value = Value::Array(dependencies.clone());
    let common = json!({
        "run_id": run_id,
        "task_id": verify.task_id,
        "run_dir": path_string(&run_dir),
        "worktree": path_string(&worktree),
        "scope_base": scope_base,
        "candidate_commit": candidate_commit,
        "candidate_tree": candidate_tree,
        "oracle": oracle_value,
        "files": file_bindings,
        "dependencies": dependency_value,
        "tool": tool,
        "comparator": comparator,
        "taskfmt": taskfmt,
        "observer": {
            "transport": "inherited-pipe/v1",
            "nonce": observer_nonce,
            "capability": path_string(&run_dir.join("observer.json")),
        },
        "outputs": {
            "runtime": path_string(&run_dir.join("outputs")),
            "taskfmt_logs": path_string(&run_dir.join("taskfmt-logs")),
        },
    });
    let trust_manifest = json!({
        "schema": "tc-proof-trust-manifest/v1",
        "common": common,
        "checks": checks.iter().map(|check| json!({
            "check_id": check.id,
            "operation": check.operation,
            "phase": check.phase,
            "context_reference": check.context_ref,
            "command_sha256": sha256_canonical(&check.command),
            "requirements": check.requirements,
            "acceptance": check.acceptance,
        })).collect::<Vec<_>>(),
    });
    let trust_sha256 = sha256_canonical(&trust_manifest);

    let contexts_dir = run_dir.join("contexts");
    let preparation_results_dir = run_dir.join("results");
    let outputs_dir = run_dir.join("outputs");
    let taskfmt_logs_dir = run_dir.join("taskfmt-logs");
    fs::create_dir(&contexts_dir)?;
    fs::create_dir(&preparation_results_dir)?;
    fs::create_dir(&outputs_dir)?;
    fs::create_dir(&taskfmt_logs_dir)?;
    let mut members = Vec::with_capacity(checks.len());
    let context_inputs = ContextInputs {
        common: &common,
        trust_manifest: &trust_manifest,
        trust_sha256: &trust_sha256,
        candidate_tree: &candidate_tree,
        oracle: &oracle,
        dependencies: &dependencies,
    };
    for check in &checks {
        let template = template_map.get(&check.id);
        let context = build_context(check, template, &context_inputs)?;
        let context_path = contexts_dir.join(format!("{}.json", check.id));
        let context_sha256 = write_json_new(&context_path, &context, "context")?;
        let preparation_result_path = preparation_results_dir.join(format!("{}.json", check.id));
        let result_path = outputs_dir.join(format!("{}.result.json", check.id));
        let preparation_result = json!({
            "schema": PREPARATION_RESULT_SCHEMA,
            "task_id": verify.task_id,
            "check_id": check.id,
            "run_id": run_id,
            "worktree_commit": candidate_commit,
            "scope_base": scope_base,
            "context_sha256": context_sha256,
            "status": "ready",
        });
        write_json_new(
            &preparation_result_path,
            &preparation_result,
            "preparation result",
        )?;
        members.push(PreparedMember {
            check_id: check.id.clone(),
            operation: check.operation.clone(),
            context_path,
            context_sha256,
            preparation_result_path,
            result_path,
            comparator_report_path: None,
        });
    }
    set_readonly_dir(&contexts_dir)?;
    set_readonly_dir(&preparation_results_dir)?;

    let observer_path = run_dir.join("observer.json");
    let observer_capability = json!({
        "schema": OBSERVER_SCHEMA,
        "task_id": verify.task_id,
        "run_id": run_id,
        "worktree_commit": candidate_commit,
        "scope_base": scope_base,
        "transport": "inherited-pipe/v1",
        "nonce_sha256": sha256_bytes(observer_nonce.as_bytes()),
    });
    let observer_sha256 =
        write_json_new(&observer_path, &observer_capability, "observer capability")?;
    let result_bindings = members
        .iter()
        .map(|member| {
            Ok(json!({
                "check_id": member.check_id,
                "path": path_string(&member.preparation_result_path),
                "sha256": hash_file(&member.preparation_result_path)?,
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    let index = json!({
        "schema": INDEX_SCHEMA,
        "task_id": verify.task_id,
        "run_id": run_id,
        "worktree_commit": candidate_commit,
        "scope_base": scope_base,
        "contexts": members.iter().map(|member| json!({
            "check_id": member.check_id,
            "path": path_string(&member.context_path),
            "sha256": member.context_sha256,
        })).collect::<Vec<_>>(),
        "results": result_bindings,
        "observer": {"path": path_string(&observer_path), "sha256": observer_sha256},
    });
    write_json_new(&run_dir.join("context-index.json"), &index, "context index")?;
    validate_run(&run_dir)
}

// The index intentionally has no private runner-only variant.  The campaign
// ledger consumes this exact shape.

/// Validate an existing native run without launching a child.
///
/// The index shape is deliberately the same shape consumed by
/// `scripts/campaign_ledger.py`.  Preparation results are immutable records in
/// `results/`; worker results are separate host-selected files in `outputs/`.
pub fn validate_run(run_dir: &Path) -> Result<PreparedRun> {
    let run_dir = canonical_existing_dir(run_dir, "run directory")?;
    let index_path = immutable_file(&run_dir.join("context-index.json"), "context index")?;
    let index_raw = fs::read(&index_path)?;
    let index = parse_json_object(&index_raw, "context index")?;
    exact_keys(
        &index,
        &[
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "contexts",
            "results",
            "observer",
        ],
        "context index",
    )?;
    if index.get("schema") != Some(&Value::String(INDEX_SCHEMA.to_string())) {
        return Err(VerifierError::new("wrong context index schema"));
    }
    let task_id = required_string(&index, "task_id")?;
    let run_id = required_string(&index, "run_id")?;
    if Path::new(&run_id) != run_dir {
        return Err(VerifierError::new(
            "context index run_id is not the run directory",
        ));
    }
    let worktree_commit = required_hex(&index, "worktree_commit", 40)?;
    let scope_base = required_hex(&index, "scope_base", 40)?;
    let contexts_dir = regular_dir(&run_dir.join("contexts"), "contexts directory")?;
    let results_dir = regular_dir(&run_dir.join("results"), "results directory")?;
    require_readonly_dir(&contexts_dir, "contexts directory")?;
    require_readonly_dir(&results_dir, "results directory")?;
    let outputs_dir = regular_dir(&run_dir.join("outputs"), "runtime output directory")?;
    let taskfmt_logs = regular_dir(&run_dir.join("taskfmt-logs"), "taskfmt log directory")?;
    validate_tree_paths(&taskfmt_logs, "taskfmt log directory")?;

    let context_entries = artifact_entries(&index, "contexts", &contexts_dir, "context")?;
    let result_entries = artifact_entries(&index, "results", &results_dir, "preparation result")?;
    if context_entries.is_empty() || context_entries.len() != result_entries.len() {
        return Err(VerifierError::new(
            "context/result sets are empty or differ",
        ));
    }
    if context_entries.keys().collect::<BTreeSet<_>>()
        != result_entries.keys().collect::<BTreeSet<_>>()
    {
        return Err(VerifierError::new("context/result check sets differ"));
    }

    let observer_binding = index
        .get("observer")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer binding is missing"))?;
    exact_keys(observer_binding, &["path", "sha256"], "observer binding")?;
    let observer_path = bound_artifact_path(
        observer_binding,
        &run_dir,
        "observer.json",
        "observer capability",
    )?;
    let observer_raw = fs::read(immutable_file(&observer_path, "observer capability")?)?;
    let observer = parse_json_object(&observer_raw, "observer capability")?;
    validate_observer_capability(&observer, &task_id, &run_id, &worktree_commit, &scope_base)?;
    if sha256_bytes(&observer_raw) != required_hex(observer_binding, "sha256", 64)? {
        return Err(VerifierError::new("observer capability hash mismatch"));
    }

    let mut members = Vec::with_capacity(context_entries.len());
    let mut candidate_tree = None;
    let mut trust_sha256 = None;
    for (check_id, (context_path, context_hash)) in &context_entries {
        let context_path = immutable_file(context_path, "context")?;
        let context_raw = fs::read(&context_path)?;
        if sha256_bytes(&context_raw) != *context_hash {
            return Err(VerifierError::new("context digest mismatch"));
        }
        let context = parse_json_object(&context_raw, "context")?;
        let operation = validate_context(
            &context,
            &run_id,
            &task_id,
            &worktree_commit,
            &scope_base,
            check_id,
        )?;
        let tree = required_hex(&context, "tree", 40)?;
        if candidate_tree
            .replace(tree.clone())
            .is_some_and(|old| old != tree)
        {
            return Err(VerifierError::new("contexts disagree on candidate tree"));
        }
        let context_trust = context
            .get("qualification")
            .and_then(Value::as_object)
            .and_then(|value| value.get("trust_manifest_sha256"))
            .and_then(Value::as_str)
            .ok_or_else(|| VerifierError::new("context trust digest is missing"))?
            .to_string();
        if trust_sha256
            .replace(context_trust.clone())
            .is_some_and(|old| old != context_trust)
        {
            return Err(VerifierError::new("contexts disagree on trust digest"));
        }
        let (preparation_result_path, preparation_hash) = result_entries
            .get(check_id)
            .ok_or_else(|| VerifierError::new("missing preparation result"))?;
        let preparation_result_path =
            immutable_file(preparation_result_path, "preparation result")?;
        let preparation_raw = fs::read(&preparation_result_path)?;
        if sha256_bytes(&preparation_raw) != *preparation_hash {
            return Err(VerifierError::new("preparation result digest mismatch"));
        }
        validate_preparation_result(
            &parse_json_object(&preparation_raw, "preparation result")?,
            &task_id,
            check_id,
            &run_id,
            &worktree_commit,
            &scope_base,
            context_hash,
        )?;
        members.push(PreparedMember {
            check_id: check_id.clone(),
            operation,
            context_path,
            context_sha256: context_hash.clone(),
            preparation_result_path,
            result_path: outputs_dir.join(format!("{check_id}.result.json")),
            comparator_report_path: context
                .get("qualification")
                .and_then(Value::as_object)
                .and_then(|qualification| qualification.get("comparator"))
                .and_then(Value::as_object)
                .and_then(|comparator| comparator.get("report_path"))
                .and_then(Value::as_str)
                .map(PathBuf::from),
        });
    }
    if directory_names(&contexts_dir)?
        != context_entries
            .keys()
            .map(|id| format!("{id}.json"))
            .collect()
    {
        return Err(VerifierError::new(
            "context directory has missing or extra files",
        ));
    }
    if directory_names(&results_dir)?
        != result_entries
            .keys()
            .map(|id| format!("{id}.json"))
            .collect()
    {
        return Err(VerifierError::new(
            "results directory has missing or extra files",
        ));
    }
    validate_runtime_outputs(&outputs_dir, &members, &run_id, false)?;
    let expected_root = BTreeSet::from([
        "contexts".to_string(),
        "results".to_string(),
        "outputs".to_string(),
        "taskfmt-logs".to_string(),
        "observer.json".to_string(),
        "context-index.json".to_string(),
    ]);
    if directory_names(&run_dir)? != expected_root {
        return Err(VerifierError::new(
            "run directory has missing or extra inputs",
        ));
    }
    let members_for_trust = members.clone();
    validate_trust_inputs(&members_for_trust, &index, &run_dir)?;
    Ok(PreparedRun {
        run_dir,
        run_id,
        task_id,
        candidate_tree: candidate_tree
            .ok_or_else(|| VerifierError::new("candidate tree missing"))?,
        index_sha256: sha256_bytes(&index_raw),
        trust_sha256: trust_sha256.ok_or_else(|| VerifierError::new("trust digest missing"))?,
        members,
    })
}

/// Launch one bounded child and accept only a complete, bound result artifact.
pub fn launch(options: &LaunchOptions) -> Result<LaunchRecord> {
    require_check_id(&options.check_id)?;
    if options.timeout.is_zero() || options.timeout > Duration::from_millis(MAX_LAUNCH_TIMEOUT_MS) {
        return Err(VerifierError::new(
            "launch timeout is outside the bounded range",
        ));
    }
    if options.program.as_os_str().is_empty() {
        return Err(VerifierError::new("launch program is empty"));
    }
    if options.observer_socket.is_some() {
        return Err(VerifierError::new(
            "alternate observer socket binding is rejected",
        ));
    }
    let prepared = validate_run(&options.run_dir)?;
    let member = prepared
        .members
        .iter()
        .find(|member| member.check_id == options.check_id)
        .ok_or_else(|| VerifierError::new("unknown check id"))?;
    if regular_path_exists(&member.result_path)? {
        return Err(VerifierError::new("result already exists; replay rejected"));
    }
    let context = parse_json_object(&fs::read(&member.context_path)?, "context")?;
    let qualification = context
        .get("qualification")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context qualification is missing"))?;
    let common = qualification
        .get("common")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context common binding is missing"))?;
    let worktree = PathBuf::from(required_string(common, "worktree")?);
    let observer = common
        .get("observer")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer binding is missing"))?;
    if required_string(observer, "transport")? != "inherited-pipe/v1" {
        return Err(VerifierError::new(
            "observer transport is not inherited-pipe/v1",
        ));
    }
    let nonce = required_string(observer, "nonce")?;
    let oracle_commit = required_string(
        common
            .get("oracle")
            .and_then(Value::as_object)
            .ok_or_else(|| VerifierError::new("oracle binding is missing"))?,
        "commit",
    )?;
    let (request_write, response_read, observer_thread) =
        start_observer_supervisor(&prepared, member, &nonce, &oracle_commit)?;
    let request_fd = request_write.as_raw_fd();
    let response_fd = response_read.as_raw_fd();
    let mut command = Command::new(&options.program);
    command
        .args(&options.args)
        .current_dir(&worktree)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("TC_PROOF_RUN_ID", &prepared.run_id)
        .env("TC_PROOF_TASK_ID", &prepared.task_id)
        .env("TC_PROOF_CHECK_ID", &options.check_id)
        .env(
            "TC_PROOF_CONTEXT_INDEX",
            prepared.run_dir.join("context-index.json"),
        )
        .env("TC_PROOF_CONTEXT_INDEX_SHA256", &prepared.index_sha256)
        .env("TC_PROOF_CONTEXT_SHA256", &member.context_sha256)
        .env("TC_PROOF_RESULT", &member.result_path)
        .env("TC_PROOF_SOURCE_TREE", &prepared.candidate_tree)
        .env("TC_PROOF_ORACLE_COMMIT", &oracle_commit)
        .env("TC_PROOF_OBSERVER_NONCE", &nonce)
        .env("TC_PROOF_OBSERVER_REQUEST_FD", request_fd.to_string())
        .env("TC_PROOF_OBSERVER_RESPONSE_FD", response_fd.to_string())
        .env_remove("TC_PROOF_OBSERVER_SOCKET");
    let mut child = command
        .spawn()
        .map_err(|error| VerifierError::new(format!("launch failed: {error}")))?;
    drop(request_write);
    drop(response_read);
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| VerifierError::new("stdout pipe unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| VerifierError::new("stderr pipe unavailable"))?;
    let stdout_thread = thread::spawn(|| read_bounded(stdout));
    let stderr_thread = thread::spawn(|| read_bounded(stderr));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > options.timeout {
            let _ = child.kill();
            let _ = child.wait();
            let observer = observer_thread
                .join()
                .map_err(|_| VerifierError::new("observer supervisor panicked"))?;
            observer?;
            return Err(VerifierError::new("child timed out"));
        }
        thread::sleep(Duration::from_millis(5));
    };
    let stdout = join_output(stdout_thread)?;
    let stderr = join_output(stderr_thread)?;
    let observer = observer_thread
        .join()
        .map_err(|_| VerifierError::new("observer supervisor panicked"))?;
    observer?;
    let exit = successful_exit(status)?;
    if !regular_path_exists(&member.result_path)? {
        return Err(VerifierError::new("child produced no result"));
    }
    let result_raw = fs::read(&member.result_path)?;
    validate_result(member, &prepared, &result_raw)?;
    let after = validate_run(&prepared.run_dir)?;
    let after_member = after
        .members
        .iter()
        .find(|candidate| candidate.check_id == options.check_id)
        .ok_or_else(|| VerifierError::new("result disappeared after validation"))?;
    let after_raw = fs::read(&after_member.result_path)?;
    if result_raw != after_raw {
        return Err(VerifierError::new("result changed during collection"));
    }
    Ok(LaunchRecord {
        schema: "tc-proof-launch/v1",
        run_id: prepared.run_id,
        check_id: options.check_id.clone(),
        exit,
        stdout_sha256: sha256_bytes(&stdout),
        stderr_sha256: sha256_bytes(&stderr),
        result_sha256: sha256_bytes(&result_raw),
    })
}

#[cfg(unix)]
fn make_pipe() -> Result<(File, File)> {
    let mut fds = [0_i32; 2];
    // SAFETY: libc fills two owned descriptors; both are immediately wrapped.
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return Err(VerifierError::new(format!(
            "observer pipe creation failed: {}",
            io::Error::last_os_error()
        )));
    }
    for fd in fds {
        // SAFETY: descriptors are valid and owned by this function.
        if unsafe { libc::fcntl(fd, libc::F_SETFD, 0) } == -1 {
            return Err(VerifierError::new(format!(
                "observer pipe inheritance failed: {}",
                io::Error::last_os_error()
            )));
        }
    }
    // SAFETY: each descriptor is transferred exactly once to a File.
    Ok(unsafe { (File::from_raw_fd(fds[0]), File::from_raw_fd(fds[1])) })
}

#[cfg(not(unix))]
fn make_pipe() -> Result<(File, File)> {
    Err(VerifierError::new(
        "native observer transport requires Unix file descriptors",
    ))
}

fn start_observer_supervisor(
    prepared: &PreparedRun,
    member: &PreparedMember,
    nonce: &str,
    oracle_commit: &str,
) -> Result<(File, File, thread::JoinHandle<Result<()>>)> {
    let (request_read, request_write) = make_pipe()?;
    let (response_read, response_write) = make_pipe()?;
    let run_id = prepared.run_id.clone();
    let task_id = prepared.task_id.clone();
    let check_id = member.check_id.clone();
    let operation = member.operation.clone();
    let tree = prepared.candidate_tree.clone();
    let nonce = nonce.to_owned();
    let oracle_commit = oracle_commit.to_owned();
    let supervisor = thread::spawn(move || {
        observe_worker(
            request_read,
            response_write,
            &run_id,
            &task_id,
            &check_id,
            &operation,
            &tree,
            &oracle_commit,
            &nonce,
        )
    });
    Ok((request_write, response_read, supervisor))
}

fn observe_worker(
    request_read: File,
    response_write: File,
    run_id: &str,
    task_id: &str,
    check_id: &str,
    operation: &str,
    tree: &str,
    oracle_commit: &str,
    nonce: &str,
) -> Result<()> {
    let expected = match operation {
        "oracle" => 2,
        "capture" | "account-tests" | "architecture" | "close" => 1,
        _ => 0,
    };
    let mut reader = io::BufReader::new(request_read);
    let mut writer = io::BufWriter::new(response_write);
    for request_id in 0..expected {
        let mut line = String::new();
        let count = reader.read_line(&mut line)?;
        if count == 0 || !line.ends_with('\n') || line.len() > 4096 {
            return Err(VerifierError::new("observer request truncated or missing"));
        }
        let request =
            parse_json_object(line.trim_end_matches('\n').as_bytes(), "observer request")?;
        exact_keys(
            &request,
            &[
                "schema",
                "nonce",
                "run_id",
                "task_id",
                "check_id",
                "request_id",
                "operation",
                "source_commit",
                "tree",
            ],
            "observer request",
        )?;
        if request.get("schema") != Some(&Value::String("tc-proof-runner-observe/v1".to_string()))
            || required_string(&request, "nonce")? != nonce
            || required_string(&request, "run_id")? != run_id
            || required_string(&request, "task_id")? != task_id
            || required_string(&request, "check_id")? != check_id
            || request.get("request_id").and_then(Value::as_u64) != Some(request_id as u64)
            || required_string(&request, "operation")? != operation
            || required_string(&request, "source_commit")? != oracle_commit
            || required_string(&request, "tree")? != tree
        {
            return Err(VerifierError::new(
                "observer request binding mismatch or replay",
            ));
        }
        let response = json!({
            "schema": "tc-proof-observation/v1",
            "nonce": nonce,
            "run_id": run_id,
            "task_id": task_id,
            "check_id": check_id,
            "request_id": request_id,
            "operation": operation,
            "source_commit": oracle_commit,
            "tree": tree,
            "exit": 0,
            "stdout": "",
            "stderr": "",
            "files": {},
            "payload": {},
            "records": [],
        });
        writer.write_all(canonical_json(&response).as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    let mut extra = [0_u8; 1];
    if reader.read(&mut extra)? != 0 {
        return Err(VerifierError::new("observer replay or incomplete close"));
    }
    Ok(())
}

fn discover_checks(verify: &VerifyFile) -> Result<Vec<CheckSpec>> {
    let mut ids = BTreeSet::new();
    let mut checks = Vec::with_capacity(verify.checks.len());
    for check in &verify.checks {
        require_check_id(&check.id)?;
        if !ids.insert(check.id.clone()) {
            return Err(VerifierError::new("duplicate check id in verify.toml"));
        }
        let (command, source) = match (&check.shell, &check.argv) {
            (Some(shell), None) => (Value::String(shell.clone()), shell.clone()),
            (None, Some(argv)) if !argv.is_empty() => (json!(argv), argv.join(" ")),
            _ => {
                return Err(VerifierError::new(format!(
                    "check {} must have exactly one command",
                    check.id
                )));
            }
        };
        let operation = discover_operation(&source)?;
        let context_ref = discover_context_reference(&source, &check.id)?;
        let lane = discover_flag(&source, "--lane").unwrap_or_else(|| "direct".to_string());
        let namespace = discover_flag(&source, "--namespace").unwrap_or_default();
        checks.push(CheckSpec {
            id: check.id.clone(),
            phase: check.phase.clone(),
            operation,
            context_ref,
            lane,
            namespace,
            requirements: check.requirements.clone(),
            acceptance: check.acceptance.clone(),
            command,
        });
    }
    let refs: Vec<_> = checks
        .iter()
        .filter_map(|check| check.context_ref.clone())
        .collect();
    if refs.iter().collect::<BTreeSet<_>>().len() != refs.len() {
        return Err(VerifierError::new(
            "duplicate context reference in verify.toml",
        ));
    }
    Ok(checks)
}

fn discover_operation(command: &str) -> Result<String> {
    let tokens = command
        .split(|character: char| {
            character.is_whitespace() || matches!(character, '"' | '\'' | ';' | '|')
        })
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let Some(position) = tokens
        .iter()
        .position(|token| token.ends_with("tc-proof") || *token == "tc-proof")
    else {
        return Ok("external".to_string());
    };
    Ok(tokens
        .get(position + 1)
        .filter(|operation| !operation.starts_with('-'))
        .map_or_else(
            || "external".to_string(),
            |operation| (*operation).to_string(),
        ))
}

fn discover_context_reference(command: &str, check_id: &str) -> Result<Option<String>> {
    let marker = "$RUN_DIR/contexts/";
    let mut found = None;
    let mut remaining = command;
    while let Some(position) = remaining.find(marker) {
        let suffix = &remaining[position + marker.len()..];
        let end = suffix
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || character == '-' || character == '.')
            })
            .unwrap_or(suffix.len());
        let reference = &suffix[..end];
        if !reference.starts_with("CHK-") || !reference.ends_with(".json") || reference.len() != 12
        {
            return Err(VerifierError::new("invalid verifier context reference"));
        }
        let reference_id = reference.trim_end_matches(".json");
        if reference_id != check_id {
            return Err(VerifierError::new(
                "context reference does not match check id",
            ));
        }
        if found.replace(reference.to_string()).is_some() {
            return Err(VerifierError::new(
                "duplicate context reference in check command",
            ));
        }
        remaining = &suffix[end..];
    }
    Ok(found)
}

fn discover_flag(command: &str, flag: &str) -> Option<String> {
    let tokens = command
        .split(|character: char| character.is_whitespace() || matches!(character, '"' | '\''))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens
        .iter()
        .position(|token| *token == flag)
        .and_then(|position| tokens.get(position + 1))
        .map(|value| (*value).to_string())
}

fn discover_templates(
    task_dir: &Path,
    checks: &[CheckSpec],
) -> Result<BTreeMap<String, TemplateInfo>> {
    let directory = task_dir.join("trusted/check-context-templates");
    if !directory.exists() {
        return Ok(BTreeMap::new());
    }
    let directory = regular_dir(&directory, "context template directory")?;
    let check_ids: BTreeSet<_> = checks.iter().map(|check| check.id.as_str()).collect();
    let mut templates = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() && !path.is_symlink() {
            continue;
        }
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.starts_with("CHK-")
            || !(name.ends_with(".json") || name.ends_with(".template.json"))
        {
            continue;
        }
        let id = name
            .split('.')
            .next()
            .ok_or_else(|| VerifierError::new("invalid context template name"))?;
        require_check_id(id)?;
        if !check_ids.contains(id) {
            return Err(VerifierError::new(
                "context template is not bound to a verify check",
            ));
        }
        if templates.contains_key(id) {
            return Err(VerifierError::new("duplicate context template"));
        }
        let path = regular_file(&path, "context template")?;
        let raw = fs::read(&path)?;
        let value = parse_json_object(&raw, "context template")?;
        templates.insert(
            id.to_string(),
            TemplateInfo {
                path,
                sha256: sha256_bytes(&raw),
                value: Value::Object(value),
            },
        );
    }
    Ok(templates)
}

fn build_context(
    check: &CheckSpec,
    template: Option<&TemplateInfo>,
    inputs: &ContextInputs<'_>,
) -> Result<Value> {
    let common = inputs.common;
    let trust_manifest = inputs.trust_manifest;
    let trust_sha256 = inputs.trust_sha256;
    let candidate_tree = inputs.candidate_tree;
    let oracle = inputs.oracle;
    let dependencies = inputs.dependencies;
    let template_value = template.map(|template| template.value.clone());
    let mut context = json!({});
    let object = context
        .as_object_mut()
        .ok_or_else(|| VerifierError::new("context template is not an object"))?;
    object.insert(
        "schema".to_string(),
        Value::String(CONTEXT_SCHEMA.to_string()),
    );
    object.insert(
        "run_id".to_string(),
        required_string(
            common
                .as_object()
                .ok_or_else(|| VerifierError::new("common binding is not an object"))?,
            "run_id",
        )
        .map(Value::String)?,
    );
    object.insert(
        "task_id".to_string(),
        required_string(
            common
                .as_object()
                .ok_or_else(|| VerifierError::new("common binding is not an object"))?,
            "task_id",
        )
        .map(Value::String)?,
    );
    object.insert("check_id".to_string(), Value::String(check.id.clone()));
    object.insert(
        "worktree_commit".to_string(),
        required_string(
            common
                .as_object()
                .ok_or_else(|| VerifierError::new("common binding is not an object"))?,
            "candidate_commit",
        )
        .map(Value::String)?,
    );
    object.insert(
        "scope_base".to_string(),
        required_string(
            common
                .as_object()
                .ok_or_else(|| VerifierError::new("common binding is not an object"))?,
            "scope_base",
        )
        .map(Value::String)?,
    );
    object.insert(
        "operation".to_string(),
        Value::String(check.operation.clone()),
    );
    object.insert(
        "tree".to_string(),
        Value::String(candidate_tree.to_string()),
    );
    object.insert(
        "oracle_commit".to_string(),
        Value::String(oracle.commit.clone()),
    );
    object.insert(
        "oracle_tree".to_string(),
        Value::String(oracle.tree.clone()),
    );
    object.insert(
        "bundle".to_string(),
        required_string(
            common
                .get("oracle")
                .and_then(Value::as_object)
                .ok_or_else(|| VerifierError::new("oracle binding is not an object"))?,
            "tag",
        )
        .map(Value::String)?,
    );
    object.insert(
        "bundle_sha256".to_string(),
        required_string(
            common
                .get("oracle")
                .and_then(Value::as_object)
                .ok_or_else(|| VerifierError::new("oracle binding is not an object"))?,
            "tag_ref_sha256",
        )
        .map(Value::String)?,
    );
    object.insert(
        "tool".to_string(),
        common
            .get("tool")
            .cloned()
            .ok_or_else(|| VerifierError::new("common tool binding is missing"))?,
    );
    object.insert(
        "dependencies".to_string(),
        Value::Array(dependencies.to_vec()),
    );
    object
        .entry("adapter".to_string())
        .or_insert_with(|| json!({"changes": []}));
    object.insert("lane".to_string(), Value::String(check.lane.clone()));
    object.insert(
        "axes".to_string(),
        json!({"lanes": ["direct", "pty"], "widths": [8, 12], "palettes": ["blue", "yellow"]}),
    );
    object.insert(
        "members".to_string(),
        json!([
            "tiny/direct/8/blue",
            "tiny/direct/8/yellow",
            "tiny/direct/12/blue",
            "tiny/direct/12/yellow",
            "tiny/pty/8/blue",
            "tiny/pty/8/yellow",
            "tiny/pty/12/blue",
            "tiny/pty/12/yellow"
        ]),
    );
    object
        .entry("inventory".to_string())
        .or_insert_with(|| json!({}));
    object
        .entry("evidence".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("configuration".to_string())
        .or_insert_with(|| json!({"seed": 0, "seed_steps": []}));
    let mut qualification = object.remove("qualification").unwrap_or_else(|| json!({}));
    let qualification_object = qualification
        .as_object_mut()
        .ok_or_else(|| VerifierError::new("template qualification is not an object"))?;
    qualification_object.insert("common".to_string(), common.clone());
    qualification_object.insert("trust_manifest".to_string(), trust_manifest.clone());
    qualification_object.insert(
        "trust_manifest_sha256".to_string(),
        Value::String(trust_sha256.to_string()),
    );
    qualification_object.insert(
        "check".to_string(),
        json!({
            "phase": check.phase,
            "context_reference": check.context_ref,
            "command_sha256": sha256_canonical(&check.command),
            "requirements": check.requirements,
            "acceptance": check.acceptance,
            "lane": check.lane,
            "namespace": check.namespace,
        }),
    );
    if let Some(template) = template {
        qualification_object.insert("worker_context".to_string(), template.value.clone());
        qualification_object.insert(
            "template_sha256".to_string(),
            Value::String(template.sha256.clone()),
        );
    }
    if let Some(template_value) = template_value
        && check.operation == "compare"
    {
        let report_path = run_output_path(common, &check.id, "compare.json")?;
        let mut comparator = template_value;
        if let Some(comparator_object) = comparator.as_object_mut() {
            comparator_object.insert(
                "run_id".to_string(),
                object
                    .get("run_id")
                    .cloned()
                    .ok_or_else(|| VerifierError::new("context run_id is missing"))?,
            );
            comparator_object.insert(
                "task_id".to_string(),
                object
                    .get("task_id")
                    .cloned()
                    .ok_or_else(|| VerifierError::new("context task_id is missing"))?,
            );
            comparator_object.insert("check_id".to_string(), Value::String(check.id.clone()));
            comparator_object.insert(
                "oracle_commit".to_string(),
                object
                    .get("oracle_commit")
                    .cloned()
                    .ok_or_else(|| VerifierError::new("context oracle_commit is missing"))?,
            );
            comparator_object.insert(
                "candidate_source_tree".to_string(),
                object
                    .get("tree")
                    .cloned()
                    .ok_or_else(|| VerifierError::new("context tree is missing"))?,
            );
            comparator_object.insert(
                "report_path".to_string(),
                Value::String(path_string(&report_path)),
            );
        }
        qualification_object.insert(
            "comparator".to_string(),
            json!({
                "schema": "tc-proof-compare-context/v1",
                "context": comparator,
                "report_path": path_string(&report_path),
            }),
        );
    }
    object.insert("qualification".to_string(), qualification);
    Ok(context)
}

fn artifact_entries(
    index: &Map<String, Value>,
    key: &str,
    directory: &Path,
    label: &str,
) -> Result<BTreeMap<String, (PathBuf, String)>> {
    let entries = index
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new(format!("context index {key} must be an array")))?;
    let mut result = BTreeMap::new();
    for entry in entries {
        let entry = entry
            .as_object()
            .ok_or_else(|| VerifierError::new(format!("{label} entry is not an object")))?;
        exact_keys(entry, &["check_id", "path", "sha256"], label)?;
        let check_id = required_string(entry, "check_id")?;
        require_check_id(&check_id)?;
        let path = absolute_path(Path::new(required_string(entry, "path")?.as_str()), label)?;
        if path.parent() != Some(directory)
            || path.file_name().map(|name| name.to_string_lossy())
                != Some(format!("{check_id}.json").into())
        {
            return Err(VerifierError::new(format!(
                "{label} escaped its bound directory"
            )));
        }
        let digest = required_hex(entry, "sha256", 64)?;
        if result.insert(check_id, (path, digest)).is_some() {
            return Err(VerifierError::new(format!("duplicate {label} identity")));
        }
    }
    Ok(result)
}

fn bound_artifact_path(
    binding: &Map<String, Value>,
    run_dir: &Path,
    filename: &str,
    label: &str,
) -> Result<PathBuf> {
    let path = absolute_path(Path::new(required_string(binding, "path")?.as_str()), label)?;
    if path.parent() != Some(run_dir)
        || path.file_name().map(|name| name.to_string_lossy()) != Some(filename.into())
    {
        return Err(VerifierError::new(format!(
            "{label} escaped the run directory"
        )));
    }
    Ok(path)
}

fn validate_observer_capability(
    observer: &Map<String, Value>,
    task_id: &str,
    run_id: &str,
    worktree_commit: &str,
    scope_base: &str,
) -> Result<()> {
    exact_keys(
        observer,
        &[
            "schema",
            "task_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "transport",
            "nonce_sha256",
        ],
        "observer capability",
    )?;
    if observer.get("schema") != Some(&Value::String(OBSERVER_SCHEMA.to_string()))
        || required_string(observer, "task_id")? != task_id
        || required_string(observer, "run_id")? != run_id
        || required_string(observer, "worktree_commit")? != worktree_commit
        || required_string(observer, "scope_base")? != scope_base
        || required_string(observer, "transport")? != "inherited-pipe/v1"
        || !is_hex(&required_string(observer, "nonce_sha256")?, 64)
    {
        return Err(VerifierError::new("observer capability identity mismatch"));
    }
    Ok(())
}

fn validate_preparation_result(
    result: &Map<String, Value>,
    task_id: &str,
    check_id: &str,
    run_id: &str,
    worktree_commit: &str,
    scope_base: &str,
    context_sha256: &str,
) -> Result<()> {
    exact_keys(
        result,
        &[
            "schema",
            "task_id",
            "check_id",
            "run_id",
            "worktree_commit",
            "scope_base",
            "context_sha256",
            "status",
        ],
        "preparation result",
    )?;
    if result.get("schema") != Some(&Value::String(PREPARATION_RESULT_SCHEMA.to_string()))
        || required_string(result, "task_id")? != task_id
        || required_string(result, "check_id")? != check_id
        || required_string(result, "run_id")? != run_id
        || required_string(result, "worktree_commit")? != worktree_commit
        || required_string(result, "scope_base")? != scope_base
        || required_string(result, "context_sha256")? != context_sha256
        || required_string(result, "status")? != "ready"
    {
        return Err(VerifierError::new("preparation result identity mismatch"));
    }
    Ok(())
}

fn validate_tree_paths(root: &Path, label: &str) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || link_count(&metadata) != 1 {
            return Err(VerifierError::new(format!(
                "{label} contains an unsafe link"
            )));
        }
        if metadata.is_dir() {
            validate_tree_paths(&path, label)?;
        } else if !metadata.is_file() {
            return Err(VerifierError::new(format!("{label} contains a non-file")));
        }
    }
    Ok(())
}

fn validate_runtime_outputs(
    outputs_dir: &Path,
    members: &[PreparedMember],
    run_id: &str,
    require_passed: bool,
) -> Result<()> {
    let mut expected = BTreeSet::new();
    for member in members {
        expected.insert(
            member
                .result_path
                .file_name()
                .ok_or_else(|| VerifierError::new("invalid runtime result path"))?
                .to_string_lossy()
                .into_owned(),
        );
        if let Some(report) = &member.comparator_report_path {
            if report.parent() != Some(outputs_dir) {
                return Err(VerifierError::new("comparator report escaped outputs"));
            }
            expected.insert(
                report
                    .file_name()
                    .ok_or_else(|| VerifierError::new("invalid comparator report path"))?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    let actual = directory_names(outputs_dir)?;
    if !actual.is_subset(&expected) {
        return Err(VerifierError::new("runtime outputs contain extra files"));
    }
    for member in members {
        if regular_path_exists(&member.result_path)? {
            validate_result_if_present(member, run_id, require_passed)?;
        }
        if let Some(report) = &member.comparator_report_path
            && regular_path_exists(report)?
        {
            let report_raw = fs::read(report)?;
            let _ = parse_json_object(&report_raw, "comparator report")?;
            set_readonly_file(report)?;
        }
    }
    Ok(())
}

fn validate_context(
    context: &Map<String, Value>,
    run_id: &str,
    task_id: &str,
    worktree_commit: &str,
    scope_base: &str,
    check_id: &str,
) -> Result<String> {
    let allowed = BTreeSet::from([
        "schema",
        "run_id",
        "task_id",
        "check_id",
        "worktree_commit",
        "scope_base",
        "operation",
        "tree",
        "oracle_commit",
        "oracle_tree",
        "bundle",
        "bundle_sha256",
        "tool",
        "dependencies",
        "adapter",
        "lane",
        "axes",
        "members",
        "inventory",
        "evidence",
        "configuration",
        "qualification",
        "architecture_profile",
        "branch_host_projection",
    ]);
    if context.keys().any(|key| !allowed.contains(key.as_str())) {
        return Err(VerifierError::new("context contains unexpected fields"));
    }
    for (key, expected) in [
        ("schema", CONTEXT_SCHEMA),
        ("run_id", run_id),
        ("task_id", task_id),
        ("check_id", check_id),
        ("worktree_commit", worktree_commit),
        ("scope_base", scope_base),
    ] {
        if context.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(VerifierError::new(format!(
                "context binding mismatch: {key}"
            )));
        }
    }
    let qualification = context
        .get("qualification")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context qualification missing"))?;
    let trust_sha256 = qualification
        .get("trust_manifest_sha256")
        .and_then(Value::as_str)
        .filter(|value| is_hex(value, 64))
        .ok_or_else(|| VerifierError::new("context trust digest is invalid"))?;
    let manifest = qualification
        .get("trust_manifest")
        .ok_or_else(|| VerifierError::new("context trust manifest missing"))?;
    if sha256_canonical(manifest) != trust_sha256 {
        return Err(VerifierError::new("trust manifest digest mismatch"));
    }
    let tool = context
        .get("tool")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context tool missing"))?;
    let tool_path = PathBuf::from(required_string(tool, "path")?);
    let actual_tool_hash = hash_file(&regular_file(&tool_path, "context tool")?)?;
    if Some(actual_tool_hash.as_str()) != tool.get("sha256").and_then(Value::as_str) {
        return Err(VerifierError::new("context tool digest mismatch"));
    }
    let operation = required_string(context, "operation")?;
    let common = qualification
        .get("common")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context common binding is missing"))?;
    if required_string(common, "run_id")? != run_id
        || required_string(common, "task_id")? != task_id
        || required_string(common, "scope_base")? != scope_base
        || required_string(common, "candidate_commit")? != worktree_commit
    {
        return Err(VerifierError::new("context common identity mismatch"));
    }
    if required_string(common, "run_dir")? != run_id {
        return Err(VerifierError::new("context run directory mismatch"));
    }
    if let Some(outputs) = common.get("outputs").and_then(Value::as_object) {
        let runtime = PathBuf::from(required_string(outputs, "runtime")?);
        let logs = PathBuf::from(required_string(outputs, "taskfmt_logs")?);
        if !runtime.is_absolute() || !logs.is_absolute() {
            return Err(VerifierError::new("context output roots must be absolute"));
        }
    } else {
        return Err(VerifierError::new("context output roots are missing"));
    }
    if let Some(comparator) = qualification.get("comparator").and_then(Value::as_object) {
        let report = PathBuf::from(required_string(comparator, "report_path")?);
        let runtime = PathBuf::from(required_string(
            common
                .get("outputs")
                .and_then(Value::as_object)
                .ok_or_else(|| VerifierError::new("runtime output root is missing"))?,
            "runtime",
        )?);
        if report.parent() != Some(runtime.as_path()) || report == runtime {
            return Err(VerifierError::new(
                "comparator report escaped runtime outputs",
            ));
        }
        let expected_name = format!("{check_id}.compare.json");
        if report.file_name().and_then(|name| name.to_str()) != Some(expected_name.as_str()) {
            return Err(VerifierError::new(
                "comparator report filename is not host-bound",
            ));
        }
    }
    Ok(operation)
}

fn validate_trust_inputs(
    members: &[PreparedMember],
    index: &Map<String, Value>,
    run_dir: &Path,
) -> Result<()> {
    let first = members
        .first()
        .ok_or_else(|| VerifierError::new("no prepared members"))?;
    let context = parse_json_object(&fs::read(&first.context_path)?, "context")?;
    let qualification = context
        .get("qualification")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("qualification missing"))?;
    let manifest = qualification
        .get("trust_manifest")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("trust manifest missing"))?;
    let common = manifest
        .get("common")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("common trust binding missing"))?;
    if qualification
        .get("trust_manifest_sha256")
        .and_then(Value::as_str)
        != Some(sha256_canonical(&Value::Object(manifest.clone())).as_str())
    {
        return Err(VerifierError::new("trust manifest/context mismatch"));
    }
    let worktree = PathBuf::from(required_string(common, "worktree")?);
    let (commit, tree) = git_identity(&worktree)?;
    if commit != required_string(common, "candidate_commit")?
        || tree != required_string(common, "candidate_tree")?
        || commit != required_string(index, "worktree_commit")?
        || required_string(index, "task_id")? != required_string(common, "task_id")?
        || required_string(index, "run_id")? != required_string(common, "run_id")?
        || required_string(index, "scope_base")? != required_string(common, "scope_base")?
    {
        return Err(VerifierError::new("candidate identity changed"));
    }
    require_clean_candidate(&worktree)?;
    let scope_base = required_string(common, "scope_base")?;
    validate_scope_base(&worktree, &scope_base, &commit)?;
    let files = manifest
        .get("common")
        .and_then(Value::as_object)
        .and_then(|common| common.get("files"))
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("trusted files missing"))?;
    for key in ["task_readme", "verify_toml"] {
        validate_bound_file(files, key)?;
    }
    if !files.get("task_toml").is_some_and(Value::is_null) {
        validate_bound_file(files, "task_toml")?;
    }
    if let Some(templates) = files.get("templates").and_then(Value::as_object) {
        for key in templates.keys() {
            validate_bound_file(templates, key)?;
        }
    }
    validate_identity_hash(common, "tool", "tool")?;
    validate_identity_hash(common, "comparator", "comparator")?;
    if !common.get("taskfmt").is_some_and(Value::is_null) {
        validate_identity_hash(common, "taskfmt", "taskfmt")?;
    }
    validate_dependency_bindings(common, &worktree)?;
    let oracle = common
        .get("oracle")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("oracle binding missing"))?;
    let tag = required_string(oracle, "tag")?;
    let tag_ref = git_output(&worktree, &["rev-parse", &tag])?;
    if sha256_bytes(tag_ref.as_bytes()) != required_string(oracle, "tag_ref_sha256")? {
        return Err(VerifierError::new("oracle tag ref changed"));
    }
    let peeled = git_output(&worktree, &["rev-parse", &format!("{tag}^{{commit}}")])?;
    let oracle_commit = required_string(oracle, "commit")?;
    if peeled != oracle_commit
        || oracle_commit != EXPECTED_ORACLE_COMMIT && tag == "refs/tags/visual-baseline"
    {
        return Err(VerifierError::new("oracle commit changed or is not frozen"));
    }
    let oracle_tree = git_output(&worktree, &["rev-parse", &format!("{tag}^{{tree}}")])?;
    if oracle_tree != required_string(oracle, "tree")? {
        return Err(VerifierError::new("oracle tree changed"));
    }
    if Path::new(required_string(common, "run_dir")?.as_str()) != run_dir {
        return Err(VerifierError::new("run directory binding changed"));
    }
    let index_hash = sha256_bytes(&fs::read(run_dir.join("context-index.json"))?);
    if index_hash.is_empty() {
        return Err(VerifierError::new("empty context index digest"));
    }
    Ok(())
}

fn validate_bound_file(files: &Map<String, Value>, key: &str) -> Result<()> {
    let binding = files
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new(format!("trusted file binding missing: {key}")))?;
    let path = regular_file(
        Path::new(required_string(binding, "path")?.as_str()),
        "trusted file",
    )?;
    if hash_file(&path)? != required_string(binding, "sha256")? {
        return Err(VerifierError::new(format!("trusted file changed: {key}")));
    }
    Ok(())
}

fn validate_identity_hash(common: &Map<String, Value>, key: &str, label: &str) -> Result<()> {
    let identity = common
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new(format!("{label} binding missing")))?;
    let path = regular_file(
        Path::new(required_string(identity, "path")?.as_str()),
        label,
    )?;
    if hash_file(&path)? != required_string(identity, "sha256")? {
        return Err(VerifierError::new(format!("{label} changed")));
    }
    Ok(())
}

fn validate_dependency_bindings(common: &Map<String, Value>, worktree: &Path) -> Result<()> {
    let dependencies = common
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new("dependency bindings missing"))?;
    for dependency in dependencies {
        let dependency = dependency
            .as_object()
            .ok_or_else(|| VerifierError::new("dependency binding is not an object"))?;
        if dependency.get("accepted") != Some(&Value::Bool(true))
            || dependency.get("integrated") != Some(&Value::Bool(true))
        {
            return Err(VerifierError::new("dependency receipt is not accepted"));
        }
        let path = regular_file(
            Path::new(required_string(dependency, "path")?.as_str()),
            "dependency receipt",
        )?;
        if hash_file(&path)? != required_string(dependency, "sha256")? {
            return Err(VerifierError::new("dependency receipt changed"));
        }
        let commit = required_string(dependency, "integration_commit")?;
        if !is_hex(&commit, 40)
            || !Command::new("git")
                .args([
                    "-C",
                    path_string(worktree).as_str(),
                    "merge-base",
                    "--is-ancestor",
                    &commit,
                    "HEAD",
                ])
                .status()?
                .success()
        {
            return Err(VerifierError::new("dependency ancestry changed"));
        }
    }
    Ok(())
}

fn validate_result_if_present(
    member: &PreparedMember,
    run_id: &str,
    require_passed: bool,
) -> Result<()> {
    let path = regular_file(&member.result_path, "result")?;
    validate_result_fields(member, run_id, &fs::read(path)?, require_passed)?;
    set_readonly_file(&member.result_path)
}

fn validate_result(member: &PreparedMember, prepared: &PreparedRun, raw: &[u8]) -> Result<()> {
    validate_result_fields(member, &prepared.run_id, raw, true)?;
    set_readonly_file(&member.result_path)
}

fn validate_result_fields(
    member: &PreparedMember,
    run_id: &str,
    raw: &[u8],
    require_passed: bool,
) -> Result<()> {
    if raw.is_empty() || raw.len() > MAX_CHILD_OUTPUT_BYTES || !raw.ends_with(b"}") {
        return Err(VerifierError::new("result is missing or truncated"));
    }
    let result = parse_json_object(raw, "result")?;
    exact_keys(
        &result,
        &[
            "schema",
            "run_id",
            "operation",
            "context_sha256",
            "status",
            "category",
            "observation_digests",
            "outputs",
        ],
        "result",
    )?;
    let status = result.get("status").and_then(Value::as_str);
    let category_valid = match status {
        Some("passed") => result.get("category").is_some_and(Value::is_null),
        Some("rejected") if !require_passed => result
            .get("category")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty()),
        _ => false,
    };
    if result.get("schema") != Some(&Value::String(RESULT_SCHEMA.to_string()))
        || result.get("run_id").and_then(Value::as_str) != Some(run_id)
        || result.get("operation").and_then(Value::as_str) != Some(member.operation.as_str())
        || result.get("context_sha256").and_then(Value::as_str)
            != Some(member.context_sha256.as_str())
        || !category_valid
        || !result.get("outputs").is_some_and(Value::is_object)
    {
        return Err(VerifierError::new("result identity or status mismatch"));
    }
    let digests = result
        .get("observation_digests")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new("result observations are not an array"))?;
    if digests
        .iter()
        .any(|digest| digest.as_str().is_none_or(|value| !is_hex(value, 64)))
    {
        return Err(VerifierError::new("result observation digest is invalid"));
    }
    Ok(())
}

fn parse_json_object(raw: &[u8], label: &str) -> Result<Map<String, Value>> {
    let value = parse_json_bytes_strict(raw)
        .map_err(|error| VerifierError::new(format!("{label} is invalid JSON: {error}")))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| VerifierError::new(format!("{label} must be a JSON object")))
}

fn exact_keys(object: &Map<String, Value>, keys: &[&str], label: &str) -> Result<()> {
    let expected: BTreeSet<_> = keys.iter().copied().collect();
    let actual: BTreeSet<_> = object.keys().map(String::as_str).collect();
    if expected != actual {
        return Err(VerifierError::new(format!(
            "{label} has unexpected or missing fields"
        )));
    }
    Ok(())
}

fn required_string(object: &Map<String, Value>, key: &str) -> Result<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| VerifierError::new(format!("missing or invalid string field: {key}")))
}

fn required_hex(object: &Map<String, Value>, key: &str, length: usize) -> Result<String> {
    let value = required_string(object, key)?;
    if !is_hex(&value, length) {
        return Err(VerifierError::new(format!("invalid digest field: {key}")));
    }
    Ok(value)
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && value == value.to_ascii_lowercase()
}

fn require_safe_id(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(VerifierError::new(format!("invalid {label}")));
    }
    Ok(())
}

fn require_check_id(value: &str) -> Result<()> {
    if value.len() != 7
        || !value.starts_with("CHK-")
        || value[4..].bytes().any(|byte| !byte.is_ascii_digit())
    {
        return Err(VerifierError::new(format!("invalid check id: {value}")));
    }
    Ok(())
}

fn task_id_from_dir(task_dir: &Path) -> Result<String> {
    let number = task_dir
        .file_name()
        .map(|value| value.to_string_lossy())
        .ok_or_else(|| VerifierError::new("task directory has no name"))?;
    if number.len() != 3 || number.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err(VerifierError::new("task directory name must be NNN"));
    }
    Ok(format!("TASK-{number}"))
}

fn absolute_path(path: &Path, label: &str) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(VerifierError::new(format!("{label} must be absolute")));
    }
    Ok(path.to_path_buf())
}

fn canonical_input_dir(path: &Path, label: &str) -> Result<PathBuf> {
    absolute_path(path, label)?
        .canonicalize()
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))
}

fn canonical_existing_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let path = absolute_path(path, label)?;
    if path.is_symlink() || !path.is_dir() {
        return Err(VerifierError::new(format!(
            "{label} must be a real directory"
        )));
    }
    path.canonicalize()
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))
}

fn regular_dir(path: &Path, label: &str) -> Result<PathBuf> {
    if path.is_symlink() || !path.is_dir() {
        return Err(VerifierError::new(format!("{label} is absent or unsafe")));
    }
    Ok(path.to_path_buf())
}

fn regular_file(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || link_count(&metadata) != 1
    {
        return Err(VerifierError::new(format!(
            "{label} is not an immutable regular file"
        )));
    }
    Ok(path.to_path_buf())
}

fn immutable_file(path: &Path, label: &str) -> Result<PathBuf> {
    let path = regular_file(path, label)?;
    let metadata = fs::symlink_metadata(&path)?;
    if !permissions_are_readonly(&metadata) {
        return Err(VerifierError::new(format!("{label} is writable")));
    }
    Ok(path)
}

fn link_count(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.nlink()
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        1
    }
}

fn regular_path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || !metadata.file_type().is_file()
                || link_count(&metadata) != 1
            {
                return Err(VerifierError::new("unsafe result path"));
            }
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn directory_names(path: &Path) -> Result<BTreeSet<String>> {
    fs::read_dir(path)?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .map_err(VerifierError::from)
        })
        .collect()
}

fn ensure_empty_dir(path: &Path, label: &str) -> Result<()> {
    if directory_names(path)?.is_empty() {
        Ok(())
    } else {
        Err(VerifierError::new(format!("{label} is not fresh")))
    }
}

fn set_readonly_file(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o444);
    }
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn set_readonly_dir(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o555);
    }
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn require_readonly_dir(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !permissions_are_readonly(&metadata) {
        return Err(VerifierError::new(format!("{label} is writable")));
    }
    Ok(())
}

fn permissions_are_readonly(metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o222 == 0
    }
    #[cfg(not(unix))]
    {
        metadata.permissions().readonly()
    }
}

fn write_json_new(path: &Path, value: &Value, label: &str) -> Result<String> {
    let raw = canonical_json(value).into_bytes();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| VerifierError::new(format!("write {label}: {error}")))?;
    file.write_all(&raw)?;
    file.sync_all()?;
    drop(file);
    set_readonly_file(path)?;
    Ok(sha256_bytes(&raw))
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(regular_file(path, "hash input")?)?))
}

fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0_u8; bytes];
    if let Ok(mut file) = File::open("/dev/urandom")
        && file.read_exact(&mut buffer).is_ok()
    {
        return buffer.iter().map(|byte| format!("{byte:02x}")).collect();
    }
    sha256_bytes(format!("{}:{:?}", std::process::id(), Instant::now()).as_bytes())[..bytes * 2]
        .to_string()
}

fn git_identity(worktree: &Path) -> Result<(String, String)> {
    let commit = git_output(worktree, &["rev-parse", "HEAD^{commit}"])?;
    let tree = git_output(worktree, &["rev-parse", "HEAD^{tree}"])?;
    if !is_hex(&commit, 40) || !is_hex(&tree, 40) {
        return Err(VerifierError::new(
            "candidate git identity is not a full object id",
        ));
    }
    Ok((commit, tree))
}

fn git_output(worktree: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(["-C", path_string(worktree).as_str()])
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(VerifierError::new(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let value =
        String::from_utf8(output.stdout).map_err(|error| VerifierError::new(error.to_string()))?;
    Ok(value.trim().to_string())
}

fn require_clean_candidate(worktree: &Path) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            path_string(worktree).as_str(),
            "status",
            "--porcelain",
            "--untracked-files=all",
        ])
        .output()?;
    if !output.status.success() || !output.stdout.is_empty() {
        return Err(VerifierError::new("candidate worktree is not clean"));
    }
    Ok(())
}

fn validate_scope_base(worktree: &Path, base: &str, candidate_commit: &str) -> Result<String> {
    if !is_hex(base, 40) {
        return Err(VerifierError::new(
            "scope base must be a full lowercase commit",
        ));
    }
    let resolved = git_output(
        worktree,
        &["rev-parse", "--verify", &format!("{base}^{{commit}}")],
    )?;
    if resolved != base {
        return Err(VerifierError::new(
            "scope base is not the explicit commit supplied",
        ));
    }
    let status = Command::new("git")
        .args([
            "-C",
            path_string(worktree).as_str(),
            "merge-base",
            "--is-ancestor",
            base,
            candidate_commit,
        ])
        .status()?;
    if !status.success() {
        return Err(VerifierError::new(
            "scope base is not an ancestor of candidate",
        ));
    }
    Ok(base.to_string())
}

#[derive(Debug, Clone)]
struct OracleIdentity {
    tag_ref_sha256: String,
    commit: String,
    tree: String,
}

fn oracle_identity(worktree: &Path, tag: &str, expected_commit: &str) -> Result<OracleIdentity> {
    if tag != "refs/tags/visual-baseline" || expected_commit != EXPECTED_ORACLE_COMMIT {
        return Err(VerifierError::new(
            "only the frozen visual-baseline oracle is permitted",
        ));
    }
    let tag_ref = git_output(worktree, &["rev-parse", tag])?;
    let commit = git_output(worktree, &["rev-parse", &format!("{tag}^{{commit}}")])?;
    let tree = git_output(worktree, &["rev-parse", &format!("{tag}^{{tree}}")])?;
    if commit != expected_commit {
        return Err(VerifierError::new(
            "oracle tag peel does not match expected commit",
        ));
    }
    Ok(OracleIdentity {
        tag_ref_sha256: sha256_bytes(tag_ref.as_bytes()),
        commit,
        tree,
    })
}

fn executable_identity(path: &Path, label: &str) -> Result<Value> {
    let path = regular_file(path, label)?;
    if !is_executable(&path)? {
        return Err(VerifierError::new(format!("{label} is not executable")));
    }
    Ok(json!({"path": path_string(&path), "sha256": hash_file(&path)?}))
}

fn is_executable(path: &Path) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(fs::metadata(path)?.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(true)
    }
}

fn dependency_receipts(
    task_dir: &Path,
    task_meta: &Path,
    receipt_paths: &[PathBuf],
    worktree: &Path,
) -> Result<Vec<Value>> {
    let mut expected = BTreeSet::new();
    if task_meta.exists() {
        let raw = fs::read(regular_file(task_meta, "task.toml")?)?;
        let value: toml::Value = toml::from_str(
            std::str::from_utf8(&raw).map_err(|error| VerifierError::new(error.to_string()))?,
        )
        .map_err(|error| VerifierError::new(error.to_string()))?;
        if let Some(dependencies) = value.get("dependencies").and_then(toml::Value::as_array) {
            for dependency in dependencies {
                let dependency = dependency
                    .as_str()
                    .ok_or_else(|| VerifierError::new("task dependency is not a string"))?;
                expected.insert(normalize_task_id(dependency));
            }
        }
    }
    if expected.is_empty() && !receipt_paths.is_empty() {
        return Err(VerifierError::new(
            "dependency receipts supplied for task with no dependencies",
        ));
    }
    let mut actual = BTreeSet::new();
    let mut values = Vec::new();
    for path in receipt_paths {
        let path = absolute_path(path, "dependency receipt")?;
        let path = regular_file(&path, "dependency receipt")?;
        let raw = fs::read(&path)?;
        let value = parse_json_object(&raw, "dependency receipt")?;
        let task_id = value
            .get("task_id")
            .or_else(|| value.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| VerifierError::new("dependency receipt has no task id"))?;
        let normalized = normalize_task_id(task_id);
        if !expected.contains(&normalized) || !actual.insert(normalized.clone()) {
            return Err(VerifierError::new(
                "dependency receipt is unexpected or duplicate",
            ));
        }
        let accepted = value
            .get("accepted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value.get("verdict").and_then(Value::as_str) == Some("VERIFIED")
            || value.get("status").and_then(Value::as_str) == Some("VERIFIED");
        let integrated = value
            .get("integrated")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !accepted || !integrated {
            return Err(VerifierError::new(
                "dependency receipt is not accepted and integrated",
            ));
        }
        let ancestry = value
            .get("integration_commit")
            .or_else(|| value.get("commit"))
            .and_then(Value::as_str)
            .ok_or_else(|| VerifierError::new("dependency receipt has no integration commit"))?;
        if !is_hex(ancestry, 40)
            || !Command::new("git")
                .args([
                    "-C",
                    path_string(worktree).as_str(),
                    "merge-base",
                    "--is-ancestor",
                    ancestry,
                    "HEAD",
                ])
                .status()?
                .success()
        {
            return Err(VerifierError::new(
                "dependency receipt ancestry is not in candidate",
            ));
        }
        values.push(json!({"task_id": task_id, "path": path_string(&path), "sha256": sha256_bytes(&raw), "accepted": true, "integrated": true, "integration_commit": ancestry}));
    }
    if actual != expected {
        return Err(VerifierError::new(
            "dependency receipt set does not match task dependencies",
        ));
    }
    let _ = task_dir;
    Ok(values)
}

fn normalize_task_id(value: &str) -> String {
    let leaf = value.rsplit('/').next().unwrap_or(value);
    leaf.strip_prefix("TASK-")
        .or_else(|| leaf.strip_prefix("task-"))
        .map_or_else(|| format!("TASK-{leaf}"), |suffix| format!("TASK-{suffix}"))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_output_path(common: &Value, check_id: &str, suffix: &str) -> Result<PathBuf> {
    require_check_id(check_id)?;
    let outputs = common
        .get("outputs")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("common output binding is missing"))?;
    let root = absolute_path(
        Path::new(required_string(outputs, "runtime")?.as_str()),
        "runtime output directory",
    )?;
    let path = root.join(format!("{check_id}.{suffix}"));
    if path.parent() != Some(root.as_path()) {
        return Err(VerifierError::new(
            "runtime output escaped its bound directory",
        ));
    }
    Ok(path)
}

fn read_bounded<R: Read>(reader: R) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    reader
        .take((MAX_CHILD_OUTPUT_BYTES + 1) as u64)
        .read_to_end(&mut output)?;
    if output.len() > MAX_CHILD_OUTPUT_BYTES {
        return Err(VerifierError::new("child output exceeded bounded limit"));
    }
    Ok(output)
}

fn join_output(handle: thread::JoinHandle<Result<Vec<u8>>>) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| VerifierError::new("child output reader panicked"))?
}

fn successful_exit(status: ExitStatus) -> Result<i32> {
    status
        .code()
        .filter(|code| *code == 0)
        .ok_or_else(|| VerifierError::new("child exited nonzero or by signal"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Stdio;

    fn observer_request(
        nonce: &str,
        request_id: u64,
        operation: &str,
        source_commit: &str,
        tree: &str,
    ) -> String {
        canonical_json(&json!({
            "schema": "tc-proof-runner-observe/v1",
            "nonce": nonce,
            "run_id": "/run",
            "task_id": "TASK-001",
            "check_id": "CHK-001",
            "request_id": request_id,
            "operation": operation,
            "source_commit": source_commit,
            "tree": tree,
        })) + "\n"
    }

    #[test]
    fn context_reference_requires_matching_check() {
        let error = discover_context_reference(
            "tc-proof preflight --context $RUN_DIR/contexts/CHK-002.json",
            "CHK-001",
        );
        assert!(error.is_err());
    }

    #[test]
    fn output_limit_is_enforced() {
        let data = vec![0_u8; MAX_CHILD_OUTPUT_BYTES + 1];
        assert!(read_bounded(data.as_slice()).is_err());
    }

    #[test]
    fn read_only_files_have_no_write_bits() {
        let Ok(directory) = tempfile::tempdir() else {
            return;
        };
        let path = directory.path().join("file");
        if fs::write(&path, b"x").is_err() || set_readonly_file(&path).is_err() {
            return;
        }
        let Ok(metadata) = fs::metadata(path) else {
            return;
        };
        assert_eq!(metadata.permissions().mode() & 0o222, 0);
    }

    struct Fixture {
        root: tempfile::TempDir,
        run_dir: PathBuf,
    }

    fn fixture() -> std::result::Result<(Fixture, PreparedRun), String> {
        let root = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let candidate = root.path().join("candidate");
        let clone = Command::new("git")
            .args(["clone", "--local", "--no-hardlinks"])
            .arg(&source)
            .arg(&candidate)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| error.to_string())?;
        if !clone.status.success() {
            return Err(String::from_utf8_lossy(&clone.stderr).into_owned());
        }
        let scope =
            git_output(&candidate, &["rev-parse", "HEAD"]).map_err(|error| error.to_string())?;
        let run_dir = root.path().join("run");
        fs::create_dir(&run_dir).map_err(|error| error.to_string())?;
        let options = PrepareOptions {
            task_dir: candidate.join("refactoring-tasks/terminal-components/completion/001"),
            run_dir: run_dir.clone(),
            worktree: candidate,
            scope_base: scope,
            oracle_tag: "refs/tags/visual-baseline".to_string(),
            oracle_commit: EXPECTED_ORACLE_COMMIT.to_string(),
            tool: PathBuf::from("/usr/bin/true"),
            comparator: PathBuf::from("/usr/bin/true"),
            taskfmt: None,
            dependency_receipts: Vec::new(),
            run_id: None,
            observer_nonce: Some("test-observer".to_string()),
            observer_socket: None,
        };
        let prepared = prepare(&options).map_err(|error| error.to_string())?;
        Ok((Fixture { root, run_dir }, prepared))
    }

    #[test]
    fn materialization_has_exact_index_bound_context_set() {
        let (fixture, prepared) = fixture().expect("fixture");
        let index_raw = fs::read(fixture.run_dir.join("context-index.json")).expect("index");
        let index = parse_json_object(&index_raw, "index").expect("parse index");
        let index_members = index
            .get("contexts")
            .and_then(Value::as_array)
            .expect("contexts");
        assert_eq!(
            index.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "schema",
                "task_id",
                "run_id",
                "worktree_commit",
                "scope_base",
                "contexts",
                "results",
                "observer",
            ])
        );
        let expected: BTreeSet<_> = index_members
            .iter()
            .filter_map(|member| member.get("check_id").and_then(Value::as_str))
            .map(|id| format!("{id}.json"))
            .collect();
        let Ok(actual) = directory_names(&fixture.run_dir.join("contexts")) else {
            return;
        };
        assert_eq!(actual, expected);
        assert_eq!(prepared.members.len(), expected.len());
        assert!(validate_run(&fixture.run_dir).is_ok());
    }

    #[test]
    fn materialization_uses_ledger_preparation_abi() {
        let (fixture, _) = fixture().expect("fixture");
        let index = parse_json_object(
            &fs::read(fixture.run_dir.join("context-index.json")).expect("index"),
            "index",
        )
        .expect("parse index");
        let context_path = index["contexts"][0]["path"].as_str().expect("context path");
        let result_path = index["results"][0]["path"].as_str().expect("result path");
        let context = parse_json_object(&fs::read(context_path).expect("context"), "context")
            .expect("parse context");
        let result = parse_json_object(&fs::read(result_path).expect("result"), "result")
            .expect("parse result");
        let observer = parse_json_object(
            &fs::read(fixture.run_dir.join("observer.json")).expect("observer"),
            "observer",
        )
        .expect("parse observer");
        assert_eq!(context["schema"], CONTEXT_SCHEMA);
        assert_eq!(result["schema"], PREPARATION_RESULT_SCHEMA);
        assert_eq!(observer["schema"], OBSERVER_SCHEMA);
        assert_eq!(index["schema"], INDEX_SCHEMA);
    }

    #[test]
    fn legacy_runner_index_shape_is_rejected_by_native_validator() {
        let (fixture, _) = fixture().expect("fixture");
        let index_path = fixture.run_dir.join("context-index.json");
        let mut permissions = fs::metadata(&index_path)
            .expect("index metadata")
            .permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&index_path, permissions).expect("make index writable");
        fs::write(
            &index_path,
            serde_json::to_vec(&json!({
                "schema": INDEX_SCHEMA,
                "task_id": "TASK-001",
                "run_id": fixture.run_dir.to_string_lossy(),
                "tree": "deadbeef",
                "trust_sha256": "deadbeef",
                "members": [],
            }))
            .expect("legacy index JSON"),
        )
        .expect("write legacy index");
        assert!(validate_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn extra_runtime_output_is_rejected() {
        let (fixture, _) = fixture().expect("fixture");
        fs::write(fixture.run_dir.join("outputs/extra.json"), b"{}").expect("extra output");
        assert!(validate_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn materialization_rejects_context_symlink_and_hardlink() {
        let (fixture, prepared) = fixture().expect("fixture");
        let member = prepared.members.first().expect("first member");
        let context_path = member.context_path.clone();
        let contexts_dir = context_path.parent().expect("contexts parent");
        let mut permissions = fs::metadata(contexts_dir)
            .expect("contexts metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(contexts_dir, permissions).expect("make contexts writable");
        let alias = fixture.root.path().join("context-alias");
        fs::hard_link(&context_path, &alias).expect("hard link");
        assert!(validate_run(&fixture.run_dir).is_err());
        let _ = fs::remove_file(&alias);
        let other = prepared
            .members
            .get(1)
            .map(|member| member.context_path.clone())
            .expect("second member");
        fs::remove_file(&context_path).expect("remove context");
        std::os::unix::fs::symlink(other, &context_path).expect("symlink");
        assert!(validate_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn index_hash_and_candidate_identity_mutations_fail_closed() {
        let (fixture, _) = fixture().expect("fixture");
        let index_path = fixture.run_dir.join("context-index.json");
        let mut permissions = fs::metadata(&index_path)
            .expect("index metadata")
            .permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&index_path, permissions).expect("make index writable");
        fs::write(&index_path, b"{}").expect("mutate index");
        assert!(validate_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn subprocess_observation_collects_bound_result() {
        let (fixture, prepared) = fixture().expect("fixture");
        let member = prepared.members.first().expect("first member");
        let script = r###"printf '{"category":null,"context_sha256":"%s","observation_digests":[],"operation":"external","outputs":{},"run_id":"%s","schema":"tc-proof-runner-result/v1","status":"passed"}' "$TC_PROOF_CONTEXT_SHA256" "$TC_PROOF_RUN_ID" > "$TC_PROOF_RESULT""###;
        let record = launch(&LaunchOptions {
            run_dir: fixture.run_dir.clone(),
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_secs(5),
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".to_string(), script.to_string()],
        });
        assert!(record.is_ok());
        let metadata = fs::symlink_metadata(&member.result_path).expect("result metadata");
        assert!(permissions_are_readonly(&metadata));
        assert!(
            launch(&LaunchOptions {
                run_dir: fixture.run_dir.clone(),
                check_id: member.check_id.clone(),
                observer_socket: None,
                timeout: Duration::from_secs(5),
                program: PathBuf::from("/bin/sh"),
                args: vec!["-c".to_string(), script.to_string()],
            })
            .is_err()
        );
    }

    #[test]
    fn subprocess_nonzero_and_timeout_are_rejected() {
        let (fixture, prepared) = fixture().expect("fixture");
        let member = prepared.members.get(1).expect("second member");
        let nonzero = launch(&LaunchOptions {
            run_dir: fixture.run_dir.clone(),
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_secs(5),
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".to_string(), "exit 7".to_string()],
        });
        assert!(nonzero.is_err());
        let timeout = launch(&LaunchOptions {
            run_dir: fixture.run_dir,
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_millis(20),
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".to_string(), "sleep 1".to_string()],
        });
        assert!(timeout.is_err());
    }

    #[test]
    fn alternate_observer_socket_binding_is_rejected() {
        let (fixture, prepared) = fixture().expect("fixture");
        let member = prepared.members.first().expect("first member");
        let error = launch(&LaunchOptions {
            run_dir: fixture.run_dir,
            check_id: member.check_id.clone(),
            observer_socket: Some(PathBuf::from("/tmp/retired-observer.sock")),
            timeout: Duration::from_secs(5),
            program: PathBuf::from("/bin/true"),
            args: Vec::new(),
        })
        .expect_err("retired socket should be rejected");
        assert!(error.to_string().contains("alternate observer"));
    }

    #[test]
    fn observer_protocol_accepts_bound_request_and_complete_close() {
        let (request_read, mut request_write) = make_pipe().expect("request pipe");
        let (response_read, response_write) = make_pipe().expect("response pipe");
        let mut response_read = BufReader::new(response_read);
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                observe_worker(
                    request_read,
                    response_write,
                    "/run",
                    "TASK-001",
                    "CHK-001",
                    "capture",
                    &tree,
                    &source_commit,
                    "nonce",
                )
            }
        });
        request_write
            .write_all(observer_request("nonce", 0, "capture", &source_commit, &tree).as_bytes())
            .expect("request");
        request_write.flush().expect("flush request");
        let mut response = String::new();
        response_read
            .read_line(&mut response)
            .expect("response line");
        let response = parse_json_object(response.trim_end().as_bytes(), "observer response")
            .expect("response JSON");
        assert_eq!(response["schema"], "tc-proof-observation/v1");
        assert_eq!(response["request_id"], 0);
        drop(request_write);
        assert!(supervisor.join().expect("supervisor join").is_ok());
    }

    #[test]
    fn observer_protocol_rejects_wrong_nonce() {
        let (request_read, mut request_write) = make_pipe().expect("request pipe");
        let (response_read, response_write) = make_pipe().expect("response pipe");
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                observe_worker(
                    request_read,
                    response_write,
                    "/run",
                    "TASK-001",
                    "CHK-001",
                    "capture",
                    &tree,
                    &source_commit,
                    "nonce",
                )
            }
        });
        request_write
            .write_all(observer_request("wrong", 0, "capture", &source_commit, &tree).as_bytes())
            .expect("request");
        request_write.flush().expect("flush request");
        drop(request_write);
        drop(response_read);
        let error = supervisor
            .join()
            .expect("supervisor join")
            .expect_err("wrong nonce");
        assert!(error.to_string().contains("binding mismatch"));
    }

    #[test]
    fn observer_protocol_rejects_truncation_and_replay() {
        let (request_read, mut request_write) = make_pipe().expect("request pipe");
        let (response_read, response_write) = make_pipe().expect("response pipe");
        let supervisor = thread::spawn(move || {
            observe_worker(
                request_read,
                response_write,
                "/run",
                "TASK-001",
                "CHK-001",
                "capture",
                &"b".repeat(40),
                &"a".repeat(40),
                "nonce",
            )
        });
        request_write
            .write_all(b"{\"schema\":\"tc-proof-runner-observe/v1\"")
            .expect("truncated request");
        drop(request_write);
        drop(response_read);
        let error = supervisor
            .join()
            .expect("supervisor join")
            .expect_err("truncation");
        assert!(error.to_string().contains("truncated"));

        let (request_read, mut request_write) = make_pipe().expect("replay request pipe");
        let (response_read, response_write) = make_pipe().expect("replay response pipe");
        let mut response_read = BufReader::new(response_read);
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                observe_worker(
                    request_read,
                    response_write,
                    "/run",
                    "TASK-001",
                    "CHK-001",
                    "capture",
                    &tree,
                    &source_commit,
                    "nonce",
                )
            }
        });
        let request = observer_request("nonce", 0, "capture", &source_commit, &tree);
        request_write
            .write_all(request.as_bytes())
            .expect("request");
        request_write.write_all(request.as_bytes()).expect("replay");
        request_write.flush().expect("flush request");
        let mut response = String::new();
        response_read
            .read_line(&mut response)
            .expect("replay response");
        drop(request_write);
        drop(response_read);
        let error = supervisor
            .join()
            .expect("supervisor join")
            .expect_err("replay");
        assert!(error.to_string().contains("replay"));
    }
}
