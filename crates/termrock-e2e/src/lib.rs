use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REGISTRY_JSON: &str = include_str!("../cases/registry.json");
pub const DEFERRED_JSON: &str = include_str!("../cases/deferred-obligations.json");
pub const PROFILE_JSON: &str = include_str!("../profile.json");
pub const RECEIPT_SCHEMA: &str = "termrock-spec/parity-run-receipt-v1";
pub const SUBJECT_SCHEMA: &str = "termrock-spec/parity-subject-manifest-v1";
pub const BUILDER_RECEIPT_SCHEMA: &str = "termrock-spec/parity-subject-build-receipt-v1";
pub const TRUST_RECORD_SCHEMA: &str = "termrock-spec/parity-trust-record-v1";
const TRUST_RECORD_PATH_ENV: &str = "TERMROCK_E2E_TRUST_RECORD";
const TRUST_RECORD_SHA_ENV: &str = "TERMROCK_E2E_TRUST_RECORD_SHA256";
const RECEIPT_PATH_ENV: &str = "TERMROCK_E2E_RECEIPT_PATH";
const WRITE_ROOT_ENV: &str = "TERMROCK_E2E_WRITE_ROOT";
const PROTECTED_ROOTS_ENV: &str = "TERMROCK_E2E_PROTECTED_ROOTS";
pub const WRITE_POLICY_SCHEMA: &str = "termrock-spec/parity-write-policy-v1";
pub const COMPILED_SUITE_SHA256: &str = env!("TERMROCK_E2E_COMPILED_SUITE_SHA256");

static CAPTURE_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize)]
pub struct Registry {
    pub schema: String,
    pub suite_revision: String,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Case {
    pub id: String,
    pub app: String,
    pub binary: String,
    pub args: Vec<String>,
    pub geometry: Geometry,
    pub color_path: String,
    pub legacy_snapshot_root: String,
    pub screen: String,
    pub substep: String,
    pub timeout_ms: u64,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Geometry {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Step {
    Press {
        key: String,
    },
    Checkpoint {
        id: String,
        wait: Vec<WaitCondition>,
        #[serde(default)]
        assertions: Vec<Assertion>,
        #[serde(default)]
        legacy_snapshot_path: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct WaitCondition {
    pub kind: String,
    pub needle: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Assertion {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub needle: Option<String>,
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
    #[serde(default)]
    pub requires: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectManifest {
    pub schema: String,
    pub run_id: String,
    pub suite_revision: String,
    pub suite_sha256: String,
    pub expected_generation: ExpectedGenerationInput,
    pub subjects: Vec<Subject>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedGenerationInput {
    pub id: String,
    #[serde(default)]
    pub root: Option<PathBuf>,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub role: String,
    pub source_commit: String,
    pub build: BuildFacts,
    pub executable: Executable,
    pub actual_output_root: PathBuf,
    pub build_inputs: BuildInputs,
    pub builder_receipt: BuilderReceipt,
    pub builder_receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildFacts {
    pub package_id: String,
    pub target_name: String,
    pub features: Vec<String>,
    pub default_features: bool,
    pub target_triple: String,
    pub toolchain: String,
    pub profile: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInputs {
    pub manifest_sha256: String,
    pub lock_sha256: String,
    pub mise_config: MiseConfigInput,
    pub cargo_version: String,
    pub rustc_version: String,
    pub host_triple: String,
    pub executed_argv: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MiseConfigInput {
    pub present: bool,
    pub path: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuilderReceipt {
    pub schema: String,
    pub source_commit: String,
    pub package_id: String,
    pub target_name: String,
    pub requested_features: Vec<String>,
    pub default_features: bool,
    pub target_triple: String,
    pub toolchain: String,
    pub profile: String,
    pub manifest_sha256: String,
    pub lock_sha256: String,
    pub mise_config: MiseConfigInput,
    pub cargo_version: String,
    pub rustc_version: String,
    pub host_triple: String,
    pub executed_argv: Vec<String>,
    pub executable_path: PathBuf,
    pub executable_sha256: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustRecord {
    pub schema: String,
    pub suite: TrustedSuiteInputs,
    pub subjects: Vec<TrustedSubjectInputs>,
    pub expected_generation: Option<TrustedExpectedGeneration>,
    pub write_policy_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedSuiteInputs {
    pub digest: String,
    pub case_set_digest: String,
    pub profile_digest: String,
    pub dependency_lock_sha256: String,
    pub review_receipt_sha256: String,
    pub test_binary_sha256: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedSubjectInputs {
    pub role: String,
    pub source_commit: String,
    pub builder_receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedExpectedGeneration {
    pub id: String,
    pub sha256: String,
    pub admission_receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Executable {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExecutableSnapshot {
    sha256: String,
    metadata: ExecutableMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExecutableMetadata {
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunReceipt {
    pub schema: String,
    pub run_id: String,
    pub source_pair: Vec<SubjectIdentity>,
    pub suite: SuiteIdentity,
    pub expected_generation: ExpectedGenerationReceipt,
    pub trust: TrustVerification,
    pub write_policy: WritePolicyReceipt,
    pub checks: Vec<CheckReceipt>,
    pub artifacts: Vec<ArtifactReceipt>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ProtectedRoot {
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActualOutputRoot {
    pub role: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WritePolicyReceipt {
    pub schema: String,
    pub write_root: PathBuf,
    pub receipt_path: PathBuf,
    pub protected_roots: Vec<ProtectedRoot>,
    pub actual_output_roots: Vec<ActualOutputRoot>,
    pub sha256: String,
}

#[derive(Clone, Debug)]
struct TrustRecordSource {
    canonical_path: PathBuf,
    expected_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WritePolicyInput {
    write_root: PathBuf,
    receipt_path: PathBuf,
    protected_roots: Vec<ProtectedRoot>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubjectIdentity {
    pub role: String,
    pub source_commit: String,
    pub build: BuildFacts,
    pub build_inputs: BuildInputs,
    pub builder_receipt: BuilderReceipt,
    pub builder_receipt_sha256: String,
    pub executable: ExecutableReceipt,
    pub actual_output_root: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutableReceipt {
    pub path: PathBuf,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SuiteIdentity {
    pub revision: String,
    pub digest: String,
    pub compiled_digest: String,
    pub case_set_digest: String,
    pub profile_digest: String,
    pub test_binary_digest: String,
    pub dependency_lock_sha256: String,
    pub platform: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExpectedGenerationReceipt {
    pub id: String,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
    pub state: String,
    pub admission_receipt_sha256: Option<String>,
    pub root: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrustVerification {
    pub status: String,
    pub record_path: PathBuf,
    pub record_sha256: String,
    pub receipt_sha256: String,
    pub write_policy_sha256: String,
    pub reason: String,
}

struct VerifiedTrustRecord {
    record: TrustRecord,
    verification: TrustVerification,
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckReceipt {
    pub id: String,
    pub subject_role: String,
    pub case_id: String,
    pub checkpoint_id: String,
    pub dimension: String,
    pub status: String,
    pub reason: String,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArtifactReceipt {
    pub subject_role: String,
    pub case_id: String,
    pub checkpoint_id: String,
    pub format: String,
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct Profile {
    readiness_timeout_ms: u64,
    input_pacing_ms: u64,
    terminal: TerminalProfile,
}

#[derive(Clone, Debug, Deserialize)]
struct TerminalProfile {
    term: String,
    colorterm: String,
    locale: String,
}

pub fn registry() -> Result<Registry, String> {
    serde_json::from_str(REGISTRY_JSON).map_err(|error| format!("parse case registry: {error}"))
}

pub fn deferred_row_count() -> Result<(usize, usize, String), String> {
    let value: serde_json::Value = serde_json::from_str(DEFERRED_JSON)
        .map_err(|error| format!("parse deferred cases: {error}"))?;
    let rows = value["rows"]
        .as_array()
        .ok_or_else(|| "deferred rows must be an array".to_string())?;
    let references = rows
        .iter()
        .map(|row| {
            row["cases"]
                .as_array()
                .map(Vec::len)
                .ok_or_else(|| "deferred row cases must be an array".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    let missing_id = value["missing_historical_id"]
        .as_str()
        .ok_or_else(|| "missing historical ID must be a string".to_string())?
        .to_string();
    Ok((rows.len(), references, missing_id))
}

pub fn suite_digest() -> Result<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    digest_tree(root)
}

fn verify_compiled_suite_digest(compiled: &str, current: &str) -> Result<(), String> {
    if compiled != current {
        return Err(format!(
            "STALE suite binary: embedded digest {compiled} does not match current suite files {current}"
        ));
    }
    Ok(())
}

pub fn digest_tree(root: &Path) -> Result<String, String> {
    let mut paths = Vec::new();
    collect_files(root, root, &mut paths)?;
    paths.sort();
    digest_paths(root, &paths)
}

fn collect_files(root: &Path, current: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("read directory {}: {error}", current.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read directory entry: {error}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("path escaped suite root: {error}"))?;
        if relative
            .components()
            .any(|component| component.as_os_str() == "target" || component.as_os_str() == ".git")
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("suite digest refuses symlink {}", path.display()));
        }
        if metadata.is_dir() {
            collect_files(root, &path, paths)?;
        } else if metadata.is_file() {
            paths.push(relative.to_path_buf());
        }
    }
    Ok(())
}

fn digest_paths(root: &Path, paths: &[PathBuf]) -> Result<String, String> {
    let mut digest = Sha256::new();
    for relative in paths {
        let contents = fs::read(root.join(relative))
            .map_err(|error| format!("read {}: {error}", root.join(relative).display()))?;
        digest.update((relative.as_os_str().len() as u64).to_be_bytes());
        digest.update(relative.to_string_lossy().as_bytes());
        digest.update((contents.len() as u64).to_be_bytes());
        digest.update(contents);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

pub fn load_subject_manifest() -> Result<SubjectManifest, String> {
    let path = std::env::var_os("TERMROCK_E2E_SUBJECT_MANIFEST")
        .map(PathBuf::from)
        .ok_or_else(|| "TERMROCK_E2E_SUBJECT_MANIFEST is required for a PTY run".to_string())?;
    let bytes = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let manifest: SubjectManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse subject manifest {}: {error}", path.display()))?;
    validate_subject_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_subject_manifest(manifest: &SubjectManifest) -> Result<(), String> {
    if manifest.schema != SUBJECT_SCHEMA {
        return Err(format!("unexpected subject schema {:?}", manifest.schema));
    }
    if manifest.run_id.trim().is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    let registry = registry()?;
    if manifest.suite_revision != registry.suite_revision {
        return Err(format!(
            "subject manifest suite revision {} does not match {}",
            manifest.suite_revision, registry.suite_revision
        ));
    }
    require_sha256("suite_sha256", &manifest.suite_sha256)?;
    if manifest.subjects.len() != 2 {
        return Err(format!(
            "subject manifest needs exactly reference and candidate, found {}",
            manifest.subjects.len()
        ));
    }
    let mut roles = BTreeSet::new();
    for subject in &manifest.subjects {
        if subject.role != "reference" && subject.role != "candidate" {
            return Err(format!("unsupported subject role {:?}", subject.role));
        }
        if !roles.insert(subject.role.as_str()) {
            return Err(format!("duplicate subject role {:?}", subject.role));
        }
        if subject.source_commit.len() != 40
            || !subject
                .source_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "{} source_commit must be a full 40-character SHA",
                subject.role
            ));
        }
        require_sha256(
            &format!("{} executable sha256", subject.role),
            &subject.executable.sha256,
        )?;
        if !subject.executable.path.is_absolute() {
            return Err(format!(
                "{} executable path must be absolute: {}",
                subject.role,
                subject.executable.path.display()
            ));
        }
        for fact in [
            subject.build.package_id.as_str(),
            subject.build.target_name.as_str(),
            subject.build.target_triple.as_str(),
            subject.build.toolchain.as_str(),
            subject.build.profile.as_str(),
        ] {
            if fact.trim().is_empty() {
                return Err(format!(
                    "{} build facts must not contain empty required fields",
                    subject.role
                ));
            }
        }
        validate_build_receipt(subject)?;
    }
    if !roles.contains("reference") || !roles.contains("candidate") {
        return Err("subject manifest must contain both reference and candidate roles".to_string());
    }
    validate_actual_output_roots(
        &manifest.subjects,
        manifest.expected_generation.root.as_deref(),
    )?;
    Ok(())
}

fn require_sha256(name: &str, digest: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a full SHA-256"));
    }
    Ok(())
}

fn validate_build_receipt(subject: &Subject) -> Result<(), String> {
    let receipt = &subject.builder_receipt;
    if receipt.schema != BUILDER_RECEIPT_SCHEMA {
        return Err(format!(
            "{} builder receipt schema is {:?}, expected {}",
            subject.role, receipt.schema, BUILDER_RECEIPT_SCHEMA
        ));
    }
    require_sha256(
        &format!("{} builder_receipt_sha256", subject.role),
        &subject.builder_receipt_sha256,
    )?;
    require_sha256(
        &format!("{} builder_receipt.sha256", subject.role),
        &receipt.sha256,
    )?;
    require_sha256(
        &format!("{} build_inputs.manifest_sha256", subject.role),
        &subject.build_inputs.manifest_sha256,
    )?;
    require_sha256(
        &format!("{} build_inputs.lock_sha256", subject.role),
        &subject.build_inputs.lock_sha256,
    )?;
    validate_mise_input(&subject.role, &subject.build_inputs.mise_config)?;
    if subject.build_inputs.cargo_version.trim().is_empty()
        || subject.build_inputs.rustc_version.trim().is_empty()
        || subject.build_inputs.host_triple.trim().is_empty()
        || subject.build_inputs.executed_argv.is_empty()
    {
        return Err(format!(
            "{} build_inputs must include Cargo/rustc/host and executed argv",
            subject.role
        ));
    }

    let mismatched = receipt.source_commit != subject.source_commit
        || receipt.package_id != subject.build.package_id
        || receipt.target_name != subject.build.target_name
        || receipt.requested_features != subject.build.features
        || receipt.default_features != subject.build.default_features
        || receipt.target_triple != subject.build.target_triple
        || receipt.toolchain != subject.build.toolchain
        || receipt.profile != subject.build.profile
        || receipt.manifest_sha256 != subject.build_inputs.manifest_sha256
        || receipt.lock_sha256 != subject.build_inputs.lock_sha256
        || receipt.mise_config != subject.build_inputs.mise_config
        || receipt.cargo_version != subject.build_inputs.cargo_version
        || receipt.rustc_version != subject.build_inputs.rustc_version
        || receipt.host_triple != subject.build_inputs.host_triple
        || receipt.executed_argv != subject.build_inputs.executed_argv
        || receipt.executable_path != subject.executable.path
        || receipt.executable_sha256 != subject.executable.sha256;
    if mismatched {
        return Err(format!(
            "{} builder receipt does not bind the supplied source/build/executable facts",
            subject.role
        ));
    }
    let canonical = builder_receipt_digest(receipt)?;
    if receipt.sha256 != canonical || subject.builder_receipt_sha256 != canonical {
        return Err(format!(
            "{} builder receipt digest mismatch: expected canonical {}, receipt {}, manifest {}",
            subject.role, canonical, receipt.sha256, subject.builder_receipt_sha256
        ));
    }
    Ok(())
}

fn validate_mise_input(role: &str, mise: &MiseConfigInput) -> Result<(), String> {
    match (mise.present, mise.path.as_deref(), mise.sha256.as_deref()) {
        (true, Some(path), Some(hash)) if !path.trim().is_empty() => {
            require_sha256(&format!("{role} build_inputs.mise_config.sha256"), hash)
        }
        (false, None, None) => Ok(()),
        _ => Err(format!(
            "{role} mise_config must have path and SHA-256 when present, and null path/hash when absent"
        )),
    }
}

pub fn builder_receipt_digest(receipt: &BuilderReceipt) -> Result<String, String> {
    let payload = serde_json::json!({
        "schema": receipt.schema,
        "source_commit": receipt.source_commit,
        "package_id": receipt.package_id,
        "target_name": receipt.target_name,
        "requested_features": receipt.requested_features,
        "default_features": receipt.default_features,
        "target_triple": receipt.target_triple,
        "toolchain": receipt.toolchain,
        "profile": receipt.profile,
        "manifest_sha256": receipt.manifest_sha256,
        "lock_sha256": receipt.lock_sha256,
        "mise_config": receipt.mise_config,
        "cargo_version": receipt.cargo_version,
        "rustc_version": receipt.rustc_version,
        "host_triple": receipt.host_triple,
        "executed_argv": receipt.executed_argv,
        "executable_path": receipt.executable_path,
        "executable_sha256": receipt.executable_sha256
    });
    let bytes = serde_json::to_vec(&payload)
        .map_err(|error| format!("serialize canonical builder receipt: {error}"))?;
    Ok(sha256_bytes(&bytes))
}

fn load_trust_record_source() -> Result<TrustRecordSource, String> {
    let path = std::env::var_os(TRUST_RECORD_PATH_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!(
                "{TRUST_RECORD_PATH_ENV} is required; source the trust record outside candidate-controlled files"
            )
        })?;
    let expected_sha256 = std::env::var(TRUST_RECORD_SHA_ENV).map_err(|_| {
        format!(
            "{TRUST_RECORD_SHA_ENV} is required; obtain it from the independent trust-record source"
        )
    })?;
    require_sha256(TRUST_RECORD_SHA_ENV, &expected_sha256)?;
    if !path.is_absolute() {
        return Err(format!("{TRUST_RECORD_PATH_ENV} must be an absolute path"));
    }
    let canonical_path = fs::canonicalize(&path).map_err(|error| {
        format!(
            "canonicalize external trust record {}: {error}",
            path.display()
        )
    })?;
    if !canonical_path.is_file() {
        return Err(format!(
            "external trust record is not a file: {}",
            canonical_path.display()
        ));
    }
    Ok(TrustRecordSource {
        canonical_path,
        expected_sha256,
    })
}

fn load_write_policy_input() -> Result<WritePolicyInput, String> {
    let write_root = std::env::var_os(WRITE_ROOT_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{WRITE_ROOT_ENV} is required for a PTY run"))?;
    let receipt_path = std::env::var_os(RECEIPT_PATH_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{RECEIPT_PATH_ENV} is required to persist a run receipt"))?;
    let protected_roots_json = std::env::var(PROTECTED_ROOTS_ENV).map_err(|_| {
        format!(
            "{PROTECTED_ROOTS_ENV} is required and must list subject sources, subject artifacts, and oracle roots"
        )
    })?;
    let protected_roots: Vec<ProtectedRoot> = serde_json::from_str(&protected_roots_json)
        .map_err(|error| format!("parse {PROTECTED_ROOTS_ENV}: {error}"))?;
    Ok(WritePolicyInput {
        write_root,
        receipt_path,
        protected_roots,
    })
}

fn resolve_write_policy(
    input: WritePolicyInput,
    subjects: &[Subject],
    expected_root: Option<&Path>,
    trust_source: &TrustRecordSource,
    test_binary: &Path,
) -> Result<WritePolicyReceipt, String> {
    let write_root = canonical_existing_directory(&input.write_root, WRITE_ROOT_ENV)?;
    let receipt_path = canonical_new_file_path(&input.receipt_path, "run receipt")?;
    let mut protected_roots = canonical_caller_protected_roots(input.protected_roots)?;

    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checkout_root = suite_checkout_root(package_root)?;
    protected_roots.push(protected_root("suite_checkout", None, checkout_root)?);

    if let Some(expected_root) = expected_root {
        protected_roots.push(protected_root(
            "expected_generation",
            None,
            canonical_output_root(expected_root)?,
        )?);
    }
    protected_roots.push(protected_root(
        "trust_record",
        None,
        trust_source.canonical_path.clone(),
    )?);
    if let Some(parent) = trust_source.canonical_path.parent() {
        protected_roots.push(protected_root(
            "trust_record_parent",
            None,
            fs::canonicalize(parent).map_err(|error| {
                format!(
                    "canonicalize trust record parent {}: {error}",
                    parent.display()
                )
            })?,
        )?);
    }
    let canonical_test_binary = fs::canonicalize(test_binary).map_err(|error| {
        format!(
            "canonicalize test binary {}: {error}",
            test_binary.display()
        )
    })?;
    let test_binary_parent = canonical_test_binary
        .parent()
        .ok_or_else(|| "test binary has no parent directory".to_string())?;
    protected_roots.push(protected_root(
        "test_binary_parent",
        None,
        fs::canonicalize(test_binary_parent).map_err(|error| {
            format!(
                "canonicalize test binary parent {}: {error}",
                test_binary_parent.display()
            )
        })?,
    )?);

    let mut actual_output_roots = Vec::with_capacity(subjects.len());
    for subject in subjects {
        let output_root = canonical_output_root(&subject.actual_output_root)?;
        actual_output_roots.push(ActualOutputRoot {
            role: subject.role.clone(),
            path: output_root,
        });

        let executable_parent = subject
            .executable
            .path
            .parent()
            .ok_or_else(|| format!("{} executable has no parent directory", subject.role))?;
        let canonical_executable_parent = canonical_output_root(executable_parent)?;
        let canonical_executable = fs::canonicalize(&subject.executable.path).map_err(|error| {
            format!(
                "canonicalize {} executable {}: {error}",
                subject.role,
                subject.executable.path.display()
            )
        })?;
        let artifact_root = protected_roots
            .iter()
            .find(|root| {
                root.kind == "subject_artifact" && root.role.as_deref() == Some(&subject.role)
            })
            .ok_or_else(|| {
                format!(
                    "{PROTECTED_ROOTS_ENV} must include subject_artifact for {}",
                    subject.role
                )
            })?;
        if !canonical_executable_parent.starts_with(&artifact_root.path)
            || !canonical_executable.starts_with(&artifact_root.path)
        {
            return Err(format!(
                "{} executable directory {} is outside its protected artifact root {}",
                subject.role,
                canonical_executable_parent.display(),
                artifact_root.path.display()
            ));
        }
        protected_roots.push(protected_root(
            "subject_executable_parent",
            Some(subject.role.clone()),
            canonical_executable_parent,
        )?);
    }
    actual_output_roots.sort_by(|left, right| left.role.cmp(&right.role));
    protected_roots.sort();
    protected_roots.dedup();

    for root in &protected_roots {
        if matches!(root.kind.as_str(), "trust_record" | "trust_record_parent") {
            continue;
        }
        if paths_overlap(&trust_source.canonical_path, &root.path) {
            return Err(format!(
                "external trust record must be outside protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }
    for output in &actual_output_roots {
        if paths_overlap(&trust_source.canonical_path, &output.path) {
            return Err(format!(
                "external trust record must be outside {} actual output root {}",
                output.role,
                output.path.display()
            ));
        }
    }

    if protected_roots.is_empty() {
        return Err("write policy has no protected roots".to_string());
    }
    for root in &protected_roots {
        if paths_overlap(&write_root, &root.path) {
            return Err(format!(
                "write root overlaps protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }

    if receipt_path == write_root || !receipt_path.starts_with(&write_root) {
        return Err(format!(
            "run receipt destination must be strictly beneath {WRITE_ROOT_ENV}: {}",
            receipt_path.display()
        ));
    }
    if path_exists_no_follow(&receipt_path)? {
        return Err(format!(
            "run receipt destination already exists; refusing to overwrite {}",
            receipt_path.display()
        ));
    }
    for root in &protected_roots {
        if paths_overlap(&receipt_path, &root.path) {
            return Err(format!(
                "run receipt destination overlaps protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }

    for output in &actual_output_roots {
        if output.path == write_root || !output.path.starts_with(&write_root) {
            return Err(format!(
                "{} actual output root must be strictly beneath {WRITE_ROOT_ENV}: {}",
                output.role,
                output.path.display()
            ));
        }
        for root in &protected_roots {
            if paths_overlap(&output.path, &root.path) {
                return Err(format!(
                    "{} actual output root overlaps protected {} root {}",
                    output.role,
                    root.kind,
                    root.path.display()
                ));
            }
        }
        if paths_overlap(&receipt_path, &output.path) {
            return Err(format!(
                "run receipt destination overlaps {} actual output root {}",
                output.role,
                output.path.display()
            ));
        }
    }
    for left in 0..actual_output_roots.len() {
        for right in (left + 1)..actual_output_roots.len() {
            if paths_overlap(
                &actual_output_roots[left].path,
                &actual_output_roots[right].path,
            ) {
                return Err(format!(
                    "actual output roots overlap: {}={} and {}={}",
                    actual_output_roots[left].role,
                    actual_output_roots[left].path.display(),
                    actual_output_roots[right].role,
                    actual_output_roots[right].path.display()
                ));
            }
        }
    }

    let mut policy = WritePolicyReceipt {
        schema: WRITE_POLICY_SCHEMA.to_string(),
        write_root,
        receipt_path,
        protected_roots,
        actual_output_roots,
        sha256: String::new(),
    };
    policy.sha256 = write_policy_digest(&policy)?;
    Ok(policy)
}

fn canonical_caller_protected_roots(
    mut roots: Vec<ProtectedRoot>,
) -> Result<Vec<ProtectedRoot>, String> {
    let mut seen = BTreeSet::new();
    let mut coverage = BTreeSet::new();
    let mut has_oracle = false;
    for root in &mut roots {
        match root.kind.as_str() {
            "subject_source" | "subject_artifact" => {
                let role = root.role.as_deref().ok_or_else(|| {
                    format!("protected {} root must name a subject role", root.kind)
                })?;
                if role != "reference" && role != "candidate" {
                    return Err(format!("protected root has unknown subject role {role:?}"));
                }
                coverage.insert((root.kind.as_str().to_string(), role.to_string()));
            }
            "oracle" => {
                if root.role.is_some() {
                    return Err("oracle protected roots must not name a subject role".to_string());
                }
                has_oracle = true;
            }
            other => {
                return Err(format!(
                    "caller protected root has unsupported kind {other:?}"
                ));
            }
        }
        root.path = canonical_existing_directory(&root.path, "protected root")?;
        if !seen.insert(root.clone()) {
            return Err(format!(
                "duplicate caller protected root {}",
                root.path.display()
            ));
        }
    }
    for role in ["reference", "candidate"] {
        for kind in ["subject_source", "subject_artifact"] {
            if !coverage.contains(&(kind.to_string(), role.to_string())) {
                return Err(format!(
                    "{PROTECTED_ROOTS_ENV} must include {kind} for {role}"
                ));
            }
        }
    }
    if !has_oracle {
        return Err(format!("{PROTECTED_ROOTS_ENV} must include an oracle root"));
    }
    roots.sort();
    Ok(roots)
}

fn canonical_existing_directory(path: &Path, name: &str) -> Result<PathBuf, String> {
    validate_absolute_normal_path(path, name)?;
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("canonicalize {name} {}: {error}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "{name} is not a directory: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn canonical_new_file_path(path: &Path, name: &str) -> Result<PathBuf, String> {
    validate_absolute_normal_path(path, name)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("{name} path has no filename: {}", path.display()))?;
    if !matches!(
        Path::new(file_name).components().next(),
        Some(Component::Normal(_))
    ) {
        return Err(format!("{name} filename must be a normal path component"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{name} path has no parent: {}", path.display()))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| format!("canonicalize {name} parent {}: {error}", parent.display()))?;
    if !canonical_parent.is_dir() {
        return Err(format!(
            "{name} parent is not a directory: {}",
            canonical_parent.display()
        ));
    }
    Ok(canonical_parent.join(file_name))
}

fn validate_absolute_normal_path(path: &Path, name: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("{name} path must be absolute: {}", path.display()));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!(
            "{name} path must not contain . or .. components: {}",
            path.display()
        ));
    }
    Ok(())
}

fn suite_checkout_root(package_root: &Path) -> Result<PathBuf, String> {
    let checkout_root = package_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "cannot locate suite checkout root".to_string())?;
    fs::canonicalize(checkout_root).map_err(|error| {
        format!(
            "canonicalize suite checkout root {}: {error}",
            checkout_root.display()
        )
    })
}

fn protected_root(
    kind: &str,
    role: Option<String>,
    path: PathBuf,
) -> Result<ProtectedRoot, String> {
    Ok(ProtectedRoot {
        kind: kind.to_string(),
        role,
        path: fs::canonicalize(&path)
            .or_else(|_| canonical_output_root(&path))
            .map_err(|error| {
                format!(
                    "canonicalize protected {kind} root {}: {error}",
                    path.display()
                )
            })?,
    })
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn path_exists_no_follow(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("inspect destination {}: {error}", path.display())),
    }
}

fn write_policy_digest(policy: &WritePolicyReceipt) -> Result<String, String> {
    let payload = serde_json::json!({
        "schema": policy.schema,
        "write_root": policy.write_root,
        "receipt_path": policy.receipt_path,
        "protected_roots": policy.protected_roots,
        "actual_output_roots": policy.actual_output_roots
    });
    let bytes = serde_json::to_vec(&payload)
        .map_err(|error| format!("serialize canonical write policy: {error}"))?;
    Ok(sha256_bytes(&bytes))
}

fn load_and_verify_trust_record(
    manifest: &SubjectManifest,
    subjects: &[Subject],
    expected_actual: &Result<String, String>,
    trust_source: &TrustRecordSource,
    write_policy_sha256: &str,
    suite_digest: &str,
    case_set_digest: &str,
    profile_digest: &str,
    dependency_lock_digest: &str,
    test_binary_digest: &str,
) -> Result<VerifiedTrustRecord, String> {
    let canonical_record_path = &trust_source.canonical_path;
    let expected_record_sha = &trust_source.expected_sha256;
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let canonical_checkout_root = suite_checkout_root(package_root)?;
    if canonical_record_path.starts_with(&canonical_checkout_root) {
        return Err(format!(
            "external trust record must be outside the suite checkout: {}",
            canonical_record_path.display()
        ));
    }
    for subject in subjects {
        let output_root = canonical_output_root(&subject.actual_output_root)?;
        if canonical_record_path.starts_with(output_root) {
            return Err(format!(
                "external trust record must be outside actual output roots: {}",
                canonical_record_path.display()
            ));
        }
    }

    let bytes = fs::read(&canonical_record_path).map_err(|error| {
        format!(
            "read external trust record {}: {error}",
            canonical_record_path.display()
        )
    })?;
    let record_sha256 = sha256_bytes(&bytes);
    verify_external_record_digest(&expected_record_sha, &record_sha256)?;
    let record: TrustRecord = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse external trust record: {error}"))?;
    let receipt_sha256 = validate_trust_record(
        &record,
        manifest,
        subjects,
        expected_actual,
        write_policy_sha256,
        suite_digest,
        case_set_digest,
        profile_digest,
        dependency_lock_digest,
        test_binary_digest,
    )?;
    Ok(VerifiedTrustRecord {
        record,
        verification: TrustVerification {
            status: "accepted".to_string(),
            record_path: canonical_record_path.clone(),
            record_sha256,
            receipt_sha256,
            write_policy_sha256: write_policy_sha256.to_string(),
            reason: "external record bytes matched the caller-pinned digest and all suite, subject, and binary identities".to_string(),
        },
    })
}

fn verify_external_record_digest(expected: &str, actual: &str) -> Result<(), String> {
    require_sha256("external trust record expected SHA-256", expected)?;
    if actual != expected {
        return Err(format!(
            "external trust record digest mismatch: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn validate_trust_record(
    record: &TrustRecord,
    manifest: &SubjectManifest,
    subjects: &[Subject],
    expected_actual: &Result<String, String>,
    write_policy_sha256: &str,
    suite_digest: &str,
    case_set_digest: &str,
    profile_digest: &str,
    dependency_lock_digest: &str,
    test_binary_digest: &str,
) -> Result<String, String> {
    if record.schema != TRUST_RECORD_SCHEMA {
        return Err(format!(
            "external trust record schema {:?} is not {}",
            record.schema, TRUST_RECORD_SCHEMA
        ));
    }
    require_sha256(
        "trust record write_policy_sha256",
        &record.write_policy_sha256,
    )?;
    if record.write_policy_sha256 != write_policy_sha256 {
        return Err(
            "trust record write policy digest does not match the resolved write policy".to_string(),
        );
    }
    for (name, digest) in [
        ("suite.digest", record.suite.digest.as_str()),
        (
            "suite.case_set_digest",
            record.suite.case_set_digest.as_str(),
        ),
        ("suite.profile_digest", record.suite.profile_digest.as_str()),
        (
            "suite.dependency_lock_sha256",
            record.suite.dependency_lock_sha256.as_str(),
        ),
        (
            "suite.review_receipt_sha256",
            record.suite.review_receipt_sha256.as_str(),
        ),
    ] {
        require_sha256(name, digest)?;
    }
    if record.suite.digest != suite_digest || manifest.suite_sha256 != suite_digest {
        return Err("trust record suite digest does not match compiled/current suite".to_string());
    }
    if record.suite.case_set_digest != case_set_digest
        || record.suite.profile_digest != profile_digest
        || record.suite.dependency_lock_sha256 != dependency_lock_digest
    {
        return Err("trust record case/profile/lock digest does not match suite files".to_string());
    }
    if !record
        .suite
        .test_binary_sha256
        .iter()
        .any(|digest| digest == test_binary_digest)
    {
        return Err(format!(
            "current test binary digest {test_binary_digest} is not in the accepted trust record"
        ));
    }
    for digest in &record.suite.test_binary_sha256 {
        require_sha256("suite.test_binary_sha256[]", digest)?;
    }
    if record.subjects.len() != 2 {
        return Err("trust record must bind exactly reference and candidate subjects".to_string());
    }
    let mut admitted = BTreeSet::new();
    let mut pair = record.subjects.clone();
    pair.sort_by(|left, right| left.role.cmp(&right.role));
    for entry in &pair {
        if entry.role != "reference" && entry.role != "candidate" {
            return Err(format!(
                "trust record has unknown subject role {:?}",
                entry.role
            ));
        }
        if !admitted.insert(entry.role.as_str()) {
            return Err(format!(
                "trust record repeats subject role {:?}",
                entry.role
            ));
        }
        require_sha256(
            &format!("trust record {} builder_receipt_sha256", entry.role),
            &entry.builder_receipt_sha256,
        )?;
        let subject = subjects
            .iter()
            .find(|subject| subject.role == entry.role)
            .ok_or_else(|| {
                format!(
                    "trust record subject {} is absent from manifest",
                    entry.role
                )
            })?;
        if entry.source_commit != subject.source_commit
            || entry.builder_receipt_sha256 != subject.builder_receipt_sha256
        {
            return Err(format!(
                "trust record does not bind {} source commit and builder receipt",
                entry.role
            ));
        }
    }
    if !admitted.contains("reference") || !admitted.contains("candidate") {
        return Err("trust record must include both reference and candidate".to_string());
    }
    let receipt_pair = pair
        .iter()
        .map(|entry| {
            serde_json::json!({
                "role": entry.role,
                "builder_receipt_sha256": entry.builder_receipt_sha256
            })
        })
        .collect::<Vec<_>>();
    let receipt_bytes = serde_json::to_vec(&receipt_pair)
        .map_err(|error| format!("serialize admitted builder receipt pair: {error}"))?;
    let receipt_sha256 = sha256_bytes(&receipt_bytes);

    if let Some(admission) = &record.expected_generation {
        require_sha256("trust record expected_generation.sha256", &admission.sha256)?;
        require_sha256(
            "trust record expected_generation.admission_receipt_sha256",
            &admission.admission_receipt_sha256,
        )?;
        if admission.id != manifest.expected_generation.id
            || admission.sha256 != manifest.expected_generation.sha256
            || expected_actual.as_ref().ok() != Some(&admission.sha256)
        {
            return Err(
                "trust record expected-generation admission does not match verified tree"
                    .to_string(),
            );
        }
    }
    Ok(receipt_sha256)
}

fn validate_actual_output_roots(
    subjects: &[Subject],
    expected_root: Option<&Path>,
) -> Result<(), String> {
    let mut roots = Vec::with_capacity(subjects.len() + usize::from(expected_root.is_some()));
    for subject in subjects {
        roots.push((
            subject.role.as_str(),
            canonical_output_root(&subject.actual_output_root)?,
        ));
    }
    if let Some(expected_root) = expected_root {
        roots.push(("expected-generation", canonical_output_root(expected_root)?));
    }
    for left in 0..roots.len() {
        for right in (left + 1)..roots.len() {
            let (left_name, left_root) = &roots[left];
            let (right_name, right_root) = &roots[right];
            if left_root == right_root
                || left_root.starts_with(right_root)
                || right_root.starts_with(left_root)
            {
                return Err(format!(
                    "output roots overlap: {left_name}={} and {right_name}={}",
                    left_root.display(),
                    right_root.display()
                ));
            }
        }
    }
    Ok(())
}

fn canonical_output_root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "actual output root must be absolute: {}",
            path.display()
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!(
            "actual output root must not contain . or .. components: {}",
            path.display()
        ));
    }

    let mut ancestor = path;
    let mut missing = Vec::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            format!(
                "actual output root has no existing ancestor: {}",
                path.display()
            )
        })?;
        missing.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            format!(
                "actual output root has no existing ancestor: {}",
                path.display()
            )
        })?;
    }
    if !ancestor.is_dir() {
        return Err(format!(
            "actual output root ancestor is not a directory: {}",
            ancestor.display()
        ));
    }
    let mut canonical = fs::canonicalize(ancestor).map_err(|error| {
        format!(
            "canonicalize actual output root ancestor {}: {error}",
            ancestor.display()
        )
    })?;
    for name in missing.iter().rev() {
        canonical.push(name);
    }
    Ok(canonical)
}

pub fn validate_case_contract(case: &Case) -> Result<(), String> {
    if case.steps.is_empty() {
        return Err(format!("case {} has no steps", case.id));
    }
    let mut assertion_ids = BTreeSet::new();
    let mut positive_ids = BTreeSet::new();
    for step in &case.steps {
        let Step::Checkpoint {
            id,
            wait,
            assertions,
            ..
        } = step
        else {
            continue;
        };
        if wait.is_empty() {
            return Err(format!(
                "case {} checkpoint {id} has no readiness predicates",
                case.id
            ));
        }
        let mut has_positive_readiness = false;
        for condition in wait {
            if condition.needle.trim().is_empty() {
                return Err(format!(
                    "case {} checkpoint {id} has an empty readiness needle",
                    case.id
                ));
            }
            match condition.kind.as_str() {
                "contains" => has_positive_readiness = true,
                "absent" => {}
                other => {
                    return Err(format!(
                        "case {} checkpoint {id} has unsupported readiness kind {other:?}",
                        case.id
                    ));
                }
            }
        }
        if !has_positive_readiness {
            return Err(format!(
                "case {} checkpoint {id} needs at least one positive contains readiness predicate",
                case.id
            ));
        }
        if assertions.is_empty() {
            return Err(format!(
                "case {} checkpoint {id} has no assertions",
                case.id
            ));
        }
        for assertion in assertions {
            if assertion.id.is_empty() || assertion_ids.contains(&assertion.id) {
                return Err(format!(
                    "case {} has an empty or duplicate assertion ID {:?}",
                    case.id, assertion.id
                ));
            }
            match assertion.kind.as_str() {
                "contains" => {
                    if !assertion
                        .needle
                        .as_deref()
                        .is_some_and(|needle| !needle.is_empty())
                        || assertion.left.is_some()
                        || assertion.right.is_some()
                        || assertion.requires.is_some()
                    {
                        return Err(format!(
                            "case {} assertion {} has invalid contains fields",
                            case.id, assertion.id
                        ));
                    }
                }
                "absent" => {
                    if !assertion
                        .needle
                        .as_deref()
                        .is_some_and(|needle| !needle.is_empty())
                        || assertion.left.is_some()
                        || assertion.right.is_some()
                    {
                        return Err(format!(
                            "case {} assertion {} has invalid absent fields",
                            case.id, assertion.id
                        ));
                    }
                    let Some(required) = &assertion.requires else {
                        return Err(format!(
                            "case {} negative assertion {} lacks a positive precondition",
                            case.id, assertion.id
                        ));
                    };
                    if !positive_ids.contains(required) {
                        return Err(format!(
                            "case {} negative assertion {} requires earlier positive assertion {required}",
                            case.id, assertion.id
                        ));
                    }
                }
                "same_line" => {
                    if !assertion
                        .left
                        .as_deref()
                        .is_some_and(|left| !left.is_empty())
                        || !assertion
                            .right
                            .as_deref()
                            .is_some_and(|right| !right.is_empty())
                        || assertion.needle.is_some()
                        || assertion.requires.is_some()
                    {
                        return Err(format!(
                            "case {} assertion {} has invalid same_line fields",
                            case.id, assertion.id
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "case {} assertion {} has unsupported kind {other:?}",
                        case.id, assertion.id
                    ));
                }
            }
            if matches!(assertion.kind.as_str(), "contains" | "same_line") {
                positive_ids.insert(assertion.id.clone());
            }
            assertion_ids.insert(assertion.id.clone());
        }
    }
    Ok(())
}

pub fn run_case(case_id: &str) -> Result<RunReceipt, String> {
    let current_suite_digest = suite_digest()?;
    verify_compiled_suite_digest(COMPILED_SUITE_SHA256, &current_suite_digest)?;
    let manifest = load_subject_manifest()?;
    if manifest.suite_sha256 != current_suite_digest {
        return Err(format!(
            "STALE suite manifest digest {} does not match compiled/current suite {}",
            manifest.suite_sha256, current_suite_digest
        ));
    }
    let registry = registry()?;
    let case = registry
        .cases
        .iter()
        .find(|case| case.id == case_id)
        .ok_or_else(|| format!("unknown case ID {case_id}"))?;
    validate_case_contract(case)?;
    if manifest
        .subjects
        .iter()
        .any(|subject| subject.build.target_name != case.binary)
    {
        return Err(format!(
            "case {case_id} requires binary {}, but subject build target does not match",
            case.binary
        ));
    }

    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_digest = digest_tree(&package_root.join("cases"))?;
    let profile_digest = sha256_file(&package_root.join("profile.json"))?;
    let dependency_lock_digest = sha256_file(&package_root.join("Cargo.lock"))?;
    let binary_path =
        std::env::current_exe().map_err(|error| format!("locate test binary: {error}"))?;
    let test_binary_digest = sha256_file(&binary_path)?;
    let expected_actual = validate_expected_generation(&manifest.expected_generation);
    let mut subjects = manifest.subjects.clone();
    subjects.sort_by(|left, right| left.role.cmp(&right.role));
    let trust_source = load_trust_record_source()?;
    let write_policy_input = load_write_policy_input()?;
    let write_policy = resolve_write_policy(
        write_policy_input,
        &subjects,
        manifest.expected_generation.root.as_deref(),
        &trust_source,
        &binary_path,
    )?;
    let trusted = load_and_verify_trust_record(
        &manifest,
        &subjects,
        &expected_actual,
        &trust_source,
        &write_policy.sha256,
        &current_suite_digest,
        &case_digest,
        &profile_digest,
        &dependency_lock_digest,
        &test_binary_digest,
    )?;
    let generation_admission = trusted.record.expected_generation.as_ref();
    let verified_executables = subjects
        .iter()
        .map(verify_subject_executable)
        .collect::<Vec<_>>();
    let subject_pair_verified = verified_executables.iter().all(Result::is_ok);

    let mut receipt = RunReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        source_pair: subjects
            .iter()
            .map(|subject| SubjectIdentity {
                role: subject.role.clone(),
                source_commit: subject.source_commit.clone(),
                build: subject.build.clone(),
                build_inputs: subject.build_inputs.clone(),
                builder_receipt: subject.builder_receipt.clone(),
                builder_receipt_sha256: subject.builder_receipt_sha256.clone(),
                executable: ExecutableReceipt {
                    path: subject.executable.path.clone(),
                    expected_sha256: subject.executable.sha256.clone(),
                    actual_sha256: None,
                },
                actual_output_root: subject.actual_output_root.clone(),
            })
            .collect(),
        suite: SuiteIdentity {
            revision: registry.suite_revision,
            digest: current_suite_digest,
            compiled_digest: COMPILED_SUITE_SHA256.to_string(),
            case_set_digest: case_digest,
            profile_digest,
            test_binary_digest,
            dependency_lock_sha256: dependency_lock_digest,
            platform: std::env::consts::OS.to_string(),
        },
        expected_generation: ExpectedGenerationReceipt {
            id: manifest.expected_generation.id.clone(),
            expected_sha256: manifest.expected_generation.sha256.clone(),
            actual_sha256: expected_actual.as_ref().ok().cloned(),
            state: expected_generation_state(&expected_actual, generation_admission.is_some())
                .to_string(),
            admission_receipt_sha256: generation_admission
                .map(|admission| admission.admission_receipt_sha256.clone()),
            root: manifest.expected_generation.root.clone(),
        },
        trust: trusted.verification,
        write_policy: write_policy.clone(),
        checks: Vec::new(),
        artifacts: Vec::new(),
    };

    let mut paired_spawn_block: Option<String> = None;
    for (subject_index, subject) in subjects.iter().enumerate() {
        let verified_digest = &verified_executables[subject_index];
        receipt.source_pair[subject_index].executable.actual_sha256 = verified_digest
            .as_ref()
            .ok()
            .map(|snapshot| snapshot.sha256.clone());
        let build_status = if verified_digest.is_ok() {
            "PASS"
        } else {
            "FAIL"
        };
        let build_reason = verified_digest
            .as_ref()
            .map(|_| "executable bytes match the supplied build identity")
            .unwrap_or_else(|error| error.as_str());
        if !subject_pair_verified {
            for checkpoint in checkpoints(case) {
                append_not_run_checkpoint(
                    &mut receipt.checks,
                    &subject.role,
                    &case.id,
                    checkpoint,
                    build_status,
                    build_reason,
                    "NOT_RUN",
                    "paired launch blocked because one or more executable hashes did not match",
                );
            }
            continue;
        }

        if let Some(reason) = &paired_spawn_block {
            for checkpoint in checkpoints(case) {
                append_not_run_checkpoint(
                    &mut receipt.checks,
                    &subject.role,
                    &case.id,
                    checkpoint,
                    build_status,
                    build_reason,
                    "NOT_RUN",
                    &format!("paired launch stopped after executable integrity failure: {reason}"),
                );
            }
            continue;
        }

        let initial_snapshot = verified_digest
            .as_ref()
            .expect("paired executable preflight succeeded");
        let pre_spawn = inspect_executable(&subject.executable.path);
        receipt.source_pair[subject_index].executable.actual_sha256 = pre_spawn
            .as_ref()
            .ok()
            .map(|snapshot| snapshot.sha256.clone());
        let pre_spawn_failure = match &pre_spawn {
            Err(error) => Some(error.clone()),
            Ok(snapshot) => validate_executable_snapshot(subject, initial_snapshot, snapshot).err(),
        };
        if let Some(reason) = pre_spawn_failure {
            for checkpoint in checkpoints(case) {
                append_not_run_checkpoint(
                    &mut receipt.checks,
                    &subject.role,
                    &case.id,
                    checkpoint,
                    "STALE",
                    "executable hash or metadata changed after paired preflight",
                    "NOT_RUN",
                    &format!("pre-spawn executable verification failed: {reason}"),
                );
            }
            paired_spawn_block = Some(format!("{}: {reason}", subject.role));
            continue;
        }

        let profile: Profile = serde_json::from_str(PROFILE_JSON)
            .map_err(|error| format!("parse suite profile: {error}"))?;
        let args = make_argv(&subject.executable.path, &case.args);
        let launch_result = launch(&args, case, &profile);
        let mut session = match launch_result {
            Ok(session) => {
                for checkpoint in checkpoints(case) {
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "build",
                        "PASS",
                        "verified executable hash",
                        vec![initial_snapshot.sha256.clone()],
                    );
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "launch",
                        "PASS",
                        "real executable launched in a Tuiscotti PTY",
                        vec![
                            subject.executable.path.display().to_string(),
                            initial_snapshot.sha256.clone(),
                        ],
                    );
                }
                session
            }
            Err(error) => {
                for checkpoint in checkpoints(case) {
                    append_not_run_checkpoint(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "PASS",
                        "executable digest matched the externally admitted builder receipt",
                        "ERROR",
                        &format!("launch failed: {error}"),
                    );
                }
                continue;
            }
        };

        let mut passed_assertions = BTreeSet::new();
        for step in &case.steps {
            match step {
                Step::Press { key } => {
                    if let Err(error) = session.press(key) {
                        receipt.checks.push(check(
                            &subject.role,
                            &case.id,
                            "input",
                            "interaction",
                            "ERROR",
                            format!("press {key} failed: {error}"),
                            Vec::new(),
                        ));
                    } else {
                        std::thread::sleep(Duration::from_millis(profile.input_pacing_ms));
                    }
                }
                Step::Checkpoint {
                    id,
                    wait,
                    assertions,
                    legacy_snapshot_path,
                } => {
                    let checkpoint = id.as_str();
                    let observation = match wait_for_observation(
                        &mut session,
                        wait,
                        case.timeout_ms.max(profile.readiness_timeout_ms),
                    ) {
                        Ok(observation) => observation,
                        Err(error) => {
                            append_readiness_failure(
                                &mut receipt.checks,
                                &subject.role,
                                &case.id,
                                checkpoint,
                                &error,
                            );
                            continue;
                        }
                    };
                    let frame =
                        tuiscotti::render::frame_from_screen(&observation.screen, "default");
                    let screen_text = frame.text();
                    let mut assertion_statuses = Vec::new();
                    for assertion in assertions {
                        let result =
                            evaluate_assertion(assertion, &screen_text, &passed_assertions);
                        if result.0 == "PASS" {
                            passed_assertions.insert(assertion.id.clone());
                        }
                        let check_id = format!("{}:{}:{}", case.id, checkpoint, assertion.id);
                        receipt.checks.push(CheckReceipt {
                            id: check_id,
                            subject_role: subject.role.clone(),
                            case_id: case.id.clone(),
                            checkpoint_id: checkpoint.to_string(),
                            dimension: "interaction".to_string(),
                            status: result.0.to_string(),
                            reason: result.1.clone(),
                            evidence: result.2.clone(),
                        });
                        assertion_statuses.push(result.0.to_string());
                    }
                    let interaction_status =
                        if assertion_statuses.iter().all(|status| status == "PASS") {
                            "PASS"
                        } else if assertion_statuses.iter().any(|status| status == "FAIL") {
                            "FAIL"
                        } else {
                            "BLOCKED"
                        };
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "first_frame",
                        "PASS",
                        "readiness predicates held on one captured PTY observation",
                        vec![format!("{}x{}", case.geometry.cols, case.geometry.rows)],
                    );
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "interaction",
                        interaction_status,
                        "checkpoint interaction assertions",
                        vec![screen_text.clone()],
                    );

                    let actual_output_root = write_policy
                        .actual_output_roots
                        .iter()
                        .find(|output| output.role == subject.role)
                        .expect("resolved write policy has each subject output root");
                    match write_capture(
                        subject,
                        &write_policy.write_root,
                        &actual_output_root.path,
                        case,
                        checkpoint,
                        &frame,
                        legacy_snapshot_path.as_deref(),
                    ) {
                        Ok(artifacts) => {
                            receipt.artifacts.extend(artifacts);
                            let expected_reason = expected_visual_reason(
                                &manifest.expected_generation,
                                &expected_actual,
                                generation_admission.is_some(),
                                legacy_snapshot_path.as_deref(),
                            );
                            push_check(
                                &mut receipt.checks,
                                &subject.role,
                                &case.id,
                                checkpoint,
                                "visual",
                                "BLOCKED",
                                &expected_reason,
                                vec![
                                    legacy_snapshot_path.clone().unwrap_or_else(|| {
                                        "no legacy approved snapshot".to_string()
                                    }),
                                ],
                            );
                        }
                        Err(error) => push_check(
                            &mut receipt.checks,
                            &subject.role,
                            &case.id,
                            checkpoint,
                            "visual",
                            "ERROR",
                            "capture export failed",
                            vec![error],
                        ),
                    }
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "exit_restoration",
                        "NOT_APPLICABLE",
                        "HELP-HOLLA-004 closes and reopens an overlay but does not exit the root TUI",
                        Vec::new(),
                    );
                }
            }
        }

        drop(session);
        let post_run = inspect_executable(&subject.executable.path);
        receipt.source_pair[subject_index].executable.actual_sha256 = post_run
            .as_ref()
            .ok()
            .map(|snapshot| snapshot.sha256.clone());
        let post_run_failure = match &post_run {
            Err(error) => Some(error.clone()),
            Ok(snapshot) => validate_executable_snapshot(subject, initial_snapshot, snapshot).err(),
        };
        if let Some(reason) = post_run_failure {
            mark_subject_build_stale(
                &mut receipt.checks,
                &subject.role,
                &case.id,
                &reason,
                post_run
                    .as_ref()
                    .ok()
                    .map(|snapshot| snapshot.sha256.as_str()),
            );
            paired_spawn_block = Some(format!(
                "{} changed during the journey: {reason}",
                subject.role
            ));
        }
    }

    validate_checkpoint_dimensions(&receipt.checks, &subjects, case)?;
    write_receipt(&receipt, &write_policy)?;
    Ok(receipt)
}

fn validate_checkpoint_dimensions(
    checks: &[CheckReceipt],
    subjects: &[Subject],
    case: &Case,
) -> Result<(), String> {
    let required = [
        "build",
        "launch",
        "first_frame",
        "interaction",
        "visual",
        "exit_restoration",
    ];
    for subject in subjects {
        for checkpoint in checkpoints(case) {
            for dimension in required {
                if !checks.iter().any(|check| {
                    check.subject_role == subject.role
                        && check.case_id == case.id
                        && check.checkpoint_id == checkpoint
                        && check.dimension == dimension
                }) {
                    return Err(format!(
                        "receipt is missing {dimension} check for {}:{}:{}",
                        subject.role, case.id, checkpoint
                    ));
                }
            }
        }
    }
    Ok(())
}

fn checkpoints(case: &Case) -> Vec<&str> {
    case.steps
        .iter()
        .filter_map(|step| match step {
            Step::Checkpoint { id, .. } => Some(id.as_str()),
            Step::Press { .. } => None,
        })
        .collect()
}

fn make_argv(executable: &Path, args: &[String]) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(executable.display().to_string());
    argv.extend(args.iter().cloned());
    argv
}

fn verify_subject_executable(subject: &Subject) -> Result<ExecutableSnapshot, String> {
    let snapshot = inspect_executable(&subject.executable.path)?;
    if snapshot.sha256 != subject.executable.sha256 {
        return Err(format!(
            "{} executable digest mismatch: expected {}, got {}",
            subject.role, subject.executable.sha256, snapshot.sha256
        ));
    }
    Ok(snapshot)
}

fn inspect_executable(path: &Path) -> Result<ExecutableSnapshot, String> {
    if !path.is_absolute() {
        return Err(format!(
            "executable path must be absolute: {}",
            path.display()
        ));
    }
    let path_before = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect executable {}: {error}", path.display()))?;
    if path_before.file_type().is_symlink() || !path_before.is_file() {
        return Err(format!(
            "executable path must be a regular file, not a symlink: {}",
            path.display()
        ));
    }
    let mut file =
        File::open(path).map_err(|error| format!("open executable {}: {error}", path.display()))?;
    let opened_before = file
        .metadata()
        .map_err(|error| format!("inspect opened executable {}: {error}", path.display()))?;
    let path_before = executable_metadata(&path_before);
    let opened_before = executable_metadata(&opened_before);
    if path_before != opened_before {
        return Err(format!(
            "executable changed while opening {}",
            path.display()
        ));
    }

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("read executable {}: {error}", path.display()))?;
    let opened_after = file
        .metadata()
        .map_err(|error| format!("reinspect opened executable {}: {error}", path.display()))?;
    let path_after = fs::symlink_metadata(path)
        .map_err(|error| format!("reinspect executable {}: {error}", path.display()))?;
    if path_after.file_type().is_symlink() || !path_after.is_file() {
        return Err(format!(
            "executable path changed to a non-regular file while hashing: {}",
            path.display()
        ));
    }
    let opened_after = executable_metadata(&opened_after);
    let path_after = executable_metadata(&path_after);
    if opened_before != opened_after || opened_after != path_after {
        return Err(format!(
            "executable metadata changed while hashing {}",
            path.display()
        ));
    }

    Ok(ExecutableSnapshot {
        sha256: sha256_bytes(&bytes),
        metadata: opened_after,
    })
}

fn executable_metadata(metadata: &fs::Metadata) -> ExecutableMetadata {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    ExecutableMetadata {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
    }
}

fn validate_executable_snapshot(
    subject: &Subject,
    initial: &ExecutableSnapshot,
    current: &ExecutableSnapshot,
) -> Result<(), String> {
    if current.sha256 != subject.executable.sha256 {
        return Err(format!(
            "{} executable digest changed: expected {}, got {}",
            subject.role, subject.executable.sha256, current.sha256
        ));
    }
    if current != initial {
        return Err(format!(
            "{} executable file identity or metadata changed after preflight",
            subject.role
        ));
    }
    Ok(())
}

fn mark_subject_build_stale(
    checks: &mut [CheckReceipt],
    role: &str,
    case_id: &str,
    reason: &str,
    actual_sha256: Option<&str>,
) {
    for check in checks.iter_mut().filter(|check| {
        check.subject_role == role && check.case_id == case_id && check.dimension == "build"
    }) {
        check.status = "STALE".to_string();
        check.reason = format!("executable changed after validation: {reason}");
        if let Some(actual_sha256) = actual_sha256 {
            check
                .evidence
                .push(format!("post-run executable SHA-256: {actual_sha256}"));
        }
    }
}

fn validate_expected_generation(expected: &ExpectedGenerationInput) -> Result<String, String> {
    let root = expected
        .root
        .as_deref()
        .ok_or_else(|| "expected generation root is absent".to_string())?;
    if !root.is_dir() {
        return Err(format!(
            "expected generation root is missing: {}",
            root.display()
        ));
    }
    let digest = digest_tree(root)?;
    if digest != expected.sha256 {
        return Err(format!(
            "expected generation digest mismatch: expected {}, got {}",
            expected.sha256, digest
        ));
    }
    Ok(digest)
}

fn expected_visual_reason(
    expected: &ExpectedGenerationInput,
    actual: &Result<String, String>,
    admitted: bool,
    legacy_path: Option<&str>,
) -> String {
    if let Err(error) = actual {
        return format!("shared expected generation is unavailable or invalid: {error}");
    }
    if admitted {
        format!(
            "expected generation {:?} hash and external admission record match; exact cell, cursor, and decoded-pixel comparison remains pending (alias: {})",
            expected.id,
            legacy_path.unwrap_or("none")
        )
    } else {
        format!(
            "expected generation {:?} matches its supplied tree hash, but no external admission binds it; visual comparison remains blocked (alias: {})",
            expected.id,
            legacy_path.unwrap_or("none")
        )
    }
}

fn expected_generation_state(actual: &Result<String, String>, admitted: bool) -> &'static str {
    if actual.is_err() {
        "BLOCKED_MISSING_OR_INVALID"
    } else if admitted {
        "ADMITTED"
    } else {
        "HASH_MATCH"
    }
}

fn launch(
    argv: &[String],
    case: &Case,
    profile: &Profile,
) -> Result<tuiscotti::tui::Session, String> {
    let tui = tuiscotti::tui::Tui::new(argv)
        .size(case.geometry.cols, case.geometry.rows)
        .env("TERM", &profile.terminal.term)
        .env("COLORTERM", &profile.terminal.colorterm)
        .env("LC_ALL", &profile.terminal.locale)
        .env("HOLLA_NO_HISTORY", "1")
        .env_remove("NO_COLOR")
        .env_remove("HOLLA_NO_MOTION")
        .env_remove("JACKIN_NO_MOTION")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .env_remove(TRUST_RECORD_PATH_ENV)
        .env_remove(TRUST_RECORD_SHA_ENV)
        .env_remove("TERMROCK_E2E_SUBJECT_MANIFEST")
        .env_remove(RECEIPT_PATH_ENV)
        .env_remove(WRITE_ROOT_ENV)
        .env_remove(PROTECTED_ROOTS_ENV);
    tui.spawn()
        .map_err(|error| format!("Tuiscotti PTY spawn failed: {error}"))
}

fn wait_for_observation(
    session: &mut tuiscotti::tui::Session,
    conditions: &[WaitCondition],
    timeout_ms: u64,
) -> Result<tuiscotti::Observation, String> {
    let timeout = Duration::from_millis(timeout_ms.max(1));
    session
        .wait_predicate_timeout(
            |observation| {
                let text =
                    tuiscotti::render::frame_from_screen(&observation.screen, "default").text();
                conditions
                    .iter()
                    .all(|condition| match condition.kind.as_str() {
                        "contains" => text.contains(&condition.needle),
                        "absent" => !text.contains(&condition.needle),
                        _ => false,
                    })
            },
            timeout,
        )
        .map_err(|error| format!("wait predicates {conditions:?} timed out: {error}"))
}

fn evaluate_assertion(
    assertion: &Assertion,
    screen_text: &str,
    passed: &BTreeSet<String>,
) -> (&'static str, String, Vec<String>) {
    if let Some(required) = &assertion.requires {
        if !passed.contains(required) {
            return (
                "BLOCKED",
                format!(
                    "negative assertion requires positive precondition {required}, which did not pass"
                ),
                Vec::new(),
            );
        }
    }
    let matched = match assertion.kind.as_str() {
        "contains" => assertion
            .needle
            .as_deref()
            .is_some_and(|needle| screen_text.contains(needle)),
        "absent" => assertion
            .needle
            .as_deref()
            .is_some_and(|needle| !screen_text.contains(needle)),
        "same_line" => {
            let (Some(left), Some(right)) = (assertion.left.as_deref(), assertion.right.as_deref())
            else {
                return (
                    "ERROR",
                    "same_line assertion needs left and right strings".to_string(),
                    Vec::new(),
                );
            };
            screen_text
                .lines()
                .any(|line| line.contains(left) && line.contains(right))
        }
        other => {
            return (
                "ERROR",
                format!("unsupported assertion kind {other:?}"),
                Vec::new(),
            );
        }
    };
    if matched {
        (
            "PASS",
            "assertion matched the captured PTY screen".to_string(),
            Vec::new(),
        )
    } else {
        (
            "FAIL",
            format!(
                "assertion {:?} did not match the captured PTY screen",
                assertion.id
            ),
            vec![screen_text.to_string()],
        )
    }
}

fn append_not_run_checkpoint(
    checks: &mut Vec<CheckReceipt>,
    role: &str,
    case_id: &str,
    checkpoint: &str,
    build_status: &str,
    build_reason: &str,
    launch_status: &str,
    launch_reason: &str,
) {
    push_check(
        checks,
        role,
        case_id,
        checkpoint,
        "build",
        build_status,
        build_reason,
        Vec::new(),
    );
    push_check(
        checks,
        role,
        case_id,
        checkpoint,
        "launch",
        launch_status,
        launch_reason,
        Vec::new(),
    );
    for dimension in ["first_frame", "interaction", "visual"] {
        push_check(
            checks,
            role,
            case_id,
            checkpoint,
            dimension,
            "NOT_RUN",
            "subject did not reach this checkpoint",
            Vec::new(),
        );
    }
    append_non_execution_dimensions(checks, role, case_id, checkpoint);
}

fn append_non_execution_dimensions(
    checks: &mut Vec<CheckReceipt>,
    role: &str,
    case_id: &str,
    checkpoint: &str,
) {
    push_check(
        checks,
        role,
        case_id,
        checkpoint,
        "exit_restoration",
        "NOT_RUN",
        "subject did not reach process exit",
        Vec::new(),
    );
}

fn append_readiness_failure(
    checks: &mut Vec<CheckReceipt>,
    role: &str,
    case_id: &str,
    checkpoint: &str,
    reason: &str,
) {
    push_check(
        checks,
        role,
        case_id,
        checkpoint,
        "first_frame",
        "ERROR",
        "checkpoint readiness failed",
        vec![reason.to_string()],
    );
    for (dimension, status, explanation) in [
        (
            "interaction",
            "NOT_RUN",
            "checkpoint assertions did not run because readiness failed",
        ),
        (
            "visual",
            "NOT_RUN",
            "capture did not run because checkpoint readiness failed",
        ),
    ] {
        push_check(
            checks,
            role,
            case_id,
            checkpoint,
            dimension,
            status,
            explanation,
            Vec::new(),
        );
    }
    append_non_execution_dimensions(checks, role, case_id, checkpoint);
}

fn push_check(
    checks: &mut Vec<CheckReceipt>,
    role: &str,
    case_id: &str,
    checkpoint: &str,
    dimension: &str,
    status: &str,
    reason: &str,
    evidence: Vec<String>,
) {
    checks.push(check(
        role,
        case_id,
        checkpoint,
        dimension,
        status,
        reason.to_string(),
        evidence,
    ));
}

fn check(
    role: &str,
    case_id: &str,
    checkpoint: &str,
    dimension: &str,
    status: &str,
    reason: String,
    evidence: Vec<String>,
) -> CheckReceipt {
    CheckReceipt {
        id: format!("{role}:{case_id}:{checkpoint}:{dimension}"),
        subject_role: role.to_string(),
        case_id: case_id.to_string(),
        checkpoint_id: checkpoint.to_string(),
        dimension: dimension.to_string(),
        status: status.to_string(),
        reason,
        evidence,
    }
}

fn write_capture(
    subject: &Subject,
    write_root: &Path,
    actual_root: &Path,
    case: &Case,
    checkpoint: &str,
    frame: &tuiscotti::Frame,
    legacy_snapshot_path: Option<&str>,
) -> Result<Vec<ArtifactReceipt>, String> {
    use tuiscotti::formats::capture_all;
    use tuiscotti::{Profile as RenderProfile, VENDORED_FACES};

    let output_substep = checkpoint_substep(case, checkpoint);
    let relative_stem = format!(
        "{}/{}/{}/{}x{}/{}",
        case.app,
        case.screen,
        output_substep,
        case.geometry.cols,
        case.geometry.rows,
        case.color_path
    );
    let mut renderer = RenderProfile::default_profile()
        .renderer(&VENDORED_FACES)
        .map_err(|error| format!("initialize Tuiscotti renderer: {error}"))?;
    let bundle = capture_all(&mut renderer, frame, &format!("{}:{checkpoint}", case.id))
        .map_err(|error| format!("capture Tuiscotti bundle: {error}"))?;
    let rendered = renderer
        .render(frame)
        .map_err(|error| format!("render Tuiscotti PNG: {error}"))?;
    let ascii_loss = serde_json::to_vec_pretty(&serde_json::json!({
        "lossy": bundle.ascii.lossy(),
        "substitutions": bundle.ascii.substitutions.iter().map(|item| serde_json::json!({
            "x": item.x,
            "y": item.y,
            "original": item.original,
            "replacement": item.replacement
        })).collect::<Vec<_>>()
    }))
    .map_err(|error| format!("serialize ASCII loss: {error}"))?;
    let observations = serde_json::to_vec_pretty(&serde_json::json!({
        "case_id": case.id,
        "checkpoint_id": checkpoint,
        "dimensions": {"cols": frame.cols, "rows": frame.rows},
        "cursor": {"x": frame.cursor.x, "y": frame.cursor.y, "visible": frame.cursor.visible},
        "provenance": {
            "tool": frame.provenance.tool,
            "tool_version": frame.provenance.tool_version,
            "profile": frame.provenance.profile,
            "source": frame.provenance.source,
            "argv": frame.provenance.argv
        },
        "toolkit_unavailable_properties": ["application-internal-state", "private-action-enum"],
        "frame_digest": frame.digest(),
        "legacy_snapshot_alias": legacy_snapshot_path
    }))
    .map_err(|error| format!("serialize observation facts: {error}"))?;

    let primary = [
        ("frame_json", "frame.json", bundle.json.into_bytes()),
        ("ansi", "ansi", bundle.ansi.into_bytes()),
        ("html", "html", bundle.html.into_bytes()),
        ("png", "png", bundle.png),
        ("ascii", "ascii", bundle.ascii.text.into_bytes()),
        ("txt", "txt", bundle.txt.into_bytes()),
        ("ascii_loss_json", "ascii.loss.json", ascii_loss),
        (
            "png_fidelity_json",
            "png.fidelity.json",
            rendered.fidelity.to_json().into_bytes(),
        ),
        ("observations_json", "observations.json", observations),
    ];
    let mut outputs = Vec::new();
    for (format, extension, bytes) in primary {
        outputs.push((format.to_string(), extension.to_string(), bytes));
    }

    let manifest = serde_json::to_vec_pretty(&serde_json::json!({
        "schema": "termrock-e2e/capture-manifest-v1",
        "case_id": case.id,
        "checkpoint_id": checkpoint,
        "canonical_path": relative_stem.as_str(),
        "legacy_snapshot_path": legacy_snapshot_path,
        "frame_digest": frame.digest(),
        "profile": frame.provenance.profile,
        "artifacts": outputs.iter().map(|(format, path, bytes)| serde_json::json!({
            "format": format,
            "path": format!("{relative_stem}/{path}"),
            "sha256": sha256_bytes(bytes),
            "bytes": bytes.len()
        })).collect::<Vec<_>>()
    }))
    .map_err(|error| format!("serialize capture manifest: {error}"))?;
    outputs.push((
        "manifest_json".to_string(),
        "manifest.json".to_string(),
        manifest,
    ));
    let final_dir = atomic_publish_capture(write_root, actual_root, &relative_stem, &outputs)?;

    Ok(outputs
        .into_iter()
        .map(|(format, filename, bytes)| ArtifactReceipt {
            subject_role: subject.role.clone(),
            case_id: case.id.clone(),
            checkpoint_id: checkpoint.to_string(),
            format,
            path: final_dir.join(filename),
            sha256: sha256_bytes(&bytes),
            bytes: bytes.len() as u64,
        })
        .collect())
}

fn atomic_publish_capture(
    write_root: &Path,
    actual_root: &Path,
    canonical_path: &str,
    outputs: &[(String, String, Vec<u8>)],
) -> Result<PathBuf, String> {
    if actual_root == write_root || !actual_root.starts_with(write_root) {
        return Err(format!(
            "actual output root must be strictly beneath the write root: {}",
            actual_root.display()
        ));
    }
    let Some((manifest_format, manifest_filename, _)) = outputs.last() else {
        return Err("capture output set is empty".to_string());
    };
    if manifest_format != "manifest_json" || manifest_filename != "manifest.json" {
        return Err("capture manifest must be the final output named manifest.json".to_string());
    }
    let mut filenames = BTreeSet::new();
    for (_, filename, _) in outputs {
        let path = Path::new(filename);
        if path.components().count() != 1
            || !matches!(path.components().next(), Some(Component::Normal(_)))
        {
            return Err(format!(
                "capture artifact filename is not a basename: {filename:?}"
            ));
        }
        if !filenames.insert(filename.as_str()) {
            return Err(format!("duplicate capture artifact filename {filename:?}"));
        }
    }

    let relative = Path::new(canonical_path);
    if relative.is_absolute()
        || relative.as_os_str().is_empty()
        || canonical_path.contains('\0')
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("invalid canonical capture path {canonical_path:?}"));
    }
    let final_dir = actual_root.join(relative);
    let parent = final_dir.parent().ok_or_else(|| {
        format!(
            "canonical capture path has no parent: {}",
            final_dir.display()
        )
    })?;
    ensure_directory_chain_without_symlinks(write_root, parent, true)?;
    if path_exists_no_follow(&final_dir)? {
        return Err(format!(
            "published capture already exists; refusing to overwrite {}",
            final_dir.display()
        ));
    }
    let stage_dir = create_capture_stage(parent, canonical_path)?;
    let write_result = (|| -> Result<(), String> {
        for (_, filename, bytes) in &outputs[..outputs.len() - 1] {
            write_new_synced(&stage_dir.join(filename), bytes)?;
        }
        // The manifest is written last, after every listed artifact is durable.
        let (_, manifest_filename, manifest_bytes) = outputs
            .last()
            .ok_or_else(|| "capture manifest is missing".to_string())?;
        write_new_synced(&stage_dir.join(manifest_filename), manifest_bytes)?;
        sync_directory(&stage_dir)?;
        fs::rename(&stage_dir, &final_dir).map_err(|error| {
            format!(
                "publish staged capture {} as {}: {error}",
                stage_dir.display(),
                final_dir.display()
            )
        })?;
        if let Err(error) = sync_directory(parent) {
            let reason = format!(
                "sync published capture parent {}: {error}",
                parent.display()
            );
            return match quarantine_capture_stage(parent, canonical_path, &final_dir) {
                Ok(quarantine) => Err(format!(
                    "{reason}; published directory retained at {}",
                    quarantine.display()
                )),
                Err(quarantine_error) => Err(format!(
                    "{reason}; could not quarantine published directory {}: {quarantine_error}",
                    final_dir.display()
                )),
            };
        }
        Ok(())
    })();
    if let Err(error) = write_result {
        if stage_dir.exists() {
            return match quarantine_capture_stage(parent, canonical_path, &stage_dir) {
                Ok(quarantine) => Err(format!(
                    "{error}; incomplete capture retained at {}",
                    quarantine.display()
                )),
                Err(quarantine_error) => Err(format!(
                    "{error}; could not quarantine stage {}: {quarantine_error}",
                    stage_dir.display()
                )),
            };
        }
        return Err(error);
    }
    Ok(final_dir)
}

fn ensure_directory_chain_without_symlinks(
    root: &Path,
    directory: &Path,
    create_missing: bool,
) -> Result<(), String> {
    validate_real_directory_beneath(root, root)?;
    let relative_directory = directory.strip_prefix(root).map_err(|_| {
        format!(
            "write directory {} is outside its allowed root {}",
            directory.display(),
            root.display()
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative_directory.components() {
        let Component::Normal(name) = component else {
            return Err(format!(
                "write directory has a non-normal path component: {}",
                directory.display()
            ));
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(_) => validate_real_directory_beneath(root, &current)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create_missing => {
                match fs::create_dir(&current) {
                    Ok(()) => validate_real_directory_beneath(root, &current)?,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        // Re-inspect a concurrently created entry without following links.
                        validate_real_directory_beneath(root, &current)?;
                    }
                    Err(error) => {
                        return Err(format!(
                            "create write directory {}: {error}",
                            current.display()
                        ));
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!(
                    "write directory does not exist: {}",
                    current.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "inspect write directory {}: {error}",
                    current.display()
                ));
            }
        }
    }
    Ok(())
}

fn validate_real_directory_beneath(root: &Path, directory: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(directory).map_err(|error| {
        format!(
            "inspect write directory {} without following links: {error}",
            directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "write path contains a symlink: {}",
            directory.display()
        ));
    }
    if !metadata.is_dir() {
        return Err(format!(
            "write path component is not a directory: {}",
            directory.display()
        ));
    }
    let canonical = fs::canonicalize(directory).map_err(|error| {
        format!(
            "canonicalize write directory {}: {error}",
            directory.display()
        )
    })?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "write directory escaped its allowed root: {} is outside {}",
            canonical.display(),
            root.display()
        ));
    }
    Ok(())
}

fn create_capture_stage(parent: &Path, canonical_path: &str) -> Result<PathBuf, String> {
    for _ in 0..16 {
        let suffix = CAPTURE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(
            ".capture-stage-{}-{suffix}-{}",
            std::process::id(),
            sha256_bytes(canonical_path.as_bytes())[..12].to_string()
        ));
        match fs::create_dir(&stage) {
            Ok(()) => return Ok(stage),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "create capture staging directory {}: {error}",
                    stage.display()
                ));
            }
        }
    }
    Err(format!(
        "could not allocate unique capture staging directory under {}",
        parent.display()
    ))
}

fn quarantine_capture_stage(
    parent: &Path,
    canonical_path: &str,
    stage_dir: &Path,
) -> Result<PathBuf, String> {
    for _ in 0..16 {
        let suffix = CAPTURE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let quarantine = parent.join(format!(
            ".capture-quarantine-{}-{suffix}-{}",
            std::process::id(),
            sha256_bytes(canonical_path.as_bytes())[..12].to_string()
        ));
        match fs::rename(stage_dir, &quarantine) {
            Ok(()) => return Ok(quarantine),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "rename staged capture to {}: {error}",
                    quarantine.display()
                ));
            }
        }
    }
    Err("could not allocate unique capture quarantine directory".to_string())
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create staged artifact {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("write staged artifact {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync staged artifact {}: {error}", path.display()))
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("sync directory {}: {error}", path.display()))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn checkpoint_substep(case: &Case, checkpoint: &str) -> String {
    match checkpoint {
        "00-boot" => format!("{}/boot", case.substep),
        "01-help" => case.substep.clone(),
        "02-finder" => format!("{}/closed", case.substep),
        "03-help-again" => format!("{}/reopened", case.substep),
        other => format!("{}/{other}", case.substep),
    }
}

fn write_receipt(receipt: &RunReceipt, policy: &WritePolicyReceipt) -> Result<(), String> {
    if receipt.write_policy != *policy
        || receipt.trust.write_policy_sha256 != policy.sha256
        || write_policy_digest(policy)? != policy.sha256
    {
        return Err("run receipt write policy binding is inconsistent".to_string());
    }
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("serialize run receipt: {error}"))?;
    publish_receipt_no_replace(&policy.write_root, &policy.receipt_path, &bytes)
}

fn publish_receipt_no_replace(write_root: &Path, path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("receipt destination has no parent: {}", path.display()))?;
    ensure_directory_chain_without_symlinks(write_root, parent, false)?;
    if path == write_root || !path.starts_with(write_root) {
        return Err(format!(
            "receipt destination is outside its allowed write root: {}",
            path.display()
        ));
    }
    if path_exists_no_follow(path)? {
        return Err(format!(
            "receipt destination already exists; refusing to overwrite {}",
            path.display()
        ));
    }

    let stage = stage_receipt_bytes(parent, bytes)?;
    if let Err(error) = fs::hard_link(&stage, path) {
        return Err(format!(
            "publish run receipt without replacement at {}: {error}; staged bytes retained at {}",
            path.display(),
            stage.display(),
        ));
    }
    fs::remove_file(&stage).map_err(|error| {
        format!(
            "remove staged receipt link {} after publication: {error}",
            stage.display()
        )
    })?;
    sync_directory(parent)
}

fn stage_receipt_bytes(parent: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    for _ in 0..16 {
        let suffix = CAPTURE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(
            ".termrock-e2e-receipt-{}-{suffix}.tmp",
            std::process::id()
        ));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "create staged receipt {}: {error}",
                    stage.display()
                ));
            }
        };
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            return Err(format!(
                "write staged receipt {}: {error}; partial staged file retained",
                stage.display()
            ));
        }
        return Ok(stage);
    }
    Err(format!(
        "could not allocate unique staged receipt under {}",
        parent.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_error_keeps_build_pass_and_all_checkpoint_dimensions() {
        let mut checks = Vec::new();
        append_not_run_checkpoint(
            &mut checks,
            "reference",
            "HELP-HOLLA-004",
            "00-boot",
            "PASS",
            "executable digest matched",
            "ERROR",
            "PTY spawn failed",
        );

        assert_eq!(dimension_status(&checks, "build"), Some("PASS"));
        assert_eq!(dimension_status(&checks, "launch"), Some("ERROR"));
        assert_eq!(dimension_status(&checks, "first_frame"), Some("NOT_RUN"));
        assert_eq!(dimension_status(&checks, "interaction"), Some("NOT_RUN"));
        assert_eq!(dimension_status(&checks, "visual"), Some("NOT_RUN"));
        assert_eq!(
            dimension_status(&checks, "exit_restoration"),
            Some("NOT_RUN")
        );
    }

    #[test]
    fn readiness_failure_records_visual_not_run() {
        let mut checks = Vec::new();
        append_readiness_failure(
            &mut checks,
            "candidate",
            "HELP-HOLLA-004",
            "01-help",
            "needle timeout",
        );

        assert_eq!(dimension_status(&checks, "first_frame"), Some("ERROR"));
        assert_eq!(dimension_status(&checks, "interaction"), Some("NOT_RUN"));
        assert_eq!(dimension_status(&checks, "visual"), Some("NOT_RUN"));
        assert_eq!(
            dimension_status(&checks, "exit_restoration"),
            Some("NOT_RUN")
        );
    }

    #[test]
    fn checkpoint_dimension_completeness_rejects_missing_visual_receipt() {
        let case = registry().unwrap().cases.remove(0);
        let subjects = vec![test_subject("candidate", unique_temp_path("dimensions"))];
        let mut checks = Vec::new();
        for checkpoint in checkpoints(&case) {
            push_check(
                &mut checks,
                "candidate",
                &case.id,
                checkpoint,
                "build",
                "PASS",
                "test build",
                Vec::new(),
            );
            push_check(
                &mut checks,
                "candidate",
                &case.id,
                checkpoint,
                "launch",
                "PASS",
                "test launch",
                Vec::new(),
            );
            append_readiness_failure(
                &mut checks,
                "candidate",
                &case.id,
                checkpoint,
                "test timeout",
            );
        }
        assert!(validate_checkpoint_dimensions(&checks, &subjects, &case).is_ok());
        checks.retain(|check| !(check.checkpoint_id == "00-boot" && check.dimension == "visual"));
        assert!(
            validate_checkpoint_dimensions(&checks, &subjects, &case)
                .unwrap_err()
                .contains("missing visual")
        );
    }

    #[test]
    fn actual_output_roots_reject_equal_nested_and_expected_overlap() {
        let base = unique_temp_path("roots");
        let reference = test_subject("reference", base.join("reference"));
        let equal = test_subject("candidate", base.join("reference"));
        assert!(
            validate_actual_output_roots(&[reference.clone(), equal], None)
                .unwrap_err()
                .contains("overlap")
        );

        let nested = test_subject("candidate", base.join("reference/nested"));
        assert!(
            validate_actual_output_roots(&[reference.clone(), nested], None)
                .unwrap_err()
                .contains("overlap")
        );

        assert!(
            validate_actual_output_roots(&[reference], Some(&base.join("reference/expected")))
                .unwrap_err()
                .contains("overlap")
        );
    }

    #[cfg(unix)]
    #[test]
    fn actual_output_roots_canonicalize_existing_symlink_ancestors() {
        use std::os::unix::fs::symlink;

        let base = unique_temp_path("symlink-roots");
        let real = base.join("real");
        let alias = base.join("alias");
        fs::create_dir_all(&real).expect("create canonical parent");
        symlink(&real, &alias).expect("create symlink alias");
        let reference = test_subject("reference", real.join("actual"));
        let candidate = test_subject("candidate", alias.join("actual/nested"));
        let result = validate_actual_output_roots(&[reference, candidate], None);
        fs::remove_dir_all(&base).expect("remove temporary symlink tree");
        assert!(result.unwrap_err().contains("overlap"));
    }

    #[test]
    fn atomic_capture_writes_manifest_last_and_refuses_overwrite() {
        let write_root = unique_temp_path("capture-write");
        fs::create_dir_all(&write_root).expect("create capture write root");
        let write_root = fs::canonicalize(&write_root).expect("canonicalize capture write root");
        let actual_root = write_root.join("actual");
        let outputs = vec![
            (
                "frame_json".to_string(),
                "frame.json".to_string(),
                b"frame".to_vec(),
            ),
            (
                "manifest_json".to_string(),
                "manifest.json".to_string(),
                b"sealed".to_vec(),
            ),
        ];
        let published = atomic_publish_capture(
            &write_root,
            &actual_root,
            "holla/finder/help-overlay/120x40/truecolor",
            &outputs,
        )
        .expect("publish complete capture");
        assert_eq!(fs::read(published.join("frame.json")).unwrap(), b"frame");
        assert_eq!(
            fs::read(published.join("manifest.json")).unwrap(),
            b"sealed"
        );
        assert!(
            atomic_publish_capture(
                &write_root,
                &actual_root,
                "holla/finder/help-overlay/120x40/truecolor",
                &outputs
            )
            .is_err()
        );
        assert_eq!(fs::read(published.join("frame.json")).unwrap(), b"frame");
        fs::remove_dir_all(write_root).expect("remove published capture tree");
    }

    #[test]
    fn failed_atomic_capture_is_quarantined_without_publishing_final_dir() {
        let write_root = unique_temp_path("capture-failure-write");
        fs::create_dir_all(&write_root).expect("create capture write root");
        let write_root = fs::canonicalize(&write_root).expect("canonicalize capture write root");
        let actual_root = write_root.join("actual");
        let outputs = vec![
            (
                "frame_json".to_string(),
                "frame.json".to_string(),
                b"frame".to_vec(),
            ),
            ("broken".to_string(), "\0".to_string(), b"fail".to_vec()),
            (
                "manifest_json".to_string(),
                "manifest.json".to_string(),
                b"sealed".to_vec(),
            ),
        ];
        let result = atomic_publish_capture(
            &write_root,
            &actual_root,
            "holla/finder/help-overlay/120x40/truecolor",
            &outputs,
        );
        assert!(result.unwrap_err().contains("retained at"));
        let parent = actual_root.join("holla/finder/help-overlay/120x40");
        assert!(!parent.join("truecolor").exists());
        assert!(
            fs::read_dir(parent)
                .expect("read capture parent")
                .filter_map(Result::ok)
                .any(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".capture-quarantine-"))
        );
        fs::remove_dir_all(write_root).expect("remove quarantined capture tree");
    }

    #[cfg(unix)]
    #[test]
    fn atomic_capture_rejects_symlink_below_output_root_without_touching_target() {
        use std::os::unix::fs::symlink;

        let write_root = unique_temp_path("capture-symlink-write");
        let protected = unique_temp_path("capture-symlink-protected");
        fs::create_dir_all(&write_root).expect("create capture write root");
        let write_root = fs::canonicalize(&write_root).expect("canonicalize capture write root");
        let actual_root = write_root.join("actual");
        fs::create_dir_all(&actual_root).expect("create capture output root");
        fs::create_dir_all(&protected).expect("create protected output");
        let sentinel = protected.join("sentinel.txt");
        fs::write(&sentinel, b"protected").expect("write protected sentinel");
        symlink(&protected, actual_root.join("holla")).expect("link nested output directory");
        let outputs = vec![
            (
                "frame_json".to_string(),
                "frame.json".to_string(),
                b"frame".to_vec(),
            ),
            (
                "manifest_json".to_string(),
                "manifest.json".to_string(),
                b"sealed".to_vec(),
            ),
        ];

        assert!(
            atomic_publish_capture(
                &write_root,
                &actual_root,
                "../capture-symlink-protected/escaped/120x40/truecolor",
                &outputs,
            )
            .unwrap_err()
            .contains("invalid canonical capture path")
        );
        let error = atomic_publish_capture(
            &write_root,
            &actual_root,
            "holla/finder/help-overlay/120x40/truecolor",
            &outputs,
        )
        .unwrap_err();
        assert!(error.contains("write path contains a symlink"));
        assert_eq!(fs::read(&sentinel).unwrap(), b"protected");
        assert!(!protected.join("finder").exists());

        fs::remove_dir_all(&write_root).expect("remove symlink output root");
        fs::remove_dir_all(&protected).expect("remove protected output");
    }

    #[test]
    fn suite_digest_is_sensitive_and_matches_its_compiled_value() {
        let root = unique_temp_path("digest");
        fs::create_dir_all(&root).expect("create temporary digest root");
        fs::write(root.join("cases.json"), b"case-a").expect("write digest input");
        let first = digest_tree(&root).expect("hash initial digest tree");
        fs::write(root.join("cases.json"), b"case-b").expect("change digest input");
        let changed = digest_tree(&root).expect("hash changed digest tree");
        fs::remove_dir_all(&root).expect("remove temporary digest root");
        assert_ne!(first, changed);

        let runtime = suite_digest().expect("hash current suite files");
        assert_eq!(runtime, COMPILED_SUITE_SHA256);
        assert!(verify_compiled_suite_digest("stale", &runtime).is_err());
    }

    #[test]
    fn builder_receipt_digest_changes_when_bound_executable_changes() {
        let subject = test_subject("reference", unique_temp_path("receipt-output"));
        let first = builder_receipt_digest(&subject.builder_receipt).unwrap();
        let mut changed = subject.builder_receipt.clone();
        changed.executable_sha256 = "f".repeat(64);
        assert_ne!(first, builder_receipt_digest(&changed).unwrap());
    }

    #[test]
    fn executable_snapshot_detects_content_change_before_spawn() {
        let root = unique_temp_path("executable-snapshot");
        fs::create_dir_all(&root).expect("create executable test root");
        let path = root.join("holla");
        fs::write(&path, b"binary-v1").expect("write first executable");
        let initial = inspect_executable(&path).expect("inspect initial executable");
        let mut subject = test_subject("reference", root.join("captures"));
        subject.executable.path = path.clone();
        subject.executable.sha256 = initial.sha256.clone();
        let verified = verify_subject_executable(&subject).expect("verify initial executable");
        assert_eq!(verified, initial);

        fs::write(&path, b"binary-v2").expect("replace executable contents");
        let current = inspect_executable(&path).expect("inspect changed executable");
        assert!(
            validate_executable_snapshot(&subject, &initial, &current)
                .unwrap_err()
                .contains("digest changed")
        );
        fs::remove_dir_all(root).expect("remove executable test root");
    }

    #[cfg(unix)]
    #[test]
    fn executable_snapshot_detects_same_bytes_at_replaced_file_identity() {
        let root = unique_temp_path("executable-replacement");
        fs::create_dir_all(&root).expect("create executable test root");
        let path = root.join("holla");
        let retained = root.join("holla-old");
        fs::write(&path, b"same-binary").expect("write original executable");
        let initial = inspect_executable(&path).expect("inspect original executable");
        let mut subject = test_subject("reference", root.join("captures"));
        subject.executable.path = path.clone();
        subject.executable.sha256 = initial.sha256.clone();

        fs::rename(&path, &retained).expect("retain original inode");
        fs::write(&path, b"same-binary").expect("write replacement executable");
        let replacement = inspect_executable(&path).expect("inspect replacement executable");
        assert_eq!(replacement.sha256, initial.sha256);
        assert!(
            validate_executable_snapshot(&subject, &initial, &replacement)
                .unwrap_err()
                .contains("file identity or metadata changed")
        );
        fs::remove_dir_all(root).expect("remove executable test root");
    }

    #[test]
    fn post_run_executable_change_marks_build_receipts_stale() {
        let mut checks = Vec::new();
        push_check(
            &mut checks,
            "reference",
            "HELP-HOLLA-004",
            "00-boot",
            "build",
            "PASS",
            "initial executable hash matched",
            Vec::new(),
        );
        let changed_digest = "a".repeat(64);
        mark_subject_build_stale(
            &mut checks,
            "reference",
            "HELP-HOLLA-004",
            "digest changed",
            Some(&changed_digest),
        );

        assert_eq!(dimension_status(&checks, "build"), Some("STALE"));
        assert!(checks[0].reason.contains("digest changed"));
        assert!(checks[0].evidence[0].starts_with("post-run executable SHA-256:"));
    }

    #[test]
    fn trust_record_binds_compiled_suite_binary_pair_and_optional_expected_generation() {
        let subjects = vec![
            test_subject("reference", unique_temp_path("trust-reference")),
            test_subject("candidate", unique_temp_path("trust-candidate")),
        ];
        let manifest = SubjectManifest {
            schema: SUBJECT_SCHEMA.to_string(),
            run_id: "trust-test".to_string(),
            suite_revision: "test-revision".to_string(),
            suite_sha256: "a".repeat(64),
            expected_generation: ExpectedGenerationInput {
                id: "shared-parity-v1".to_string(),
                root: None,
                sha256: "b".repeat(64),
            },
            subjects: subjects.clone(),
        };
        let test_binary = "c".repeat(64);
        let write_policy_sha256 = "7".repeat(64);
        let mut record = test_trust_record(&subjects, &test_binary);
        let expected_actual = Err("no expected generation".to_string());
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &expected_actual,
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_ok()
        );

        record.suite.digest = "0".repeat(64);
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &expected_actual,
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_err()
        );
        record.suite.digest = manifest.suite_sha256.clone();
        record.subjects[0].builder_receipt_sha256 = "0".repeat(64);
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &expected_actual,
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_err()
        );

        record.subjects[0].builder_receipt_sha256 = subjects[0].builder_receipt_sha256.clone();
        record.expected_generation = Some(TrustedExpectedGeneration {
            id: manifest.expected_generation.id.clone(),
            sha256: manifest.expected_generation.sha256.clone(),
            admission_receipt_sha256: "9".repeat(64),
        });
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &Ok(manifest.expected_generation.sha256.clone()),
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_ok()
        );
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &Err("generation bytes mismatch".to_string()),
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_err()
        );
    }

    #[test]
    fn trust_record_rejects_a_different_write_policy_digest() {
        let subjects = vec![
            test_subject("reference", unique_temp_path("trust-policy-reference")),
            test_subject("candidate", unique_temp_path("trust-policy-candidate")),
        ];
        let manifest = SubjectManifest {
            schema: SUBJECT_SCHEMA.to_string(),
            run_id: "trust-policy-test".to_string(),
            suite_revision: "test-revision".to_string(),
            suite_sha256: "a".repeat(64),
            expected_generation: ExpectedGenerationInput {
                id: "shared-parity-v1".to_string(),
                root: None,
                sha256: "b".repeat(64),
            },
            subjects: subjects.clone(),
        };
        let test_binary = "c".repeat(64);
        let mut record = test_trust_record(&subjects, &test_binary);
        record.write_policy_sha256 = "0".repeat(64);
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &Err("no expected generation".to_string()),
                &"7".repeat(64),
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .unwrap_err()
            .contains("write policy digest")
        );
    }

    #[test]
    fn write_policy_is_canonical_and_binds_outputs_receipt_and_protected_roots() {
        let fixture = test_write_policy_fixture("policy-digest");
        let first = resolve_test_write_policy(&fixture, None).expect("resolve initial policy");

        let mut reordered = fixture.input.clone();
        reordered.protected_roots.reverse();
        let mut reversed_subjects = fixture.subjects.clone();
        reversed_subjects.reverse();
        let reordered_policy = resolve_write_policy(
            reordered,
            &reversed_subjects,
            None,
            &fixture.trust_source,
            &fixture.test_binary,
        )
        .expect("resolve policy with reordered inputs");
        assert_eq!(first.sha256, reordered_policy.sha256);

        let mut changed_receipt = fixture.input.clone();
        changed_receipt.receipt_path = fixture.base.join("write/run-next.json");
        let receipt_changed = resolve_write_policy(
            changed_receipt,
            &fixture.subjects,
            None,
            &fixture.trust_source,
            &fixture.test_binary,
        )
        .expect("resolve policy with a different receipt destination");
        assert_ne!(first.sha256, receipt_changed.sha256);

        let mut changed_protection = fixture.input.clone();
        changed_protection.protected_roots[4].path = fixture.base.join("oracle-extra");
        fs::create_dir_all(&changed_protection.protected_roots[4].path)
            .expect("create alternate oracle root");
        let roots_changed = resolve_write_policy(
            changed_protection,
            &fixture.subjects,
            None,
            &fixture.trust_source,
            &fixture.test_binary,
        )
        .expect("resolve policy with a different protected root");
        assert_ne!(first.sha256, roots_changed.sha256);

        let mut changed_outputs = fixture.subjects.clone();
        changed_outputs[0].actual_output_root = fixture.base.join("write/actual-reference-next");
        fs::create_dir_all(&changed_outputs[0].actual_output_root)
            .expect("create alternate actual output root");
        let outputs_changed = resolve_write_policy(
            fixture.input.clone(),
            &changed_outputs,
            None,
            &fixture.trust_source,
            &fixture.test_binary,
        )
        .expect("resolve policy with a different actual output root");
        assert_ne!(first.sha256, outputs_changed.sha256);

        let mut changed_write_root = fixture.input.clone();
        changed_write_root.write_root = fixture.base.join("write-next");
        fs::create_dir_all(&changed_write_root.write_root).expect("create alternate write root");
        changed_write_root.receipt_path = changed_write_root.write_root.join("run.json");
        let mut moved_outputs = fixture.subjects.clone();
        for (index, subject) in moved_outputs.iter_mut().enumerate() {
            subject.actual_output_root = changed_write_root
                .write_root
                .join(format!("actual-{index}"));
            fs::create_dir_all(&subject.actual_output_root)
                .expect("create moved actual output root");
        }
        let write_root_changed = resolve_write_policy(
            changed_write_root,
            &moved_outputs,
            None,
            &fixture.trust_source,
            &fixture.test_binary,
        )
        .expect("resolve policy with a different write root");
        assert_ne!(first.sha256, write_root_changed.sha256);
        let canonical_write_root =
            fs::canonicalize(&fixture.input.write_root).expect("canonicalize fixture write root");
        assert_eq!(first.receipt_path, canonical_write_root.join("run.json"));
        assert_eq!(first.actual_output_roots.len(), 2);
        assert!(
            first
                .protected_roots
                .iter()
                .any(|root| root.kind == "oracle")
        );
        fs::remove_dir_all(&fixture.base).expect("remove policy test fixture");
    }

    #[test]
    fn write_policy_rejects_protected_checkout_and_expected_roots_before_any_write() {
        let fixture = test_write_policy_fixture("policy-protected");

        let mut source_overlap = fixture.input.clone();
        source_overlap.write_root = fixture.input.protected_roots[0].path.clone();
        source_overlap.receipt_path = source_overlap.write_root.join("unused-receipt.json");
        assert!(
            resolve_write_policy(
                source_overlap,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("overlaps protected subject_source")
        );

        let mut artifact_overlap = fixture.input.clone();
        artifact_overlap.write_root = fixture.input.protected_roots[1].path.clone();
        artifact_overlap.receipt_path = artifact_overlap.write_root.join("unused-receipt.json");
        assert!(
            resolve_write_policy(
                artifact_overlap,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("overlaps protected subject_artifact")
        );

        let mut trust_overlap = fixture.input.clone();
        trust_overlap.write_root = fixture
            .trust_source
            .canonical_path
            .parent()
            .expect("trust record has parent")
            .to_path_buf();
        trust_overlap.receipt_path = trust_overlap.write_root.join("unused-receipt.json");
        assert!(
            resolve_write_policy(
                trust_overlap,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("overlaps protected trust_record")
        );

        let mut checkout_overlap = fixture.input.clone();
        checkout_overlap.write_root = suite_checkout_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("locate suite checkout");
        checkout_overlap.receipt_path = checkout_overlap.write_root.join("unused-receipt.json");
        assert!(
            resolve_write_policy(
                checkout_overlap,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("overlaps protected suite_checkout")
        );

        let expected_overlap =
            resolve_test_write_policy(&fixture, Some(&fixture.subjects[0].actual_output_root));
        assert!(
            expected_overlap
                .unwrap_err()
                .contains("overlaps protected expected_generation")
        );
        assert!(!fixture.input.receipt_path.exists());
        fs::remove_dir_all(&fixture.base).expect("remove protected-root test fixture");
    }

    #[test]
    fn write_policy_rejects_oracle_overlap_and_outputs_outside_write_root() {
        let fixture = test_write_policy_fixture("policy-oracle");
        let oracle = fixture
            .input
            .protected_roots
            .iter()
            .find(|root| root.kind == "oracle")
            .expect("fixture has oracle root")
            .path
            .clone();
        let mut oracle_overlap = fixture.input.clone();
        oracle_overlap.write_root = oracle.clone();
        oracle_overlap.receipt_path = oracle.join("run.json");
        assert!(
            resolve_write_policy(
                oracle_overlap,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("overlaps protected oracle")
        );

        let mut outside_output = fixture.subjects.clone();
        outside_output[0].actual_output_root = oracle.join("captures");
        assert!(
            resolve_write_policy(
                fixture.input.clone(),
                &outside_output,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("actual output root must be strictly beneath")
        );

        let mut receipt_outside = fixture.input.clone();
        receipt_outside.receipt_path = fixture.base.join("receipt-outside.json");
        assert!(
            resolve_write_policy(
                receipt_outside,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("receipt destination must be strictly beneath")
        );

        let mut receipt_inside_capture = fixture.input.clone();
        receipt_inside_capture.receipt_path =
            fixture.subjects[0].actual_output_root.join("receipt.json");
        assert!(
            resolve_write_policy(
                receipt_inside_capture,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("receipt destination overlaps reference actual output root")
        );

        let mut missing_oracle = fixture.input.clone();
        missing_oracle
            .protected_roots
            .retain(|root| root.kind != "oracle");
        assert!(
            resolve_write_policy(
                missing_oracle,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("must include an oracle root")
        );

        let mut missing_subject_source = fixture.input.clone();
        missing_subject_source.protected_roots.retain(|root| {
            root.kind != "subject_source" || root.role.as_deref() != Some("candidate")
        });
        assert!(
            resolve_write_policy(
                missing_subject_source,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("must include subject_source for candidate")
        );

        let mut missing_subject_artifact = fixture.input.clone();
        missing_subject_artifact.protected_roots.retain(|root| {
            root.kind != "subject_artifact" || root.role.as_deref() != Some("candidate")
        });
        assert!(
            resolve_write_policy(
                missing_subject_artifact,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .unwrap_err()
            .contains("must include subject_artifact for candidate")
        );
        fs::remove_dir_all(&fixture.base).expect("remove oracle test fixture");
    }

    #[test]
    fn trust_record_must_be_outside_sources_artifacts_oracles_and_runner_roots() {
        let fixture = test_write_policy_fixture("trust-location");
        let assert_rejected = |path: &Path, expected_root: Option<&Path>, expected_reason: &str| {
            let trust_source = TrustRecordSource {
                canonical_path: fs::canonicalize(path).expect("canonicalize trust fixture"),
                expected_sha256: "a".repeat(64),
            };
            let error = resolve_write_policy(
                fixture.input.clone(),
                &fixture.subjects,
                expected_root,
                &trust_source,
                &fixture.test_binary,
            )
            .unwrap_err();
            assert!(
                error.contains(expected_reason),
                "expected {expected_reason:?}, got {error:?}"
            );
        };

        for root in &fixture.input.protected_roots {
            let trust_path = root.path.join("trust-record-fixture.json");
            fs::write(&trust_path, b"fixture").expect("write nested trust record");
            let expected = format!("outside protected {} root", root.kind);
            assert_rejected(&trust_path, None, &expected);
        }

        let suite_record = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        assert_rejected(&suite_record, None, "outside protected suite_checkout root");

        let expected_root = fixture.base.join("expected");
        fs::create_dir_all(&expected_root).expect("create expected generation root");
        let expected_record = expected_root.join("trust-record-fixture.json");
        fs::write(&expected_record, b"fixture").expect("write expected-root trust record");
        assert_rejected(
            &expected_record,
            Some(&expected_root),
            "outside protected expected_generation root",
        );

        let actual_root = &fixture.subjects[0].actual_output_root;
        let actual_record = actual_root.join("trust-record-fixture.json");
        fs::write(&actual_record, b"fixture").expect("write output-root trust record");
        assert_rejected(&actual_record, None, "outside reference actual output root");

        assert_rejected(
            &fixture.test_binary,
            None,
            "outside protected test_binary_parent root",
        );
        fs::remove_dir_all(&fixture.base).expect("remove trust-location fixture");
    }

    #[cfg(unix)]
    #[test]
    fn receipt_publication_rejects_symlinks_without_touching_the_target() {
        use std::os::unix::fs::symlink;

        let fixture = test_write_policy_fixture("receipt-symlink");
        let sentinel = fixture.base.join("oracle/sentinel.json");
        fs::write(&sentinel, b"protected").expect("write protected sentinel");
        symlink(&sentinel, &fixture.input.receipt_path).expect("link receipt to sentinel");
        let write_root = canonical_output_root(&fixture.input.write_root).unwrap();
        let receipt_path = write_root.join("run.json");
        let result = resolve_test_write_policy(&fixture, None);
        assert!(
            result
                .unwrap_err()
                .contains("already exists; refusing to overwrite")
        );
        assert!(
            publish_receipt_no_replace(&write_root, &receipt_path, b"overwrite",)
                .unwrap_err()
                .contains("already exists; refusing to overwrite")
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"protected");

        let nested = write_root.join("nested");
        symlink(fixture.base.join("oracle"), &nested).expect("link receipt parent to oracle");
        let nested_receipt = nested.join("receipt.json");
        assert!(
            publish_receipt_no_replace(&write_root, &nested_receipt, b"overwrite")
                .unwrap_err()
                .contains("write path contains a symlink")
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"protected");
        assert!(!fixture.base.join("oracle/receipt.json").exists());
        fs::remove_dir_all(&fixture.base).expect("remove symlink test fixture");
    }

    #[test]
    fn receipt_publication_is_single_write_and_preserves_existing_bytes() {
        let fixture = test_write_policy_fixture("receipt-publish");
        let policy = resolve_test_write_policy(&fixture, None).expect("resolve write policy");
        let receipt = test_run_receipt(&policy);
        write_receipt(&receipt, &policy).expect("publish first receipt");
        let first = fs::read(&policy.receipt_path).expect("read published receipt");
        assert!(!first.is_empty());
        assert!(
            write_receipt(&receipt, &policy)
                .unwrap_err()
                .contains("already exists; refusing to overwrite")
        );
        assert_eq!(fs::read(&policy.receipt_path).unwrap(), first);
        fs::remove_dir_all(&fixture.base).expect("remove receipt test fixture");
    }

    #[test]
    fn readiness_rejects_empty_blank_and_absence_only_wait_conditions() {
        let case = registry().unwrap().cases.remove(0);

        let mut empty = case.clone();
        {
            let Step::Checkpoint { wait, .. } = &mut empty.steps[0] else {
                panic!("pilot begins with a checkpoint");
            };
            wait.clear();
        }
        assert!(
            validate_case_contract(&empty)
                .unwrap_err()
                .contains("has no readiness predicates")
        );

        let mut blank = case.clone();
        {
            let Step::Checkpoint { wait, .. } = &mut blank.steps[0] else {
                panic!("pilot begins with a checkpoint");
            };
            wait[0].needle = " \t ".to_string();
        }
        assert!(
            validate_case_contract(&blank)
                .unwrap_err()
                .contains("empty readiness needle")
        );

        let mut absence_only = case;
        {
            let Step::Checkpoint { wait, .. } = &mut absence_only.steps[0] else {
                panic!("pilot begins with a checkpoint");
            };
            for condition in wait.iter_mut() {
                condition.kind = "absent".to_string();
            }
        }
        assert!(
            validate_case_contract(&absence_only)
                .unwrap_err()
                .contains("at least one positive contains readiness predicate")
        );
    }

    #[cfg(unix)]
    #[test]
    fn failed_write_policy_preflight_does_not_create_receipt() {
        use std::os::unix::fs::symlink;

        let fixture = test_write_policy_fixture("policy-preflight-no-write");
        let redirect = fixture.base.join("redirect");
        symlink(&fixture.base.join("oracle"), &redirect).expect("create protected redirect");
        let mut invalid = fixture.input.clone();
        invalid.write_root = redirect;
        invalid.receipt_path = invalid.write_root.join("run.json");
        assert!(
            resolve_write_policy(
                invalid,
                &fixture.subjects,
                None,
                &fixture.trust_source,
                &fixture.test_binary,
            )
            .is_err()
        );
        assert!(!fixture.input.receipt_path.exists());
        assert!(!fixture.base.join("oracle/run.json").exists());
        fs::remove_dir_all(&fixture.base).expect("remove preflight fixture");
    }

    #[test]
    fn external_trust_record_digest_must_match_caller_pin() {
        let actual = "a".repeat(64);
        assert!(verify_external_record_digest(&actual, &actual).is_ok());
        assert!(verify_external_record_digest(&"b".repeat(64), &actual).is_err());
        assert!(verify_external_record_digest("missing", &actual).is_err());
    }

    #[test]
    fn matching_expected_hash_is_integrity_only_without_admission() {
        assert_eq!(
            expected_generation_state(&Ok("a".repeat(64)), false),
            "HASH_MATCH"
        );
        assert_eq!(
            expected_generation_state(&Ok("a".repeat(64)), true),
            "ADMITTED"
        );
        assert_eq!(
            expected_generation_state(&Err("missing".to_string()), false),
            "BLOCKED_MISSING_OR_INVALID"
        );
    }

    fn dimension_status<'a>(checks: &'a [CheckReceipt], dimension: &str) -> Option<&'a str> {
        checks
            .iter()
            .find(|check| check.dimension == dimension)
            .map(|check| check.status.as_str())
    }

    fn test_subject(role: &str, actual_output_root: PathBuf) -> Subject {
        let source_commit = "a".repeat(40);
        let build = BuildFacts {
            package_id: "holla 0.1.0".to_string(),
            target_name: "holla".to_string(),
            features: Vec::new(),
            default_features: true,
            target_triple: "aarch64-apple-darwin".to_string(),
            toolchain: "1.98.1".to_string(),
            profile: "release".to_string(),
        };
        let build_inputs = BuildInputs {
            manifest_sha256: "c".repeat(64),
            lock_sha256: "d".repeat(64),
            mise_config: MiseConfigInput {
                present: false,
                path: None,
                sha256: None,
            },
            cargo_version: "cargo 1.98.1".to_string(),
            rustc_version: "rustc 1.98.1".to_string(),
            host_triple: "aarch64-apple-darwin".to_string(),
            executed_argv: vec![
                "cargo".to_string(),
                "build".to_string(),
                "--release".to_string(),
            ],
        };
        let executable = Executable {
            path: PathBuf::from("/tmp/holla"),
            sha256: "b".repeat(64),
        };
        let mut builder_receipt = BuilderReceipt {
            schema: BUILDER_RECEIPT_SCHEMA.to_string(),
            source_commit: source_commit.clone(),
            package_id: build.package_id.clone(),
            target_name: build.target_name.clone(),
            requested_features: build.features.clone(),
            default_features: build.default_features,
            target_triple: build.target_triple.clone(),
            toolchain: build.toolchain.clone(),
            profile: build.profile.clone(),
            manifest_sha256: build_inputs.manifest_sha256.clone(),
            lock_sha256: build_inputs.lock_sha256.clone(),
            mise_config: build_inputs.mise_config.clone(),
            cargo_version: build_inputs.cargo_version.clone(),
            rustc_version: build_inputs.rustc_version.clone(),
            host_triple: build_inputs.host_triple.clone(),
            executed_argv: build_inputs.executed_argv.clone(),
            executable_path: executable.path.clone(),
            executable_sha256: executable.sha256.clone(),
            sha256: String::new(),
        };
        builder_receipt.sha256 = builder_receipt_digest(&builder_receipt).unwrap();
        Subject {
            role: role.to_string(),
            source_commit,
            build,
            executable,
            actual_output_root,
            build_inputs,
            builder_receipt_sha256: builder_receipt.sha256.clone(),
            builder_receipt,
        }
    }

    fn test_trust_record(subjects: &[Subject], test_binary_digest: &str) -> TrustRecord {
        TrustRecord {
            schema: TRUST_RECORD_SCHEMA.to_string(),
            suite: TrustedSuiteInputs {
                digest: "a".repeat(64),
                case_set_digest: "d".repeat(64),
                profile_digest: "e".repeat(64),
                dependency_lock_sha256: "f".repeat(64),
                review_receipt_sha256: "1".repeat(64),
                test_binary_sha256: vec![test_binary_digest.to_string()],
            },
            subjects: subjects
                .iter()
                .map(|subject| TrustedSubjectInputs {
                    role: subject.role.clone(),
                    source_commit: subject.source_commit.clone(),
                    builder_receipt_sha256: subject.builder_receipt_sha256.clone(),
                })
                .collect(),
            expected_generation: None,
            write_policy_sha256: "7".repeat(64),
        }
    }

    struct TestWritePolicyFixture {
        base: PathBuf,
        input: WritePolicyInput,
        subjects: Vec<Subject>,
        trust_source: TrustRecordSource,
        test_binary: PathBuf,
    }

    fn test_write_policy_fixture(label: &str) -> TestWritePolicyFixture {
        let base = unique_temp_path(label);
        let write_root = base.join("write");
        let trust_root = base.join("trust");
        let oracle_root = base.join("oracle");
        fs::create_dir_all(&write_root).expect("create write root");
        fs::create_dir_all(&trust_root).expect("create trust root");
        fs::create_dir_all(&oracle_root).expect("create oracle root");

        let mut subjects = Vec::new();
        let mut protected_roots = Vec::new();
        for role in ["reference", "candidate"] {
            let source_root = base.join(format!("source-{role}"));
            let artifact_root = base.join(format!("artifact-{role}"));
            let actual_output_root = write_root.join(format!("actual-{role}"));
            fs::create_dir_all(&source_root).expect("create protected source root");
            fs::create_dir_all(&artifact_root).expect("create protected artifact root");
            fs::create_dir_all(&actual_output_root).expect("create actual output root");
            protected_roots.push(ProtectedRoot {
                kind: "subject_source".to_string(),
                role: Some(role.to_string()),
                path: source_root,
            });
            protected_roots.push(ProtectedRoot {
                kind: "subject_artifact".to_string(),
                role: Some(role.to_string()),
                path: artifact_root.clone(),
            });

            let executable_path = artifact_root.join("holla");
            fs::write(&executable_path, format!("{role} executable"))
                .expect("write protected test executable");
            let executable_sha256 = sha256_file(&executable_path).expect("hash test executable");
            let mut subject = test_subject(role, actual_output_root);
            subject.executable = Executable {
                path: executable_path.clone(),
                sha256: executable_sha256.clone(),
            };
            subject.builder_receipt.executable_path = executable_path;
            subject.builder_receipt.executable_sha256 = executable_sha256;
            subject.builder_receipt.sha256 =
                builder_receipt_digest(&subject.builder_receipt).expect("hash build receipt");
            subject.builder_receipt_sha256 = subject.builder_receipt.sha256.clone();
            subjects.push(subject);
        }
        protected_roots.push(ProtectedRoot {
            kind: "oracle".to_string(),
            role: None,
            path: oracle_root,
        });

        let trust_path = trust_root.join("trust-record.json");
        fs::write(&trust_path, b"pinned trust record fixture").expect("write trust record");
        let trust_source = TrustRecordSource {
            canonical_path: fs::canonicalize(&trust_path).expect("canonicalize trust record"),
            expected_sha256: "a".repeat(64),
        };
        TestWritePolicyFixture {
            input: WritePolicyInput {
                write_root: write_root.clone(),
                receipt_path: write_root.join("run.json"),
                protected_roots,
            },
            base,
            subjects,
            trust_source,
            test_binary: std::env::current_exe().expect("locate running unit-test binary"),
        }
    }

    fn resolve_test_write_policy(
        fixture: &TestWritePolicyFixture,
        expected_root: Option<&Path>,
    ) -> Result<WritePolicyReceipt, String> {
        resolve_write_policy(
            fixture.input.clone(),
            &fixture.subjects,
            expected_root,
            &fixture.trust_source,
            &fixture.test_binary,
        )
    }

    fn test_run_receipt(policy: &WritePolicyReceipt) -> RunReceipt {
        let trust_record_path = policy
            .protected_roots
            .iter()
            .find(|root| root.kind == "trust_record")
            .expect("policy protects trust record")
            .path
            .clone();
        RunReceipt {
            schema: RECEIPT_SCHEMA.to_string(),
            run_id: "receipt-publication-test".to_string(),
            source_pair: Vec::new(),
            suite: SuiteIdentity {
                revision: "test".to_string(),
                digest: "1".repeat(64),
                compiled_digest: "2".repeat(64),
                case_set_digest: "3".repeat(64),
                profile_digest: "4".repeat(64),
                test_binary_digest: "5".repeat(64),
                dependency_lock_sha256: "6".repeat(64),
                platform: std::env::consts::OS.to_string(),
            },
            expected_generation: ExpectedGenerationReceipt {
                id: "shared-parity-v1".to_string(),
                expected_sha256: "7".repeat(64),
                actual_sha256: None,
                state: "BLOCKED_MISSING_OR_INVALID".to_string(),
                admission_receipt_sha256: None,
                root: None,
            },
            trust: TrustVerification {
                status: "accepted".to_string(),
                record_path: trust_record_path,
                record_sha256: "8".repeat(64),
                receipt_sha256: "9".repeat(64),
                write_policy_sha256: policy.sha256.clone(),
                reason: "test fixture".to_string(),
            },
            write_policy: policy.clone(),
            checks: Vec::new(),
            artifacts: Vec::new(),
        }
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let suffix = CAPTURE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "termrock-e2e-{label}-{}-{suffix}",
            std::process::id()
        ))
    }
}

pub fn receipt_has_blocking_result(receipt: &RunReceipt) -> bool {
    receipt.checks.iter().any(|check| {
        matches!(
            check.status.as_str(),
            "FAIL" | "ERROR" | "BLOCKED" | "NOT_RUN" | "STALE"
        )
    })
}
