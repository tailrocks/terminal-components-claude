//! Visual-baseline suite: every capturable surface of the four binaries,
//! driven as real processes in PTYs via the tuiscotti library and gated
//! cell-exact (`.ansi`), content (`.txt`), render-level (`.html`) and
//! pixel-exact (`.png`) against the approved frames in
//! `baselines/tuiscotti-v1` (the grouped multi-artifact store; scratch
//! under `target/tuiscotti/`).
//!
//! The capture matrix is the vendored case registry
//! (`tests/conformance/required_cases.json`: 302 `LEGACY:` roots × 5 sizes
//! × 5 colours); [`support::suite_capture_names`] expands it, and the
//! [`store_integrity`] gate proves store==suite registry coverage. Capture
//! names are screen-first grouped paths
//! (`<app>/<screen>/.../<cols>x<rows>/<color>`); argv, boot needles, send
//! steps and per-capture timeouts are the ported runner's, verbatim (via
//! [`support::run_canonical`]); the `pointer` module adds the mouse/resize
//! group (hover, drag-select, wheel scroll-fade, resize sequences). The
//! `audit` module drives the 10-fixture × 5 sizes × 5 colours audit
//! matrices data-drivenly; the audit-flow variant matrices live in
//! `showcase.rs` (keyboard) and `pointer.rs` (drag-select).
//!
//! Every capture test is `#[ignore]`d: default `cargo nextest run` compiles
//! the suite and runs only the cheap non-PTY [`store_integrity`] check. Run
//! the PTY baseline explicitly:
//!
//! ```sh
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(holla_)'
//! cargo nextest run --run-ignored only --ignore-default-filter -E 'test(rebuild_review_html)'
//! ```
//!
//! Gate policy (fail-closed): only `matched` passes. Pending and
//! missing-approval captures fail until the whole suite is generated and
//! explicitly blessed out-of-band into `baselines/tuiscotti-v1`; drift
//! after approval or a capture error also fails.

#![cfg(any(target_os = "macos", target_os = "linux"))]

mod audit;
mod holla;
mod jackin;
mod pointer;
mod showcase;
mod support;
mod tablepro;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use rayon::prelude::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tuiscotti::grouped::{self, GroupedStore};
use tuiscotti::{Profile, VENDORED_FACES};

const STORE_EXTS: [&str; 10] = [
    "frame.json",
    "ansi",
    "txt",
    "png",
    "html",
    "ascii",
    "ascii.loss.json",
    "png.fidelity.json",
    "observations.json",
    "manifest.json",
];

const MANIFEST_ARTIFACT_KEYS: [&str; 9] = [
    "ansi",
    "ascii",
    "ascii_loss_json",
    "frame_json",
    "html",
    "observations_json",
    "png",
    "png_fidelity_json",
    "txt",
];

#[derive(Debug, Deserialize)]
struct CorpusIndex {
    #[serde(default)]
    manifest_hashes: BTreeMap<String, String>,
    total_artifacts: usize,
    total_captures: usize,
    total_screens: usize,
}

#[derive(Debug, Deserialize)]
struct ScenarioManifest {
    schema_version: u32,
    complete: bool,
    dimensions: ManifestDimensions,
    artifacts: BTreeMap<String, ArtifactEntry>,
    ansi_sha256: String,
    frame_sha256: String,
    html_sha256: String,
    png_sha256: String,
    txt_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ManifestDimensions {
    cols: u16,
    rows: u16,
}

#[derive(Debug, Deserialize)]
struct ArtifactEntry {
    file: String,
    bytes: usize,
    sha256: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_geometry(geom: &str) -> Option<(u16, u16)> {
    let (cols, rows) = geom.split_once('x')?;
    let cols: u16 = cols.parse().ok()?;
    let rows: u16 = rows.parse().ok()?;
    Some((cols, rows))
}

fn verify_corpus_index(approved_root: &Path) -> Result<(CorpusIndex, Vec<u8>), String> {
    let index_path = approved_root.join("corpus-index.json");
    if !index_path.is_file() {
        return Err("missing corpus-index.json".to_string());
    }
    let bytes = std::fs::read(&index_path).map_err(|e| format!("read corpus-index.json: {e}"))?;
    let index: CorpusIndex =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse corpus-index.json: {e}"))?;
    if index.total_artifacts != 75500 {
        return Err(format!(
            "expected total_artifacts == 75500, got {}",
            index.total_artifacts
        ));
    }
    if index.total_captures != 7550 {
        return Err(format!(
            "expected total_captures == 7550, got {}",
            index.total_captures
        ));
    }
    if index.total_screens != 302 {
        return Err(format!(
            "expected total_screens == 302, got {}",
            index.total_screens
        ));
    }
    Ok((index, bytes))
}

fn verify_admission(approved_root: &Path, corpus_index_bytes: &[u8]) -> Result<(), String> {
    let admission_path = approved_root.join("admission-record.json");
    if !admission_path.is_file() {
        return Err("missing admission-record.json".to_string());
    }
    let bytes =
        std::fs::read(&admission_path).map_err(|e| format!("read admission-record.json: {e}"))?;
    let admission: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse admission-record.json: {e}"))?;

    let expected_corpus_sha256 = sha256_hex(corpus_index_bytes);
    let recorded_sha256 = admission
        .get("corpus_index_sha256")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "admission-record.json missing corpus_index_sha256".to_string())?;

    if recorded_sha256 != expected_corpus_sha256 {
        return Err(format!(
            "corpus_index_sha256 mismatch: recorded {recorded_sha256} != actual {expected_corpus_sha256}"
        ));
    }

    let acquisition_method = admission
        .get("acquisition_method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "admission-record.json missing acquisition_method".to_string())?;
    if acquisition_method != "legacy_replayed_conversion" {
        return Err(format!(
            "unexpected acquisition_method: expected 'legacy_replayed_conversion', got '{acquisition_method}'"
        ));
    }

    let working_branch = admission
        .get("working_branch")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "admission-record.json missing working_branch".to_string())?;
    if working_branch != "termrock-refactor" {
        return Err(format!(
            "unexpected working_branch: expected 'termrock-refactor', got '{working_branch}'"
        ));
    }

    Ok(())
}

fn verify_scenario_manifest(
    approved_root: &Path,
    scenario_name: &str,
    expected_manifest_hash: Option<&str>,
) -> Result<(), String> {
    let manifest_path = approved_root.join(format!("{scenario_name}.manifest.json"));
    if !manifest_path.is_file() {
        return Err(format!("missing manifest: {}", manifest_path.display()));
    }
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|e| format!("read manifest {}: {e}", manifest_path.display()))?;

    if let Some(expected_hash) = expected_manifest_hash {
        let actual_hash = sha256_hex(&manifest_bytes);
        if actual_hash != expected_hash {
            return Err(format!(
                "manifest hash mismatch for {scenario_name}: expected {expected_hash}, got {actual_hash}"
            ));
        }
    }

    let manifest: ScenarioManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("parse manifest {scenario_name}: {e}"))?;

    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported schema_version {} in {scenario_name}",
            manifest.schema_version
        ));
    }
    if !manifest.complete {
        return Err(format!("manifest complete is false in {scenario_name}"));
    }

    let mut parts = scenario_name.rsplit('/');
    let _color = parts
        .next()
        .ok_or_else(|| format!("invalid scenario name: {scenario_name}"))?;
    let size = parts
        .next()
        .ok_or_else(|| format!("invalid scenario name: {scenario_name}"))?;
    let (expected_cols, expected_rows) = parse_geometry(size)
        .ok_or_else(|| format!("cannot parse geometry from size '{size}' in {scenario_name}"))?;

    if manifest.dimensions.cols != expected_cols || manifest.dimensions.rows != expected_rows {
        return Err(format!(
            "dimension mismatch for {scenario_name}: expected {}x{}, got {}x{}",
            expected_cols, expected_rows, manifest.dimensions.cols, manifest.dimensions.rows
        ));
    }

    for key in MANIFEST_ARTIFACT_KEYS {
        let entry = manifest.artifacts.get(key).ok_or_else(|| {
            format!("manifest for {scenario_name} missing artifact entry '{key}'")
        })?;

        let artifact_path = approved_root.join(&entry.file);
        if !artifact_path.is_file() {
            return Err(format!(
                "artifact file missing: {}",
                artifact_path.display()
            ));
        }

        let bytes = std::fs::read(&artifact_path)
            .map_err(|e| format!("read artifact {}: {e}", artifact_path.display()))?;

        if bytes.len() != entry.bytes {
            return Err(format!(
                "byte length mismatch for {}: expected {}, got {}",
                entry.file,
                entry.bytes,
                bytes.len()
            ));
        }

        let digest = sha256_hex(&bytes);
        if digest != entry.sha256 {
            return Err(format!(
                "sha256 mismatch for {}: expected {}, got {}",
                entry.file, entry.sha256, digest
            ));
        }

        let expected_root_hash = match key {
            "ansi" => Some(("ansi", &manifest.ansi_sha256)),
            "frame_json" => Some(("frame", &manifest.frame_sha256)),
            "html" => Some(("html", &manifest.html_sha256)),
            "png" => Some(("png", &manifest.png_sha256)),
            "txt" => Some(("txt", &manifest.txt_sha256)),
            _ => None,
        };
        if let Some((name, expected)) =
            expected_root_hash.filter(|(_, expected)| **expected != entry.sha256)
        {
            return Err(format!(
                "root {name}_sha256 mismatch in {scenario_name}: manifest has {expected}, entry has {}",
                entry.sha256
            ));
        }
    }

    Ok(())
}

/// Cheap non-PTY gate: committed `baselines/tuiscotti-v1` names match the suite, each
/// scenario is screen-first `app/screen/.../geometry/color` with exactly ten artifacts,
/// cryptographic hashes are fully verified fail-closed, and legacy stores are gone.
#[test]
fn store_integrity() {
    let approved = support::baseline_store_root();
    let store = GroupedStore::new(&approved);
    let store_names: BTreeSet<String> = store
        .approved_names()
        .expect("list approved names")
        .into_iter()
        .collect();

    let (by_name, extra) = walk_store(&approved);
    assert!(
        extra.is_empty(),
        "baselines/tuiscotti-v1 has unexpected files: {}",
        extra.join(", ")
    );

    let mut incomplete = Vec::new();
    for (name, exts) in &by_name {
        let missing: Vec<_> = STORE_EXTS
            .iter()
            .copied()
            .filter(|ext| !exts.contains(*ext))
            .collect();
        if !missing.is_empty() || exts.len() != STORE_EXTS.len() {
            incomplete.push(format!("{name} has {exts:?} (missing {missing:?})"));
        }
    }
    assert!(
        incomplete.is_empty(),
        "every scenario needs exactly ten artifacts: {}",
        incomplete.join("; ")
    );

    for name in &store_names {
        grouped::validate_name(name).unwrap_or_else(|e| panic!("{e}"));
        let slashes = name.bytes().filter(|&b| b == b'/').count();
        assert!(
            slashes >= 3,
            "scenario `{name}` must be nested under screen-first hierarchy (at least three `/`)"
        );
        let mut parts = name.rsplit('/');
        let color = parts.next().expect("color leaf");
        let size = parts.next().expect("size folder");
        assert!(
            matches!(color, "truecolor" | "256" | "16" | "none" | "nocolor"),
            "scenario `{name}` last component must be a color suffix"
        );
        let size_ok = size.split_once('x').is_some_and(|(cols, rows)| {
            !cols.is_empty()
                && !rows.is_empty()
                && cols.chars().all(|ch| ch.is_ascii_digit())
                && rows.chars().all(|ch| ch.is_ascii_digit())
        });
        assert!(
            size_ok,
            "scenario `{name}` must put terminal size in its own <cols>x<rows> folder"
        );
    }

    let suite = support::suite_capture_names();
    let pending: Vec<_> = suite.difference(&store_names).cloned().collect();
    let orphans: Vec<_> = store_names.difference(&suite).cloned().collect();
    assert!(
        orphans.is_empty(),
        "store contains stale names not in suite inventory ({}): {:?}",
        orphans.len(),
        orphans
    );
    eprintln!(
        "store inventory: {} approved, {} suite captures, {} pending first approval",
        store_names.len(),
        suite.len(),
        pending.len()
    );
    assert!(
        pending.is_empty(),
        "store is missing approved snapshots ({}): {:?}",
        pending.len(),
        pending
    );

    assert!(
        !Path::new("snapshots").exists(),
        "legacy snapshots/ corpus must be deleted"
    );
    assert!(
        !Path::new("shots").exists(),
        "legacy shots/ corpus must be deleted"
    );
    assert!(
        approved.join("corpus-index.json").is_file(),
        "corpus-index.json must exist"
    );
    assert!(
        approved.join("admission-record.json").is_file(),
        "admission-record.json must exist"
    );

    let (corpus_index, corpus_index_bytes) = verify_corpus_index(&approved)
        .unwrap_or_else(|e| panic!("corpus-index verification failed: {e}"));

    verify_admission(&approved, &corpus_index_bytes)
        .unwrap_or_else(|e| panic!("admission-record verification failed: {e}"));

    let failures: Vec<String> = store_names
        .par_iter()
        .filter_map(|name| {
            let expected_hash = corpus_index.manifest_hashes.get(name).map(|s| s.as_str());
            if let Err(e) = verify_scenario_manifest(&approved, name, expected_hash) {
                Some(format!("{name}: {e}"))
            } else {
                None
            }
        })
        .collect();

    assert!(
        failures.is_empty(),
        "store scenario verification failed for {} scenarios: {:?}",
        failures.len(),
        &failures[..failures.len().min(10)]
    );
}

fn copy_scenario_to_tempdir(src_approved: &Path, scenario_name: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exts = [
        "manifest.json",
        "frame.json",
        "ansi",
        "txt",
        "png",
        "html",
        "ascii",
        "ascii.loss.json",
        "png.fidelity.json",
        "observations.json",
    ];
    for ext in exts {
        let file_name = format!("{scenario_name}.{ext}");
        let src_file = src_approved.join(&file_name);
        let dst_file = tmp.path().join(&file_name);
        if let Some(parent) = dst_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(&src_file, &dst_file).expect("copy artifact");
    }
    tmp
}

#[test]
fn corrupted_artifact_byte_fails_verification() {
    let approved = support::baseline_store_root();
    let scenario = "jackin/settings/mounts/72x20/16";
    let tmp = copy_scenario_to_tempdir(&approved, scenario);

    let png_path = tmp.path().join(format!("{scenario}.png"));
    let mut bytes = std::fs::read(&png_path).expect("read png");
    bytes[0] ^= 0xff;
    std::fs::write(&png_path, bytes).expect("write corrupted png");

    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    assert!(
        result.is_err(),
        "verification must fail when artifact byte is corrupted"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("sha256 mismatch"),
        "expected sha256 mismatch error, got: {err}"
    );
}

#[test]
fn altered_manifest_hash_fails_verification() {
    let approved = support::baseline_store_root();
    let scenario = "jackin/settings/mounts/72x20/16";
    let tmp = copy_scenario_to_tempdir(&approved, scenario);

    let bogus_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let result = verify_scenario_manifest(tmp.path(), scenario, Some(bogus_hash));
    assert!(
        result.is_err(),
        "verification must fail when manifest hash is altered"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("manifest hash mismatch"),
        "expected manifest hash mismatch error, got: {err}"
    );
}

#[test]
fn missing_companion_artifact_fails_verification() {
    let approved = support::baseline_store_root();
    let scenario = "jackin/settings/mounts/72x20/16";
    let tmp = copy_scenario_to_tempdir(&approved, scenario);

    let companion_path = tmp.path().join(format!("{scenario}.ascii.loss.json"));
    std::fs::remove_file(&companion_path).expect("remove companion file");

    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    assert!(
        result.is_err(),
        "verification must fail when companion artifact is missing"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("artifact file missing"),
        "expected artifact file missing error, got: {err}"
    );
}

#[test]
fn mismatched_corpus_index_digest_fails_verification() {
    let approved = support::baseline_store_root();
    let tmp = tempfile::tempdir().expect("tempdir");

    let admission_path = tmp.path().join("admission-record.json");
    std::fs::copy(approved.join("admission-record.json"), &admission_path).expect("copy admission");

    let tampered_corpus_index_bytes = b"{\"tampered\": true}";
    let result = verify_admission(tmp.path(), tampered_corpus_index_bytes);
    assert!(
        result.is_err(),
        "verification must fail when corpus index digest mismatches"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("corpus_index_sha256 mismatch"),
        "expected corpus_index_sha256 mismatch error, got: {err}"
    );
}

fn walk_store(root: &Path) -> (BTreeMap<String, BTreeSet<String>>, Vec<String>) {
    let mut by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut extra = Vec::new();
    walk_store_dir(root, root, &mut by_name, &mut extra);
    (by_name, extra)
}

fn walk_store_dir(
    root: &Path,
    dir: &Path,
    by_name: &mut BTreeMap<String, BTreeSet<String>>,
    extra: &mut Vec<String>,
) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("list {}: {e}", dir.display()))
            .path();
        if path.is_dir() {
            walk_store_dir(root, &path, by_name, extra);
            continue;
        }
        if support::is_macos_platform_metadata(&path) {
            continue;
        }
        let rel = posix_rel(root, &path);
        if rel == "corpus-index.json" || rel == "admission-record.json" {
            continue;
        }
        let mut matched = false;
        for ext in STORE_EXTS {
            let dot_ext = format!(".{ext}");
            if let Some(name) = rel.strip_suffix(&dot_ext) {
                by_name
                    .entry(name.to_string())
                    .or_default()
                    .insert(ext.to_string());
                matched = true;
                break;
            }
        }
        if !matched {
            extra.push(rel);
        }
    }
}

fn posix_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Write a fast file-link index at `target/tuisnap/report.html`. Does not
/// re-render PNGs and does not embed them. Per-capture `#[ignore]` tests are
/// the gate; this only indexes on-disk actual vs approved bytes.
#[test]
#[ignore = "rebuilds target/tuisnap/report.html; run after accept; skip with --skip rebuild_review_html"]
fn rebuild_review_html() {
    let store = support::store();
    let mut renderer = Profile::default_profile()
        .renderer(&VENDORED_FACES)
        .expect("vendored faces parse");
    let report = store
        .report_with(&mut renderer, 1.0, "tuisnap visual report")
        .expect("report generation");
    eprintln!(
        "report: {} ({} captures, {} failed)",
        report.path.display(),
        report.outcomes.len(),
        report.failed()
    );
    assert_eq!(report.failed(), 0, "unmatched gates — review report.html");
}
