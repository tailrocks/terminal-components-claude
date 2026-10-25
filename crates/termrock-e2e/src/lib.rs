use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

mod admission;
mod visual;
pub use visual::VisualComparisonEvidence;

pub const REGISTRY_JSON: &str = include_str!("../cases/registry.json");

pub const PROFILE_JSON: &str = include_str!("../profile.json");
pub const RECEIPT_SCHEMA: &str = "termrock-spec/parity-run-receipt-v3";
pub const SUBJECT_SCHEMA: &str = "termrock-spec/parity-subject-manifest-v2";
pub const BUILDER_RECEIPT_SCHEMA: &str = "termrock-spec/parity-subject-build-receipt-v1";

const ORACLE_TAG_REF: &str = "refs/tags/visual-baseline";
const ORACLE_TAG_OBJECT: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const ORACLE_TAG_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const REFERENCE_BRANCH_REF: &str = "refs/remotes/origin/visual-baseline";
const REFERENCE_BRANCH_MEMBERSHIP: &str = "verified-reachable";

const TAG_SOURCE_VALIDATOR_PATH_ENV: &str = "TERMROCK_VIS06_SUBJECTS_PY";
const TAG_SOURCE_REPOSITORY_ENV: &str = "TERMROCK_VIS06_GIT_REPOSITORY";
const TAG_VALIDATOR_TOOLCHAIN_PATH_ENV: &str = "TERMROCK_VIS06_VALIDATOR_TOOLCHAIN";
const TAG_VALIDATOR_TOOLCHAIN_SHA_ENV: &str = "TERMROCK_VIS06_VALIDATOR_TOOLCHAIN_SHA256";

const TAG_SOURCE_VALIDATOR_SHA256: &str =
    "d990b77fbec0d01f242dbd191698939cbf5790d8dcd203337166c26688d85853";
const TAG_VALIDATOR_TOOLCHAIN_SCHEMA: &str = "termrock-spec/visual-tag-validator-toolchain-v1";
const TAG_VALIDATOR_HELPER_TRANSPORT: &str = "checked-source-absolute-git-dispatch-v1";
const TAG_VALIDATED_IDENTITY_SCHEMA: &str = "termrock-spec/visual-tag-holla-validated-identity-v1";

pub const ORACLE_CAPTURE_RECEIPT_SCHEMA: &str = "termrock-spec/parity-oracle-capture-receipt-v1";
pub const ORACLE_CAPTURE_RECEIPT_SCHEMA_JSON: &str =
    include_str!("../schemas/oracle-capture-receipt-v1.schema.json");
const TAG_BUILD_EVIDENCE_SCHEMA: &str = "termrock-spec/visual-tag-holla-build-evidence-v1";
pub const WRITE_POLICY_SCHEMA: &str = "termrock-spec/parity-write-policy-v2";
pub const COMPILED_SUITE_SHA256: &str = env!("TERMROCK_E2E_COMPILED_SUITE_SHA256");

struct NoDuplicateJson;

impl<'de> Deserialize<'de> for NoDuplicateJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NoDuplicateJsonVisitor)
    }
}

struct NoDuplicateJsonVisitor;

impl<'de> Visitor<'de> for NoDuplicateJsonVisitor {
    type Value = NoDuplicateJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        NoDuplicateJson::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element::<NoDuplicateJson>()?.is_some() {}
        Ok(NoDuplicateJson)
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut seen = BTreeSet::new();
        while let Some(key) = object.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(A::Error::custom(format!(
                    "duplicate JSON object key {key:?}"
                )));
            }
            object.next_value::<NoDuplicateJson>()?;
        }
        Ok(NoDuplicateJson)
    }
}

fn reject_duplicate_json_keys(bytes: &[u8], label: &str) -> Result<(), String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    NoDuplicateJson::deserialize(&mut deserializer)
        .map_err(|error| format!("{label} contains invalid JSON: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("{label} has trailing JSON data: {error}"))
}

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
    KeyEvent {
        key: String,
        kind: KeyEventKind,
    },
    Text {
        text: String,
    },
    Paste {
        text: String,
    },
    Mouse {
        input: MouseInput,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
    ExpectExit {
        code: u32,
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyEventKind {
    Down,
    Repeat,
    Up,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WheelDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MouseModifiers {
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub ctrl: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum MouseInput {
    Click {
        button: MouseButton,
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
    },
    Down {
        button: MouseButton,
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
    },
    Release {
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
    },
    Move {
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
    },
    Drag {
        button: MouseButton,
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
    },
    Wheel {
        direction: WheelDirection,
        x: u16,
        y: u16,
        #[serde(default)]
        modifiers: MouseModifiers,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TagBuilderEvidence {
    schema: String,
    run_id: String,
    purpose: String,
    build_result: String,
    qualification: QualificationEvidence,
    oracle_lineage: TagOracleLineage,
    source_snapshot: TagSourceSnapshotEvidence,
    build_artifact_root: PathBuf,
    build: BuildFacts,
    executable: Executable,
    builder_receipt: BuilderReceipt,
    builder_receipt_sha256: String,
    build_environment: TagBuildEnvironmentEvidence,
    capture_status: String,
    admission_status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TagSourceSnapshotEvidence {
    materialized_root: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TagValidatorToolIdentity {
    pub path: PathBuf,
    pub sha256: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TagValidatorToolchain {
    schema: String,
    python: TagValidatorToolIdentity,
    git: TagValidatorToolIdentity,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TagValidatorExecutionEvidence {
    pub repository: PathBuf,
    pub git_dir: PathBuf,
    pub git_common_dir: PathBuf,
    pub toolchain: EvidenceReference,
    pub python: TagValidatorToolIdentity,
    pub git: TagValidatorToolIdentity,
    pub helper_transport: String,
    pub executed_helper_sha256: String,
    pub python_sha256_before: String,
    pub python_sha256_after: String,
    pub git_sha256_before: String,
    pub git_sha256_after: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TagBuildEnvironmentEvidence {
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TagMetadataTargetEvidence {
    package_id: String,
    package_name: String,
    target_name: String,
    manifest_path: String,
    source_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TagValidatedEvidenceReference {
    path: PathBuf,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TagValidatedSourceSnapshot {
    source_commit: String,
    tree_oid: String,
    materialized_root: PathBuf,
    included_path_blob_map_sha256: String,
    included_file_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagValidatedIdentity {
    schema: String,
    receipt: TagValidatedEvidenceReference,
    run: TagValidatedEvidenceReference,
    oracle_lineage: TagOracleLineage,
    source_snapshot: TagValidatedSourceSnapshot,
    metadata_target: TagMetadataTargetEvidence,
    build: BuildFacts,
    executable: Executable,
    builder_receipt_sha256: String,
    build_environment: TagBuildEnvironmentEvidence,
    execution_anchor_status: String,
    qualification: QualificationEvidence,
    capture_status: String,
    admission_status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TagOracleLineage {
    tag_ref: String,
    tag_object: String,
    tag_commit: String,
    git_object_replacement_policy: String,
    git_lazy_fetch_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OracleCaptureReceipt {
    pub schema: String,
    pub run_id: String,
    pub state: String,
    pub oracle_identity: OracleCaptureIdentity,
    pub builder_evidence: EvidenceReference,
    pub builder_run: EvidenceReference,
    pub validator_helper: EvidenceReference,
    pub suite: SuiteIdentity,
    pub case: CaseCaptureIdentity,
    pub environment: EnvironmentReceipt,
    pub renderer: RendererIdentity,
    pub write_policy: WritePolicyReceipt,
    pub checks: Vec<CheckReceipt>,
    pub artifacts: Vec<ArtifactReceipt>,
    pub capture_status: String,
    pub execution_anchor_status: String,
    pub qualification: QualificationEvidence,
    pub admission_status: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionWriteReceipt {
    pub schema: String,
    pub admission_path: PathBuf,
    pub admission_sha256: String,
    pub generation_id: String,
    pub generation_tree_sha256: String,
    pub decision: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OracleCaptureIdentity {
    pub tag_ref: String,
    pub tag_object: String,
    pub tag_commit: String,
    pub git_object_replacement_policy: String,
    pub git_lazy_fetch_policy: String,
    pub source_snapshot_commit: String,
    pub source_snapshot_tree: String,
    pub source_snapshot_root: PathBuf,
    pub source_snapshot_included_file_count: u64,
    pub build_artifact_root: PathBuf,
    pub build: BuildFacts,
    pub builder_receipt_sha256: String,
    pub source_inputs_sha256: String,
    pub build_environment_sha256: String,
    pub executable: ExecutableReceipt,
    pub validator_execution: TagValidatorExecutionEvidence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseCaptureIdentity {
    pub id: String,
    pub app: String,
    pub binary: String,
    pub args: Vec<String>,
    pub geometry: Geometry,
    pub color_path: String,
    pub legacy_snapshot_root: String,
    pub screen: String,
    pub substep: String,
    pub case_set_sha256: String,
    pub input_program_sha256: String,
    pub checkpoints: Vec<CheckpointCaptureIdentity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointCaptureIdentity {
    pub id: String,
    pub assertions: Vec<Assertion>,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WritePolicyReceipt {
    pub schema: String,
    pub write_root: PathBuf,
    pub receipt_path: PathBuf,
    pub subject_cwd: PathBuf,
    pub protected_roots: Vec<ProtectedRoot>,
    pub actual_output_roots: Vec<ActualOutputRoot>,
    pub sha256: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RustupEvidence {
    path: String,
    sha256: String,
    home: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ToolchainBinaryEvidence {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QualificationEvidence {
    pub status: String,
    pub reason: String,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutableReceipt {
    pub path: PathBuf,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckReceipt {
    pub id: String,
    pub subject_role: String,
    pub case_id: String,
    pub checkpoint_id: String,
    pub dimension: String,
    pub status: String,
    pub reason: String,
    pub evidence: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    pub visual_comparison: Option<VisualComparisonEvidence>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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
    validate_geometry(&case.geometry)?;
    let mut geometry = case.geometry.clone();
    let mut checkpoint_ids = BTreeSet::new();
    let mut checkpoint_count = 0usize;
    if case
        .steps
        .iter()
        .filter(|step| matches!(step, Step::ExpectExit { .. }))
        .count()
        > 1
    {
        return Err(format!(
            "case {} has more than one expect_exit step",
            case.id
        ));
    }
    for (index, step) in case.steps.iter().enumerate() {
        match step {
            Step::Press { key } | Step::KeyEvent { key, .. } => {
                tuiscotti::tui::parse_chord(key).map_err(|error| {
                    format!("case {} has invalid key chord {key:?}: {error}", case.id)
                })?;
            }
            Step::Text { .. } => {}
            Step::Paste { text } => {
                if text.contains("\u{1b}[200~") || text.contains("\u{1b}[201~") {
                    return Err(format!(
                        "case {} paste contains a bracketed-paste delimiter",
                        case.id
                    ));
                }
            }
            Step::Mouse { input } => validate_mouse_input(input, &geometry, &case.id)?,
            Step::Resize { cols, rows } => {
                geometry = Geometry {
                    cols: *cols,
                    rows: *rows,
                };
                validate_geometry(&geometry).map_err(|error| {
                    format!("case {} has invalid resize target: {error}", case.id)
                })?;
            }
            Step::ExpectExit { .. } => {
                if checkpoint_count == 0 {
                    return Err(format!(
                        "case {} expect_exit must follow at least one checkpoint",
                        case.id
                    ));
                }
                if index + 1 != case.steps.len() {
                    return Err(format!("case {} has steps after expect_exit", case.id));
                }
            }
            Step::Checkpoint { id, .. } => {
                if !valid_checkpoint_id(id) {
                    return Err(format!("case {} has invalid checkpoint ID {id:?}", case.id));
                }
                if !checkpoint_ids.insert(id.as_str()) {
                    return Err(format!(
                        "case {} has duplicate checkpoint ID {id:?}",
                        case.id
                    ));
                }
                checkpoint_count += 1;
            }
        }
    }
    if checkpoint_count == 0 {
        return Err(format!("case {} has no checkpoints", case.id));
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

pub(crate) fn validate_geometry(geometry: &Geometry) -> Result<(), String> {
    use tuiscotti::tui::{MAX_COLS, MAX_ROWS, MIN_COLS, MIN_ROWS};

    if !(MIN_COLS..=MAX_COLS).contains(&geometry.cols) {
        return Err(format!(
            "cols {} outside Tuiscotti range {MIN_COLS}..={MAX_COLS}",
            geometry.cols
        ));
    }
    if !(MIN_ROWS..=MAX_ROWS).contains(&geometry.rows) {
        return Err(format!(
            "rows {} outside Tuiscotti range {MIN_ROWS}..={MAX_ROWS}",
            geometry.rows
        ));
    }
    Ok(())
}

pub(crate) fn valid_checkpoint_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn validate_mouse_input(
    input: &MouseInput,
    geometry: &Geometry,
    case_id: &str,
) -> Result<(), String> {
    let (x, y) = match input {
        MouseInput::Click { x, y, .. }
        | MouseInput::Down { x, y, .. }
        | MouseInput::Release { x, y, .. }
        | MouseInput::Move { x, y, .. }
        | MouseInput::Drag { x, y, .. }
        | MouseInput::Wheel { x, y, .. } => (*x, *y),
    };
    if x >= geometry.cols || y >= geometry.rows {
        return Err(format!(
            "case {case_id} mouse coordinate ({x},{y}) is outside current geometry {}x{}",
            geometry.cols, geometry.rows
        ));
    }
    Ok(())
}

pub(crate) fn checkpoint_geometries(case: &Case) -> Result<BTreeMap<String, Geometry>, String> {
    let mut geometry = case.geometry.clone();
    let mut checkpoints = BTreeMap::new();
    for step in &case.steps {
        match step {
            Step::Resize { cols, rows } => {
                geometry = Geometry {
                    cols: *cols,
                    rows: *rows,
                }
            }
            Step::Checkpoint { id, .. } => {
                if checkpoints.insert(id.clone(), geometry.clone()).is_some() {
                    return Err(format!(
                        "case {} has duplicate checkpoint ID {id:?}",
                        case.id
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(checkpoints)
}

pub(crate) fn checkpoint_geometry(case: &Case, checkpoint: &str) -> Result<Geometry, String> {
    checkpoint_geometries(case)?
        .remove(checkpoint)
        .ok_or_else(|| format!("case {} has no checkpoint {checkpoint:?}", case.id))
}

fn load_caller_pinned_tag_validator_toolchain()
-> Result<(TagValidatorToolchain, EvidenceReference), String> {
    let toolchain_path = std::env::var_os(TAG_VALIDATOR_TOOLCHAIN_PATH_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{TAG_VALIDATOR_TOOLCHAIN_PATH_ENV} is required"))?;
    let expected_toolchain_sha256 = std::env::var(TAG_VALIDATOR_TOOLCHAIN_SHA_ENV)
        .map_err(|_| format!("{TAG_VALIDATOR_TOOLCHAIN_SHA_ENV} must be caller-pinned"))?;
    require_sha256(TAG_VALIDATOR_TOOLCHAIN_SHA_ENV, &expected_toolchain_sha256)?;
    load_tag_validator_toolchain(&toolchain_path, &expected_toolchain_sha256)
}

fn require_caller_pinned_toolchain_reference(
    captured: &EvidenceReference,
    caller_pinned: &EvidenceReference,
) -> Result<(), String> {
    if captured != caller_pinned {
        return Err("captured validator toolchain differs from the current caller pin".to_string());
    }
    Ok(())
}

fn load_tag_validator_toolchain(
    supplied_path: &Path,
    expected_sha256: &str,
) -> Result<(TagValidatorToolchain, EvidenceReference), String> {
    if !supplied_path.is_absolute() {
        return Err(format!(
            "{TAG_VALIDATOR_TOOLCHAIN_PATH_ENV} must be absolute"
        ));
    }
    require_sha256(TAG_VALIDATOR_TOOLCHAIN_SHA_ENV, expected_sha256)?;
    let path = canonical_existing_regular_file_without_symlinks(
        supplied_path,
        "pinned immutable-tag validator toolchain",
    )?;
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "read immutable-tag validator toolchain {}: {error}",
            path.display()
        )
    })?;
    let sha256 = sha256_bytes(&bytes);
    if sha256 != expected_sha256 {
        return Err(format!(
            "immutable-tag validator toolchain digest mismatch: expected {expected_sha256}, got {sha256}"
        ));
    }
    reject_duplicate_json_keys(&bytes, "immutable-tag validator toolchain")?;
    let toolchain: TagValidatorToolchain = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse immutable-tag validator toolchain: {error}"))?;
    if toolchain.schema != TAG_VALIDATOR_TOOLCHAIN_SCHEMA {
        return Err("unexpected immutable-tag validator toolchain schema".to_string());
    }
    validate_tag_validator_tool("Python runtime", &toolchain.python, false)?;
    validate_tag_validator_tool("Git executable", &toolchain.git, true)?;
    Ok((
        toolchain,
        EvidenceReference {
            path: path.display().to_string(),
            sha256,
        },
    ))
}

fn validate_tag_validator_tool(
    label: &str,
    tool: &TagValidatorToolIdentity,
    require_git_basename: bool,
) -> Result<(), String> {
    if !tool.path.is_absolute() || tool.version.trim().is_empty() {
        return Err(format!(
            "pinned {label} path and version must be absolute and nonempty"
        ));
    }
    require_sha256(&format!("pinned {label} SHA-256"), &tool.sha256)?;
    let canonical = canonical_existing_regular_file_without_symlinks(&tool.path, label)?;
    if canonical != tool.path {
        return Err(format!("pinned {label} path is not canonical"));
    }
    if require_git_basename && tool.path.file_name().and_then(|name| name.to_str()) != Some("git") {
        return Err("pinned Git executable must have the basename git".to_string());
    }
    let actual_sha256 = sha256_file(&canonical)?;
    if actual_sha256 != tool.sha256 {
        return Err(format!("pinned {label} bytes changed"));
    }
    let output = Command::new(&canonical)
        .arg("--version")
        .env_clear()
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| format!("run pinned {label} version query: {error}"))?;
    if !output.status.success() || output.stdout.len() > 4096 || output.stderr.len() > 4096 {
        return Err(format!(
            "pinned {label} version query failed or exceeded its output limit"
        ));
    }
    let version_bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let version = std::str::from_utf8(version_bytes)
        .map_err(|error| format!("pinned {label} version is not UTF-8: {error}"))?
        .trim()
        .to_string();
    if version != tool.version {
        return Err(format!(
            "pinned {label} version mismatch: expected {:?}, got {:?}",
            tool.version, version
        ));
    }
    Ok(())
}

fn execute_tag_source_validator(
    receipt_path: &Path,
    expected_receipt_sha256: &str,
    supplied_helper_path: &Path,
    supplied_repository: &Path,
    toolchain: &TagValidatorToolchain,
    toolchain_reference: EvidenceReference,
) -> Result<
    (
        TagValidatedIdentity,
        TagValidatedEvidenceReference,
        TagValidatorExecutionEvidence,
    ),
    String,
> {
    let helper_path = canonical_existing_regular_file_without_symlinks(
        supplied_helper_path,
        "pinned immutable-tag source validator",
    )?;

    let helper_bytes = fs::read(&helper_path)
        .map_err(|error| format!("read pinned immutable-tag source validator: {error}"))?;
    let helper_sha256 = sha256_bytes(&helper_bytes);
    if helper_sha256 != TAG_SOURCE_VALIDATOR_SHA256 {
        return Err(format!(
            "immutable-tag source validator digest mismatch: expected {TAG_SOURCE_VALIDATOR_SHA256}, got {helper_sha256}"
        ));
    }
    validate_tag_validator_tool("Python runtime", &toolchain.python, false)?;
    validate_tag_validator_tool("Git executable", &toolchain.git, true)?;
    let executed_helper_bytes =
        transform_tag_validator_git_dispatch(&helper_bytes, &toolchain.git.path)?;
    let executed_helper_sha256 = sha256_bytes(&executed_helper_bytes);
    let repository = canonical_existing_directory_without_symlinks(
        supplied_repository,
        "immutable-tag Git object repository",
    )?;
    let git_dir_before = pinned_git_metadata_directory(
        &toolchain.git.path,
        &repository,
        &["rev-parse", "--absolute-git-dir"],
        "immutable-tag Git directory",
    )?;
    let git_common_dir_before = pinned_git_metadata_directory(
        &toolchain.git.path,
        &repository,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        "immutable-tag Git common directory",
    )?;
    let receipt_path = canonical_existing_regular_file_without_symlinks(
        receipt_path,
        "immutable-tag builder receipt",
    )?;
    let output = run_pinned_validator_python(
        &toolchain.python.path,
        &executed_helper_bytes,
        &repository,
        &receipt_path,
        expected_receipt_sha256,
    )?;
    validate_tag_validator_tool("Python runtime", &toolchain.python, false)?;
    validate_tag_validator_tool("Git executable", &toolchain.git, true)?;
    let git_dir_after = pinned_git_metadata_directory(
        &toolchain.git.path,
        &repository,
        &["rev-parse", "--absolute-git-dir"],
        "immutable-tag Git directory",
    )?;
    let git_common_dir_after = pinned_git_metadata_directory(
        &toolchain.git.path,
        &repository,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        "immutable-tag Git common directory",
    )?;
    if git_dir_before != git_dir_after || git_common_dir_before != git_common_dir_after {
        return Err("immutable-tag Git metadata paths changed during validation".to_string());
    }
    if output.stdout.len() > 2 * 1024 * 1024 || output.stderr.len() > 64 * 1024 {
        return Err("immutable-tag source validator output exceeded its size limit".to_string());
    }
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "immutable-tag source validation failed with {}: {}",
            output.status,
            detail.trim()
        ));
    }
    let identity =
        parse_tag_validator_identity(&output.stdout, &receipt_path, expected_receipt_sha256)?;
    Ok((
        identity,
        TagValidatedEvidenceReference {
            path: helper_path,
            sha256: helper_sha256,
        },
        TagValidatorExecutionEvidence {
            repository,
            git_dir: git_dir_before,
            git_common_dir: git_common_dir_before,
            toolchain: toolchain_reference,
            python: toolchain.python.clone(),
            git: toolchain.git.clone(),
            helper_transport: TAG_VALIDATOR_HELPER_TRANSPORT.to_string(),
            executed_helper_sha256,
            python_sha256_before: toolchain.python.sha256.clone(),
            python_sha256_after: toolchain.python.sha256.clone(),
            git_sha256_before: toolchain.git.sha256.clone(),
            git_sha256_after: toolchain.git.sha256.clone(),
        },
    ))
}

fn pinned_git_metadata_directory(
    git_executable: &Path,
    repository: &Path,
    rev_parse_arguments: &[&str],
    label: &str,
) -> Result<PathBuf, String> {
    if !git_executable.is_absolute() || !repository.is_absolute() {
        return Err(format!("{label} query requires absolute paths"));
    }
    let output = Command::new(git_executable)
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(repository)
        .args(rev_parse_arguments)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| format!("run pinned {label} query: {error}"))?;
    if !output.status.success() || output.stdout.len() > 4096 || output.stderr.len() > 4096 {
        return Err(format!(
            "pinned {label} query failed or exceeded its output limit"
        ));
    }
    let mut reported = output.stdout.as_slice();
    if reported.last() == Some(&b'\n') {
        reported = &reported[..reported.len() - 1];
    }
    if reported.last() == Some(&b'\r') {
        reported = &reported[..reported.len() - 1];
    }
    if reported.is_empty() || reported.contains(&b'\n') || reported.contains(&b'\r') {
        return Err(format!("pinned {label} query did not return one path"));
    }
    let reported = std::str::from_utf8(reported)
        .map_err(|error| format!("pinned {label} path is not UTF-8: {error}"))?;
    let reported_path = Path::new(reported);
    if !reported_path.is_absolute() {
        return Err(format!("pinned {label} path is not absolute"));
    }
    let canonical = canonical_existing_directory_without_symlinks(reported_path, label)?;
    if canonical.as_path() != reported_path {
        return Err(format!("pinned {label} path is not canonical"));
    }
    Ok(canonical)
}

fn transform_tag_validator_git_dispatch(
    helper_bytes: &[u8],
    git_path: &Path,
) -> Result<Vec<u8>, String> {
    if !git_path.is_absolute() {
        return Err("pinned Git executable path must be absolute".to_string());
    }
    let needle = br#"["git", "--no-replace-objects", *arguments]"#;
    let positions = helper_bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect::<Vec<_>>();
    if positions.len() != 1 {
        return Err(format!(
            "pinned validator must contain exactly one expected Git dispatch site; found {}",
            positions.len()
        ));
    }
    let git_text = git_path
        .to_str()
        .ok_or_else(|| "pinned Git executable path is not UTF-8".to_string())?;
    let python_literal = serde_json::to_string(git_text)
        .map_err(|error| format!("encode pinned Git executable path: {error}"))?;
    let replacement = format!("[{python_literal}, \"--no-replace-objects\", *arguments]");
    let position = positions[0];
    let mut transformed = Vec::with_capacity(helper_bytes.len() - needle.len() + replacement.len());
    transformed.extend_from_slice(&helper_bytes[..position]);
    transformed.extend_from_slice(replacement.as_bytes());
    transformed.extend_from_slice(&helper_bytes[position + needle.len()..]);
    Ok(transformed)
}

fn run_pinned_validator_python(
    python: &Path,
    helper_bytes: &[u8],
    repository: &Path,
    receipt_path: &Path,
    expected_receipt_sha256: &str,
) -> Result<std::process::Output, String> {
    let mut child = Command::new(python)
        .arg("-I")
        .arg("-")
        .arg("validate-frozen-tag-holla")
        .arg("--repository")
        .arg(repository)
        .arg("--receipt")
        .arg(receipt_path)
        .arg("--receipt-sha256")
        .arg(expected_receipt_sha256)
        .env_clear()
        .env("LC_ALL", "C")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("start pinned immutable-tag source validator: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "pinned validator stdin was not opened".to_string())?;
    if let Err(error) = stdin.write_all(helper_bytes) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "write checked validator bytes to Python stdin: {error}"
        ));
    }
    drop(stdin);
    child
        .wait_with_output()
        .map_err(|error| format!("wait for pinned immutable-tag source validator: {error}"))
}

fn parse_tag_validator_identity(
    bytes: &[u8],
    receipt_path: &Path,
    expected_receipt_sha256: &str,
) -> Result<TagValidatedIdentity, String> {
    reject_duplicate_json_keys(bytes, "immutable-tag validated identity")?;
    let identity: TagValidatedIdentity = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse immutable-tag validated identity: {error}"))?;
    validate_tag_validator_identity(&identity, receipt_path, expected_receipt_sha256)?;
    Ok(identity)
}

fn validate_tag_validator_identity(
    identity: &TagValidatedIdentity,
    receipt_path: &Path,
    expected_receipt_sha256: &str,
) -> Result<(), String> {
    require_sha256("pinned tag receipt SHA-256", expected_receipt_sha256)?;
    if identity.schema != TAG_VALIDATED_IDENTITY_SCHEMA
        || identity.execution_anchor_status != "unverified"
        || identity.qualification.status != "blocked"
        || identity.qualification.reason.trim().is_empty()
        || identity.capture_status != "NOT_RUN"
        || identity.admission_status != "NOT_RUN"
    {
        return Err(
            "immutable-tag validator identity must remain unverified and blocked".to_string(),
        );
    }
    if identity.receipt.path != receipt_path
        || identity.receipt.sha256 != expected_receipt_sha256
        || receipt_path.file_name().and_then(|name| name.to_str())
            != Some("tag-builder-receipt.json")
    {
        return Err(
            "immutable-tag validator identity does not bind the requested receipt".to_string(),
        );
    }
    if !receipt_path.is_absolute() {
        return Err("immutable-tag receipt path must be absolute".to_string());
    }
    let run_root = receipt_path
        .parent()
        .ok_or_else(|| "immutable-tag receipt has no run directory".to_string())?;
    if identity.run.path != run_root.join("run.json") {
        return Err(
            "immutable-tag validator identity does not bind the sibling run.json".to_string(),
        );
    }
    require_sha256("validated tag run record", &identity.run.sha256)?;
    for (name, actual, expected) in [
        (
            "tag ref",
            identity.oracle_lineage.tag_ref.as_str(),
            ORACLE_TAG_REF,
        ),
        (
            "tag object",
            identity.oracle_lineage.tag_object.as_str(),
            ORACLE_TAG_OBJECT,
        ),
        (
            "tag commit",
            identity.oracle_lineage.tag_commit.as_str(),
            ORACLE_TAG_COMMIT,
        ),
        (
            "replacement-object policy",
            identity
                .oracle_lineage
                .git_object_replacement_policy
                .as_str(),
            "disabled-by-option-and-environment",
        ),
        (
            "lazy-fetch policy",
            identity.oracle_lineage.git_lazy_fetch_policy.as_str(),
            "disabled-by-environment",
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "immutable-tag validated {name} differs from the fixed oracle"
            ));
        }
    }
    require_git_oid(
        "validated immutable-tag object",
        &identity.oracle_lineage.tag_object,
    )?;
    require_git_oid(
        "validated immutable-tag commit",
        &identity.oracle_lineage.tag_commit,
    )?;
    require_git_oid(
        "validated immutable-tag tree",
        &identity.source_snapshot.tree_oid,
    )?;
    if identity.source_snapshot.source_commit != ORACLE_TAG_COMMIT
        || identity.source_snapshot.materialized_root != run_root.join("source/oracle")
        || identity.source_snapshot.included_file_count == 0
    {
        return Err(
            "immutable-tag validator source closure is not rooted at the fixed tag".to_string(),
        );
    }
    require_sha256(
        "validated included source path/blob map",
        &identity.source_snapshot.included_path_blob_map_sha256,
    )?;
    if identity.build.target_name != "holla"
        || identity.build.features.len() != 0
        || !identity.build.default_features
        || identity.build.profile != "release"
        || identity.build.toolchain != "1.98.1"
        || identity.build.package_id.trim().is_empty()
        || identity.build.target_triple.trim().is_empty()
        || identity.metadata_target.package_id != identity.build.package_id
        || identity.metadata_target.target_name != "holla"
        || identity.metadata_target.package_name.trim().is_empty()
    {
        return Err(
            "immutable-tag validator identity is not the pinned Holla release target".to_string(),
        );
    }
    require_sha256(
        "validated immutable-tag executable",
        &identity.executable.sha256,
    )?;
    if !identity.executable.path.is_absolute() {
        return Err("validated immutable-tag executable path must be absolute".to_string());
    }
    require_sha256(
        "validated nested builder receipt",
        &identity.builder_receipt_sha256,
    )?;
    if identity.build_environment.target_triple != identity.build.target_triple
        || identity.build_environment.host_triple != identity.build.target_triple
        || identity.build_environment.external_cargo_config_policy
            != "reject-ancestor-config-and-hash-source-local-config"
        || identity.build_environment.environment.len() != 1
        || !identity
            .build_environment
            .environment
            .contains_key("oracle")
        || identity.build_environment.cargo_config_sha256.len() != 1
        || !identity
            .build_environment
            .cargo_config_sha256
            .contains_key("oracle")
    {
        return Err(
            "immutable-tag validator build environment does not match its Holla target".to_string(),
        );
    }
    let source_root = &identity.source_snapshot.materialized_root;
    if !Path::new(&identity.metadata_target.manifest_path).starts_with(source_root)
        || !Path::new(&identity.metadata_target.source_path).starts_with(source_root)
        || Path::new(&identity.metadata_target.manifest_path) != source_root.join("Cargo.toml")
    {
        return Err("immutable-tag metadata target escapes its validated source root".to_string());
    }
    Ok(())
}

fn validate_tag_record_binding(
    record: &TagBuilderEvidence,
    identity: &TagValidatedIdentity,
    receipt_path: &Path,
    expected_receipt_sha256: &str,
) -> Result<(), String> {
    validate_tag_validator_identity(identity, receipt_path, expected_receipt_sha256)?;
    let run_root = receipt_path
        .parent()
        .ok_or_else(|| "immutable-tag receipt has no run directory".to_string())?;
    if record.schema != TAG_BUILD_EVIDENCE_SCHEMA
        || record.purpose != "frozen-visual-tag-holla-build-only"
        || record.build_result != "PASS"
        || record.qualification.status != "blocked"
        || record.qualification.reason.trim().is_empty()
        || record.capture_status != "NOT_RUN"
        || record.admission_status != "NOT_RUN"
        || record.run_id.trim().is_empty()
        || record.oracle_lineage != identity.oracle_lineage
        || record.source_snapshot.source_commit != identity.source_snapshot.source_commit
        || record.source_snapshot.tree_oid != identity.source_snapshot.tree_oid
        || Path::new(&record.source_snapshot.materialized_root)
            != identity.source_snapshot.materialized_root
        || record.source_snapshot.included_path_blob_map_sha256
            != identity.source_snapshot.included_path_blob_map_sha256
        || record.source_snapshot.included_file_count
            != identity.source_snapshot.included_file_count
        || record.build_artifact_root != run_root.join("targets/oracle")
        || record.build != identity.build
        || record.executable != identity.executable
        || record.builder_receipt_sha256 != identity.builder_receipt_sha256
        || record.build_environment != identity.build_environment
        || record.builder_receipt.source_commit != ORACLE_TAG_COMMIT
        || record.builder_receipt.package_id != identity.build.package_id
        || record.builder_receipt.target_name != "holla"
        || record.builder_receipt.executable_path != identity.executable.path
        || record.builder_receipt.executable_sha256 != identity.executable.sha256
    {
        return Err(
            "immutable-tag receipt fields do not match the pinned validator identity".to_string(),
        );
    }
    let nested_sha256 = builder_receipt_digest(&record.builder_receipt)?;
    if record.builder_receipt.sha256 != nested_sha256
        || record.builder_receipt_sha256 != nested_sha256
    {
        return Err("immutable-tag nested Cargo builder receipt digest is invalid".to_string());
    }
    Ok(())
}

fn require_tag_validator_outside_write_roots(
    validator_path: &Path,
    execution: &TagValidatorExecutionEvidence,
    write_root: &Path,
    actual_output_roots: &[ActualOutputRoot],
) -> Result<(), String> {
    let write = resolve_path_for_overlap(write_root, "tag write root")?;
    for path in tag_validator_protected_paths(validator_path, execution)? {
        let protected = resolve_path_for_overlap(&path, "tag validator protected input")?;
        if physical_paths_overlap(&protected, &write) {
            return Err(format!(
                "tag validator protected input {} overlaps the writable receipt root",
                path.display()
            ));
        }
        for actual in actual_output_roots {
            let output = resolve_path_for_overlap(&actual.path, "tag actual output root")?;
            if physical_paths_overlap(&protected, &output) {
                return Err(format!(
                    "tag validator protected input {} overlaps actual output root for {}",
                    path.display(),
                    actual.role
                ));
            }
        }
    }
    Ok(())
}

fn tag_validator_protected_paths(
    validator_path: &Path,
    execution: &TagValidatorExecutionEvidence,
) -> Result<Vec<PathBuf>, String> {
    let mut protected = Vec::new();
    let repository = canonical_existing_directory_without_symlinks(
        &execution.repository,
        "pinned immutable-tag Git repository",
    )?;
    if repository != execution.repository {
        return Err("pinned immutable-tag Git repository path is not canonical".to_string());
    }

    let git_dir = canonical_existing_directory_without_symlinks(
        &execution.git_dir,
        "captured immutable-tag Git directory",
    )?;
    if git_dir != execution.git_dir {
        return Err("captured immutable-tag Git directory is not canonical".to_string());
    }
    protected.push(git_dir);

    let git_common_dir = canonical_existing_directory_without_symlinks(
        &execution.git_common_dir,
        "captured immutable-tag Git common directory",
    )?;
    if git_common_dir != execution.git_common_dir {
        return Err("captured immutable-tag Git common directory is not canonical".to_string());
    }
    protected.push(git_common_dir);

    let git_entry = repository.join(".git");
    match fs::symlink_metadata(&git_entry) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("immutable-tag Git metadata entry must not be a symlink".to_string());
        }
        Ok(metadata) if metadata.is_dir() => {
            protected.push(canonical_existing_directory_without_symlinks(
                &git_entry,
                "immutable-tag Git metadata entry",
            )?)
        }
        Ok(metadata) if metadata.is_file() => {
            protected.push(canonical_existing_regular_file_without_symlinks(
                &git_entry,
                "immutable-tag Git metadata entry",
            )?)
        }
        Ok(_) => {
            return Err("immutable-tag Git metadata entry has an unsupported type".to_string());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "inspect immutable-tag Git metadata entry {}: {error}",
                git_entry.display()
            ));
        }
    }

    push_protected_validator_file(
        &mut protected,
        validator_path,
        "pinned tag source validator",
    )?;
    push_protected_validator_file(
        &mut protected,
        Path::new(&execution.toolchain.path),
        "pinned validator toolchain",
    )?;
    push_protected_validator_file(
        &mut protected,
        &execution.python.path,
        "pinned Python runtime",
    )?;
    push_protected_validator_file(&mut protected, &execution.git.path, "pinned Git executable")?;

    Ok(protected)
}

fn push_protected_validator_file(
    protected: &mut Vec<PathBuf>,
    path: &Path,
    label: &str,
) -> Result<(), String> {
    let canonical_file = canonical_existing_regular_file_without_symlinks(path, label)?;
    let parent = canonical_file
        .parent()
        .ok_or_else(|| format!("{label} has no parent directory"))?;
    let canonical_parent =
        canonical_existing_directory_without_symlinks(parent, &format!("{label} parent"))?;
    protected.push(canonical_file);
    protected.push(canonical_parent);
    Ok(())
}

fn tag_case_capture_identity(
    case: &Case,
    case_set_sha256: &str,
) -> Result<CaseCaptureIdentity, String> {
    let steps_bytes = serde_json::to_vec(&case.steps)
        .map_err(|error| format!("serialize Holla case input program: {error}"))?;
    let checkpoints = case
        .steps
        .iter()
        .filter_map(|step| match step {
            Step::Checkpoint { id, assertions, .. } => Some(CheckpointCaptureIdentity {
                id: id.clone(),
                assertions: assertions.clone(),
            }),
            Step::Press { .. }
            | Step::KeyEvent { .. }
            | Step::Text { .. }
            | Step::Paste { .. }
            | Step::Mouse { .. }
            | Step::Resize { .. }
            | Step::ExpectExit { .. } => None,
        })
        .collect();
    Ok(CaseCaptureIdentity {
        id: case.id.clone(),
        app: case.app.clone(),
        binary: case.binary.clone(),
        args: case.args.clone(),
        geometry: case.geometry.clone(),
        color_path: case.color_path.clone(),
        legacy_snapshot_root: case.legacy_snapshot_root.clone(),
        screen: case.screen.clone(),
        substep: case.substep.clone(),
        case_set_sha256: case_set_sha256.to_string(),
        input_program_sha256: sha256_bytes(&steps_bytes),
        checkpoints,
    })
}

fn validate_oracle_capture_receipt(
    receipt: &OracleCaptureReceipt,
    policy: &WritePolicyReceipt,
) -> Result<(), String> {
    let expected_state = if receipt.capture_status == "COMPLETE" {
        "capture_recorded"
    } else {
        "capture_partial"
    };
    if receipt.schema != ORACLE_CAPTURE_RECEIPT_SCHEMA
        || receipt.write_policy != *policy
        || write_policy_digest(policy)? != policy.sha256
        || receipt.qualification.status != "blocked"
        || receipt.qualification.reason.trim().is_empty()
        || receipt.admission_status != "NOT_RUN"
        || receipt.execution_anchor_status != "unverified"
        || receipt.state != expected_state
        || !["COMPLETE", "PARTIAL"].contains(&receipt.capture_status.as_str())
        || receipt.case.id != "HELP-HOLLA-004"
        || receipt.oracle_identity.tag_ref != ORACLE_TAG_REF
        || receipt.oracle_identity.tag_object != ORACLE_TAG_OBJECT
        || receipt.oracle_identity.tag_commit != ORACLE_TAG_COMMIT
        || receipt.oracle_identity.source_snapshot_commit != ORACLE_TAG_COMMIT
    {
        return Err(
            "immutable-tag capture receipt has an inconsistent identity or blocked state"
                .to_string(),
        );
    }
    let registry = registry()?;
    let case = registry
        .cases
        .iter()
        .find(|case| case.id == receipt.case.id)
        .ok_or_else(|| {
            "immutable-tag capture case is absent from the current registry".to_string()
        })?;
    validate_case_contract(case)?;
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let current_case_set_digest = digest_tree(&package_root.join("cases"))?;
    let expected_case = tag_case_capture_identity(case, &current_case_set_digest)?;
    if receipt.suite.case_set_digest != current_case_set_digest
        || serde_json::to_vec(&expected_case)
            .map_err(|error| format!("serialize current immutable-tag case: {error}"))?
            != serde_json::to_vec(&receipt.case)
                .map_err(|error| format!("serialize captured immutable-tag case: {error}"))?
    {
        return Err(
            "immutable-tag capture case identity does not match the current suite registry"
                .to_string(),
        );
    }
    if receipt
        .checks
        .iter()
        .any(|check| check.visual_comparison.is_some())
        || receipt
            .checks
            .iter()
            .any(|check| check.dimension == "visual" && check.status == "PASS")
    {
        return Err("immutable-tag capture cannot report a visual comparison PASS".to_string());
    }
    let write_root = canonical_existing_directory_without_symlinks(
        &policy.write_root,
        "immutable-tag capture write root",
    )?;
    if write_root != policy.write_root
        || policy.receipt_path == write_root
        || !policy.receipt_path.starts_with(&write_root)
        || canonical_new_file_path(&policy.receipt_path, "immutable-tag capture receipt")?
            != policy.receipt_path
    {
        return Err(
            "immutable-tag capture receipt must be a canonical new file under its write root"
                .to_string(),
        );
    }
    let mut output_roles = BTreeSet::new();
    for output in &policy.actual_output_roots {
        let canonical = canonical_existing_directory_without_symlinks(
            &output.path,
            "immutable-tag actual-output root",
        )?;
        if canonical != output.path
            || !output_roles.insert(output.role.as_str())
            || output.path == write_root
            || !output.path.starts_with(&write_root)
        {
            return Err("immutable-tag actual-output roots must be unique canonical children of the write root".to_string());
        }
    }
    if output_roles.len() != 1 || !output_roles.contains("oracle") {
        return Err(
            "immutable-tag capture policy must have exactly one oracle actual-output root"
                .to_string(),
        );
    }
    require_git_oid(
        "immutable-tag source snapshot tree",
        &receipt.oracle_identity.source_snapshot_tree,
    )?;
    for (name, digest) in [
        (
            "tag builder receipt",
            receipt.oracle_identity.builder_receipt_sha256.as_str(),
        ),
        (
            "tag source inputs",
            receipt.oracle_identity.source_inputs_sha256.as_str(),
        ),
        (
            "tag build environment",
            receipt.oracle_identity.build_environment_sha256.as_str(),
        ),
        (
            "tag executable",
            receipt.oracle_identity.executable.expected_sha256.as_str(),
        ),
        (
            "tag validator toolchain",
            receipt
                .oracle_identity
                .validator_execution
                .toolchain
                .sha256
                .as_str(),
        ),
        (
            "tag validator Python runtime",
            receipt
                .oracle_identity
                .validator_execution
                .python
                .sha256
                .as_str(),
        ),
        (
            "tag validator Git executable",
            receipt
                .oracle_identity
                .validator_execution
                .git
                .sha256
                .as_str(),
        ),
        (
            "transformed tag validator helper",
            receipt
                .oracle_identity
                .validator_execution
                .executed_helper_sha256
                .as_str(),
        ),
    ] {
        require_sha256(name, digest)?;
    }
    if !receipt
        .oracle_identity
        .validator_execution
        .repository
        .is_absolute()
        || !receipt
            .oracle_identity
            .validator_execution
            .python
            .path
            .is_absolute()
        || !receipt
            .oracle_identity
            .validator_execution
            .git
            .path
            .is_absolute()
        || receipt
            .oracle_identity
            .validator_execution
            .python
            .version
            .trim()
            .is_empty()
        || receipt
            .oracle_identity
            .validator_execution
            .git
            .version
            .trim()
            .is_empty()
        || receipt.oracle_identity.validator_execution.helper_transport
            != TAG_VALIDATOR_HELPER_TRANSPORT
        || receipt
            .oracle_identity
            .validator_execution
            .python_sha256_before
            != receipt.oracle_identity.validator_execution.python.sha256
        || receipt
            .oracle_identity
            .validator_execution
            .python_sha256_after
            != receipt.oracle_identity.validator_execution.python.sha256
        || receipt
            .oracle_identity
            .validator_execution
            .git_sha256_before
            != receipt.oracle_identity.validator_execution.git.sha256
        || receipt.oracle_identity.validator_execution.git_sha256_after
            != receipt.oracle_identity.validator_execution.git.sha256
    {
        return Err(
            "capture validator tool identity must use absolute paths and versions".to_string(),
        );
    }
    let actual_root = policy
        .actual_output_roots
        .iter()
        .find(|root| root.role == "oracle")
        .ok_or_else(|| "capture write policy lacks the oracle actual-output root".to_string())?;
    let mut seen = BTreeSet::new();
    for artifact in &receipt.artifacts {
        if artifact.subject_role != "oracle" || artifact.case_id != receipt.case.id {
            return Err(
                "oracle capture contains an artifact for a different role or case".to_string(),
            );
        }
        let path = canonical_existing_regular_file_without_symlinks(
            &artifact.path,
            "oracle capture artifact",
        )?;
        let relative_stem = capture_relative_stem(case, &artifact.checkpoint_id)?;
        let filename = capture_filename(&artifact.format)
            .ok_or_else(|| format!("unsupported oracle capture format {:?}", artifact.format))?;
        let expected_path = actual_root.path.join(relative_stem).join(filename);
        if path != expected_path {
            return Err(format!(
                "oracle capture artifact is outside its canonical case/checkpoint path: {}",
                path.display()
            ));
        }
        if !path.starts_with(&actual_root.path) {
            return Err(format!(
                "oracle capture artifact escapes its output root: {}",
                path.display()
            ));
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("read oracle capture artifact {}: {error}", path.display()))?;
        if bytes.len() as u64 != artifact.bytes || sha256_bytes(&bytes) != artifact.sha256 {
            return Err(format!(
                "oracle capture artifact digest mismatch: {}",
                path.display()
            ));
        }
        let key = (artifact.checkpoint_id.as_str(), artifact.format.as_str());
        if !seen.insert(key) {
            return Err("oracle capture repeats a checkpoint artifact format".to_string());
        }
    }
    if receipt.capture_status == "COMPLETE" {
        if receipt.artifacts.len() != 40
            || seen.len() != 40
            || receipt.case.checkpoints.len() != 4
            || receipt.case.checkpoints.iter().any(|checkpoint| {
                [
                    "frame_json",
                    "ansi",
                    "html",
                    "png",
                    "ascii",
                    "txt",
                    "ascii_loss_json",
                    "png_fidelity_json",
                    "observations_json",
                    "manifest_json",
                ]
                .iter()
                .any(|format| !seen.contains(&(checkpoint.id.as_str(), *format)))
            })
        {
            return Err(
                "complete oracle capture must bind all 40 artifacts across four checkpoints"
                    .to_string(),
            );
        }
        let expected_assertions = receipt
            .case
            .checkpoints
            .iter()
            .flat_map(|checkpoint| {
                checkpoint.assertions.iter().map(move |assertion| {
                    format!("{}:{}:{}", receipt.case.id, checkpoint.id, assertion.id)
                })
            })
            .collect::<BTreeSet<_>>();
        if expected_assertions.len() != 12
            || expected_assertions.iter().any(|id| {
                receipt
                    .checks
                    .iter()
                    .filter(|check| {
                        check.id == *id
                            && check.subject_role == "oracle"
                            && check.case_id == receipt.case.id
                            && check.dimension == "interaction"
                            && check.status == "PASS"
                    })
                    .count()
                    != 1
            })
            || receipt.checks.iter().any(|check| {
                check.dimension != "visual"
                    && !matches!(check.status.as_str(), "PASS" | "NOT_APPLICABLE")
            })
        {
            return Err("complete oracle capture must record all 12 passing assertions and nonvisual checks".to_string());
        }
        for checkpoint in &receipt.case.checkpoints {
            verify_oracle_capture_manifest(receipt, &actual_root.path, &checkpoint.id)?;
        }
    }
    Ok(())
}

fn capture_filename(format: &str) -> Option<&'static str> {
    match format {
        "frame_json" => Some("frame.json"),
        "ansi" => Some("ansi"),
        "html" => Some("html"),
        "png" => Some("png"),
        "ascii" => Some("ascii"),
        "txt" => Some("txt"),
        "ascii_loss_json" => Some("ascii.loss.json"),
        "png_fidelity_json" => Some("png.fidelity.json"),
        "observations_json" => Some("observations.json"),
        "manifest_json" => Some("manifest.json"),
        _ => None,
    }
}

fn verify_oracle_capture_manifest(
    receipt: &OracleCaptureReceipt,
    actual_root: &Path,
    checkpoint_id: &str,
) -> Result<(), String> {
    let manifest_artifact = receipt
        .artifacts
        .iter()
        .find(|artifact| {
            artifact.checkpoint_id == checkpoint_id && artifact.format == "manifest_json"
        })
        .ok_or_else(|| format!("capture manifest is missing for {checkpoint_id}"))?;
    let manifest_path = canonical_existing_regular_file_without_symlinks(
        &manifest_artifact.path,
        "oracle capture manifest",
    )?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        format!(
            "read oracle capture manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    reject_duplicate_json_keys(&manifest_bytes, "oracle capture manifest")?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse oracle capture manifest: {error}"))?;
    let parent = manifest_path
        .parent()
        .ok_or_else(|| "oracle capture manifest has no parent".to_string())?;
    let relative_stem = parent
        .strip_prefix(actual_root)
        .map_err(|_| "oracle capture manifest escaped its actual-output root".to_string())?
        .to_str()
        .ok_or_else(|| "oracle capture manifest path is not UTF-8".to_string())?
        .replace(std::path::MAIN_SEPARATOR, "/");
    let frame_artifact = receipt
        .artifacts
        .iter()
        .find(|artifact| artifact.checkpoint_id == checkpoint_id && artifact.format == "frame_json")
        .ok_or_else(|| format!("canonical frame is missing for {checkpoint_id}"))?;
    let frame_bytes = fs::read(&frame_artifact.path).map_err(|error| {
        format!(
            "read canonical frame {}: {error}",
            frame_artifact.path.display()
        )
    })?;
    let frame = visual::parse_canonical_frame(&frame_bytes, &receipt.case.geometry)?;
    let frame_digest = frame.digest();
    let expected_renderer = serde_json::to_value(&receipt.renderer)
        .map_err(|error| format!("serialize capture renderer identity: {error}"))?;
    if manifest.as_object().map(serde_json::Map::len) != Some(9)
        || manifest["schema"].as_str() != Some("termrock-e2e/capture-manifest-v1")
        || manifest["case_id"].as_str() != Some(receipt.case.id.as_str())
        || manifest["checkpoint_id"].as_str() != Some(checkpoint_id)
        || manifest["canonical_path"].as_str() != Some(relative_stem.as_str())
        || manifest["frame_digest"].as_u64() != Some(frame_digest)
        || manifest["screen_profile"].as_str() != Some(frame.provenance.profile.as_str())
        || manifest["renderer_profile"] != expected_renderer
    {
        return Err(format!(
            "oracle capture manifest identity mismatch for {checkpoint_id}"
        ));
    }
    let entries = manifest["artifacts"].as_array().ok_or_else(|| {
        format!("oracle capture manifest has no artifact array for {checkpoint_id}")
    })?;
    if entries.len() != 9 {
        return Err(format!(
            "oracle capture manifest must list nine companion artifacts for {checkpoint_id}"
        ));
    }
    let expected = receipt
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.checkpoint_id == checkpoint_id && artifact.format != "manifest_json"
        })
        .collect::<Vec<_>>();
    if expected.len() != 9 {
        return Err(format!(
            "capture receipt does not have nine nonmanifest artifacts for {checkpoint_id}"
        ));
    }
    let mut matched = BTreeSet::new();
    for entry in entries {
        let object = entry
            .as_object()
            .ok_or_else(|| "capture manifest artifact entry must be an object".to_string())?;
        if object.len() != 4 {
            return Err(
                "capture manifest artifact entry has missing or unknown fields".to_string(),
            );
        }
        let format = entry["format"]
            .as_str()
            .ok_or_else(|| "capture manifest artifact format must be a string".to_string())?;
        let path = entry["path"]
            .as_str()
            .ok_or_else(|| "capture manifest artifact path must be a string".to_string())?;
        let sha256 = entry["sha256"]
            .as_str()
            .ok_or_else(|| "capture manifest artifact SHA-256 must be a string".to_string())?;
        let bytes = entry["bytes"].as_u64().ok_or_else(|| {
            "capture manifest artifact size must be an unsigned integer".to_string()
        })?;
        let artifact = expected
            .iter()
            .find(|artifact| {
                let relative = artifact
                    .path
                    .strip_prefix(actual_root)
                    .ok()
                    .and_then(Path::to_str)
                    .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"));
                artifact.format == format && relative.as_deref() == Some(path)
            })
            .ok_or_else(|| format!("capture manifest lists an unbound artifact path {path:?}"))?;
        if artifact.sha256 != sha256
            || artifact.bytes != bytes
            || !matched.insert(artifact.format.as_str())
        {
            return Err(format!(
                "capture manifest artifact digest mismatch for {checkpoint_id}:{format}"
            ));
        }
    }
    if matched.len() != 9 {
        return Err(format!(
            "capture manifest does not bind every format for {checkpoint_id}"
        ));
    }
    Ok(())
}

struct LoadedOracleCapture {
    path: PathBuf,
    sha256: String,
    receipt: OracleCaptureReceipt,
}

fn verify_evidence_reference(
    reference: &EvidenceReference,
    label: &str,
) -> Result<PathBuf, String> {
    let supplied = Path::new(&reference.path);
    if !supplied.is_absolute() {
        return Err(format!("{label} path must be absolute"));
    }
    require_sha256(&format!("{label} SHA-256"), &reference.sha256)?;
    let path = canonical_existing_regular_file_without_symlinks(supplied, label)?;
    if path.display().to_string() != reference.path {
        return Err(format!("{label} path is not canonical"));
    }
    let actual = sha256_file(&path)?;
    if actual != reference.sha256 {
        return Err(format!("{label} bytes no longer match the captured digest"));
    }
    Ok(path)
}

fn load_oracle_capture(
    supplied_path: &Path,
    expected_sha256: &str,
) -> Result<LoadedOracleCapture, String> {
    require_sha256("oracle capture receipt SHA-256", expected_sha256)?;
    if !supplied_path.is_absolute() {
        return Err("oracle capture receipt path must be absolute".to_string());
    }
    let path =
        canonical_existing_regular_file_without_symlinks(supplied_path, "oracle capture receipt")?;
    let checkout_root = suite_checkout_root(Path::new(env!("CARGO_MANIFEST_DIR")))?;
    if path.starts_with(&checkout_root) {
        return Err("oracle capture receipt must be stored outside the suite checkout".to_string());
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("read oracle capture receipt {}: {error}", path.display()))?;
    let sha256 = sha256_bytes(&bytes);
    if sha256 != expected_sha256 {
        return Err(format!(
            "oracle capture receipt SHA-256 mismatch: expected {expected_sha256}, got {sha256}"
        ));
    }
    reject_duplicate_json_keys(&bytes, "oracle capture receipt")?;
    let envelope: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse oracle capture receipt envelope: {error}"))?;
    if envelope["checks"].as_array().is_none_or(|checks| {
        checks.iter().any(|check| {
            check
                .as_object()
                .is_none_or(|object| object.contains_key("visual_comparison"))
        })
    }) {
        return Err(
            "actual-only oracle capture cannot contain visual-comparison evidence".to_string(),
        );
    }
    let receipt: OracleCaptureReceipt = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse oracle capture receipt: {error}"))?;
    let canonical = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize oracle capture receipt: {error}"))?;
    if canonical != bytes {
        return Err("oracle capture receipt must use canonical pretty JSON".to_string());
    }
    if receipt.write_policy.receipt_path != path {
        return Err(
            "oracle capture receipt path differs from its protected write policy".to_string(),
        );
    }
    validate_oracle_capture_receipt(&receipt, &receipt.write_policy)?;
    if receipt.validator_helper.sha256 != TAG_SOURCE_VALIDATOR_SHA256 {
        return Err("tag source validator is not the pinned immutable-tag helper".to_string());
    }
    verify_evidence_reference(&receipt.validator_helper, "tag source validator")?;
    verify_evidence_reference(
        &receipt.oracle_identity.validator_execution.toolchain,
        "tag validator toolchain",
    )?;
    if Path::new(&receipt.validator_helper.path).starts_with(&checkout_root)
        || Path::new(&receipt.oracle_identity.validator_execution.toolchain.path)
            .starts_with(&checkout_root)
        || receipt
            .oracle_identity
            .validator_execution
            .repository
            .starts_with(&checkout_root)
        || receipt
            .oracle_identity
            .validator_execution
            .python
            .path
            .starts_with(&checkout_root)
        || receipt
            .oracle_identity
            .validator_execution
            .git
            .path
            .starts_with(&checkout_root)
    {
        return Err("tag validator inputs must be stored outside the suite checkout".to_string());
    }
    require_tag_validator_outside_write_roots(
        Path::new(&receipt.validator_helper.path),
        &receipt.oracle_identity.validator_execution,
        &receipt.write_policy.write_root,
        &receipt.write_policy.actual_output_roots,
    )?;
    if receipt.capture_status != "COMPLETE" {
        return Err("partial oracle capture cannot be admitted".to_string());
    }

    let builder_path = verify_evidence_reference(&receipt.builder_evidence, "tag builder receipt")?;
    if builder_path.starts_with(&checkout_root) {
        return Err("tag builder evidence must be stored outside the suite checkout".to_string());
    }
    let builder_bytes = fs::read(&builder_path).map_err(|error| {
        format!(
            "read tag builder receipt {}: {error}",
            builder_path.display()
        )
    })?;
    reject_duplicate_json_keys(&builder_bytes, "tag builder receipt")?;
    let builder: TagBuilderEvidence = serde_json::from_slice(&builder_bytes)
        .map_err(|error| format!("parse tag builder receipt: {error}"))?;
    revalidate_captured_tag_source(&receipt, &builder, &builder_path)?;
    let builder_run_path =
        verify_evidence_reference(&receipt.builder_run, "tag builder run record")?;
    let expected_builder_run = builder_path
        .parent()
        .ok_or_else(|| "tag builder receipt has no parent directory".to_string())?
        .join("run.json");
    if builder_run_path != expected_builder_run {
        return Err(
            "tag builder run evidence must be the sibling run.json validated with the receipt"
                .to_string(),
        );
    }
    if builder_run_path.starts_with(&checkout_root) {
        return Err(
            "tag builder run evidence must be stored outside the suite checkout".to_string(),
        );
    }
    let source_inputs_bytes = serde_json::to_vec(&builder.source_snapshot)
        .map_err(|error| format!("serialize tag source inputs: {error}"))?;
    let build_environment_bytes = serde_json::to_vec(&builder.build_environment)
        .map_err(|error| format!("serialize tag build environment: {error}"))?;
    let nested_builder_sha = builder_receipt_digest(&builder.builder_receipt)?;
    if builder.schema != TAG_BUILD_EVIDENCE_SCHEMA
        || builder.run_id != receipt.run_id
        || builder.purpose != "frozen-visual-tag-holla-build-only"
        || builder.build_result != "PASS"
        || builder.qualification.status != "blocked"
        || builder.capture_status != "NOT_RUN"
        || builder.admission_status != "NOT_RUN"
        || builder.oracle_lineage.tag_ref != receipt.oracle_identity.tag_ref
        || builder.oracle_lineage.tag_object != receipt.oracle_identity.tag_object
        || builder.oracle_lineage.tag_commit != receipt.oracle_identity.tag_commit
        || builder.source_snapshot.source_commit != receipt.oracle_identity.source_snapshot_commit
        || builder.source_snapshot.tree_oid != receipt.oracle_identity.source_snapshot_tree
        || Path::new(&builder.source_snapshot.materialized_root)
            != receipt.oracle_identity.source_snapshot_root
        || builder.build_artifact_root != receipt.oracle_identity.build_artifact_root
        || builder.build != receipt.oracle_identity.build
        || builder.builder_receipt_sha256 != receipt.oracle_identity.builder_receipt_sha256
        || nested_builder_sha != receipt.oracle_identity.builder_receipt_sha256
        || sha256_bytes(&source_inputs_bytes) != receipt.oracle_identity.source_inputs_sha256
        || sha256_bytes(&build_environment_bytes)
            != receipt.oracle_identity.build_environment_sha256
        || builder.executable.path != receipt.oracle_identity.executable.path
        || builder.executable.sha256 != receipt.oracle_identity.executable.expected_sha256
    {
        return Err(
            "oracle capture does not match the raw immutable-tag builder evidence".to_string(),
        );
    }

    let source_root = canonical_existing_directory_without_symlinks(
        &receipt.oracle_identity.source_snapshot_root,
        "captured immutable-tag source snapshot",
    )?;
    let artifact_root = canonical_existing_directory_without_symlinks(
        &receipt.oracle_identity.build_artifact_root,
        "captured immutable-tag build artifacts",
    )?;
    let actual_root = receipt
        .write_policy
        .actual_output_roots
        .iter()
        .find(|root| root.role == "oracle")
        .ok_or_else(|| "capture write policy lacks the oracle actual-output root".to_string())?;
    if source_root.starts_with(&checkout_root)
        || artifact_root.starts_with(&checkout_root)
        || actual_root.path.starts_with(&checkout_root)
    {
        return Err(
            "tag source, build, and actual-output roots must be outside the suite checkout"
                .to_string(),
        );
    }
    let executable_path = canonical_existing_regular_file_without_symlinks(
        &receipt.oracle_identity.executable.path,
        "captured immutable-tag executable",
    )?;
    if source_root != receipt.oracle_identity.source_snapshot_root
        || artifact_root != receipt.oracle_identity.build_artifact_root
        || !executable_path.starts_with(&artifact_root)
        || receipt.oracle_identity.executable.actual_sha256.as_deref()
            != Some(receipt.oracle_identity.executable.expected_sha256.as_str())
        || sha256_file(&executable_path)? != receipt.oracle_identity.executable.expected_sha256
    {
        return Err(
            "captured immutable-tag source/build/executable paths or hashes changed".to_string(),
        );
    }
    Ok(LoadedOracleCapture {
        path,
        sha256,
        receipt,
    })
}

fn revalidate_captured_tag_source(
    capture: &OracleCaptureReceipt,
    builder: &TagBuilderEvidence,
    builder_path: &Path,
) -> Result<(), String> {
    let oracle = &capture.oracle_identity;

    let (toolchain, toolchain_reference) = load_caller_pinned_tag_validator_toolchain()?;
    require_caller_pinned_toolchain_reference(
        &oracle.validator_execution.toolchain,
        &toolchain_reference,
    )?;
    if toolchain.python != oracle.validator_execution.python
        || toolchain.git != oracle.validator_execution.git
    {
        return Err("captured validator tool identity changed".to_string());
    }
    let (identity, helper, execution) = execute_tag_source_validator(
        builder_path,
        &capture.builder_evidence.sha256,
        Path::new(&capture.validator_helper.path),
        &oracle.validator_execution.repository,
        &toolchain,
        oracle.validator_execution.toolchain.clone(),
    )?;
    if helper.path.display().to_string() != capture.validator_helper.path
        || helper.sha256 != capture.validator_helper.sha256
        || execution != oracle.validator_execution
    {
        return Err("captured source-validator execution identity changed".to_string());
    }
    validate_tag_record_binding(
        builder,
        &identity,
        builder_path,
        &capture.builder_evidence.sha256,
    )?;
    let source_inputs_bytes = serde_json::to_vec(&builder.source_snapshot)
        .map_err(|error| format!("serialize revalidated tag source inputs: {error}"))?;
    let captured_snapshot = TagValidatedSourceSnapshot {
        source_commit: oracle.source_snapshot_commit.clone(),
        tree_oid: oracle.source_snapshot_tree.clone(),
        materialized_root: oracle.source_snapshot_root.clone(),
        included_path_blob_map_sha256: builder
            .source_snapshot
            .included_path_blob_map_sha256
            .clone(),
        included_file_count: oracle.source_snapshot_included_file_count,
    };
    if sha256_bytes(&source_inputs_bytes) != oracle.source_inputs_sha256
        || validate_tag_source_snapshot_still_matches(&captured_snapshot, &identity.source_snapshot)
            .is_err()
        || identity.source_snapshot.included_path_blob_map_sha256
            != builder.source_snapshot.included_path_blob_map_sha256
        || identity.source_snapshot.included_file_count
            != builder.source_snapshot.included_file_count
    {
        return Err("materialized immutable-tag source changed since capture".to_string());
    }
    Ok(())
}

fn validate_tag_source_snapshot_still_matches(
    captured: &TagValidatedSourceSnapshot,
    current: &TagValidatedSourceSnapshot,
) -> Result<(), String> {
    if captured != current {
        return Err("materialized immutable-tag source changed since capture".to_string());
    }
    Ok(())
}

fn build_admission_binding(
    generation: &visual::ExpectedGeneration,
    capture: &LoadedOracleCapture,
    suite_revision: &str,
    suite_digest: &str,
    case: &Case,
    profile: &Profile,
    renderer: &RendererIdentity,
) -> Result<admission::AdmissionBinding, String> {
    let receipt = &capture.receipt;
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_set_digest = digest_tree(&package_root.join("cases"))?;
    let profile_digest = sha256_file(&package_root.join("profile.json"))?;
    let dependency_lock_sha256 = sha256_file(&package_root.join("Cargo.lock"))?;
    let case_identity = tag_case_capture_identity(case, &case_set_digest)?;
    let case_bytes = serde_json::to_vec(&case_identity)
        .map_err(|error| format!("serialize selected case identity: {error}"))?;
    let captured_case_bytes = serde_json::to_vec(&receipt.case)
        .map_err(|error| format!("serialize captured case identity: {error}"))?;
    if generation.case_id != case.id
        || generation.case_set_digest != case_set_digest
        || generation.profile_digest != profile_digest
        || generation.dependency_lock_sha256 != dependency_lock_sha256
        || generation.renderer_sha256 != renderer.hash
        || receipt.suite.revision != suite_revision
        || receipt.suite.digest != suite_digest
        || receipt.suite.case_set_digest != case_set_digest
        || receipt.suite.profile_digest != profile_digest
        || receipt.suite.dependency_lock_sha256 != dependency_lock_sha256
        || receipt.renderer != *renderer
        || captured_case_bytes != case_bytes
    {
        return Err("capture or generation does not match the current suite, case, profile, lock, or renderer".to_string());
    }
    let checkout_root = suite_checkout_root(package_root)?;
    if generation.root.starts_with(&checkout_root) {
        return Err(
            "expected-generation input must be stored outside the suite checkout".to_string(),
        );
    }
    if generation.oracle.tag_ref != receipt.oracle_identity.tag_ref
        || generation.oracle.tag_object != receipt.oracle_identity.tag_object
        || generation.oracle.tag_commit != receipt.oracle_identity.tag_commit
        || generation.oracle.capture_run_sha256 != capture.sha256
        || generation.oracle.source_inputs_sha256 != receipt.oracle_identity.source_inputs_sha256
        || generation.oracle.build_environment_sha256
            != receipt.oracle_identity.build_environment_sha256
        || generation.oracle.builder_receipt_sha256
            != receipt.oracle_identity.builder_receipt_sha256
        || generation.oracle.oracle_executable_sha256
            != receipt.oracle_identity.executable.expected_sha256
    {
        return Err(
            "expected-generation manifest does not bind this immutable-tag capture".to_string(),
        );
    }
    for checkpoint in checkpoints(case) {
        let (expected_frame, expected_png) = generation.checkpoint_refs(checkpoint)?;
        for (format, expected) in [("frame_json", expected_frame), ("png", expected_png)] {
            let artifact = receipt
                .artifacts
                .iter()
                .find(|artifact| artifact.checkpoint_id == checkpoint && artifact.format == format)
                .ok_or_else(|| format!("tag capture lacks {format} for {checkpoint}"))?;
            if artifact.sha256 != expected.sha256 || artifact.bytes != expected.bytes {
                return Err(format!(
                    "expected-generation {format} differs from the immutable-tag capture at {checkpoint}"
                ));
            }
        }
    }
    let checkpoint_ids = receipt
        .case
        .checkpoints
        .iter()
        .map(|checkpoint| checkpoint.id.clone())
        .collect();
    let binding = admission::AdmissionBinding {
        generation: admission::GenerationBinding {
            id: generation.id.clone(),
            tree_sha256: generation.tree_sha256.clone(),
            manifest_sha256: generation.manifest_sha256.clone(),
        },
        oracle: admission::OracleBinding {
            tag_ref: receipt.oracle_identity.tag_ref.clone(),
            tag_object: receipt.oracle_identity.tag_object.clone(),
            tag_commit: receipt.oracle_identity.tag_commit.clone(),
            source_snapshot_tree: receipt.oracle_identity.source_snapshot_tree.clone(),
            source_inputs_sha256: receipt.oracle_identity.source_inputs_sha256.clone(),
            build_environment_sha256: receipt.oracle_identity.build_environment_sha256.clone(),
            builder_receipt_sha256: receipt.oracle_identity.builder_receipt_sha256.clone(),
            executable_sha256: receipt.oracle_identity.executable.expected_sha256.clone(),
            validator_toolchain: receipt
                .oracle_identity
                .validator_execution
                .toolchain
                .clone(),
        },
        capture: EvidenceReference {
            path: capture.path.display().to_string(),
            sha256: capture.sha256.clone(),
        },
        suite: admission::SuiteBinding {
            revision: suite_revision.to_string(),
            digest: suite_digest.to_string(),
            case_set_digest,
            profile_digest,
            dependency_lock_sha256,
            tuiscotti_revision: profile.tool_revision.clone(),
        },
        case: admission::CaseBinding {
            id: case.id.clone(),
            input_program_sha256: case_identity.input_program_sha256,
            checkpoint_ids,
        },
        renderer_sha256: renderer.hash.clone(),
    };
    admission::validate_binding(&binding)?;
    Ok(binding)
}

fn require_admission_write_targets_disjoint(
    write_root: &ResolvedDirectory,
    output: &ResolvedPath,
    protected_paths: &[PathBuf],
) -> Result<(), String> {
    let resolved_write_root = resolved_path_from_directory(write_root);
    for path in protected_paths {
        let resolved = resolve_path_for_overlap(path, "admission protected input")?;
        if physical_paths_overlap(&resolved_write_root, &resolved)
            || physical_paths_overlap(output, &resolved)
        {
            return Err(format!(
                "admission output overlaps protected input {}",
                path.display()
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn admit_expected_generation(
    generation_id: &str,
    generation_root: &Path,
    generation_sha256: &str,
    capture_path: &Path,
    capture_sha256: &str,
    qualification_review_path: &Path,
    qualification_review_sha256: &str,
    write_root_path: &Path,
    admission_path: &Path,
) -> Result<AdmissionWriteReceipt, String> {
    let current_suite_digest = suite_digest()?;
    verify_compiled_suite_digest(COMPILED_SUITE_SHA256, &current_suite_digest)?;
    let registry = registry()?;
    let case = registry
        .cases
        .iter()
        .find(|case| case.id == "HELP-HOLLA-004")
        .ok_or_else(|| "suite lacks HELP-HOLLA-004".to_string())?;
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
        return Err("resolved renderer does not match the compiled suite profile".to_string());
    }
    let generation_input = ExpectedGenerationInput {
        id: generation_id.to_string(),
        root: Some(generation_root.to_path_buf()),
        sha256: generation_sha256.to_string(),
    };
    let generation = visual::load_expected_generation(
        &generation_input,
        case,
        &digest_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("cases"))?,
        &sha256_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("profile.json"))?,
        &sha256_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"))?,
        &profile.tool_revision,
        &renderer,
    )?;
    let capture = load_oracle_capture(capture_path, capture_sha256)?;
    let binding = build_admission_binding(
        &generation,
        &capture,
        &registry.suite_revision,
        &current_suite_digest,
        case,
        &profile,
        &renderer,
    )?;
    let review = admission::load_qualification_review(
        Some(qualification_review_path),
        Some(qualification_review_sha256),
        &binding,
    )?;
    let receipt = admission::new_admission_receipt(binding.clone(), &review)?;
    let receipt_bytes = serde_json::to_vec(&receipt)
        .map_err(|error| format!("serialize generation admission receipt: {error}"))?;

    let write_root =
        resolve_existing_directory_without_symlinks(write_root_path, "admission write root")?;
    let output_path = canonical_new_file_path(admission_path, "generation admission receipt")?;
    if path_exists_no_follow(&output_path)? {
        return Err(format!(
            "generation admission destination already exists: {}",
            output_path.display()
        ));
    }
    let output = resolve_path_for_overlap(&output_path, "generation admission receipt")?;
    if !path_is_within_directory(&write_root, &output) {
        return Err(
            "generation admission destination must be beneath its explicit write root".to_string(),
        );
    }
    let suite_source_root = canonical_existing_directory_without_symlinks(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        "suite source package",
    )?;
    let mut protected_paths = vec![
        suite_source_root,
        generation.root.clone(),
        capture.receipt.write_policy.write_root.clone(),
        capture.receipt.oracle_identity.source_snapshot_root.clone(),
        capture.receipt.oracle_identity.build_artifact_root.clone(),
    ];
    protected_paths.extend(tag_validator_protected_paths(
        Path::new(&capture.receipt.validator_helper.path),
        &capture.receipt.oracle_identity.validator_execution,
    )?);
    for evidence_path in [
        capture.path.as_path(),
        review.path.as_path(),
        Path::new(&capture.receipt.builder_evidence.path),
        Path::new(&capture.receipt.builder_run.path),
        Path::new(&capture.receipt.validator_helper.path),
    ]
    .into_iter()
    .chain(
        review
            .review
            .execution_anchor_evidence
            .iter()
            .map(|evidence| Path::new(&evidence.path)),
    ) {
        protected_paths.push(canonical_existing_regular_file_without_symlinks(
            evidence_path,
            "admission evidence",
        )?);
        protected_paths.push(
            evidence_path
                .parent()
                .ok_or_else(|| "admission evidence has no parent".to_string())?
                .to_path_buf(),
        );
    }
    if let Some(actual_root) = capture
        .receipt
        .write_policy
        .actual_output_roots
        .iter()
        .find(|root| root.role == "oracle")
    {
        protected_paths.push(actual_root.path.clone());
    }
    require_admission_write_targets_disjoint(&write_root, &output, &protected_paths)?;
    let current_generation = visual::load_expected_generation(
        &generation_input,
        case,
        &digest_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("cases"))?,
        &sha256_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("profile.json"))?,
        &sha256_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock"))?,
        &profile.tool_revision,
        &renderer,
    )?;
    let current_capture = load_oracle_capture(capture_path, capture_sha256)?;
    let current_binding = build_admission_binding(
        &current_generation,
        &current_capture,
        &registry.suite_revision,
        &current_suite_digest,
        case,
        &profile,
        &renderer,
    )?;
    let current_review = admission::load_qualification_review(
        Some(qualification_review_path),
        Some(qualification_review_sha256),
        &current_binding,
    )?;
    if current_binding != binding || current_review.sha256 != review.sha256 {
        return Err("admission inputs changed while the receipt was being prepared".to_string());
    }
    publish_receipt_no_replace(&write_root.path, &output_path, &receipt_bytes)?;
    Ok(AdmissionWriteReceipt {
        schema: "termrock-spec/visual-generation-admission-write-result-v1".to_string(),
        admission_path: output_path,
        admission_sha256: sha256_bytes(&receipt_bytes),
        generation_id: generation.id,
        generation_tree_sha256: generation.tree_sha256,
        decision: "admitted".to_string(),
    })
}

fn checkpoints(case: &Case) -> Vec<&str> {
    case.steps
        .iter()
        .filter_map(|step| match step {
            Step::Checkpoint { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .collect()
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

fn capture_relative_stem(case: &Case, checkpoint: &str) -> Result<String, String> {
    let output_substep = checkpoint_substep(case, checkpoint);
    let geometry = checkpoint_geometry(case, checkpoint)?;
    let relative = format!(
        "{}/{}/{}/{}x{}/{}",
        case.app, case.screen, output_substep, geometry.cols, geometry.rows, case.color_path
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
