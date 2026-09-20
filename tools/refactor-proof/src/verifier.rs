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
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::json_util::{canonical_json, parse_json_bytes_strict, sha256_bytes, sha256_canonical};

/// The ledger-owned preparation ABI.  Runner-specific fields live below this
/// envelope; the index, contexts, preparation results, and observer capability
/// all use this same binding.
const CONTEXT_SCHEMA: &str = "tc-proof-context/v1";
const COMPARE_CONTEXT_SCHEMA: &str = "tc-proof-compare-context/v1";
const INDEX_SCHEMA: &str = "tc-proof-context-index/v1";
const RESULT_SCHEMA: &str = "tc-proof-runner-result/v1";
const PREPARATION_RESULT_SCHEMA: &str = "tc-proof-preparation-result/v1";
const PREPARATION_RECEIPT_SCHEMA: &str = "campaign-proof-preparation/v1";
const OBSERVER_SCHEMA: &str = "tc-proof-observer-capability/v1";
const NATIVE_HANDOFF_SCHEMA: &str = "tc-proof-native-handoff/v1";
const NATIVE_BUILD_SCHEMA: &str = "tc-proof-native-build/v1";
const PREPARATION_RECEIPT_FILE: &str = "proof-preparation.json";
const NATIVE_TARGET_DIR_NAME: &str = "target";
const OBSERVER_PROVIDER_ENV: &str = "TC_PROOF_OBSERVER_PROVIDER";
// Native proof workers and the independent observer provider must not resolve
// interpreters through candidate-controlled configuration managers.  The
// verified comparator and all proof outputs use absolute paths; the worker
// and provider only need the host-standard Unix tools below.
const NATIVE_EXECUTION_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";
const EXPECTED_ORACLE_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const QUALIFIED_TASKFMT_REVISION: &str = "afd3b575dbcc7044620bec4b9493a74eca3e5ef2";
const QUALIFIED_TASKFMT_VERSION: &str = "0.2.0";
const QUALIFIED_TASKFMT_SHA256: &str =
    "f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de";
const MAX_LAUNCH_TIMEOUT_MS: u64 = 600_000;
const MAX_CHILD_OUTPUT_BYTES: usize = 32 * 1024 * 1024;
const CLEANUP_POLL_INTERVAL: Duration = Duration::from_millis(5);
const OBSERVER_CLEANUP_TIMEOUT: Duration = Duration::from_secs(1);
const OBSERVER_PROVIDER_GRACE_TIMEOUT: Duration = Duration::from_secs(1);

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
    /// Native build receipt produced by campaign-build-proof.sh.
    pub native_build_receipt: PathBuf,
    /// Clean source checkout for the qualified standalone taskfmt.
    pub taskfmt_source: PathBuf,
    /// Full taskfmt source revision.
    pub taskfmt_revision: String,
    /// Qualified taskfmt version.
    pub taskfmt_version: String,
    /// Qualified taskfmt executable SHA-256.
    pub taskfmt_sha256: String,
    /// Independent observer response provider executable.
    pub observer_provider: PathBuf,
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
    observer_sequence: Vec<String>,
    context_path: PathBuf,
    context_sha256: String,
    preparation_result_path: PathBuf,
    result_path: PathBuf,
    comparator_report_path: Option<PathBuf>,
}

/// Spawn-time process-group id. Captured immediately after `spawn` so later
/// `wait` reaping cannot redirect group signals through a reused `Child::id()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProcessGroupId(u32);

impl ProcessGroupId {
    fn from_child(child: &Child) -> Result<Self> {
        let pid = child.id();
        if pid == 0 || pid == 1 {
            return Err(VerifierError::new(
                "child pid is not a valid process-group id",
            ));
        }
        Ok(Self(pid))
    }
}

struct ObserverSupervisor {
    request_write: Option<File>,
    response_read: Option<File>,
    provider: Arc<Mutex<Child>>,
    provider_group: ProcessGroupId,
    thread: Option<thread::JoinHandle<Result<Vec<String>>>>,
}

impl ObserverSupervisor {
    fn worker_fds(&self) -> Result<(i32, i32)> {
        let request_write = self
            .request_write
            .as_ref()
            .ok_or_else(|| VerifierError::new("observer request pipe is closed"))?;
        let response_read = self
            .response_read
            .as_ref()
            .ok_or_else(|| VerifierError::new("observer response pipe is closed"))?;
        Ok((request_write.as_raw_fd(), response_read.as_raw_fd()))
    }

    fn close_worker_ends(&mut self) {
        self.request_write.take();
        self.response_read.take();
    }

    fn join_thread_until(&mut self, deadline: Instant) -> Result<Vec<String>> {
        let thread = self
            .thread
            .take()
            .ok_or_else(|| VerifierError::new("observer supervisor thread is missing"))?;
        join_result_thread(thread, deadline, "observer supervisor")
    }

    fn kill_provider_until(&self, deadline: Instant) -> Result<()> {
        let mut provider = lock_provider_until(&self.provider, deadline)?;
        terminate_child_until(&mut provider, self.provider_group, deadline).map(|_| ())
    }

    fn wait(mut self, timeout: Duration) -> Result<Vec<String>> {
        self.close_worker_ends();
        let deadline = cleanup_deadline(timeout, "observer supervisor")?;
        loop {
            if self
                .thread
                .as_ref()
                .is_some_and(thread::JoinHandle::is_finished)
            {
                return self.join_thread_until(Instant::now());
            }
            if Instant::now() >= deadline {
                let cleanup_deadline =
                    cleanup_deadline(OBSERVER_CLEANUP_TIMEOUT, "observer supervisor cleanup")?;
                let provider_error = self.kill_provider_until(cleanup_deadline).err();
                let thread_result = self.join_thread_until(cleanup_deadline);
                return match provider_error {
                    None => thread_result,
                    Some(provider_error) => match thread_result {
                        Ok(_) => Err(provider_error),
                        Err(thread_error) => Err(VerifierError::new(format!(
                            "{provider_error}; observer supervisor cleanup: {thread_error}"
                        ))),
                    },
                };
            }
            thread::sleep(CLEANUP_POLL_INTERVAL);
        }
    }

    fn abort(self) -> Result<()> {
        let deadline = cleanup_deadline(OBSERVER_CLEANUP_TIMEOUT, "observer supervisor abort")?;
        self.abort_until(deadline)
    }

    fn abort_until(mut self, deadline: Instant) -> Result<()> {
        self.close_worker_ends();
        let provider_error = self.kill_provider_until(deadline).err();
        let thread_error = self.join_thread_until(deadline).map(|_| ()).err();
        let errors = [provider_error, thread_error]
            .into_iter()
            .flatten()
            .collect();
        combine_cleanup_errors(errors)
    }
}

struct ObserverBinding<'a> {
    run_id: &'a str,
    task_id: &'a str,
    check_id: &'a str,
    tree: &'a str,
    oracle_commit: &'a str,
    nonce: &'a str,
    sequence: &'a [String],
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
///
/// # Errors
///
/// Returns a verifier error when any bound input is missing, inconsistent,
/// mutable, or cannot be materialized in the external run directory.
#[expect(
    clippy::too_many_lines,
    reason = "one preparation pipeline keeps all immutable bindings and materialization steps reviewable together"
)]
pub fn prepare(options: &PrepareOptions) -> Result<PreparedRun> {
    let task_dir = canonical_input_dir(&options.task_dir, "task directory")?;
    let worktree = canonical_input_dir(&options.worktree, "candidate worktree")?;
    let run_dir = canonical_existing_dir(&options.run_dir, "run directory")?;
    if worktree == run_dir || run_dir.starts_with(&worktree) {
        return Err(VerifierError::new(
            "run directory must be external to worktree",
        ));
    }
    let native_target = native_target_from_receipt_path(&options.native_build_receipt, &run_dir)?;
    ensure_run_directory_for_preparation(&run_dir, &native_target)?;
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

    let expected_tool = regular_file(
        &worktree.join("tools/refactor-proof/bin/tc-proof"),
        "candidate proof tool",
    )?
    .canonicalize()?;
    let requested_tool = regular_file(&options.tool, "proof tool")?.canonicalize()?;
    if requested_tool != expected_tool {
        return Err(VerifierError::new(
            "proof tool is not the candidate worktree worker",
        ));
    }
    let tool = executable_identity(&options.tool, "proof tool")?;
    let comparator = executable_identity(&options.comparator, "native comparator")?;
    let observer_provider =
        executable_identity(&options.observer_provider, "observer response provider")?;
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
            "provider": observer_provider,
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
        let observer_sequence = observer_sequence_for_check(check, template)?;
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
            observer_sequence,
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
    let observer_sequences = members
        .iter()
        .map(|member| {
            (
                member.check_id.clone(),
                Value::Array(
                    member
                        .observer_sequence
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            )
        })
        .collect::<Map<String, Value>>();
    let observer_capability = json!({
        "schema": OBSERVER_SCHEMA,
        "task_id": verify.task_id,
        "run_id": run_id,
        "worktree_commit": candidate_commit,
        "scope_base": scope_base,
        "transport": "inherited-pipe/v1",
        "nonce_sha256": sha256_bytes(observer_nonce.as_bytes()),
        "sequences": observer_sequences,
        "provider": observer_provider,
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
        "observer_sequences": observer_sequences,
        "observer": {"path": path_string(&observer_path), "sha256": observer_sha256},
    });
    write_json_new(&run_dir.join("context-index.json"), &index, "context index")?;
    // Validate the complete materialized set before publishing the external
    // preparation receipt.  The receipt is evidence about this already
    // validated run; it cannot make an invalid run valid.
    let prepared = validate_run_inner(&run_dir, false, Some(&native_target), false)?;
    write_preparation_receipt(options, &prepared)?;
    validate_run_inner(&run_dir, true, None, false)
}

#[expect(
    clippy::too_many_lines,
    reason = "receipt construction keeps all schema bindings in one fail-closed boundary"
)]
fn write_preparation_receipt(options: &PrepareOptions, prepared: &PreparedRun) -> Result<()> {
    let index_path = immutable_file(
        &prepared.run_dir.join("context-index.json"),
        "context index",
    )?;
    let index_raw = fs::read(&index_path)?;
    let index = parse_json_object(&index_raw, "context index")?;
    let candidate_commit = required_hex(&index, "worktree_commit", 40)?;
    let scope_base = required_hex(&index, "scope_base", 40)?;
    let context_entries = index
        .get("contexts")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new("context index contexts are missing"))?;
    let first_context = context_entries
        .first()
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context index has no context"))?;
    let first_context_path = bound_artifact_path(
        first_context,
        &prepared.run_dir.join("contexts"),
        &format!("{}.json", required_string(first_context, "check_id")?),
        "context",
    )?;
    let context = parse_json_object(&fs::read(&first_context_path)?, "context")?;
    let common = context
        .get("qualification")
        .and_then(Value::as_object)
        .and_then(|qualification| qualification.get("common"))
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context common binding is missing"))?;
    let worktree = canonical_input_dir(
        Path::new(required_string(common, "worktree")?.as_str()),
        "candidate worktree",
    )?;
    let expected_worktree = canonical_input_dir(&options.worktree, "candidate worktree")?;
    if worktree != expected_worktree {
        return Err(VerifierError::new(
            "preparation context worktree differs from requested worktree",
        ));
    }
    let (current_commit, current_tree) = git_identity(&worktree)?;
    if candidate_commit != current_commit || prepared.candidate_tree != current_tree {
        return Err(VerifierError::new(
            "preparation candidate identity changed before receipt publication",
        ));
    }
    let comparator = common
        .get("comparator")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context comparator binding is missing"))?;
    let comparator_path = regular_file(
        Path::new(required_string(comparator, "path")?.as_str()),
        "native comparator",
    )?
    .canonicalize()?;
    let requested_comparator =
        regular_file(&options.comparator, "native comparator")?.canonicalize()?;
    if comparator_path != requested_comparator {
        return Err(VerifierError::new(
            "preparation comparator differs from context comparator",
        ));
    }
    let native_build = native_build_binding(
        &options.native_build_receipt,
        &prepared.run_dir,
        &worktree,
        &candidate_commit,
        &prepared.candidate_tree,
        &comparator_path,
    )?;
    let taskfmt = taskfmt_provenance(options)?;
    let context_index = json!({
        "path": path_string(&index_path),
        "sha256": sha256_bytes(&index_raw),
    });
    let contexts = Value::Array(context_entries.clone());
    let results = index
        .get("results")
        .cloned()
        .ok_or_else(|| VerifierError::new("context index results are missing"))?;
    let observer = index
        .get("observer")
        .cloned()
        .ok_or_else(|| VerifierError::new("context index observer is missing"))?;
    let receipt = json!({
        "schema": PREPARATION_RECEIPT_SCHEMA,
        "task_id": prepared.task_id,
        "worktree": path_string(&worktree),
        "commit": candidate_commit,
        "scope_base": scope_base,
        "run_id": prepared.run_id,
        "native_build": native_build,
        "taskfmt": taskfmt,
        "context_index": context_index,
        "contexts": contexts,
        "results": results,
        "observer": observer,
    });
    write_json_atomic_new(
        &prepared.run_dir.join(PREPARATION_RECEIPT_FILE),
        &receipt,
        "proof preparation receipt",
    )?;
    Ok(())
}

fn taskfmt_provenance(options: &PrepareOptions) -> Result<Value> {
    let taskfmt_path = options
        .taskfmt
        .as_ref()
        .ok_or_else(|| VerifierError::new("taskfmt is required for proof preparation"))?;
    let taskfmt_path = regular_file(taskfmt_path, "taskfmt")?.canonicalize()?;
    if !is_executable(&taskfmt_path)? {
        return Err(VerifierError::new("taskfmt is not executable"));
    }
    if !is_hex(&options.taskfmt_revision, 40)
        || options.taskfmt_version.is_empty()
        || !is_hex(&options.taskfmt_sha256, 64)
    {
        return Err(VerifierError::new("taskfmt provenance is malformed"));
    }
    if options.taskfmt_revision != QUALIFIED_TASKFMT_REVISION
        || options.taskfmt_version != QUALIFIED_TASKFMT_VERSION
        || options.taskfmt_sha256 != QUALIFIED_TASKFMT_SHA256
    {
        return Err(VerifierError::new(
            "taskfmt identity is not the qualified standalone pin",
        ));
    }
    let source = canonical_input_dir(&options.taskfmt_source, "taskfmt source")?;
    let (revision, _) = git_identity(&source)?;
    if revision != options.taskfmt_revision || revision != QUALIFIED_TASKFMT_REVISION {
        return Err(VerifierError::new("taskfmt source revision mismatch"));
    }
    require_clean_candidate(&source)?;
    let actual_hash = hash_file(&taskfmt_path)?;
    if actual_hash != options.taskfmt_sha256 || actual_hash != QUALIFIED_TASKFMT_SHA256 {
        return Err(VerifierError::new("taskfmt executable hash mismatch"));
    }
    let observed = observed_taskfmt_version(&taskfmt_path)?;
    let expected_observed =
        format!("taskfmt {QUALIFIED_TASKFMT_VERSION} (git {QUALIFIED_TASKFMT_REVISION})");
    if observed != expected_observed {
        return Err(VerifierError::new(format!(
            "taskfmt --version is '{observed}'; expected '{expected_observed}'"
        )));
    }
    Ok(json!({
        "taskfmt_revision": QUALIFIED_TASKFMT_REVISION,
        "taskfmt_version": QUALIFIED_TASKFMT_VERSION,
        "taskfmt_sha256": actual_hash,
        "taskfmt_source": path_string(&source),
        "taskfmt_path": path_string(&taskfmt_path),
    }))
}

fn observed_taskfmt_version(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("--version")
        .env("PATH", NATIVE_EXECUTION_PATH)
        .output()
        .map_err(|error| VerifierError::new(format!("taskfmt --version failed: {error}")))?;
    if !output.status.success() {
        return Err(VerifierError::new(
            "taskfmt --version exited unsuccessfully",
        ));
    }
    let observed = String::from_utf8(output.stdout)
        .map_err(|error| VerifierError::new(format!("taskfmt --version is not UTF-8: {error}")))?;
    let observed = observed.trim();
    if observed.is_empty() || observed.contains('\n') {
        return Err(VerifierError::new("taskfmt --version is malformed"));
    }
    Ok(observed.to_string())
}

fn native_build_binding(
    receipt_path: &Path,
    run_dir: &Path,
    worktree: &Path,
    candidate_commit: &str,
    candidate_tree: &str,
    comparator_path: &Path,
) -> Result<Value> {
    let receipt_path = regular_file(receipt_path, "native build receipt")?.canonicalize()?;
    let debug_dir = receipt_path
        .parent()
        .ok_or_else(|| VerifierError::new("native build receipt has no parent"))?;
    let target_dir = canonical_existing_dir(
        debug_dir
            .parent()
            .ok_or_else(|| VerifierError::new("native build receipt has no target parent"))?,
        "native build target",
    )?;
    require_native_target_dir_name(&target_dir)?;
    if target_dir == *run_dir || !target_dir.starts_with(run_dir) {
        return Err(VerifierError::new(
            "native build target must be inside the external run directory",
        ));
    }
    if receipt_path != target_dir.join("debug/tc-proof.build.json") {
        return Err(VerifierError::new(
            "native build receipt is not target/debug/tc-proof.build.json",
        ));
    }
    let raw = fs::read(&receipt_path)?;
    let build = parse_json_object(&raw, "native build receipt")?;
    exact_keys(
        &build,
        &[
            "schema",
            "worktree",
            "target_dir",
            "cargo_target_dir",
            "commit",
            "tree",
            "binary",
            "binary_sha256",
            "command",
        ],
        "native build receipt",
    )?;
    if required_string(&build, "schema")? != NATIVE_BUILD_SCHEMA {
        return Err(VerifierError::new("native build receipt schema mismatch"));
    }
    let build_worktree = canonical_input_dir(
        Path::new(required_string(&build, "worktree")?.as_str()),
        "native build worktree",
    )?;
    let build_commit = required_string(&build, "commit")?;
    let build_tree = required_string(&build, "tree")?;
    let build_target = required_string(&build, "target_dir")?;
    let build_cargo_target = required_string(&build, "cargo_target_dir")?;
    if build_worktree != *worktree
        || build_commit != candidate_commit
        || build_tree != candidate_tree
        || build_target != path_string(&target_dir)
        || build_cargo_target != path_string(&target_dir)
    {
        return Err(VerifierError::new(format!(
            "native build receipt identity mismatch: worktree={}/{}, commit={build_commit}/{candidate_commit}, tree={build_tree}/{candidate_tree}, target={build_target}/{target}, cargo_target={build_cargo_target}/{target}",
            build_worktree.display(),
            worktree.display(),
            target = path_string(&target_dir),
        )));
    }
    let binary_path = regular_file(
        Path::new(required_string(&build, "binary")?.as_str()),
        "native proof binary",
    )?
    .canonicalize()?;
    let expected_binary = target_dir.join("debug/tc-proof");
    if binary_path != expected_binary || binary_path != *comparator_path {
        return Err(VerifierError::new(
            "native build binary does not match comparator",
        ));
    }
    if !is_executable(&binary_path)? {
        return Err(VerifierError::new("native proof binary is not executable"));
    }
    let binary_sha256 = hash_file(&binary_path)?;
    if required_string(&build, "binary_sha256")? != binary_sha256 {
        return Err(VerifierError::new("native proof binary hash mismatch"));
    }
    Ok(json!({
        "receipt": path_string(&receipt_path),
        "receipt_sha256": sha256_bytes(&raw),
        "binary": path_string(&binary_path),
        "binary_sha256": binary_sha256,
    }))
}

#[expect(
    clippy::too_many_lines,
    reason = "receipt validation is one complete fail-closed boundary"
)]
fn validate_preparation_receipt(prepared: &PreparedRun) -> Result<()> {
    let receipt_path = immutable_file(
        &prepared.run_dir.join(PREPARATION_RECEIPT_FILE),
        "proof preparation receipt",
    )?;
    let receipt = parse_json_object(&fs::read(&receipt_path)?, "proof preparation receipt")?;
    exact_keys(
        &receipt,
        &[
            "schema",
            "task_id",
            "worktree",
            "commit",
            "scope_base",
            "run_id",
            "native_build",
            "taskfmt",
            "context_index",
            "contexts",
            "results",
            "observer",
        ],
        "proof preparation receipt",
    )?;
    if required_string(&receipt, "schema")? != PREPARATION_RECEIPT_SCHEMA
        || required_string(&receipt, "task_id")? != prepared.task_id
        || required_string(&receipt, "run_id")? != prepared.run_id
    {
        return Err(VerifierError::new(
            "proof preparation receipt identity mismatch",
        ));
    }
    let index_path = immutable_file(
        &prepared.run_dir.join("context-index.json"),
        "context index",
    )?;
    let index_raw = fs::read(&index_path)?;
    let index = parse_json_object(&index_raw, "context index")?;
    let first_member = prepared
        .members
        .first()
        .ok_or_else(|| VerifierError::new("proof preparation has no members"))?;
    let context = parse_json_object(&fs::read(&first_member.context_path)?, "context")?;
    let common = context
        .get("qualification")
        .and_then(Value::as_object)
        .and_then(|qualification| qualification.get("common"))
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("proof preparation common binding is missing"))?;
    if required_string(&receipt, "worktree")? != required_string(common, "worktree")?
        || required_string(&receipt, "commit")? != required_string(&index, "worktree_commit")?
        || required_string(&receipt, "scope_base")? != required_string(&index, "scope_base")?
    {
        return Err(VerifierError::new(
            "proof preparation receipt source binding mismatch",
        ));
    }
    let index_reference = json!({
        "path": path_string(&index_path),
        "sha256": sha256_bytes(&index_raw),
    });
    if receipt.get("context_index") != Some(&index_reference) {
        return Err(VerifierError::new(
            "proof preparation context index reference mismatch",
        ));
    }
    let (_, context_paths) = preparation_artifact_maps(
        receipt.get("contexts"),
        index.get("contexts"),
        &prepared.run_dir.join("contexts"),
        "context",
    )?;
    let (_, result_paths) = preparation_artifact_maps(
        receipt.get("results"),
        index.get("results"),
        &prepared.run_dir.join("results"),
        "preparation result",
    )?;
    if context_paths.len() != prepared.members.len() || result_paths.len() != prepared.members.len()
    {
        return Err(VerifierError::new(
            "proof preparation receipt member count mismatch",
        ));
    }
    let observer = receipt
        .get("observer")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("proof preparation observer is missing"))?;
    exact_keys(observer, &["path", "sha256"], "proof preparation observer")?;
    if index.get("observer") != Some(&Value::Object(observer.clone())) {
        return Err(VerifierError::new(
            "proof preparation observer reference mismatch",
        ));
    }
    let observer_path = bound_artifact_path(
        observer,
        &prepared.run_dir,
        "observer.json",
        "observer capability",
    )?;
    if sha256_bytes(&fs::read(immutable_file(
        &observer_path,
        "observer capability",
    )?)?)
        != required_hex(observer, "sha256", 64)?
    {
        return Err(VerifierError::new(
            "proof preparation observer hash mismatch",
        ));
    }
    let comparator = common
        .get("comparator")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("proof preparation comparator is missing"))?;
    let comparator_path = regular_file(
        Path::new(required_string(comparator, "path")?.as_str()),
        "native comparator",
    )?
    .canonicalize()?;
    let native = receipt
        .get("native_build")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("native build binding is missing"))?;
    let expected_native = native_build_binding(
        Path::new(required_string(native, "receipt")?.as_str()),
        &prepared.run_dir,
        Path::new(required_string(common, "worktree")?.as_str()),
        required_string(&receipt, "commit")?.as_str(),
        &prepared.candidate_tree,
        &comparator_path,
    )?;
    if receipt.get("native_build") != Some(&expected_native) {
        return Err(VerifierError::new(
            "proof preparation native build binding mismatch",
        ));
    }
    let taskfmt = receipt
        .get("taskfmt")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("proof preparation taskfmt is missing"))?;
    validate_taskfmt_receipt(taskfmt, common.get("taskfmt"))?;
    Ok(())
}

fn validate_taskfmt_receipt(
    taskfmt: &Map<String, Value>,
    context_taskfmt: Option<&Value>,
) -> Result<()> {
    exact_keys(
        taskfmt,
        &[
            "taskfmt_revision",
            "taskfmt_version",
            "taskfmt_sha256",
            "taskfmt_source",
            "taskfmt_path",
        ],
        "proof preparation taskfmt",
    )?;
    let revision = required_hex(taskfmt, "taskfmt_revision", 40)?;
    let version = required_string(taskfmt, "taskfmt_version")?;
    if revision != QUALIFIED_TASKFMT_REVISION || version != QUALIFIED_TASKFMT_VERSION {
        return Err(VerifierError::new(
            "taskfmt receipt is not the qualified standalone pin",
        ));
    }
    let source = canonical_input_dir(
        Path::new(required_string(taskfmt, "taskfmt_source")?.as_str()),
        "taskfmt source",
    )?;
    if git_identity(&source)?.0 != revision {
        return Err(VerifierError::new("taskfmt receipt source is stale"));
    }
    require_clean_candidate(&source)?;
    let path = regular_file(
        Path::new(required_string(taskfmt, "taskfmt_path")?.as_str()),
        "taskfmt",
    )?
    .canonicalize()?;
    let actual_hash = hash_file(&path)?;
    if !is_executable(&path)?
        || actual_hash != required_hex(taskfmt, "taskfmt_sha256", 64)?
        || actual_hash != QUALIFIED_TASKFMT_SHA256
    {
        return Err(VerifierError::new("taskfmt receipt executable mismatch"));
    }
    let observed = observed_taskfmt_version(&path)?;
    let expected_observed =
        format!("taskfmt {QUALIFIED_TASKFMT_VERSION} (git {QUALIFIED_TASKFMT_REVISION})");
    if observed != expected_observed {
        return Err(VerifierError::new(format!(
            "taskfmt --version is '{observed}'; expected '{expected_observed}'"
        )));
    }
    let context_taskfmt = context_taskfmt
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context taskfmt binding is missing"))?;
    exact_keys(context_taskfmt, &["path", "sha256"], "context taskfmt")?;
    if Path::new(required_string(context_taskfmt, "path")?.as_str()).canonicalize()? != path
        || required_string(context_taskfmt, "sha256")?
            != required_string(taskfmt, "taskfmt_sha256")?
    {
        return Err(VerifierError::new(
            "taskfmt receipt is not bound to context taskfmt",
        ));
    }
    Ok(())
}

#[expect(
    clippy::type_complexity,
    reason = "paired reference maps and path sets prevent cross-category path reuse"
)]
fn preparation_artifact_maps(
    receipt_value: Option<&Value>,
    index_value: Option<&Value>,
    directory: &Path,
    label: &str,
) -> Result<(BTreeMap<String, Map<String, Value>>, BTreeSet<PathBuf>)> {
    let receipt = preparation_artifact_map(receipt_value, label)?;
    let index = preparation_artifact_map(index_value, &format!("context index {label}"))?;
    if receipt != index {
        return Err(VerifierError::new(format!(
            "proof preparation {label} references differ from context index"
        )));
    }
    let mut paths = BTreeSet::new();
    for (check_id, reference) in &receipt {
        let path = bound_artifact_path(reference, directory, &format!("{check_id}.json"), label)?;
        let path = immutable_file(&path, label)?;
        if !paths.insert(path.clone()) {
            return Err(VerifierError::new(format!(
                "proof preparation {label} contains a duplicate path"
            )));
        }
        if sha256_bytes(&fs::read(&path)?) != required_hex(reference, "sha256", 64)? {
            return Err(VerifierError::new(format!(
                "proof preparation {label} hash mismatch"
            )));
        }
    }
    Ok((receipt, paths))
}

fn preparation_artifact_map(
    value: Option<&Value>,
    label: &str,
) -> Result<BTreeMap<String, Map<String, Value>>> {
    let entries = value
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new(format!("{label} references are missing")))?;
    if entries.is_empty() {
        return Err(VerifierError::new(format!("{label} references are empty")));
    }
    let mut result = BTreeMap::new();
    for entry in entries {
        let entry = entry
            .as_object()
            .ok_or_else(|| VerifierError::new(format!("{label} reference is not an object")))?;
        exact_keys(entry, &["check_id", "path", "sha256"], label)?;
        let check_id = required_string(entry, "check_id")?;
        require_check_id(&check_id)?;
        let _ = absolute_path(Path::new(required_string(entry, "path")?.as_str()), label)?;
        let _ = required_hex(entry, "sha256", 64)?;
        if result.insert(check_id, entry.clone()).is_some() {
            return Err(VerifierError::new(format!("duplicate {label} reference")));
        }
    }
    Ok(result)
}

// The index intentionally has no private runner-only variant.  The campaign
// ledger consumes this exact shape.

/// Validate an existing native run without launching a child.
///
/// The index shape is deliberately the same shape consumed by
/// `scripts/campaign_ledger.py`.  Preparation results are immutable records in
/// `results/`; worker results are separate host-selected files in `outputs/`.
///
/// # Errors
///
/// Returns a verifier error when the run index, bound artifacts, or directory
/// contents fail validation.
pub fn validate_run(run_dir: &Path) -> Result<PreparedRun> {
    validate_run_inner(run_dir, true, None, true)
}

/// Validate a prepared run before every runtime result exists.
///
/// Launch uses this gate. Final `validate_run` requires complete closure.
///
/// # Errors
///
/// Returns a verifier error when the prepared index, bound artifacts, or
/// directory contents fail validation.
pub fn validate_prepared_run(run_dir: &Path) -> Result<PreparedRun> {
    validate_run_inner(run_dir, true, None, false)
}

#[expect(
    clippy::too_many_lines,
    reason = "one fail-closed validation boundary checks the complete run contract"
)]
fn validate_run_inner(
    run_dir: &Path,
    require_preparation_receipt: bool,
    native_target: Option<&Path>,
    require_runtime_closure: bool,
) -> Result<PreparedRun> {
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
            "observer_sequences",
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
    let taskfmt_logs = trust_dir(&run_dir.join("taskfmt-logs"), "taskfmt log directory")?;
    validate_tree_paths(&taskfmt_logs, "taskfmt log directory")?;
    if require_runtime_closure {
        seal_taskfmt_logs(&taskfmt_logs)?;
    }
    let observer_sequences = index
        .get("observer_sequences")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer sequences are missing"))?;
    let preparation_receipt_path = run_dir.join(PREPARATION_RECEIPT_FILE);
    let preparation_receipt_present = regular_path_exists(&preparation_receipt_path)?;
    if require_preparation_receipt && !preparation_receipt_present {
        return Err(VerifierError::new("proof preparation receipt is missing"));
    }

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
    if observer_sequences.keys().collect::<BTreeSet<_>>()
        != context_entries.keys().collect::<BTreeSet<_>>()
    {
        return Err(VerifierError::new("observer sequence check sets differ"));
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
    validate_observer_capability(
        &observer,
        &task_id,
        &run_id,
        &worktree_commit,
        &scope_base,
        observer_sequences,
    )?;
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
        let sequence = observer_sequence_from_context(&context, &operation)?;
        let indexed_sequence = observer_sequences
            .get(check_id)
            .and_then(Value::as_array)
            .ok_or_else(|| VerifierError::new("observer sequence is missing from index"))?;
        if indexed_sequence
            != &sequence
                .iter()
                .cloned()
                .map(Value::String)
                .collect::<Vec<_>>()
        {
            return Err(VerifierError::new(
                "observer sequence is not bound to context/index",
            ));
        }
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
            observer_sequence: sequence,
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
    validate_runtime_outputs(&outputs_dir, &members, &run_id, require_runtime_closure)?;
    let mut expected_root = BTreeSet::from([
        "contexts".to_string(),
        "results".to_string(),
        "outputs".to_string(),
        "taskfmt-logs".to_string(),
        "observer.json".to_string(),
        "context-index.json".to_string(),
    ]);
    if let Some(target) = native_target {
        if target.parent() != Some(run_dir.as_path()) {
            return Err(VerifierError::new(
                "native build target must be a direct run-directory member",
            ));
        }
        require_native_target_dir_name(target)?;
        regular_dir(target, "native build target")?;
        expected_root.insert(NATIVE_TARGET_DIR_NAME.to_string());
    } else if preparation_receipt_present {
        let receipt = parse_json_object(
            &fs::read(immutable_file(
                &preparation_receipt_path,
                "proof preparation receipt",
            )?)?,
            "proof preparation receipt",
        )?;
        let native_build = receipt
            .get("native_build")
            .and_then(Value::as_object)
            .ok_or_else(|| VerifierError::new("native build binding is missing"))?;
        native_target_from_receipt_path(
            Path::new(required_string(native_build, "receipt")?.as_str()),
            &run_dir,
        )?;
        expected_root.insert(NATIVE_TARGET_DIR_NAME.to_string());
    }
    if preparation_receipt_present {
        expected_root.insert(PREPARATION_RECEIPT_FILE.to_string());
    }
    if directory_names(&run_dir)? != expected_root {
        return Err(VerifierError::new(
            "run directory has missing or extra inputs",
        ));
    }
    let members_for_trust = members.clone();
    validate_trust_inputs(&members_for_trust, &index, &run_dir)?;
    let prepared = PreparedRun {
        run_dir,
        run_id,
        task_id,
        candidate_tree: candidate_tree
            .ok_or_else(|| VerifierError::new("candidate tree missing"))?,
        index_sha256: sha256_bytes(&index_raw),
        trust_sha256: trust_sha256.ok_or_else(|| VerifierError::new("trust digest missing"))?,
        members,
    };
    if require_preparation_receipt {
        validate_preparation_receipt(&prepared)?;
    }
    Ok(prepared)
}

/// Launch one bounded child and accept only a complete, bound result artifact.
///
/// # Errors
///
/// Returns a verifier error when launch inputs, observer evidence, child
/// output, or the resulting bound artifact fails validation.
#[expect(
    clippy::too_many_lines,
    reason = "child, observer, cleanup, and result acceptance ordering is one lifecycle"
)]
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
    let prepared = validate_prepared_run(&options.run_dir)?;
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
    let mut observer_supervisor =
        start_observer_supervisor(&prepared, member, &nonce, &oracle_commit)?;
    let (handoff_read, mut handoff_write) = match make_handoff_pipe() {
        Ok(pipe) => pipe,
        Err(error) => {
            return Err(report_cleanup_error(
                error,
                abort_observer_supervisor(observer_supervisor),
            ));
        }
    };
    let handoff = json!({
        "schema": NATIVE_HANDOFF_SCHEMA,
        "run_id": prepared.run_id.clone(),
        "task_id": prepared.task_id.clone(),
        "check_id": member.check_id.clone(),
        "operation": member.operation.clone(),
        "context_sha256": member.context_sha256.clone(),
        "index_sha256": prepared.index_sha256.clone(),
        "token": random_hex(32),
    });
    if let Err(error) = handoff_write.write_all(canonical_json(&handoff).as_bytes()) {
        drop(handoff_read);
        drop(handoff_write);
        return Err(report_cleanup_error(
            error.into(),
            abort_observer_supervisor(observer_supervisor),
        ));
    }
    if let Err(error) = handoff_write.write_all(b"\n") {
        drop(handoff_read);
        drop(handoff_write);
        return Err(report_cleanup_error(
            error.into(),
            abort_observer_supervisor(observer_supervisor),
        ));
    }
    if let Err(error) = handoff_write.flush() {
        drop(handoff_read);
        drop(handoff_write);
        return Err(report_cleanup_error(
            error.into(),
            abort_observer_supervisor(observer_supervisor),
        ));
    }
    let handoff_fd = handoff_read.as_raw_fd();
    let (request_fd, response_fd) = match observer_supervisor.worker_fds() {
        Ok(fds) => fds,
        Err(error) => {
            drop(handoff_read);
            drop(handoff_write);
            return Err(report_cleanup_error(
                error,
                abort_observer_supervisor(observer_supervisor),
            ));
        }
    };
    let mut command = Command::new(&options.program);
    command
        .args(&options.args)
        .current_dir(&worktree)
        .env("PATH", NATIVE_EXECUTION_PATH)
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
        .env("TC_PROOF_NATIVE_HANDOFF_FD", handoff_fd.to_string())
        .env_remove("TC_PROOF_OBSERVER_SOCKET");
    #[cfg(unix)]
    // SAFETY: the pre-exec hook only creates a private process group for the
    // worker, using the async-signal-safe setpgid operation.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            drop(handoff_read);
            drop(handoff_write);
            return Err(report_cleanup_error(
                VerifierError::new(format!("launch failed: {error}")),
                abort_observer_supervisor(observer_supervisor),
            ));
        }
    };
    let worker_group = match ProcessGroupId::from_child(&child) {
        Ok(worker_group) => worker_group,
        Err(error) => {
            let _ = child.kill();
            drop(handoff_read);
            drop(handoff_write);
            return Err(report_cleanup_error(
                error,
                abort_observer_supervisor(observer_supervisor),
            ));
        }
    };
    observer_supervisor.close_worker_ends();
    drop(handoff_read);
    drop(handoff_write);
    let Some(stdout) = child.stdout.take() else {
        return Err(report_cleanup_error(
            VerifierError::new("stdout pipe unavailable"),
            cleanup_failed_launch(
                &mut child,
                worker_group,
                None,
                None,
                Some(observer_supervisor),
            ),
        ));
    };
    let Some(stderr) = child.stderr.take() else {
        return Err(report_cleanup_error(
            VerifierError::new("stderr pipe unavailable"),
            cleanup_failed_launch(
                &mut child,
                worker_group,
                None,
                None,
                Some(observer_supervisor),
            ),
        ));
    };
    let mut stdout_thread = Some(thread::spawn(|| read_bounded(stdout)));
    let mut stderr_thread = Some(thread::spawn(|| read_bounded(stderr)));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() > options.timeout => {
                return Err(report_cleanup_error(
                    VerifierError::new("child timed out"),
                    cleanup_failed_launch(
                        &mut child,
                        worker_group,
                        stdout_thread.take(),
                        stderr_thread.take(),
                        Some(observer_supervisor),
                    ),
                ));
            }
            Ok(None) => thread::sleep(CLEANUP_POLL_INTERVAL),
            Err(error) => {
                return Err(report_cleanup_error(
                    error.into(),
                    cleanup_failed_launch(
                        &mut child,
                        worker_group,
                        stdout_thread.take(),
                        stderr_thread.take(),
                        Some(observer_supervisor),
                    ),
                ));
            }
        }
    };
    if let Err(error) = kill_process_group(worker_group) {
        return Err(report_cleanup_error(
            VerifierError::new(format!("process-group termination failed: {error}")),
            cleanup_failed_launch(
                &mut child,
                worker_group,
                stdout_thread.take(),
                stderr_thread.take(),
                Some(observer_supervisor),
            ),
        ));
    }
    let stdout_result = stdout_thread.take().map(join_output).transpose();
    let stderr_result = stderr_thread.take().map(join_output).transpose();
    let observed_result = observer_supervisor.wait(Duration::from_secs(5));
    let stdout = stdout_result?.ok_or_else(|| VerifierError::new("stdout reader missing"))?;
    let stderr = stderr_result?.ok_or_else(|| VerifierError::new("stderr reader missing"))?;
    let observed_digests = observed_result.map_err(|error| {
        VerifierError::new(format!(
            "{error}; child stderr: {}",
            String::from_utf8_lossy(&stderr)
        ))
    })?;
    let exit = successful_exit(status)?;
    if !regular_path_exists(&member.result_path)? {
        return Err(VerifierError::new("child produced no result"));
    }
    let result_raw = fs::read(&member.result_path)?;
    validate_result(member, &prepared, &result_raw, &observed_digests)?;
    let after = validate_prepared_run(&prepared.run_dir)?;
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
    // SAFETY: each descriptor is transferred exactly once to a File before
    // any fallible flag operation, so an error still closes both descriptors.
    let read = unsafe { File::from_raw_fd(fds[0]) };
    // SAFETY: the second descriptor is transferred exactly once to a File.
    let write = unsafe { File::from_raw_fd(fds[1]) };
    set_close_on_exec(&read, false)?;
    set_close_on_exec(&write, false)?;
    Ok((read, write))
}

#[cfg(unix)]
fn set_close_on_exec(file: &File, enabled: bool) -> Result<()> {
    // SAFETY: `file` owns a valid descriptor for the duration of this call.
    let current = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
    if current == -1 {
        return Err(VerifierError::new(format!(
            "observer descriptor flags read failed: {}",
            io::Error::last_os_error()
        )));
    }
    let flags = if enabled {
        current | libc::FD_CLOEXEC
    } else {
        current & !libc::FD_CLOEXEC
    };
    // SAFETY: `file` owns a valid descriptor and `flags` came from F_GETFD.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, flags) } == -1 {
        return Err(VerifierError::new(format!(
            "observer descriptor inheritance configuration failed: {}",
            io::Error::last_os_error()
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn protect_supervisor_observer_ends(request_read: &File, response_write: &File) -> Result<()> {
    set_close_on_exec(request_read, true)?;
    set_close_on_exec(response_write, true)?;
    Ok(())
}

#[cfg(unix)]
fn make_handoff_pipe() -> Result<(File, File)> {
    let (read, write) = make_pipe()?;
    // The worker receives only the read end. Without this close-on-exec bit,
    // its inherited write end would prevent the one-shot read from reaching
    // EOF and would turn the handoff into a reusable environment convention.
    set_close_on_exec(&write, true)?;
    Ok((read, write))
}

#[cfg(not(unix))]
fn make_handoff_pipe() -> Result<(File, File)> {
    Err(VerifierError::new(
        "native launch handoff requires Unix file descriptors",
    ))
}

#[cfg(not(unix))]
fn make_pipe() -> Result<(File, File)> {
    Err(VerifierError::new(
        "native observer transport requires Unix file descriptors",
    ))
}

#[expect(
    clippy::too_many_lines,
    reason = "observer spawn, pipe wiring, and provider-group capture stay in one supervisor constructor"
)]
fn bound_observer_provider_path(prepared: &PreparedRun) -> Result<PathBuf> {
    let observer = parse_json_object(
        &fs::read(immutable_file(
            &prepared.run_dir.join("observer.json"),
            "observer capability",
        )?)?,
        "observer capability",
    )?;
    let provider = observer
        .get("provider")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer provider identity is missing"))?;
    validate_bound_observer_provider(provider)
}

fn reject_observer_provider_env_rebinding(bound: &Path) -> Result<()> {
    let Some(env_path) = std::env::var_os(OBSERVER_PROVIDER_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
    else {
        return Ok(());
    };
    let env_path = regular_file(&env_path, "observer response provider env")?.canonicalize()?;
    if env_path != *bound {
        return Err(VerifierError::new(
            "observer provider env rebinding is rejected",
        ));
    }
    Ok(())
}

fn start_observer_supervisor(
    prepared: &PreparedRun,
    member: &PreparedMember,
    nonce: &str,
    oracle_commit: &str,
) -> Result<ObserverSupervisor> {
    let provider_path = bound_observer_provider_path(prepared)?;
    reject_observer_provider_env_rebinding(&provider_path)?;
    let before_hash = hash_file(&provider_path)?;
    let (request_read, request_write) = make_pipe()?;
    let (response_read, response_write) = make_pipe()?;
    // Only request_write and response_read belong in the worker. The
    // supervisor-owned counterparts must not survive worker exec, or a
    // worker descendant can keep the observer pipes open after the worker
    // itself exits.
    #[cfg(unix)]
    protect_supervisor_observer_ends(&request_read, &response_write)?;
    // The provider is a separate host process. Spawn it before the worker
    // handoff exists, and close every observer-pipe endpoint in its child.
    // Otherwise it can retain the worker-facing writer and make the
    // supervisor's completion read wait forever after a successful worker.
    #[cfg(unix)]
    let observer_fds = [
        request_read.as_raw_fd(),
        request_write.as_raw_fd(),
        response_read.as_raw_fd(),
        response_write.as_raw_fd(),
    ];
    let mut child = {
        let mut command = Command::new(&provider_path);
        command
            .current_dir(&prepared.run_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .env("PATH", NATIVE_EXECUTION_PATH)
            .env_remove(OBSERVER_PROVIDER_ENV);
        #[cfg(unix)]
        // SAFETY: the pre-exec hook only closes inherited file descriptors;
        // close is async-signal-safe and the descriptors are owned by this
        // launcher process.
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                for fd in observer_fds {
                    if fd > 2 && libc::close(fd) == -1 {
                        return Err(io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        command.spawn().map_err(|error| {
            VerifierError::new(format!("observer response provider launch failed: {error}"))
        })?
    };
    let provider_group = match ProcessGroupId::from_child(&child) {
        Ok(provider_group) => provider_group,
        Err(error) => {
            let _ = child.kill();
            return Err(error);
        }
    };
    let Some(stdin) = child.stdin.take() else {
        return Err(report_cleanup_error(
            VerifierError::new("observer response provider stdin unavailable"),
            terminate_child(&mut child, provider_group, OBSERVER_CLEANUP_TIMEOUT).map(|_| ()),
        ));
    };
    let Some(stdout) = child.stdout.take() else {
        return Err(report_cleanup_error(
            VerifierError::new("observer response provider stdout unavailable"),
            terminate_child(&mut child, provider_group, OBSERVER_CLEANUP_TIMEOUT).map(|_| ()),
        ));
    };
    let mut provider = PipeObserverProvider {
        request: BufWriter::new(stdin),
        response: BufReader::new(stdout),
    };
    let provider_process = Arc::new(Mutex::new(child));
    let run_id = prepared.run_id.clone();
    let task_id = prepared.task_id.clone();
    let check_id = member.check_id.clone();
    let sequence = member.observer_sequence.clone();
    let tree = prepared.candidate_tree.clone();
    let nonce = nonce.to_owned();
    let oracle_commit = oracle_commit.to_owned();
    let provider_process_for_supervisor = Arc::clone(&provider_process);
    let supervisor = thread::spawn(move || {
        let binding = ObserverBinding {
            run_id: &run_id,
            task_id: &task_id,
            check_id: &check_id,
            tree: &tree,
            oracle_commit: &oracle_commit,
            nonce: &nonce,
            sequence: &sequence,
        };
        let result = observe_worker(request_read, response_write, &mut provider, &binding);
        let observer_error = result.as_ref().err().map(ToString::to_string);
        drop(provider);
        let status = {
            let deadline = cleanup_deadline(
                OBSERVER_CLEANUP_TIMEOUT,
                "observer supervisor provider cleanup",
            )?;
            let mut child = lock_provider_until(&provider_process_for_supervisor, deadline)?;
            finish_observer_provider(&mut child, provider_group, result.is_err())?
        };
        let after_hash = hash_file(&provider_path);
        if after_hash.as_ref().ok() != Some(&before_hash) {
            return Err(VerifierError::new(format!(
                "observer response provider hash changed during execution: before={before_hash}; after={after_hash:?}"
            )));
        }
        if !status.success() {
            return Err(VerifierError::new(format!(
                "observer response provider exited unsuccessfully (status={status:?}; observer={})",
                observer_error.as_deref().unwrap_or("none")
            )));
        }
        result
    });
    Ok(ObserverSupervisor {
        request_write: Some(request_write),
        response_read: Some(response_read),
        provider: provider_process,
        provider_group,
        thread: Some(supervisor),
    })
}

fn abort_observer_supervisor(supervisor: ObserverSupervisor) -> Result<()> {
    supervisor.abort()
}

fn cleanup_deadline(timeout: Duration, label: &str) -> Result<Instant> {
    Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| VerifierError::new(format!("{label} deadline overflowed")))
}

fn lock_provider_until(
    provider: &Mutex<Child>,
    deadline: Instant,
) -> Result<MutexGuard<'_, Child>> {
    loop {
        match provider.try_lock() {
            Ok(provider) => return Ok(provider),
            Err(std::sync::TryLockError::Poisoned(provider)) => return Ok(provider.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => {}
        }
        if Instant::now() >= deadline {
            return Err(VerifierError::new(
                "observer response provider lock did not become available before cleanup deadline",
            ));
        }
        thread::sleep(CLEANUP_POLL_INTERVAL);
    }
}

fn join_thread_bounded<T>(
    thread: thread::JoinHandle<T>,
    deadline: Instant,
    label: &str,
) -> Result<T> {
    loop {
        if thread.is_finished() {
            return thread
                .join()
                .map_err(|_| VerifierError::new(format!("{label} thread panicked")));
        }
        if Instant::now() >= deadline {
            return Err(VerifierError::new(format!(
                "{label} thread did not finish before cleanup deadline"
            )));
        }
        thread::sleep(CLEANUP_POLL_INTERVAL);
    }
}

fn join_result_thread<T>(
    thread: thread::JoinHandle<Result<T>>,
    deadline: Instant,
    label: &str,
) -> Result<T> {
    join_thread_bounded(thread, deadline, label)?
}

fn combine_cleanup_errors(errors: Vec<VerifierError>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    let details = errors
        .into_iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    Err(VerifierError::new(format!("cleanup failed: {details}")))
}

fn report_cleanup_error(error: VerifierError, cleanup: Result<()>) -> VerifierError {
    match cleanup {
        Ok(()) => error,
        Err(cleanup_error) => {
            VerifierError::new(format!("{error}; cleanup failed: {cleanup_error}"))
        }
    }
}

fn kill_process_group(process_group: ProcessGroupId) -> io::Result<()> {
    #[cfg(unix)]
    {
        let process_group = libc::pid_t::try_from(process_group.0).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "child pid does not fit the native process-group type",
            )
        })?;
        let Some(negative) = process_group.checked_neg() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "process-group id cannot be negated",
            ));
        };
        if negative == 0 || negative == -1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "refusing to signal process-group 0 or every process",
            ));
        }
        // SAFETY: worker and provider launch hooks create private groups with
        // the captured child pid as the group id before either process can run.
        if unsafe { libc::kill(negative, libc::SIGKILL) } == -1 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
    }
    #[cfg(not(unix))]
    let _ = process_group;
    Ok(())
}

fn child_gone_error(error: &io::Error) -> bool {
    let gone = matches!(
        error.kind(),
        io::ErrorKind::InvalidInput | io::ErrorKind::NotFound
    );
    #[cfg(unix)]
    {
        gone || error.raw_os_error() == Some(libc::ESRCH)
    }
    #[cfg(not(unix))]
    {
        gone
    }
}

fn wait_for_child_until(child: &mut Child, deadline: Instant, label: &str) -> Result<ExitStatus> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(VerifierError::new(format!(
                "{label} did not terminate before cleanup deadline"
            )));
        }
        thread::sleep(CLEANUP_POLL_INTERVAL);
    }
}

fn terminate_child_until(
    child: &mut Child,
    process_group: ProcessGroupId,
    deadline: Instant,
) -> Result<ExitStatus> {
    let reaped = child.try_wait()?;
    let mut group_error = kill_process_group(process_group).err();
    let direct_error = if reaped.is_some() {
        None
    } else {
        child.kill().err()
    };
    let status = match reaped {
        Some(status) => status,
        None => wait_for_child_until(child, deadline, "child")?,
    };
    if let Err(error) = kill_process_group(process_group) {
        group_error = Some(error);
    }
    if let Some(error) = group_error {
        return Err(VerifierError::new(format!(
            "process-group termination failed: {error}"
        )));
    }
    if let Some(error) = direct_error.filter(|error| !child_gone_error(error)) {
        return Err(VerifierError::new(format!(
            "child termination failed: {error}"
        )));
    }
    Ok(status)
}

fn terminate_child(
    child: &mut Child,
    process_group: ProcessGroupId,
    timeout: Duration,
) -> Result<ExitStatus> {
    let deadline = cleanup_deadline(timeout, "child termination")?;
    terminate_child_until(child, process_group, deadline)
}

fn cleanup_failed_launch(
    child: &mut Child,
    process_group: ProcessGroupId,
    stdout_thread: Option<thread::JoinHandle<Result<Vec<u8>>>>,
    stderr_thread: Option<thread::JoinHandle<Result<Vec<u8>>>>,
    observer_supervisor: Option<ObserverSupervisor>,
) -> Result<()> {
    let deadline = cleanup_deadline(OBSERVER_CLEANUP_TIMEOUT, "launch cleanup")?;
    let mut errors = Vec::new();
    if let Err(error) = terminate_child_until(child, process_group, deadline) {
        errors.push(error);
    }
    if let Some(thread) = stdout_thread
        && let Err(error) = join_result_thread(thread, deadline, "child stdout reader")
    {
        errors.push(error);
    }
    if let Some(thread) = stderr_thread
        && let Err(error) = join_result_thread(thread, deadline, "child stderr reader")
    {
        errors.push(error);
    }
    if let Some(supervisor) = observer_supervisor
        && let Err(error) = supervisor.abort_until(deadline)
    {
        errors.push(error);
    }
    combine_cleanup_errors(errors)
}

fn finish_observer_provider(
    child: &mut Child,
    process_group: ProcessGroupId,
    kill: bool,
) -> Result<ExitStatus> {
    if kill {
        return terminate_child(child, process_group, OBSERVER_CLEANUP_TIMEOUT);
    }
    let graceful_deadline = cleanup_deadline(
        OBSERVER_PROVIDER_GRACE_TIMEOUT,
        "observer response provider",
    )?;
    match wait_for_child_until(child, graceful_deadline, "observer response provider") {
        Ok(status) => {
            let deadline = cleanup_deadline(
                OBSERVER_CLEANUP_TIMEOUT,
                "observer response provider group reap",
            )?;
            terminate_child_until(child, process_group, deadline)?;
            Ok(status)
        }
        Err(wait_error) => match terminate_child(child, process_group, OBSERVER_CLEANUP_TIMEOUT) {
            Ok(_) => Err(VerifierError::new(format!(
                "observer response provider did not terminate gracefully: {wait_error}"
            ))),
            Err(cleanup_error) => Err(VerifierError::new(format!(
                "observer response provider cleanup failed: {cleanup_error}; original wait: {wait_error}"
            ))),
        },
    }
}

trait ObserverProvider {
    fn observe(&mut self, request: &Map<String, Value>) -> Result<Map<String, Value>>;
}

struct PipeObserverProvider {
    request: BufWriter<ChildStdin>,
    response: BufReader<ChildStdout>,
}

impl ObserverProvider for PipeObserverProvider {
    fn observe(&mut self, request: &Map<String, Value>) -> Result<Map<String, Value>> {
        self.request
            .write_all(canonical_json(&Value::Object(request.clone())).as_bytes())?;
        self.request.write_all(b"\n")?;
        self.request.flush()?;
        let mut line = String::new();
        let count = self.response.read_line(&mut line)?;
        if count == 0 || !line.ends_with('\n') || line.len() > MAX_CHILD_OUTPUT_BYTES {
            return Err(VerifierError::new(
                "independent observer response truncated or missing",
            ));
        }
        parse_json_object(line.trim_end_matches('\n').as_bytes(), "observer response")
    }
}

fn observe_worker<P: ObserverProvider>(
    request_read: File,
    response_write: File,
    provider: &mut P,
    binding: &ObserverBinding<'_>,
) -> Result<Vec<String>> {
    let expected = binding.sequence.len();
    let mut reader = BufReader::new(request_read);
    let mut writer = BufWriter::new(response_write);
    let mut observed_digests = Vec::with_capacity(expected);
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
            || required_string(&request, "nonce")? != binding.nonce
            || required_string(&request, "run_id")? != binding.run_id
            || required_string(&request, "task_id")? != binding.task_id
            || required_string(&request, "check_id")? != binding.check_id
            || request.get("request_id").and_then(Value::as_u64) != Some(request_id as u64)
            || required_string(&request, "operation")? != binding.sequence[request_id]
            || required_string(&request, "source_commit")? != binding.oracle_commit
            || required_string(&request, "tree")? != binding.tree
        {
            return Err(VerifierError::new(
                "observer request binding mismatch or replay",
            ));
        }
        let response = provider.observe(&request)?;
        validate_observer_response(&response, request_id as u64, binding)?;
        let response = Value::Object(response);
        observed_digests.push(sha256_canonical(&response));
        writer.write_all(canonical_json(&response).as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    let mut extra = [0_u8; 1];
    if reader.read(&mut extra)? != 0 {
        return Err(VerifierError::new("observer replay or incomplete close"));
    }
    Ok(observed_digests)
}

fn validate_observer_response(
    response: &Map<String, Value>,
    request_id: u64,
    binding: &ObserverBinding<'_>,
) -> Result<()> {
    exact_keys(
        response,
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
            "exit",
            "stdout",
            "stderr",
            "files",
            "payload",
            "records",
        ],
        "observer response",
    )?;
    if response.get("schema") != Some(&Value::String("tc-proof-observation/v1".to_string()))
        || required_string(response, "nonce")? != binding.nonce
        || required_string(response, "run_id")? != binding.run_id
        || required_string(response, "task_id")? != binding.task_id
        || required_string(response, "check_id")? != binding.check_id
        || response.get("request_id").and_then(Value::as_u64) != Some(request_id)
        || required_string(response, "operation")? != binding.sequence[request_id as usize]
        || required_string(response, "source_commit")? != binding.oracle_commit
        || required_string(response, "tree")? != binding.tree
        || response.get("exit").and_then(Value::as_i64) != Some(0)
        || response.get("stdout").and_then(Value::as_str).is_none()
        || response.get("stderr").and_then(Value::as_str).is_none()
    {
        return Err(VerifierError::new(
            "observer response binding, status, or stream mismatch",
        ));
    }
    let files = response
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer response files are invalid"))?;
    if files
        .iter()
        .any(|(name, value)| name.is_empty() || value.as_str().is_none())
    {
        return Err(VerifierError::new("observer response files are invalid"));
    }
    let payload = response
        .get("payload")
        .and_then(Value::as_object)
        .filter(|payload| !payload.is_empty())
        .ok_or_else(|| VerifierError::new("observer response payload is empty or invalid"))?;
    let records = response
        .get("records")
        .and_then(Value::as_array)
        .filter(|records| !records.is_empty())
        .ok_or_else(|| VerifierError::new("observer response records are empty or invalid"))?;
    if records
        .iter()
        .any(|record| record.as_object().is_none_or(Map::is_empty))
    {
        return Err(VerifierError::new(
            "observer response contains an invalid record",
        ));
    }
    let _ = payload;
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
        let operation = discover_operation(&source);
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

fn discover_operation(command: &str) -> String {
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
        return "external".to_string();
    };
    tokens
        .get(position.saturating_add(1))
        .filter(|operation| !operation.starts_with('-'))
        .map_or_else(
            || "external".to_string(),
            |operation| (*operation).to_string(),
        )
}

fn discover_context_reference(command: &str, check_id: &str) -> Result<Option<String>> {
    let marker = "$RUN_DIR/contexts/";
    let mut found = None;
    let mut remaining = command;
    while let Some(position) = remaining.find(marker) {
        let suffix = &remaining[position.saturating_add(marker.len())..];
        let end = suffix
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || character == '-' || character == '.')
            })
            .unwrap_or(suffix.len());
        let reference = &suffix[..end];
        if !reference.starts_with("CHK-")
            || reference.strip_suffix(".json").is_none()
            || reference.len() != 12
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
        .and_then(|position| tokens.get(position.saturating_add(1)))
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
            || !(name.strip_suffix(".json").is_some()
                || name.strip_suffix(".template.json").is_some())
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

fn observer_sequence_for_check(
    check: &CheckSpec,
    template: Option<&TemplateInfo>,
) -> Result<Vec<String>> {
    if check.operation.is_empty() {
        return Err(VerifierError::new(
            "observer sequence cannot be derived for an external operation",
        ));
    }
    if check.operation != "oracle" {
        return Ok(vec![check.operation.clone()]);
    }
    match observer_family_for_check(check, template)? {
        "native" => Ok(vec![check.operation.clone()]),
        "synthetic" => Ok(vec![check.operation.clone(), check.operation.clone()]),
        _ => Err(VerifierError::new("oracle observer family is invalid")),
    }
}

fn template_qualification_family(template: Option<&TemplateInfo>) -> Option<&str> {
    template
        .and_then(|template| template.value.as_object())
        .and_then(|value| value.get("qualification"))
        .and_then(Value::as_object)
        .and_then(|qualification| qualification.get("family"))
        .and_then(Value::as_str)
}

fn is_synthetic_oracle_namespace(namespace: &str) -> bool {
    namespace.is_empty() || namespace == "synthetic"
}

fn observer_family_for_check(
    check: &CheckSpec,
    template: Option<&TemplateInfo>,
) -> Result<&'static str> {
    match template_qualification_family(template) {
        Some("native") => Ok("native"),
        Some("synthetic") => Ok("synthetic"),
        None => {
            if is_synthetic_oracle_namespace(&check.namespace) {
                Ok("synthetic")
            } else {
                Ok("native")
            }
        }
        Some(_) => Err(VerifierError::new(
            "oracle template must declare native or omit its qualification family",
        )),
    }
}

fn apply_template_overlays(object: &mut Map<String, Value>, template: Option<&TemplateInfo>) {
    let Some(template_object) = template.and_then(|template| template.value.as_object()) else {
        return;
    };
    for key in [
        "adapter",
        "inventory",
        "evidence",
        "configuration",
        "architecture_profile",
        "branch_host_projection",
    ] {
        if let Some(value) = template_object.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
}

fn bind_qualification_family(
    qualification_object: &mut Map<String, Value>,
    check: &CheckSpec,
    template: Option<&TemplateInfo>,
) -> Result<()> {
    if check.operation == "oracle" {
        qualification_object.insert(
            "family".to_string(),
            Value::String(observer_family_for_check(check, template)?.to_string()),
        );
        return Ok(());
    }
    if let Some(family) = template_qualification_family(template) {
        qualification_object.insert("family".to_string(), Value::String(family.to_string()));
    }
    Ok(())
}

fn bind_compare_qualification(
    qualification_object: &mut Map<String, Value>,
    context: &Map<String, Value>,
    check: &CheckSpec,
    common: &Value,
    template_value: Option<Value>,
) -> Result<()> {
    let report_path = run_output_path(common, &check.id, "compare.json")?;
    let report = path_string(&report_path);
    let mut nested = template_value.unwrap_or_else(|| {
        json!({
            "schema": COMPARE_CONTEXT_SCHEMA,
        })
    });
    let nested_object = nested
        .as_object_mut()
        .ok_or_else(|| VerifierError::new("compare template is not an object"))?;
    nested_object.insert(
        "schema".to_string(),
        Value::String(COMPARE_CONTEXT_SCHEMA.to_string()),
    );
    nested_object.insert(
        "run_id".to_string(),
        context
            .get("run_id")
            .cloned()
            .ok_or_else(|| VerifierError::new("context run_id is missing"))?,
    );
    nested_object.insert(
        "task_id".to_string(),
        context
            .get("task_id")
            .cloned()
            .ok_or_else(|| VerifierError::new("context task_id is missing"))?,
    );
    nested_object.insert("check_id".to_string(), Value::String(check.id.clone()));
    nested_object.insert(
        "oracle_commit".to_string(),
        context
            .get("oracle_commit")
            .cloned()
            .ok_or_else(|| VerifierError::new("context oracle_commit is missing"))?,
    );
    nested_object.insert(
        "candidate_source_tree".to_string(),
        context
            .get("tree")
            .cloned()
            .ok_or_else(|| VerifierError::new("context tree is missing"))?,
    );
    nested_object.insert("report_path".to_string(), Value::String(report.clone()));
    qualification_object.insert(
        "comparator".to_string(),
        json!({
            "schema": COMPARE_CONTEXT_SCHEMA,
            "context": nested,
            "report_path": report,
        }),
    );
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "context construction mirrors the exact schema and preserves field-binding order"
)]
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
    let observer_sequence = observer_sequence_for_check(check, template)?;
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
        "observer_sequence".to_string(),
        Value::Array(
            observer_sequence
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
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
    apply_template_overlays(object, template);
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
    bind_qualification_family(qualification_object, check, template)?;
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
    if check.operation == "compare" {
        bind_compare_qualification(qualification_object, object, check, common, template_value)?;
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
    observer_sequences: &Map<String, Value>,
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
            "sequences",
            "provider",
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
        || observer.get("sequences") != Some(&Value::Object(observer_sequences.clone()))
    {
        return Err(VerifierError::new("observer capability identity mismatch"));
    }
    validate_bound_observer_provider(
        observer
            .get("provider")
            .and_then(Value::as_object)
            .ok_or_else(|| VerifierError::new("observer provider identity is missing"))?,
    )?;
    Ok(())
}

fn validate_bound_observer_provider(provider: &Map<String, Value>) -> Result<PathBuf> {
    exact_keys(provider, &["path", "sha256"], "observer provider")?;
    let path = regular_file(
        Path::new(required_string(provider, "path")?.as_str()),
        "observer response provider",
    )?
    .canonicalize()?;
    if !is_executable(&path)? {
        return Err(VerifierError::new(
            "observer response provider is not executable",
        ));
    }
    let actual = hash_file(&path)?;
    if actual != required_hex(provider, "sha256", 64)? {
        return Err(VerifierError::new(
            "observer response provider hash mismatch",
        ));
    }
    Ok(path)
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
    trust_dir(root, label)?;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            VerifierError::new(format!("{label} member is unreadable: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(VerifierError::new(format!(
                "{label} contains an unsafe link"
            )));
        }
        if metadata.is_dir() {
            validate_tree_paths(&path, label)?;
        } else {
            trust_file(&path, label)?;
        }
    }
    Ok(())
}

fn seal_taskfmt_logs(root: &Path) -> Result<()> {
    let mut hashed = Vec::new();
    let mut saw_done = false;
    seal_taskfmt_log_tree(root, "taskfmt log", &mut hashed, &mut saw_done)?;
    if hashed.is_empty() {
        return Err(VerifierError::new("taskfmt logs are missing"));
    }
    if !saw_done {
        return Err(VerifierError::new("taskfmt logs do not end in DONE"));
    }
    Ok(())
}

fn seal_taskfmt_log_tree(
    root: &Path,
    label: &str,
    hashed: &mut Vec<(PathBuf, String)>,
    saw_done: &mut bool,
) -> Result<()> {
    trust_dir(root, label)?;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            VerifierError::new(format!("{label} member is unreadable: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(VerifierError::new(format!(
                "{label} contains an unsafe link"
            )));
        }
        if metadata.is_dir() {
            seal_taskfmt_log_tree(&path, label, hashed, saw_done)?;
            continue;
        }
        let path = seal_trust_file(&path, label)?;
        let raw = fs::read(&path)?;
        if last_nonempty_line(&raw) == Some("DONE") {
            *saw_done = true;
        }
        hashed.push((path, sha256_bytes(&raw)));
    }
    Ok(())
}

fn last_nonempty_line(raw: &[u8]) -> Option<&str> {
    std::str::from_utf8(raw).ok().and_then(|text| {
        text.split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .next_back()
    })
}

fn validate_runtime_outputs(
    outputs_dir: &Path,
    members: &[PreparedMember],
    run_id: &str,
    require_runtime_closure: bool,
) -> Result<()> {
    trust_dir(outputs_dir, "runtime output directory")?;
    let mut expected = BTreeSet::new();
    let mut close_member = None;
    for member in members {
        expected.insert(
            member
                .result_path
                .file_name()
                .ok_or_else(|| VerifierError::new("invalid runtime result path"))?
                .to_string_lossy()
                .into_owned(),
        );
        if member.operation == "close" {
            close_member = Some(member);
        }
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
    if require_runtime_closure {
        if let Some(close) = close_member
            && !regular_path_exists(&close.result_path)?
        {
            return Err(VerifierError::new("close result is missing"));
        }
        if actual != expected {
            return Err(VerifierError::new(
                "runtime outputs are missing, extra, or not indexed exactly once",
            ));
        }
    } else if !actual.is_subset(&expected) {
        return Err(VerifierError::new("runtime outputs contain extra files"));
    }
    for member in members {
        let result_exists = regular_path_exists(&member.result_path)?;
        if require_runtime_closure && !result_exists {
            return Err(VerifierError::new(format!(
                "missing runtime result for {}",
                member.check_id
            )));
        }
        if result_exists {
            validate_result_if_present(member, run_id, require_runtime_closure)?;
            seal_trust_file(&member.result_path, "runtime result")?;
        }
        if let Some(report) = &member.comparator_report_path {
            let report_exists = regular_path_exists(report)?;
            if require_runtime_closure && !report_exists {
                return Err(VerifierError::new(format!(
                    "missing comparator report for {}",
                    member.check_id
                )));
            }
            if report_exists {
                let report_raw = fs::read(trust_file(report, "comparator report")?)?;
                let _ = parse_json_object(&report_raw, "comparator report")?;
                seal_trust_file(report, "comparator report")?;
            }
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "context validation keeps the complete allowlist and identity contract visible"
)]
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
        "observer_sequence",
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
    exact_keys(tool, &["path", "sha256"], "context tool")?;
    let tool_path = PathBuf::from(required_string(tool, "path")?);
    let actual_tool_hash = hash_file(&regular_file(&tool_path, "context tool")?)?;
    if Some(actual_tool_hash.as_str()) != tool.get("sha256").and_then(Value::as_str) {
        return Err(VerifierError::new("context tool digest mismatch"));
    }
    let operation = required_string(context, "operation")?;
    let _ = observer_sequence_from_context(context, &operation)?;
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
    if required_hex(context, "tree", 40)? != required_string(common, "candidate_tree")? {
        return Err(VerifierError::new(
            "context source tree is not bound to trusted candidate tree",
        ));
    }
    let oracle = common
        .get("oracle")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("context common oracle binding is missing"))?;
    if required_hex(context, "oracle_commit", 40)? != required_string(oracle, "commit")? {
        return Err(VerifierError::new(
            "context oracle commit is not bound to trusted oracle",
        ));
    }
    if required_hex(context, "oracle_tree", 40)? != required_string(oracle, "tree")? {
        return Err(VerifierError::new(
            "context oracle tree is not bound to trusted oracle",
        ));
    }
    if context.get("tool") != common.get("tool") {
        return Err(VerifierError::new(
            "context tool identity is not bound to trusted tool",
        ));
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

fn observer_sequence_from_context(
    context: &Map<String, Value>,
    operation: &str,
) -> Result<Vec<String>> {
    let sequence = context
        .get("observer_sequence")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifierError::new("context observer sequence is missing"))?;
    if sequence.is_empty() || sequence.len() > 8 {
        return Err(VerifierError::new(
            "context observer sequence length is invalid",
        ));
    }
    let sequence = sequence
        .iter()
        .map(|value| {
            let operation = value
                .as_str()
                .filter(|operation| !operation.is_empty())
                .ok_or_else(|| VerifierError::new("context observer sequence is invalid"))?;
            Ok(operation.to_owned())
        })
        .collect::<Result<Vec<_>>>()?;
    if sequence.first().map(String::as_str) != Some(operation) {
        return Err(VerifierError::new(
            "context observer sequence starts with the wrong operation",
        ));
    }
    if operation == "oracle" {
        if sequence.iter().any(|item| item != "oracle") || sequence.len() > 2 {
            return Err(VerifierError::new("oracle observer sequence is invalid"));
        }
        let family = context
            .get("qualification")
            .and_then(Value::as_object)
            .and_then(|qualification| qualification.get("family"))
            .and_then(Value::as_str);
        let expected_len = match family {
            Some("native") => 1,
            Some("synthetic") => 2,
            _ => {
                return Err(VerifierError::new(
                    "oracle observer family is missing or invalid",
                ));
            }
        };
        if sequence.len() != expected_len {
            return Err(VerifierError::new(
                "oracle observer sequence does not match qualification family",
            ));
        }
    } else if sequence != [operation.to_owned()] {
        return Err(VerifierError::new(
            "non-oracle observer sequence must contain exactly its operation",
        ));
    }
    Ok(sequence)
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
    validate_candidate_tool_binding(common, &worktree)?;
    validate_identity_hash(common, "tool", "tool")?;
    validate_identity_hash(common, "comparator", "comparator")?;
    if !common.get("taskfmt").is_some_and(Value::is_null) {
        validate_identity_hash(common, "taskfmt", "taskfmt")?;
    }
    let observer = common
        .get("observer")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer binding missing"))?;
    let provider = observer
        .get("provider")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer provider identity is missing"))?;
    let bound_provider = validate_bound_observer_provider(provider)?;
    let capability_provider = parse_json_object(
        &fs::read(immutable_file(
            &run_dir.join("observer.json"),
            "observer capability",
        )?)?,
        "observer capability",
    )?;
    let capability_provider = capability_provider
        .get("provider")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("observer capability provider is missing"))?;
    if validate_bound_observer_provider(capability_provider)? != bound_provider {
        return Err(VerifierError::new(
            "observer provider is not bound to the capability",
        ));
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

fn validate_candidate_tool_binding(common: &Map<String, Value>, worktree: &Path) -> Result<()> {
    let expected = regular_file(
        &worktree.join("tools/refactor-proof/bin/tc-proof"),
        "candidate proof tool",
    )?
    .canonicalize()?;
    let tool = common
        .get("tool")
        .and_then(Value::as_object)
        .ok_or_else(|| VerifierError::new("tool binding missing"))?;
    exact_keys(tool, &["path", "sha256"], "tool binding")?;
    let actual =
        regular_file(Path::new(required_string(tool, "path")?.as_str()), "tool")?.canonicalize()?;
    if actual != expected {
        return Err(VerifierError::new(
            "trusted proof tool is not the candidate worktree worker",
        ));
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
    validate_result_fields(member, run_id, &fs::read(path)?, require_passed, None)?;
    set_readonly_file(&member.result_path)
}

fn validate_result(
    member: &PreparedMember,
    prepared: &PreparedRun,
    raw: &[u8],
    observed_digests: &[String],
) -> Result<()> {
    validate_result_fields(member, &prepared.run_id, raw, true, Some(observed_digests))?;
    set_readonly_file(&member.result_path)
}

fn validate_result_fields(
    member: &PreparedMember,
    run_id: &str,
    raw: &[u8],
    require_passed: bool,
    observed_digests: Option<&[String]>,
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
    if status == Some("passed") && digests.is_empty() {
        return Err(VerifierError::new("passed result has no observed events"));
    }
    if let Some(observed_digests) = observed_digests
        && digests
            != &observed_digests
                .iter()
                .map(|digest| Value::String(digest.clone()))
                .collect::<Vec<_>>()
    {
        return Err(VerifierError::new(
            "result observations are not bound to native observer events",
        ));
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

fn allowed_system_symlink_target(path: &Path) -> Option<&'static Path> {
    match path {
        path if path == Path::new("/etc") => Some(Path::new("/private/etc")),
        path if path == Path::new("/home") => Some(Path::new("/System/Volumes/Data/home")),
        path if path == Path::new("/tmp") => Some(Path::new("/private/tmp")),
        path if path == Path::new("/var") => Some(Path::new("/private/var")),
        _ => None,
    }
}

fn validate_path_components(path: &Path, label: &str, allow_missing_final: bool) -> Result<()> {
    absolute_path(path, label)?;
    let mut current = PathBuf::new();
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        current.push(component.as_os_str());
        let is_final = components.peek().is_none();
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    let allowed = allowed_system_symlink_target(&current).is_some_and(|target| {
                        current
                            .canonicalize()
                            .is_ok_and(|resolved| resolved == target)
                    });
                    if !allowed {
                        return Err(VerifierError::new(format!(
                            "{label} path component must not be a symlink: {}",
                            current.display()
                        )));
                    }
                } else if !is_final && !metadata.is_dir() {
                    return Err(VerifierError::new(format!(
                        "{label} path component is not a directory: {}",
                        current.display()
                    )));
                }
            }
            Err(error)
                if is_final && allow_missing_final && error.kind() == io::ErrorKind::NotFound =>
            {
                return Ok(());
            }
            Err(error) => {
                return Err(VerifierError::new(format!(
                    "{label} path component is unreadable: {}: {error}",
                    current.display()
                )));
            }
        }
    }
    Ok(())
}

fn canonical_input_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let path = absolute_path(path, label)?;
    validate_path_components(&path, label, false)?;
    if path.is_symlink() {
        return Err(VerifierError::new(format!(
            "{label} must be a real directory"
        )));
    }
    path.canonicalize()
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))
}

fn canonical_existing_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let path = absolute_path(path, label)?;
    validate_path_components(&path, label, false)?;
    if path.is_symlink() || !path.is_dir() {
        return Err(VerifierError::new(format!(
            "{label} must be a real directory"
        )));
    }
    path.canonicalize()
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))
}

fn trust_metadata(path: &Path, label: &str) -> Result<fs::Metadata> {
    validate_path_components(path, label, false)?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| VerifierError::new(format!("{label}: {error}")))?;
    if metadata.file_type().is_symlink() {
        return Err(VerifierError::new(format!(
            "{label} path component must not be a symlink: {}",
            path.display()
        )));
    }
    Ok(metadata)
}

fn trust_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = trust_metadata(path, label)?;
    if !metadata.is_dir() {
        return Err(VerifierError::new(format!("{label} is absent or unsafe")));
    }
    Ok(path.to_path_buf())
}

fn trust_file(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = trust_metadata(path, label)?;
    if !metadata.file_type().is_file() || link_count(&metadata) != 1 {
        return Err(VerifierError::new(format!(
            "{label} is not an immutable regular file"
        )));
    }
    Ok(path.to_path_buf())
}

fn seal_trust_file(path: &Path, label: &str) -> Result<PathBuf> {
    let path = trust_file(path, label)?;
    set_readonly_file(&path)?;
    immutable_file(&path, label)
}

fn regular_dir(path: &Path, label: &str) -> Result<PathBuf> {
    trust_dir(path, label)
}

fn regular_file(path: &Path, label: &str) -> Result<PathBuf> {
    trust_file(path, label)
}

fn immutable_file(path: &Path, label: &str) -> Result<PathBuf> {
    let path = trust_file(path, label)?;
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
    validate_path_components(path, "path", true)?;
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

fn native_target_from_receipt_path(receipt_path: &Path, run_dir: &Path) -> Result<PathBuf> {
    let receipt_path = regular_file(receipt_path, "native build receipt")?.canonicalize()?;
    let debug_dir = receipt_path
        .parent()
        .ok_or_else(|| VerifierError::new("native build receipt has no parent"))?;
    let target = canonical_existing_dir(
        debug_dir
            .parent()
            .ok_or_else(|| VerifierError::new("native build receipt has no target parent"))?,
        "native build target",
    )?;
    if target.parent() != Some(run_dir) {
        return Err(VerifierError::new(
            "native build target must be a direct run-directory member",
        ));
    }
    require_native_target_dir_name(&target)?;
    Ok(target)
}

fn ensure_run_directory_for_preparation(run_dir: &Path, native_target: &Path) -> Result<()> {
    if native_target.parent() != Some(run_dir) {
        return Err(VerifierError::new(
            "native build target must be a direct run-directory member",
        ));
    }
    require_native_target_dir_name(native_target)?;
    let expected = BTreeSet::from([NATIVE_TARGET_DIR_NAME.to_string()]);
    if directory_names(run_dir)? != expected {
        return Err(VerifierError::new(
            "run directory must contain only the verifier-owned native target before preparation",
        ));
    }
    Ok(())
}

fn require_native_target_dir_name(target: &Path) -> Result<()> {
    if target.file_name().and_then(|name| name.to_str()) != Some(NATIVE_TARGET_DIR_NAME) {
        return Err(VerifierError::new(format!(
            "native build target directory must be named {NATIVE_TARGET_DIR_NAME}"
        )));
    }
    Ok(())
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

fn write_json_atomic_new(path: &Path, value: &Value, label: &str) -> Result<String> {
    let raw = canonical_json(value).into_bytes();
    let parent = path
        .parent()
        .ok_or_else(|| VerifierError::new(format!("{label} has no parent")))?;
    let parent = canonical_existing_dir(parent, &format!("{label} parent"))?;
    if fs::symlink_metadata(path).is_ok() {
        return Err(VerifierError::new(format!(
            "{label} already exists; replay rejected"
        )));
    }
    let temporary = parent.join(format!(
        ".{PREPARATION_RECEIPT_FILE}.tmp-{}",
        random_hex(16)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| VerifierError::new(format!("write {label}: {error}")))?;
        file.write_all(&raw)?;
        file.sync_all()?;
        drop(file);
        set_readonly_file(&temporary)?;
        let metadata = fs::symlink_metadata(&temporary)?;
        if metadata.file_type().is_symlink() || link_count(&metadata) != 1 {
            return Err(VerifierError::new(format!(
                "{label} temporary output is not a single-link regular file"
            )));
        }
        #[cfg(unix)]
        {
            // hard_link fails when the destination already exists, unlike
            // rename, so publication cannot overwrite a prior receipt.
            fs::hard_link(&temporary, path)?;
            fs::remove_file(&temporary)?;
        }
        #[cfg(not(unix))]
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        let published = immutable_file(path, label)?;
        if sha256_bytes(&fs::read(&published)?) != sha256_bytes(&raw) {
            return Err(VerifierError::new(format!(
                "{label} changed during publication"
            )));
        }
        Ok(sha256_bytes(&raw))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn hash_file(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(regular_file(path, "hash input")?)?))
}

fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0_u8; bytes];
    if let Ok(mut file) = File::open("/dev/urandom")
        && file.read_exact(&mut buffer).is_ok()
    {
        use std::fmt::Write as _;
        let mut output = String::with_capacity(bytes.saturating_mul(2));
        for byte in buffer {
            let _ = write!(output, "{byte:02x}");
        }
        return output;
    }
    sha256_bytes(format!("{}:{:?}", std::process::id(), Instant::now()).as_bytes())
        .chars()
        .take(bytes.saturating_mul(2))
        .collect()
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
    let path = regular_file(path, label)?.canonicalize().map_err(|error| {
        VerifierError::new(format!("{label} canonical path is unreadable: {error}"))
    })?;
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
    let deadline = cleanup_deadline(OBSERVER_CLEANUP_TIMEOUT, "child output collection")?;
    join_result_thread(handle, deadline, "child output reader")
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
    use std::sync::{Mutex, MutexGuard};

    static OBSERVER_PROVIDER_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock_observer_provider_env() -> MutexGuard<'static, ()> {
        OBSERVER_PROVIDER_ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    struct ObserverProviderEnvGuard {
        _lock: MutexGuard<'static, ()>,
        previous: Option<std::ffi::OsString>,
    }

    impl ObserverProviderEnvGuard {
        fn set(provider: &Path) -> Self {
            let lock = lock_observer_provider_env();
            let previous = std::env::var_os(OBSERVER_PROVIDER_ENV);
            // SAFETY: the process-wide mutation is serialized by the test lock
            // and restored by this guard before the lock is released.
            unsafe {
                std::env::set_var(OBSERVER_PROVIDER_ENV, provider.as_os_str());
            }
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for ObserverProviderEnvGuard {
        fn drop(&mut self) {
            // SAFETY: the guard still owns the serialized environment binding.
            unsafe {
                match self.previous.take() {
                    Some(value) => std::env::set_var(OBSERVER_PROVIDER_ENV, value),
                    None => std::env::remove_var(OBSERVER_PROVIDER_ENV),
                }
            }
        }
    }

    macro_rules! require_ok {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_ok(), "{}: {:?}", $message, result.as_ref().err());
            let Ok(value) = result else { return };
            value
        }};
    }

    macro_rules! require_some {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_some(), "{}", $message);
            let Some(value) = result else { return };
            value
        }};
    }

    macro_rules! require_err {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_err(), "{}", $message);
            let Err(error) = result else { return };
            error
        }};
    }

    macro_rules! require_ok_error {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_ok(), "{}", $message);
            let Ok(value) = result else {
                return VerifierError::new($message);
            };
            value
        }};
    }

    macro_rules! require_some_error {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_some(), "{}", $message);
            let Some(value) = result else {
                return VerifierError::new($message);
            };
            value
        }};
    }

    macro_rules! require_err_error {
        ($expression:expr, $message:expr) => {{
            let result = $expression;
            assert!(result.is_err(), "{}", $message);
            let Err(error) = result else {
                return VerifierError::new($message);
            };
            error
        }};
    }

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

    struct FailingProvider;

    fn observer_binding<'a>(
        operation: &'a str,
        tree: &'a str,
        oracle_commit: &'a str,
        nonce: &'a str,
    ) -> ObserverBinding<'a> {
        let sequence = Box::leak(vec![operation.to_string()].into_boxed_slice());
        ObserverBinding {
            run_id: "/run",
            task_id: "TASK-001",
            check_id: "CHK-001",
            tree,
            oracle_commit,
            nonce,
            sequence,
        }
    }

    impl ObserverProvider for FailingProvider {
        fn observe(&mut self, _request: &Map<String, Value>) -> Result<Map<String, Value>> {
            Err(VerifierError::new(
                "independent observer response is unavailable",
            ))
        }
    }

    struct ValidProvider;

    impl ObserverProvider for ValidProvider {
        fn observe(&mut self, request: &Map<String, Value>) -> Result<Map<String, Value>> {
            json!({
                "schema": "tc-proof-observation/v1",
                "nonce": request["nonce"].clone(),
                "run_id": request["run_id"].clone(),
                "task_id": request["task_id"].clone(),
                "check_id": request["check_id"].clone(),
                "request_id": request["request_id"].clone(),
                "operation": request["operation"].clone(),
                "source_commit": request["source_commit"].clone(),
                "tree": request["tree"].clone(),
                "exit": 0,
                "stdout": "host-observer",
                "stderr": "",
                "files": {},
                "payload": {"schema": "tc-proof-host-event/v1"},
                "records": [{"schema": "tc-proof-host-record/v1"}],
            })
            .as_object()
            .cloned()
            .ok_or_else(|| VerifierError::new("test provider response is not an object"))
        }
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

    #[cfg(unix)]
    #[test]
    fn supervisor_owned_observer_ends_are_close_on_exec() {
        let (request_read, request_write) = require_ok!(make_pipe(), "request pipe");
        let (response_read, response_write) = require_ok!(make_pipe(), "response pipe");
        require_ok!(
            protect_supervisor_observer_ends(&request_read, &response_write),
            "protect supervisor ends"
        );

        // SAFETY: each descriptor is owned by the corresponding live `File`.
        let request_read_flags = unsafe { libc::fcntl(request_read.as_raw_fd(), libc::F_GETFD) };
        // SAFETY: each descriptor is owned by the corresponding live `File`.
        let request_write_flags = unsafe { libc::fcntl(request_write.as_raw_fd(), libc::F_GETFD) };
        // SAFETY: each descriptor is owned by the corresponding live `File`.
        let response_read_flags = unsafe { libc::fcntl(response_read.as_raw_fd(), libc::F_GETFD) };
        let response_write_flags =
            // SAFETY: each descriptor is owned by the corresponding live `File`.
            unsafe { libc::fcntl(response_write.as_raw_fd(), libc::F_GETFD) };
        assert_ne!(request_read_flags & libc::FD_CLOEXEC, 0);
        assert_eq!(request_write_flags & libc::FD_CLOEXEC, 0);
        assert_eq!(response_read_flags & libc::FD_CLOEXEC, 0);
        assert_ne!(response_write_flags & libc::FD_CLOEXEC, 0);
    }

    #[cfg(unix)]
    #[test]
    fn kill_process_group_refuses_zero_and_broadcast() {
        assert!(kill_process_group(ProcessGroupId(0)).is_err());
        assert!(kill_process_group(ProcessGroupId(1)).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn terminate_child_kills_descendants_after_leader_exit() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;

        struct ProcessGroupGuard(ProcessGroupId);
        impl Drop for ProcessGroupGuard {
            fn drop(&mut self) {
                let _ = kill_process_group(self.0);
            }
        }

        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            // Redirect the descendant away from the leader pipe so reading the
            // pid cannot block on sleep, and so the descendant stays in the
            // captured group after the leader execs /usr/bin/true.
            .arg("/bin/sleep 30 </dev/null >/dev/null 2>&1 & printf '%s\\n' \"$!\"; exec /usr/bin/true")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // SAFETY: the pre-exec hook only creates a private process group for
        // the test leader, using the async-signal-safe setpgid operation.
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = require_ok!(command.spawn(), "spawn process-group leader");
        let process_group = require_ok!(ProcessGroupId::from_child(&child), "capture pgid");
        let _guard = ProcessGroupGuard(process_group);
        let mut stdout = BufReader::new(require_some!(child.stdout.take(), "leader stdout"));
        let mut listed = String::new();
        require_ok!(stdout.read_line(&mut listed), "read descendant pid");
        drop(stdout);
        let status = require_ok!(child.wait(), "reap leader");
        assert!(
            status.success(),
            "leader should exit after exec /usr/bin/true: {status:?}"
        );
        let descendant = require_ok!(listed.trim().parse::<libc::pid_t>(), "parse descendant pid");
        assert!(
            descendant > 1,
            "descendant pid {descendant} is not killable"
        );

        // SAFETY: descendant is the sleep child started in the captured group.
        let alive = unsafe { libc::kill(descendant, 0) };
        assert_eq!(alive, 0, "descendant should outlive the reaped leader");
        let deadline = require_ok!(
            cleanup_deadline(OBSERVER_CLEANUP_TIMEOUT, "descendant group kill"),
            "deadline"
        );
        require_ok!(
            terminate_child_until(&mut child, process_group, deadline),
            "kill captured group after leader reap"
        );
        let started = Instant::now();
        loop {
            // SAFETY: same descendant pid; ESRCH means the captured group kill landed.
            let gone = unsafe { libc::kill(descendant, 0) };
            if gone == -1 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "descendant {descendant} still exists after captured group kill"
            );
            thread::sleep(CLEANUP_POLL_INTERVAL);
        }
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

    #[cfg(unix)]
    #[test]
    fn trust_path_validation_rejects_symlinked_parents_and_preserves_allowed_ancestry()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let temporary = tempfile::tempdir_in("/tmp")?;
        let real_root = temporary.path().join("real");
        fs::create_dir_all(real_root.join("nested"))?;
        let file = real_root.join("nested/trusted.json");
        fs::write(&file, b"trusted")?;
        assert_eq!(regular_file(&file, "trusted file")?, file);
        assert!(canonical_existing_dir(&real_root, "candidate root")?.is_absolute());

        let hardlink = temporary.path().join("hardlink.json");
        fs::hard_link(&file, &hardlink)?;
        let Err(error) = regular_file(&hardlink, "trusted file") else {
            return Err("hard-linked trusted file was accepted".into());
        };
        assert!(error.to_string().contains("immutable regular file"));

        let alias = temporary.path().join("alias");
        symlink(&real_root, &alias)?;
        let aliased_file = alias.join("nested/trusted.json");
        let Err(error) = regular_file(&aliased_file, "trusted file") else {
            return Err("symlinked parent was accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("path component must not be a symlink")
        );
        let Err(error) = regular_path_exists(&alias.join("nested/result.json")) else {
            return Err("symlinked output parent was accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("path component must not be a symlink")
        );
        let Err(error) = canonical_existing_dir(&alias, "candidate root") else {
            return Err("symlinked directory was accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("path component must not be a symlink")
        );

        let executable = real_root.join("tool");
        fs::copy(std::env::current_exe()?, &executable)?;
        let mut permissions = fs::metadata(&executable)?.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&executable, permissions)?;
        let Err(error) = executable_identity(&alias.join("tool"), "proof tool") else {
            return Err("symlinked executable parent was accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("path component must not be a symlink")
        );
        Ok(())
    }

    struct Fixture {
        root: tempfile::TempDir,
        run_dir: PathBuf,
        observer_provider: PathBuf,
    }

    const QUALIFIED_TASKFMT_PATH: &str = "/tmp/taskfmt-latest-install/bin/taskfmt";
    const QUALIFIED_TASKFMT_SOURCE: &str = "/Users/donbeave/Projects/taskfmt/task-format";

    fn write_observer_provider_script(path: &Path, body: &str) -> std::result::Result<(), String> {
        fs::write(path, body).map_err(|error| error.to_string())?;
        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())
    }

    const DEFAULT_OBSERVER_PROVIDER: &str = r#"#!/usr/bin/env python3
import json
import sys
from pathlib import Path

try:
    for line in sys.stdin:
        request = json.loads(line)
        response = {
            "schema": "tc-proof-observation/v1",
            "nonce": request["nonce"],
            "run_id": request["run_id"],
            "task_id": request["task_id"],
            "check_id": request["check_id"],
            "request_id": request["request_id"],
            "operation": request["operation"],
            "source_commit": request["source_commit"],
            "tree": request["tree"],
            "exit": 0,
            "stdout": "host-observer",
            "stderr": "",
            "files": {},
            "payload": {"schema": "tc-proof-host-event/v1"},
            "records": [{"schema": "tc-proof-host-record/v1"}],
        }
        print(json.dumps(response, sort_keys=True, separators=(",", ":")), flush=True)
finally:
    Path(__file__).with_suffix(".closed").write_text("closed", encoding="utf-8")
"#;

    fn fixture() -> std::result::Result<(Fixture, PreparedRun), String> {
        let (fixture, options) = fixture_options()?;
        let prepared = prepare(&options).map_err(|error| error.to_string())?;
        Ok((fixture, prepared))
    }

    fn fixture_options() -> std::result::Result<(Fixture, PrepareOptions), String> {
        fixture_options_for("001", &[])
    }

    fn fixture_options_for(
        completion_task: &str,
        dependency_task_ids: &[&str],
    ) -> std::result::Result<(Fixture, PrepareOptions), String> {
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
        let worker = candidate.join("tools/refactor-proof/bin/tc-proof");
        let source_worker = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/tc-proof");
        fs::copy(&source_worker, &worker).map_err(|error| error.to_string())?;
        let mut worker_permissions = fs::metadata(&worker)
            .map_err(|error| error.to_string())?
            .permissions();
        worker_permissions.set_mode(worker_permissions.mode() | 0o111);
        fs::set_permissions(&worker, worker_permissions).map_err(|error| error.to_string())?;
        let staged = Command::new("git")
            .args(["-C"])
            .arg(&candidate)
            .args(["add", "--", "tools/refactor-proof/bin/tc-proof"])
            .output()
            .map_err(|error| error.to_string())?;
        if !staged.status.success() {
            return Err(String::from_utf8_lossy(&staged.stderr).into_owned());
        }
        let porcelain = Command::new("git")
            .args(["-C"])
            .arg(&candidate)
            .args([
                "status",
                "--porcelain",
                "--",
                "tools/refactor-proof/bin/tc-proof",
            ])
            .output()
            .map_err(|error| error.to_string())?;
        if !porcelain.status.success() {
            return Err(String::from_utf8_lossy(&porcelain.stderr).into_owned());
        }
        if !porcelain.stdout.is_empty() {
            let committed = Command::new("git")
                .args(["-C"])
                .arg(&candidate)
                .args([
                    "-c",
                    "user.name=proof-fixture",
                    "-c",
                    "user.email=proof-fixture@example.com",
                    "commit",
                    "-s",
                    "--no-gpg-sign",
                    "-m",
                    "overlay current dispatcher",
                ])
                .output()
                .map_err(|error| error.to_string())?;
            if !committed.status.success() {
                return Err(String::from_utf8_lossy(&committed.stderr).into_owned());
            }
        }
        let scope =
            git_output(&candidate, &["rev-parse", "HEAD"]).map_err(|error| error.to_string())?;
        let dependency_receipts = dependency_task_ids
            .iter()
            .map(|task_id| {
                let path = root.path().join(format!("{task_id}.receipt.json"));
                let receipt = json!({
                    "task_id": task_id,
                    "accepted": true,
                    "integrated": true,
                    "integration_commit": scope.clone(),
                });
                fs::write(&path, canonical_json(&receipt)).map_err(|error| error.to_string())?;
                Ok(path)
            })
            .collect::<std::result::Result<Vec<_>, String>>()?;
        let tree = git_output(&candidate, &["rev-parse", "HEAD^{tree}"])
            .map_err(|error| error.to_string())?;
        let run_dir = root.path().join("run");
        fs::create_dir(&run_dir).map_err(|error| error.to_string())?;
        let run_dir = run_dir.canonicalize().map_err(|error| error.to_string())?;
        let target_debug = run_dir.join("target/debug");
        fs::create_dir_all(&target_debug).map_err(|error| error.to_string())?;
        let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let comparator = target_debug.join("tc-proof");
        fs::copy(&current_exe, &comparator).map_err(|error| error.to_string())?;
        let mut permissions = fs::metadata(&comparator)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&comparator, permissions).map_err(|error| error.to_string())?;
        let build_receipt = target_debug.join("tc-proof.build.json");
        let build = json!({
            "schema": NATIVE_BUILD_SCHEMA,
            "worktree": candidate.to_string_lossy(),
            "target_dir": run_dir.join("target").to_string_lossy(),
            "cargo_target_dir": run_dir.join("target").to_string_lossy(),
            "commit": scope,
            "tree": tree,
            "binary": comparator.to_string_lossy(),
            "binary_sha256": hash_file(&comparator).map_err(|error| error.to_string())?,
            "command": ["fixture"],
        });
        fs::write(&build_receipt, canonical_json(&build)).map_err(|error| error.to_string())?;
        let taskfmt = PathBuf::from(QUALIFIED_TASKFMT_PATH);
        let observer_provider = root.path().join("observer-provider.py");
        write_observer_provider_script(&observer_provider, DEFAULT_OBSERVER_PROVIDER)?;
        let options = PrepareOptions {
            task_dir: candidate
                .join("refactoring-tasks/terminal-components/completion")
                .join(completion_task),
            run_dir: run_dir.clone(),
            worktree: candidate.clone(),
            scope_base: scope,
            oracle_tag: "refs/tags/visual-baseline".to_string(),
            oracle_commit: EXPECTED_ORACLE_COMMIT.to_string(),
            tool: worker,
            comparator,
            taskfmt: Some(taskfmt.clone()),
            native_build_receipt: build_receipt,
            taskfmt_source: PathBuf::from(QUALIFIED_TASKFMT_SOURCE),
            taskfmt_revision: QUALIFIED_TASKFMT_REVISION.to_string(),
            taskfmt_version: QUALIFIED_TASKFMT_VERSION.to_string(),
            taskfmt_sha256: QUALIFIED_TASKFMT_SHA256.to_string(),
            observer_provider: observer_provider.clone(),
            dependency_receipts,
            run_id: None,
            observer_nonce: Some("test-observer".to_string()),
            observer_socket: None,
        };
        Ok((
            Fixture {
                root,
                run_dir,
                observer_provider,
            },
            options,
        ))
    }

    #[test]
    fn materialization_has_exact_index_bound_context_set() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let index_raw = require_ok!(
            fs::read(fixture.run_dir.join("context-index.json")),
            "index"
        );
        let index = require_ok!(parse_json_object(&index_raw, "index"), "parse index");
        let index_members =
            require_some!(index.get("contexts").and_then(Value::as_array), "contexts");
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
                "observer_sequences",
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
        assert!(validate_prepared_run(&fixture.run_dir).is_ok());
    }

    #[test]
    fn materialization_uses_ledger_preparation_abi() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let index_raw = require_ok!(
            fs::read(fixture.run_dir.join("context-index.json")),
            "index"
        );
        let index = require_ok!(parse_json_object(&index_raw, "index"), "parse index");
        let context_path = require_some!(
            index
                .get("contexts")
                .and_then(Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.get("path"))
                .and_then(Value::as_str),
            "context path"
        );
        let result_path = require_some!(
            index
                .get("results")
                .and_then(Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.get("path"))
                .and_then(Value::as_str),
            "result path"
        );
        let context = require_ok!(
            parse_json_object(&require_ok!(fs::read(context_path), "context"), "context"),
            "parse context"
        );
        let result = require_ok!(
            parse_json_object(&require_ok!(fs::read(result_path), "result"), "result"),
            "parse result"
        );
        let observer_raw = require_ok!(fs::read(fixture.run_dir.join("observer.json")), "observer");
        let observer = require_ok!(
            parse_json_object(&observer_raw, "observer"),
            "parse observer"
        );
        assert_eq!(
            context.get("schema").and_then(Value::as_str),
            Some(CONTEXT_SCHEMA)
        );
        assert_eq!(
            result.get("schema").and_then(Value::as_str),
            Some(PREPARATION_RESULT_SCHEMA)
        );
        assert_eq!(
            observer.get("schema").and_then(Value::as_str),
            Some(OBSERVER_SCHEMA)
        );
        assert_eq!(
            index.get("schema").and_then(Value::as_str),
            Some(INDEX_SCHEMA)
        );
    }

    fn context_by_check(
        index: &Map<String, Value>,
        check_id: &str,
    ) -> std::result::Result<Map<String, Value>, String> {
        let entry = index
            .get("contexts")
            .and_then(Value::as_array)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|entry| entry.get("check_id").and_then(Value::as_str) == Some(check_id))
            })
            .ok_or_else(|| format!("missing {check_id} context entry"))?;
        let context_path = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing {check_id} path"))?;
        parse_json_object(
            &fs::read(context_path).map_err(|error| error.to_string())?,
            "context",
        )
        .map_err(|error| error.to_string())
    }

    #[test]
    fn oracle_context_binds_explicit_synthetic_family() {
        let (fixture, options) = require_ok!(
            fixture_options_for("002", &["071", "072"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        let index_raw = require_ok!(
            fs::read(fixture.run_dir.join("context-index.json")),
            "index"
        );
        let index = require_ok!(parse_json_object(&index_raw, "index"), "parse index");
        let context = require_ok!(context_by_check(&index, "CHK-003"), "CHK-003 context");
        assert_eq!(
            context.get("operation").and_then(Value::as_str),
            Some("oracle")
        );
        assert_eq!(
            context.get("observer_sequence").and_then(Value::as_array),
            Some(&vec![Value::String("oracle".to_string())]),
        );
        assert_eq!(
            context
                .get("qualification")
                .and_then(Value::as_object)
                .and_then(|qualification| qualification.get("family"))
                .and_then(Value::as_str),
            Some("native"),
        );
        assert_eq!(
            context
                .get("qualification")
                .and_then(Value::as_object)
                .and_then(|qualification| qualification.get("check"))
                .and_then(Value::as_object)
                .and_then(|check| check.get("namespace"))
                .and_then(Value::as_str),
            Some("showcase"),
        );

        let compare = require_ok!(context_by_check(&index, "CHK-004"), "CHK-004 context");
        let comparator = require_some!(
            compare
                .get("qualification")
                .and_then(Value::as_object)
                .and_then(|qualification| qualification.get("comparator"))
                .and_then(Value::as_object),
            "nested comparator"
        );
        assert_eq!(
            comparator.get("schema").and_then(Value::as_str),
            Some(COMPARE_CONTEXT_SCHEMA)
        );
        assert_eq!(
            comparator
                .get("context")
                .and_then(Value::as_object)
                .and_then(|nested| nested.get("schema"))
                .and_then(Value::as_str),
            Some(COMPARE_CONTEXT_SCHEMA)
        );
        let report_path = require_some!(
            comparator.get("report_path").and_then(Value::as_str),
            "comparator report_path"
        );
        assert_eq!(
            Path::new(report_path)
                .file_name()
                .and_then(|name| name.to_str()),
            Some("CHK-004.compare.json")
        );
        assert_eq!(
            Path::new(report_path).parent().map(PathBuf::from),
            Some(fixture.run_dir.join("outputs"))
        );

        let accounting = require_ok!(context_by_check(&index, "CHK-005"), "CHK-005 context");
        assert_eq!(
            accounting
                .get("qualification")
                .and_then(Value::as_object)
                .and_then(|qualification| qualification.get("family"))
                .and_then(Value::as_str),
            Some("accounting"),
        );
        assert!(
            prepared
                .members
                .iter()
                .any(|member| member.check_id == "CHK-003")
        );
        assert!(
            prepared
                .members
                .iter()
                .any(|member| member.check_id == "CHK-004"
                    && member.comparator_report_path.is_some())
        );
    }

    #[test]
    fn oracle_observer_family_is_explicit_and_fail_closed() {
        let check = CheckSpec {
            id: "CHK-003".to_string(),
            phase: "focused".to_string(),
            operation: "oracle".to_string(),
            context_ref: None,
            lane: "direct".to_string(),
            namespace: String::new(),
            requirements: Vec::new(),
            acceptance: Vec::new(),
            command: json!(["tc-proof", "oracle"]),
        };
        let template = |family: &str| TemplateInfo {
            path: PathBuf::from("/tmp/template.json"),
            sha256: String::new(),
            value: json!({"qualification": {"family": family}}),
        };
        assert_eq!(
            observer_sequence_for_check(&check, Some(&template("native")))
                .expect("native sequence"),
            vec!["oracle"],
        );
        assert_eq!(
            observer_sequence_for_check(&check, None).expect("synthetic sequence"),
            vec!["oracle", "oracle"],
        );
        assert_eq!(
            observer_family_for_check(&check, None).expect("empty namespace stays synthetic"),
            "synthetic",
        );
        assert!(observer_sequence_for_check(&check, Some(&template("accounting"))).is_err());

        let native_namespace = CheckSpec {
            namespace: "showcase".to_string(),
            command: json!(["tc-proof", "oracle", "--namespace", "showcase"]),
            ..check.clone()
        };
        assert_eq!(
            observer_family_for_check(&native_namespace, None).expect("app namespace is native"),
            "native",
        );
        assert_eq!(
            observer_sequence_for_check(&native_namespace, None)
                .expect("native namespace sequence"),
            vec!["oracle"],
        );
        let synthetic_namespace = CheckSpec {
            namespace: "synthetic".to_string(),
            command: json!(["tc-proof", "oracle", "--namespace", "synthetic"]),
            ..check.clone()
        };
        assert_eq!(
            observer_family_for_check(&synthetic_namespace, None)
                .expect("synthetic namespace stays synthetic"),
            "synthetic",
        );

        let mut missing = Map::new();
        missing.insert("observer_sequence".to_string(), json!(["oracle", "oracle"]));
        missing.insert("qualification".to_string(), json!({}));
        assert!(observer_sequence_from_context(&missing, "oracle").is_err());

        let mut synthetic = Map::new();
        synthetic.insert("observer_sequence".to_string(), json!(["oracle", "oracle"]));
        synthetic.insert("qualification".to_string(), json!({"family": "synthetic"}));
        assert_eq!(
            observer_sequence_from_context(&synthetic, "oracle").expect("synthetic context"),
            vec!["oracle", "oracle"],
        );
    }

    #[test]
    fn empty_namespace_oracle_stays_synthetic() {
        let check = CheckSpec {
            id: "CHK-003".to_string(),
            phase: "focused".to_string(),
            operation: "oracle".to_string(),
            context_ref: None,
            lane: "direct".to_string(),
            namespace: String::new(),
            requirements: Vec::new(),
            acceptance: Vec::new(),
            command: json!(["tc-proof", "oracle"]),
        };
        assert_eq!(
            observer_family_for_check(&check, None).expect("empty namespace family"),
            "synthetic",
        );
        assert_eq!(
            observer_sequence_for_check(&check, None).expect("empty namespace sequence"),
            vec!["oracle", "oracle"],
        );
    }

    #[test]
    fn preparation_receipt_is_immutable_and_binds_exact_run_members() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let receipt_path = fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let metadata = require_ok!(fs::symlink_metadata(&receipt_path), "receipt metadata");
        assert_eq!(link_count(&metadata), 1);
        assert!(permissions_are_readonly(&metadata));
        let receipt = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&receipt_path), "receipt bytes"),
                "receipt",
            ),
            "receipt JSON"
        );
        assert_eq!(
            receipt.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "schema",
                "task_id",
                "worktree",
                "commit",
                "scope_base",
                "run_id",
                "native_build",
                "taskfmt",
                "context_index",
                "contexts",
                "results",
                "observer",
            ])
        );
        assert_eq!(
            receipt.get("schema").and_then(Value::as_str),
            Some(PREPARATION_RECEIPT_SCHEMA)
        );
        assert!(validate_prepared_run(&fixture.run_dir).is_ok());
        assert!(validate_preparation_receipt(&prepared).is_ok());
        let root_names = require_ok!(directory_names(&fixture.run_dir), "run root names");
        assert_eq!(
            root_names,
            BTreeSet::from([
                "contexts".to_string(),
                "results".to_string(),
                "outputs".to_string(),
                "taskfmt-logs".to_string(),
                "target".to_string(),
                "observer.json".to_string(),
                "context-index.json".to_string(),
                PREPARATION_RECEIPT_FILE.to_string(),
            ])
        );
    }

    #[test]
    fn preparation_receipt_mutation_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let receipt_path = fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let mut permissions =
            require_ok!(fs::metadata(&receipt_path), "receipt metadata").permissions();
        permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&receipt_path, permissions),
            "make receipt writable"
        );
        let mut receipt = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&receipt_path), "receipt bytes"),
                "receipt",
            ),
            "receipt JSON"
        );
        receipt.insert("scope_base".to_string(), Value::String("0".repeat(40)));
        require_ok!(
            fs::write(&receipt_path, canonical_json(&Value::Object(receipt))),
            "mutate receipt"
        );
        require_ok!(set_readonly_file(&receipt_path), "restore receipt readonly");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn preparation_receipt_deletion_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        require_ok!(
            fs::remove_file(fixture.run_dir.join(PREPARATION_RECEIPT_FILE)),
            "delete receipt"
        );
        let error = require_err!(validate_prepared_run(&fixture.run_dir), "missing receipt");
        assert!(error.to_string().contains("receipt"));
    }

    #[cfg(unix)]
    #[test]
    fn preparation_receipt_symlink_and_hardlink_substitution_are_rejected() {
        use std::os::unix::fs::symlink;

        let (symlink_fixture, _) = require_ok!(fixture(), "fixture");
        let receipt_path = symlink_fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let symlink_target = symlink_fixture.root.path().join("receipt-copy.json");
        require_ok!(
            fs::copy(&receipt_path, &symlink_target),
            "copy receipt for symlink"
        );
        require_ok!(fs::remove_file(&receipt_path), "remove receipt");
        require_ok!(
            symlink(&symlink_target, &receipt_path),
            "replace receipt with symlink"
        );
        assert!(validate_prepared_run(&symlink_fixture.run_dir).is_err());

        let (hardlink_fixture, _) = require_ok!(fixture(), "hardlink fixture");
        let receipt_path = hardlink_fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let hardlink = hardlink_fixture.root.path().join("receipt-hardlink.json");
        require_ok!(fs::hard_link(&receipt_path, &hardlink), "hardlink receipt");
        assert!(validate_prepared_run(&hardlink_fixture.run_dir).is_err());
    }

    #[test]
    fn taskfmt_provenance_mutation_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let receipt_path = fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let mut permissions =
            require_ok!(fs::metadata(&receipt_path), "receipt metadata").permissions();
        permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&receipt_path, permissions),
            "make receipt writable"
        );
        let mut receipt = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&receipt_path), "receipt bytes"),
                "receipt",
            ),
            "receipt JSON"
        );
        let taskfmt = require_some!(
            receipt.get_mut("taskfmt").and_then(Value::as_object_mut),
            "taskfmt object"
        );
        taskfmt.insert("taskfmt_sha256".to_string(), Value::String("0".repeat(64)));
        require_ok!(
            fs::write(&receipt_path, canonical_json(&Value::Object(receipt))),
            "mutate taskfmt provenance"
        );
        require_ok!(set_readonly_file(&receipt_path), "restore receipt readonly");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn native_build_provenance_mutation_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let build_path = fixture.run_dir.join("target/debug/tc-proof.build.json");
        let mut build_permissions =
            require_ok!(fs::metadata(&build_path), "build receipt metadata").permissions();
        build_permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&build_path, build_permissions),
            "make build receipt writable"
        );
        let mut build = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&build_path), "build receipt bytes"),
                "build receipt",
            ),
            "build receipt JSON"
        );
        build.insert("tree".to_string(), Value::String("0".repeat(40)));
        require_ok!(
            fs::write(&build_path, canonical_json(&Value::Object(build))),
            "mutate build receipt"
        );
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn preparation_rejects_non_target_direct_native_build_member() {
        let (fixture, mut options) = require_ok!(fixture_options(), "fixture options");
        let target = fixture.run_dir.join(NATIVE_TARGET_DIR_NAME);
        let non_target = fixture.run_dir.join("native-cache");
        require_ok!(fs::rename(&target, &non_target), "rename native target");

        let build_path = non_target.join("debug/tc-proof.build.json");
        let binary_path = non_target.join("debug/tc-proof");
        options.native_build_receipt = build_path.clone();
        options.comparator = binary_path.clone();
        let mut build = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&build_path), "build receipt"),
                "build"
            ),
            "parse build receipt"
        );
        build.insert(
            "target_dir".to_string(),
            Value::String(non_target.to_string_lossy().into_owned()),
        );
        build.insert(
            "cargo_target_dir".to_string(),
            Value::String(non_target.to_string_lossy().into_owned()),
        );
        build.insert(
            "binary".to_string(),
            Value::String(binary_path.to_string_lossy().into_owned()),
        );
        require_ok!(
            fs::write(&build_path, canonical_json(&Value::Object(build))),
            "rewrite build receipt"
        );

        let error = require_err!(
            prepare(&options),
            "non-target native build member must be rejected"
        );
        assert!(error.to_string().contains("must be named target"));
        assert!(!fixture.run_dir.join(PREPARATION_RECEIPT_FILE).exists());
    }

    #[test]
    fn preparation_failure_does_not_publish_receipt() {
        let (fixture, options) = require_ok!(fixture_options(), "fixture options");
        require_ok!(
            fs::remove_file(&options.native_build_receipt),
            "remove native build receipt"
        );
        let error = require_err!(prepare(&options), "preparation failure");
        assert!(error.to_string().contains("native build receipt"));
        assert!(!fixture.run_dir.join(PREPARATION_RECEIPT_FILE).exists());
    }

    #[test]
    fn legacy_runner_index_shape_is_rejected_by_native_validator() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let index_path = fixture.run_dir.join("context-index.json");
        let mut permissions =
            require_ok!(fs::metadata(&index_path), "index metadata").permissions();
        permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&index_path, permissions),
            "make index writable"
        );
        let legacy_index = require_ok!(
            serde_json::to_vec(&json!({
                "schema": INDEX_SCHEMA,
                "task_id": "TASK-001",
                "run_id": fixture.run_dir.to_string_lossy(),
                "tree": "deadbeef",
                "trust_sha256": "deadbeef",
                "members": [],
            })),
            "legacy index JSON"
        );
        require_ok!(fs::write(&index_path, legacy_index), "write legacy index");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn extra_runtime_output_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        require_ok!(
            fs::write(fixture.run_dir.join("outputs/extra.json"), b"{}"),
            "extra output"
        );
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn materialization_rejects_context_symlink_and_hardlink() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        let context_path = member.context_path.clone();
        let contexts_dir = require_some!(context_path.parent(), "contexts parent");
        let mut permissions =
            require_ok!(fs::metadata(contexts_dir), "contexts metadata").permissions();
        permissions.set_mode(0o755);
        require_ok!(
            fs::set_permissions(contexts_dir, permissions),
            "make contexts writable"
        );
        let alias = fixture.root.path().join("context-alias");
        require_ok!(fs::hard_link(&context_path, &alias), "hard link");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
        let _ = fs::remove_file(&alias);
        let other = require_some!(
            prepared
                .members
                .get(1)
                .map(|member| member.context_path.clone()),
            "second member"
        );
        require_ok!(fs::remove_file(&context_path), "remove context");
        require_ok!(std::os::unix::fs::symlink(other, &context_path), "symlink");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[test]
    fn index_hash_and_candidate_identity_mutations_fail_closed() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let index_path = fixture.run_dir.join("context-index.json");
        let mut permissions =
            require_ok!(fs::metadata(&index_path), "index metadata").permissions();
        permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&index_path, permissions),
            "make index writable"
        );
        require_ok!(fs::write(&index_path, b"{}"), "mutate index");
        assert!(validate_prepared_run(&fixture.run_dir).is_err());
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the test fixture mirrors the complete context/result/index rebinding attack"
    )]
    fn mutate_first_context_and_rebind_hashes(
        fixture: &Fixture,
        mutate: impl FnOnce(&mut Map<String, Value>),
    ) -> VerifierError {
        let index_path = fixture.run_dir.join("context-index.json");
        let contexts_dir = fixture.run_dir.join("contexts");
        let results_dir = fixture.run_dir.join("results");
        let mut directory_permissions =
            require_ok_error!(fs::metadata(&contexts_dir), "contexts metadata").permissions();
        directory_permissions.set_mode(0o755);
        require_ok_error!(
            fs::set_permissions(&contexts_dir, directory_permissions.clone()),
            "make contexts writable"
        );
        require_ok_error!(
            fs::set_permissions(&results_dir, directory_permissions),
            "make results writable"
        );
        let mut index_permissions =
            require_ok_error!(fs::metadata(&index_path), "index metadata").permissions();
        index_permissions.set_mode(0o644);
        require_ok_error!(
            fs::set_permissions(&index_path, index_permissions),
            "make index writable"
        );

        let mut index = require_ok_error!(
            parse_json_object(&require_ok_error!(fs::read(&index_path), "index"), "index"),
            "parse index"
        );
        let context_path = PathBuf::from(require_some_error!(
            index
                .get("contexts")
                .and_then(Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.get("path"))
                .and_then(Value::as_str),
            "context path"
        ));
        let result_path = PathBuf::from(require_some_error!(
            index
                .get("results")
                .and_then(Value::as_array)
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.get("path"))
                .and_then(Value::as_str),
            "result path"
        ));
        let mut context = require_ok_error!(
            parse_json_object(
                &require_ok_error!(fs::read(&context_path), "context"),
                "context"
            ),
            "parse context"
        );
        let mut context_permissions =
            require_ok_error!(fs::metadata(&context_path), "context metadata").permissions();
        context_permissions.set_mode(0o644);
        require_ok_error!(
            fs::set_permissions(&context_path, context_permissions.clone()),
            "make context writable"
        );
        let mut result_permissions =
            require_ok_error!(fs::metadata(&result_path), "result metadata").permissions();
        result_permissions.set_mode(0o644);
        require_ok_error!(
            fs::set_permissions(&result_path, result_permissions),
            "make result writable"
        );

        mutate(&mut context);
        let context_raw = canonical_json(&Value::Object(context)).into_bytes();
        require_ok_error!(fs::write(&context_path, &context_raw), "rewrite context");
        let context_hash = sha256_bytes(&context_raw);
        if let Some(contexts) = index.get_mut("contexts").and_then(Value::as_array_mut)
            && let Some(context_entry) = contexts.first_mut().and_then(Value::as_object_mut)
        {
            context_entry.insert("sha256".to_owned(), Value::String(context_hash.clone()));
        } else {
            return VerifierError::new("context entry missing");
        }

        let mut preparation = require_ok_error!(
            parse_json_object(
                &require_ok_error!(fs::read(&result_path), "preparation"),
                "preparation"
            ),
            "parse preparation"
        );
        preparation.insert("context_sha256".to_owned(), Value::String(context_hash));
        let preparation_raw = canonical_json(&Value::Object(preparation)).into_bytes();
        require_ok_error!(
            fs::write(&result_path, &preparation_raw),
            "rewrite preparation"
        );
        if let Some(results) = index.get_mut("results").and_then(Value::as_array_mut)
            && let Some(result_entry) = results.first_mut().and_then(Value::as_object_mut)
        {
            result_entry.insert(
                "sha256".to_owned(),
                Value::String(sha256_bytes(&preparation_raw)),
            );
        } else {
            return VerifierError::new("result entry missing");
        }
        let index_raw = canonical_json(&Value::Object(index)).into_bytes();
        require_ok_error!(fs::write(&index_path, &index_raw), "rewrite index");

        require_ok_error!(set_readonly_file(&context_path), "restore context readonly");
        require_ok_error!(set_readonly_file(&result_path), "restore result readonly");
        require_ok_error!(set_readonly_file(&index_path), "restore index readonly");
        require_ok_error!(set_readonly_dir(&contexts_dir), "restore contexts readonly");
        require_ok_error!(set_readonly_dir(&results_dir), "restore results readonly");

        require_err_error!(
            validate_prepared_run(&fixture.run_dir),
            "mutated context was accepted"
        )
    }

    #[test]
    fn recheck_rejects_wrong_source_tree_after_context_rebinding() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let error = mutate_first_context_and_rebind_hashes(&fixture, |context| {
            context["tree"] = Value::String("f".repeat(40));
        });
        assert!(error.to_string().contains("source tree"));
    }

    #[test]
    fn recheck_rejects_wrong_oracle_commit_after_context_rebinding() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let error = mutate_first_context_and_rebind_hashes(&fixture, |context| {
            context["oracle_commit"] = Value::String("f".repeat(40));
        });
        assert!(error.to_string().contains("oracle commit"));
    }

    #[test]
    fn recheck_rejects_wrong_tool_after_context_rebinding() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let wrong_tool = Path::new("/bin/sh");
        let wrong_hash = sha256_bytes(&require_ok!(fs::read(wrong_tool), "wrong tool"));
        let error = mutate_first_context_and_rebind_hashes(&fixture, |context| {
            context["tool"] = json!({
                "path": wrong_tool,
                "sha256": wrong_hash,
            });
        });
        assert!(error.to_string().contains("tool identity"));
    }

    #[test]
    fn worker_passed_result_without_observer_is_rejected() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        let _env_lock = lock_observer_provider_env();
        let script = r#"printf '{"category":null,"context_sha256":"%s","observation_digests":[],"operation":"external","outputs":{},"run_id":"%s","schema":"tc-proof-runner-result/v1","status":"passed"}' "$TC_PROOF_CONTEXT_SHA256" "$TC_PROOF_RUN_ID" > "$TC_PROOF_RESULT""#;
        let record = launch(&LaunchOptions {
            run_dir: fixture.run_dir.clone(),
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_secs(5),
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".to_string(), script.to_string()],
        });
        let error = require_err!(record, "worker result without an observation");
        assert!(error.to_string().contains("observer"));
    }

    #[test]
    fn tracked_worker_preflight_launch_succeeds_with_bound_observer() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "fixture preparation");
        let member = require_some!(prepared.members.first(), "first member");
        assert_eq!(member.operation, "preflight");
        let provider_closed = fixture.observer_provider.with_extension("closed");

        let launch_result = launch(&LaunchOptions {
            run_dir: fixture.run_dir.clone(),
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_secs(10),
            program: fixture
                .root
                .path()
                .join("candidate/tools/refactor-proof/bin/tc-proof"),
            args: vec![
                "preflight".to_string(),
                "--context".to_string(),
                member.context_path.to_string_lossy().into_owned(),
            ],
        });

        let record = match launch_result {
            Ok(record) => record,
            Err(error) => {
                let run_dir = fixture.run_dir.clone();
                let result = fs::read_to_string(&member.result_path)
                    .unwrap_or_else(|read_error| format!("<unreadable: {read_error}>"));
                let _fixture = std::mem::ManuallyDrop::new(fixture);
                panic!("tracked worker launch: {error}; result: {result}; run_dir: {run_dir:?}");
            }
        };
        assert_eq!(record.exit, 0);
        assert_eq!(
            require_ok!(fs::read(&provider_closed), "observer provider close marker"),
            b"closed"
        );
        let result_metadata = require_ok!(fs::metadata(&member.result_path), "result metadata");
        assert!(permissions_are_readonly(&result_metadata));
        assert!(validate_prepared_run(&fixture.run_dir).is_ok());
    }

    #[test]
    fn provider_hang_after_acceptance_is_bounded_and_rejected() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        require_ok!(
            write_observer_provider_script(
                &fixture.observer_provider,
                r#"#!/usr/bin/env python3
import json
import sys
import time
from pathlib import Path

for line in sys.stdin:
    json.loads(line)
    Path(__file__).with_suffix(".accepted").write_text("accepted", encoding="utf-8")
    while True:
        time.sleep(1)
"#,
            ),
            "hanging observer provider"
        );
        let accepted = fixture.observer_provider.with_extension("accepted");
        let prepared = require_ok!(prepare(&options), "fixture preparation");
        let member = require_some!(prepared.members.first(), "first member");

        let started = Instant::now();
        let launch_result = launch(&LaunchOptions {
            run_dir: fixture.run_dir.clone(),
            check_id: member.check_id.clone(),
            observer_socket: None,
            timeout: Duration::from_secs(1),
            program: fixture
                .root
                .path()
                .join("candidate/tools/refactor-proof/bin/tc-proof"),
            args: vec![
                "preflight".to_string(),
                "--context".to_string(),
                member.context_path.to_string_lossy().into_owned(),
            ],
        });
        let elapsed = started.elapsed();
        let error = require_err!(
            launch_result,
            "a provider that hangs after accepting a request must be rejected"
        );
        assert!(
            elapsed < Duration::from_secs(4),
            "provider hang was not bounded: {elapsed:?}; error: {error}"
        );
        assert_eq!(
            require_ok!(fs::read(&accepted), "provider acceptance marker"),
            b"accepted"
        );
    }

    #[test]
    fn subprocess_nonzero_and_timeout_are_rejected() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.get(1), "second member");
        let _env_lock = lock_observer_provider_env();
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
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        let _env_lock = lock_observer_provider_env();
        let error = require_err!(
            launch(&LaunchOptions {
                run_dir: fixture.run_dir,
                check_id: member.check_id.clone(),
                observer_socket: Some(PathBuf::from("/tmp/retired-observer.sock")),
                timeout: Duration::from_secs(5),
                program: PathBuf::from("/bin/true"),
                args: Vec::new(),
            }),
            "retired socket should be rejected"
        );
        assert!(error.to_string().contains("alternate observer"));
    }

    #[test]
    fn observer_protocol_rejects_missing_independent_response() {
        let (request_read, mut request_write) = require_ok!(make_pipe(), "request pipe");
        let (response_read, response_write) = require_ok!(make_pipe(), "response pipe");
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                let mut provider = FailingProvider;
                observe_worker(
                    request_read,
                    response_write,
                    &mut provider,
                    &observer_binding("capture", &tree, &source_commit, "nonce"),
                )
            }
        });
        require_ok!(
            request_write.write_all(
                observer_request("nonce", 0, "capture", &source_commit, &tree).as_bytes()
            ),
            "request"
        );
        require_ok!(request_write.flush(), "flush request");
        drop(request_write);
        drop(response_read);
        let error = require_err!(
            require_ok!(supervisor.join(), "supervisor join"),
            "missing independent response"
        );
        assert!(error.to_string().contains("independent observer response"));
    }

    #[test]
    fn observer_protocol_rejects_wrong_nonce() {
        let (request_read, mut request_write) = require_ok!(make_pipe(), "request pipe");
        let (response_read, response_write) = require_ok!(make_pipe(), "response pipe");
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                let mut provider = FailingProvider;
                observe_worker(
                    request_read,
                    response_write,
                    &mut provider,
                    &observer_binding("capture", &tree, &source_commit, "nonce"),
                )
            }
        });
        require_ok!(
            request_write.write_all(
                observer_request("wrong", 0, "capture", &source_commit, &tree).as_bytes()
            ),
            "request"
        );
        require_ok!(request_write.flush(), "flush request");
        drop(request_write);
        drop(response_read);
        let error = require_err!(
            require_ok!(supervisor.join(), "supervisor join"),
            "wrong nonce"
        );
        assert!(error.to_string().contains("binding mismatch"));
    }

    #[test]
    fn observer_protocol_rejects_truncation_and_replay() {
        let (request_read, mut request_write) = require_ok!(make_pipe(), "request pipe");
        let (response_read, response_write) = require_ok!(make_pipe(), "response pipe");
        let supervisor = thread::spawn(move || {
            let mut provider = FailingProvider;
            observe_worker(
                request_read,
                response_write,
                &mut provider,
                &observer_binding("capture", &"b".repeat(40), &"a".repeat(40), "nonce"),
            )
        });
        require_ok!(
            request_write.write_all(b"{\"schema\":\"tc-proof-runner-observe/v1\""),
            "truncated request"
        );
        drop(request_write);
        drop(response_read);
        let error = require_err!(
            require_ok!(supervisor.join(), "supervisor join"),
            "truncation"
        );
        assert!(error.to_string().contains("truncated"));

        let (request_read, mut request_write) = require_ok!(make_pipe(), "replay request pipe");
        let (response_read, response_write) = require_ok!(make_pipe(), "replay response pipe");
        let mut response_read = BufReader::new(response_read);
        let source_commit = "a".repeat(40);
        let tree = "b".repeat(40);
        let supervisor = thread::spawn({
            let source_commit = source_commit.clone();
            let tree = tree.clone();
            move || {
                let mut provider = ValidProvider;
                observe_worker(
                    request_read,
                    response_write,
                    &mut provider,
                    &observer_binding("capture", &tree, &source_commit, "nonce"),
                )
            }
        });
        let request = observer_request("nonce", 0, "capture", &source_commit, &tree);
        require_ok!(request_write.write_all(request.as_bytes()), "request");
        require_ok!(request_write.write_all(request.as_bytes()), "replay");
        require_ok!(request_write.flush(), "flush request");
        let mut response = String::new();
        require_ok!(response_read.read_line(&mut response), "replay response");
        drop(request_write);
        drop(response_read);
        let error = require_err!(require_ok!(supervisor.join(), "supervisor join"), "replay");
        assert!(error.to_string().contains("replay"));
    }

    fn write_done_log(run_dir: &Path) -> std::result::Result<(), String> {
        let path = run_dir.join("taskfmt-logs/verify.log");
        fs::write(&path, "taskfmt verify\nDONE\n").map_err(|error| error.to_string())
    }

    fn write_member_result(
        member: &PreparedMember,
        run_id: &str,
        status: &str,
    ) -> std::result::Result<(), String> {
        let digest = "a".repeat(64);
        let result = json!({
            "schema": RESULT_SCHEMA,
            "run_id": run_id,
            "operation": member.operation,
            "context_sha256": member.context_sha256,
            "status": status,
            "category": if status == "passed" { Value::Null } else { Value::String("PROTOCOL".into()) },
            "observation_digests": [digest],
            "outputs": {},
        });
        fs::write(&member.result_path, canonical_json(&result))
            .map_err(|error| error.to_string())?;
        if let Some(report) = &member.comparator_report_path {
            fs::write(report, b"{}").map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn write_complete_runtime_closure(prepared: &PreparedRun) -> std::result::Result<(), String> {
        write_done_log(&prepared.run_dir)?;
        for member in &prepared.members {
            write_member_result(member, &prepared.run_id, "passed")?;
        }
        Ok(())
    }

    #[test]
    fn observer_provider_byte_mutation_is_rejected() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        let mut raw = require_ok!(fs::read(&fixture.observer_provider), "provider bytes");
        raw.push(b'x');
        require_ok!(
            fs::write(&fixture.observer_provider, raw),
            "mutate provider bytes"
        );
        let error = require_err!(
            launch(&LaunchOptions {
                run_dir: fixture.run_dir,
                check_id: member.check_id.clone(),
                observer_socket: None,
                timeout: Duration::from_secs(5),
                program: PathBuf::from("/bin/true"),
                args: Vec::new(),
            }),
            "mutated provider bytes must be rejected"
        );
        assert!(
            error.to_string().contains("hash"),
            "expected hash rejection, got {error}"
        );
    }

    #[test]
    fn observer_provider_binary_replacement_is_rejected() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        require_ok!(
            fs::copy("/bin/sh", &fixture.observer_provider),
            "replace provider binary"
        );
        let mut permissions = require_ok!(
            fs::metadata(&fixture.observer_provider),
            "replacement metadata"
        )
        .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        require_ok!(
            fs::set_permissions(&fixture.observer_provider, permissions),
            "replacement executable"
        );
        let error = require_err!(
            launch(&LaunchOptions {
                run_dir: fixture.run_dir,
                check_id: member.check_id.clone(),
                observer_socket: None,
                timeout: Duration::from_secs(5),
                program: PathBuf::from("/bin/true"),
                args: Vec::new(),
            }),
            "replaced provider binary must be rejected"
        );
        assert!(
            error.to_string().contains("hash"),
            "expected hash rejection, got {error}"
        );
    }

    #[test]
    fn observer_provider_wrong_hash_is_rejected() {
        let (fixture, _) = require_ok!(fixture(), "fixture");
        let observer_path = fixture.run_dir.join("observer.json");
        let mut permissions =
            require_ok!(fs::metadata(&observer_path), "observer metadata").permissions();
        permissions.set_mode(0o644);
        require_ok!(
            fs::set_permissions(&observer_path, permissions),
            "make observer writable"
        );
        let mut observer = require_ok!(
            parse_json_object(
                &require_ok!(fs::read(&observer_path), "observer bytes"),
                "observer",
            ),
            "observer JSON"
        );
        let provider = require_some!(
            observer.get_mut("provider").and_then(Value::as_object_mut),
            "provider object"
        );
        provider.insert("sha256".to_string(), Value::String("0".repeat(64)));
        require_ok!(
            fs::write(&observer_path, canonical_json(&Value::Object(observer))),
            "write wrong provider hash"
        );
        require_ok!(
            set_readonly_file(&observer_path),
            "restore observer readonly"
        );
        let error = require_err!(
            validate_prepared_run(&fixture.run_dir),
            "wrong provider hash must be rejected"
        );
        assert!(
            error.to_string().contains("hash"),
            "expected hash rejection, got {error}"
        );
    }

    #[test]
    fn observer_provider_env_rebinding_is_rejected() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        let member = require_some!(prepared.members.first(), "first member");
        let other = fixture.root.path().join("other-provider");
        require_ok!(
            fs::copy(&fixture.observer_provider, &other),
            "copy provider"
        );
        let mut permissions =
            require_ok!(fs::metadata(&other), "other provider metadata").permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        require_ok!(fs::set_permissions(&other, permissions), "other executable");
        let error = {
            let _env = ObserverProviderEnvGuard::set(&other);
            require_err!(
                launch(&LaunchOptions {
                    run_dir: fixture.run_dir,
                    check_id: member.check_id.clone(),
                    observer_socket: None,
                    timeout: Duration::from_secs(5),
                    program: PathBuf::from("/bin/true"),
                    args: Vec::new(),
                }),
                "env rebinding must be rejected"
            )
        };
        assert!(
            error.to_string().contains("rebinding"),
            "expected rebinding rejection, got {error}"
        );
    }

    #[test]
    fn final_validate_rejects_omitted_runtime_result() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        require_ok!(write_done_log(&fixture.run_dir), "DONE log");
        for member in prepared.members.iter().skip(1) {
            require_ok!(
                write_member_result(member, &prepared.run_id, "passed"),
                "runtime result"
            );
        }
        let error = require_err!(
            validate_run(&fixture.run_dir),
            "omitted result must fail final validate"
        );
        assert!(
            error.to_string().contains("runtime")
                || error.to_string().contains("missing")
                || error.to_string().contains("indexed"),
            "expected omitted-result rejection, got {error}"
        );
    }

    #[test]
    fn final_validate_rejects_rejected_result_when_taskfmt_exits_zero() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        require_ok!(write_done_log(&fixture.run_dir), "DONE log");
        for member in &prepared.members {
            let status = if member.operation == "close" {
                "rejected"
            } else {
                "passed"
            };
            require_ok!(
                write_member_result(member, &prepared.run_id, status),
                "runtime result"
            );
        }
        let error = require_err!(
            validate_run(&fixture.run_dir),
            "rejected close result must fail even with DONE"
        );
        assert!(
            error.to_string().contains("status")
                || error.to_string().contains("result")
                || error.to_string().contains("identity"),
            "expected rejected-result rejection, got {error}"
        );
    }

    #[test]
    fn final_validate_rejects_forged_runtime_outputs() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        require_ok!(write_complete_runtime_closure(&prepared), "closure");
        require_ok!(
            fs::write(fixture.run_dir.join("outputs/forged.json"), b"{}"),
            "forged output"
        );
        let error = require_err!(
            validate_run(&fixture.run_dir),
            "forged output must fail final validate"
        );
        assert!(
            error.to_string().contains("runtime")
                || error.to_string().contains("extra")
                || error.to_string().contains("indexed"),
            "expected forged-output rejection, got {error}"
        );
    }

    #[test]
    fn final_validate_accepts_complete_passed_closure() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        require_ok!(write_complete_runtime_closure(&prepared), "closure");
        require_ok!(validate_run(&fixture.run_dir), "complete closure");
        assert!(
            prepared
                .members
                .iter()
                .any(|member| member.operation == "close"),
            "TASK-060 fixture must include a close check"
        );
    }

    #[test]
    fn native_prepare_output_passes_validate_proof_preparation() {
        let (fixture, options) = require_ok!(fixture_options(), "fixture options");
        require_ok!(prepare(&options), "prepare");
        let scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts");
        let receipt = fixture.run_dir.join(PREPARATION_RECEIPT_FILE);
        let commit = require_ok!(
            git_output(&options.worktree, &["rev-parse", "HEAD"]),
            "candidate commit"
        );
        let tree = require_ok!(
            git_output(&options.worktree, &["rev-parse", "HEAD^{tree}"]),
            "candidate tree"
        );
        let output = require_ok!(
            Command::new("/usr/bin/python3")
                .current_dir(&scripts)
                .env("PYTHONPATH", &scripts)
                .args([
                    "-c",
                    "import json, sys\nfrom pathlib import Path\nfrom campaign_ledger import validate_proof_preparation\nprep = json.loads(Path(sys.argv[1]).read_text())\nvalidate_proof_preparation(prep, worktree=sys.argv[2], current_head=sys.argv[3], current_tree=sys.argv[4], run_dir=sys.argv[5], expected_taskfmt=prep['taskfmt'])\n",
                    receipt.to_str().unwrap_or(""),
                    options.worktree.to_str().unwrap_or(""),
                    &commit,
                    &tree,
                    fixture.run_dir.to_str().unwrap_or(""),
                ])
                .output(),
            "ledger validate"
        );
        assert!(
            output.status.success(),
            "validate_proof_preparation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn taskfmt_caller_argv_version_is_not_trusted() {
        let (fixture, mut options) = require_ok!(fixture_options(), "fixture options");
        options.taskfmt_version = "fixture".to_string();
        let error = require_err!(prepare(&options), "caller argv version must be rejected");
        assert!(
            error.to_string().contains("qualified") || error.to_string().contains("version"),
            "expected qualified-version rejection, got {error}"
        );
        let _ = fixture;
    }

    #[test]
    fn taskfmt_log_symlink_is_rejected_by_trust_path() {
        let (fixture, prepared) = require_ok!(fixture(), "fixture");
        require_ok!(write_complete_runtime_closure(&prepared), "closure");
        let log = fixture.run_dir.join("taskfmt-logs/verify.log");
        let alias = fixture.root.path().join("verify-alias.log");
        require_ok!(fs::copy(&log, &alias), "copy log");
        require_ok!(fs::remove_file(&log), "remove log");
        require_ok!(std::os::unix::fs::symlink(&alias, &log), "symlink log");
        let error = require_err!(
            validate_run(&fixture.run_dir),
            "symlinked taskfmt log must be rejected"
        );
        assert!(
            error.to_string().contains("symlink")
                || error.to_string().contains("unsafe")
                || error.to_string().contains("immutable"),
            "expected trust-path rejection, got {error}"
        );
    }

    #[test]
    fn comparator_report_hardlink_is_rejected_by_trust_path() {
        let (fixture, options) = require_ok!(
            fixture_options_for("060", &["TASK-059", "TASK-022"]),
            "fixture options"
        );
        let prepared = require_ok!(prepare(&options), "prepare");
        require_ok!(write_complete_runtime_closure(&prepared), "closure");
        let fallback = require_some!(prepared.members.first(), "first member")
            .result_path
            .clone();
        let report = prepared
            .members
            .iter()
            .find_map(|member| member.comparator_report_path.clone())
            .unwrap_or(fallback);
        let alias = fixture.root.path().join("report-hardlink.json");
        require_ok!(fs::hard_link(&report, &alias), "hardlink report");
        let error = require_err!(
            validate_run(&fixture.run_dir),
            "hardlinked comparator report must be rejected"
        );
        assert!(
            error.to_string().contains("immutable")
                || error.to_string().contains("hard")
                || error.to_string().contains("link")
                || error.to_string().contains("unsafe"),
            "expected trust-path rejection, got {error}"
        );
    }
}
