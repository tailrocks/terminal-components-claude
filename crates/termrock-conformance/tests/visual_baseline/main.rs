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
mod button_busy_live;
mod holla;
mod holla_journeys;
mod jackin;
mod jackin_journeys;
mod negative_controls_phase6a;
mod pointer;
mod showcase;
mod showcase_journeys;
mod showcase_pending_s1;
mod support;
mod tablepro;
mod tablepro_journeys;
mod tablepro_pending_t1;

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

/// Expected admission pins, compiled from frozen evidence (Q04/FIX-012 A2).
/// Each value cites its source; `store_integrity` re-proves every pin against
/// the full 7550-capture corpus on each run, so a wrong constant fails loudly.
mod expected {
    /// Evidence: `git rev-parse 'visual-baseline^{commit}'` (tag object 1ee5ebdc
    /// points at this commit; tag itself is frozen, never moved) and
    /// `baselines/tuiscotti-v1/corpus-index.json` (`historical_oracle_commit`) /
    /// `admission-record.json` (`historical_oracle_commit`).
    pub const HISTORICAL_ORACLE_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
    /// Evidence: `baselines/tuiscotti-v1/corpus-index.json`
    /// (`historical_oracle_tag`).
    pub const HISTORICAL_ORACLE_TAG: &str = "visual-baseline";
    /// Evidence: `baselines/tuiscotti-v1/corpus-index.json` and
    /// `admission-record.json` (`reference_app_sha`).
    pub const REFERENCE_APP_SHA: &str = "7bd6a331721737514a2477c894d922cb262ef07b";
    /// Evidence: workspace `Cargo.lock` tuiscotti source rev
    /// (`git+https://github.com/tailrocks/tuiscotti?rev=a47c9aae…`), matching
    /// `corpus-index.json` (`tuiscotti_source_sha`) and `admission-record.json`
    /// (`tuiscotti_pin`).
    pub const TUISCOTTI_PIN: &str = "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274";
    /// Evidence: `baselines/tuiscotti-v1/admission-record.json` (`status`).
    pub const ADMISSION_STATUS: &str = "admitted";
    /// Evidence: `baselines/tuiscotti-v1/admission-record.json` (`schema`).
    pub const ADMISSION_SCHEMA: &str = "termrock-spec/tuiscotti-admission-record-v1";
    /// Evidence: `baselines/tuiscotti-v1/corpus-index.json` (`schema`).
    pub const CORPUS_INDEX_SCHEMA: &str = "termrock-spec/tuiscotti-corpus-index-v1";
    /// Evidence: `profile` field uniform across all 7550
    /// `*.manifest.json` and all 7550 `*.png.fidelity.json` files.
    pub const RENDER_PROFILE: &str = "tuiscotti-default";
    /// Evidence: `font_sha256` uniform across all 7550 `*.png.fidelity.json`.
    pub const FONT_SHA256: &str =
        "f2a5ea6cfab397445ffab00c0370927b66d61e560a05db5db271b42006381c1a";
    /// Evidence: `font_desc` uniform across all 7550 `*.png.fidelity.json`.
    pub const FONT_DESC: &str = "vendored JetBrainsMonoNerdFontMono-Regular (SIL OFL 1.1)";
    /// Evidence: `scale` uniform across all 7550 `*.png.fidelity.json`.
    pub const RENDER_SCALE: u32 = 2;
    /// Evidence: `provenance.tool` uniform across sampled `*.observations.json`
    /// (re-proved for every capture by `store_integrity`).
    pub const PROVENANCE_TOOL: &str = "tuiscotti";
    /// Evidence: `provenance.tool_version` uniform across sampled
    /// `*.observations.json`; matches the `tuiscotti 0.2.0` entry in
    /// `Cargo.lock`.
    pub const PROVENANCE_TOOL_VERSION: &str = "0.2.0";
}

#[derive(Debug, Deserialize)]
struct CorpusIndex {
    #[serde(default)]
    manifest_hashes: BTreeMap<String, String>,
    total_artifacts: usize,
    total_captures: usize,
    total_screens: usize,
    schema: String,
    historical_oracle_commit: String,
    historical_oracle_tag: String,
    reference_app_sha: String,
    tuiscotti_source_sha: String,
}

#[derive(Debug, Deserialize)]
struct ScenarioManifest {
    schema_version: u32,
    complete: bool,
    generation: String,
    profile: String,
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
    if index.schema != expected::CORPUS_INDEX_SCHEMA {
        return Err(format!(
            "unexpected corpus-index schema: expected '{}', got '{}'",
            expected::CORPUS_INDEX_SCHEMA,
            index.schema
        ));
    }
    if index.historical_oracle_commit != expected::HISTORICAL_ORACLE_COMMIT {
        return Err(format!(
            "unexpected corpus-index historical_oracle_commit: expected '{}', got '{}'",
            expected::HISTORICAL_ORACLE_COMMIT,
            index.historical_oracle_commit
        ));
    }
    if index.historical_oracle_tag != expected::HISTORICAL_ORACLE_TAG {
        return Err(format!(
            "unexpected corpus-index historical_oracle_tag: expected '{}', got '{}'",
            expected::HISTORICAL_ORACLE_TAG,
            index.historical_oracle_tag
        ));
    }
    if index.reference_app_sha != expected::REFERENCE_APP_SHA {
        return Err(format!(
            "unexpected corpus-index reference_app_sha: expected '{}', got '{}'",
            expected::REFERENCE_APP_SHA,
            index.reference_app_sha
        ));
    }
    if index.tuiscotti_source_sha != expected::TUISCOTTI_PIN {
        return Err(format!(
            "unexpected corpus-index tuiscotti_source_sha: expected '{}', got '{}'",
            expected::TUISCOTTI_PIN,
            index.tuiscotti_source_sha
        ));
    }
    Ok((index, bytes))
}

fn admission_str<'a>(admission: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    admission
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("admission-record.json missing {field}"))
}

fn verify_admission(
    approved_root: &Path,
    corpus_index: &CorpusIndex,
    corpus_index_bytes: &[u8],
) -> Result<(), String> {
    let admission_path = approved_root.join("admission-record.json");
    if !admission_path.is_file() {
        return Err("missing admission-record.json".to_string());
    }
    let bytes =
        std::fs::read(&admission_path).map_err(|e| format!("read admission-record.json: {e}"))?;
    let admission: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse admission-record.json: {e}"))?;

    let expected_corpus_sha256 = sha256_hex(corpus_index_bytes);
    let recorded_sha256 = admission_str(&admission, "corpus_index_sha256")?;

    if recorded_sha256 != expected_corpus_sha256 {
        return Err(format!(
            "corpus_index_sha256 mismatch: recorded {recorded_sha256} != actual {expected_corpus_sha256}"
        ));
    }

    let acquisition_method = admission_str(&admission, "acquisition_method")?;
    if acquisition_method != "legacy_replayed_conversion" {
        return Err(format!(
            "unexpected acquisition_method: expected 'legacy_replayed_conversion', got '{acquisition_method}'"
        ));
    }

    let working_branch = admission_str(&admission, "working_branch")?;
    if working_branch != "termrock-refactor" {
        return Err(format!(
            "unexpected working_branch: expected 'termrock-refactor', got '{working_branch}'"
        ));
    }

    let schema = admission_str(&admission, "schema")?;
    if schema != expected::ADMISSION_SCHEMA {
        return Err(format!(
            "unexpected admission schema: expected '{}', got '{schema}'",
            expected::ADMISSION_SCHEMA
        ));
    }

    let status = admission_str(&admission, "status")?;
    if status != expected::ADMISSION_STATUS {
        return Err(format!(
            "unexpected admission status: expected '{}', got '{status}'",
            expected::ADMISSION_STATUS
        ));
    }

    let oracle_commit = admission_str(&admission, "historical_oracle_commit")?;
    if oracle_commit != expected::HISTORICAL_ORACLE_COMMIT {
        return Err(format!(
            "unexpected admission historical_oracle_commit: expected '{}', got '{oracle_commit}'",
            expected::HISTORICAL_ORACLE_COMMIT
        ));
    }
    if oracle_commit != corpus_index.historical_oracle_commit {
        return Err(format!(
            "admission/corpus-index historical_oracle_commit disagreement: admission '{oracle_commit}' != index '{}'",
            corpus_index.historical_oracle_commit
        ));
    }

    let reference_app_sha = admission_str(&admission, "reference_app_sha")?;
    if reference_app_sha != expected::REFERENCE_APP_SHA {
        return Err(format!(
            "unexpected admission reference_app_sha: expected '{}', got '{reference_app_sha}'",
            expected::REFERENCE_APP_SHA
        ));
    }
    if reference_app_sha != corpus_index.reference_app_sha {
        return Err(format!(
            "admission/corpus-index reference_app_sha disagreement: admission '{reference_app_sha}' != index '{}'",
            corpus_index.reference_app_sha
        ));
    }

    let tuiscotti_pin = admission_str(&admission, "tuiscotti_pin")?;
    if tuiscotti_pin != expected::TUISCOTTI_PIN {
        return Err(format!(
            "unexpected admission tuiscotti_pin: expected '{}', got '{tuiscotti_pin}'",
            expected::TUISCOTTI_PIN
        ));
    }
    if tuiscotti_pin != corpus_index.tuiscotti_source_sha {
        return Err(format!(
            "admission/corpus-index tuiscotti sha disagreement: admission '{tuiscotti_pin}' != index '{}'",
            corpus_index.tuiscotti_source_sha
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

    // Corpus-wide render-profile pin: every manifest must carry the admitted
    // profile, so a mixed-profile corpus fails instead of passing silently.
    if manifest.profile != expected::RENDER_PROFILE {
        return Err(format!(
            "profile mismatch in {scenario_name}: expected '{}', got '{}'",
            expected::RENDER_PROFILE,
            manifest.profile
        ));
    }

    // Per-capture generation digest must be well-formed: 64 lowercase hex.
    // Uniqueness-per-capture is by design (content digest); binding to the
    // rendered html below is what makes a mixed-generation corpus fail.
    if manifest.generation.len() != 64
        || !manifest
            .generation
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(format!(
            "malformed generation digest in {scenario_name}: expected 64 lowercase hex, got '{}'",
            manifest.generation
        ));
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

    let mut html_bytes: Option<Vec<u8>> = None;
    let mut fidelity_bytes: Option<Vec<u8>> = None;
    let mut observations_bytes: Option<Vec<u8>> = None;

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

        match key {
            "html" => html_bytes = Some(bytes),
            "png_fidelity_json" => fidelity_bytes = Some(bytes),
            "observations_json" => observations_bytes = Some(bytes),
            _ => {}
        }
    }

    verify_generation_binding(scenario_name, &manifest, &html_bytes)?;
    verify_render_identity(scenario_name, &fidelity_bytes, &observations_bytes)?;

    Ok(())
}

/// Bind the manifest `generation` digest to the generation marker embedded in
/// the rendered html artifact (`<!-- generation: <hex> -->`). A manifest
/// spliced in from another capture or corpus (mixed generation) carries a
/// digest the html does not contain, so the mix fails here.
fn verify_generation_binding(
    scenario_name: &str,
    manifest: &ScenarioManifest,
    html_bytes: &Option<Vec<u8>>,
) -> Result<(), String> {
    let html = html_bytes
        .as_ref()
        .ok_or_else(|| format!("missing html bytes for generation binding in {scenario_name}"))?;
    let marker = format!("<!-- generation: {} -->", manifest.generation);
    if !html
        .windows(marker.len())
        .any(|window| window == marker.as_bytes())
    {
        return Err(format!(
            "generation binding mismatch in {scenario_name}: html artifact does not embed manifest generation '{}'",
            manifest.generation
        ));
    }
    Ok(())
}

/// Pin renderer/font/provenance identity recorded in the companion artifacts:
/// fidelity profile, font sha/desc, render scale, and the observations
/// provenance tool identity. A capture rendered with a different
/// renderer/font/toolchain fails here even if its pixels hash-match.
fn verify_render_identity(
    scenario_name: &str,
    fidelity_bytes: &Option<Vec<u8>>,
    observations_bytes: &Option<Vec<u8>>,
) -> Result<(), String> {
    let fidelity_bytes = fidelity_bytes
        .as_ref()
        .ok_or_else(|| format!("missing fidelity bytes for render identity in {scenario_name}"))?;
    let fidelity: serde_json::Value = serde_json::from_slice(fidelity_bytes)
        .map_err(|e| format!("parse fidelity json in {scenario_name}: {e}"))?;
    let fidelity_str = |field: &str| -> Result<&str, String> {
        fidelity
            .get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("fidelity json in {scenario_name} missing {field}"))
    };
    if fidelity_str("profile")? != expected::RENDER_PROFILE {
        return Err(format!(
            "fidelity profile mismatch in {scenario_name}: expected '{}'",
            expected::RENDER_PROFILE
        ));
    }
    if fidelity_str("font_sha256")? != expected::FONT_SHA256 {
        return Err(format!(
            "fidelity font_sha256 mismatch in {scenario_name}: expected '{}'",
            expected::FONT_SHA256
        ));
    }
    if fidelity_str("font_desc")? != expected::FONT_DESC {
        return Err(format!(
            "fidelity font_desc mismatch in {scenario_name}: expected '{}'",
            expected::FONT_DESC
        ));
    }
    let scale = fidelity
        .get("scale")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("fidelity json in {scenario_name} missing scale"))?;
    if scale != u64::from(expected::RENDER_SCALE) {
        return Err(format!(
            "fidelity scale mismatch in {scenario_name}: expected {}, got {scale}",
            expected::RENDER_SCALE
        ));
    }
    if fidelity.get("approximate").and_then(|v| v.as_bool()) != Some(false) {
        return Err(format!(
            "fidelity approximate must be false in {scenario_name}"
        ));
    }

    let observations_bytes = observations_bytes.as_ref().ok_or_else(|| {
        format!("missing observations bytes for render identity in {scenario_name}")
    })?;
    let observations: serde_json::Value = serde_json::from_slice(observations_bytes)
        .map_err(|e| format!("parse observations json in {scenario_name}: {e}"))?;
    let provenance = observations
        .get("provenance")
        .ok_or_else(|| format!("observations json in {scenario_name} missing provenance"))?;
    let provenance_str = |field: &str| -> Result<&str, String> {
        provenance
            .get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("observations provenance in {scenario_name} missing {field}"))
    };
    if provenance_str("tool")? != expected::PROVENANCE_TOOL {
        return Err(format!(
            "provenance tool mismatch in {scenario_name}: expected '{}'",
            expected::PROVENANCE_TOOL
        ));
    }
    if provenance_str("tool_version")? != expected::PROVENANCE_TOOL_VERSION {
        return Err(format!(
            "provenance tool_version mismatch in {scenario_name}: expected '{}'",
            expected::PROVENANCE_TOOL_VERSION
        ));
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

    verify_admission(&approved, &corpus_index, &corpus_index_bytes)
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

    let (corpus_index, _) = verify_corpus_index(&approved).expect("real corpus-index parses");
    let tampered_corpus_index_bytes = b"{\"tampered\": true}";
    let result = verify_admission(tmp.path(), &corpus_index, tampered_corpus_index_bytes);
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

/// Copy the real admission record into a tempdir, patch one field with a
/// synthetic value, and run the admission gate against the real corpus index.
/// The frozen `baselines/` tree is only read, never written.
fn verify_admission_with_patched_field(field: &str, value: &str) -> Result<(), String> {
    let approved = support::baseline_store_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = std::fs::read(approved.join("admission-record.json")).expect("read admission");
    let mut admission: serde_json::Value = serde_json::from_slice(&src).expect("parse admission");
    admission[field] = serde_json::Value::String(value.to_string());
    std::fs::write(
        tmp.path().join("admission-record.json"),
        serde_json::to_vec_pretty(&admission).expect("encode admission"),
    )
    .expect("write patched admission");
    let (corpus_index, corpus_index_bytes) =
        verify_corpus_index(&approved).expect("real corpus-index parses");
    verify_admission(tmp.path(), &corpus_index, &corpus_index_bytes)
}

#[test]
fn tampered_admission_oracle_commit_fails_verification() {
    let bogus = "0000000000000000000000000000000000000000";
    let err = verify_admission_with_patched_field("historical_oracle_commit", bogus)
        .expect_err("verification must fail when historical_oracle_commit is tampered");
    assert!(
        err.contains("historical_oracle_commit"),
        "expected historical_oracle_commit error, got: {err}"
    );
}

#[test]
fn tampered_admission_reference_app_sha_fails_verification() {
    let bogus = "1111111111111111111111111111111111111111";
    let err = verify_admission_with_patched_field("reference_app_sha", bogus)
        .expect_err("verification must fail when reference_app_sha is tampered");
    assert!(
        err.contains("reference_app_sha"),
        "expected reference_app_sha error, got: {err}"
    );
}

#[test]
fn tampered_admission_tuiscotti_pin_fails_verification() {
    let bogus = "2222222222222222222222222222222222222222";
    let err = verify_admission_with_patched_field("tuiscotti_pin", bogus)
        .expect_err("verification must fail when tuiscotti_pin is tampered");
    assert!(
        err.contains("tuiscotti_pin"),
        "expected tuiscotti_pin error, got: {err}"
    );
}

#[test]
fn non_admitted_status_fails_verification() {
    let err = verify_admission_with_patched_field("status", "pending")
        .expect_err("verification must fail when status is not admitted");
    assert!(
        err.contains("admission status"),
        "expected admission status error, got: {err}"
    );
}

#[test]
fn admission_index_tuiscotti_disagreement_fails_verification() {
    // Admission record is genuine; the corpus index presented alongside it
    // carries a different tuiscotti sha, so the cross-check must fail.
    let approved = support::baseline_store_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::copy(
        approved.join("admission-record.json"),
        tmp.path().join("admission-record.json"),
    )
    .expect("copy admission");
    let (corpus_index, corpus_index_bytes) =
        verify_corpus_index(&approved).expect("real corpus-index parses");
    let mut tampered: serde_json::Value =
        serde_json::from_slice(&corpus_index_bytes).expect("parse index");
    tampered["tuiscotti_source_sha"] =
        serde_json::Value::String("3333333333333333333333333333333333333333".to_string());
    let tampered_index: CorpusIndex =
        serde_json::from_value(tampered).expect("tampered index parses");
    assert_ne!(
        tampered_index.tuiscotti_source_sha,
        corpus_index.tuiscotti_source_sha
    );
    let err = verify_admission(tmp.path(), &tampered_index, &corpus_index_bytes)
        .expect_err("verification must fail when admission and index disagree on tuiscotti sha");
    assert!(
        err.contains("tuiscotti"),
        "expected tuiscotti disagreement error, got: {err}"
    );
}

/// Copy one scenario into a tempdir and patch a top-level manifest field with
/// a synthetic value. Callers pass `None` as the expected manifest hash so
/// only content checks run.
fn copy_scenario_with_patched_manifest(
    scenario_name: &str,
    field: &str,
    value: &str,
) -> tempfile::TempDir {
    let approved = support::baseline_store_root();
    let tmp = copy_scenario_to_tempdir(&approved, scenario_name);
    let manifest_path = tmp.path().join(format!("{scenario_name}.manifest.json"));
    let src = std::fs::read(&manifest_path).expect("read manifest");
    let mut manifest: serde_json::Value = serde_json::from_slice(&src).expect("parse manifest");
    manifest[field] = serde_json::Value::String(value.to_string());
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("encode manifest"),
    )
    .expect("write patched manifest");
    tmp
}

fn read_manifest_field(scenario_name: &str, field: &str) -> String {
    let approved = support::baseline_store_root();
    let src = std::fs::read(approved.join(format!("{scenario_name}.manifest.json")))
        .expect("read donor manifest");
    let manifest: serde_json::Value = serde_json::from_slice(&src).expect("parse donor manifest");
    manifest
        .get(field)
        .and_then(|v| v.as_str())
        .expect("donor field present")
        .to_string()
}

#[test]
fn mixed_profile_fails_verification() {
    let scenario = "jackin/settings/mounts/72x20/16";
    let tmp = copy_scenario_with_patched_manifest(scenario, "profile", "foreign-profile");
    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    let err = result.expect_err("verification must fail when manifest profile is mixed");
    assert!(
        err.contains("profile mismatch"),
        "expected profile mismatch error, got: {err}"
    );
}

#[test]
fn mixed_generation_fails_verification() {
    // Splice the generation digest of a *different* real capture into this
    // manifest: a genuinely mixed-generation corpus. The html binding must fail.
    let scenario = "jackin/settings/mounts/72x20/16";
    let donor = "holla/upgrade/excluded/72x20/truecolor";
    let foreign_generation = read_manifest_field(donor, "generation");
    let own_generation = read_manifest_field(scenario, "generation");
    assert_ne!(
        foreign_generation, own_generation,
        "donor must have a distinct generation digest"
    );
    let tmp = copy_scenario_with_patched_manifest(scenario, "generation", &foreign_generation);
    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    let err = result.expect_err("verification must fail when manifest generation is mixed");
    assert!(
        err.contains("generation binding mismatch"),
        "expected generation binding mismatch error, got: {err}"
    );
}

#[test]
fn malformed_generation_fails_verification() {
    let scenario = "jackin/settings/mounts/72x20/16";
    let tmp = copy_scenario_with_patched_manifest(scenario, "generation", "not-a-hex-digest");
    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    let err = result.expect_err("verification must fail when manifest generation is malformed");
    assert!(
        err.contains("malformed generation"),
        "expected malformed generation error, got: {err}"
    );
}

#[test]
fn tampered_font_identity_fails_verification() {
    // Rewrite the fidelity companion with a foreign font_sha256, patching the
    // manifest artifact entry (hash + length) so only the identity pin fires.
    let scenario = "jackin/settings/mounts/72x20/16";
    let approved = support::baseline_store_root();
    let tmp = copy_scenario_to_tempdir(&approved, scenario);
    let fidelity_path = tmp.path().join(format!("{scenario}.png.fidelity.json"));
    let src = std::fs::read(&fidelity_path).expect("read fidelity");
    let mut fidelity: serde_json::Value = serde_json::from_slice(&src).expect("parse fidelity");
    fidelity["font_sha256"] = serde_json::Value::String("0".repeat(64));
    let patched = serde_json::to_vec_pretty(&fidelity).expect("encode fidelity");
    std::fs::write(&fidelity_path, &patched).expect("write patched fidelity");

    let manifest_path = tmp.path().join(format!("{scenario}.manifest.json"));
    let manifest_src = std::fs::read(&manifest_path).expect("read manifest");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&manifest_src).expect("parse manifest");
    let entry = manifest
        .get_mut("artifacts")
        .and_then(|v| v.get_mut("png_fidelity_json"))
        .expect("fidelity entry present");
    entry["sha256"] = serde_json::Value::String(sha256_hex(&patched));
    entry["bytes"] = serde_json::Value::from(patched.len() as u64);
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("encode manifest"),
    )
    .expect("write patched manifest");

    let result = verify_scenario_manifest(tmp.path(), scenario, None);
    let err = result.expect_err("verification must fail when font identity is tampered");
    assert!(
        err.contains("font_sha256 mismatch"),
        "expected font_sha256 mismatch error, got: {err}"
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
