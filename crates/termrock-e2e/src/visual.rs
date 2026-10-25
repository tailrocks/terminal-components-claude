use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Case, ExpectedGenerationInput, Geometry, RendererIdentity};

pub(crate) const EXPECTED_GENERATION_SCHEMA: &str = "termrock-spec/parity-expected-generation-v1";
const EXPECTED_TREE_ALGORITHM: &str = "termrock-e2e/expected-tree-v1";
const TUISCOTTI_REVISION: &str = "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274";
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

    let (files, directories) = read_exact_generation_tree(&root)?;
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
    validate_generation_tree_layout(&files, &directories, &manifest.checkpoints)?;
    let tree_sha256 = expected_tree_sha256(&files)?;
    if tree_sha256 != expected.sha256 {
        return Err(format!(
            "expected generation digest mismatch: expected {}, got {tree_sha256}",
            expected.sha256
        ));
    }

    let strict_profile = tuiscotti::profile::RenderProfile::vendored();
    let mut checkpoints = BTreeMap::new();
    for entry in &manifest.checkpoints {
        let expected_geometry = &entry.geometry;
        let expected_png_size =
            strict_profile.image_size(expected_geometry.cols, expected_geometry.rows);
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
        let frame = parse_canonical_frame(&frame_bytes, expected_geometry)?;
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
    crate::validate_geometry(&case.geometry)?;
    let case_checkpoints = crate::checkpoints(case);
    if case_checkpoints.is_empty() {
        return Err("expected-generation case has no checkpoints".to_string());
    }
    if manifest.checkpoints.len() != case_checkpoints.len() {
        return Err(format!(
            "expected-generation has {} checkpoints; case requires {}",
            manifest.checkpoints.len(),
            case_checkpoints.len()
        ));
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
    for (entry, id) in manifest.checkpoints.iter().zip(case_checkpoints) {
        let expected_geometry = crate::checkpoint_geometry(case, id)?;
        crate::validate_geometry(&entry.geometry)?;
        if entry.id != id
            || entry.observation_id.trim().is_empty()
            || !observation_ids.insert(entry.observation_id.as_str())
            || entry.geometry != expected_geometry
            || entry.color_path != case.color_path
        {
            return Err(format!(
                "expected-generation checkpoint metadata does not match the selected case for {id}"
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
        let pixel_size = tuiscotti::profile::RenderProfile::vendored()
            .image_size(expected_geometry.cols, expected_geometry.rows);
        if (entry.png.width, entry.png.height) != pixel_size {
            return Err(format!(
                "expected PNG dimensions do not match geometry for {id}"
            ));
        }
    }
    Ok(())
}

fn read_exact_generation_tree(
    root: &Path,
) -> Result<(BTreeMap<String, Vec<u8>>, BTreeSet<String>), String> {
    let mut files = BTreeMap::new();
    let mut directories = BTreeSet::new();
    walk_generation_tree(root, root, &mut files, &mut directories)?;
    let checkpoint_ids = directories
        .iter()
        .filter_map(|path| path.strip_prefix("checkpoints/"))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if checkpoint_ids.is_empty()
        || checkpoint_ids
            .iter()
            .any(|id| !crate::valid_checkpoint_id(id))
    {
        return Err("expected-generation tree has invalid checkpoint directories".to_string());
    }
    let (expected_files, expected_directories) = generation_tree_paths(&checkpoint_ids);
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected_files {
        return Err("expected-generation tree has missing or unexpected files".to_string());
    }
    if directories != expected_directories {
        return Err(
            "expected-generation tree contains a missing or unexpected directory".to_string(),
        );
    }
    Ok((files, directories))
}

fn validate_generation_tree_layout(
    files: &BTreeMap<String, Vec<u8>>,
    directories: &BTreeSet<String>,
    checkpoints: &[ExpectedCheckpointManifest],
) -> Result<(), String> {
    let checkpoint_ids = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.id.clone())
        .collect::<Vec<_>>();
    if checkpoint_ids
        .iter()
        .any(|id| !crate::valid_checkpoint_id(id))
        || checkpoint_ids.iter().collect::<BTreeSet<_>>().len() != checkpoint_ids.len()
    {
        return Err("expected-generation has invalid or duplicate checkpoint IDs".to_string());
    }
    let (expected_files, expected_directories) = generation_tree_paths(&checkpoint_ids);
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected_files
        || directories != &expected_directories
    {
        return Err("expected-generation tree does not match its manifest checkpoints".to_string());
    }
    Ok(())
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

fn generation_tree_paths(checkpoint_ids: &[String]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut files = BTreeSet::from(["expected-generation.json".to_string()]);
    let mut directories = BTreeSet::from(["checkpoints".to_string()]);
    for checkpoint in checkpoint_ids {
        files.insert(format!("checkpoints/{checkpoint}/frame.json"));
        files.insert(format!("checkpoints/{checkpoint}/screen.png"));
        directories.insert(format!("checkpoints/{checkpoint}"));
    }
    (files, directories)
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

fn expected_file_ref(root: &Path, relative: &str, sha256: &str, bytes: u64) -> ArtifactRef {
    ArtifactRef {
        path: root.join(relative).display().to_string(),
        sha256: sha256.to_string(),
        bytes,
    }
}
