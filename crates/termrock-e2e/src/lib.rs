use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

mod visual;
pub use visual::VisualComparisonEvidence;

pub const REGISTRY_JSON: &str = include_str!("../cases/registry.json");
pub const DEFERRED_JSON: &str = include_str!("../cases/deferred-obligations.json");
pub const PROFILE_JSON: &str = include_str!("../profile.json");
pub const RECEIPT_SCHEMA_V2: &str = "termrock-spec/parity-run-receipt-v2";
pub const RECEIPT_SCHEMA: &str = "termrock-spec/parity-run-receipt-v3";
pub const SUBJECT_SCHEMA: &str = "termrock-spec/parity-subject-manifest-v2";
pub const BUILDER_RECEIPT_SCHEMA: &str = "termrock-spec/parity-subject-build-receipt-v1";
pub const TRUST_RECORD_SCHEMA: &str = "termrock-spec/parity-trust-record-v2";
const ORACLE_TAG_REF: &str = "refs/tags/visual-baseline";
const ORACLE_TAG_OBJECT: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const ORACLE_TAG_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const REFERENCE_BRANCH_REF: &str = "refs/remotes/origin/visual-baseline";
const REFERENCE_BRANCH_MEMBERSHIP: &str = "verified-reachable";
const TRUST_RECORD_PATH_ENV: &str = "TERMROCK_E2E_TRUST_RECORD";
const TRUST_RECORD_SHA_ENV: &str = "TERMROCK_E2E_TRUST_RECORD_SHA256";
const RECEIPT_PATH_ENV: &str = "TERMROCK_E2E_RECEIPT_PATH";
const WRITE_ROOT_ENV: &str = "TERMROCK_E2E_WRITE_ROOT";
const PROTECTED_ROOTS_ENV: &str = "TERMROCK_E2E_PROTECTED_ROOTS";
pub const WRITE_POLICY_SCHEMA: &str = "termrock-spec/parity-write-policy-v2";
pub const COMPILED_SUITE_SHA256: &str = env!("TERMROCK_E2E_COMPILED_SUITE_SHA256");

static CAPTURE_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize)]
pub struct Registry {
    pub schema: String,
    pub suite_revision: String,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WaitCondition {
    pub kind: String,
    pub needle: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectManifest {
    pub schema: String,
    pub run_id: String,
    pub suite_revision: String,
    pub suite_sha256: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub expected_generation: Option<ExpectedGenerationInput>,
    pub build_evidence: SubjectBuildEvidence,
    pub subjects: Vec<Subject>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectBuildEvidence {
    pub source_inputs: EvidenceReference,
    pub build_environment: EvidenceReference,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildEvidenceReceipt {
    pub subject_manifest: EvidenceReference,
    pub source_inputs: EvidenceReference,
    pub build_environment: EvidenceReference,
    pub builder_run: EvidenceReference,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedGenerationInput {
    pub id: String,
    #[serde(default)]
    pub root: Option<PathBuf>,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(deserialize_with = "deserialize_required_option")]
    pub path: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
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
    #[serde(deserialize_with = "deserialize_required_option")]
    pub expected_generation: Option<TrustedExpectedGeneration>,
    pub build_evidence: TrustedBuildEvidence,
    pub write_policy_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedBuildEvidence {
    pub subject_manifest: EvidenceReference,
    pub builder_run: EvidenceReference,
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
#[serde(deny_unknown_fields)]
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
    pub expected_generation: Option<ExpectedGenerationReceipt>,
    pub build_evidence: BuildEvidenceReceipt,
    pub environment: EnvironmentReceipt,
    pub renderer: RendererIdentity,
    pub trust: TrustVerification,
    pub write_policy: WritePolicyReceipt,
    pub checks: Vec<CheckReceipt>,
    pub artifacts: Vec<ArtifactReceipt>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ProtectedRoot {
    pub kind: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub role: Option<String>,
    pub path: PathBuf,
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActualOutputRoot {
    pub role: String,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedDirectory {
    path: PathBuf,
    identity: DirectoryIdentity,
    ancestors: Vec<DirectoryIdentity>,
}

#[derive(Clone, Debug)]
struct ResolvedPath {
    path: PathBuf,
    directory: Option<ResolvedDirectory>,
    nearest_directory: ResolvedDirectory,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WritePolicyReceipt {
    pub schema: String,
    pub write_root: PathBuf,
    pub receipt_path: PathBuf,
    pub subject_cwd: PathBuf,
    pub protected_roots: Vec<ProtectedRoot>,
    pub actual_output_roots: Vec<ActualOutputRoot>,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WritePolicyPreflightReceipt {
    pub schema: String,
    pub state: String,
    pub case_id: String,
    pub suite: SuiteIdentity,
    pub test_executable: TestExecutableIdentity,
    pub expected_generation: Option<ExpectedGenerationInput>,
    pub build_evidence: BuildEvidenceReceipt,
    pub source_pair: Vec<SubjectIdentity>,
    pub renderer: RendererIdentity,
    pub trust_record: PreparedTrustRecord,
    pub write_policy: WritePolicyReceipt,
}

#[derive(Clone, Debug, Serialize)]
pub struct TestExecutableIdentity {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PreparedTrustRecord {
    pub path: PathBuf,
    pub state: String,
}

#[derive(Clone, Debug)]
struct TrustRecordSource {
    canonical_path: PathBuf,
    expected_sha256: String,
}

#[derive(Clone, Debug)]
struct LoadedEvidenceContext {
    manifest_path: PathBuf,
    manifest_sha256: String,
    source_inputs_path: PathBuf,
    source_inputs_sha256: String,
    build_environment_path: PathBuf,
    build_environment_sha256: String,
    builder_run_path: PathBuf,
    builder_run_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceInputsEvidence {
    schema: String,
    run_id: String,
    qualification_status: String,
    qualification_reason: String,
    git_object_replacement_policy: String,
    oracle_lineage: OracleLineage,
    reference_branch: ReferenceBranchEvidence,
    recipe: String,
    excluded_roots: Vec<String>,
    path_blob_digest_algorithm: String,
    sources: BTreeMap<String, SourceSnapshotEvidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct OracleLineage {
    tag_ref: String,
    tag_object: String,
    tag_commit: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReferenceBranchEvidence {
    r#ref: String,
    oid: String,
    selected_commit: String,
    membership: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceSnapshotEvidence {
    role: String,
    path: String,
    source_commit: String,
    tree_oid: String,
    git_object_replacement_policy: String,
    recipe: String,
    archive_argv: Vec<String>,
    git_ls_tree_stdout_sha256: String,
    excluded_roots: Vec<String>,
    path_blob_digest_algorithm: String,
    tracked_path_blob_map_sha256: String,
    tracked_file_count: u64,
    included_path_blob_map_sha256: String,
    included_file_count: u64,
    excluded_path_blob_map_sha256: String,
    excluded_file_count: u64,
    archive_sha256: String,
    archive_member_count: u64,
    read_only: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BuildEnvironmentEvidence {
    schema: String,
    run_id: String,
    source_input_evidence: EvidenceReference,
    source_commits: BTreeMap<String, String>,
    reference_branch: ReferenceBranchEvidence,
    oracle_lineage: OracleLineage,
    cargo_config_sha256: BTreeMap<String, BTreeMap<String, String>>,
    external_cargo_config_policy: String,
    rustup: RustupEvidence,
    toolchain_binaries: BTreeMap<String, ToolchainBinaryEvidence>,
    cargo_version: String,
    rustc_version: String,
    host_triple: String,
    target_triple: String,
    environment: BTreeMap<String, BuildEnvironmentFacts>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RustupEvidence {
    path: String,
    sha256: String,
    home: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ToolchainBinaryEvidence {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BuildEnvironmentFacts {
    home: String,
    cargo_home: String,
    cargo_home_config: String,
    cache_home: String,
    cache_links: BTreeMap<String, String>,
    rustup_home: String,
    tmpdir: String,
    path: String,
    platform_inputs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuilderRunEvidence {
    schema: String,
    state: String,
    qualification: QualificationEvidence,
    run_id: String,
    app: String,
    subject_schema_sha256: String,
    reference_tag_object: String,
    reference_tag_commit: String,
    reference_branch_ref: String,
    reference_commit: String,
    candidate_commit: String,
    target_triple: String,
    toolchain: String,
    rustup_path: String,
    rustup_sha256: String,
    rustup_home: String,
    toolchain_binaries: BTreeMap<String, ToolchainBinaryEvidence>,
    git_identity: GitIdentityEvidence,
    reference_branch_oid: String,
    git_object_replacement_policy: String,
    cargo_version: String,
    rustc_version: String,
    host_triple: String,
    snapshots: BTreeMap<String, SnapshotReferenceEvidence>,
    snapshot_materialization: BTreeMap<String, SourceSnapshotEvidence>,
    cargo_config_sha256: BTreeMap<String, BTreeMap<String, String>>,
    source_input_evidence: EvidenceReference,
    build_environment: BTreeMap<String, BuildEnvironmentFacts>,
    build_environment_evidence: EvidenceReference,
    metadata_targets: BTreeMap<String, BTreeMap<String, MetadataTargetEvidence>>,
    subject_manifest: String,
    subject_manifest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct QualificationEvidence {
    status: String,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GitIdentityEvidence {
    reference_tag_object: String,
    reference_tag_commit: String,
    reference_branch_ref: String,
    reference_branch_oid: String,
    reference_commit: String,
    candidate_commit: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SnapshotReferenceEvidence {
    commit: String,
    path: String,
    read_only: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct MetadataTargetEvidence {
    package_id: String,
    package_name: String,
    target_name: String,
    manifest_path: String,
    source_path: String,
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
pub struct EnvironmentReceipt {
    pub cleared: bool,
    pub cwd: PathBuf,
    pub common: BTreeMap<String, String>,
    pub case_allowlist: Vec<String>,
    pub case_values: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RendererIdentity {
    pub schema: String,
    pub hash: String,
    pub name: String,
    pub renderer_version: u32,
    pub font_px: f32,
    pub cell_w: u32,
    pub cell_h: u32,
    pub pad: u32,
    pub scale: u32,
    pub default_fg: [u8; 3],
    pub default_bg: [u8; 3],
    pub indexed_palette: String,
    pub face_hashes: [String; 4],
    pub fallback_order: Vec<RendererFallbackIdentity>,
    pub cursor_policy: String,
    pub blink_phase: String,
    pub missing_glyph_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RendererFallbackIdentity {
    pub description: String,
    pub sha256: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_comparison: Option<VisualComparisonEvidence>,
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
#[serde(deny_unknown_fields)]
struct Profile {
    schema: String,
    id: String,
    tool: String,
    tool_revision: String,
    screen_profile: String,
    geometry_policy: String,
    readiness_timeout_ms: u64,
    input_pacing_ms: u64,
    terminal: TerminalProfile,
    environment: CommonEnvironment,
    renderer: RendererProfile,
    exports: Vec<String>,
    companions: Vec<String>,
    visual_acceptance: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalProfile {
    term: String,
    colorterm: String,
    locale: String,
    shell: String,
    cwd_policy: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommonEnvironment {
    clear: bool,
    case_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RendererProfile {
    id: String,
    implementation: String,
    expected_hash: String,
    renderer_version: u32,
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
    Ok(load_subject_evidence()?.manifest)
}

struct LoadedSubjectEvidence {
    manifest: SubjectManifest,
    context: LoadedEvidenceContext,
}

fn load_subject_evidence() -> Result<LoadedSubjectEvidence, String> {
    let path = std::env::var_os("TERMROCK_E2E_SUBJECT_MANIFEST")
        .map(PathBuf::from)
        .ok_or_else(|| "TERMROCK_E2E_SUBJECT_MANIFEST is required for a PTY run".to_string())?;
    let canonical_path =
        canonical_existing_regular_file_without_symlinks(&path, "subject manifest")?;
    let bytes = fs::read(&canonical_path)
        .map_err(|error| format!("read {}: {error}", canonical_path.display()))?;
    let manifest: SubjectManifest = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "parse subject manifest {}: {error}",
            canonical_path.display()
        )
    })?;
    validate_subject_manifest(&manifest)?;
    let run_root = canonical_path
        .parent()
        .ok_or_else(|| "subject manifest has no parent directory".to_string())?
        .to_path_buf();
    let source_inputs_path = resolve_fixed_evidence_file(
        &run_root,
        &manifest.build_evidence.source_inputs,
        "source-inputs.json",
    )?;
    let build_environment_path = resolve_fixed_evidence_file(
        &run_root,
        &manifest.build_evidence.build_environment,
        "build-environment.json",
    )?;
    let builder_run_path = canonical_existing_regular_file_without_symlinks(
        &run_root.join("run.json"),
        "builder run record",
    )?;
    let source_inputs_bytes = fs::read(&source_inputs_path).map_err(|error| {
        format!(
            "read source inputs {}: {error}",
            source_inputs_path.display()
        )
    })?;
    let build_environment_bytes = fs::read(&build_environment_path).map_err(|error| {
        format!(
            "read build environment {}: {error}",
            build_environment_path.display()
        )
    })?;
    let builder_run_bytes = fs::read(&builder_run_path)
        .map_err(|error| format!("read builder run {}: {error}", builder_run_path.display()))?;
    let source_inputs_sha256 = sha256_bytes(&source_inputs_bytes);
    let build_environment_sha256 = sha256_bytes(&build_environment_bytes);
    let builder_run_sha256 = sha256_bytes(&builder_run_bytes);
    verify_evidence_digest(
        "source-inputs.json",
        &manifest.build_evidence.source_inputs.sha256,
        &source_inputs_sha256,
    )?;
    verify_evidence_digest(
        "build-environment.json",
        &manifest.build_evidence.build_environment.sha256,
        &build_environment_sha256,
    )?;
    let source_inputs: SourceInputsEvidence = serde_json::from_slice(&source_inputs_bytes)
        .map_err(|error| format!("parse source-inputs.json: {error}"))?;
    let build_environment: BuildEnvironmentEvidence =
        serde_json::from_slice(&build_environment_bytes)
            .map_err(|error| format!("parse build-environment.json: {error}"))?;
    let builder_run: BuilderRunEvidence = serde_json::from_slice(&builder_run_bytes)
        .map_err(|error| format!("parse builder run.json: {error}"))?;
    let manifest_sha256 = sha256_bytes(&bytes);
    validate_build_evidence_chain(
        &manifest,
        &canonical_path,
        &manifest_sha256,
        &source_inputs,
        &source_inputs_path,
        &source_inputs_sha256,
        &build_environment,
        &build_environment_path,
        &build_environment_sha256,
        &builder_run,
        &builder_run_path,
        &builder_run_sha256,
    )?;
    Ok(LoadedSubjectEvidence {
        manifest,
        context: LoadedEvidenceContext {
            manifest_path: canonical_path,
            manifest_sha256,
            source_inputs_path,
            source_inputs_sha256,
            build_environment_path,
            build_environment_sha256,
            builder_run_path,
            builder_run_sha256,
        },
    })
}

fn resolve_fixed_evidence_file(
    run_root: &Path,
    reference: &EvidenceReference,
    basename: &str,
) -> Result<PathBuf, String> {
    require_sha256(&format!("{basename} reference SHA-256"), &reference.sha256)?;
    if reference.path != basename {
        return Err(format!(
            "build evidence path must be the fixed relative basename {basename:?}, got {:?}",
            reference.path
        ));
    }
    let path =
        canonical_existing_regular_file_without_symlinks(&run_root.join(basename), basename)?;
    if path.parent() != Some(run_root) {
        return Err(format!(
            "{basename} resolved outside the manifest run directory"
        ));
    }
    Ok(path)
}

fn canonical_existing_regular_file_without_symlinks(
    path: &Path,
    name: &str,
) -> Result<PathBuf, String> {
    validate_absolute_normal_path(path, name)?;
    let mut current = PathBuf::new();
    let components = path.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        match component {
            Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(part) => {
                current.push(part);
                let metadata = fs::symlink_metadata(&current).map_err(|error| {
                    format!("inspect {name} path {}: {error}", current.display())
                })?;
                if metadata.file_type().is_symlink() {
                    return Err(format!(
                        "{name} path contains a symlink: {}",
                        current.display()
                    ));
                }
                let is_last = index + 1 == components.len();
                if is_last && !metadata.is_file() {
                    return Err(format!(
                        "{name} is not a regular file: {}",
                        current.display()
                    ));
                }
                if !is_last && !metadata.is_dir() {
                    return Err(format!(
                        "{name} ancestor is not a directory: {}",
                        current.display()
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "{name} path has a non-normal component: {}",
                    path.display()
                ));
            }
        }
    }
    fs::canonicalize(path)
        .map_err(|error| format!("canonicalize {name} {}: {error}", path.display()))
}

fn canonical_existing_directory_without_symlinks(
    path: &Path,
    name: &str,
) -> Result<PathBuf, String> {
    validate_absolute_normal_path(path, name)?;
    let mut current = PathBuf::new();
    let components = path.components().collect::<Vec<_>>();
    for component in components {
        match component {
            Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(part) => {
                current.push(part);
                let metadata = fs::symlink_metadata(&current).map_err(|error| {
                    format!("inspect {name} path {}: {error}", current.display())
                })?;
                if metadata.file_type().is_symlink() {
                    return Err(format!(
                        "{name} path contains a symlink: {}",
                        current.display()
                    ));
                }
                if !metadata.is_dir() {
                    return Err(format!(
                        "{name} path component is not a directory: {}",
                        current.display()
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "{name} path has a non-normal component: {}",
                    path.display()
                ));
            }
        }
    }
    fs::canonicalize(path)
        .map_err(|error| format!("canonicalize {name} {}: {error}", path.display()))
}

fn resolve_existing_directory_without_symlinks(
    path: &Path,
    name: &str,
) -> Result<ResolvedDirectory, String> {
    let canonical_path = canonical_existing_directory_without_symlinks(path, name)?;
    resolve_canonical_directory(&canonical_path, name)
}

fn resolve_existing_directory(path: &Path, name: &str) -> Result<ResolvedDirectory, String> {
    let canonical_path = canonical_existing_directory(path, name)?;
    resolve_canonical_directory(&canonical_path, name)
}

fn resolve_canonical_directory(path: &Path, name: &str) -> Result<ResolvedDirectory, String> {
    validate_absolute_normal_path(path, name)?;
    let mut current = PathBuf::new();
    let mut ancestors = Vec::new();
    for component in path.components() {
        match component {
            Component::RootDir => {
                current.push(component.as_os_str());
                let metadata = fs::symlink_metadata(&current).map_err(|error| {
                    format!("inspect {name} directory {}: {error}", current.display())
                })?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(format!(
                        "{name} path component is not a real directory: {}",
                        current.display()
                    ));
                }
                ancestors.push(directory_identity(&metadata, &current)?);
            }
            Component::Normal(part) => {
                current.push(part);
                let metadata = fs::symlink_metadata(&current).map_err(|error| {
                    format!("inspect {name} directory {}: {error}", current.display())
                })?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(format!(
                        "{name} path component is not a real directory: {}",
                        current.display()
                    ));
                }
                ancestors.push(directory_identity(&metadata, &current)?);
            }
            _ => {
                return Err(format!(
                    "{name} path has a non-normal component: {}",
                    path.display()
                ));
            }
        }
    }
    let identity = ancestors
        .last()
        .copied()
        .ok_or_else(|| format!("{name} path has no directory identity: {}", path.display()))?;
    Ok(ResolvedDirectory {
        path: path.to_path_buf(),
        identity,
        ancestors,
    })
}

#[cfg(unix)]
fn directory_identity(metadata: &fs::Metadata, path: &Path) -> Result<DirectoryIdentity, String> {
    use std::os::unix::fs::MetadataExt;

    if !metadata.is_dir() {
        return Err(format!(
            "filesystem identity requested for non-directory {}",
            path.display()
        ));
    }
    Ok(DirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(not(unix))]
fn directory_identity(_metadata: &fs::Metadata, path: &Path) -> Result<DirectoryIdentity, String> {
    Err(format!(
        "physical directory identity is unsupported on this target: {}",
        path.display()
    ))
}

fn resolve_path_for_overlap(path: &Path, name: &str) -> Result<ResolvedPath, String> {
    let canonical_path = match fs::symlink_metadata(path) {
        Ok(_) => fs::canonicalize(path)
            .map_err(|error| format!("canonicalize {name} {}: {error}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => canonical_output_root(path)?,
        Err(error) => {
            return Err(format!("inspect {name} {}: {error}", path.display()));
        }
    };
    let metadata = match fs::symlink_metadata(&canonical_path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "inspect canonical {name} {}: {error}",
                canonical_path.display()
            ));
        }
    };
    if metadata.as_ref().is_some_and(fs::Metadata::is_dir) {
        let directory = resolve_existing_directory_without_symlinks(&canonical_path, name)?;
        return Ok(ResolvedPath {
            path: canonical_path,
            directory: Some(directory.clone()),
            nearest_directory: directory,
        });
    }

    let nearest_directory = if metadata.is_some() {
        let parent = canonical_path
            .parent()
            .ok_or_else(|| format!("{name} path has no parent: {}", canonical_path.display()))?;
        resolve_existing_directory_without_symlinks(parent, name)?
    } else {
        resolve_nearest_existing_directory(&canonical_path, name)?
    };
    Ok(ResolvedPath {
        path: canonical_path,
        directory: None,
        nearest_directory,
    })
}

fn resolve_nearest_existing_directory(
    path: &Path,
    name: &str,
) -> Result<ResolvedDirectory, String> {
    let mut ancestor = path;
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                return resolve_existing_directory_without_symlinks(ancestor, name);
            }
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "{name} path contains a symlink: {}",
                    ancestor.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "inspect {name} ancestor {}: {error}",
                    ancestor.display()
                ));
            }
        }
        ancestor = ancestor.parent().ok_or_else(|| {
            format!(
                "{name} path has no existing directory ancestor: {}",
                path.display()
            )
        })?;
    }
}

fn physical_directories_overlap(left: &ResolvedDirectory, right: &ResolvedDirectory) -> bool {
    paths_overlap(&left.path, &right.path)
        || left.ancestors.contains(&right.identity)
        || right.ancestors.contains(&left.identity)
}

fn validate_external_trust_parent_relation(
    trust_parent: &ResolvedDirectory,
    protected_root: &ResolvedDirectory,
    protected_kind: &str,
    protected_path: &Path,
) -> Result<(), String> {
    if trust_parent.identity == protected_root.identity
        || trust_parent.ancestors.contains(&protected_root.identity)
    {
        return Err(format!(
            "external trust record parent must be outside protected {protected_kind} root {}",
            protected_path.display()
        ));
    }

    if protected_root.ancestors.contains(&trust_parent.identity) {
        return Ok(());
    }

    if paths_overlap(&trust_parent.path, &protected_root.path) {
        return Err(format!(
            "cannot establish physical separation between external trust record parent {} and protected {protected_kind} root {}",
            trust_parent.path.display(),
            protected_path.display()
        ));
    }

    Ok(())
}

fn directory_contains(parent: &ResolvedDirectory, child: &ResolvedDirectory) -> bool {
    child.path.starts_with(&parent.path) || child.ancestors.contains(&parent.identity)
}

fn directory_overlaps_path(directory: &ResolvedDirectory, path: &ResolvedPath) -> bool {
    if paths_overlap(&directory.path, &path.path) {
        return true;
    }
    if let Some(path_directory) = &path.directory {
        return physical_directories_overlap(directory, path_directory);
    }
    path.nearest_directory
        .ancestors
        .contains(&directory.identity)
}

fn physical_paths_overlap(left: &ResolvedPath, right: &ResolvedPath) -> bool {
    if paths_overlap(&left.path, &right.path) {
        return true;
    }
    match (&left.directory, &right.directory) {
        (Some(left_directory), Some(right_directory)) => {
            physical_directories_overlap(left_directory, right_directory)
        }
        (Some(left_directory), None) => directory_overlaps_path(left_directory, right),
        (None, Some(right_directory)) => directory_overlaps_path(right_directory, left),
        (None, None) => false,
    }
}

fn resolved_path_from_directory(directory: &ResolvedDirectory) -> ResolvedPath {
    ResolvedPath {
        path: directory.path.clone(),
        directory: Some(directory.clone()),
        nearest_directory: directory.clone(),
    }
}

fn path_is_within_directory(directory: &ResolvedDirectory, path: &ResolvedPath) -> bool {
    if let Some(path_directory) = &path.directory {
        directory_contains(directory, path_directory)
    } else {
        path.path.starts_with(&directory.path)
            || path
                .nearest_directory
                .ancestors
                .contains(&directory.identity)
    }
}

fn verify_evidence_digest(name: &str, expected: &str, actual: &str) -> Result<(), String> {
    require_sha256(&format!("{name} expected SHA-256"), expected)?;
    if expected != actual {
        return Err(format!(
            "{name} SHA-256 mismatch: expected {expected}, got {actual}"
        ));
    }
    Ok(())
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
    for (reference, basename) in [
        (&manifest.build_evidence.source_inputs, "source-inputs.json"),
        (
            &manifest.build_evidence.build_environment,
            "build-environment.json",
        ),
    ] {
        if reference.path != basename {
            return Err(format!(
                "build evidence reference must use fixed basename {basename:?}"
            ));
        }
        require_sha256(&format!("{basename} reference sha256"), &reference.sha256)?;
    }
    if let Some(expected) = &manifest.expected_generation {
        if expected.id.trim().is_empty() {
            return Err("expected-generation ID must not be empty".to_string());
        }
        require_sha256("expected_generation.sha256", &expected.sha256)?;
        if let Some(root) = &expected.root {
            validate_absolute_normal_path(root, "expected-generation root")?;
        }
    }
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
        manifest
            .expected_generation
            .as_ref()
            .and_then(|expected| expected.root.as_deref()),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_build_evidence_chain(
    manifest: &SubjectManifest,
    manifest_path: &Path,
    manifest_sha256: &str,
    source_inputs: &SourceInputsEvidence,
    source_inputs_path: &Path,
    source_inputs_sha256: &str,
    build_environment: &BuildEnvironmentEvidence,
    build_environment_path: &Path,
    build_environment_sha256: &str,
    builder_run: &BuilderRunEvidence,
    builder_run_path: &Path,
    builder_run_sha256: &str,
) -> Result<(), String> {
    let run_root = manifest_path
        .parent()
        .ok_or_else(|| "subject manifest has no parent directory".to_string())?;
    if source_inputs.schema != "termrock-spec/parity-subject-source-inputs-v1"
        || build_environment.schema != "termrock-spec/parity-subject-build-environment-v1"
        || builder_run.schema != "termrock-spec/parity-subject-build-evidence-v1"
    {
        return Err("builder evidence sidecar schema is unsupported".to_string());
    }
    for (name, oid) in [
        (
            "oracle tag object",
            source_inputs.oracle_lineage.tag_object.as_str(),
        ),
        (
            "oracle tag commit",
            source_inputs.oracle_lineage.tag_commit.as_str(),
        ),
        (
            "reference branch object",
            source_inputs.reference_branch.oid.as_str(),
        ),
        (
            "selected reference commit",
            source_inputs.reference_branch.selected_commit.as_str(),
        ),
    ] {
        require_git_oid(name, oid)?;
    }
    if source_inputs.oracle_lineage.tag_ref != ORACLE_TAG_REF
        || source_inputs.oracle_lineage.tag_object != ORACLE_TAG_OBJECT
        || source_inputs.oracle_lineage.tag_commit != ORACLE_TAG_COMMIT
    {
        return Err(
            "source-inputs oracle lineage does not match the pinned tag identity".to_string(),
        );
    }
    if source_inputs.reference_branch.r#ref != REFERENCE_BRANCH_REF
        || source_inputs.reference_branch.membership != REFERENCE_BRANCH_MEMBERSHIP
        || source_inputs.reference_branch.oid != source_inputs.reference_branch.selected_commit
    {
        return Err(
            "source-inputs reference branch does not match the fixed reachable branch identity"
                .to_string(),
        );
    }
    if source_inputs.recipe.trim().is_empty()
        || source_inputs.path_blob_digest_algorithm.trim().is_empty()
    {
        return Err("source-inputs recipe fields must be nonempty".to_string());
    }
    if source_inputs.run_id != manifest.run_id
        || build_environment.run_id != manifest.run_id
        || builder_run.run_id != manifest.run_id
    {
        return Err("builder evidence run IDs do not match the subject manifest".to_string());
    }
    if builder_run.state != "complete" {
        return Err(format!(
            "builder run state must be complete, got {:?}",
            builder_run.state
        ));
    }
    if builder_run.qualification.status != "blocked"
        || source_inputs.qualification_status != "blocked"
    {
        return Err(
            "builder/source qualification must remain blocked until external admission".to_string(),
        );
    }
    if builder_run.qualification.reason.trim().is_empty()
        || source_inputs.qualification_reason.trim().is_empty()
    {
        return Err("builder/source qualification reasons must be recorded".to_string());
    }
    if builder_run.app.trim().is_empty()
        || builder_run.target_triple.trim().is_empty()
        || builder_run.toolchain.trim().is_empty()
    {
        return Err("builder run app, target triple, and toolchain must be recorded".to_string());
    }
    for (name, oid) in [
        (
            "builder run reference tag object",
            builder_run.reference_tag_object.as_str(),
        ),
        (
            "builder run reference tag commit",
            builder_run.reference_tag_commit.as_str(),
        ),
        (
            "builder run reference branch object",
            builder_run.reference_branch_oid.as_str(),
        ),
        (
            "builder run reference commit",
            builder_run.reference_commit.as_str(),
        ),
        (
            "builder run candidate commit",
            builder_run.candidate_commit.as_str(),
        ),
        (
            "Git identity reference tag object",
            builder_run.git_identity.reference_tag_object.as_str(),
        ),
        (
            "Git identity reference tag commit",
            builder_run.git_identity.reference_tag_commit.as_str(),
        ),
        (
            "Git identity reference branch object",
            builder_run.git_identity.reference_branch_oid.as_str(),
        ),
        (
            "Git identity reference commit",
            builder_run.git_identity.reference_commit.as_str(),
        ),
        (
            "Git identity candidate commit",
            builder_run.git_identity.candidate_commit.as_str(),
        ),
    ] {
        require_git_oid(name, oid)?;
    }
    let manifest_path_text = manifest_path.display().to_string();
    if builder_run.subject_manifest != manifest_path_text
        || builder_run.subject_manifest_sha256 != manifest_sha256
    {
        return Err(
            "final builder run does not bind the raw subject manifest path and digest".to_string(),
        );
    }
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_schema_digest =
        sha256_file(&package_root.join("schemas/subject-manifest-v2.schema.json"))?;
    if builder_run.subject_schema_sha256 != manifest_schema_digest {
        return Err(
            "builder run subject-manifest schema digest does not match v2 suite schema".to_string(),
        );
    }
    let role_commits = manifest
        .subjects
        .iter()
        .map(|subject| (subject.role.as_str(), subject.source_commit.as_str()))
        .collect::<BTreeMap<_, _>>();
    if builder_run.reference_commit != role_commits.get("reference").copied().unwrap_or_default()
        || builder_run.candidate_commit
            != role_commits.get("candidate").copied().unwrap_or_default()
        || builder_run.git_identity.reference_commit != builder_run.reference_commit
        || builder_run.git_identity.candidate_commit != builder_run.candidate_commit
    {
        return Err("builder final run commit pair does not match subject manifest".to_string());
    }
    for (name, digest) in [
        ("manifest raw SHA-256", manifest_sha256),
        ("source-inputs raw SHA-256", source_inputs_sha256),
        ("build-environment raw SHA-256", build_environment_sha256),
        ("builder run raw SHA-256", builder_run_sha256),
    ] {
        require_sha256(name, digest)?;
    }
    require_sha256(
        "builder run subject schema SHA-256",
        &builder_run.subject_schema_sha256,
    )?;
    require_sha256("builder run rustup SHA-256", &builder_run.rustup_sha256)?;

    let source_inputs_ref = EvidenceReference {
        path: source_inputs_path.display().to_string(),
        sha256: source_inputs_sha256.to_string(),
    };
    let build_environment_ref = EvidenceReference {
        path: build_environment_path.display().to_string(),
        sha256: build_environment_sha256.to_string(),
    };
    if builder_run.source_input_evidence != source_inputs_ref
        || build_environment.source_input_evidence != source_inputs_ref
        || builder_run.build_environment_evidence != build_environment_ref
    {
        return Err(
            "builder run and build-environment sidecar do not bind the manifest sidecars"
                .to_string(),
        );
    }
    if build_environment.source_commits.len() != 2
        || source_inputs.sources.len() != 2
        || builder_run.snapshots.len() != 2
        || builder_run.snapshot_materialization.len() != 2
        || build_environment.environment.len() != 2
        || builder_run.build_environment.len() != 2
    {
        return Err(
            "builder evidence must contain exactly the reference/candidate pair".to_string(),
        );
    }
    if builder_run.snapshot_materialization != source_inputs.sources
        || builder_run.build_environment != build_environment.environment
        || builder_run.cargo_config_sha256 != build_environment.cargo_config_sha256
    {
        return Err(
            "builder run duplicates source/environment sidecars with different facts".to_string(),
        );
    }
    if builder_run.reference_tag_object != source_inputs.oracle_lineage.tag_object
        || builder_run.reference_tag_commit != source_inputs.oracle_lineage.tag_commit
        || builder_run.reference_branch_ref != source_inputs.reference_branch.r#ref
        || builder_run.reference_branch_oid != source_inputs.reference_branch.oid
        || builder_run.git_identity.reference_tag_object != source_inputs.oracle_lineage.tag_object
        || builder_run.git_identity.reference_tag_commit != source_inputs.oracle_lineage.tag_commit
        || builder_run.git_identity.reference_branch_ref != source_inputs.reference_branch.r#ref
        || builder_run.git_identity.reference_branch_oid != source_inputs.reference_branch.oid
        || builder_run.git_identity.reference_commit
            != source_inputs.reference_branch.selected_commit
        || builder_run.git_identity.candidate_commit != builder_run.candidate_commit
        || build_environment.reference_branch != source_inputs.reference_branch
        || build_environment.oracle_lineage != source_inputs.oracle_lineage
    {
        return Err("builder run/source evidence lineage fields disagree".to_string());
    }

    for subject in &manifest.subjects {
        let role = subject.role.as_str();
        let snapshot = source_inputs
            .sources
            .get(role)
            .ok_or_else(|| format!("source-inputs sidecar lacks {role} snapshot"))?;
        let run_snapshot = builder_run
            .snapshots
            .get(role)
            .ok_or_else(|| format!("builder run lacks {role} snapshot"))?;
        let materialized = builder_run
            .snapshot_materialization
            .get(role)
            .ok_or_else(|| format!("builder run lacks {role} materialization"))?;
        let source_commit = build_environment
            .source_commits
            .get(role)
            .ok_or_else(|| format!("build environment lacks {role} source commit"))?;
        let environment = build_environment
            .environment
            .get(role)
            .ok_or_else(|| format!("build environment lacks {role} environment"))?;
        let run_environment = builder_run
            .build_environment
            .get(role)
            .ok_or_else(|| format!("builder run lacks {role} environment"))?;
        if snapshot.role != role
            || snapshot.source_commit != subject.source_commit
            || source_commit != &subject.source_commit
            || run_snapshot.commit != subject.source_commit
            || run_snapshot.path != snapshot.path
            || materialized.role != role
            || materialized.source_commit != subject.source_commit
            || !materialized.read_only
            || !run_snapshot.read_only
            || environment != run_environment
            || snapshot.git_object_replacement_policy != source_inputs.git_object_replacement_policy
            || snapshot.recipe != source_inputs.recipe
            || snapshot.excluded_roots != source_inputs.excluded_roots
            || snapshot.path_blob_digest_algorithm != source_inputs.path_blob_digest_algorithm
            || build_environment.source_commits.get(role) != Some(&snapshot.source_commit)
        {
            return Err(format!(
                "builder source/environment identity disagrees for {role}"
            ));
        }
        require_git_oid(&format!("{role} source tree OID"), &snapshot.tree_oid)?;
        require_git_oid(&format!("{role} source commit"), &snapshot.source_commit)?;
        if snapshot.archive_argv.is_empty() {
            return Err(format!(
                "{role} source snapshot has an empty archive command"
            ));
        }
        for (name, digest) in [
            (
                "git ls-tree output",
                snapshot.git_ls_tree_stdout_sha256.as_str(),
            ),
            (
                "tracked path/blob map",
                snapshot.tracked_path_blob_map_sha256.as_str(),
            ),
            (
                "included path/blob map",
                snapshot.included_path_blob_map_sha256.as_str(),
            ),
            (
                "excluded path/blob map",
                snapshot.excluded_path_blob_map_sha256.as_str(),
            ),
            ("source archive", snapshot.archive_sha256.as_str()),
        ] {
            require_sha256(&format!("{role} {name} SHA-256"), digest)?;
        }
        let snapshot_path = canonical_existing_directory_without_symlinks(
            Path::new(&snapshot.path),
            "source snapshot",
        )?;
        if !snapshot_path.starts_with(run_root) {
            return Err(format!(
                "{role} source snapshot is outside the builder run root"
            ));
        }
        let target = builder_run
            .metadata_targets
            .get(role)
            .and_then(|targets| targets.get(&builder_run.app))
            .ok_or_else(|| {
                format!(
                    "builder run lacks {} metadata target for {role}",
                    builder_run.app
                )
            })?;
        if target.package_name.trim().is_empty()
            || target.target_name != subject.build.target_name
            || target.package_id != subject.build.package_id
            || subject.build.target_triple != builder_run.target_triple
            || subject.build.toolchain != builder_run.toolchain
            || target.source_path.is_empty()
            || target.manifest_path.is_empty()
        {
            return Err(format!(
                "builder metadata/build identity disagrees for {role}"
            ));
        }
        let target_source = Path::new(&target.source_path);
        let target_manifest = Path::new(&target.manifest_path);
        if !target_source.is_absolute()
            || !target_manifest.is_absolute()
            || !target_source.starts_with(&snapshot_path)
            || !target_manifest.starts_with(&snapshot_path)
        {
            return Err(format!(
                "{role} Cargo target paths escape the immutable source snapshot"
            ));
        }
        if environment.rustup_home != builder_run.rustup_home
            || environment.cargo_home_config != "absent"
        {
            return Err(format!(
                "{role} build environment is inconsistent with run facts"
            ));
        }
    }
    for (role, environment) in &build_environment.environment {
        for (name, path) in [
            ("HOME", environment.home.as_str()),
            ("CARGO_HOME", environment.cargo_home.as_str()),
            ("Cargo cache home", environment.cache_home.as_str()),
            ("RUSTUP_HOME", environment.rustup_home.as_str()),
            ("TMPDIR", environment.tmpdir.as_str()),
        ] {
            if !Path::new(path).is_absolute() {
                return Err(format!(
                    "{role} build environment {name} path must be absolute"
                ));
            }
        }
        for path in environment.cache_links.values() {
            if !Path::new(path).is_absolute() {
                return Err(format!("{role} Cargo cache link target must be absolute"));
            }
        }
    }
    require_sha256("builder run rustup SHA-256", &builder_run.rustup_sha256)?;
    if build_environment.rustup.path != builder_run.rustup_path
        || build_environment.rustup.sha256 != builder_run.rustup_sha256
        || build_environment.rustup.home != builder_run.rustup_home
        || build_environment.cargo_version != builder_run.cargo_version
        || build_environment.rustc_version != builder_run.rustc_version
        || build_environment.host_triple != builder_run.host_triple
        || build_environment.target_triple != builder_run.target_triple
    {
        return Err(
            "build-environment toolchain facts disagree with final builder run".to_string(),
        );
    }
    require_git_oid(
        "build-environment reference branch object",
        &build_environment.reference_branch.oid,
    )?;
    require_sha256(
        "build-environment rustup SHA-256",
        &build_environment.rustup.sha256,
    )?;
    if !Path::new(&build_environment.rustup.path).is_absolute()
        || !Path::new(&builder_run.rustup_path).is_absolute()
        || !Path::new(&build_environment.rustup.home).is_absolute()
    {
        return Err("build-environment Rust tool paths must be absolute".to_string());
    }
    for config_map in build_environment.cargo_config_sha256.values() {
        for digest in config_map.values() {
            require_sha256("Cargo config SHA-256", digest)?;
        }
    }
    if builder_run.toolchain_binaries != build_environment.toolchain_binaries {
        return Err(
            "toolchain binary facts disagree between sidecar and final builder run".to_string(),
        );
    }
    for required_binary in ["cargo", "rustc"] {
        if !build_environment
            .toolchain_binaries
            .contains_key(required_binary)
        {
            return Err(format!(
                "build environment lacks the pinned {required_binary} binary"
            ));
        }
    }
    for binary in build_environment.toolchain_binaries.values() {
        require_sha256("toolchain binary sha256", &binary.sha256)?;
        if !Path::new(&binary.path).is_absolute() {
            return Err("toolchain binary paths must be absolute".to_string());
        }
    }
    for subject in &manifest.subjects {
        if subject.build.profile != "release"
            || subject.build.target_triple != build_environment.target_triple
            || subject.build_inputs.cargo_version != build_environment.cargo_version
            || subject.build_inputs.rustc_version != build_environment.rustc_version
            || subject.build_inputs.host_triple != build_environment.host_triple
        {
            return Err(format!(
                "{} build receipt disagrees with build-environment sidecar",
                subject.role
            ));
        }
    }
    Ok(())
}

fn require_sha256(name: &str, digest: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a full SHA-256"));
    }
    Ok(())
}

fn require_git_oid(name: &str, oid: &str) -> Result<(), String> {
    if oid.len() != 40 || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a full 40-character Git object ID"));
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
    let canonical_path =
        canonical_existing_regular_file_without_symlinks(&path, "external trust record")?;
    Ok(TrustRecordSource {
        canonical_path,
        expected_sha256,
    })
}

fn load_future_trust_record_path() -> Result<PathBuf, String> {
    let path = std::env::var_os(TRUST_RECORD_PATH_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!(
                "{TRUST_RECORD_PATH_ENV} is required; supply the future trust-record path from outside candidate-controlled files"
            )
        })?;
    canonical_absent_file_path(&path, "future external trust record")
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

fn resolve_write_policy_with_evidence(
    input: WritePolicyInput,
    subjects: &[Subject],
    expected_root: Option<&Path>,
    trust_record_path: &Path,
    evidence: Option<&LoadedEvidenceContext>,
    test_binary: &Path,
) -> Result<WritePolicyReceipt, String> {
    let resolved_write_root =
        resolve_existing_directory_without_symlinks(&input.write_root, WRITE_ROOT_ENV)?;
    let write_root = resolved_write_root.path.clone();
    let write_root_path = resolved_path_from_directory(&resolved_write_root);
    let receipt_path = canonical_new_file_path(&input.receipt_path, "run receipt")?;
    if path_exists_no_follow(&receipt_path)? {
        return Err(format!(
            "run receipt destination already exists; refusing to overwrite {}",
            receipt_path.display()
        ));
    }
    let resolved_receipt_path = resolve_path_for_overlap(&receipt_path, "run receipt")?;
    let mut protected_roots = canonical_caller_protected_roots(input.protected_roots)?;

    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checkout_root = suite_checkout_root(package_root)?;
    protected_roots.push(protected_root(
        "suite_checkout",
        None,
        checkout_root.clone(),
    )?);
    if let Some(evidence) = evidence {
        let evidence_root = evidence
            .manifest_path
            .parent()
            .ok_or_else(|| "subject manifest has no run directory".to_string())?;
        protected_roots.push(protected_root(
            "subject_build_evidence",
            None,
            evidence_root.to_path_buf(),
        )?);
    }

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
        trust_record_path.to_path_buf(),
    )?);
    if let Some(parent) = trust_record_path.parent() {
        let canonical_parent =
            canonical_existing_directory_without_symlinks(parent, "trust record parent")?;
        protected_roots.push(protected_root(
            "trust_record_parent",
            None,
            canonical_parent,
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

    let mut resolved_actual_output_roots = resolve_actual_output_roots(subjects)?;
    for subject in subjects {
        let executable_parent = subject
            .executable
            .path
            .parent()
            .ok_or_else(|| format!("{} executable has no parent directory", subject.role))?;
        let resolved_executable_parent =
            resolve_existing_directory(executable_parent, "subject executable parent")?;
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
        let resolved_artifact_root =
            resolve_path_for_overlap(&artifact_root.path, "artifact root")?;
        let Some(artifact_directory) = resolved_artifact_root.directory.as_ref() else {
            return Err(format!(
                "protected artifact root for {} is not an existing directory",
                subject.role
            ));
        };
        let resolved_executable = resolve_path_for_overlap(&canonical_executable, "executable")?;
        if !directory_contains(artifact_directory, &resolved_executable_parent)
            || !path_is_within_directory(artifact_directory, &resolved_executable)
        {
            return Err(format!(
                "{} executable directory {} is outside its protected artifact root {}",
                subject.role,
                resolved_executable_parent.path.display(),
                artifact_root.path.display()
            ));
        }
        protected_roots.push(protected_root(
            "subject_executable_parent",
            Some(subject.role.clone()),
            resolved_executable_parent.path,
        )?);
    }
    resolved_actual_output_roots.sort_by(|left, right| left.0.cmp(&right.0));
    let mut actual_output_roots = resolved_actual_output_roots
        .iter()
        .map(|(role, directory)| ActualOutputRoot {
            role: role.clone(),
            path: directory.path.clone(),
        })
        .collect::<Vec<_>>();
    actual_output_roots.sort_by(|left, right| left.role.cmp(&right.role));
    protected_roots.sort();
    protected_roots.dedup();

    let resolved_protected_roots = protected_roots
        .iter()
        .map(|root| {
            resolve_path_for_overlap(&root.path, &format!("protected {} root", root.kind))
                .map(|resolved| (root, resolved))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let resolved_trust_record =
        resolve_path_for_overlap(trust_record_path, "external trust record")?;
    let resolved_trust_parent = resolved_protected_roots
        .iter()
        .find(|(root, _)| root.kind == "trust_record_parent")
        .and_then(|(_, resolved)| resolved.directory.as_ref())
        .ok_or_else(|| "external trust record parent is not an existing directory".to_string())?;

    for (root, resolved_root) in &resolved_protected_roots {
        if !matches!(
            root.kind.as_str(),
            "subject_source" | "subject_artifact" | "oracle"
        ) {
            continue;
        }
        let protected_directory = resolved_root
            .directory
            .as_ref()
            .ok_or_else(|| format!("protected {} root is not an existing directory", root.kind))?;
        validate_external_trust_parent_relation(
            resolved_trust_parent,
            protected_directory,
            &root.kind,
            &root.path,
        )?;
    }

    for (root, resolved_root) in &resolved_protected_roots {
        if matches!(root.kind.as_str(), "trust_record" | "trust_record_parent") {
            continue;
        }
        if physical_paths_overlap(&resolved_trust_record, resolved_root) {
            return Err(format!(
                "external trust record must be outside protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }
    for (role, output) in &resolved_actual_output_roots {
        if directory_overlaps_path(output, &resolved_trust_record) {
            return Err(format!(
                "external trust record must be outside {} actual output root {}",
                role,
                output.path.display()
            ));
        }
    }

    if protected_roots.is_empty() {
        return Err("write policy has no protected roots".to_string());
    }
    for (root, resolved_root) in &resolved_protected_roots {
        if physical_paths_overlap(&write_root_path, resolved_root) {
            return Err(format!(
                "write root overlaps protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }

    if receipt_path == write_root
        || !path_is_within_directory(&resolved_write_root, &resolved_receipt_path)
    {
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
    for (root, resolved_root) in &resolved_protected_roots {
        if physical_paths_overlap(&resolved_receipt_path, resolved_root) {
            return Err(format!(
                "run receipt destination overlaps protected {} root {}",
                root.kind,
                root.path.display()
            ));
        }
    }

    for (role, output) in &resolved_actual_output_roots {
        if output.identity == resolved_write_root.identity
            || !directory_contains(&resolved_write_root, output)
        {
            return Err(format!(
                "{} actual output root must be strictly beneath {WRITE_ROOT_ENV}: {}",
                role,
                output.path.display()
            ));
        }
        for (root, resolved_root) in &resolved_protected_roots {
            if directory_overlaps_path(output, resolved_root) {
                return Err(format!(
                    "{} actual output root overlaps protected {} root {}",
                    role,
                    root.kind,
                    root.path.display()
                ));
            }
        }
        let output_path = resolved_path_from_directory(output);
        if physical_paths_overlap(&resolved_receipt_path, &output_path) {
            return Err(format!(
                "run receipt destination overlaps {} actual output root {}",
                role,
                output.path.display()
            ));
        }
    }
    let mut policy = WritePolicyReceipt {
        schema: WRITE_POLICY_SCHEMA.to_string(),
        write_root,
        receipt_path,
        subject_cwd: checkout_root,
        protected_roots,
        actual_output_roots,
        sha256: String::new(),
    };
    policy.sha256 = write_policy_digest(&policy)?;
    Ok(policy)
}

#[cfg(test)]
fn resolve_write_policy(
    input: WritePolicyInput,
    subjects: &[Subject],
    expected_root: Option<&Path>,
    trust_source: &TrustRecordSource,
    test_binary: &Path,
) -> Result<WritePolicyReceipt, String> {
    resolve_write_policy_with_evidence(
        input,
        subjects,
        expected_root,
        &trust_source.canonical_path,
        None,
        test_binary,
    )
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
    let canonical_parent =
        canonical_existing_directory_without_symlinks(parent, &format!("{name} parent"))?;
    Ok(canonical_parent.join(file_name))
}

fn canonical_absent_file_path(path: &Path, name: &str) -> Result<PathBuf, String> {
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
    let canonical_parent = canonical_existing_directory_without_symlinks(parent, name)?;
    let canonical = canonical_parent.join(file_name);
    if path_exists_no_follow(&canonical)? {
        return Err(format!(
            "{name} destination must be absent: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
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
        "subject_cwd": policy.subject_cwd,
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
    evidence: &LoadedEvidenceContext,
    expected_actual: Option<&Result<String, String>>,
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
        evidence,
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
    evidence: &LoadedEvidenceContext,
    expected_actual: Option<&Result<String, String>>,
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
    for (name, reference, expected_path, expected_digest) in [
        (
            "trust record subject manifest",
            &record.build_evidence.subject_manifest,
            &evidence.manifest_path,
            evidence.manifest_sha256.as_str(),
        ),
        (
            "trust record builder run",
            &record.build_evidence.builder_run,
            &evidence.builder_run_path,
            evidence.builder_run_sha256.as_str(),
        ),
    ] {
        require_sha256(&format!("{name} digest"), &reference.sha256)?;
        if Path::new(&reference.path) != expected_path || reference.sha256 != expected_digest {
            return Err(format!(
                "{name} path/digest does not match the loaded raw evidence"
            ));
        }
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

    match (&manifest.expected_generation, &record.expected_generation) {
        (None, Some(_)) => {
            return Err(
                "trust record admits an expected generation absent from the subject manifest"
                    .to_string(),
            );
        }
        (None, None) => {}
        (Some(_), None) => {}
        (Some(expected), Some(admission)) => {
            require_sha256("trust record expected_generation.sha256", &admission.sha256)?;
            require_sha256(
                "trust record expected_generation.admission_receipt_sha256",
                &admission.admission_receipt_sha256,
            )?;
            if admission.id != expected.id
                || admission.sha256 != expected.sha256
                || expected_actual.and_then(|actual| actual.as_ref().ok())
                    != Some(&admission.sha256)
            {
                return Err(
                    "trust record expected-generation admission does not match verified tree"
                        .to_string(),
                );
            }
        }
    }
    Ok(receipt_sha256)
}

fn validate_actual_output_roots(
    subjects: &[Subject],
    expected_root: Option<&Path>,
) -> Result<(), String> {
    let roots = resolve_actual_output_roots(subjects)?;
    if let Some(expected_root) = expected_root {
        let expected = resolve_path_for_overlap(expected_root, "expected-generation root")?;
        for (role, root) in &roots {
            if directory_overlaps_path(root, &expected) {
                return Err(format!(
                    "output roots overlap: {role}={} and expected-generation={}",
                    root.path.display(),
                    expected.path.display()
                ));
            }
        }
    }
    Ok(())
}

fn resolve_actual_output_roots(
    subjects: &[Subject],
) -> Result<Vec<(String, ResolvedDirectory)>, String> {
    let mut roots = Vec::with_capacity(subjects.len());
    for subject in subjects {
        roots.push((
            subject.role.clone(),
            resolve_existing_directory_without_symlinks(
                &subject.actual_output_root,
                &format!("{} actual output root", subject.role),
            )?,
        ));
    }
    roots.sort_by(|left, right| left.0.cmp(&right.0));
    for left in 0..roots.len() {
        for right in (left + 1)..roots.len() {
            let (left_name, left_root) = &roots[left];
            let (right_name, right_root) = &roots[right];
            if physical_directories_overlap(left_root, right_root) {
                return Err(format!(
                    "output roots overlap: {left_name}={} and {right_name}={}",
                    left_root.path.display(),
                    right_root.path.display()
                ));
            }
        }
    }
    Ok(roots)
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

pub fn prepare_holla_help_overlay_preflight() -> Result<WritePolicyPreflightReceipt, String> {
    const CASE_ID: &str = "HELP-HOLLA-004";

    let current_suite_digest = suite_digest()?;
    verify_compiled_suite_digest(COMPILED_SUITE_SHA256, &current_suite_digest)?;
    let loaded = load_subject_evidence()?;
    let manifest = loaded.manifest;
    let evidence = loaded.context;
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
        .find(|case| case.id == CASE_ID)
        .ok_or_else(|| format!("missing preflight case ID {CASE_ID}"))?;
    validate_case_contract(case)?;
    let mut subjects = manifest.subjects.clone();
    subjects.sort_by(|left, right| left.role.cmp(&right.role));
    if subjects
        .iter()
        .any(|subject| subject.build.target_name != case.binary)
    {
        return Err(format!(
            "case {CASE_ID} requires binary {}, but subject build target does not match",
            case.binary
        ));
    }

    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_digest = digest_tree(&package_root.join("cases"))?;
    let profile_digest = sha256_file(&package_root.join("profile.json"))?;
    let dependency_lock_digest = sha256_file(&package_root.join("Cargo.lock"))?;
    let current_binary =
        std::env::current_exe().map_err(|error| format!("locate test binary: {error}"))?;
    let test_binary_path = fs::canonicalize(&current_binary).map_err(|error| {
        format!(
            "canonicalize test binary {}: {error}",
            current_binary.display()
        )
    })?;
    let test_binary_digest = sha256_file(&test_binary_path)?;

    let profile: Profile = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse suite profile: {error}"))?;
    validate_profile_and_case(&profile, case)?;
    let renderer_profile = tuiscotti::profile::RenderProfile::vendored();
    let renderer = renderer_identity(&renderer_profile);
    if renderer.hash != profile.renderer.expected_hash
        || renderer.renderer_version != profile.renderer.renderer_version
        || renderer.name != profile.renderer.id
        || profile.renderer.implementation != "strict-vendored-profile"
    {
        return Err(
            "resolved Tuiscotti renderer identity does not match the suite profile".to_string(),
        );
    }
    let _expected_generation = manifest
        .expected_generation
        .as_ref()
        .map(|expected| {
            visual::load_expected_generation(
                expected,
                case,
                &case_digest,
                &profile_digest,
                &dependency_lock_digest,
                &profile.tool_revision,
                &renderer,
            )
        })
        .transpose()?;

    let mut source_pair = Vec::with_capacity(subjects.len());
    for subject in &subjects {
        let snapshot = verify_subject_executable(subject)?;
        source_pair.push(SubjectIdentity {
            role: subject.role.clone(),
            source_commit: subject.source_commit.clone(),
            build: subject.build.clone(),
            build_inputs: subject.build_inputs.clone(),
            builder_receipt: subject.builder_receipt.clone(),
            builder_receipt_sha256: subject.builder_receipt_sha256.clone(),
            executable: ExecutableReceipt {
                path: subject.executable.path.clone(),
                expected_sha256: subject.executable.sha256.clone(),
                actual_sha256: Some(snapshot.sha256),
            },
            actual_output_root: subject.actual_output_root.clone(),
        });
    }

    let trust_record_path = load_future_trust_record_path()?;
    let write_policy_input = load_write_policy_input()?;
    let write_policy = resolve_write_policy_with_evidence(
        write_policy_input,
        &subjects,
        manifest
            .expected_generation
            .as_ref()
            .and_then(|expected| expected.root.as_deref()),
        &trust_record_path,
        Some(&evidence),
        &test_binary_path,
    )?;
    validate_case_capture_paths(case, &write_policy.actual_output_roots)?;

    Ok(WritePolicyPreflightReceipt {
        schema: "termrock-spec/parity-write-policy-preflight-v1".to_string(),
        state: "prepared_only".to_string(),
        case_id: CASE_ID.to_string(),
        suite: SuiteIdentity {
            revision: registry.suite_revision,
            digest: current_suite_digest,
            compiled_digest: COMPILED_SUITE_SHA256.to_string(),
            case_set_digest: case_digest,
            profile_digest,
            test_binary_digest: test_binary_digest.clone(),
            dependency_lock_sha256: dependency_lock_digest,
            platform: std::env::consts::OS.to_string(),
        },
        test_executable: TestExecutableIdentity {
            path: test_binary_path,
            sha256: test_binary_digest,
        },
        expected_generation: manifest.expected_generation,
        build_evidence: BuildEvidenceReceipt {
            subject_manifest: EvidenceReference {
                path: evidence.manifest_path.display().to_string(),
                sha256: evidence.manifest_sha256,
            },
            source_inputs: EvidenceReference {
                path: evidence.source_inputs_path.display().to_string(),
                sha256: evidence.source_inputs_sha256,
            },
            build_environment: EvidenceReference {
                path: evidence.build_environment_path.display().to_string(),
                sha256: evidence.build_environment_sha256,
            },
            builder_run: EvidenceReference {
                path: evidence.builder_run_path.display().to_string(),
                sha256: evidence.builder_run_sha256,
            },
        },
        source_pair,
        renderer,
        trust_record: PreparedTrustRecord {
            path: trust_record_path,
            state: "required_absent".to_string(),
        },
        write_policy,
    })
}

pub fn run_case(case_id: &str) -> Result<RunReceipt, String> {
    let current_suite_digest = suite_digest()?;
    verify_compiled_suite_digest(COMPILED_SUITE_SHA256, &current_suite_digest)?;
    let loaded = load_subject_evidence()?;
    let manifest = loaded.manifest;
    let evidence = loaded.context;
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
    let profile: Profile = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse suite profile: {error}"))?;
    validate_profile_and_case(&profile, case)?;
    let renderer_profile = tuiscotti::profile::RenderProfile::vendored();
    let renderer = renderer_identity(&renderer_profile);
    if renderer.hash != profile.renderer.expected_hash
        || renderer.renderer_version != profile.renderer.renderer_version
        || renderer.name != profile.renderer.id
        || profile.renderer.implementation != "strict-vendored-profile"
    {
        return Err(
            "resolved Tuiscotti renderer identity does not match the suite profile".to_string(),
        );
    }
    let loaded_expected = manifest.expected_generation.as_ref().map(|expected| {
        visual::load_expected_generation(
            expected,
            case,
            &case_digest,
            &profile_digest,
            &dependency_lock_digest,
            &profile.tool_revision,
            &renderer,
        )
    });
    let expected_actual = loaded_expected.as_ref().map(|loaded| {
        loaded
            .as_ref()
            .map(|generation| generation.tree_sha256.clone())
            .map_err(Clone::clone)
    });
    let mut subjects = manifest.subjects.clone();
    subjects.sort_by(|left, right| left.role.cmp(&right.role));
    let trust_source = load_trust_record_source()?;
    let write_policy_input = load_write_policy_input()?;
    let write_policy = resolve_write_policy_with_evidence(
        write_policy_input,
        &subjects,
        manifest
            .expected_generation
            .as_ref()
            .and_then(|expected| expected.root.as_deref()),
        &trust_source.canonical_path,
        Some(&evidence),
        &binary_path,
    )?;
    validate_case_capture_paths(case, &write_policy.actual_output_roots)?;
    let trusted = load_and_verify_trust_record(
        &manifest,
        &subjects,
        &evidence,
        expected_actual.as_ref(),
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
        expected_generation: manifest.expected_generation.as_ref().map(|expected| {
            ExpectedGenerationReceipt {
                id: expected.id.clone(),
                expected_sha256: expected.sha256.clone(),
                actual_sha256: expected_actual
                    .as_ref()
                    .and_then(|actual| actual.as_ref().ok().cloned()),
                state: expected_generation_state(
                    expected_actual
                        .as_ref()
                        .expect("expected generation result exists"),
                    generation_admission.is_some(),
                )
                .to_string(),
                admission_receipt_sha256: generation_admission
                    .map(|admission| admission.admission_receipt_sha256.clone()),
                root: expected.root.clone(),
            }
        }),
        build_evidence: BuildEvidenceReceipt {
            subject_manifest: EvidenceReference {
                path: evidence.manifest_path.display().to_string(),
                sha256: evidence.manifest_sha256.clone(),
            },
            source_inputs: EvidenceReference {
                path: evidence.source_inputs_path.display().to_string(),
                sha256: evidence.source_inputs_sha256.clone(),
            },
            build_environment: EvidenceReference {
                path: evidence.build_environment_path.display().to_string(),
                sha256: evidence.build_environment_sha256.clone(),
            },
            builder_run: EvidenceReference {
                path: evidence.builder_run_path.display().to_string(),
                sha256: evidence.builder_run_sha256.clone(),
            },
        },
        environment: EnvironmentReceipt {
            cleared: profile.environment.clear,
            cwd: write_policy.subject_cwd.clone(),
            common: common_environment(&profile),
            case_allowlist: profile.environment.case_allowlist.clone(),
            case_values: case.environment.clone(),
        },
        renderer: renderer.clone(),
        trust: trusted.verification.clone(),
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

        let args = make_argv(&subject.executable.path, &case.args);
        let launch_result = launch(&args, case, &profile, &write_policy.subject_cwd);
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
                            visual_comparison: None,
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
                        Ok((artifacts, captured)) => {
                            let frame_artifact = artifacts
                                .iter()
                                .find(|artifact| artifact.format == "frame_json")
                                .cloned()
                                .ok_or_else(|| {
                                    format!("capture {checkpoint} omitted canonical frame artifact")
                                })?;
                            let png_artifact = artifacts
                                .iter()
                                .find(|artifact| artifact.format == "png")
                                .cloned()
                                .ok_or_else(|| {
                                    format!("capture {checkpoint} omitted PNG artifact")
                                })?;
                            receipt.artifacts.extend(artifacts);
                            let actual = visual::validate_actual_observation(
                                &captured,
                                &frame_artifact,
                                &png_artifact,
                                &case.geometry,
                                &renderer,
                            );
                            let visual_outcome: Result<VisualComparisonEvidence, (String, String)> =
                                match actual {
                                    Err(error) => Err(("ERROR".to_string(), error)),
                                    Ok(_) if manifest.expected_generation.is_none() => Err((
                                        "BLOCKED".to_string(),
                                        format!(
                                            "no shared expected generation was supplied; exact cell, cursor, and decoded-pixel comparison remains blocked (alias: {})",
                                            legacy_snapshot_path.as_deref().unwrap_or("none")
                                        ),
                                    )),
                                    Ok(_)
                                        if loaded_expected.as_ref().is_some_and(Result::is_err) =>
                                    {
                                        Err((
                                            "ERROR".to_string(),
                                            loaded_expected
                                                .as_ref()
                                                .and_then(|result| result.as_ref().err())
                                                .cloned()
                                                .unwrap_or_else(|| {
                                                    "expected generation failed validation"
                                                        .to_string()
                                                }),
                                        ))
                                    }
                                    Ok(_) if generation_admission.is_none() => Err((
                                        "BLOCKED".to_string(),
                                        format!(
                                            "expected generation hash is valid but no independent admission binds it; visual comparison remains blocked (alias: {})",
                                            legacy_snapshot_path.as_deref().unwrap_or("none")
                                        ),
                                    )),
                                    Ok(validated) => {
                                        let generation = loaded_expected
                                            .as_ref()
                                            .and_then(|loaded| loaded.as_ref().ok())
                                            .expect("admitted generation passed validation");
                                        let admission = generation_admission
                                            .expect("trust validation requires expected admission");
                                        visual::compare_admitted_observations(
                                            generation,
                                            checkpoint,
                                            validated,
                                            &renderer,
                                            &trusted.verification.record_sha256,
                                            &admission.admission_receipt_sha256,
                                        )
                                        .map_err(|error| ("ERROR".to_string(), error))
                                    }
                                };
                            match visual_outcome {
                                Ok(comparison) => {
                                    let passed = comparison.renderer.equal
                                        && comparison.frame.equal
                                        && comparison.png.equal
                                        && !comparison.fidelity.expected.approximate
                                        && !comparison.fidelity.actual.approximate
                                        && comparison.fidelity.expected.rerender_matches_bound_png
                                        && comparison.fidelity.actual.rerender_matches_bound_png;
                                    let status = if passed { "PASS" } else { "FAIL" };
                                    let reason = if passed {
                                        "canonical Frame v3, cursor, strict fidelity, renderer profile, and decoded opaque RGB pixels match the admitted generation"
                                    } else {
                                        "admitted expected and actual frame or decoded opaque RGB pixels differ"
                                    };
                                    let mut visual_check = check(
                                        &subject.role,
                                        &case.id,
                                        checkpoint,
                                        "visual",
                                        status,
                                        reason.to_string(),
                                        Vec::new(),
                                    );
                                    visual_check.visual_comparison = Some(comparison);
                                    receipt.checks.push(visual_check);
                                }
                                Err((status, reason)) => receipt.checks.push(check(
                                    &subject.role,
                                    &case.id,
                                    checkpoint,
                                    "visual",
                                    &status,
                                    reason,
                                    vec![legacy_snapshot_path.clone().unwrap_or_else(|| {
                                        "no legacy approved snapshot".to_string()
                                    })],
                                )),
                            }
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
                        "exit",
                        "NOT_APPLICABLE",
                        "HELP-HOLLA-004 closes and reopens an overlay but does not exit the root TUI",
                        Vec::new(),
                    );
                    push_check(
                        &mut receipt.checks,
                        &subject.role,
                        &case.id,
                        checkpoint,
                        "restoration",
                        "NOT_APPLICABLE",
                        "HELP-HOLLA-004 runs inside an isolated PTY and does not assert the parent terminal state",
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
    validate_visual_receipt_contract(
        &receipt,
        case,
        loaded_expected
            .as_ref()
            .and_then(|result| result.as_ref().ok()),
        generation_admission,
    )?;
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
        "exit",
        "restoration",
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

fn validate_visual_receipt_contract(
    receipt: &RunReceipt,
    case: &Case,
    expected: Option<&visual::ExpectedGeneration>,
    admission: Option<&TrustedExpectedGeneration>,
) -> Result<(), String> {
    if receipt.schema != RECEIPT_SCHEMA {
        return Err("visual comparison requires run receipt v3".to_string());
    }
    match (&receipt.expected_generation, expected, admission) {
        (None, None, None) => {}
        (Some(run_expected), Some(generation), run_admission) => {
            if run_expected.id != generation.id
                || run_expected.expected_sha256 != generation.tree_sha256
                || run_expected.actual_sha256.as_deref() != Some(generation.tree_sha256.as_str())
            {
                return Err(
                    "run receipt expected-generation identity differs from loaded tree".to_string(),
                );
            }
            match run_admission {
                Some(admitted)
                    if admitted.id == generation.id
                        && admitted.sha256 == generation.tree_sha256
                        && run_expected.state == "ADMITTED"
                        && run_expected.admission_receipt_sha256.as_deref()
                            == Some(admitted.admission_receipt_sha256.as_str())
                        && receipt.trust.status == "accepted" => {}
                Some(_) => {
                    return Err(
                        "run receipt expected-generation admission binding is inconsistent"
                            .to_string(),
                    );
                }
                None if run_expected.state == "HASH_MATCH"
                    && run_expected.admission_receipt_sha256.is_none() => {}
                None => {
                    return Err(
                        "unadmitted expected generation has an invalid run state".to_string()
                    );
                }
            }
        }
        (Some(run_expected), None, None)
            if run_expected.state == "BLOCKED_MISSING_OR_INVALID"
                && run_expected.actual_sha256.is_none()
                && run_expected.admission_receipt_sha256.is_none() => {}
        (None, None, Some(_)) => {
            return Err("trusted expected-generation admission has no run input".to_string());
        }
        _ => return Err("run receipt expected-generation state is inconsistent".to_string()),
    }

    if receipt
        .checks
        .iter()
        .any(|check| check.dimension != "visual" && check.visual_comparison.is_some())
    {
        return Err("visual comparison evidence is attached to a non-visual check".to_string());
    }

    let roles = receipt
        .source_pair
        .iter()
        .map(|subject| subject.role.as_str())
        .collect::<BTreeSet<_>>();
    if roles != BTreeSet::from(["reference", "candidate"]) {
        return Err(
            "visual receipt does not contain the paired reference and candidate".to_string(),
        );
    }
    let checkpoint_ids = checkpoints(case);
    let visual_checks = receipt
        .checks
        .iter()
        .filter(|check| check.dimension == "visual")
        .collect::<Vec<_>>();
    if visual_checks.len() != roles.len() * checkpoint_ids.len() {
        return Err(format!(
            "visual receipt has {} checks; expected {} paired checkpoint checks",
            visual_checks.len(),
            roles.len() * checkpoint_ids.len()
        ));
    }

    for role in roles {
        for checkpoint in &checkpoint_ids {
            let matches = visual_checks
                .iter()
                .filter(|check| {
                    check.subject_role == role
                        && check.case_id == case.id
                        && check.checkpoint_id.as_str() == *checkpoint
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(format!(
                    "visual receipt needs one check for {role}:{}:{checkpoint}, found {}",
                    case.id,
                    matches.len()
                ));
            }
            let check = matches[0];
            if check.id != format!("{role}:{}:{checkpoint}:visual", case.id) {
                return Err(format!(
                    "visual check ID is inconsistent for {role}:{checkpoint}"
                ));
            }
            if check.reason.trim().is_empty() {
                return Err(format!(
                    "visual check reason is empty for {role}:{checkpoint}"
                ));
            }
            match check.status.as_str() {
                "PASS" | "FAIL" => {
                    let evidence = check.visual_comparison.as_ref().ok_or_else(|| {
                        format!("{} visual check lacks comparison evidence", check.id)
                    })?;
                    let generation = expected.ok_or_else(|| {
                        format!(
                            "{} has a visual verdict without a loaded expected generation",
                            check.id
                        )
                    })?;
                    let admitted = admission.ok_or_else(|| {
                        format!(
                            "{} has a visual verdict without an external admission",
                            check.id
                        )
                    })?;
                    validate_visual_comparison_evidence(
                        receipt,
                        case,
                        role,
                        checkpoint,
                        check.status.as_str(),
                        evidence,
                        generation,
                        admitted,
                    )?;
                }
                "BLOCKED" | "ERROR" | "NOT_RUN" | "STALE" | "NOT_APPLICABLE" => {
                    if check.visual_comparison.is_some() {
                        return Err(format!(
                            "{} non-verdict visual check carries comparison evidence",
                            check.id
                        ));
                    }
                }
                status => return Err(format!("unsupported visual check status {status:?}")),
            }
        }
    }
    Ok(())
}

fn validate_visual_comparison_evidence(
    receipt: &RunReceipt,
    case: &Case,
    role: &str,
    checkpoint: &str,
    status: &str,
    comparison: &VisualComparisonEvidence,
    generation: &visual::ExpectedGeneration,
    admission: &TrustedExpectedGeneration,
) -> Result<(), String> {
    let run_expected = receipt
        .expected_generation
        .as_ref()
        .ok_or_else(|| "visual verdict has no expected-generation receipt".to_string())?;
    let visual_admission = &comparison.admission;
    if comparison.schema != "termrock-spec/parity-visual-comparison-v1"
        || comparison.method != "frame-v3-diff-cells+opaque-rgb-decoded-exact-v1"
        || !visual_admission.verified
        || receipt.trust.status != "accepted"
        || visual_admission.trust_record_sha256 != receipt.trust.record_sha256
        || visual_admission.expected_generation_id != generation.id
        || visual_admission.expected_tree_sha256 != generation.tree_sha256
        || visual_admission.expected_manifest_sha256 != generation.manifest_sha256
        || visual_admission.admission_receipt_sha256 != admission.admission_receipt_sha256
        || run_expected.state != "ADMITTED"
        || run_expected.admission_receipt_sha256.as_deref()
            != Some(admission.admission_receipt_sha256.as_str())
    {
        return Err(format!(
            "{role}:{checkpoint} visual admission is not bound to the run trust record"
        ));
    }

    let (expected_frame, expected_png) = generation.checkpoint_refs(checkpoint)?;
    let actual_frame = capture_artifact(receipt, role, &case.id, checkpoint, "frame_json")?;
    let actual_png = capture_artifact(receipt, role, &case.id, checkpoint, "png")?;
    let actual_fidelity =
        capture_artifact(receipt, role, &case.id, checkpoint, "png_fidelity_json")?;
    if !artifact_ref_matches(&expected_frame, &comparison.frame.expected)
        || !artifact_receipt_ref_matches(actual_frame, &comparison.frame.actual)
        || !artifact_ref_matches(&expected_png, &comparison.png.expected)
        || !artifact_receipt_ref_matches(actual_png, &comparison.png.actual)
    {
        return Err(format!(
            "{role}:{checkpoint} visual artifact references do not match the bound files"
        ));
    }
    let frame = &comparison.frame;
    let expected_geometry_matches = frame.expected_geometry == case.geometry;
    let actual_geometry_matches = frame.actual_geometry == case.geometry;
    let frame_equal = frame.dimensions_equal && frame.cells_equal && frame.cursor_equal;
    if frame.version != 3
        || !expected_geometry_matches
        || !actual_geometry_matches
        || frame.dimensions_equal != (frame.expected_geometry == frame.actual_geometry)
        || frame.equal != frame_equal
    {
        return Err(format!(
            "{role}:{checkpoint} Frame v3 comparison fields are inconsistent"
        ));
    }
    let png = &comparison.png;
    let png_equal = png.dimensions_equal && png.pixels_equal;
    let profile_pixel_size = tuiscotti::profile::RenderProfile::vendored()
        .image_size(case.geometry.cols, case.geometry.rows);
    if png.alpha_policy != "opaque"
        || png.equal != png_equal
        || png.dimensions_equal
            != ((png.expected_info.width, png.expected_info.height)
                == (png.actual_info.width, png.actual_info.height))
        || png.expected_info.color_type != 2
        || png.actual_info.color_type != 2
        || png.expected_info.bit_depth != 8
        || png.actual_info.bit_depth != 8
        || (png.expected_info.width, png.expected_info.height) != profile_pixel_size
        || (png.actual_info.width, png.actual_info.height) != profile_pixel_size
        || (png.expected_info.width, png.expected_info.height)
            != (png.actual_info.width, png.actual_info.height)
    {
        return Err(format!(
            "{role}:{checkpoint} opaque RGB comparison fields are inconsistent"
        ));
    }

    let renderer = &comparison.renderer;
    if renderer.expected_sha256 != generation.renderer_sha256
        || renderer.actual_sha256 != receipt.renderer.hash
        || renderer.equal != (renderer.expected_sha256 == renderer.actual_sha256)
    {
        return Err(format!(
            "{role}:{checkpoint} renderer comparison is not bound to the run"
        ));
    }
    let fidelity = &comparison.fidelity;
    if fidelity.expected.source_frame_sha256 != expected_frame.sha256
        || fidelity.expected.renderer_sha256 != generation.renderer_sha256
        || fidelity.actual.source_frame_sha256 != actual_frame.sha256
        || fidelity.actual.renderer_sha256 != receipt.renderer.hash
        || fidelity.actual.sha256 != actual_fidelity.sha256
        || fidelity.expected.approximate
        || fidelity.actual.approximate
        || !fidelity.expected.rerender_matches_bound_png
        || !fidelity.actual.rerender_matches_bound_png
    {
        return Err(format!(
            "{role}:{checkpoint} fidelity is not bound to strict frame and PNG artifacts"
        ));
    }
    let all_equal = renderer.equal && frame.equal && png.equal;
    if (status == "PASS" && !all_equal) || (status == "FAIL" && all_equal) {
        return Err(format!(
            "{role}:{checkpoint} visual status disagrees with exact comparison fields"
        ));
    }
    Ok(())
}

fn capture_artifact<'a>(
    receipt: &'a RunReceipt,
    role: &str,
    case_id: &str,
    checkpoint: &str,
    format: &str,
) -> Result<&'a ArtifactReceipt, String> {
    let matches = receipt
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.subject_role == role
                && artifact.case_id == case_id
                && artifact.checkpoint_id == checkpoint
                && artifact.format == format
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "expected one {format} artifact for {role}:{case_id}:{checkpoint}, found {}",
            matches.len()
        ));
    }
    Ok(matches[0])
}

fn artifact_ref_matches(expected: &visual::ArtifactRef, actual: &visual::ArtifactRef) -> bool {
    expected.path == actual.path
        && expected.sha256 == actual.sha256
        && expected.bytes == actual.bytes
}

fn artifact_receipt_ref_matches(
    artifact: &ArtifactReceipt,
    reference: &visual::ArtifactRef,
) -> bool {
    artifact.path.display().to_string() == reference.path
        && artifact.sha256 == reference.sha256
        && artifact.bytes == reference.bytes
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
    subject_cwd: &Path,
) -> Result<tuiscotti::tui::Session, String> {
    let mut tui = tuiscotti::tui::Tui::new(argv)
        .size(case.geometry.cols, case.geometry.rows)
        .env_clear(profile.environment.clear)
        .env("TERM", &profile.terminal.term)
        .env("COLORTERM", &profile.terminal.colorterm)
        .env("LC_ALL", &profile.terminal.locale)
        .env("SHELL", &profile.terminal.shell)
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
        .env_remove(PROTECTED_ROOTS_ENV)
        .cwd(subject_cwd.to_path_buf());
    for (name, value) in &case.environment {
        if !profile
            .environment
            .case_allowlist
            .iter()
            .any(|allowed| allowed == name)
        {
            return Err(format!(
                "case environment variable {name:?} is not in the profile allowlist"
            ));
        }
        tui = tui.env(name, value);
    }
    tui.spawn()
        .map_err(|error| format!("Tuiscotti PTY spawn failed: {error}"))
}

fn common_environment(profile: &Profile) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("TERM".to_string(), profile.terminal.term.clone()),
        ("COLORTERM".to_string(), profile.terminal.colorterm.clone()),
        ("LC_ALL".to_string(), profile.terminal.locale.clone()),
        ("SHELL".to_string(), profile.terminal.shell.clone()),
    ])
}

fn validate_profile_and_case(profile: &Profile, case: &Case) -> Result<(), String> {
    if profile.schema != "termrock-e2e/profile-v1"
        || profile.id != "tuiscotti-default-parity-v1"
        || profile.tool != "tuiscotti"
        || profile.tool_revision != "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274"
        || profile.screen_profile != "default"
        || profile.geometry_policy != "case-declared"
    {
        return Err(
            "suite screen profile identity does not match the pinned Tuiscotti input".to_string(),
        );
    }
    for required_export in ["ansi", "html", "png", "ascii", "txt", "frame_json"] {
        if !profile
            .exports
            .iter()
            .any(|export| export == required_export)
        {
            return Err(format!(
                "suite profile is missing required export {required_export}"
            ));
        }
    }
    for required_companion in [
        "ascii_loss_json",
        "png_fidelity_json",
        "observations_json",
        "manifest_json",
    ] {
        if !profile
            .companions
            .iter()
            .any(|companion| companion == required_companion)
        {
            return Err(format!(
                "suite profile is missing required capture companion {required_companion}"
            ));
        }
    }
    if !profile.visual_acceptance.starts_with("BLOCKED-until-") {
        return Err(
            "pilot visual acceptance must remain blocked until comparator and oracle admission"
                .to_string(),
        );
    }
    if !profile.environment.clear {
        return Err("PTY environment profile must clear inherited variables".to_string());
    }
    if profile.terminal.cwd_policy != "suite-checkout" {
        return Err("PTY CWD policy must be suite-checkout".to_string());
    }
    if profile.terminal.shell != "/bin/sh" {
        return Err("PTY shell must use the deterministic absolute /bin/sh path".to_string());
    }
    if profile
        .environment
        .case_allowlist
        .iter()
        .collect::<BTreeSet<_>>()
        .len()
        != profile.environment.case_allowlist.len()
    {
        return Err("PTY case environment allowlist has duplicate names".to_string());
    }
    for name in case.environment.keys() {
        if !profile
            .environment
            .case_allowlist
            .iter()
            .any(|allowed| allowed == name)
        {
            return Err(format!(
                "case environment variable {name:?} is not in the profile allowlist"
            ));
        }
    }
    if common_environment(profile).values().any(String::is_empty) {
        return Err("PTY common environment values must be nonempty".to_string());
    }
    if profile.renderer.implementation != "strict-vendored-profile"
        || profile.renderer.id.trim().is_empty()
        || profile.renderer.expected_hash.trim().is_empty()
    {
        return Err("renderer profile identity is incomplete".to_string());
    }
    require_sha256(
        "profile.renderer.expected_hash",
        &profile.renderer.expected_hash,
    )?;
    Ok(())
}

fn renderer_identity(profile: &tuiscotti::profile::RenderProfile<'_>) -> RendererIdentity {
    let palette = profile.palette();
    let foreground = palette.default_fg;
    let background = palette.default_bg;
    RendererIdentity {
        schema: "termrock-spec/tuiscotti-renderer-identity-v1".to_string(),
        hash: profile.hash(),
        name: profile.name().to_string(),
        renderer_version: profile.renderer_version(),
        font_px: profile.font_px(),
        cell_w: profile.cell_w(),
        cell_h: profile.cell_h(),
        pad: profile.pad(),
        scale: profile.scale(),
        default_fg: [foreground.r, foreground.g, foreground.b],
        default_bg: [background.r, background.g, background.b],
        indexed_palette: format!("{:?}", palette.indexed),
        face_hashes: profile.face_hashes().clone(),
        fallback_order: profile
            .fallback_order()
            .iter()
            .map(|fallback| RendererFallbackIdentity {
                description: fallback.desc.to_string(),
                sha256: fallback.sha256.to_string(),
            })
            .collect(),
        cursor_policy: format!("{:?}", profile.cursor()),
        blink_phase: format!("{:?}", profile.blink_phase()),
        missing_glyph_policy: format!("{:?}", profile.missing()),
    }
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
    for dimension in ["exit", "restoration"] {
        push_check(
            checks,
            role,
            case_id,
            checkpoint,
            dimension,
            "NOT_RUN",
            "subject did not reach process exit",
            Vec::new(),
        );
    }
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
        visual_comparison: None,
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
) -> Result<(Vec<ArtifactReceipt>, visual::CapturedObservation), String> {
    use tuiscotti::formats::capture_all;
    use tuiscotti::profile::RenderProfile;
    use tuiscotti::render::Renderer;

    let relative_stem = capture_relative_stem(case, checkpoint)?;
    let strict_profile = RenderProfile::vendored();
    let renderer_profile = renderer_identity(&strict_profile);
    let mut renderer = Renderer::for_render_profile(&strict_profile)
        .map_err(|error| format!("initialize strict Tuiscotti renderer: {error}"))?;
    let bundle = capture_all(&mut renderer, frame, &format!("{}:{checkpoint}", case.id))
        .map_err(|error| format!("capture Tuiscotti bundle: {error}"))?;
    let rendered = renderer
        .render(frame)
        .map_err(|error| format!("render Tuiscotti PNG: {error}"))?;
    let frame_bytes = bundle.json.into_bytes();
    let capture_png_bytes = bundle.png;
    let rerender_png_bytes = rendered.png;
    let fidelity_approximate = rendered.fidelity.approximate;
    let fidelity_bytes = rendered.fidelity.to_json().into_bytes();
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
            "screen_profile": frame.provenance.profile,
            "source": frame.provenance.source,
            "argv": frame.provenance.argv
        },
        "renderer_profile": renderer_profile,
        "toolkit_unavailable_properties": ["application-internal-state", "private-action-enum"],
        "frame_digest": frame.digest(),
        "legacy_snapshot_alias": legacy_snapshot_path
    }))
    .map_err(|error| format!("serialize observation facts: {error}"))?;

    let primary = [
        ("frame_json", "frame.json", frame_bytes.clone()),
        ("ansi", "ansi", bundle.ansi.into_bytes()),
        ("html", "html", bundle.html.into_bytes()),
        ("png", "png", capture_png_bytes.clone()),
        ("ascii", "ascii", bundle.ascii.text.into_bytes()),
        ("txt", "txt", bundle.txt.into_bytes()),
        ("ascii_loss_json", "ascii.loss.json", ascii_loss),
        (
            "png_fidelity_json",
            "png.fidelity.json",
            fidelity_bytes.clone(),
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
        "screen_profile": frame.provenance.profile,
        "renderer_profile": renderer_profile,
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

    let artifacts = outputs
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
        .collect();
    Ok((
        artifacts,
        visual::CapturedObservation {
            frame_bytes,
            png_bytes: capture_png_bytes,
            rerender_png_bytes,
            fidelity_bytes,
            fidelity_approximate,
            renderer_sha256: renderer_profile.hash,
        },
    ))
}

fn capture_relative_stem(case: &Case, checkpoint: &str) -> Result<String, String> {
    let output_substep = checkpoint_substep(case, checkpoint);
    let relative = format!(
        "{}/{}/{}/{}x{}/{}",
        case.app,
        case.screen,
        output_substep,
        case.geometry.cols,
        case.geometry.rows,
        case.color_path
    );
    let path = Path::new(&relative);
    if path.is_absolute()
        || relative.is_empty()
        || relative.contains('\0')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("invalid case capture path {relative:?}"));
    }
    Ok(relative)
}

fn validate_case_capture_paths(
    case: &Case,
    actual_output_roots: &[ActualOutputRoot],
) -> Result<(), String> {
    for output in actual_output_roots {
        let root = resolve_existing_directory_without_symlinks(
            &output.path,
            &format!("{} actual output root", output.role),
        )?;
        for checkpoint in checkpoints(case) {
            let relative = capture_relative_stem(case, checkpoint)?;
            validate_capture_leaf_before_launch(&root, &relative, &output.role, checkpoint)?;
        }
    }
    Ok(())
}

fn validate_capture_leaf_before_launch(
    output_root: &ResolvedDirectory,
    relative: &str,
    role: &str,
    checkpoint: &str,
) -> Result<(), String> {
    let relative_path = Path::new(relative);
    let components = relative_path.components().collect::<Vec<_>>();
    if relative_path.is_absolute()
        || components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("invalid relative capture path {relative:?}"));
    }

    let mut current = output_root.path.clone();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(format!("invalid relative capture path {relative:?}"));
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "{role} capture path contains a symlink before launch: {}",
                    current.display()
                ));
            }
            Ok(_) if index + 1 == components.len() => {
                return Err(format!(
                    "{role} capture leaf already exists before launch for {checkpoint}: {}",
                    current.display()
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(format!(
                    "{role} capture path ancestor is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) => {
                let resolved_child = resolve_existing_directory_without_symlinks(
                    &current,
                    &format!("{role} capture path ancestor"),
                )?;
                if !directory_contains(output_root, &resolved_child) {
                    return Err(format!(
                        "{role} capture path escaped its physical output root: {}",
                        current.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "inspect {role} capture path before launch {}: {error}",
                    current.display()
                ));
            }
        }
    }
    Ok(())
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
    fn pty_environment_drops_parent_sentinel_and_uses_pinned_cwd() {
        let executable = std::env::current_exe().expect("locate unit test binary");
        let output = std::process::Command::new(executable)
            .args([
                "--exact",
                "tests::environment_probe_child_process",
                "--nocapture",
            ])
            .env("TERMROCK_E2E_ENV_PROBE_MODE", "1")
            .env(
                "TERMROCK_E2E_PARENT_SECRET_SENTINEL",
                "must-not-reach-subject",
            )
            .output()
            .expect("run child test process with sentinel");
        assert!(
            output.status.success(),
            "child environment probe failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("PTY_ENV_PROBE_OK"));
    }

    #[test]
    fn environment_probe_child_process() {
        if std::env::var_os("TERMROCK_E2E_ENV_PROBE_MODE").is_none() {
            return;
        }
        let profile: Profile = serde_json::from_str(PROFILE_JSON).expect("parse suite profile");
        let case = registry().unwrap().cases.remove(0);
        validate_profile_and_case(&profile, &case).expect("validate controlled profile");
        let cwd = suite_checkout_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("resolve suite checkout CWD");
        let argv = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "pwd; /usr/bin/env".to_string(),
        ];
        let mut session =
            launch(&argv, &case, &profile, &cwd).expect("spawn environment probe PTY");
        let observation = session
            .wait_predicate_timeout(
                |observation| {
                    tuiscotti::render::frame_from_screen(&observation.screen, "default")
                        .text()
                        .contains("HOLLA_NO_HISTORY=1")
                },
                Duration::from_secs(3),
            )
            .expect("observe probe environment");
        let screen = tuiscotti::render::frame_from_screen(&observation.screen, "default").text();
        assert!(
            screen.contains(&cwd.display().to_string()),
            "pinned CWD absent: {screen}"
        );
        assert!(screen.contains("TERM=xterm-256color"));
        assert!(screen.contains("COLORTERM=truecolor"));
        assert!(screen.contains("LC_ALL=C.UTF-8"));
        assert!(screen.contains("SHELL=/bin/sh"));
        assert!(screen.contains("HOLLA_NO_HISTORY=1"));
        assert!(!screen.contains("TERMROCK_E2E_PARENT_SECRET_SENTINEL"));
        assert!(!screen.contains("TERMROCK_E2E_ENV_PROBE_MODE"));
        assert!(!screen.contains("HOME="));
        assert!(!screen.contains("PATH="));
        drop(session);
        println!("PTY_ENV_PROBE_OK");
    }

    #[cfg(unix)]
    #[test]
    fn prepared_only_preflight_child_process() {
        if std::env::var_os("TERMROCK_E2E_PREFLIGHT_CHILD").is_none() {
            return;
        }
        let prepared = prepare_holla_help_overlay_preflight()
            .expect("prepare the synthetic paired Holla policy");
        assert_eq!(prepared.state, "prepared_only");
        assert_eq!(prepared.trust_record.state, "required_absent");
        println!(
            "TERMROCK_E2E_PREFLIGHT_PREPARED={}",
            serde_json::to_string(&prepared).expect("serialize prepared-only record")
        );
    }

    #[cfg(unix)]
    #[test]
    fn prepared_only_preflight_does_not_launch_or_write_capture_or_receipt() {
        use std::os::unix::fs::PermissionsExt;

        // This fixture is synthetic test material, not an approved oracle or product build.
        let mut evidence = build_evidence_chain_fixture();
        let expected = visual::synthetic_expected_generation_fixture();
        let write_root = unique_temp_path("preflight-write-root");
        let trust_root = unique_temp_path("preflight-trust-root");
        let oracle_root = unique_temp_path("preflight-oracle-root");
        let actual_parent = write_root.join("actual");
        fs::create_dir_all(&actual_parent).expect("create temporary write root");
        fs::create_dir_all(&trust_root).expect("create temporary trust parent");
        fs::create_dir_all(&oracle_root).expect("create temporary oracle root");

        let write_sentinel = write_root.join("existing-run-data.txt");
        fs::write(&write_sentinel, b"retain write-root data")
            .expect("write pre-existing write-root sentinel");
        let trust_sentinel = trust_root.join("external-owner-data.txt");
        fs::write(&trust_sentinel, b"retain trust-parent data")
            .expect("write pre-existing trust-parent sentinel");
        let trust_record_path = trust_root.join("future-trust-record.json");
        let receipt_path = write_root.join("run.json");

        let mut protected_roots = Vec::new();
        let mut launch_markers = Vec::new();
        for subject in &mut evidence.manifest.subjects {
            let role = subject.role.as_str();
            let artifact_root = evidence.run_root.join("artifacts").join(role);
            fs::create_dir_all(&artifact_root).expect("create synthetic executable root");
            let actual_root = actual_parent.join(role);
            fs::create_dir_all(&actual_root).expect("create actual output root");
            fs::write(
                actual_root.join("pre-existing-case.txt"),
                b"retain capture data",
            )
            .expect("write pre-existing capture sentinel");

            let marker = write_root.join(format!("{role}-launched.txt"));
            let marker_text = marker.display().to_string().replace('\'', "'\\''");
            let executable_path = artifact_root.join("holla");
            fs::write(
                &executable_path,
                format!("#!/bin/sh\nprintf launched > '{marker_text}'\n"),
            )
            .expect("write launch-detecting synthetic executable");
            fs::set_permissions(&executable_path, fs::Permissions::from_mode(0o755))
                .expect("make synthetic executable runnable");
            let executable_sha256 = sha256_file(&executable_path).expect("hash test executable");
            subject.executable = Executable {
                path: executable_path.clone(),
                sha256: executable_sha256.clone(),
            };
            subject.actual_output_root = actual_root.clone();
            subject.builder_receipt.executable_path = executable_path;
            subject.builder_receipt.executable_sha256 = executable_sha256;
            subject.builder_receipt.sha256 = builder_receipt_digest(&subject.builder_receipt)
                .expect("recompute synthetic builder receipt");
            subject.builder_receipt_sha256 = subject.builder_receipt.sha256.clone();

            let source_path = PathBuf::from(
                &evidence
                    .source_inputs
                    .sources
                    .get(role)
                    .expect("source input for each subject")
                    .path,
            );
            protected_roots.push(ProtectedRoot {
                kind: "subject_source".to_string(),
                role: Some(role.to_string()),
                path: source_path,
            });
            protected_roots.push(ProtectedRoot {
                kind: "subject_artifact".to_string(),
                role: Some(role.to_string()),
                path: artifact_root,
            });
            launch_markers.push(marker);
        }
        protected_roots.push(ProtectedRoot {
            kind: "oracle".to_string(),
            role: None,
            path: oracle_root.clone(),
        });

        evidence.manifest.suite_sha256 = suite_digest().expect("hash frozen suite");
        evidence.manifest.expected_generation = Some(expected.input.clone());
        validate_subject_manifest(&evidence.manifest).expect("validate synthetic subject pair");
        let manifest_bytes =
            serde_json::to_vec(&evidence.manifest).expect("serialize synthetic subject manifest");
        fs::write(&evidence.manifest_path, &manifest_bytes)
            .expect("write synthetic subject manifest");
        let manifest_sha256 = sha256_bytes(&manifest_bytes);
        let mut builder_run: serde_json::Value = serde_json::from_slice(
            &fs::read(&evidence.builder_run_path).expect("read synthetic builder run"),
        )
        .expect("parse synthetic builder run");
        builder_run["subject_manifest_sha256"] = serde_json::json!(manifest_sha256);
        fs::write(
            &evidence.builder_run_path,
            serde_json::to_vec(&builder_run).expect("serialize synthetic builder run"),
        )
        .expect("update synthetic builder run binding");

        let case = registry()
            .expect("load Holla case registry")
            .cases
            .remove(0);
        let expected_tree_before =
            digest_tree(expected.root.as_path()).expect("hash expected tree");
        let evidence_paths = [
            evidence.manifest_path.clone(),
            evidence.source_inputs_path.clone(),
            evidence.build_environment_path.clone(),
            evidence.builder_run_path.clone(),
        ];
        let evidence_before = evidence_paths
            .iter()
            .map(|path| fs::read(path).expect("read evidence before preflight"))
            .collect::<Vec<_>>();
        let actual_sentinels = ["reference", "candidate"]
            .map(|role| actual_parent.join(role).join("pre-existing-case.txt"));
        let actual_before = actual_sentinels
            .iter()
            .map(|path| fs::read(path).expect("read capture sentinel before preflight"))
            .collect::<Vec<_>>();
        let mut protected_tree_roots = vec![
            ("oracle_root".to_string(), oracle_root.clone()),
            ("evidence.run_root".to_string(), evidence.run_root.clone()),
            ("write_root".to_string(), write_root.clone()),
            ("trust_root".to_string(), trust_root.clone()),
            (
                "expected_generation.root".to_string(),
                expected.root.clone(),
            ),
        ];
        protected_tree_roots.extend(evidence.manifest.subjects.iter().map(|subject| {
            (
                format!("actual_output_root.{}", subject.role),
                subject.actual_output_root.clone(),
            )
        }));
        let protected_tree_before = protected_tree_roots
            .iter()
            .map(|(_, root)| protected_tree_inventory(root))
            .collect::<Vec<_>>();
        let protected_json =
            serde_json::to_string(&protected_roots).expect("serialize protected roots");
        let executable = std::env::current_exe().expect("locate test binary");
        let output = std::process::Command::new(executable)
            .args([
                "--exact",
                "tests::prepared_only_preflight_child_process",
                "--nocapture",
            ])
            .env_clear()
            .env("TERMROCK_E2E_PREFLIGHT_CHILD", "1")
            .env("TERMROCK_E2E_SUBJECT_MANIFEST", &evidence.manifest_path)
            .env("TERMROCK_E2E_TRUST_RECORD", &trust_record_path)
            .env("TERMROCK_E2E_WRITE_ROOT", &write_root)
            .env("TERMROCK_E2E_RECEIPT_PATH", &receipt_path)
            .env("TERMROCK_E2E_PROTECTED_ROOTS", protected_json)
            .output()
            .expect("run isolated preflight child process");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "prepared-only child failed: stdout={stdout} stderr={stderr}"
        );
        let json_line = stdout
            .lines()
            .find_map(|line| line.strip_prefix("TERMROCK_E2E_PREFLIGHT_PREPARED="))
            .expect("child emitted prepared-only receipt marker");
        let prepared: serde_json::Value =
            serde_json::from_str(json_line).expect("parse prepared-only receipt");
        assert_eq!(prepared["state"], "prepared_only");
        assert_eq!(prepared["trust_record"]["state"], "required_absent");
        assert_eq!(
            prepared["expected_generation"]["id"],
            "synthetic-not-admitted"
        );

        assert!(!receipt_path.exists(), "preflight wrote a run receipt");
        assert!(
            !trust_record_path.exists(),
            "preflight created the future trust record"
        );
        assert_eq!(
            fs::read(&write_sentinel).unwrap(),
            b"retain write-root data"
        );
        assert_eq!(
            fs::read(&trust_sentinel).unwrap(),
            b"retain trust-parent data"
        );
        assert_eq!(
            actual_sentinels
                .iter()
                .map(|path| fs::read(path).expect("read capture sentinel after preflight"))
                .collect::<Vec<_>>(),
            actual_before,
            "preflight changed a pre-existing capture file"
        );
        assert!(launch_markers.iter().all(|path| !path.exists()));
        assert_eq!(
            digest_tree(expected.root.as_path()).unwrap(),
            expected_tree_before
        );
        assert_eq!(
            evidence_paths
                .iter()
                .map(|path| fs::read(path).expect("read evidence after preflight"))
                .collect::<Vec<_>>(),
            evidence_before,
            "preflight changed build evidence"
        );
        for ((label, root), before) in protected_tree_roots.iter().zip(&protected_tree_before) {
            assert_eq!(
                &protected_tree_inventory(root),
                before,
                "preflight changed the complete {label} path/type/hash inventory at {}",
                root.display()
            );
        }
        for checkpoint in checkpoints(&case) {
            let relative = capture_relative_stem(&case, checkpoint).expect("derive capture path");
            for role in ["reference", "candidate"] {
                assert!(!actual_parent.join(role).join(&relative).exists());
            }
        }

        fs::remove_dir_all(&evidence.run_root).expect("remove synthetic evidence fixture");
        fs::remove_dir_all(&write_root).expect("remove preflight write fixture");
        fs::remove_dir_all(&trust_root).expect("remove trust fixture");
        fs::remove_dir_all(&oracle_root).expect("remove oracle fixture");
    }

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
        assert_eq!(dimension_status(&checks, "exit"), Some("NOT_RUN"));
        assert_eq!(dimension_status(&checks, "restoration"), Some("NOT_RUN"));
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
        assert_eq!(dimension_status(&checks, "exit"), Some("NOT_RUN"));
        assert_eq!(dimension_status(&checks, "restoration"), Some("NOT_RUN"));
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

    #[cfg(unix)]
    #[test]
    fn actual_output_roots_reject_equal_nested_and_expected_overlap() {
        let base = unique_temp_path("roots");
        let reference_root = base.join("reference");
        fs::create_dir_all(reference_root.join("nested")).expect("create nested roots");
        let reference = test_subject("reference", reference_root.clone());
        let equal = test_subject("candidate", reference_root.clone());
        assert!(
            validate_actual_output_roots(&[reference.clone(), equal], None)
                .unwrap_err()
                .contains("overlap")
        );

        let nested = test_subject("candidate", reference_root.join("nested"));
        assert!(
            validate_actual_output_roots(&[reference.clone(), nested], None)
                .unwrap_err()
                .contains("overlap")
        );

        assert!(
            validate_actual_output_roots(&[reference], Some(&reference_root.join("expected")))
                .unwrap_err()
                .contains("overlap")
        );
        fs::remove_dir_all(&base).expect("remove actual-root fixture");
    }

    #[cfg(unix)]
    #[test]
    fn actual_output_roots_require_existing_directories_but_allow_nonempty_roots() {
        let base = unique_temp_path("existing-output-roots");
        let reference_root = base.join("reference");
        let candidate_root = base.join("candidate");
        fs::create_dir_all(&reference_root).expect("create reference output root");
        fs::create_dir_all(&candidate_root).expect("create candidate output root");
        fs::write(
            reference_root.join("prior-case.txt"),
            b"kept reference case",
        )
        .expect("write existing reference case");
        fs::write(
            candidate_root.join("prior-case.txt"),
            b"kept candidate case",
        )
        .expect("write existing candidate case");

        let subjects = [
            test_subject("reference", reference_root.clone()),
            test_subject("candidate", candidate_root.clone()),
        ];
        assert!(validate_actual_output_roots(&subjects, None).is_ok());

        let missing = test_subject("candidate", base.join("missing-candidate"));
        let error =
            validate_actual_output_roots(&[subjects[0].clone(), missing], None).unwrap_err();
        assert!(error.contains("actual output root"));
        assert_eq!(
            fs::read(reference_root.join("prior-case.txt")).expect("read prior case"),
            b"kept reference case"
        );
        fs::remove_dir_all(&base).expect("remove existing output-root fixture");
    }

    #[cfg(unix)]
    #[test]
    fn actual_output_roots_reject_existing_symlink_ancestors() {
        use std::os::unix::fs::symlink;

        let base = unique_temp_path("symlink-roots");
        let real = base.join("real");
        let alias = base.join("alias");
        fs::create_dir_all(real.join("actual/nested")).expect("create canonical roots");
        symlink(&real, &alias).expect("create symlink alias");
        let reference = test_subject("reference", real.join("actual"));
        let candidate = test_subject("candidate", alias.join("actual/nested"));
        let result = validate_actual_output_roots(&[reference, candidate], None);
        fs::remove_dir_all(&base).expect("remove temporary symlink tree");
        assert!(result.unwrap_err().contains("symlink"));
    }

    #[cfg(unix)]
    #[test]
    fn actual_output_roots_reject_equal_filesystem_identity_aliases() {
        let reference = ResolvedDirectory {
            path: PathBuf::from("/tmp/termrock/reference"),
            identity: DirectoryIdentity {
                device: 7,
                inode: 42,
            },
            ancestors: vec![
                DirectoryIdentity {
                    device: 7,
                    inode: 1,
                },
                DirectoryIdentity {
                    device: 7,
                    inode: 42,
                },
            ],
        };
        let candidate = ResolvedDirectory {
            path: PathBuf::from("/tmp/termrock/REFERENCE"),
            identity: DirectoryIdentity {
                device: 7,
                inode: 42,
            },
            ancestors: vec![
                DirectoryIdentity {
                    device: 7,
                    inode: 1,
                },
                DirectoryIdentity {
                    device: 7,
                    inode: 42,
                },
            ],
        };
        assert!(physical_directories_overlap(&reference, &candidate));
    }

    #[cfg(unix)]
    #[test]
    fn native_case_alias_probe_rejects_alias_or_reports_case_sensitive_volume() {
        let base = unique_temp_path("native-case-alias");
        let reference_root = base.join("Reference");
        let candidate_root = base.join("reference");
        fs::create_dir_all(&reference_root).expect("create reference output root");

        match fs::symlink_metadata(&candidate_root) {
            Ok(candidate_metadata) => {
                let reference_resolved = resolve_existing_directory_without_symlinks(
                    &reference_root,
                    "reference output root",
                )
                .expect("resolve reference output root");
                let candidate_resolved = resolve_existing_directory_without_symlinks(
                    &candidate_root,
                    "candidate output root",
                )
                .expect("resolve candidate output root");
                let candidate_identity = directory_identity(&candidate_metadata, &candidate_root)
                    .expect("read candidate directory identity");
                if candidate_identity == reference_resolved.identity {
                    assert!(physical_directories_overlap(
                        &reference_resolved,
                        &candidate_resolved
                    ));
                    let subjects = [
                        test_subject("reference", reference_root.clone()),
                        test_subject("candidate", candidate_root.clone()),
                    ];
                    assert!(
                        validate_actual_output_roots(&subjects, None)
                            .unwrap_err()
                            .contains("overlap")
                    );
                } else {
                    eprintln!(
                        "UNSUPPORTED: case-only output-root paths are distinct on this filesystem"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "UNSUPPORTED: case-only output-root lookup is distinct on this filesystem"
                );
            }
            Err(error) => panic!("probe candidate case alias: {error}"),
        }
        fs::remove_dir_all(&base).expect("remove case-alias fixture");
    }

    #[cfg(unix)]
    #[test]
    fn native_case_alias_nested_roots_are_rejected_by_ancestor_identity() {
        let base = unique_temp_path("native-case-nested");
        let reference_root = base.join("Reference");
        let candidate_root = base.join("reference/nested");
        fs::create_dir_all(&candidate_root).expect("create nested candidate root");
        match fs::symlink_metadata(&reference_root) {
            Ok(_) => {
                let reference = resolve_existing_directory_without_symlinks(
                    &reference_root,
                    "reference output root",
                )
                .expect("resolve reference output root");
                let candidate = resolve_existing_directory_without_symlinks(
                    &candidate_root,
                    "candidate output root",
                )
                .expect("resolve candidate output root");
                if candidate.ancestors.contains(&reference.identity) {
                    assert!(physical_directories_overlap(&reference, &candidate));
                    let subjects = [
                        test_subject("reference", reference_root.clone()),
                        test_subject("candidate", candidate_root.clone()),
                    ];
                    assert!(
                        validate_actual_output_roots(&subjects, None)
                            .unwrap_err()
                            .contains("overlap")
                    );
                } else {
                    eprintln!(
                        "UNSUPPORTED: case-only ancestor lookup is distinct on this filesystem"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("UNSUPPORTED: case-only ancestor lookup is distinct on this filesystem");
            }
            Err(error) => panic!("probe reference case alias: {error}"),
        }
        fs::remove_dir_all(&base).expect("remove nested case-alias fixture");
    }

    #[cfg(unix)]
    #[test]
    fn capture_preflight_allows_nonempty_shared_roots_with_new_case_leaves() {
        let registry = registry().expect("load case registry");
        let case = registry
            .cases
            .iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("find Holla help case");
        let base = unique_temp_path("capture-preflight-shared-roots");
        let reference_root = base.join("reference");
        let candidate_root = base.join("candidate");
        fs::create_dir_all(&reference_root).expect("create reference output root");
        fs::create_dir_all(&candidate_root).expect("create candidate output root");
        fs::write(reference_root.join("other-case.txt"), b"retained")
            .expect("write prior case in reference root");
        fs::write(candidate_root.join("other-case.txt"), b"retained")
            .expect("write prior case in candidate root");
        let roots = [
            ActualOutputRoot {
                role: "reference".to_string(),
                path: reference_root.clone(),
            },
            ActualOutputRoot {
                role: "candidate".to_string(),
                path: candidate_root.clone(),
            },
        ];

        validate_case_capture_paths(case, &roots)
            .expect("allow existing sibling data when Holla capture leaves are new");
        assert_eq!(
            fs::read(reference_root.join("other-case.txt")).expect("read prior reference case"),
            b"retained"
        );
        assert_eq!(
            fs::read(candidate_root.join("other-case.txt")).expect("read prior candidate case"),
            b"retained"
        );
        fs::remove_dir_all(&base).expect("remove shared-root capture fixture");
    }

    #[cfg(unix)]
    #[test]
    fn capture_preflight_rejects_existing_checkpoint_leaf_before_launch() {
        let registry = registry().expect("load case registry");
        let case = registry
            .cases
            .iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("find Holla help case");
        let base = unique_temp_path("capture-preflight-existing-leaf");
        let reference_root = base.join("reference");
        let candidate_root = base.join("candidate");
        fs::create_dir_all(&reference_root).expect("create reference output root");
        fs::create_dir_all(&candidate_root).expect("create candidate output root");
        let checkpoint = checkpoints(case)[0];
        let relative = capture_relative_stem(case, checkpoint).expect("derive capture path");
        let occupied_leaf = reference_root.join(relative);
        fs::create_dir_all(&occupied_leaf).expect("create existing capture leaf");
        let roots = [
            ActualOutputRoot {
                role: "reference".to_string(),
                path: reference_root.clone(),
            },
            ActualOutputRoot {
                role: "candidate".to_string(),
                path: candidate_root.clone(),
            },
        ];

        assert!(
            validate_case_capture_paths(case, &roots)
                .unwrap_err()
                .contains("already exists before launch")
        );
        fs::remove_dir_all(&base).expect("remove occupied capture fixture");
    }

    #[cfg(unix)]
    #[test]
    fn capture_preflight_rejects_existing_checkpoint_leaf_through_native_case_alias() {
        let registry = registry().expect("load case registry");
        let case = registry
            .cases
            .iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("find Holla help case");
        let base = unique_temp_path("capture-preflight-case-alias");
        let reference_root = base.join("reference");
        let candidate_root = base.join("candidate");
        fs::create_dir_all(&reference_root).expect("create reference output root");
        fs::create_dir_all(&candidate_root).expect("create candidate output root");

        let checkpoint = checkpoints(case)[0];
        let original_relative =
            capture_relative_stem(case, checkpoint).expect("derive original capture path");
        let occupied_leaf = reference_root.join(&original_relative);
        fs::create_dir_all(&occupied_leaf).expect("create existing capture leaf");

        let mut case_alias = case.clone();
        case_alias.app = case.app.to_ascii_uppercase();
        assert_ne!(
            case_alias.app, case.app,
            "case fixture must vary app spelling"
        );
        let alias_relative =
            capture_relative_stem(&case_alias, checkpoint).expect("derive case-alias capture path");
        let alias_leaf = reference_root.join(alias_relative);
        let output_roots = [
            ActualOutputRoot {
                role: "reference".to_string(),
                path: reference_root.clone(),
            },
            ActualOutputRoot {
                role: "candidate".to_string(),
                path: candidate_root.clone(),
            },
        ];

        match fs::symlink_metadata(&alias_leaf) {
            Ok(alias_metadata) => {
                let original_metadata = fs::symlink_metadata(&occupied_leaf)
                    .expect("inspect original existing capture leaf");
                let original_identity = directory_identity(&original_metadata, &occupied_leaf)
                    .expect("read original capture leaf identity");
                let alias_identity = directory_identity(&alias_metadata, &alias_leaf)
                    .expect("read case-alias capture leaf identity");
                if original_identity == alias_identity {
                    assert!(
                        validate_case_capture_paths(&case_alias, &output_roots)
                            .unwrap_err()
                            .contains("already exists before launch")
                    );
                } else {
                    eprintln!(
                        "UNSUPPORTED: case-only checkpoint lookup did not resolve to the same directory identity"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "UNSUPPORTED: case-only checkpoint lookup is distinct on this filesystem"
                );
            }
            Err(error) => panic!("probe case-alias checkpoint leaf: {error}"),
        }

        assert!(occupied_leaf.is_dir());
        fs::remove_dir_all(&base).expect("remove case-alias capture fixture");
    }

    #[cfg(unix)]
    #[test]
    fn capture_preflight_rejects_symlinked_checkpoint_ancestor_before_launch() {
        use std::os::unix::fs::symlink;

        let registry = registry().expect("load case registry");
        let case = registry
            .cases
            .iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("find Holla help case");
        let base = unique_temp_path("capture-preflight-symlink");
        let reference_root = base.join("reference");
        let candidate_root = base.join("candidate");
        let outside = base.join("outside");
        fs::create_dir_all(&reference_root).expect("create reference output root");
        fs::create_dir_all(&candidate_root).expect("create candidate output root");
        fs::create_dir_all(&outside).expect("create outside directory");
        let checkpoint = checkpoints(case)[0];
        let relative = capture_relative_stem(case, checkpoint).expect("derive capture path");
        let first_component = Path::new(&relative)
            .components()
            .next()
            .and_then(|component| match component {
                Component::Normal(name) => Some(name),
                _ => None,
            })
            .expect("capture path has a first component");
        symlink(&outside, reference_root.join(first_component))
            .expect("link checkpoint ancestor outside output root");
        let roots = [
            ActualOutputRoot {
                role: "reference".to_string(),
                path: reference_root.clone(),
            },
            ActualOutputRoot {
                role: "candidate".to_string(),
                path: candidate_root.clone(),
            },
        ];

        assert!(
            validate_case_capture_paths(case, &roots)
                .unwrap_err()
                .contains("contains a symlink before launch")
        );
        assert!(outside.is_dir());
        fs::remove_dir_all(&base).expect("remove symlink capture fixture");
    }

    #[cfg(unix)]
    #[test]
    fn receipt_parent_rejects_symlink_before_policy_resolution() {
        use std::os::unix::fs::symlink;

        let base = unique_temp_path("receipt-parent-symlink");
        let real_parent = base.join("real-parent");
        let linked_parent = base.join("linked-parent");
        fs::create_dir_all(&real_parent).expect("create receipt parent target");
        symlink(&real_parent, &linked_parent).expect("create receipt parent symlink");
        let result = canonical_new_file_path(&linked_parent.join("run.json"), "run receipt");
        assert!(result.unwrap_err().contains("symlink"));
        assert!(!real_parent.join("run.json").exists());
        fs::remove_dir_all(&base).expect("remove receipt parent fixture");
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
    fn fixed_build_evidence_files_are_raw_hashed_and_symlink_free() {
        let run_root = unique_temp_path("evidence-files");
        fs::create_dir_all(&run_root).expect("create evidence run root");
        let run_root = fs::canonicalize(run_root).expect("canonicalize evidence run root");
        let source_path = run_root.join("source-inputs.json");
        let source_bytes = br#"{"schema":"fixture"}"#;
        fs::write(&source_path, source_bytes).expect("write source input evidence");
        let evidence = EvidenceReference {
            path: "source-inputs.json".to_string(),
            sha256: sha256_bytes(source_bytes),
        };
        assert_eq!(
            resolve_fixed_evidence_file(&run_root, &evidence, "source-inputs.json").unwrap(),
            source_path
        );
        assert!(
            verify_evidence_digest(
                "source-inputs.json",
                &"0".repeat(64),
                &sha256_bytes(source_bytes)
            )
            .unwrap_err()
            .contains("SHA-256 mismatch")
        );

        let wrong_name = EvidenceReference {
            path: "../oracle/source-inputs.json".to_string(),
            sha256: evidence.sha256.clone(),
        };
        assert!(resolve_fixed_evidence_file(&run_root, &wrong_name, "source-inputs.json").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let outside = unique_temp_path("evidence-outside");
            fs::write(&outside, source_bytes).expect("write external evidence target");
            fs::remove_file(&source_path).expect("remove regular evidence file");
            symlink(&outside, &source_path).expect("replace evidence file with symlink");
            assert!(
                resolve_fixed_evidence_file(&run_root, &evidence, "source-inputs.json")
                    .unwrap_err()
                    .contains("symlink")
            );
            fs::remove_file(outside).expect("remove external evidence target");
        }

        fs::remove_dir_all(run_root).expect("remove evidence fixture");
    }

    #[test]
    fn build_evidence_chain_binds_manifest_sidecars_and_source_pair() {
        let fixture = build_evidence_chain_fixture();
        let validate = |source_inputs: &SourceInputsEvidence,
                        builder_run: &BuilderRunEvidence,
                        build_environment: &BuildEnvironmentEvidence| {
            validate_build_evidence_chain(
                &fixture.manifest,
                &fixture.manifest_path,
                &fixture.manifest_sha256,
                source_inputs,
                &fixture.source_inputs_path,
                &fixture.source_inputs_sha256,
                build_environment,
                &fixture.build_environment_path,
                &fixture.build_environment_sha256,
                builder_run,
                &fixture.builder_run_path,
                &fixture.builder_run_sha256,
            )
        };

        validate(
            &fixture.source_inputs,
            &fixture.builder_run,
            &fixture.build_environment,
        )
        .expect("matching builder evidence chain validates");

        let mut distinct_reason = fixture.builder_run.clone();
        distinct_reason.qualification.reason = "builder wording differs from source sidecar".into();
        validate(
            &fixture.source_inputs,
            &distinct_reason,
            &fixture.build_environment,
        )
        .expect("independent nonempty blocked reasons remain valid");

        let mut wrong_source_status = fixture.source_inputs.clone();
        wrong_source_status.qualification_status = "qualified".into();
        assert!(
            validate(
                &wrong_source_status,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut wrong_run_status = fixture.builder_run.clone();
        wrong_run_status.qualification.status = "qualified".into();
        assert!(
            validate(
                &fixture.source_inputs,
                &wrong_run_status,
                &fixture.build_environment,
            )
            .is_err()
        );

        let mut wrong_tag_ref = fixture.source_inputs.clone();
        wrong_tag_ref.oracle_lineage.tag_ref = "refs/tags/other".into();
        assert!(
            validate(
                &wrong_tag_ref,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut wrong_tag_object = fixture.source_inputs.clone();
        wrong_tag_object.oracle_lineage.tag_object = "1".repeat(40);
        assert!(
            validate(
                &wrong_tag_object,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut wrong_tag_commit = fixture.source_inputs.clone();
        wrong_tag_commit.oracle_lineage.tag_commit = "2".repeat(40);
        assert!(
            validate(
                &wrong_tag_commit,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut wrong_branch_ref = fixture.source_inputs.clone();
        wrong_branch_ref.reference_branch.r#ref = "refs/remotes/origin/other".into();
        assert!(
            validate(
                &wrong_branch_ref,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut wrong_membership = fixture.source_inputs.clone();
        wrong_membership.reference_branch.membership = "reachable".into();
        assert!(
            validate(
                &wrong_membership,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut different_branch_oid = fixture.source_inputs.clone();
        different_branch_oid.reference_branch.oid = "b".repeat(40);
        assert!(
            validate(
                &different_branch_oid,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );
        let mut different_selected_commit = fixture.source_inputs.clone();
        different_selected_commit.reference_branch.selected_commit = "b".repeat(40);
        assert!(
            validate(
                &different_selected_commit,
                &fixture.builder_run,
                &fixture.build_environment,
            )
            .is_err()
        );

        for (path, parse_typed) in [
            (&fixture.source_inputs_path, "source inputs"),
            (&fixture.build_environment_path, "build environment"),
            (&fixture.builder_run_path, "builder run"),
        ] {
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(path).expect("read typed evidence fixture"))
                    .expect("parse typed evidence fixture JSON");
            value["unexpected_contract_field"] = serde_json::json!(true);
            let result = match parse_typed {
                "source inputs" => {
                    serde_json::from_value::<SourceInputsEvidence>(value).map(|_| ())
                }
                "build environment" => {
                    serde_json::from_value::<BuildEnvironmentEvidence>(value).map(|_| ())
                }
                "builder run" => serde_json::from_value::<BuilderRunEvidence>(value).map(|_| ()),
                _ => unreachable!(),
            };
            assert!(result.is_err(), "{parse_typed} accepted an unknown field");
        }

        let mut stale_manifest_binding = fixture.builder_run.clone();
        stale_manifest_binding.subject_manifest_sha256 = "0".repeat(64);
        assert!(
            validate(
                &fixture.source_inputs,
                &stale_manifest_binding,
                &fixture.build_environment,
            )
            .unwrap_err()
            .contains("raw subject manifest")
        );

        let mut mismatched_source = fixture.build_environment.clone();
        mismatched_source
            .source_commits
            .insert("candidate".to_string(), "f".repeat(40));
        let source_error = validate(
            &fixture.source_inputs,
            &fixture.builder_run,
            &mismatched_source,
        )
        .unwrap_err();
        assert!(source_error.contains("identity disagrees"));

        let mut mismatched_sidecar_digest = fixture.builder_run.clone();
        mismatched_sidecar_digest.source_input_evidence.sha256 = "0".repeat(64);
        assert!(
            validate(
                &fixture.source_inputs,
                &mismatched_sidecar_digest,
                &fixture.build_environment,
            )
            .unwrap_err()
            .contains("do not bind the manifest sidecars")
        );

        drop(validate);
        fs::remove_dir_all(fixture.run_root).expect("remove build evidence fixture");
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
        let mut manifest = SubjectManifest {
            schema: SUBJECT_SCHEMA.to_string(),
            run_id: "trust-test".to_string(),
            suite_revision: "test-revision".to_string(),
            suite_sha256: "a".repeat(64),
            expected_generation: None,
            build_evidence: test_build_evidence(),
            subjects: subjects.clone(),
        };
        let test_binary = "c".repeat(64);
        let write_policy_sha256 = "7".repeat(64);
        let evidence = test_loaded_evidence_context();
        let mut record = test_trust_record(&subjects, &test_binary);
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &evidence,
                None,
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
                &evidence,
                None,
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
                &evidence,
                None,
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
        manifest.expected_generation = Some(ExpectedGenerationInput {
            id: "shared-parity-v1".to_string(),
            root: None,
            sha256: "b".repeat(64),
        });
        record.expected_generation = Some(TrustedExpectedGeneration {
            id: manifest.expected_generation.as_ref().unwrap().id.clone(),
            sha256: manifest
                .expected_generation
                .as_ref()
                .unwrap()
                .sha256
                .clone(),
            admission_receipt_sha256: "9".repeat(64),
        });
        let expected_ok = Ok(manifest
            .expected_generation
            .as_ref()
            .unwrap()
            .sha256
            .clone());
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &evidence,
                Some(&expected_ok),
                &write_policy_sha256,
                &manifest.suite_sha256,
                &"d".repeat(64),
                &"e".repeat(64),
                &"f".repeat(64),
                &test_binary,
            )
            .is_ok()
        );
        let expected_error = Err("generation bytes mismatch".to_string());
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &evidence,
                Some(&expected_error),
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
            expected_generation: None,
            build_evidence: test_build_evidence(),
            subjects: subjects.clone(),
        };
        let test_binary = "c".repeat(64);
        let mut record = test_trust_record(&subjects, &test_binary);
        let evidence = test_loaded_evidence_context();
        record.write_policy_sha256 = "0".repeat(64);
        assert!(
            validate_trust_record(
                &record,
                &manifest,
                &subjects,
                &evidence,
                None,
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
    fn future_trust_record_path_requires_existing_parent_and_absent_leaf() {
        let fixture = test_write_policy_fixture("future-trust-path");
        let trust_path = fixture.trust_source.canonical_path.clone();
        assert!(
            canonical_absent_file_path(&trust_path, "future trust record")
                .unwrap_err()
                .contains("must be absent")
        );

        fs::remove_file(&trust_path).expect("remove fixture trust record");
        let canonical = canonical_absent_file_path(&trust_path, "future trust record")
            .expect("accept absent trust record under existing parent");
        assert_eq!(canonical, trust_path);
        assert!(!canonical.exists());

        let missing_parent = fixture.base.join("missing/future-trust.json");
        assert!(canonical_absent_file_path(&missing_parent, "future trust record").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let linked_parent = fixture.base.join("linked-trust-parent");
            symlink(fixture.base.join("oracle"), &linked_parent)
                .expect("create future trust parent symlink");
            assert!(
                canonical_absent_file_path(
                    &linked_parent.join("future-trust.json"),
                    "future trust record"
                )
                .unwrap_err()
                .contains("contains a symlink")
            );
            fs::remove_file(&linked_parent).expect("remove future trust parent symlink");

            symlink(fixture.base.join("oracle"), &trust_path)
                .expect("create future trust leaf symlink");
            assert!(
                canonical_absent_file_path(&trust_path, "future trust record")
                    .unwrap_err()
                    .contains("must be absent")
            );
            assert!(fixture.base.join("oracle").is_dir());
            fs::remove_file(&trust_path).expect("remove trust leaf symlink");
        }

        assert!(!fixture.input.receipt_path.exists());
        assert!(!fixture.base.join("oracle/run.json").exists());
        fs::remove_dir_all(&fixture.base).expect("remove future trust fixture");
    }

    #[test]
    fn write_policy_digest_matches_before_and_after_future_trust_record_creation() {
        let fixture = test_write_policy_fixture("future-trust-policy");
        let trust_path = fixture.trust_source.canonical_path.clone();
        fs::remove_file(&trust_path).expect("remove fixture trust record");

        let future_path = canonical_absent_file_path(&trust_path, "future trust record")
            .expect("resolve absent future trust path");
        let prepared = resolve_test_write_policy_at_trust_path(&fixture, None, &future_path)
            .expect("resolve preflight policy");
        assert!(!trust_path.exists());
        assert!(!fixture.input.receipt_path.exists());

        fs::write(
            &trust_path,
            b"external trust record created after preflight",
        )
        .expect("write fixture trust record after preparation");
        let accepted_path =
            canonical_existing_regular_file_without_symlinks(&trust_path, "external trust record")
                .expect("resolve existing trust record path");
        let accepted = resolve_test_write_policy_at_trust_path(&fixture, None, &accepted_path)
            .expect("resolve acceptance policy");
        assert_eq!(prepared, accepted);
        assert!(!fixture.input.receipt_path.exists());
        for subject in &fixture.subjects {
            let mut entries =
                fs::read_dir(&subject.actual_output_root).expect("inspect actual output root");
            assert!(
                entries.next().is_none(),
                "preflight wrote an actual capture"
            );
        }

        fs::remove_dir_all(&fixture.base).expect("remove future trust policy fixture");
    }

    #[test]
    fn write_policy_rejects_future_trust_file_inside_caller_protected_root() {
        let fixture = test_write_policy_fixture("future-trust-protected");
        let source_root = fixture
            .input
            .protected_roots
            .iter()
            .find(|root| root.kind == "subject_source" && root.role.as_deref() == Some("reference"))
            .expect("reference source root is protected");
        let trust_path = source_root.path.join("future-trust.json");
        let canonical = canonical_absent_file_path(&trust_path, "future trust record")
            .expect("resolve absent trust path inside protected source root");
        let error =
            resolve_test_write_policy_at_trust_path(&fixture, None, &canonical).unwrap_err();
        assert!(error.contains(
            "external trust record parent must be outside protected subject_source root"
        ));
        assert!(!trust_path.exists());
        assert!(!fixture.input.receipt_path.exists());
        fs::remove_dir_all(&fixture.base).expect("remove protected future trust fixture");
    }

    #[cfg(unix)]
    #[test]
    fn external_trust_parent_uses_physical_ancestry_and_identity_aliases() {
        let identity = |inode| DirectoryIdentity { device: 7, inode };
        let root_id = identity(1);
        let shared_id = identity(2);
        let protected_id = identity(3);
        let nested_id = identity(4);

        let trust_parent = ResolvedDirectory {
            path: PathBuf::from("/tmp/shared/trust"),
            identity: shared_id,
            ancestors: vec![root_id, shared_id],
        };
        let protected_child = ResolvedDirectory {
            path: PathBuf::from("/tmp/shared/trust/source"),
            identity: protected_id,
            ancestors: vec![root_id, shared_id, protected_id],
        };
        assert!(
            validate_external_trust_parent_relation(
                &trust_parent,
                &protected_child,
                "subject_source",
                &protected_child.path,
            )
            .is_ok()
        );

        let trust_beneath_root = ResolvedDirectory {
            path: PathBuf::from("/tmp/shared/trust/source/external"),
            identity: nested_id,
            ancestors: vec![root_id, shared_id, protected_id, nested_id],
        };
        assert!(
            validate_external_trust_parent_relation(
                &trust_beneath_root,
                &protected_child,
                "subject_source",
                &protected_child.path,
            )
            .unwrap_err()
            .contains("must be outside protected subject_source")
        );

        let equal_identity_alias = ResolvedDirectory {
            path: PathBuf::from("/tmp/shared/SOURCE"),
            identity: protected_id,
            ancestors: vec![root_id, shared_id, protected_id],
        };
        assert!(
            validate_external_trust_parent_relation(
                &equal_identity_alias,
                &protected_child,
                "subject_source",
                &protected_child.path,
            )
            .unwrap_err()
            .contains("must be outside protected subject_source")
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_policy_rejects_trust_parent_with_native_case_alias_of_protected_root() {
        let fixture = test_write_policy_fixture("future-trust-native-case-parent");
        let source_root = fixture
            .input
            .protected_roots
            .iter()
            .find(|root| root.kind == "subject_source" && root.role.as_deref() == Some("reference"))
            .expect("reference source root is protected")
            .path
            .clone();
        let alias_parent = source_root.with_file_name("SOURCE-REFERENCE");

        match fs::symlink_metadata(&alias_parent) {
            Ok(_) => {
                let source = resolve_existing_directory_without_symlinks(
                    &source_root,
                    "reference source root",
                )
                .expect("resolve reference source root");
                let alias = resolve_existing_directory_without_symlinks(
                    &alias_parent,
                    "trust parent alias",
                )
                .expect("resolve trust parent alias");
                if source.identity == alias.identity {
                    let trust_path = canonical_absent_file_path(
                        &alias_parent.join("future-trust.json"),
                        "future trust record",
                    )
                    .expect("resolve trust leaf below aliased parent");
                    let error =
                        resolve_test_write_policy_at_trust_path(&fixture, None, &trust_path)
                            .unwrap_err();
                    assert!(error.contains(
                        "external trust record parent must be outside protected subject_source"
                    ));
                    assert!(!trust_path.exists());
                } else {
                    eprintln!(
                        "UNSUPPORTED: case-only trust-parent paths are distinct on this filesystem"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "UNSUPPORTED: case-only trust-parent lookup is distinct on this filesystem"
                );
            }
            Err(error) => panic!("probe trust-parent case alias: {error}"),
        }

        assert!(!fixture.input.receipt_path.exists());
        fs::remove_dir_all(&fixture.base).expect("remove native-case trust-parent fixture");
    }

    #[cfg(unix)]
    #[test]
    fn write_policy_allows_trust_parent_as_physical_ancestor_of_protected_root() {
        let mut fixture = test_write_policy_fixture("future-trust-parent-ancestor");
        let trust_parent = fixture
            .trust_source
            .canonical_path
            .parent()
            .expect("trust record parent")
            .to_path_buf();
        let source_index = fixture
            .input
            .protected_roots
            .iter()
            .position(|root| {
                root.kind == "subject_source" && root.role.as_deref() == Some("reference")
            })
            .expect("reference source root is protected");
        let old_source_root = fixture.input.protected_roots[source_index].path.clone();
        let nested_source_root = trust_parent.join("source-reference");
        fs::rename(&old_source_root, &nested_source_root)
            .expect("move source fixture below trust parent");
        fixture.input.protected_roots[source_index].path = nested_source_root.clone();

        let policy = resolve_test_write_policy(&fixture, None)
            .expect("allow strict trust-parent ancestor with sibling trust leaf");
        let stored_parent = policy
            .protected_roots
            .iter()
            .find(|root| root.kind == "trust_record_parent")
            .expect("policy records trust parent");
        assert_eq!(stored_parent.path, trust_parent);
        assert!(nested_source_root.is_dir());
        assert!(!fixture.input.receipt_path.exists());
        fs::remove_dir_all(&fixture.base).expect("remove trust-parent ancestor fixture");
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
        fs::create_dir_all(&outside_output[0].actual_output_root)
            .expect("create outside actual output root");
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

    fn test_build_evidence() -> SubjectBuildEvidence {
        SubjectBuildEvidence {
            source_inputs: EvidenceReference {
                path: "source-inputs.json".to_string(),
                sha256: "a".repeat(64),
            },
            build_environment: EvidenceReference {
                path: "build-environment.json".to_string(),
                sha256: "b".repeat(64),
            },
        }
    }

    fn load_synthetic_expected_generation(
        fixture: &visual::SyntheticExpectedGenerationFixture,
        case: &Case,
    ) -> visual::ExpectedGeneration {
        let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let profile: Profile = serde_json::from_str(PROFILE_JSON).expect("parse suite profile");
        let renderer_profile = tuiscotti::profile::RenderProfile::vendored();
        let renderer = renderer_identity(&renderer_profile);
        visual::load_expected_generation(
            &fixture.input,
            case,
            &digest_tree(&package_root.join("cases")).expect("hash case registry"),
            &sha256_file(&package_root.join("profile.json")).expect("hash profile"),
            &sha256_file(&package_root.join("Cargo.lock")).expect("hash dependency lock"),
            &profile.tool_revision,
            &renderer,
        )
        .expect("load synthetic expected generation")
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
            build_evidence: TrustedBuildEvidence {
                subject_manifest: EvidenceReference {
                    path: "/tmp/test-run/subject-manifest.json".to_string(),
                    sha256: "2".repeat(64),
                },
                builder_run: EvidenceReference {
                    path: "/tmp/test-run/run.json".to_string(),
                    sha256: "4".repeat(64),
                },
            },
            write_policy_sha256: "7".repeat(64),
        }
    }

    fn test_loaded_evidence_context() -> LoadedEvidenceContext {
        LoadedEvidenceContext {
            manifest_path: PathBuf::from("/tmp/test-run/subject-manifest.json"),
            manifest_sha256: "2".repeat(64),
            source_inputs_path: PathBuf::from("/tmp/test-run/source-inputs.json"),
            source_inputs_sha256: "a".repeat(64),
            build_environment_path: PathBuf::from("/tmp/test-run/build-environment.json"),
            build_environment_sha256: "b".repeat(64),
            builder_run_path: PathBuf::from("/tmp/test-run/run.json"),
            builder_run_sha256: "4".repeat(64),
        }
    }

    struct BuildEvidenceChainFixture {
        run_root: PathBuf,
        manifest: SubjectManifest,
        manifest_path: PathBuf,
        manifest_sha256: String,
        source_inputs: SourceInputsEvidence,
        source_inputs_path: PathBuf,
        source_inputs_sha256: String,
        build_environment: BuildEnvironmentEvidence,
        build_environment_path: PathBuf,
        build_environment_sha256: String,
        builder_run: BuilderRunEvidence,
        builder_run_path: PathBuf,
        builder_run_sha256: String,
    }

    fn build_evidence_chain_fixture() -> BuildEvidenceChainFixture {
        use serde_json::{Map, json};

        let run_root = unique_temp_path("evidence-chain");
        fs::create_dir_all(&run_root).expect("create builder evidence root");
        let run_root = fs::canonicalize(run_root).expect("canonicalize builder evidence root");
        let mut subjects = Vec::new();
        let mut sources = Map::new();
        let mut snapshots = Map::new();
        let mut environments = Map::new();
        let mut source_commits = Map::new();
        let mut cargo_configs = Map::new();
        let mut target_metadata = Map::new();
        let mut snapshot_materialization = Map::new();
        let replacement_policy = "disabled-by-option-and-environment";
        let recipe = "git-archive-pinned-commit-v1";
        let path_algorithm = "sha256-path-blob-map-v1";
        let reference_branch = json!({
            "ref": REFERENCE_BRANCH_REF,
            "oid": "a".repeat(40),
            "selected_commit": "a".repeat(40),
            "membership": REFERENCE_BRANCH_MEMBERSHIP
        });
        let oracle_lineage = json!({
            "tag_ref": ORACLE_TAG_REF,
            "tag_object": ORACLE_TAG_OBJECT,
            "tag_commit": ORACLE_TAG_COMMIT
        });

        for (role, commit_nibble) in [("reference", 'a'), ("candidate", 'b')] {
            let mut subject = test_subject(role, run_root.join("actual").join(role));
            subject.source_commit = commit_nibble.to_string().repeat(40);
            subject.builder_receipt.source_commit = subject.source_commit.clone();
            subject.builder_receipt.sha256 = builder_receipt_digest(&subject.builder_receipt)
                .expect("recompute fixture receipt");
            subject.builder_receipt_sha256 = subject.builder_receipt.sha256.clone();

            let snapshot_path = run_root.join("source").join(role);
            fs::create_dir_all(&snapshot_path).expect("create immutable snapshot fixture");
            let snapshot_path =
                fs::canonicalize(snapshot_path).expect("canonicalize snapshot fixture");
            let snapshot = json!({
                "role": role,
                "path": snapshot_path.display().to_string(),
                "source_commit": subject.source_commit,
                "tree_oid": subject.source_commit,
                "git_object_replacement_policy": replacement_policy,
                "recipe": recipe,
                "archive_argv": ["git", "archive", subject.source_commit],
                "git_ls_tree_stdout_sha256": "1".repeat(64),
                "excluded_roots": ["target"],
                "path_blob_digest_algorithm": path_algorithm,
                "tracked_path_blob_map_sha256": "2".repeat(64),
                "tracked_file_count": 10,
                "included_path_blob_map_sha256": "3".repeat(64),
                "included_file_count": 8,
                "excluded_path_blob_map_sha256": "4".repeat(64),
                "excluded_file_count": 2,
                "archive_sha256": "5".repeat(64),
                "archive_member_count": 10,
                "read_only": true
            });
            sources.insert(role.to_string(), snapshot.clone());
            snapshot_materialization.insert(role.to_string(), snapshot);
            snapshots.insert(
                role.to_string(),
                json!({
                    "commit": subject.source_commit,
                    "path": snapshot_path.display().to_string(),
                    "read_only": true
                }),
            );
            source_commits.insert(role.to_string(), json!(subject.source_commit));
            cargo_configs.insert(role.to_string(), json!({}));
            environments.insert(role.to_string(), json!({
                "home": run_root.join("environment").join(role).join("home").display().to_string(),
                "cargo_home": run_root.join("environment").join(role).join("cargo-home").display().to_string(),
                "cargo_home_config": "absent",
                "cache_home": "/tool-cache/cargo",
                "cache_links": {},
                "rustup_home": "/toolchain/rustup-home",
                "tmpdir": run_root.join("environment").join(role).join("tmp").display().to_string(),
                "path": "/toolchain/bin",
                "platform_inputs": {}
            }));
            target_metadata.insert(role.to_string(), json!({
                "holla": {
                    "package_id": subject.build.package_id,
                    "package_name": "holla",
                    "target_name": subject.build.target_name,
                    "manifest_path": snapshot_path.join("Cargo.toml").display().to_string(),
                    "source_path": snapshot_path.join("crates/holla/src/main.rs").display().to_string()
                }
            }));
            subjects.push(subject);
        }

        let source_inputs_value = json!({
            "schema": "termrock-spec/parity-subject-source-inputs-v1",
            "run_id": "evidence-chain-test",
            "qualification_status": "blocked",
            "qualification_reason": "fixture remains unqualified",
            "git_object_replacement_policy": replacement_policy,
            "oracle_lineage": oracle_lineage,
            "reference_branch": reference_branch,
            "recipe": recipe,
            "excluded_roots": ["target"],
            "path_blob_digest_algorithm": path_algorithm,
            "sources": sources
        });
        let source_inputs_path = run_root.join("source-inputs.json");
        let source_inputs_bytes =
            serde_json::to_vec(&source_inputs_value).expect("serialize source inputs");
        fs::write(&source_inputs_path, &source_inputs_bytes).expect("write source inputs");
        let source_inputs_path =
            fs::canonicalize(source_inputs_path).expect("canonicalize source inputs");
        let source_inputs_sha256 = sha256_bytes(&source_inputs_bytes);
        let source_input_reference = json!({
            "path": source_inputs_path.display().to_string(),
            "sha256": source_inputs_sha256
        });

        let toolchain_binaries = json!({
            "cargo": {"path": "/toolchain/bin/cargo", "sha256": "6".repeat(64)},
            "rustc": {"path": "/toolchain/bin/rustc", "sha256": "7".repeat(64)}
        });
        let build_environment_value = json!({
            "schema": "termrock-spec/parity-subject-build-environment-v1",
            "run_id": "evidence-chain-test",
            "source_input_evidence": source_input_reference,
            "source_commits": source_commits,
            "reference_branch": reference_branch,
            "oracle_lineage": oracle_lineage,
            "cargo_config_sha256": cargo_configs,
            "external_cargo_config_policy": "reject-any-config-from-snapshot-parent-through-filesystem-root",
            "rustup": {
                "path": "/toolchain/bin/rustup",
                "sha256": "8".repeat(64),
                "home": "/toolchain/rustup-home"
            },
            "toolchain_binaries": toolchain_binaries,
            "cargo_version": "cargo 1.98.1",
            "rustc_version": "rustc 1.98.1",
            "host_triple": "aarch64-apple-darwin",
            "target_triple": "aarch64-apple-darwin",
            "environment": environments
        });
        let build_environment_path = run_root.join("build-environment.json");
        let build_environment_bytes =
            serde_json::to_vec(&build_environment_value).expect("serialize build environment");
        fs::write(&build_environment_path, &build_environment_bytes)
            .expect("write build environment");
        let build_environment_path =
            fs::canonicalize(build_environment_path).expect("canonicalize build environment");
        let build_environment_sha256 = sha256_bytes(&build_environment_bytes);
        let build_environment_reference = json!({
            "path": build_environment_path.display().to_string(),
            "sha256": build_environment_sha256
        });

        let manifest = SubjectManifest {
            schema: SUBJECT_SCHEMA.to_string(),
            run_id: "evidence-chain-test".to_string(),
            suite_revision: registry().expect("read suite revision").suite_revision,
            suite_sha256: "9".repeat(64),
            expected_generation: None,
            build_evidence: SubjectBuildEvidence {
                source_inputs: EvidenceReference {
                    path: "source-inputs.json".to_string(),
                    sha256: source_inputs_sha256.clone(),
                },
                build_environment: EvidenceReference {
                    path: "build-environment.json".to_string(),
                    sha256: build_environment_sha256.clone(),
                },
            },
            subjects,
        };
        let manifest_path = run_root.join("subject-manifest.json");
        let manifest_bytes = serde_json::to_vec(&manifest).expect("serialize subject manifest");
        fs::write(&manifest_path, &manifest_bytes).expect("write subject manifest");
        let manifest_path = fs::canonicalize(manifest_path).expect("canonicalize subject manifest");
        let manifest_sha256 = sha256_bytes(&manifest_bytes);

        let builder_run_value = json!({
            "schema": "termrock-spec/parity-subject-build-evidence-v1",
            "state": "complete",
            "qualification": {"status": "blocked", "reason": "fixture remains unqualified"},
            "run_id": "evidence-chain-test",
            "app": "holla",
            "subject_schema_sha256": sha256_file(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/subject-manifest-v2.schema.json")
            ).expect("hash v2 subject schema"),
            "reference_tag_object": ORACLE_TAG_OBJECT,
            "reference_tag_commit": ORACLE_TAG_COMMIT,
            "reference_branch_ref": REFERENCE_BRANCH_REF,
            "reference_commit": manifest.subjects[0].source_commit,
            "candidate_commit": manifest.subjects[1].source_commit,
            "target_triple": "aarch64-apple-darwin",
            "toolchain": "1.98.1",
            "rustup_path": "/toolchain/bin/rustup",
            "rustup_sha256": "8".repeat(64),
            "rustup_home": "/toolchain/rustup-home",
            "toolchain_binaries": toolchain_binaries,
            "git_identity": {
                "reference_tag_object": ORACLE_TAG_OBJECT,
                "reference_tag_commit": ORACLE_TAG_COMMIT,
                "reference_branch_ref": REFERENCE_BRANCH_REF,
                "reference_branch_oid": "a".repeat(40),
                "reference_commit": manifest.subjects[0].source_commit,
                "candidate_commit": manifest.subjects[1].source_commit
            },
            "reference_branch_oid": "a".repeat(40),
            "git_object_replacement_policy": replacement_policy,
            "cargo_version": "cargo 1.98.1",
            "rustc_version": "rustc 1.98.1",
            "host_triple": "aarch64-apple-darwin",
            "snapshots": snapshots,
            "snapshot_materialization": snapshot_materialization,
            "cargo_config_sha256": cargo_configs,
            "source_input_evidence": {
                "path": source_inputs_path.display().to_string(),
                "sha256": source_inputs_sha256
            },
            "build_environment": environments,
            "build_environment_evidence": build_environment_reference,
            "metadata_targets": target_metadata,
            "subject_manifest": manifest_path.display().to_string(),
            "subject_manifest_sha256": manifest_sha256
        });
        let builder_run_path = run_root.join("run.json");
        let builder_run_bytes =
            serde_json::to_vec(&builder_run_value).expect("serialize final builder run");
        fs::write(&builder_run_path, &builder_run_bytes).expect("write final builder run");
        let builder_run_path =
            fs::canonicalize(builder_run_path).expect("canonicalize final builder run");
        let builder_run_sha256 = sha256_bytes(&builder_run_bytes);

        BuildEvidenceChainFixture {
            run_root,
            manifest,
            manifest_path,
            manifest_sha256,
            source_inputs: serde_json::from_value(source_inputs_value).expect("type source inputs"),
            source_inputs_path,
            source_inputs_sha256,
            build_environment: serde_json::from_value(build_environment_value)
                .expect("type build environment"),
            build_environment_path,
            build_environment_sha256,
            builder_run: serde_json::from_value(builder_run_value).expect("type builder run"),
            builder_run_path,
            builder_run_sha256,
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
        resolve_test_write_policy_at_trust_path(
            fixture,
            expected_root,
            &fixture.trust_source.canonical_path,
        )
    }

    fn resolve_test_write_policy_at_trust_path(
        fixture: &TestWritePolicyFixture,
        expected_root: Option<&Path>,
        trust_record_path: &Path,
    ) -> Result<WritePolicyReceipt, String> {
        resolve_write_policy_with_evidence(
            fixture.input.clone(),
            &fixture.subjects,
            expected_root,
            trust_record_path,
            None,
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
            expected_generation: None,
            build_evidence: BuildEvidenceReceipt {
                subject_manifest: EvidenceReference {
                    path: "/tmp/subject-manifest.json".to_string(),
                    sha256: "a".repeat(64),
                },
                source_inputs: EvidenceReference {
                    path: "/tmp/source-inputs.json".to_string(),
                    sha256: "b".repeat(64),
                },
                build_environment: EvidenceReference {
                    path: "/tmp/build-environment.json".to_string(),
                    sha256: "c".repeat(64),
                },
                builder_run: EvidenceReference {
                    path: "/tmp/run.json".to_string(),
                    sha256: "d".repeat(64),
                },
            },
            environment: EnvironmentReceipt {
                cleared: true,
                cwd: policy.subject_cwd.clone(),
                common: BTreeMap::from([
                    ("TERM".to_string(), "xterm-256color".to_string()),
                    ("COLORTERM".to_string(), "truecolor".to_string()),
                    ("LC_ALL".to_string(), "C.UTF-8".to_string()),
                    ("SHELL".to_string(), "/bin/sh".to_string()),
                ]),
                case_allowlist: vec!["HOLLA_NO_HISTORY".to_string()],
                case_values: BTreeMap::from([("HOLLA_NO_HISTORY".to_string(), "1".to_string())]),
            },
            renderer: renderer_identity(&tuiscotti::profile::RenderProfile::vendored()),
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

    #[test]
    fn visual_receipt_requires_all_paired_checkpoints_and_blocks_without_expected_content() {
        let fixture = test_write_policy_fixture("visual-receipt-contract");
        let policy = resolve_test_write_policy(&fixture, None).expect("resolve test write policy");
        let mut receipt = test_run_receipt(&policy);
        receipt.source_pair = fixture
            .subjects
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
            .collect();
        let case = registry()
            .expect("load registry")
            .cases
            .into_iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("Holla pilot case");
        for role in ["reference", "candidate"] {
            for checkpoint in checkpoints(&case) {
                receipt.checks.push(check(
                    role,
                    &case.id,
                    checkpoint,
                    "visual",
                    "BLOCKED",
                    "no admitted expected generation".to_string(),
                    Vec::new(),
                ));
            }
        }

        validate_visual_receipt_contract(&receipt, &case, None, None)
            .expect("unadmitted pilot remains explicitly blocked");

        let expected_fixture = visual::synthetic_expected_generation_fixture();
        let generation = load_synthetic_expected_generation(&expected_fixture, &case);
        let mut hash_match_without_admission = receipt.clone();
        hash_match_without_admission.expected_generation = Some(ExpectedGenerationReceipt {
            id: generation.id.clone(),
            expected_sha256: generation.tree_sha256.clone(),
            actual_sha256: Some(generation.tree_sha256.clone()),
            state: "HASH_MATCH".to_string(),
            admission_receipt_sha256: None,
            root: expected_fixture.input.root.clone(),
        });
        validate_visual_receipt_contract(
            &hash_match_without_admission,
            &case,
            Some(&generation),
            None,
        )
        .expect("matching synthetic tree remains blocked without independent admission");
        let mut unadmitted_false_pass = hash_match_without_admission.clone();
        unadmitted_false_pass.checks[0].status = "PASS".to_string();
        assert!(
            validate_visual_receipt_contract(
                &unadmitted_false_pass,
                &case,
                Some(&generation),
                None,
            )
            .is_err()
        );

        let mut missing_checkpoint = receipt.clone();
        missing_checkpoint.checks.pop();
        assert!(validate_visual_receipt_contract(&missing_checkpoint, &case, None, None).is_err());

        let mut false_pass = receipt.clone();
        false_pass.checks[0].status = "PASS".to_string();
        assert!(validate_visual_receipt_contract(&false_pass, &case, None, None).is_err());

        fs::remove_dir_all(&fixture.base).expect("remove visual receipt fixture");
    }

    #[test]
    fn visual_comparison_runtime_join_requires_matching_external_admission_and_artifacts() {
        // Synthetic digests exercise the local join only; they do not create trust or admission.
        let expected_fixture = visual::synthetic_expected_generation_fixture();
        let case = registry()
            .expect("load case registry")
            .cases
            .into_iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("Holla pilot case");
        let generation = load_synthetic_expected_generation(&expected_fixture, &case);
        let frame_bytes = fs::read(expected_fixture.root.join("checkpoints/00-boot/frame.json"))
            .expect("read synthetic expected frame");
        let frame = visual::parse_canonical_frame(&frame_bytes, &case.geometry)
            .expect("parse synthetic expected frame");
        let capture = visual::synthetic_capture_fixture(&frame, "reference", "00-boot");
        let renderer = renderer_identity(&tuiscotti::profile::RenderProfile::vendored());
        let actual = visual::validate_actual_observation(
            &capture.captured,
            &capture.frame_artifact,
            &capture.png_artifact,
            &case.geometry,
            &renderer,
        )
        .expect("validate synthetic actual capture");

        let trust_record_sha256 = "a".repeat(64);
        let admission_receipt_sha256 = "b".repeat(64);
        let comparison = visual::compare_admitted_observations(
            &generation,
            "00-boot",
            actual,
            &renderer,
            &trust_record_sha256,
            &admission_receipt_sha256,
        )
        .expect("compare synthetic observations");

        let write_fixture = test_write_policy_fixture("visual-runtime-join");
        let policy =
            resolve_test_write_policy(&write_fixture, expected_fixture.input.root.as_deref())
                .expect("resolve test write policy");
        let mut receipt = test_run_receipt(&policy);
        receipt.trust.record_sha256 = trust_record_sha256;
        receipt.expected_generation = Some(ExpectedGenerationReceipt {
            id: generation.id.clone(),
            expected_sha256: generation.tree_sha256.clone(),
            actual_sha256: Some(generation.tree_sha256.clone()),
            state: "ADMITTED".to_string(),
            admission_receipt_sha256: Some(admission_receipt_sha256.clone()),
            root: expected_fixture.input.root.clone(),
        });
        receipt.artifacts = vec![
            capture.frame_artifact.clone(),
            capture.png_artifact.clone(),
            capture.fidelity_artifact.clone(),
        ];
        let admission = TrustedExpectedGeneration {
            id: generation.id.clone(),
            sha256: generation.tree_sha256.clone(),
            admission_receipt_sha256: admission_receipt_sha256.clone(),
        };

        validate_visual_comparison_evidence(
            &receipt,
            &case,
            "reference",
            "00-boot",
            "PASS",
            &comparison,
            &generation,
            &admission,
        )
        .expect("accept exact synthetic evidence bound to the external admission");

        let mut wrong_admission = comparison.clone();
        wrong_admission.admission.admission_receipt_sha256 = "c".repeat(64);
        assert!(
            validate_visual_comparison_evidence(
                &receipt,
                &case,
                "reference",
                "00-boot",
                "PASS",
                &wrong_admission,
                &generation,
                &admission,
            )
            .unwrap_err()
            .contains("visual admission is not bound")
        );

        let mut wrong_artifact = comparison.clone();
        wrong_artifact.frame.actual.sha256 = "d".repeat(64);
        assert!(
            validate_visual_comparison_evidence(
                &receipt,
                &case,
                "reference",
                "00-boot",
                "PASS",
                &wrong_artifact,
                &generation,
                &admission,
            )
            .unwrap_err()
            .contains("artifact references do not match")
        );

        let mut wrong_fidelity = comparison;
        wrong_fidelity.fidelity.actual.approximate = true;
        assert!(
            validate_visual_comparison_evidence(
                &receipt,
                &case,
                "reference",
                "00-boot",
                "PASS",
                &wrong_fidelity,
                &generation,
                &admission,
            )
            .unwrap_err()
            .contains("fidelity is not bound")
        );

        fs::remove_dir_all(&write_fixture.base).expect("remove runtime-join fixture");
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let suffix = CAPTURE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        fs::canonicalize(std::env::temp_dir())
            .expect("canonicalize temporary directory")
            .join(format!(
                "termrock-e2e-{label}-{}-{suffix}",
                std::process::id()
            ))
    }

    #[cfg(unix)]
    #[derive(Debug, PartialEq, Eq)]
    struct PathTypeHashEntry {
        kind: &'static str,
        bytes: Option<u64>,
        sha256: Option<String>,
    }

    #[cfg(unix)]
    fn protected_tree_inventory(root: &Path) -> BTreeMap<PathBuf, PathTypeHashEntry> {
        use std::os::unix::ffi::OsStrExt;

        fn visit(root: &Path, current: &Path, entries: &mut BTreeMap<PathBuf, PathTypeHashEntry>) {
            for item in fs::read_dir(current).expect("read protected tree directory") {
                let path = item.expect("read protected tree entry").path();
                let relative = path
                    .strip_prefix(root)
                    .expect("protected tree entry stays beneath root")
                    .to_path_buf();
                let metadata = fs::symlink_metadata(&path).expect("inspect protected tree entry");
                let entry = if metadata.file_type().is_symlink() {
                    let target = fs::read_link(&path).expect("read symlink target");
                    let target_bytes = target.as_os_str().as_bytes();
                    PathTypeHashEntry {
                        kind: "symlink",
                        bytes: Some(target_bytes.len() as u64),
                        sha256: Some(sha256_bytes(target_bytes)),
                    }
                } else if metadata.is_dir() {
                    PathTypeHashEntry {
                        kind: "directory",
                        bytes: None,
                        sha256: None,
                    }
                } else if metadata.is_file() {
                    let contents = fs::read(&path).expect("read protected tree file");
                    PathTypeHashEntry {
                        kind: "file",
                        bytes: Some(contents.len() as u64),
                        sha256: Some(sha256_bytes(&contents)),
                    }
                } else {
                    PathTypeHashEntry {
                        kind: "other",
                        bytes: None,
                        sha256: None,
                    }
                };
                let is_directory = entry.kind == "directory";
                assert!(entries.insert(relative, entry).is_none());
                if is_directory {
                    visit(root, &path, entries);
                }
            }
        }

        let metadata = fs::symlink_metadata(root).expect("inspect protected tree root");
        assert!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "protected tree root must be a real directory: {}",
            root.display()
        );
        let mut entries = BTreeMap::from([(
            PathBuf::from("."),
            PathTypeHashEntry {
                kind: "directory",
                bytes: None,
                sha256: None,
            },
        )]);
        visit(root, root, &mut entries);
        entries
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
