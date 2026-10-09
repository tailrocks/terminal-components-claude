use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArtifactReceipt, Case, ExpectedGenerationInput, Geometry, RendererIdentity};

pub(crate) const EXPECTED_GENERATION_SCHEMA: &str = "termrock-spec/parity-expected-generation-v1";
const EXPECTED_TREE_ALGORITHM: &str = "termrock-e2e/expected-tree-v1";
const TUISCOTTI_REVISION: &str = "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274";
const CHECKPOINT_IDS: [&str; 4] = ["00-boot", "01-help", "02-finder", "03-help-again"];
const FRAME_VERSION: u8 = 3;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedGenerationManifest {
    schema: String,
    id: String,
    case_id: String,
    case_set_digest: String,
    profile_digest: String,
    dependency_lock_sha256: String,
    case_input_sha256: String,
    tuiscotti_revision: String,
    tree_hash_algorithm: String,
    oracle: OracleLineage,
    renderer: RendererIdentity,
    checkpoints: Vec<ExpectedCheckpointManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OracleLineage {
    pub(crate) tag_ref: String,
    pub(crate) tag_object: String,
    pub(crate) tag_commit: String,
    pub(crate) capture_run_sha256: String,
    pub(crate) source_inputs_sha256: String,
    pub(crate) build_environment_sha256: String,
    pub(crate) builder_receipt_sha256: String,
    pub(crate) oracle_executable_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedCheckpointManifest {
    id: String,
    observation_id: String,
    geometry: Geometry,
    color_path: String,
    frame: ExpectedFrameFile,
    png: ExpectedPngFile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedFrameFile {
    path: String,
    sha256: String,
    bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedPngFile {
    path: String,
    sha256: String,
    bytes: u64,
    width: u32,
    height: u32,
    color_type: u8,
    bit_depth: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct ExpectedGeneration {
    pub(crate) id: String,
    pub(crate) tree_sha256: String,
    pub(crate) manifest_sha256: String,
    pub(crate) root: PathBuf,
    pub(crate) renderer_sha256: String,
    pub(crate) case_id: String,
    pub(crate) case_set_digest: String,
    pub(crate) profile_digest: String,
    pub(crate) dependency_lock_sha256: String,
    pub(crate) oracle: OracleLineage,
    checkpoints: BTreeMap<String, LoadedExpectedCheckpoint>,
}

impl ExpectedGeneration {
    pub(crate) fn checkpoint_refs(
        &self,
        checkpoint_id: &str,
    ) -> Result<(ArtifactRef, ArtifactRef), String> {
        let checkpoint = self
            .checkpoints
            .get(checkpoint_id)
            .ok_or_else(|| format!("expected generation lacks checkpoint {checkpoint_id}"))?;
        Ok((
            expected_file_ref(
                &self.root,
                &checkpoint.metadata.frame.path,
                &checkpoint.metadata.frame.sha256,
                checkpoint.metadata.frame.bytes,
            ),
            expected_file_ref(
                &self.root,
                &checkpoint.metadata.png.path,
                &checkpoint.metadata.png.sha256,
                checkpoint.metadata.png.bytes,
            ),
        ))
    }
}

#[derive(Clone, Debug)]
struct LoadedExpectedCheckpoint {
    metadata: ExpectedCheckpointManifest,
    frame_bytes: Vec<u8>,
    png_bytes: Vec<u8>,
    frame: tuiscotti::Frame,
    png_info: PngInfoReceipt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VisualComparisonEvidence {
    pub schema: String,
    pub method: String,
    pub admission: VisualAdmission,
    pub renderer: RendererComparison,
    pub frame: FrameComparison,
    pub png: PngComparison,
    pub fidelity: FidelityComparison,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VisualAdmission {
    pub verified: bool,
    pub trust_record_sha256: String,
    pub expected_generation_id: String,
    pub expected_tree_sha256: String,
    pub expected_manifest_sha256: String,
    pub admission_receipt_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RendererComparison {
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub equal: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrameComparison {
    pub expected: ArtifactRef,
    pub actual: ArtifactRef,
    pub version: u8,
    pub expected_geometry: Geometry,
    pub actual_geometry: Geometry,
    pub dimensions_equal: bool,
    pub cells_equal: bool,
    pub cursor_equal: bool,
    pub differing_positions: Vec<DifferingPosition>,
    pub equal: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DifferingPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PngInfoReceipt {
    pub width: u32,
    pub height: u32,
    pub color_type: u8,
    pub bit_depth: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PngComparison {
    pub expected: ArtifactRef,
    pub actual: ArtifactRef,
    pub expected_info: PngInfoReceipt,
    pub actual_info: PngInfoReceipt,
    pub alpha_policy: String,
    pub dimensions_equal: bool,
    pub pixels_equal: bool,
    pub equal: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FidelityBinding {
    pub sha256: String,
    pub bytes: u64,
    pub source_frame_sha256: String,
    pub renderer_sha256: String,
    pub rerender_png_sha256: String,
    pub approximate: bool,
    pub rerender_matches_bound_png: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FidelityComparison {
    pub expected: FidelityBinding,
    pub actual: FidelityBinding,
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedObservation {
    pub(crate) frame_bytes: Vec<u8>,
    pub(crate) png_bytes: Vec<u8>,
    pub(crate) rerender_png_bytes: Vec<u8>,
    pub(crate) fidelity_bytes: Vec<u8>,
    pub(crate) fidelity_approximate: bool,
    pub(crate) renderer_sha256: String,
}

pub(crate) fn load_expected_generation(
    expected: &ExpectedGenerationInput,
    case: &Case,
    case_set_digest: &str,
    profile_digest: &str,
    dependency_lock_sha256: &str,
    tuiscotti_revision: &str,
    renderer: &RendererIdentity,
) -> Result<ExpectedGeneration, String> {
    if expected.id.trim().is_empty() {
        return Err("expected-generation ID must not be empty".to_string());
    }
    require_sha256_lower("subject expected-generation tree SHA-256", &expected.sha256)?;
    let supplied_root = expected
        .root
        .as_deref()
        .ok_or_else(|| "expected generation root is absent".to_string())?;
    let resolved_root = crate::resolve_existing_directory_without_symlinks(
        supplied_root,
        "expected-generation root",
    )?;
    let root = resolved_root.path;

    let files = read_exact_generation_tree(&root)?;
    let manifest_bytes = files
        .get("expected-generation.json")
        .ok_or_else(|| "expected generation manifest is missing".to_string())?;
    let manifest: ExpectedGenerationManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("parse expected-generation manifest: {error}"))?;
    let canonical = serde_json::to_vec(&manifest)
        .map_err(|error| format!("serialize expected-generation manifest: {error}"))?;
    if canonical != *manifest_bytes {
        return Err("expected-generation manifest is not canonical compact JSON".to_string());
    }

    validate_generation_manifest(
        &manifest,
        expected,
        case,
        case_set_digest,
        profile_digest,
        dependency_lock_sha256,
        tuiscotti_revision,
        renderer,
    )?;
    let tree_sha256 = expected_tree_sha256(&files)?;
    if tree_sha256 != expected.sha256 {
        return Err(format!(
            "expected generation digest mismatch: expected {}, got {tree_sha256}",
            expected.sha256
        ));
    }

    let strict_profile = tuiscotti::profile::RenderProfile::vendored();
    let expected_png_size = strict_profile.image_size(case.geometry.cols, case.geometry.rows);
    let mut checkpoints = BTreeMap::new();
    for entry in &manifest.checkpoints {
        let frame_bytes = files
            .get(&entry.frame.path)
            .ok_or_else(|| format!("expected frame file is missing: {}", entry.frame.path))?
            .clone();
        verify_file_record(
            "expected frame",
            &entry.frame.path,
            &entry.frame.sha256,
            entry.frame.bytes,
            &frame_bytes,
        )?;
        let frame = parse_canonical_frame(&frame_bytes, &case.geometry)?;
        validate_screen_provenance(&frame, "expected")?;

        let png_bytes = files
            .get(&entry.png.path)
            .ok_or_else(|| format!("expected PNG file is missing: {}", entry.png.path))?
            .clone();
        verify_file_record(
            "expected PNG",
            &entry.png.path,
            &entry.png.sha256,
            entry.png.bytes,
            &png_bytes,
        )?;
        let png_info = opaque_png_info(&png_bytes, expected_png_size)?;
        if (entry.png.width, entry.png.height) != expected_png_size
            || entry.png.color_type != 2
            || entry.png.bit_depth != 8
        {
            return Err(format!(
                "expected PNG manifest geometry/format is invalid for checkpoint {}",
                entry.id
            ));
        }
        let decoded = tuiscotti::diff::compare_png_with_alpha(
            &png_bytes,
            &png_bytes,
            tuiscotti::diff::AlphaPolicy::Opaque,
        )
        .map_err(|error| format!("decode expected PNG {}: {error}", entry.id))?;
        if !decoded.dims_equal || !decoded.pixels_equal {
            return Err(format!("expected PNG {} is not valid opaque RGB", entry.id));
        }
        checkpoints.insert(
            entry.id.clone(),
            LoadedExpectedCheckpoint {
                metadata: entry.clone(),
                frame_bytes,
                png_bytes,
                frame,
                png_info,
            },
        );
    }

    Ok(ExpectedGeneration {
        id: manifest.id,
        tree_sha256,
        manifest_sha256: crate::sha256_bytes(manifest_bytes),
        root,
        renderer_sha256: renderer.hash.clone(),
        case_id: manifest.case_id,
        case_set_digest: manifest.case_set_digest,
        profile_digest: manifest.profile_digest,
        dependency_lock_sha256: manifest.dependency_lock_sha256,
        oracle: manifest.oracle,
        checkpoints,
    })
}

fn validate_generation_manifest(
    manifest: &ExpectedGenerationManifest,
    expected: &ExpectedGenerationInput,
    case: &Case,
    case_set_digest: &str,
    profile_digest: &str,
    dependency_lock_sha256: &str,
    tuiscotti_revision: &str,
    renderer: &RendererIdentity,
) -> Result<(), String> {
    if manifest.schema != EXPECTED_GENERATION_SCHEMA
        || manifest.case_id != case.id
        || manifest.id != expected.id
        || manifest.tree_hash_algorithm != EXPECTED_TREE_ALGORITHM
    {
        return Err(
            "expected-generation schema, case, ID, or tree algorithm is invalid".to_string(),
        );
    }
    for (name, declared, actual) in [
        (
            "case_set_digest",
            &manifest.case_set_digest,
            case_set_digest,
        ),
        ("profile_digest", &manifest.profile_digest, profile_digest),
        (
            "dependency_lock_sha256",
            &manifest.dependency_lock_sha256,
            dependency_lock_sha256,
        ),
    ] {
        require_sha256_lower(name, declared)?;
        if declared != actual {
            return Err(format!(
                "expected-generation {name} does not match current suite"
            ));
        }
    }
    require_sha256_lower("case_input_sha256", &manifest.case_input_sha256)?;
    let case_bytes = serde_json::to_vec(case)
        .map_err(|error| format!("serialize strict case input: {error}"))?;
    if manifest.case_input_sha256 != crate::sha256_bytes(&case_bytes) {
        return Err("expected-generation selected case input digest does not match".to_string());
    }
    if manifest.tuiscotti_revision != TUISCOTTI_REVISION || tuiscotti_revision != TUISCOTTI_REVISION
    {
        return Err(
            "expected-generation Tuiscotti revision is not the pinned revision".to_string(),
        );
    }
    for (name, declared, wanted) in [
        (
            "oracle tag ref",
            manifest.oracle.tag_ref.as_str(),
            "refs/tags/visual-baseline",
        ),
        (
            "oracle tag object",
            manifest.oracle.tag_object.as_str(),
            "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5",
        ),
        (
            "oracle tag commit",
            manifest.oracle.tag_commit.as_str(),
            "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b",
        ),
    ] {
        if declared != wanted {
            return Err(format!(
                "expected-generation {name} does not match pinned oracle"
            ));
        }
    }
    for (name, digest) in [
        (
            "capture_run_sha256",
            manifest.oracle.capture_run_sha256.as_str(),
        ),
        (
            "source_inputs_sha256",
            manifest.oracle.source_inputs_sha256.as_str(),
        ),
        (
            "build_environment_sha256",
            manifest.oracle.build_environment_sha256.as_str(),
        ),
        (
            "builder_receipt_sha256",
            manifest.oracle.builder_receipt_sha256.as_str(),
        ),
        (
            "oracle_executable_sha256",
            manifest.oracle.oracle_executable_sha256.as_str(),
        ),
    ] {
        require_sha256_lower(name, digest)?;
    }
    if case.id != "HELP-HOLLA-004"
        || case.geometry.cols != 120
        || case.geometry.rows != 40
        || case.color_path != "truecolor"
    {
        return Err(
            "expected-generation is only valid for HELP-HOLLA-004 at 120x40 truecolor".to_string(),
        );
    }
    let case_checkpoints = case
        .steps
        .iter()
        .filter_map(|step| match step {
            crate::Step::Checkpoint { id, .. } => Some(id.as_str()),
            crate::Step::Press { .. } => None,
        })
        .collect::<Vec<_>>();
    if case_checkpoints != CHECKPOINT_IDS {
        return Err(
            "HELP-HOLLA-004 checkpoint sequence does not match the fixed pilot".to_string(),
        );
    }
    if manifest.checkpoints.len() != CHECKPOINT_IDS.len() {
        return Err("expected-generation must contain exactly four checkpoints".to_string());
    }
    let expected_renderer = serde_json::to_value(renderer)
        .map_err(|error| format!("serialize resolved renderer identity: {error}"))?;
    let declared_renderer = serde_json::to_value(&manifest.renderer)
        .map_err(|error| format!("serialize expected renderer identity: {error}"))?;
    if expected_renderer != declared_renderer {
        return Err(
            "expected-generation renderer identity does not match vendored renderer".to_string(),
        );
    }

    let mut observation_ids = BTreeSet::new();
    for (index, entry) in manifest.checkpoints.iter().enumerate() {
        let id = CHECKPOINT_IDS[index];
        if entry.id != id
            || entry.observation_id.trim().is_empty()
            || !observation_ids.insert(entry.observation_id.as_str())
            || entry.geometry.cols != 120
            || entry.geometry.rows != 40
            || entry.color_path != "truecolor"
        {
            return Err(format!(
                "expected-generation checkpoint metadata is invalid for {id}"
            ));
        }
        let frame_path = format!("checkpoints/{id}/frame.json");
        let png_path = format!("checkpoints/{id}/screen.png");
        if entry.frame.path != frame_path || entry.png.path != png_path {
            return Err(format!(
                "expected-generation checkpoint paths are not fixed for {id}"
            ));
        }
        require_sha256_lower(&format!("{id} frame sha256"), &entry.frame.sha256)?;
        require_sha256_lower(&format!("{id} PNG sha256"), &entry.png.sha256)?;
        if entry.png.width == 0 || entry.png.height == 0 {
            return Err(format!("expected PNG dimensions are zero for {id}"));
        }
    }
    Ok(())
}

fn read_exact_generation_tree(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    let mut directories = BTreeSet::new();
    walk_generation_tree(root, root, &mut files, &mut directories)?;
    let expected_files = expected_file_paths();
    let expected_directories = expected_directory_paths();
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected_files {
        return Err(
            "expected-generation tree must contain exactly the nine fixed files".to_string(),
        );
    }
    if directories != expected_directories {
        return Err(
            "expected-generation tree contains a missing or unexpected directory".to_string(),
        );
    }
    Ok(files)
}

fn walk_generation_tree(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
    directories: &mut BTreeSet<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|error| {
        format!(
            "read expected-generation directory {}: {error}",
            current.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read expected-generation entry: {error}"))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "inspect expected-generation path {}: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "expected-generation tree contains a symlink: {}",
                path.display()
            ));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| format!("expected-generation path escaped root: {}", path.display()))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| format!("expected-generation path is not UTF-8: {}", path.display()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        if metadata.is_dir() {
            directories.insert(relative);
            walk_generation_tree(root, &path, files, directories)?;
        } else if metadata.is_file() {
            let bytes = fs::read(&path).map_err(|error| {
                format!("read expected-generation file {}: {error}", path.display())
            })?;
            files.insert(relative, bytes);
        } else {
            return Err(format!(
                "expected-generation entry is not a regular file or directory: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn expected_file_paths() -> BTreeSet<String> {
    let mut files = BTreeSet::from(["expected-generation.json".to_string()]);
    for checkpoint in CHECKPOINT_IDS {
        files.insert(format!("checkpoints/{checkpoint}/frame.json"));
        files.insert(format!("checkpoints/{checkpoint}/screen.png"));
    }
    files
}

fn expected_directory_paths() -> BTreeSet<String> {
    let mut directories = BTreeSet::from(["checkpoints".to_string()]);
    for checkpoint in CHECKPOINT_IDS {
        directories.insert(format!("checkpoints/{checkpoint}"));
    }
    directories
}

fn expected_tree_sha256(files: &BTreeMap<String, Vec<u8>>) -> Result<String, String> {
    let mut digest = Sha256::new();
    digest.update(b"termrock-e2e/expected-tree-v1\n");
    for (path, bytes) in files {
        let path_bytes = path.as_bytes();
        let path_length = u64::try_from(path_bytes.len())
            .map_err(|_| "expected-generation path length does not fit u64".to_string())?;
        let content_length = u64::try_from(bytes.len())
            .map_err(|_| "expected-generation file length does not fit u64".to_string())?;
        digest.update(path_length.to_be_bytes());
        digest.update(path_bytes);
        digest.update(content_length.to_be_bytes());
        digest.update(bytes);
    }
    let digest = digest.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn verify_file_record(
    label: &str,
    path: &str,
    expected_sha256: &str,
    expected_bytes: u64,
    bytes: &[u8],
) -> Result<(), String> {
    require_sha256_lower(&format!("{label} SHA-256"), expected_sha256)?;
    if bytes.len() as u64 != expected_bytes {
        return Err(format!(
            "{label} byte count does not match manifest for {path}"
        ));
    }
    let actual_sha256 = crate::sha256_bytes(bytes);
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "{label} SHA-256 does not match manifest for {path}"
        ));
    }
    Ok(())
}

fn require_sha256_lower(name: &str, digest: &str) -> Result<(), String> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{name} must be lowercase 64-character SHA-256"));
    }
    Ok(())
}

pub(crate) fn parse_canonical_frame(
    bytes: &[u8],
    expected_geometry: &Geometry,
) -> Result<tuiscotti::Frame, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("frame JSON is not UTF-8: {error}"))?;
    let frame = tuiscotti::Frame::from_json(text)
        .map_err(|error| format!("parse canonical Frame v3: {error}"))?;
    if frame.version != FRAME_VERSION {
        return Err(format!("frame version {} is not Frame v3", frame.version));
    }
    if (frame.cols, frame.rows) != (expected_geometry.cols, expected_geometry.rows) {
        return Err(format!(
            "frame geometry {}x{} does not match {}x{}",
            frame.cols, frame.rows, expected_geometry.cols, expected_geometry.rows
        ));
    }
    if frame.to_json().as_bytes() != bytes {
        return Err("frame JSON is not canonical compact Frame v3".to_string());
    }
    Ok(frame)
}

fn validate_screen_provenance(frame: &tuiscotti::Frame, label: &str) -> Result<(), String> {
    let provenance = &frame.provenance;
    if provenance.tool != "tuiscotti"
        || provenance.profile != "default"
        || provenance.source != "screen"
        || provenance.tool_version.trim().is_empty()
        || !provenance.argv.is_empty()
        || provenance.created_unix != 0
    {
        return Err(format!(
            "{label} frame provenance is not a Tuiscotti screen observation"
        ));
    }
    Ok(())
}

fn opaque_png_info(bytes: &[u8], expected_size: (u32, u32)) -> Result<PngInfoReceipt, String> {
    let info = tuiscotti::formats::assert_opaque_rgb(bytes)
        .map_err(|error| format!("PNG is not opaque 8-bit RGB: {error}"))?;
    if (info.width, info.height) != expected_size {
        return Err(format!(
            "PNG dimensions {:?} do not match renderer geometry {:?}",
            (info.width, info.height),
            expected_size
        ));
    }
    Ok(PngInfoReceipt {
        width: info.width,
        height: info.height,
        color_type: info.color_type,
        bit_depth: info.bit_depth,
    })
}

pub(crate) fn validate_actual_observation(
    captured: &CapturedObservation,
    frame_artifact: &ArtifactReceipt,
    png_artifact: &ArtifactReceipt,
    geometry: &Geometry,
    renderer: &RendererIdentity,
) -> Result<ValidatedActualObservation, String> {
    let frame = parse_canonical_frame(&captured.frame_bytes, geometry)?;
    validate_screen_provenance(&frame, "actual")?;
    if captured.renderer_sha256 != renderer.hash {
        return Err("actual capture renderer hash differs from the resolved profile".to_string());
    }
    verify_artifact_bytes(frame_artifact, &captured.frame_bytes, "actual frame")?;
    verify_artifact_bytes(png_artifact, &captured.png_bytes, "actual PNG")?;
    let strict_profile = tuiscotti::profile::RenderProfile::vendored();
    let expected_size = strict_profile.image_size(geometry.cols, geometry.rows);
    let png_info = opaque_png_info(&captured.png_bytes, expected_size)?;
    opaque_png_info(&captured.rerender_png_bytes, expected_size)?;
    let pixel_verdict = tuiscotti::diff::compare_png_with_alpha(
        &captured.rerender_png_bytes,
        &captured.png_bytes,
        tuiscotti::diff::AlphaPolicy::Opaque,
    )
    .map_err(|error| format!("compare actual strict render to capture PNG: {error}"))?;
    if captured.fidelity_approximate {
        return Err("actual strict render reports approximate fidelity".to_string());
    }
    if !pixel_verdict.dims_equal || !pixel_verdict.pixels_equal {
        return Err("actual strict rerender does not match its captured PNG pixels".to_string());
    }
    let fidelity = fidelity_binding(
        &captured.fidelity_bytes,
        &captured.frame_bytes,
        &captured.renderer_sha256,
        &captured.rerender_png_bytes,
        captured.fidelity_approximate,
        true,
    );
    Ok(ValidatedActualObservation {
        frame,
        frame_ref: artifact_ref(frame_artifact),
        png_ref: artifact_ref(png_artifact),
        png_bytes: captured.png_bytes.clone(),
        png_info,
        fidelity,
    })
}

pub(crate) struct ValidatedActualObservation {
    frame: tuiscotti::Frame,
    frame_ref: ArtifactRef,
    png_ref: ArtifactRef,
    png_bytes: Vec<u8>,
    png_info: PngInfoReceipt,
    fidelity: FidelityBinding,
}

pub(crate) fn compare_admitted_observations(
    generation: &ExpectedGeneration,
    checkpoint_id: &str,
    actual: ValidatedActualObservation,
    actual_renderer: &RendererIdentity,
    trust_record_sha256: &str,
    admission_receipt_sha256: &str,
) -> Result<VisualComparisonEvidence, String> {
    let expected = generation
        .checkpoints
        .get(checkpoint_id)
        .ok_or_else(|| format!("expected generation lacks checkpoint {checkpoint_id}"))?;
    require_sha256_lower("verified trust-record SHA-256", trust_record_sha256)?;
    require_sha256_lower("admission receipt SHA-256", admission_receipt_sha256)?;

    let strict_profile = tuiscotti::profile::RenderProfile::vendored();
    let mut strict_renderer = tuiscotti::render::Renderer::for_render_profile(&strict_profile)
        .map_err(|error| format!("initialize expected strict renderer: {error}"))?;
    let rendered = strict_renderer
        .render(&expected.frame)
        .map_err(|error| format!("strictly render expected checkpoint {checkpoint_id}: {error}"))?;
    let fidelity_bytes = rendered.fidelity.to_json().into_bytes();
    let expected_size = strict_profile.image_size(
        expected.metadata.geometry.cols,
        expected.metadata.geometry.rows,
    );
    opaque_png_info(&rendered.png, expected_size)?;
    let expected_pixels = tuiscotti::diff::compare_png_with_alpha(
        &rendered.png,
        &expected.png_bytes,
        tuiscotti::diff::AlphaPolicy::Opaque,
    )
    .map_err(|error| format!("bind expected strict rendering to admitted PNG: {error}"))?;
    if rendered.fidelity.approximate {
        return Err(format!(
            "expected checkpoint {checkpoint_id} has approximate strict fidelity"
        ));
    }
    if !expected_pixels.dims_equal || !expected_pixels.pixels_equal {
        return Err(format!(
            "expected strict rendering differs from admitted PNG at {checkpoint_id}"
        ));
    }
    let expected_fidelity = fidelity_binding(
        &fidelity_bytes,
        &expected.frame_bytes,
        &generation.renderer_sha256,
        &rendered.png,
        rendered.fidelity.approximate,
        true,
    );

    let renderer_equal = generation.renderer_sha256 == actual_renderer.hash;
    let (expected_frame_ref, expected_png_ref) = generation.checkpoint_refs(checkpoint_id)?;
    let frame_comparison = compare_frames(
        &expected.frame,
        &actual.frame,
        expected_frame_ref,
        actual.frame_ref,
        expected.metadata.geometry.clone(),
        checkpoint_id,
    )?;

    let png_verdict = tuiscotti::diff::compare_png_with_alpha(
        &expected.png_bytes,
        &actual.png_bytes,
        tuiscotti::diff::AlphaPolicy::Opaque,
    )
    .map_err(|error| format!("compare expected/actual PNG at {checkpoint_id}: {error}"))?;
    let png_equal = png_verdict.dims_equal && png_verdict.pixels_equal;
    let png_comparison = PngComparison {
        expected: expected_png_ref,
        actual: actual.png_ref,
        expected_info: expected.png_info.clone(),
        actual_info: actual.png_info,
        alpha_policy: "opaque".to_string(),
        dimensions_equal: png_verdict.dims_equal,
        pixels_equal: png_verdict.pixels_equal,
        equal: png_equal,
    };
    Ok(VisualComparisonEvidence {
        schema: "termrock-spec/parity-visual-comparison-v1".to_string(),
        method: "frame-v3-diff-cells+opaque-rgb-decoded-exact-v1".to_string(),
        admission: VisualAdmission {
            verified: true,
            trust_record_sha256: trust_record_sha256.to_string(),
            expected_generation_id: generation.id.clone(),
            expected_tree_sha256: generation.tree_sha256.clone(),
            expected_manifest_sha256: generation.manifest_sha256.clone(),
            admission_receipt_sha256: admission_receipt_sha256.to_string(),
        },
        renderer: RendererComparison {
            expected_sha256: generation.renderer_sha256.clone(),
            actual_sha256: actual_renderer.hash.clone(),
            equal: renderer_equal,
        },
        frame: frame_comparison,
        png: png_comparison,
        fidelity: FidelityComparison {
            expected: expected_fidelity,
            actual: actual.fidelity,
        },
    })
}

fn verify_artifact_bytes(
    artifact: &ArtifactReceipt,
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    if artifact.bytes != bytes.len() as u64 || artifact.sha256 != crate::sha256_bytes(bytes) {
        return Err(format!(
            "{label} bytes do not match published artifact receipt"
        ));
    }
    Ok(())
}

fn compare_frames(
    expected: &tuiscotti::Frame,
    actual: &tuiscotti::Frame,
    expected_ref: ArtifactRef,
    actual_ref: ArtifactRef,
    expected_geometry: Geometry,
    checkpoint_id: &str,
) -> Result<FrameComparison, String> {
    let dimensions_equal = (expected.cols, expected.rows) == (actual.cols, actual.rows);
    let mut without_cursor = expected.clone();
    without_cursor.cursor = actual.cursor;
    let cells_equal = without_cursor
        .diff_cells(actual)
        .map_err(|error| format!("compare frame cells at {checkpoint_id}: {error}"))?
        .is_empty();
    let cursor_equal = expected.cursor == actual.cursor;
    let differing_positions = expected
        .diff_cells(actual)
        .map_err(|error| format!("compare frame dimensions at {checkpoint_id}: {error}"))?
        .into_iter()
        .map(|(x, y)| DifferingPosition { x, y })
        .collect::<Vec<_>>();
    let equal = dimensions_equal && cells_equal && cursor_equal;
    Ok(FrameComparison {
        expected: expected_ref,
        actual: actual_ref,
        version: FRAME_VERSION,
        expected_geometry,
        actual_geometry: Geometry {
            cols: actual.cols,
            rows: actual.rows,
        },
        dimensions_equal,
        cells_equal,
        cursor_equal,
        differing_positions,
        equal,
    })
}

fn artifact_ref(artifact: &ArtifactReceipt) -> ArtifactRef {
    ArtifactRef {
        path: artifact.path.display().to_string(),
        sha256: artifact.sha256.clone(),
        bytes: artifact.bytes,
    }
}

fn expected_file_ref(root: &Path, relative: &str, sha256: &str, bytes: u64) -> ArtifactRef {
    ArtifactRef {
        path: root.join(relative).display().to_string(),
        sha256: sha256.to_string(),
        bytes,
    }
}

fn fidelity_binding(
    fidelity_bytes: &[u8],
    frame_bytes: &[u8],
    renderer_sha256: &str,
    rerender_png: &[u8],
    approximate: bool,
    rerender_matches_bound_png: bool,
) -> FidelityBinding {
    FidelityBinding {
        sha256: crate::sha256_bytes(fidelity_bytes),
        bytes: fidelity_bytes.len() as u64,
        source_frame_sha256: crate::sha256_bytes(frame_bytes),
        renderer_sha256: renderer_sha256.to_string(),
        rerender_png_sha256: crate::sha256_bytes(rerender_png),
        approximate,
        rerender_matches_bound_png,
    }
}

#[cfg(test)]
pub(crate) struct SyntheticExpectedGenerationFixture {
    pub(crate) root: PathBuf,
    pub(crate) input: ExpectedGenerationInput,
}

#[cfg(test)]
pub(crate) struct SyntheticCaptureFixture {
    pub(crate) captured: CapturedObservation,
    pub(crate) frame_artifact: ArtifactReceipt,
    pub(crate) png_artifact: ArtifactReceipt,
    pub(crate) fidelity_artifact: ArtifactReceipt,
}

#[cfg(test)]
impl Drop for SyntheticExpectedGenerationFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
pub(crate) fn synthetic_expected_generation_fixture() -> SyntheticExpectedGenerationFixture {
    use tuiscotti::{Frame, Provenance};

    let root = fs::canonicalize(std::env::temp_dir())
        .expect("canonicalize test temporary root")
        .join(format!(
            "termrock-e2e-expected-generation-{}-{}",
            std::process::id(),
            crate::CAPTURE_DIRECTORY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
    fs::create_dir(&root).expect("create synthetic expected-generation root");

    let case = crate::registry()
        .expect("load synthetic expected-generation case")
        .cases
        .into_iter()
        .find(|case| case.id == "HELP-HOLLA-004")
        .expect("find Holla pilot case");
    let geometry = case.geometry.clone();
    let frame = Frame::blank(
        geometry.cols,
        geometry.rows,
        Provenance {
            tool: "tuiscotti".to_string(),
            tool_version: "synthetic-fixture".to_string(),
            profile: "default".to_string(),
            source: "screen".to_string(),
            argv: Vec::new(),
            created_unix: 0,
        },
    );
    let profile = tuiscotti::profile::RenderProfile::vendored();
    let mut renderer = tuiscotti::render::Renderer::for_render_profile(&profile)
        .expect("create synthetic fixture renderer");
    let rendered = renderer
        .render(&frame)
        .expect("render synthetic fixture frame");
    let png_bytes = rendered.png.clone();
    let png_info = opaque_png_info(&png_bytes, profile.image_size(geometry.cols, geometry.rows))
        .expect("synthetic fixture renders opaque RGB");
    let frame_bytes = frame.to_json().into_bytes();
    let frame_sha256 = crate::sha256_bytes(&frame_bytes);
    let png_sha256 = crate::sha256_bytes(&png_bytes);
    let checkpoint_entries = CHECKPOINT_IDS
        .iter()
        .map(|id| {
            let frame_path = format!("checkpoints/{id}/frame.json");
            let png_path = format!("checkpoints/{id}/screen.png");
            let directory = root.join("checkpoints").join(id);
            fs::create_dir_all(&directory).expect("create synthetic checkpoint directory");
            fs::write(root.join(&frame_path), &frame_bytes)
                .expect("write synthetic canonical frame");
            fs::write(root.join(&png_path), &png_bytes).expect("write synthetic rendered PNG");
            ExpectedCheckpointManifest {
                id: (*id).to_string(),
                observation_id: format!("synthetic-{id}"),
                geometry: geometry.clone(),
                color_path: case.color_path.clone(),
                frame: ExpectedFrameFile {
                    path: frame_path,
                    sha256: frame_sha256.clone(),
                    bytes: frame_bytes.len() as u64,
                },
                png: ExpectedPngFile {
                    path: png_path,
                    sha256: png_sha256.clone(),
                    bytes: png_bytes.len() as u64,
                    width: png_info.width,
                    height: png_info.height,
                    color_type: png_info.color_type,
                    bit_depth: png_info.bit_depth,
                },
            }
        })
        .collect::<Vec<_>>();
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_bytes = serde_json::to_vec(&case).expect("serialize selected case fixture");
    let renderer_identity = crate::renderer_identity(&profile);
    let manifest = ExpectedGenerationManifest {
        schema: EXPECTED_GENERATION_SCHEMA.to_string(),
        id: "synthetic-not-admitted".to_string(),
        case_id: case.id,
        case_set_digest: crate::digest_tree(&package_root.join("cases"))
            .expect("hash case registry fixture"),
        profile_digest: crate::sha256_file(&package_root.join("profile.json"))
            .expect("hash suite profile fixture"),
        dependency_lock_sha256: crate::sha256_file(&package_root.join("Cargo.lock"))
            .expect("hash dependency lock fixture"),
        case_input_sha256: crate::sha256_bytes(&case_bytes),
        tuiscotti_revision: TUISCOTTI_REVISION.to_string(),
        tree_hash_algorithm: EXPECTED_TREE_ALGORITHM.to_string(),
        oracle: OracleLineage {
            tag_ref: "refs/tags/visual-baseline".to_string(),
            tag_object: "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5".to_string(),
            tag_commit: "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b".to_string(),
            capture_run_sha256: "1".repeat(64),
            source_inputs_sha256: "2".repeat(64),
            build_environment_sha256: "3".repeat(64),
            builder_receipt_sha256: "4".repeat(64),
            oracle_executable_sha256: "5".repeat(64),
        },
        renderer: renderer_identity,
        checkpoints: checkpoint_entries,
    };
    let manifest_bytes = serde_json::to_vec(&manifest).expect("serialize synthetic manifest");
    fs::write(root.join("expected-generation.json"), manifest_bytes)
        .expect("write synthetic expected-generation manifest");
    let files = read_exact_generation_tree(&root).expect("read exact synthetic generation tree");
    let tree_sha256 = expected_tree_sha256(&files).expect("hash synthetic generation tree");

    SyntheticExpectedGenerationFixture {
        root: root.clone(),
        input: ExpectedGenerationInput {
            id: "synthetic-not-admitted".to_string(),
            root: Some(root),
            sha256: tree_sha256,
        },
    }
}

#[cfg(test)]
pub(crate) fn synthetic_capture_fixture(
    frame: &tuiscotti::Frame,
    role: &str,
    checkpoint: &str,
) -> SyntheticCaptureFixture {
    let profile = tuiscotti::profile::RenderProfile::vendored();
    let renderer_identity = crate::renderer_identity(&profile);
    let mut renderer = tuiscotti::render::Renderer::for_render_profile(&profile)
        .expect("create synthetic capture renderer");
    let bundle = tuiscotti::formats::capture_all(
        &mut renderer,
        frame,
        &format!("synthetic-not-product:{checkpoint}"),
    )
    .expect("capture synthetic observation");
    let rendered = renderer
        .render(frame)
        .expect("strictly rerender synthetic observation");
    let frame_bytes = bundle.json.into_bytes();
    let png_bytes = bundle.png;
    let fidelity_bytes = rendered.fidelity.to_json().into_bytes();
    let case_id = "HELP-HOLLA-004";
    let base = std::env::temp_dir()
        .join("termrock-e2e-synthetic-not-product")
        .join(role)
        .join(checkpoint);
    let frame_artifact = ArtifactReceipt {
        subject_role: role.to_string(),
        case_id: case_id.to_string(),
        checkpoint_id: checkpoint.to_string(),
        format: "frame_json".to_string(),
        path: base.join("frame.json"),
        sha256: crate::sha256_bytes(&frame_bytes),
        bytes: frame_bytes.len() as u64,
    };
    let png_artifact = ArtifactReceipt {
        subject_role: role.to_string(),
        case_id: case_id.to_string(),
        checkpoint_id: checkpoint.to_string(),
        format: "png".to_string(),
        path: base.join("screen.png"),
        sha256: crate::sha256_bytes(&png_bytes),
        bytes: png_bytes.len() as u64,
    };
    let fidelity_artifact = ArtifactReceipt {
        subject_role: role.to_string(),
        case_id: case_id.to_string(),
        checkpoint_id: checkpoint.to_string(),
        format: "png_fidelity_json".to_string(),
        path: base.join("png.fidelity.json"),
        sha256: crate::sha256_bytes(&fidelity_bytes),
        bytes: fidelity_bytes.len() as u64,
    };
    SyntheticCaptureFixture {
        captured: CapturedObservation {
            frame_bytes,
            png_bytes,
            rerender_png_bytes: rendered.png,
            fidelity_bytes,
            fidelity_approximate: rendered.fidelity.approximate,
            renderer_sha256: renderer_identity.hash,
        },
        frame_artifact,
        png_artifact,
        fidelity_artifact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuiscotti::{Cell, Cursor, CursorStyle, Frame, Provenance};

    fn holla_case() -> Case {
        crate::registry()
            .expect("load Holla registry case")
            .cases
            .into_iter()
            .find(|case| case.id == "HELP-HOLLA-004")
            .expect("find Holla pilot case")
    }

    fn load_synthetic_generation(
        fixture: &SyntheticExpectedGenerationFixture,
    ) -> Result<ExpectedGeneration, String> {
        load_synthetic_generation_with_input(&fixture.input)
    }

    fn load_synthetic_generation_with_input(
        input: &ExpectedGenerationInput,
    ) -> Result<ExpectedGeneration, String> {
        let case = holla_case();
        let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let profile = tuiscotti::profile::RenderProfile::vendored();
        let renderer = crate::renderer_identity(&profile);
        load_expected_generation(
            input,
            &case,
            &crate::digest_tree(&package_root.join("cases"))?,
            &crate::sha256_file(&package_root.join("profile.json"))?,
            &crate::sha256_file(&package_root.join("Cargo.lock"))?,
            TUISCOTTI_REVISION,
            &renderer,
        )
    }

    fn rewrite_synthetic_manifest(
        fixture: &mut SyntheticExpectedGenerationFixture,
        edit: impl FnOnce(&mut ExpectedGenerationManifest),
    ) {
        let path = fixture.root.join("expected-generation.json");
        let bytes = fs::read(&path).expect("read synthetic expected manifest");
        let mut manifest: ExpectedGenerationManifest =
            serde_json::from_slice(&bytes).expect("parse synthetic expected manifest");
        edit(&mut manifest);
        let bytes = serde_json::to_vec(&manifest).expect("serialize synthetic expected manifest");
        fs::write(path, bytes).expect("rewrite synthetic expected manifest");
        let files = read_exact_generation_tree(&fixture.root)
            .expect("read rewritten synthetic expected tree");
        fixture.input.sha256 =
            expected_tree_sha256(&files).expect("rehash synthetic expected tree");
    }

    fn screen_frame() -> Frame {
        Frame::blank(
            120,
            40,
            Provenance {
                tool: "tuiscotti".to_string(),
                tool_version: "test-version".to_string(),
                profile: "default".to_string(),
                source: "screen".to_string(),
                argv: Vec::new(),
                created_unix: 0,
            },
        )
    }

    fn artifact_ref(path: &str) -> ArtifactRef {
        ArtifactRef {
            path: path.to_string(),
            sha256: "a".repeat(64),
            bytes: 1,
        }
    }

    fn bytes_from_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let digit = |byte| match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    _ => panic!("invalid fixture hex"),
                };
                digit(pair[0]) * 16 + digit(pair[1])
            })
            .collect()
    }

    #[test]
    fn canonical_frame_parser_rejects_noncanonical_and_unexpected_data() {
        let frame = screen_frame();
        let canonical = frame.to_json().into_bytes();
        let geometry = Geometry {
            cols: 120,
            rows: 40,
        };
        assert!(parse_canonical_frame(&canonical, &geometry).is_ok());

        let mut whitespace = canonical.clone();
        whitespace.push(b' ');
        assert!(parse_canonical_frame(&whitespace, &geometry).is_err());

        let mut unknown: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        unknown["unrecognized"] = serde_json::json!(true);
        let unknown = serde_json::to_vec(&unknown).unwrap();
        assert!(parse_canonical_frame(&unknown, &geometry).is_err());

        let duplicate = String::from_utf8(canonical).unwrap().replacen(
            "\"version\":3",
            "\"version\":3,\"version\":3",
            1,
        );
        assert!(parse_canonical_frame(duplicate.as_bytes(), &geometry).is_err());
    }

    #[test]
    fn frame_comparison_keeps_cells_and_cursor_as_separate_exact_checks() {
        let expected = screen_frame();
        let mut cursor_changed = expected.clone();
        cursor_changed.cursor = Cursor {
            x: 3,
            y: 2,
            visible: true,
            style: CursorStyle::Block,
            blinking: false,
        };
        let cursor_result = compare_frames(
            &expected,
            &cursor_changed,
            artifact_ref("expected/frame.json"),
            artifact_ref("actual/frame.json"),
            Geometry {
                cols: 120,
                rows: 40,
            },
            "cursor-test",
        )
        .unwrap();
        assert!(cursor_result.cells_equal);
        assert!(!cursor_result.cursor_equal);
        assert!(!cursor_result.equal);
        assert_eq!(cursor_result.differing_positions.len(), 1);

        let mut cell_changed = expected.clone();
        cell_changed.set(Cell {
            x: 3,
            y: 2,
            symbol: "x".to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: tuiscotti::Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        let cell_result = compare_frames(
            &expected,
            &cell_changed,
            artifact_ref("expected/frame.json"),
            artifact_ref("actual/frame.json"),
            Geometry {
                cols: 120,
                rows: 40,
            },
            "cell-test",
        )
        .unwrap();
        assert!(!cell_result.cells_equal);
        assert!(cell_result.cursor_equal);
        assert!(!cell_result.equal);
        assert_eq!(cell_result.differing_positions.len(), 1);
    }

    #[test]
    fn frame_dimension_change_is_not_treated_as_a_cell_match() {
        let expected = screen_frame();
        let smaller = Frame::blank(
            119,
            40,
            Provenance {
                tool: "tuiscotti".to_string(),
                tool_version: "test-version".to_string(),
                profile: "default".to_string(),
                source: "screen".to_string(),
                argv: Vec::new(),
                created_unix: 0,
            },
        );
        assert!(
            compare_frames(
                &expected,
                &smaller,
                artifact_ref("expected/frame.json"),
                artifact_ref("actual/frame.json"),
                Geometry {
                    cols: 120,
                    rows: 40,
                },
                "dimension-test",
            )
            .is_err()
        );
    }

    #[test]
    fn opaque_rgb_comparison_uses_decoded_pixels_not_png_bytes() {
        const RED_COMPRESSED: &str = "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de0000000c49444154789c63f8cfc0000003010100c9fe92ef0000000049454e44ae426082";
        const RED_STORED: &str = "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de0000000f494441547801010400fbff00ff0000030101008d1de5820000000049454e44ae426082";
        const BLUE: &str = "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de0000000c49444154789c636060f80f00010301000889c2ec0000000049454e44ae426082";
        let red_compressed = bytes_from_hex(RED_COMPRESSED);
        let red_stored = bytes_from_hex(RED_STORED);
        let blue = bytes_from_hex(BLUE);
        assert_ne!(red_compressed, red_stored);
        assert!(opaque_png_info(&red_compressed, (1, 1)).is_ok());
        assert!(opaque_png_info(&red_stored, (1, 1)).is_ok());

        let same_pixels = tuiscotti::diff::compare_png_with_alpha(
            &red_compressed,
            &red_stored,
            tuiscotti::diff::AlphaPolicy::Opaque,
        )
        .expect("decode same-color PNG fixtures");
        assert!(same_pixels.dims_equal);
        assert!(same_pixels.pixels_equal);

        let different_pixel = tuiscotti::diff::compare_png_with_alpha(
            &red_compressed,
            &blue,
            tuiscotti::diff::AlphaPolicy::Opaque,
        )
        .expect("decode different-color PNG fixtures");
        assert!(different_pixel.dims_equal);
        assert!(!different_pixel.pixels_equal);
    }

    #[test]
    fn synthetic_expected_generation_loader_accepts_exact_tree_and_rejects_tampering() {
        let fixture = synthetic_expected_generation_fixture();
        let loaded = load_synthetic_generation(&fixture).expect("load exact synthetic tree");
        assert_eq!(loaded.id, "synthetic-not-admitted");
        assert_eq!(loaded.checkpoints.len(), CHECKPOINT_IDS.len());
        assert_eq!(
            loaded
                .checkpoints
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            CHECKPOINT_IDS
        );

        let mut stale_tree_digest = fixture.input.clone();
        stale_tree_digest.sha256 = "0".repeat(64);
        let stale_result = load_synthetic_generation_with_input(&stale_tree_digest);
        assert!(
            stale_result
                .unwrap_err()
                .contains("expected generation digest mismatch")
        );

        let mut altered = synthetic_expected_generation_fixture();
        let altered_file = altered.root.join("checkpoints/00-boot/frame.json");
        let mut bytes = fs::read(&altered_file).expect("read synthetic frame file");
        bytes.push(b' ');
        fs::write(&altered_file, bytes).expect("alter synthetic frame file");
        let altered_files = read_exact_generation_tree(&altered.root)
            .expect("read altered synthetic expected tree");
        altered.input.sha256 =
            expected_tree_sha256(&altered_files).expect("rehash altered synthetic tree");
        let hash_error = load_synthetic_generation(&altered).unwrap_err();
        assert!(hash_error.contains("byte count does not match manifest"));

        let extra = synthetic_expected_generation_fixture();
        fs::write(extra.root.join("unexpected.txt"), b"extra")
            .expect("add unexpected synthetic tree member");
        assert!(
            load_synthetic_generation(&extra)
                .unwrap_err()
                .contains("exactly the nine fixed files")
        );
    }

    #[test]
    fn synthetic_expected_generation_rejects_manifest_path_checkpoint_and_profile_changes() {
        let mut bad_path = synthetic_expected_generation_fixture();
        rewrite_synthetic_manifest(&mut bad_path, |manifest| {
            manifest.checkpoints[0].frame.path = "checkpoints/00-boot/other.json".to_string();
        });
        assert!(
            load_synthetic_generation(&bad_path)
                .unwrap_err()
                .contains("paths are not fixed")
        );

        let mut missing_checkpoint = synthetic_expected_generation_fixture();
        rewrite_synthetic_manifest(&mut missing_checkpoint, |manifest| {
            manifest.checkpoints.pop();
        });
        assert!(
            load_synthetic_generation(&missing_checkpoint)
                .unwrap_err()
                .contains("exactly four checkpoints")
        );

        let mut bad_profile = synthetic_expected_generation_fixture();
        rewrite_synthetic_manifest(&mut bad_profile, |manifest| {
            manifest.profile_digest = "0".repeat(64);
        });
        assert!(
            load_synthetic_generation(&bad_profile)
                .unwrap_err()
                .contains("profile_digest does not match current suite")
        );

        let mut bad_renderer = synthetic_expected_generation_fixture();
        rewrite_synthetic_manifest(&mut bad_renderer, |manifest| {
            manifest.renderer.hash = "0".repeat(64);
        });
        assert!(
            load_synthetic_generation(&bad_renderer)
                .unwrap_err()
                .contains("renderer identity does not match vendored renderer")
        );
    }

    #[cfg(unix)]
    #[test]
    fn synthetic_expected_generation_rejects_symlink_members() {
        use std::os::unix::fs::symlink;

        let fixture = synthetic_expected_generation_fixture();
        let checkpoint = fixture.root.join("checkpoints/00-boot");
        let frame = checkpoint.join("frame.json");
        let target = checkpoint.join("frame-target.json");
        fs::rename(&frame, &target).expect("move synthetic frame target");
        symlink(&target, &frame).expect("replace frame with symlink");
        assert!(
            load_synthetic_generation(&fixture)
                .unwrap_err()
                .contains("contains a symlink")
        );
    }

    #[test]
    fn synthetic_observation_comparison_checks_admitted_frame_cursor_and_pixels_exactly() {
        let fixture = synthetic_expected_generation_fixture();
        let generation = load_synthetic_generation(&fixture).expect("load synthetic generation");
        let case = holla_case();
        let checkpoint = generation
            .checkpoints
            .get("00-boot")
            .expect("synthetic boot checkpoint");
        let renderer = crate::renderer_identity(&tuiscotti::profile::RenderProfile::vendored());

        let matching = synthetic_capture_fixture(&checkpoint.frame, "reference", "00-boot");
        let validated = validate_actual_observation(
            &matching.captured,
            &matching.frame_artifact,
            &matching.png_artifact,
            &case.geometry,
            &renderer,
        )
        .expect("validate synthetic capture from one frame observation");
        let comparison = compare_admitted_observations(
            &generation,
            "00-boot",
            validated,
            &renderer,
            &"a".repeat(64),
            &"b".repeat(64),
        )
        .expect("compare exact synthetic observations");
        assert!(comparison.admission.verified);
        assert!(comparison.renderer.equal);
        assert!(comparison.frame.equal);
        assert!(comparison.png.equal);
        assert!(!comparison.fidelity.expected.approximate);
        assert!(!comparison.fidelity.actual.approximate);
        assert!(comparison.fidelity.expected.rerender_matches_bound_png);
        assert!(comparison.fidelity.actual.rerender_matches_bound_png);

        let mut changed_cell = checkpoint.frame.clone();
        changed_cell.set(Cell {
            x: 0,
            y: 0,
            symbol: "X".to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: tuiscotti::Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        let changed = synthetic_capture_fixture(&changed_cell, "candidate", "00-boot");
        let changed_validated = validate_actual_observation(
            &changed.captured,
            &changed.frame_artifact,
            &changed.png_artifact,
            &case.geometry,
            &renderer,
        )
        .expect("changed synthetic observation remains internally valid");
        let changed_comparison = compare_admitted_observations(
            &generation,
            "00-boot",
            changed_validated,
            &renderer,
            &"a".repeat(64),
            &"b".repeat(64),
        )
        .expect("different observations produce an exact unequal verdict");
        assert!(!changed_comparison.frame.cells_equal);
        assert!(!changed_comparison.frame.equal);
        assert!(!changed_comparison.png.pixels_equal);
        assert!(!changed_comparison.png.equal);

        let mut changed_cursor = checkpoint.frame.clone();
        changed_cursor.cursor = Cursor {
            x: 1,
            y: 1,
            visible: true,
            style: CursorStyle::Block,
            blinking: false,
        };
        let cursor_capture = synthetic_capture_fixture(&changed_cursor, "candidate", "00-boot");
        let cursor_validated = validate_actual_observation(
            &cursor_capture.captured,
            &cursor_capture.frame_artifact,
            &cursor_capture.png_artifact,
            &case.geometry,
            &renderer,
        )
        .expect("cursor-only synthetic observation remains internally valid");
        let cursor_comparison = compare_admitted_observations(
            &generation,
            "00-boot",
            cursor_validated,
            &renderer,
            &"a".repeat(64),
            &"b".repeat(64),
        )
        .expect("cursor delta produces exact frame verdict");
        assert!(cursor_comparison.frame.cells_equal);
        assert!(!cursor_comparison.frame.cursor_equal);
        assert!(!cursor_comparison.frame.equal);
    }

    #[test]
    fn synthetic_actual_observation_rejects_approximate_or_unbound_fidelity() {
        let geometry = Geometry {
            cols: 120,
            rows: 40,
        };
        let renderer = crate::renderer_identity(&tuiscotti::profile::RenderProfile::vendored());
        let frame = screen_frame();

        let mut approximate = synthetic_capture_fixture(&frame, "reference", "00-boot");
        approximate.captured.fidelity_approximate = true;
        assert!(
            validate_actual_observation(
                &approximate.captured,
                &approximate.frame_artifact,
                &approximate.png_artifact,
                &geometry,
                &renderer,
            )
            .err()
            .expect("reject approximate fidelity")
            .contains("approximate fidelity")
        );

        let mut changed_frame = frame.clone();
        changed_frame.set(Cell {
            x: 0,
            y: 0,
            symbol: "X".to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: tuiscotti::Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        let different_render = synthetic_capture_fixture(&changed_frame, "candidate", "00-boot");
        let mut unbound = synthetic_capture_fixture(&frame, "candidate", "00-boot");
        unbound.captured.rerender_png_bytes = different_render.captured.png_bytes;
        assert!(
            validate_actual_observation(
                &unbound.captured,
                &unbound.frame_artifact,
                &unbound.png_artifact,
                &geometry,
                &renderer,
            )
            .err()
            .expect("reject rerender not bound to captured PNG")
            .contains("strict rerender does not match")
        );
    }
}
