//! One-time, read-only audit of the Cargo source trees added during VIS-06 R11.
//!
//! This ignored case compares the 153 new extracted package trees with the
//! lock-checksummed archives used by the R11 run. It never unpacks an archive
//! and never writes to the Cargo cache. Its receipt is separate from the
//! original R11 result.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use flate2::read::MultiGzDecoder;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tar::{Archive, EntryType};

const R11_RESULT_PATH: &str = "/private/tmp/termrock-vis06-tag-linker-r2-20261010/execution/tag-r11-fresh-target/attempt-01/runner-result.json";
const R11_RESULT_SHA256: &str = "dbdf8853ed8b616c09209c096560194372a2713518f4a6676fa6f3585beccd73";
const CACHE_PREPARATION_PATH: &str = "/private/tmp/termrock-vis06-tag-linker-r1-20261009/execution/tag-r4-20261009-ba912a28e055/cache-preparation.json";
const CACHE_PREPARATION_SHA256: &str =
    "3200168cb388657b90009d49f938479ecfc8d4d023d6c998413ddacee548cb12";
const CACHE_HOME_PATH: &str = "/private/tmp/termrock-vis06-cargo-home-tag-r4-20261009-ba912a28e055";
const R14_SOURCE_TREE_SHA256: &str =
    "2cca2a085971778b4ba437b4c360249975f8c8a6d78d1b9daead1429b7353a9a";
const R14_VISIBILITY_PACKAGE_SHA256: &str =
    "75e0a945b2c7eb4c2bd7cbb0a5ad88551a83a74358b919b636e9a2b680c7ee37";
const REGISTRY_HASH: &str = "index.crates.io-1949cf8c6b5b557f";
const REFERENCE_TAG_REF: &str = "refs/tags/visual-baseline";
const REFERENCE_TAG_OBJECT: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const REFERENCE_TAG_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const REFERENCE_LOCK_SHA256: &str =
    "3edfd7975cfe1449aad4b30d65dbbcb9d695f01dfd3e1eb9025ed346f0a94460";
const TEST_LOCK_SHA256: &str = "d6372475262b723ebdfe3bec9134ba6c675ff1d3becb3fa4625eb9e38c40b2c2";
const EXPECTED_UNMANIFESTED_FILES: usize = 6_054;
const EXPECTED_UNMANIFESTED_DIRECTORIES: usize = 1_177;
const EXPECTED_UNMANIFESTED_PATHS: usize = 7_231;
const EXPECTED_NEW_PACKAGE_ROOTS: usize = 153;
const EXPECTED_WARM_FILES: usize = 1_365;
const EXPECTED_WARM_DIRECTORIES: usize = 420;
const EXPECTED_LOCKED_ARCHIVES: usize = 319;
const MAX_SOURCE_TREE_NODES: usize =
    EXPECTED_WARM_FILES + EXPECTED_WARM_DIRECTORIES + EXPECTED_UNMANIFESTED_PATHS;
const MAX_SOURCE_TREE_DEPTH: usize = 64;
const MAX_RESULT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PREPARATION_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SOURCE_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_TREE_BYTES: u64 = 512 * 1024 * 1024;
const TAR_OVERHEAD_BUDGET_PER_OBSERVED_NODE: u64 = 4 * 1024;
const TAR_END_MARKER_BUDGET: u64 = 1024;
const EXTRA_TAR_NODE_BUDGET: u64 = 16;
const RECEIPT_SCHEMA: &str = "termrock-vis06-cache-source-audit/v1";
const AUDIT_TEST_SOURCE: &str = include_str!("cache_source_audit.rs");
const AUDIT_PACKAGE_MANIFEST: &[u8] = include_bytes!("../Cargo.toml");
const AUDIT_PACKAGE_LOCK: &[u8] = include_bytes!("../Cargo.lock");
const CLAIM_WORK_ID: &str = "VIS-18";
const CLAIM_TOKEN: &str =
    "visibility-vis18-cache-source-audit-q64-0031d378-5d8d-46f6-8c54-420ca9b5dac6";
const CLAIM_QUEUE_REVISION: u64 = 65;
const CLAIM_BASE_SHA: &str = "766ae1e925e32b4b28aa9100bb3fde5279aa223b";

#[test]
#[ignore = "audits the recorded VIS-06 R11 Cargo extraction; read-only and separate from ordinary tests"]
fn audits_r11_unmanifested_sources_against_locked_crate_archives() {
    let receipt_path = required_absolute_env_path("TERMROCK_VIS18_AUDIT_RECEIPT_PATH")
        .expect("set a fresh external TERMROCK_VIS18_AUDIT_RECEIPT_PATH");
    validate_receipt_destination(
        &receipt_path,
        Path::new(R11_RESULT_PATH),
        Path::new(CACHE_PREPARATION_PATH),
        Path::new(CACHE_HOME_PATH),
    )
    .expect("receipt destination must be fresh and outside protected inputs");
    let (report, passed) = match validate_receipt_destination(
        &receipt_path,
        Path::new(R11_RESULT_PATH),
        Path::new(CACHE_PREPARATION_PATH),
        Path::new(CACHE_HOME_PATH),
    )
    .and_then(|()| audit_r11_sources(&receipt_path))
    {
        Ok(report) => (report, true),
        Err(failure) => (
            json!({
                "schema": RECEIPT_SCHEMA,
                "status": "FAIL",
                "failed_stage": failure.stage,
                "errors": [failure.message],
                "claim": {
                    "work_id": CLAIM_WORK_ID,
                    "token": CLAIM_TOKEN,
                    "accepted_queue_revision": CLAIM_QUEUE_REVISION,
                    "base_sha": CLAIM_BASE_SHA,
                },
                "original_r11_result_sha256": R11_RESULT_SHA256,
                "original_cache_preparation_sha256": CACHE_PREPARATION_SHA256,
                "historical_added_file_digest_binding": "NOT_AVAILABLE_IN_R11_RESULT",
                "scope_boundary": "This failure report does not establish historical content identity for the unmanifested additions.",
            }),
            false,
        ),
    };

    validate_receipt_destination(
        &receipt_path,
        Path::new(R11_RESULT_PATH),
        Path::new(CACHE_PREPARATION_PATH),
        Path::new(CACHE_HOME_PATH),
    )
    .expect("receipt destination must remain fresh and outside protected inputs");
    write_new_receipt(&receipt_path, &report)
        .expect("write a new external audit receipt without replacing an existing file");
    let receipt_sha256 = hash_file(&receipt_path)
        .expect("hash the external audit receipt after writing")
        .1;
    eprintln!(
        "VIS-18 cache source audit receipt: path={} sha256={}",
        receipt_path.display(),
        receipt_sha256
    );
    assert!(
        passed,
        "cache source audit failed; see {}",
        receipt_path.display()
    );
}

#[derive(Debug)]
struct AuditFailure {
    stage: &'static str,
    message: String,
}

impl AuditFailure {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }
}

type AuditResult<T> = Result<T, AuditFailure>;

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeKind {
    File,
    Directory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedNode {
    kind: NodeKind,
    bytes: u64,
    sha256: Option<String>,
    mode: u64,
    dev: String,
    ino: String,
}

#[derive(Clone, Debug)]
struct ExpectedNode {
    kind: NodeKind,
    bytes: Option<u64>,
    sha256: Option<String>,
    mode: Option<u64>,
    dev: Option<String>,
    ino: Option<String>,
}

#[derive(Clone, Debug)]
struct ArchiveRow {
    name: String,
    version: String,
    checksum: String,
    bytes: u64,
}

#[derive(Clone, Debug)]
struct PackageRoot {
    relative_path: String,
    row: ArchiveRow,
}

#[derive(Clone, Debug)]
struct PackageReport {
    name: String,
    version: String,
    root: String,
    archive_path: String,
    archive_mode: u64,
    archive_dev: String,
    archive_ino: String,
    archive_sha256: String,
    archive_bytes: u64,
    source_file_count: usize,
    source_directory_count: usize,
    archived_payload_bytes: u64,
    decompressed_cap_bytes: u64,
    decompressed_bytes: u64,
    archive_entry_count: usize,
}

fn audit_r11_sources(receipt_path: &Path) -> AuditResult<Value> {
    let result_path = required_absolute_env_path("TERMROCK_VIS18_R11_RESULT_PATH")
        .map_err(|message| AuditFailure::new("input_paths", message))?;
    let preparation_path = required_absolute_env_path("TERMROCK_VIS18_CACHE_PREPARATION_PATH")
        .map_err(|message| AuditFailure::new("input_paths", message))?;
    if result_path != Path::new(R11_RESULT_PATH) {
        return Err(AuditFailure::new(
            "input_paths",
            format!(
                "R11 result path is not the accepted input: {}",
                result_path.display()
            ),
        ));
    }
    if preparation_path != Path::new(CACHE_PREPARATION_PATH) {
        return Err(AuditFailure::new(
            "input_paths",
            format!(
                "cache-preparation path is not the accepted input: {}",
                preparation_path.display()
            ),
        ));
    }

    let result_input = read_pinned_json(
        &result_path,
        R11_RESULT_SHA256,
        MAX_RESULT_BYTES,
        "r11_result",
    )?;
    let preparation_input = read_pinned_json(
        &preparation_path,
        CACHE_PREPARATION_SHA256,
        MAX_PREPARATION_BYTES,
        "cache_preparation",
    )?;
    let result = &result_input.value;
    let preparation = &preparation_input.value;
    validate_bound_receipts(result, preparation, &result_path, &preparation_path)?;

    let cache_home = PathBuf::from(CACHE_HOME_PATH);
    validate_cache_home(result, &cache_home)?;
    validate_receipt_destination(receipt_path, &result_path, &preparation_path, &cache_home)?;

    let expected_nodes = expected_source_nodes(result)?;
    let actual_nodes = walk_registry_source_tree(&cache_home)?;
    validate_source_inventory(&expected_nodes, &actual_nodes)?;
    let registry_source_file_bytes = actual_nodes
        .values()
        .filter(|node| node.kind == NodeKind::File)
        .try_fold(0u64, |total, node| {
            total.checked_add(node.bytes).ok_or_else(|| {
                AuditFailure::new("source_inventory", "registry source byte sum overflow")
            })
        })?;

    let archive_rows = archive_rows(preparation)?;
    let package_roots = package_roots_from_errors(result, &archive_rows)?;
    let mut package_reports = Vec::with_capacity(package_roots.len());
    let mut package_error_paths = BTreeSet::new();

    for package in &package_roots {
        for path in actual_nodes.keys() {
            if path == &package.relative_path
                || path.starts_with(&format!("{}/", package.relative_path))
            {
                package_error_paths.insert(path.clone());
            }
        }
        let report = audit_package_archive(&cache_home, package, &actual_nodes)?;
        package_reports.push(report);
    }

    if package_error_paths.len() != EXPECTED_UNMANIFESTED_PATHS {
        return Err(AuditFailure::new(
            "unmanifested_coverage",
            format!(
                "the audited roots cover {} source paths, expected {}",
                package_error_paths.len(),
                EXPECTED_UNMANIFESTED_PATHS
            ),
        ));
    }

    let before_result_sha256 = result_input.sha256.clone();
    let before_preparation_sha256 = preparation_input.sha256.clone();
    for report in &package_reports {
        let archive_path = Path::new(&report.archive_path);
        let metadata = fs::symlink_metadata(archive_path).map_err(|error| {
            AuditFailure::new(
                "read_only_postflight",
                format!("{}: {error}", archive_path.display()),
            )
        })?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata_link_count(&metadata) != 1
            || metadata.len() != report.archive_bytes
            || metadata_mode(&metadata) != report.archive_mode
            || metadata_dev(&metadata) != report.archive_dev
            || metadata_ino(&metadata) != report.archive_ino
        {
            return Err(AuditFailure::new(
                "read_only_postflight",
                format!(
                    "archive path or identity changed during audit: {}",
                    archive_path.display()
                ),
            ));
        }
        let current_sha256 = hash_file(archive_path)
            .map_err(|message| AuditFailure::new("read_only_postflight", message))?
            .1;
        let after_hash_metadata = fs::symlink_metadata(archive_path).map_err(|error| {
            AuditFailure::new(
                "read_only_postflight",
                format!("{}: {error}", archive_path.display()),
            )
        })?;
        if after_hash_metadata.file_type().is_symlink()
            || !after_hash_metadata.is_file()
            || metadata_link_count(&after_hash_metadata) != 1
            || after_hash_metadata.len() != report.archive_bytes
            || metadata_mode(&after_hash_metadata) != report.archive_mode
            || metadata_dev(&after_hash_metadata) != report.archive_dev
            || metadata_ino(&after_hash_metadata) != report.archive_ino
        {
            return Err(AuditFailure::new(
                "read_only_postflight",
                format!(
                    "archive path or identity changed while hashing: {}",
                    archive_path.display()
                ),
            ));
        }
        if current_sha256 != report.archive_sha256 {
            return Err(AuditFailure::new(
                "read_only_postflight",
                format!("archive changed during audit: {}", report.archive_path),
            ));
        }
    }
    let after_result_sha256 = hash_file(&result_path)
        .map_err(|message| AuditFailure::new("read_only_postflight", message))?
        .1;
    let after_preparation_sha256 = hash_file(&preparation_path)
        .map_err(|message| AuditFailure::new("read_only_postflight", message))?
        .1;
    validate_cache_home(result, &cache_home)
        .map_err(|failure| AuditFailure::new("read_only_postflight", failure.message))?;
    let after_nodes = walk_registry_source_tree(&cache_home)?;
    if before_result_sha256 != after_result_sha256
        || before_preparation_sha256 != after_preparation_sha256
        || actual_nodes != after_nodes
    {
        return Err(AuditFailure::new(
            "read_only_postflight",
            "an original receipt or audited registry source path changed during the audit",
        ));
    }

    Ok(json!({
        "schema": RECEIPT_SCHEMA,
        "status": "PASS",
        "claim": {
            "work_id": CLAIM_WORK_ID,
            "token": CLAIM_TOKEN,
            "accepted_queue_revision": CLAIM_QUEUE_REVISION,
            "base_sha": CLAIM_BASE_SHA,
        },
        "audit_test_source_sha256": sha256_bytes(AUDIT_TEST_SOURCE.as_bytes()),
        "audit_package_manifest_sha256": sha256_bytes(AUDIT_PACKAGE_MANIFEST),
        "audit_package_lock_sha256": sha256_bytes(AUDIT_PACKAGE_LOCK),
        "source_fixture": {
            "r14_source_tree_sha256": R14_SOURCE_TREE_SHA256,
            "r14_visibility_package_sha256": R14_VISIBILITY_PACKAGE_SHA256,
        },
        "inputs": {
            "r11_result_path": result_path.to_string_lossy(),
            "r11_result_sha256_before": before_result_sha256,
            "r11_result_sha256_after": after_result_sha256,
            "cache_preparation_path": preparation_path.to_string_lossy(),
            "cache_preparation_sha256_before": before_preparation_sha256,
            "cache_preparation_sha256_after": after_preparation_sha256,
            "reference_tag_ref": REFERENCE_TAG_REF,
            "reference_tag_object": REFERENCE_TAG_OBJECT,
            "reference_commit": REFERENCE_TAG_COMMIT,
            "reference_lock_sha256": REFERENCE_LOCK_SHA256,
            "test_lock_sha256": TEST_LOCK_SHA256,
        },
        "original_r11_outcome": {
            "product_build": "PASS",
            "selected_test": "FAIL",
            "capture": "NOT_RUN",
            "admission": "NOT_RUN",
            "qualification": "blocked",
            "protected_inputs_unchanged": false,
            "original_outcome_changed": false,
        },
        "scope_boundary": {
            "historical_added_file_digest_binding": "NOT_AVAILABLE_IN_R11_RESULT",
            "statement": "The R11 result records added paths and types but not their content digests. This audit verifies current files at the R11 cache path against lock-pinned archives and verifies paths, file bytes, modes, device IDs and inode identities stayed unchanged during this audit. It does not measure timestamps and cannot prove the files' historical bytes at R11 completion.",
            "timestamps_measured": false,
        },
        "audit_conclusion": "CURRENT_CACHE_CONTENTS_MATCH_LOCKED_ARCHIVES; HISTORICAL_R11_BYTES_NOT_PROVEN",
        "historical_r11_content_status": "NOT_VERIFIED",
        "cache": {
            "cargo_home": CACHE_HOME_PATH,
            "registry_hash": REGISTRY_HASH,
            "unmanifested_path_count": EXPECTED_UNMANIFESTED_PATHS,
            "unmanifested_file_count": EXPECTED_UNMANIFESTED_FILES,
            "unmanifested_directory_count": EXPECTED_UNMANIFESTED_DIRECTORIES,
            "new_package_root_count": package_roots.len(),
            "locked_archive_union_count": archive_rows.len(),
            "registry_source_path_count": actual_nodes.len(),
            "registry_source_file_bytes": registry_source_file_bytes,
            "registry_source_paths_bytes_modes_and_identity_unchanged": true,
        },
        "checks": {
            "bound_r11_and_preparation_receipts": "PASS",
            "immutable_reference_identity": "PASS",
            "original_r11_failure_preserved": "PASS",
            "all_registry_source_paths_reconciled": "PASS",
            "all_7231_unmanifested_paths_covered": "PASS",
            "153_package_roots_match_lock_rows": "PASS",
            "archive_checksums_match_lock_rows": "PASS",
            "archive_members_match_extracted_content_and_types": "PASS",
            "archive_decompression_bounded_per_package": "PASS",
            "bound_receipts_archives_and_source_paths_bytes_modes_and_identity_unchanged": "PASS",
            "archive_members_streamed_without_extraction": true,
            "cache_write_operations_in_test_code": 0,
        },
        "packages": package_reports.iter().map(PackageReport::to_json).collect::<Vec<_>>(),
        "errors": [],
    }))
}

struct PinnedJson {
    value: Value,
    sha256: String,
}

fn read_pinned_json(
    path: &Path,
    expected_sha256: &str,
    max_bytes: u64,
    label: &'static str,
) -> AuditResult<PinnedJson> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| AuditFailure::new(label, format!("{}: {error}", path.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AuditFailure::new(
            label,
            format!("{} is not a regular non-symlink file", path.display()),
        ));
    }
    if metadata.len() > max_bytes {
        return Err(AuditFailure::new(
            label,
            format!(
                "{} exceeds the {}-byte input cap",
                path.display(),
                max_bytes
            ),
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| AuditFailure::new(label, format!("{}: {error}", path.display())))?;
    let sha256 = sha256_bytes(&bytes);
    if sha256 != expected_sha256 {
        return Err(AuditFailure::new(
            label,
            format!(
                "{} has SHA-256 {}, expected {}",
                path.display(),
                sha256,
                expected_sha256
            ),
        ));
    }
    let value = serde_json::from_slice(&bytes).map_err(|error| {
        AuditFailure::new(
            label,
            format!("{} is invalid JSON: {error}", path.display()),
        )
    })?;
    Ok(PinnedJson { value, sha256 })
}

fn validate_bound_receipts(
    result: &Value,
    preparation: &Value,
    result_path: &Path,
    preparation_path: &Path,
) -> AuditResult<()> {
    require_string(
        result,
        "schema",
        "termrock-vis06-tag-holla-r11-nextest-result/v1",
        "r11_identity",
    )?;
    require_string(
        preparation,
        "schema",
        "termrock-vis06-tag-holla-r4-private-cache-preparation-v1",
        "preparation_identity",
    )?;
    require_string(
        preparation,
        "status",
        "PREPARED_NO_CARGO_OR_NEXTEST",
        "preparation_identity",
    )?;
    require_string(
        result,
        "cache_preparation_path",
        &preparation_path.to_string_lossy(),
        "preparation_binding",
    )?;
    require_string(
        result,
        "cache_preparation_sha256",
        CACHE_PREPARATION_SHA256,
        "preparation_binding",
    )?;
    if result_path != Path::new(R11_RESULT_PATH)
        || preparation_path != Path::new(CACHE_PREPARATION_PATH)
    {
        return Err(AuditFailure::new(
            "input_binding",
            "input receipt paths changed",
        ));
    }

    let tag = preparation.pointer("/frozen_inputs").ok_or_else(|| {
        AuditFailure::new("reference_identity", "preparation has no frozen_inputs")
    })?;
    require_string(tag, "tag_ref", REFERENCE_TAG_REF, "reference_identity")?;
    require_string(
        tag,
        "tag_object",
        REFERENCE_TAG_OBJECT,
        "reference_identity",
    )?;
    require_string(
        tag,
        "tag_commit",
        REFERENCE_TAG_COMMIT,
        "reference_identity",
    )?;
    require_string(
        tag,
        "tag_lock_sha256",
        REFERENCE_LOCK_SHA256,
        "reference_identity",
    )?;
    require_string(
        tag,
        "test_lock_sha256",
        TEST_LOCK_SHA256,
        "reference_identity",
    )?;

    let after_tag = result.pointer("/inputs_after/tag").ok_or_else(|| {
        AuditFailure::new(
            "reference_identity",
            "R11 result has no post-run tag record",
        )
    })?;
    require_string(
        after_tag,
        "tag_ref",
        REFERENCE_TAG_REF,
        "reference_identity",
    )?;
    require_string(
        after_tag,
        "tag_object",
        REFERENCE_TAG_OBJECT,
        "reference_identity",
    )?;
    require_string(
        after_tag,
        "peeled_commit",
        REFERENCE_TAG_COMMIT,
        "reference_identity",
    )?;
    require_string(
        after_tag,
        "cargo_lock_content_sha256",
        REFERENCE_LOCK_SHA256,
        "reference_identity",
    )?;

    require_string(
        result.pointer("/product_artifacts").unwrap_or(&Value::Null),
        "build_result",
        "PASS",
        "original_outcome",
    )?;
    require_string(
        result.pointer("/product_artifacts").unwrap_or(&Value::Null),
        "capture_status",
        "NOT_RUN",
        "original_outcome",
    )?;
    require_string(
        result.pointer("/product_artifacts").unwrap_or(&Value::Null),
        "admission_status",
        "NOT_RUN",
        "original_outcome",
    )?;
    require_string(
        result
            .pointer("/product_artifacts/qualification")
            .unwrap_or(&Value::Null),
        "status",
        "blocked",
        "original_outcome",
    )?;
    if result
        .pointer("/protected_inputs_unchanged")
        .and_then(Value::as_bool)
        != Some(false)
    {
        return Err(AuditFailure::new(
            "original_outcome",
            "R11 protected_inputs_unchanged must remain false",
        ));
    }
    let selected = result.pointer("/selected_rows/0").ok_or_else(|| {
        AuditFailure::new("original_outcome", "R11 selected test record is missing")
    })?;
    require_string(selected, "status", "FAIL", "original_outcome")?;
    require_string(
        selected,
        "test",
        "supervises_real_frozen_tag_holla_build",
        "original_outcome",
    )?;
    if selected.get("numerator").and_then(Value::as_u64) != Some(1)
        || selected.get("denominator").and_then(Value::as_u64) != Some(1)
    {
        return Err(AuditFailure::new(
            "original_outcome",
            "R11 selected test denominator/numerator changed",
        ));
    }
    require_string(
        result,
        "conclusion",
        "TAG_BUILD_GATE_INCOMPLETE; preserve raw outcome and postflight evidence",
        "original_outcome",
    )?;

    if preparation
        .pointer("/cache_source/registry_src_copied")
        .and_then(Value::as_bool)
        != Some(false)
    {
        return Err(AuditFailure::new(
            "preparation_identity",
            "R4 preparation no longer records registry_src_copied=false",
        ));
    }
    Ok(())
}

fn validate_cache_home(result: &Value, cache_home: &Path) -> AuditResult<()> {
    let expected_home = result
        .pointer("/inputs_after/cache_inventory/home")
        .ok_or_else(|| AuditFailure::new("cache_home", "R11 result has no cache-home identity"))?;
    require_string(expected_home, "path", CACHE_HOME_PATH, "cache_home")?;
    if expected_home.get("mode").and_then(Value::as_str) != Some("0700") {
        return Err(AuditFailure::new(
            "cache_home",
            "R11 cache-home mode is not 0700",
        ));
    }
    let metadata = fs::symlink_metadata(cache_home).map_err(|error| {
        AuditFailure::new("cache_home", format!("{}: {error}", cache_home.display()))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AuditFailure::new(
            "cache_home",
            "R11 Cargo home is not a real directory",
        ));
    }
    if !cfg!(unix) {
        return Err(AuditFailure::new(
            "cache_home",
            "the bound R11 cache identity is from a Unix execution host",
        ));
    }
    let mode = metadata_mode(&metadata);
    if mode != 0o700 {
        return Err(AuditFailure::new(
            "cache_home",
            format!("Cargo home mode is {:04o}, expected 0700", mode),
        ));
    }
    let dev = metadata_dev(&metadata);
    let ino = metadata_ino(&metadata);
    if expected_home.get("dev").and_then(Value::as_str) != Some(dev.as_str())
        || expected_home.get("ino").and_then(Value::as_str) != Some(ino.as_str())
    {
        return Err(AuditFailure::new(
            "cache_home",
            "Cargo-home device/inode differs from R11",
        ));
    }
    let before_home = result
        .pointer("/inputs_before/cache_inventory/home")
        .ok_or_else(|| {
            AuditFailure::new(
                "cache_home",
                "R11 result has no pre-run cache-home identity",
            )
        })?;
    if before_home != expected_home {
        return Err(AuditFailure::new(
            "cache_home",
            "R11 pre/post cache-home identities differ",
        ));
    }
    for path in [
        cache_home.join("registry"),
        cache_home.join("registry/src"),
        cache_home.join("registry/src").join(REGISTRY_HASH),
        cache_home.join("registry/cache"),
        cache_home.join("registry/cache").join(REGISTRY_HASH),
    ] {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            AuditFailure::new("cache_home", format!("{}: {error}", path.display()))
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AuditFailure::new(
                "cache_home",
                format!(
                    "Cargo cache path is not a real directory: {}",
                    path.display()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_receipt_destination(
    receipt_path: &Path,
    result_path: &Path,
    preparation_path: &Path,
    cache_home: &Path,
) -> AuditResult<()> {
    if !receipt_path.is_absolute() {
        return Err(AuditFailure::new(
            "receipt_path",
            "receipt path must be absolute",
        ));
    }
    let parent = receipt_path
        .parent()
        .ok_or_else(|| AuditFailure::new("receipt_path", "receipt has no parent directory"))?;
    let parent_meta = fs::symlink_metadata(parent).map_err(|error| {
        AuditFailure::new("receipt_path", format!("{}: {error}", parent.display()))
    })?;
    if parent_meta.file_type().is_symlink() || !parent_meta.is_dir() {
        return Err(AuditFailure::new(
            "receipt_path",
            "receipt parent must be an existing non-symlink directory",
        ));
    }
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?;
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let executable_root = env::current_exe()
        .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AuditFailure::new("receipt_path", "test executable has no parent"))?;
    for protected in [
        fs::canonicalize(source_root)
            .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?,
        fs::canonicalize(cache_home)
            .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?,
        fs::canonicalize(result_path)
            .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?,
        fs::canonicalize(preparation_path)
            .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?,
        fs::canonicalize(executable_root)
            .map_err(|error| AuditFailure::new("receipt_path", error.to_string()))?,
    ] {
        if path_overlap(&canonical_parent, &protected) {
            return Err(AuditFailure::new(
                "receipt_path",
                format!(
                    "receipt parent overlaps protected input/output root {}",
                    protected.display()
                ),
            ));
        }
    }
    match fs::symlink_metadata(receipt_path) {
        Ok(_) => Err(AuditFailure::new(
            "receipt_path",
            format!(
                "receipt already exists; use a fresh path: {}",
                receipt_path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AuditFailure::new("receipt_path", error.to_string())),
    }
}

fn path_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn expected_source_nodes(result: &Value) -> AuditResult<BTreeMap<String, ExpectedNode>> {
    let inventory = result
        .pointer("/inputs_after/cache_inventory")
        .ok_or_else(|| AuditFailure::new("inventory", "R11 post-run cache inventory is missing"))?;
    let before_inventory = result
        .pointer("/inputs_before/cache_inventory")
        .ok_or_else(|| AuditFailure::new("inventory", "R11 pre-run cache inventory is missing"))?;
    let after_warm = inventory
        .pointer("/generated_outputs")
        .ok_or_else(|| AuditFailure::new("inventory", "R11 warm generated_outputs is missing"))?;
    let before_warm = before_inventory
        .pointer("/generated_outputs")
        .ok_or_else(|| {
            AuditFailure::new("inventory", "R11 pre-run generated_outputs is missing")
        })?;
    if after_warm != before_warm {
        return Err(AuditFailure::new(
            "inventory",
            "R11 warm source inventory changed between preflight and postflight",
        ));
    }

    let warm_files = json_array(after_warm, "files", "inventory")?;
    let warm_directories = json_array(after_warm, "directories", "inventory")?;
    if warm_files.len() != EXPECTED_WARM_FILES
        || warm_directories.len() != EXPECTED_WARM_DIRECTORIES
    {
        return Err(AuditFailure::new(
            "inventory",
            format!(
                "R11 warm inventory has {} files/{} directories, expected {}/{}",
                warm_files.len(),
                warm_directories.len(),
                EXPECTED_WARM_FILES,
                EXPECTED_WARM_DIRECTORIES
            ),
        ));
    }
    if after_warm.get("file_count").and_then(Value::as_u64) != Some(EXPECTED_WARM_FILES as u64)
        || after_warm.get("directory_count").and_then(Value::as_u64)
            != Some(EXPECTED_WARM_DIRECTORIES as u64)
    {
        return Err(AuditFailure::new(
            "inventory",
            "R11 warm inventory declared counts changed",
        ));
    }

    let mut expected = BTreeMap::new();
    for row in warm_files {
        let relative = field_string(row, "relative_path", "inventory")?;
        validate_source_relative_path(&relative, "inventory")?;
        if !relative.starts_with("registry/src/") {
            return Err(AuditFailure::new(
                "inventory",
                format!("warm file is outside registry/src: {relative}"),
            ));
        }
        let node = ExpectedNode {
            kind: NodeKind::File,
            bytes: Some(field_u64(row, "bytes", "inventory")?),
            sha256: Some(field_string(row, "sha256", "inventory")?),
            mode: Some(field_u64(row, "mode", "inventory")?),
            dev: Some(field_string(row, "dev", "inventory")?),
            ino: Some(field_string(row, "ino", "inventory")?),
        };
        insert_expected(&mut expected, relative, node, "inventory")?;
    }
    for row in warm_directories {
        let relative = field_string(row, "relative_path", "inventory")?;
        validate_source_relative_path(&relative, "inventory")?;
        if !relative.starts_with("registry/src") {
            return Err(AuditFailure::new(
                "inventory",
                format!("warm directory is outside registry/src: {relative}"),
            ));
        }
        let node = ExpectedNode {
            kind: NodeKind::Directory,
            bytes: None,
            sha256: None,
            mode: Some(field_u64(row, "mode", "inventory")?),
            dev: Some(field_string(row, "dev", "inventory")?),
            ino: Some(field_string(row, "ino", "inventory")?),
        };
        insert_expected(&mut expected, relative, node, "inventory")?;
    }

    let error_rows = json_array(inventory, "errors", "unmanifested_paths")?;
    if error_rows.len() != EXPECTED_UNMANIFESTED_PATHS {
        return Err(AuditFailure::new(
            "unmanifested_paths",
            format!(
                "R11 records {} unmanifested paths, expected {}",
                error_rows.len(),
                EXPECTED_UNMANIFESTED_PATHS
            ),
        ));
    }
    let mut error_file_count = 0usize;
    let mut error_directory_count = 0usize;
    let mut error_paths = BTreeSet::new();
    for row in error_rows {
        if field_string(row, "stage", "unmanifested_paths")? != "private_cache_tree" {
            return Err(AuditFailure::new(
                "unmanifested_paths",
                "an unmanifested path has an unexpected stage",
            ));
        }
        let relative = field_string(row, "path", "unmanifested_paths")?;
        validate_source_relative_path(&relative, "unmanifested_paths")?;
        if !relative.starts_with("registry/src/") {
            return Err(AuditFailure::new(
                "unmanifested_paths",
                format!("unmanifested path is outside registry/src: {relative}"),
            ));
        }
        let error = field_string(row, "error", "unmanifested_paths")?;
        let kind = match error.as_str() {
            "unmanifested Cargo-home file" => {
                error_file_count += 1;
                NodeKind::File
            }
            "unmanifested Cargo-home directory" => {
                error_directory_count += 1;
                NodeKind::Directory
            }
            _ => {
                return Err(AuditFailure::new(
                    "unmanifested_paths",
                    format!("unexpected R11 unmanifested-path error for {relative}: {error}"),
                ));
            }
        };
        if !error_paths.insert(relative.clone()) {
            return Err(AuditFailure::new(
                "unmanifested_paths",
                format!("duplicate R11 error path: {relative}"),
            ));
        }
        insert_expected(
            &mut expected,
            relative,
            ExpectedNode {
                kind,
                bytes: None,
                sha256: None,
                mode: None,
                dev: None,
                ino: None,
            },
            "unmanifested_paths",
        )?;
    }
    if error_file_count != EXPECTED_UNMANIFESTED_FILES
        || error_directory_count != EXPECTED_UNMANIFESTED_DIRECTORIES
    {
        return Err(AuditFailure::new(
            "unmanifested_paths",
            format!(
                "R11 path errors contain {error_file_count} files/{error_directory_count} directories, expected {EXPECTED_UNMANIFESTED_FILES}/{EXPECTED_UNMANIFESTED_DIRECTORIES}"
            ),
        ));
    }
    Ok(expected)
}

fn insert_expected(
    expected: &mut BTreeMap<String, ExpectedNode>,
    path: String,
    node: ExpectedNode,
    stage: &'static str,
) -> AuditResult<()> {
    if expected.insert(path.clone(), node).is_some() {
        return Err(AuditFailure::new(
            stage,
            format!("duplicate inventory path: {path}"),
        ));
    }
    Ok(())
}

fn validate_source_inventory(
    expected: &BTreeMap<String, ExpectedNode>,
    actual: &BTreeMap<String, ObservedNode>,
) -> AuditResult<()> {
    if expected.len() != actual.len() {
        return Err(AuditFailure::new(
            "source_inventory",
            format!(
                "expected {} registry source paths but found {}",
                expected.len(),
                actual.len()
            ),
        ));
    }
    for (relative, expected_node) in expected {
        let actual_node = actual.get(relative).ok_or_else(|| {
            AuditFailure::new(
                "source_inventory",
                format!("recorded path is missing: {relative}"),
            )
        })?;
        if actual_node.kind != expected_node.kind {
            return Err(AuditFailure::new(
                "source_inventory",
                format!("file/directory type differs for {relative}"),
            ));
        }
        if let Some(bytes) = expected_node.bytes {
            if actual_node.bytes != bytes {
                return Err(AuditFailure::new(
                    "source_inventory",
                    format!("size differs for {relative}"),
                ));
            }
        }
        if let Some(sha256) = &expected_node.sha256 {
            if actual_node.sha256.as_ref() != Some(sha256) {
                return Err(AuditFailure::new(
                    "source_inventory",
                    format!("R11 warm-file digest differs for {relative}"),
                ));
            }
        }
        if let Some(mode) = expected_node.mode {
            if actual_node.mode != mode {
                return Err(AuditFailure::new(
                    "source_inventory",
                    format!("mode differs for {relative}"),
                ));
            }
        }
        if let Some(dev) = &expected_node.dev {
            if &actual_node.dev != dev {
                return Err(AuditFailure::new(
                    "source_inventory",
                    format!("device differs for {relative}"),
                ));
            }
        }
        if let Some(ino) = &expected_node.ino {
            if &actual_node.ino != ino {
                return Err(AuditFailure::new(
                    "source_inventory",
                    format!("inode differs for {relative}"),
                ));
            }
        }
    }
    if actual.keys().any(|path| !expected.contains_key(path)) {
        return Err(AuditFailure::new(
            "source_inventory",
            "registry/src contains an unrecorded path",
        ));
    }
    Ok(())
}

fn walk_registry_source_tree(cache_home: &Path) -> AuditResult<BTreeMap<String, ObservedNode>> {
    let root = cache_home.join("registry/src");
    let metadata = fs::symlink_metadata(&root).map_err(|error| {
        AuditFailure::new("source_walk", format!("{}: {error}", root.display()))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AuditFailure::new(
            "source_walk",
            "registry/src is not a real directory",
        ));
    }
    let mut nodes = BTreeMap::new();
    let mut total_file_bytes = 0u64;
    walk_directory(&root, "registry/src", 0, &mut nodes, &mut total_file_bytes)?;
    Ok(nodes)
}

fn walk_directory(
    absolute: &Path,
    relative: &str,
    depth: usize,
    nodes: &mut BTreeMap<String, ObservedNode>,
    total_file_bytes: &mut u64,
) -> AuditResult<()> {
    if depth > MAX_SOURCE_TREE_DEPTH {
        return Err(AuditFailure::new(
            "source_walk",
            format!("registry source tree exceeds depth {MAX_SOURCE_TREE_DEPTH}"),
        ));
    }
    let metadata = fs::symlink_metadata(absolute).map_err(|error| {
        AuditFailure::new("source_walk", format!("{}: {error}", absolute.display()))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AuditFailure::new(
            "source_walk",
            format!("not a regular directory: {relative}"),
        ));
    }
    if nodes.len() >= MAX_SOURCE_TREE_NODES {
        return Err(AuditFailure::new(
            "source_walk",
            format!("registry source tree exceeds {MAX_SOURCE_TREE_NODES} nodes"),
        ));
    }
    let node = directory_node(&metadata);
    if nodes.insert(relative.to_owned(), node).is_some() {
        return Err(AuditFailure::new(
            "source_walk",
            format!("duplicate directory path: {relative}"),
        ));
    }

    let children = fs::read_dir(absolute).map_err(|error| {
        AuditFailure::new("source_walk", format!("{}: {error}", absolute.display()))
    })?;
    let mut child_count = 0usize;
    for child in children {
        child_count = child_count
            .checked_add(1)
            .ok_or_else(|| AuditFailure::new("source_walk", "directory child count overflow"))?;
        if child_count > MAX_SOURCE_TREE_NODES {
            return Err(AuditFailure::new(
                "source_walk",
                format!("directory {relative} exceeds the child-count cap"),
            ));
        }
        let child = child.map_err(|error| AuditFailure::new("source_walk", error.to_string()))?;
        let file_name = child.file_name();
        let name = file_name.to_str().ok_or_else(|| {
            AuditFailure::new(
                "source_walk",
                format!("non-UTF-8 Cargo source path under {relative}"),
            )
        })?;
        validate_component(name, "source_walk")?;
        let child_relative = format!("{relative}/{name}");
        let child_absolute = child.path();
        let child_metadata = fs::symlink_metadata(&child_absolute).map_err(|error| {
            AuditFailure::new(
                "source_walk",
                format!("{}: {error}", child_absolute.display()),
            )
        })?;
        let file_type = child_metadata.file_type();
        if file_type.is_symlink() {
            return Err(AuditFailure::new(
                "source_walk",
                format!("symbolic link found: {child_relative}"),
            ));
        }
        if file_type.is_dir() {
            walk_directory(
                &child_absolute,
                &child_relative,
                depth + 1,
                nodes,
                total_file_bytes,
            )?;
        } else if file_type.is_file() {
            if metadata_link_count(&child_metadata) != 1 {
                return Err(AuditFailure::new(
                    "source_walk",
                    format!("hard-linked file found: {child_relative}"),
                ));
            }
            if nodes.len() >= MAX_SOURCE_TREE_NODES {
                return Err(AuditFailure::new(
                    "source_walk",
                    format!("registry source tree exceeds {MAX_SOURCE_TREE_NODES} nodes"),
                ));
            }
            if child_metadata.len() > MAX_SOURCE_FILE_BYTES {
                return Err(AuditFailure::new(
                    "source_walk",
                    format!("source file exceeds the per-file cap: {child_relative}"),
                ));
            }
            *total_file_bytes = total_file_bytes
                .checked_add(child_metadata.len())
                .ok_or_else(|| {
                    AuditFailure::new("source_walk", "registry source byte sum overflow")
                })?;
            if *total_file_bytes > MAX_SOURCE_TREE_BYTES {
                return Err(AuditFailure::new(
                    "source_walk",
                    "registry source tree exceeds the 512 MiB read cap",
                ));
            }
            let (bytes, sha256) = hash_file(&child_absolute)
                .map_err(|error| AuditFailure::new("source_walk", error))?;
            if bytes != child_metadata.len() {
                return Err(AuditFailure::new(
                    "source_walk",
                    format!("file changed while hashing: {child_relative}"),
                ));
            }
            let node = ObservedNode {
                kind: NodeKind::File,
                bytes,
                sha256: Some(sha256),
                mode: metadata_mode(&child_metadata),
                dev: metadata_dev(&child_metadata),
                ino: metadata_ino(&child_metadata),
            };
            if nodes.insert(child_relative.clone(), node).is_some() {
                return Err(AuditFailure::new(
                    "source_walk",
                    format!("duplicate file path: {child_relative}"),
                ));
            }
        } else {
            return Err(AuditFailure::new(
                "source_walk",
                format!("special filesystem entry found: {child_relative}"),
            ));
        }
    }
    Ok(())
}

fn directory_node(metadata: &Metadata) -> ObservedNode {
    ObservedNode {
        kind: NodeKind::Directory,
        bytes: 0,
        sha256: None,
        mode: metadata_mode(metadata),
        dev: metadata_dev(metadata),
        ino: metadata_ino(metadata),
    }
}

#[cfg(unix)]
fn metadata_mode(metadata: &Metadata) -> u64 {
    (metadata.permissions().mode() & 0o777) as u64
}

#[cfg(not(unix))]
fn metadata_mode(_metadata: &Metadata) -> u64 {
    0
}

#[cfg(unix)]
fn metadata_dev(metadata: &Metadata) -> String {
    metadata.dev().to_string()
}

#[cfg(not(unix))]
fn metadata_dev(_metadata: &Metadata) -> String {
    String::new()
}

#[cfg(unix)]
fn metadata_ino(metadata: &Metadata) -> String {
    metadata.ino().to_string()
}

#[cfg(not(unix))]
fn metadata_ino(_metadata: &Metadata) -> String {
    String::new()
}

#[cfg(unix)]
fn metadata_link_count(metadata: &Metadata) -> u64 {
    metadata.nlink()
}

#[cfg(windows)]
fn metadata_link_count(metadata: &Metadata) -> u64 {
    metadata.number_of_links() as u64
}

#[cfg(not(any(unix, windows)))]
fn metadata_link_count(_metadata: &Metadata) -> u64 {
    1
}

fn archive_rows(preparation: &Value) -> AuditResult<BTreeMap<String, ArchiveRow>> {
    let closure = preparation
        .pointer("/registry_closure")
        .ok_or_else(|| AuditFailure::new("archive_rows", "preparation has no registry closure"))?;
    require_string(closure, "registry_hash", REGISTRY_HASH, "archive_rows")?;
    if closure
        .get("unique_lock_union_package_versions")
        .and_then(Value::as_u64)
        != Some(EXPECTED_LOCKED_ARCHIVES as u64)
    {
        return Err(AuditFailure::new(
            "archive_rows",
            "locked archive union count changed",
        ));
    }
    let rows = json_array(closure, "archives", "archive_rows")?;
    if rows.len() != EXPECTED_LOCKED_ARCHIVES {
        return Err(AuditFailure::new(
            "archive_rows",
            format!("found {} archive rows", rows.len()),
        ));
    }
    let copies = json_array(preparation, "copies", "archive_rows")?;
    let mut copy_by_destination = BTreeMap::new();
    for copy in copies {
        if copy.get("role").and_then(Value::as_str) != Some("locked-registry-archive") {
            continue;
        }
        let destination = field_string(copy, "destination", "archive_rows")?;
        if copy_by_destination
            .insert(destination.clone(), copy)
            .is_some()
        {
            return Err(AuditFailure::new(
                "archive_rows",
                format!("duplicate archive copy destination: {destination}"),
            ));
        }
    }
    if copy_by_destination.len() != EXPECTED_LOCKED_ARCHIVES {
        return Err(AuditFailure::new(
            "archive_rows",
            format!(
                "preparation records {} archive copies",
                copy_by_destination.len()
            ),
        ));
    }

    let mut archive_rows = BTreeMap::new();
    for row in rows {
        let name = field_string(row, "name", "archive_rows")?;
        let version = field_string(row, "version", "archive_rows")?;
        let checksum = field_string(row, "checksum", "archive_rows")?;
        let bytes = field_u64(row, "bytes", "archive_rows")?;
        if !valid_sha256(&checksum) || bytes == 0 {
            return Err(AuditFailure::new(
                "archive_rows",
                format!("invalid lock metadata for {name}-{version}"),
            ));
        }
        validate_package_component(&name, "archive_rows")?;
        validate_package_component(&version, "archive_rows")?;
        let archive_name = format!("{name}-{version}.crate");
        let archive_path =
            format!("{CACHE_HOME_PATH}/registry/cache/{REGISTRY_HASH}/{archive_name}");
        let copy = copy_by_destination.get(&archive_path).ok_or_else(|| {
            AuditFailure::new(
                "archive_rows",
                format!("preparation has no copied archive row for {archive_path}"),
            )
        })?;
        if field_string(copy, "sha256", "archive_rows")? != checksum
            || field_string(copy, "destination_sha256", "archive_rows")? != checksum
            || field_u64(copy, "bytes", "archive_rows")? != bytes
            || field_u64(copy, "destination_bytes", "archive_rows")? != bytes
        {
            return Err(AuditFailure::new(
                "archive_rows",
                format!("preparation copy metadata differs for {archive_name}"),
            ));
        }
        let row = ArchiveRow {
            name: name.clone(),
            version: version.clone(),
            checksum,
            bytes,
        };
        if archive_rows
            .insert(format!("{name}-{version}"), row)
            .is_some()
        {
            return Err(AuditFailure::new(
                "archive_rows",
                format!("duplicate locked package row: {name}-{version}"),
            ));
        }
    }
    Ok(archive_rows)
}

fn package_roots_from_errors(
    result: &Value,
    archive_rows: &BTreeMap<String, ArchiveRow>,
) -> AuditResult<Vec<PackageRoot>> {
    let errors = json_array(
        result
            .pointer("/inputs_after/cache_inventory")
            .ok_or_else(|| {
                AuditFailure::new("package_roots", "post-run cache inventory is missing")
            })?,
        "errors",
        "package_roots",
    )?;
    let mut roots = BTreeSet::new();
    for error in errors {
        let relative = field_string(error, "path", "package_roots")?;
        let parts = validate_source_relative_path(&relative, "package_roots")?;
        if parts.len() < 4
            || parts[0] != "registry"
            || parts[1] != "src"
            || parts[2] != REGISTRY_HASH
        {
            return Err(AuditFailure::new(
                "package_roots",
                format!("unmanifested source path is outside the expected registry: {relative}"),
            ));
        }
        let root = parts[..4].join("/");
        if parts.len() == 4 {
            if field_string(error, "error", "package_roots")? != "unmanifested Cargo-home directory"
            {
                return Err(AuditFailure::new(
                    "package_roots",
                    format!("package root is not recorded as a directory: {root}"),
                ));
            }
            roots.insert(root);
        }
    }
    if roots.len() != EXPECTED_NEW_PACKAGE_ROOTS {
        return Err(AuditFailure::new(
            "package_roots",
            format!(
                "found {} new package roots, expected {}",
                roots.len(),
                EXPECTED_NEW_PACKAGE_ROOTS
            ),
        ));
    }
    let mut packages = Vec::with_capacity(roots.len());
    for root in roots {
        let suffix = root
            .strip_prefix(&format!("registry/src/{REGISTRY_HASH}/"))
            .ok_or_else(|| {
                AuditFailure::new("package_roots", format!("invalid package root: {root}"))
            })?;
        let row = archive_rows.get(suffix).cloned().ok_or_else(|| {
            AuditFailure::new(
                "package_roots",
                format!("new source root has no matching lock row: {suffix}"),
            )
        })?;
        packages.push(PackageRoot {
            relative_path: root,
            row,
        });
    }
    Ok(packages)
}

fn audit_package_archive(
    cache_home: &Path,
    package: &PackageRoot,
    actual_nodes: &BTreeMap<String, ObservedNode>,
) -> AuditResult<PackageReport> {
    let prefix = format!("{}/", package.relative_path);
    let mut source_files = BTreeMap::new();
    let mut source_directories = BTreeSet::new();
    let root_node = actual_nodes.get(&package.relative_path).ok_or_else(|| {
        AuditFailure::new(
            "package_source",
            format!("package root is missing: {}", package.relative_path),
        )
    })?;
    if root_node.kind != NodeKind::Directory {
        return Err(AuditFailure::new(
            "package_source",
            format!("package root is not a directory: {}", package.relative_path),
        ));
    }
    source_directories.insert(String::new());
    for (path, node) in actual_nodes {
        let Some(relative) = path.strip_prefix(&prefix) else {
            continue;
        };
        if relative.is_empty() {
            continue;
        }
        if node.kind == NodeKind::Directory {
            source_directories.insert(relative.to_owned());
        } else {
            source_files.insert(relative.to_owned(), node.clone());
        }
    }
    let marker = source_files.remove(".cargo-ok").ok_or_else(|| {
        AuditFailure::new(
            "package_source",
            format!(
                "Cargo .cargo-ok marker missing from {}",
                package.relative_path
            ),
        )
    })?;
    let marker_sha256 = hash_file(&cache_home.join(&package.relative_path).join(".cargo-ok"))
        .map_err(|message| AuditFailure::new("package_source", message))?
        .1;
    if marker.bytes != 7
        || marker_sha256 != "afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91"
    {
        return Err(AuditFailure::new(
            "package_source",
            format!(
                "Cargo .cargo-ok marker is not the exact seven-byte {{\"v\":1}} value in {}",
                package.relative_path
            ),
        ));
    }

    let archived_payload_bytes = source_files.values().try_fold(0u64, |total, node| {
        total
            .checked_add(node.bytes)
            .ok_or_else(|| AuditFailure::new("decompression_cap", "package byte sum overflow"))
    })?;
    let observed_node_count = source_files
        .len()
        .checked_add(source_directories.len())
        .ok_or_else(|| AuditFailure::new("decompression_cap", "package node count overflow"))?;
    let cap_nodes = observed_node_count
        .checked_add(EXTRA_TAR_NODE_BUDGET as usize)
        .ok_or_else(|| AuditFailure::new("decompression_cap", "package node cap overflow"))?;
    let overhead = (cap_nodes as u64)
        .checked_mul(TAR_OVERHEAD_BUDGET_PER_OBSERVED_NODE)
        .and_then(|value| value.checked_add(TAR_END_MARKER_BUDGET))
        .ok_or_else(|| {
            AuditFailure::new("decompression_cap", "package TAR overhead cap overflow")
        })?;
    let decompressed_cap_bytes = archived_payload_bytes
        .checked_add(overhead)
        .ok_or_else(|| {
            AuditFailure::new("decompression_cap", "package decompression cap overflow")
        })?;
    let max_entry_count = observed_node_count
        .checked_mul(3)
        .and_then(|value| value.checked_add(64))
        .ok_or_else(|| {
            AuditFailure::new("decompression_cap", "package entry count cap overflow")
        })?;

    let archive_path = cache_home
        .join("registry/cache")
        .join(REGISTRY_HASH)
        .join(format!(
            "{}-{}.crate",
            package.row.name, package.row.version
        ));
    let archive_metadata = fs::symlink_metadata(&archive_path).map_err(|error| {
        AuditFailure::new(
            "locked_archive",
            format!("{}: {error}", archive_path.display()),
        )
    })?;
    if archive_metadata.file_type().is_symlink()
        || !archive_metadata.is_file()
        || metadata_link_count(&archive_metadata) != 1
    {
        return Err(AuditFailure::new(
            "locked_archive",
            format!(
                "archive is not a regular, unlinked file: {}",
                archive_path.display()
            ),
        ));
    }
    if archive_metadata.len() != package.row.bytes {
        return Err(AuditFailure::new(
            "locked_archive",
            format!(
                "archive size differs from lock metadata: {}",
                archive_path.display()
            ),
        ));
    }
    let (archive_bytes, archive_sha256) =
        hash_file(&archive_path).map_err(|error| AuditFailure::new("locked_archive", error))?;
    if archive_bytes != package.row.bytes || archive_sha256 != package.row.checksum {
        return Err(AuditFailure::new(
            "locked_archive",
            format!(
                "archive checksum differs from the lock row: {}",
                archive_path.display()
            ),
        ));
    }
    let hashed_metadata = fs::symlink_metadata(&archive_path).map_err(|error| {
        AuditFailure::new(
            "locked_archive",
            format!("{}: {error}", archive_path.display()),
        )
    })?;
    if hashed_metadata.file_type().is_symlink()
        || !hashed_metadata.is_file()
        || metadata_link_count(&hashed_metadata) != 1
        || hashed_metadata.len() != archive_metadata.len()
        || metadata_mode(&hashed_metadata) != metadata_mode(&archive_metadata)
        || metadata_dev(&hashed_metadata) != metadata_dev(&archive_metadata)
        || metadata_ino(&hashed_metadata) != metadata_ino(&archive_metadata)
    {
        return Err(AuditFailure::new(
            "locked_archive",
            format!(
                "archive path or identity changed while hashing: {}",
                archive_path.display()
            ),
        ));
    }

    let (archive_entry_count, decompressed_bytes) = compare_archive_members(
        &archive_path,
        &package.row,
        &source_files,
        &source_directories,
        decompressed_cap_bytes,
        max_entry_count,
    )?;

    Ok(PackageReport {
        name: package.row.name.clone(),
        version: package.row.version.clone(),
        root: package.relative_path.clone(),
        archive_path: archive_path.to_string_lossy().into_owned(),
        archive_mode: metadata_mode(&hashed_metadata),
        archive_dev: metadata_dev(&hashed_metadata),
        archive_ino: metadata_ino(&hashed_metadata),
        archive_sha256,
        archive_bytes,
        source_file_count: source_files.len() + 1,
        source_directory_count: source_directories.len(),
        archived_payload_bytes,
        decompressed_cap_bytes,
        decompressed_bytes,
        archive_entry_count,
    })
}

fn compare_archive_members(
    archive_path: &Path,
    row: &ArchiveRow,
    source_files: &BTreeMap<String, ObservedNode>,
    source_directories: &BTreeSet<String>,
    decompressed_cap_bytes: u64,
    max_entry_count: usize,
) -> AuditResult<(usize, u64)> {
    let expected_root = format!("{}-{}", row.name, row.version);
    let file = File::open(archive_path).map_err(|error| {
        AuditFailure::new(
            "archive_open",
            format!("{}: {error}", archive_path.display()),
        )
    })?;
    let decoder = MultiGzDecoder::new(file);
    let mut archive = Archive::new(BoundedReader::new(decoder, decompressed_cap_bytes));
    let mut archive_files = BTreeSet::new();
    let mut archive_directories = BTreeSet::new();
    let mut seen_paths = BTreeSet::new();
    let mut entry_count = 0usize;

    {
        let entries = archive.entries().map_err(|error| {
            AuditFailure::new("tar_read", format!("{}: {error}", archive_path.display()))
        })?;
        for entry in entries {
            let mut entry = entry.map_err(|error| {
                AuditFailure::new("tar_read", format!("{}: {error}", archive_path.display()))
            })?;
            entry_count = entry_count
                .checked_add(1)
                .ok_or_else(|| AuditFailure::new("tar_read", "TAR member count overflow"))?;
            if entry_count > max_entry_count {
                return Err(AuditFailure::new(
                    "decompression_cap",
                    format!(
                        "{} exceeds its {max_entry_count}-member TAR cap",
                        archive_path.display()
                    ),
                ));
            }
            let entry_type = entry.header().entry_type();
            if matches!(
                entry_type,
                EntryType::GNULongName
                    | EntryType::GNULongLink
                    | EntryType::XHeader
                    | EntryType::XGlobalHeader
            ) {
                io::copy(&mut entry, &mut io::sink()).map_err(|error| {
                    AuditFailure::new(
                        "tar_metadata",
                        format!("{}: {error}", archive_path.display()),
                    )
                })?;
                continue;
            }
            let path = entry
                .path()
                .map_err(|error| {
                    AuditFailure::new("tar_path", format!("{}: {error}", archive_path.display()))
                })?
                .into_owned();
            let relative = archive_relative_path(&path, &expected_root)?;
            if !seen_paths.insert(relative.clone()) {
                return Err(AuditFailure::new(
                    "tar_path",
                    format!(
                        "duplicate TAR member path {} in {}",
                        path.display(),
                        archive_path.display()
                    ),
                ));
            }
            add_parent_directories(&relative, &mut archive_directories);

            match entry_type {
                EntryType::Regular => {
                    if relative.is_empty() || relative == ".cargo-ok" {
                        return Err(AuditFailure::new(
                            "tar_type",
                            format!(
                                "unexpected regular member {} in {}",
                                path.display(),
                                archive_path.display()
                            ),
                        ));
                    }
                    let expected = source_files.get(&relative).ok_or_else(|| {
                        AuditFailure::new(
                            "tar_extra",
                            format!(
                                "archive has an extra file {relative} in {}",
                                archive_path.display()
                            ),
                        )
                    })?;
                    if entry.size() != expected.bytes {
                        return Err(AuditFailure::new(
                            "tar_size",
                            format!(
                                "TAR size differs for {relative} in {}",
                                archive_path.display()
                            ),
                        ));
                    }
                    let (bytes, sha256) = hash_reader(&mut entry).map_err(|error| {
                        AuditFailure::new("tar_content", format!("{relative}: {error}"))
                    })?;
                    if bytes != expected.bytes
                        || expected.sha256.as_deref() != Some(sha256.as_str())
                    {
                        return Err(AuditFailure::new(
                            "tar_content",
                            format!("archive bytes differ from the observed R11 file {relative}"),
                        ));
                    }
                    archive_files.insert(relative);
                }
                EntryType::Directory => {
                    if entry.size() != 0 {
                        return Err(AuditFailure::new(
                            "tar_type",
                            format!("directory TAR member has data: {}", path.display()),
                        ));
                    }
                    if !relative.is_empty() && !source_directories.contains(&relative) {
                        return Err(AuditFailure::new(
                            "tar_extra",
                            format!(
                                "archive has an extra directory {relative} in {}",
                                archive_path.display()
                            ),
                        ));
                    }
                    archive_directories.insert(relative);
                }
                other => {
                    return Err(AuditFailure::new(
                        "tar_type",
                        format!(
                            "unsupported TAR entry type 0x{:02x} at {}",
                            other.as_byte(),
                            path.display()
                        ),
                    ));
                }
            }
        }
    }

    let mut bounded_decoder = archive.into_inner();
    io::copy(&mut bounded_decoder, &mut io::sink()).map_err(|error| {
        AuditFailure::new(
            "decompression_cap",
            format!("{}: {error}", archive_path.display()),
        )
    })?;
    let decompressed_bytes = bounded_decoder.bytes_read();
    if archive_files != source_files.keys().cloned().collect::<BTreeSet<_>>() {
        return Err(AuditFailure::new(
            "tar_missing",
            format!(
                "archive file set differs from the R11 source tree for {}",
                archive_path.display()
            ),
        ));
    }
    if &archive_directories != source_directories {
        return Err(AuditFailure::new(
            "tar_directories",
            format!(
                "archive directory set differs from the R11 source tree for {}",
                archive_path.display()
            ),
        ));
    }
    Ok((entry_count, decompressed_bytes))
}

fn archive_relative_path(path: &Path, expected_root: &str) -> AuditResult<String> {
    if path.is_absolute() {
        return Err(AuditFailure::new(
            "tar_path",
            format!("absolute TAR path: {}", path.display()),
        ));
    }
    let mut components = path.components();
    match components.next() {
        Some(Component::Normal(root)) if root == expected_root => {}
        _ => {
            return Err(AuditFailure::new(
                "tar_path",
                format!("TAR member has the wrong package root: {}", path.display()),
            ));
        }
    }
    let mut relative = Vec::new();
    for component in components {
        match component {
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(|| {
                    AuditFailure::new(
                        "tar_path",
                        format!("non-UTF-8 TAR path: {}", path.display()),
                    )
                })?;
                validate_component(name, "tar_path")?;
                relative.push(name.to_owned());
            }
            _ => {
                return Err(AuditFailure::new(
                    "tar_path",
                    format!("unsafe TAR path component in {}", path.display()),
                ));
            }
        }
    }
    Ok(relative.join("/"))
}

fn add_parent_directories(relative: &str, directories: &mut BTreeSet<String>) {
    directories.insert(String::new());
    let mut parent = String::new();
    let parts: Vec<&str> = relative.split('/').collect();
    for component in parts.iter().take(parts.len().saturating_sub(1)) {
        if parent.is_empty() {
            parent.push_str(component);
        } else {
            parent.push('/');
            parent.push_str(component);
        }
        directories.insert(parent.clone());
    }
}

struct BoundedReader<R> {
    inner: R,
    cap: u64,
    bytes_read: u64,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, cap: u64) -> Self {
        Self {
            inner,
            cap,
            bytes_read: 0,
        }
    }

    fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.bytes_read == self.cap {
            let mut probe = [0u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "TAR decompression cap exceeded",
                )),
            };
        }
        let remaining = self.cap - self.bytes_read;
        let remaining = usize::try_from(remaining).unwrap_or(usize::MAX);
        let allowed = buffer.len().min(remaining);
        let read = self.inner.read(&mut buffer[..allowed])?;
        self.bytes_read = self.bytes_read.checked_add(read as u64).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "decompressed byte counter overflow",
            )
        })?;
        Ok(read)
    }
}

impl PackageReport {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name.as_str(),
            "version": self.version.as_str(),
            "root": self.root.as_str(),
            "archive_path": self.archive_path.as_str(),
            "archive_mode": self.archive_mode,
            "archive_dev": self.archive_dev.as_str(),
            "archive_ino": self.archive_ino.as_str(),
            "archive_sha256": self.archive_sha256.as_str(),
            "archive_bytes": self.archive_bytes,
            "source_file_count_including_cargo_marker": self.source_file_count,
            "source_directory_count_including_root": self.source_directory_count,
            "archive_payload_bytes": self.archived_payload_bytes,
            "decompressed_cap_bytes": self.decompressed_cap_bytes,
            "decompressed_bytes": self.decompressed_bytes,
            "archive_entry_count": self.archive_entry_count,
            "status": "PASS",
        })
    }
}

fn required_absolute_env_path(name: &str) -> Result<PathBuf, String> {
    let value = env::var_os(name)
        .ok_or_else(|| format!("required environment variable {name} is missing"))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!("{name} must be an absolute path"));
    }
    Ok(path)
}

fn write_new_receipt(path: &Path, report: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(report).map_err(io::Error::other)?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

fn hash_file(path: &Path) -> Result<(u64, String), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata_link_count(&metadata) != 1
    {
        return Err(format!("{} is not a regular unlinked file", path.display()));
    }
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    hash_reader(file).map_err(|error| format!("{}: {error}", path.display()))
}

fn hash_reader(mut reader: impl Read) -> io::Result<(u64, String)> {
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(read as u64)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "byte counter overflow"))?;
        hasher.update(&buffer[..read]);
    }
    Ok((bytes, format!("{:x}", hasher.finalize())))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn require_string(
    value: &Value,
    key: &str,
    expected: &str,
    stage: &'static str,
) -> AuditResult<()> {
    if field_string(value, key, stage)? != expected {
        return Err(AuditFailure::new(
            stage,
            format!("{key} differs from its accepted value"),
        ));
    }
    Ok(())
}

fn field_string(value: &Value, key: &str, stage: &'static str) -> AuditResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| AuditFailure::new(stage, format!("missing string field {key}")))
}

fn field_u64(value: &Value, key: &str, stage: &'static str) -> AuditResult<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| AuditFailure::new(stage, format!("missing unsigned integer field {key}")))
}

fn json_array<'a>(value: &'a Value, key: &str, stage: &'static str) -> AuditResult<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| AuditFailure::new(stage, format!("missing array field {key}")))
}

fn validate_source_relative_path<'a>(
    path: &'a str,
    stage: &'static str,
) -> AuditResult<Vec<&'a str>> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(AuditFailure::new(
            stage,
            format!("unsafe relative path: {path}"),
        ));
    }
    let components: Vec<_> = path.split('/').collect();
    for component in &components {
        validate_component(component, stage)?;
    }
    Ok(components)
}

fn validate_component(component: &str, stage: &'static str) -> AuditResult<()> {
    if component.is_empty()
        || component == "."
        || component == ".."
        || component.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(AuditFailure::new(
            stage,
            format!("unsafe path component: {component:?}"),
        ));
    }
    Ok(())
}

fn validate_package_component(component: &str, stage: &'static str) -> AuditResult<()> {
    validate_component(component, stage)?;
    if component.contains('/') || component.contains('\\') {
        return Err(AuditFailure::new(
            stage,
            format!("invalid package identifier component: {component}"),
        ));
    }
    Ok(())
}
