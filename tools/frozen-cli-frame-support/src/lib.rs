//! Runtime support for paired captures of the frozen CLI and a candidate CLI.
//!
//! Copy this file into the frozen test harness as a module. It intentionally
//! depends only on `std`, pinned `tuiscotti`, and `sha2`.

use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tuiscotti::grouped::{ArtifactSet, GroupedStore};
use tuiscotti::{Frame, Profile, VENDORED_FACES};

const PHASE_ENV: &str = "TERMROCK_PAIR_PHASE";
const EXPECTED_ROOT_ENV: &str = "TERMROCK_PAIR_EXPECTED_ROOT";
const RECEIPT_ROOT_ENV: &str = "TERMROCK_PAIR_RECEIPT_ROOT";

/// Runtime pairing errors. Display text carries the underlying failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// The only accepted runtime phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Record a new runtime frame after checking any existing frozen approval.
    BaselineCapture,
    /// Compare a replay of the frozen binary against captured runtime frames.
    BaselineCheck,
    /// Compare the candidate binary against the same captured runtime frames.
    CandidateCheck,
}

impl Phase {
    fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "baseline-capture" => Ok(Self::BaselineCapture),
            "baseline-check" => Ok(Self::BaselineCheck),
            "candidate-check" => Ok(Self::CandidateCheck),
            _ => Err(Error::new(format!("invalid {PHASE_ENV}: {value:?}"))),
        }
    }
}

/// Explicit configuration for one paired capture phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Selected phase.
    pub phase: Phase,
    /// Runtime ANSI expectation store, separate from frozen snapshots.
    pub expected_root: PathBuf,
    /// New per-checkpoint receipt store.
    pub receipt_root: PathBuf,
}

impl Config {
    /// Parse the all-or-none environment contract without mutating process state.
    pub fn from_values(
        phase: Option<&str>,
        expected_root: Option<&str>,
        receipt_root: Option<&str>,
    ) -> Result<Option<Self>, Error> {
        if phase.is_none() && expected_root.is_none() && receipt_root.is_none() {
            return Ok(None);
        }
        let (Some(phase), Some(expected_root), Some(receipt_root)) =
            (phase, expected_root, receipt_root)
        else {
            return Err(Error::new(format!(
                "{PHASE_ENV}, {EXPECTED_ROOT_ENV}, and {RECEIPT_ROOT_ENV} must be set together"
            )));
        };
        if expected_root.trim().is_empty() || receipt_root.trim().is_empty() {
            return Err(Error::new("paired capture roots must not be empty"));
        }
        Ok(Some(Self {
            phase: Phase::parse(phase)?,
            expected_root: PathBuf::from(expected_root),
            receipt_root: PathBuf::from(receipt_root),
        }))
    }

    fn from_env() -> Result<Option<Self>, Error> {
        let phase = env_value(PHASE_ENV)?;
        let expected_root = env_value(EXPECTED_ROOT_ENV)?;
        let receipt_root = env_value(RECEIPT_ROOT_ENV)?;
        Self::from_values(
            phase.as_deref(),
            expected_root.as_deref(),
            receipt_root.as_deref(),
        )
    }
}

fn env_value(name: &str) -> Result<Option<String>, Error> {
    std::env::var_os(name)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| Error::new(format!("{name} is not valid Unicode")))
        })
        .transpose()
}

#[derive(Debug)]
struct Recorder {
    config: Config,
    case_name: String,
    next_ordinal: u64,
}

impl Recorder {
    fn new(config: Config, case_name: &str) -> Result<Self, Error> {
        validate_logical_name(case_name)?;
        Ok(Self {
            config,
            case_name: case_name.to_string(),
            next_ordinal: 0,
        })
    }

    fn handle_frame(
        &mut self,
        name: &str,
        frame: &Frame,
        original_expected_root: &Path,
    ) -> Result<(), Error> {
        validate_logical_name(name)?;
        validate_roots(
            original_expected_root,
            &self.config.expected_root,
            &self.config.receipt_root,
        )?;
        let ordinal = self.next_ordinal;
        self.next_ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| Error::new("paired capture checkpoint ordinal overflow"))?;
        let store_name = checkpoint_name(&self.case_name, ordinal, name);
        let ansi = tuiscotti::render::ansi_dump_with_cursor(frame);
        let receipt_path = self.receipt_path(ordinal, name);
        validate_concrete_path(&self.config.receipt_root, &receipt_path, "receipt", true)?;
        if receipt_path.exists() {
            return Err(Error::new(format!(
                "paired capture receipt already exists: {}",
                receipt_path.display()
            )));
        }

        match self.config.phase {
            Phase::BaselineCapture => {
                let expected = self.config.expected_root.join(format!("{store_name}.ansi"));
                validate_concrete_path(
                    &self.config.expected_root,
                    &expected,
                    "runtime expectation",
                    true,
                )?;
                if expected.exists() {
                    return Err(Error::new(format!(
                        "paired capture expectation already exists: {}",
                        expected.display()
                    )));
                }
                check_original_if_present(
                    original_expected_root,
                    name,
                    frame,
                    &self.config.receipt_root,
                )?;
                create_new_bytes(&expected, ansi.as_bytes())?;
                self.check_runtime(&store_name, frame)?;
            }
            Phase::BaselineCheck | Phase::CandidateCheck => {
                self.check_runtime(&store_name, frame)?;
            }
        }

        self.write_receipt(ordinal, name, &ansi)
    }

    fn check_runtime(&self, name: &str, frame: &Frame) -> Result<(), Error> {
        let sidecar_root = artifact_root(&self.config.receipt_root, "runtime-expected");
        let actual_root = artifact_root(&self.config.receipt_root, "actual");
        let diff_root = artifact_root(&self.config.receipt_root, "diff");
        validate_grouped_paths(
            &self.config.expected_root,
            &sidecar_root,
            &actual_root,
            &diff_root,
            name,
        )?;
        let profile = Profile::default_profile();
        let mut renderer = profile
            .renderer(&VENDORED_FACES)
            .map_err(|error| Error::new(error.to_string()))?;
        let store = GroupedStore::new(&self.config.expected_root)
            .with_artifacts(ArtifactSet::AnsiCursor)
            .with_expected_root(&sidecar_root)
            .with_actual_root(&actual_root)
            .with_diff_root(&diff_root)
            .with_review_evidence(false);
        store
            .check_with(&mut renderer, name, frame, 1.0)
            .map_err(|error| Error::new(error.to_string()))?
            .ensure_matched()
            .map_err(|error| Error::new(error.to_string()))
    }

    fn write_receipt(&self, ordinal: u64, name: &str, ansi: &str) -> Result<(), Error> {
        let path = self.receipt_path(ordinal, name);
        validate_concrete_path(&self.config.receipt_root, &path, "receipt", true)?;
        let hash = Sha256::digest(ansi.as_bytes());
        let record = format!(
            "{{\n  \"case_id\": {},\n  \"ordinal\": {ordinal},\n  \"logical_name\": {},\n  \"sha256\": \"{}\"\n}}\n",
            json_string(&self.case_name),
            json_string(name),
            hex(&hash),
        );
        create_new_bytes(&path, record.as_bytes())
    }

    fn receipt_path(&self, ordinal: u64, name: &str) -> PathBuf {
        let case_id = case_directory(&self.case_name);
        let checkpoint = format!("{}.json", checkpoint_name(&self.case_name, ordinal, name));
        self.config.receipt_root.join(case_id).join(checkpoint)
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

/// Start a runtime case. Returns `false` when no paired-capture environment is set.
///
/// The runner calls this once per scenario, including scenarios whose visual
/// gate is triggered by resize.
pub fn begin_case(name: &str) -> Result<bool, Error> {
    let config = Config::from_env()?;
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        *active = match config {
            Some(config) => Some(Recorder::new(config, name)?),
            None => None,
        };
        Ok(active.is_some())
    })
}

/// Handle one central frozen-harness visual gate.
///
/// Returns `false` when runtime pairing is disabled, preserving the frozen
/// harness gate unchanged. Returns `true` after the selected runtime phase
/// passes and a receipt is written; caller then skips its legacy gate body.
pub fn handle_frame(
    name: &str,
    frame: &Frame,
    original_expected_root: &Path,
) -> Result<bool, Error> {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(recorder) = active.as_mut() else {
            return Ok(false);
        };
        recorder.handle_frame(name, frame, original_expected_root)?;
        Ok(true)
    })
}

fn check_original_if_present(
    root: &Path,
    name: &str,
    frame: &Frame,
    receipt_root: &Path,
) -> Result<(), Error> {
    let original = root.join(format!("{name}.ansi"));
    validate_concrete_path(root, &original, "original approval", false)?;
    if !original_approval_present(&original)? {
        return Ok(());
    }
    let expected_root = artifact_root(receipt_root, "original-expected");
    let actual_root = artifact_root(receipt_root, "original-actual");
    let diff_root = artifact_root(receipt_root, "original-diff");
    validate_grouped_paths(root, &expected_root, &actual_root, &diff_root, name)?;
    let profile = Profile::default_profile();
    let mut renderer = profile
        .renderer(&VENDORED_FACES)
        .map_err(|error| Error::new(error.to_string()))?;
    GroupedStore::new(root)
        .with_artifacts(ArtifactSet::AnsiCursor)
        .with_expected_root(&expected_root)
        .with_actual_root(&actual_root)
        .with_diff_root(&diff_root)
        .with_review_evidence(false)
        .check_with(&mut renderer, name, frame, 1.0)
        .map_err(|error| Error::new(error.to_string()))?
        .ensure_matched()
        .map_err(|error| Error::new(error.to_string()))
}

fn original_approval_present(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(metadata) => Err(Error::new(format!(
            "original approval is not a regular file: {} ({})",
            path.display(),
            if metadata.file_type().is_symlink() {
                "symlink"
            } else if metadata.is_dir() {
                "directory"
            } else {
                "non-file"
            }
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Error::new(format!(
            "cannot inspect original approval {}: {error}",
            path.display()
        ))),
    }
}

fn validate_grouped_paths(
    approved_root: &Path,
    expected_root: &Path,
    actual_root: &Path,
    diff_root: &Path,
    name: &str,
) -> Result<(), Error> {
    for extension in ["ansi"] {
        validate_concrete_path(
            approved_root,
            &named_artifact(approved_root, name, extension),
            "approved artifact",
            false,
        )?;
    }
    for extension in ["png", "html", "png.fidelity.json"] {
        validate_concrete_path(
            expected_root,
            &named_artifact(expected_root, name, extension),
            "expected sidecar",
            true,
        )?;
    }
    for extension in [
        "ansi",
        "txt",
        "png",
        "html",
        "frame.json",
        "png.fidelity.json",
        "manifest.json",
        "verdict.json",
    ] {
        validate_concrete_path(
            actual_root,
            &named_artifact(actual_root, name, extension),
            "actual artifact",
            true,
        )?;
    }
    validate_concrete_path(
        diff_root,
        &named_artifact(diff_root, name, "png"),
        "diff artifact",
        true,
    )
}

fn named_artifact(root: &Path, name: &str, extension: &str) -> PathBuf {
    root.join(format!("{name}.{extension}"))
}

fn validate_concrete_path(
    root: &Path,
    path: &Path,
    label: &str,
    write_destination: bool,
) -> Result<(), Error> {
    let relative = path.strip_prefix(root).map_err(|error| {
        Error::new(format!(
            "{label} path {} is outside {}: {error}",
            path.display(),
            root.display()
        ))
    })?;
    let canonical_root = canonical_future_path(root)?;
    let expected = canonical_root.join(relative);
    let resolved = canonical_future_path(path)?;
    if resolved != expected {
        return Err(Error::new(format!(
            "{label} path resolves through a symlink: {} -> {}",
            path.display(),
            resolved.display()
        )));
    }

    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::new(format!(
                    "{label} path contains a symlink: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(Error::new(format!(
                    "cannot inspect {label} path {}: {error}",
                    current.display()
                )));
            }
        }
    }
    if write_destination {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => reject_hardlink(path, &metadata, label)?,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Error::new(format!(
                    "cannot inspect {label} destination {}: {error}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn reject_hardlink(path: &Path, metadata: &fs::Metadata, label: &str) -> Result<(), Error> {
    use std::os::unix::fs::MetadataExt as _;
    if metadata.nlink() > 1 {
        return Err(Error::new(format!(
            "{label} destination has multiple hard links: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn reject_hardlink(_path: &Path, _metadata: &fs::Metadata, _label: &str) -> Result<(), Error> {
    Ok(())
}

fn artifact_root(receipt_root: &Path, suffix: &str) -> PathBuf {
    let name = receipt_root.file_name().map_or_else(
        || "receipts".to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    receipt_root.with_file_name(format!("{name}.{suffix}"))
}

fn validate_roots(original: &Path, expected: &Path, receipts: &Path) -> Result<(), Error> {
    let paths = [
        ("original expectations", original.to_path_buf()),
        ("runtime expectations", expected.to_path_buf()),
        ("receipts", receipts.to_path_buf()),
        (
            "runtime expected artifacts",
            artifact_root(receipts, "runtime-expected"),
        ),
        ("runtime actuals", artifact_root(receipts, "actual")),
        ("runtime diffs", artifact_root(receipts, "diff")),
        (
            "original expected artifacts",
            artifact_root(receipts, "original-expected"),
        ),
        (
            "original actuals",
            artifact_root(receipts, "original-actual"),
        ),
        ("original diffs", artifact_root(receipts, "original-diff")),
    ];
    let mut canonical = Vec::with_capacity(paths.len());
    for (label, path) in paths {
        canonical.push((label, canonical_future_path(&path)?));
    }
    for (index, (left_label, left)) in canonical.iter().enumerate() {
        for (right_label, right) in canonical.iter().skip(index + 1) {
            if left == right || left.starts_with(right) || right.starts_with(left) {
                return Err(Error::new(format!(
                    "paired capture roots overlap: {left_label} ({}) and {right_label} ({})",
                    left.display(),
                    right.display()
                )));
            }
        }
    }
    Ok(())
}

fn canonical_future_path(path: &Path) -> Result<PathBuf, Error> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| Error::new(format!("cannot resolve current directory: {error}")))?
            .join(path)
    };
    let mut suffix = Vec::new();
    let mut existing = absolute.as_path();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = existing.file_name().ok_or_else(|| {
                    Error::new(format!(
                        "cannot resolve paired capture root {}",
                        path.display()
                    ))
                })?;
                suffix.push(name.to_os_string());
                existing = existing.parent().ok_or_else(|| {
                    Error::new(format!(
                        "cannot resolve paired capture root {}",
                        path.display()
                    ))
                })?;
            }
            Err(error) => {
                return Err(Error::new(format!(
                    "cannot inspect paired capture root {}: {error}",
                    existing.display()
                )));
            }
        }
    }
    let mut result = fs::canonicalize(existing).map_err(|error| {
        Error::new(format!(
            "cannot canonicalize {}: {error}",
            existing.display()
        ))
    })?;
    for component in suffix.into_iter().rev() {
        result.push(component);
    }
    Ok(result)
}

fn create_new_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::new(format!("path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|error| Error::new(format!("cannot create {}: {error}", parent.display())))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| Error::new(format!("cannot create {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| Error::new(format!("cannot write {}: {error}", path.display())))
}

fn validate_logical_name(name: &str) -> Result<(), Error> {
    tuiscotti::grouped::validate_name(name).map_err(|error| Error::new(error.to_string()))
}

fn checkpoint_name(case_name: &str, ordinal: u64, logical_name: &str) -> String {
    format!(
        "checkpoint-{ordinal:08}-{}",
        checkpoint_identity(case_name, ordinal, logical_name)
    )
}

fn checkpoint_identity(case_name: &str, ordinal: u64, logical_name: &str) -> String {
    let mut identity = Sha256::new();
    identity.update(b"termrock-paired-checkpoint-v1\0");
    identity.update(case_name.as_bytes());
    identity.update([0]);
    identity.update(ordinal.to_be_bytes());
    identity.update([0]);
    identity.update(logical_name.as_bytes());
    hex(&identity.finalize())
}

fn case_directory(case_name: &str) -> String {
    hex(&Sha256::digest(case_name.as_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", u32::from(control));
            }
            ordinary => output.push(ordinary),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tuiscotti::{Cursor, Frame, Provenance};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            let path = std::env::temp_dir().join(format!(
                "termrock-pair-frame-{label}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self, child: &str) -> PathBuf {
            self.0.join(child)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn frame() -> Frame {
        let mut frame = Frame::blank(4, 2, Provenance::default());
        frame.cells[0].symbol = "A".into();
        frame.cursor = Cursor {
            x: 2,
            y: 1,
            visible: true,
            ..Cursor::default()
        };
        frame
    }

    fn config(phase: Phase, scratch: &Scratch) -> Config {
        Config {
            phase,
            expected_root: scratch.path("expected"),
            receipt_root: scratch.path("receipts"),
        }
    }

    fn capture(scratch: &Scratch, case: &str, name: &str, frame: &Frame) -> Result<(), Error> {
        let mut recorder = Recorder::new(config(Phase::BaselineCapture, scratch), case)?;
        recorder.handle_frame(name, frame, &scratch.path("original"))
    }

    #[test]
    fn config_is_absent_only_when_every_variable_is_absent() {
        assert_eq!(Config::from_values(None, None, None).unwrap(), None);
        assert!(Config::from_values(Some("baseline-capture"), None, None).is_err());
        assert!(Config::from_values(None, Some("expected"), Some("receipts")).is_err());
        assert!(Config::from_values(Some("capture"), Some("expected"), Some("receipts")).is_err());
    }

    #[test]
    fn capture_creates_runtime_expectation_and_receipt_without_touching_original() {
        let scratch = Scratch::new("capture");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        capture(&scratch, "dashboard/home", "home", &frame()).unwrap();

        assert!(!original.join("home.ansi").exists());
        assert_eq!(fs::read_dir(scratch.path("expected")).unwrap().count(), 1);
        let receipt = scratch
            .path("receipts")
            .join(case_directory("dashboard/home"))
            .join(format!(
                "{}.json",
                checkpoint_name("dashboard/home", 0, "home")
            ));
        let receipt_text = fs::read_to_string(receipt).unwrap();
        assert!(receipt_text.contains("\"case_id\": \"dashboard/home\""));
        assert!(receipt_text.contains("\"ordinal\": 0"));
        assert!(receipt_text.contains("\"logical_name\": \"home\""));
        assert!(receipt_text.contains("\"sha256\": \""));
    }

    #[test]
    fn capture_preserves_existing_matching_original_bytes() {
        let scratch = Scratch::new("matching-original");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        let original_file = original.join("home.ansi");
        let bytes = tuiscotti::render::ansi_dump_with_cursor(&frame());
        fs::write(&original_file, &bytes).unwrap();

        capture(&scratch, "dashboard", "home", &frame()).unwrap();

        assert_eq!(fs::read(original_file).unwrap(), bytes.as_bytes());
    }

    #[test]
    fn capture_rejects_conflicting_existing_original_expectation() {
        let scratch = Scratch::new("original-conflict");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        fs::write(original.join("home.ansi"), "wrong").unwrap();
        assert!(capture(&scratch, "scenario", "home", &frame()).is_err());
        assert!(!scratch.path("expected").exists());
        assert!(!scratch.path("receipts").exists());
    }

    #[test]
    fn replay_rejects_character_color_modifier_cursor_and_shape_mutations() {
        let scratch = Scratch::new("mutation");
        let expected = frame();
        capture(&scratch, "scenario", "home", &expected).unwrap();
        let mut changes: Vec<(&str, Box<dyn Fn(&mut Frame)>)> = vec![
            (
                "character",
                Box::new(|value| value.cells[0].symbol = "B".into()),
            ),
            (
                "color",
                Box::new(|value| value.cells[0].fg = tuiscotti::Color::Indexed(3)),
            ),
            (
                "modifier",
                Box::new(|value| value.cells[0].mods.bold = true),
            ),
            ("cursor-position", Box::new(|value| value.cursor.x += 1)),
            (
                "cursor-visibility",
                Box::new(|value| value.cursor.visible = false),
            ),
            (
                "cursor-shape",
                Box::new(|value| value.cursor.style = tuiscotti::CursorStyle::Bar),
            ),
            (
                "dimensions",
                Box::new(|value| *value = Frame::blank(5, 2, Provenance::default())),
            ),
        ];
        for (label, change) in changes.drain(..) {
            let mut changed = expected.clone();
            change(&mut changed);
            let mut candidate_config = config(Phase::CandidateCheck, &scratch);
            candidate_config.receipt_root = scratch.path(&format!("receipts-{label}"));
            let mut candidate = Recorder::new(candidate_config.clone(), "scenario").unwrap();
            assert!(
                candidate
                    .handle_frame("home", &changed, &scratch.path("original"))
                    .is_err(),
                "{label}"
            );
            assert!(!candidate_config.receipt_root.exists(), "{label}");
        }
    }

    #[test]
    fn second_baseline_replay_detects_nondeterminism() {
        let scratch = Scratch::new("nondeterminism");
        capture(&scratch, "scenario", "home", &frame()).unwrap();
        let mut replay_config = config(Phase::BaselineCheck, &scratch);
        replay_config.receipt_root = scratch.path("replay-receipts");
        let mut replay = Recorder::new(replay_config, "scenario").unwrap();
        let mut changed = frame();
        changed.cells[0].symbol = "N".into();
        assert!(
            replay
                .handle_frame("home", &changed, &scratch.path("original"))
                .is_err()
        );
        assert!(!scratch.path("replay-receipts").exists());
    }

    #[test]
    fn expected_and_receipt_writes_refuse_overwrite() {
        let scratch = Scratch::new("duplicate");
        capture(&scratch, "scenario", "home", &frame()).unwrap();
        assert!(capture(&scratch, "scenario", "home", &frame()).is_err());
        let mut check_config = config(Phase::BaselineCheck, &scratch);
        check_config.receipt_root = scratch.path("replay-receipts");
        let mut recorder = Recorder::new(check_config.clone(), "scenario").unwrap();
        recorder
            .handle_frame("home", &frame(), &scratch.path("original"))
            .unwrap();
        let mut duplicate = Recorder::new(check_config, "scenario").unwrap();
        assert!(
            duplicate
                .handle_frame("home", &frame(), &scratch.path("original"))
                .is_err()
        );
    }

    #[test]
    fn strict_phases_read_but_never_change_runtime_expectations() {
        let scratch = Scratch::new("phase");
        let mut capture =
            Recorder::new(config(Phase::BaselineCapture, &scratch), "scenario").unwrap();
        capture
            .handle_frame("home", &frame(), &scratch.path("original"))
            .unwrap();
        let expected = scratch
            .path("expected")
            .join(format!("{}.ansi", checkpoint_name("scenario", 0, "home")));
        let original = fs::read(&expected).unwrap();
        let mut check_config = config(Phase::BaselineCheck, &scratch);
        check_config.receipt_root = scratch.path("baseline-receipts");
        let mut check = Recorder::new(check_config, "scenario").unwrap();
        check
            .handle_frame("home", &frame(), &scratch.path("original"))
            .unwrap();
        assert_eq!(fs::read(&expected).unwrap(), original);
        let mut changed = frame();
        changed.cells[0].symbol = "Z".into();
        let mut candidate_config = config(Phase::CandidateCheck, &scratch);
        candidate_config.receipt_root = scratch.path("candidate-receipts");
        let mut candidate = Recorder::new(candidate_config, "other-case").unwrap();
        assert!(
            candidate
                .handle_frame("home", &changed, &scratch.path("original"))
                .is_err()
        );
        assert_eq!(fs::read(expected).unwrap(), original);
    }

    #[test]
    fn multiple_checkpoints_keep_order_and_logical_names() {
        let scratch = Scratch::new("multiple");
        let mut recorder =
            Recorder::new(config(Phase::BaselineCapture, &scratch), "scenario").unwrap();
        recorder
            .handle_frame("home", &frame(), &scratch.path("original"))
            .unwrap();
        let mut second = frame();
        second.cells[0].symbol = "B".into();
        recorder
            .handle_frame("detail", &second, &scratch.path("original"))
            .unwrap();
        let first_file = scratch
            .path("expected")
            .join(format!("{}.ansi", checkpoint_name("scenario", 0, "home")));
        let second_file = scratch
            .path("expected")
            .join(format!("{}.ansi", checkpoint_name("scenario", 1, "detail")));
        assert!(first_file.is_file());
        assert!(second_file.is_file());
        let case = case_directory("scenario");
        assert!(
            scratch
                .path(&format!(
                    "receipts/{case}/{}.json",
                    checkpoint_name("scenario", 0, "home")
                ))
                .is_file()
        );
        assert!(
            scratch
                .path(&format!(
                    "receipts/{case}/{}.json",
                    checkpoint_name("scenario", 1, "detail")
                ))
                .is_file()
        );
    }

    #[cfg(unix)]
    #[test]
    fn roots_reject_symlink_alias_before_any_write() {
        use std::os::unix::fs::symlink;

        let scratch = Scratch::new("root-alias");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        let alias = scratch.path("expected-alias");
        symlink(&original, &alias).unwrap();
        let mut config = config(Phase::BaselineCapture, &scratch);
        config.expected_root = alias;
        let mut recorder = Recorder::new(config, "scenario").unwrap();
        assert!(recorder.handle_frame("home", &frame(), &original).is_err());
        assert_eq!(fs::read_dir(&original).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn runtime_approval_symlink_to_actual_cannot_self_compare() {
        use std::os::unix::fs::symlink;

        let scratch = Scratch::new("approval-actual-alias");
        let source = frame();
        capture(&scratch, "scenario", "home", &source).unwrap();
        let name = checkpoint_name("scenario", 0, "home");
        let expected = scratch.path("expected").join(format!("{name}.ansi"));
        fs::remove_file(&expected).unwrap();

        let mut candidate_config = config(Phase::CandidateCheck, &scratch);
        candidate_config.receipt_root = scratch.path("candidate-receipts");
        let candidate_actual =
            artifact_root(&candidate_config.receipt_root, "actual").join(format!("{name}.ansi"));
        fs::create_dir_all(candidate_actual.parent().unwrap()).unwrap();
        fs::write(&candidate_actual, "stale actual before gate").unwrap();
        symlink(&candidate_actual, &expected).unwrap();

        let mut candidate = Recorder::new(candidate_config, "scenario").unwrap();
        assert!(
            candidate
                .handle_frame("home", &source, &scratch.path("original"))
                .is_err()
        );
        assert_eq!(
            fs::read(candidate_actual).unwrap(),
            b"stale actual before gate"
        );
    }

    #[cfg(unix)]
    #[test]
    fn original_check_rejects_nested_scratch_symlink_before_touching_approval() {
        use std::os::unix::fs::symlink;

        let scratch = Scratch::new("nested-original-alias");
        let original = scratch.path("original");
        let original_subdir = original.join("sub");
        fs::create_dir_all(&original_subdir).unwrap();
        let original_file = original_subdir.join("home.ansi");
        let approved = tuiscotti::render::ansi_dump_with_cursor(&frame());
        fs::write(&original_file, &approved).unwrap();

        let actual_root = artifact_root(&scratch.path("receipts"), "original-actual");
        fs::create_dir_all(&actual_root).unwrap();
        symlink(&original_subdir, actual_root.join("sub")).unwrap();

        let mut recorder =
            Recorder::new(config(Phase::BaselineCapture, &scratch), "scenario").unwrap();
        assert!(
            recorder
                .handle_frame("sub/home", &frame(), &original)
                .is_err()
        );
        assert_eq!(fs::read(original_file).unwrap(), approved.as_bytes());
        assert!(!scratch.path("expected").exists());
    }

    #[cfg(unix)]
    #[test]
    fn actual_destination_hardlinked_to_original_is_rejected_before_store_write() {
        use std::os::unix::fs::MetadataExt as _;

        let scratch = Scratch::new("hardlinked-actual");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        let original_file = original.join("protected.ansi");
        fs::write(&original_file, b"frozen approval").unwrap();
        capture(&scratch, "scenario", "home", &frame()).unwrap();

        let mut candidate_config = config(Phase::CandidateCheck, &scratch);
        candidate_config.receipt_root = scratch.path("hardlink-receipts");
        let store_name = checkpoint_name("scenario", 0, "home");
        let actual_file = artifact_root(&candidate_config.receipt_root, "actual")
            .join(format!("{store_name}.ansi"));
        fs::create_dir_all(actual_file.parent().unwrap()).unwrap();
        fs::hard_link(&original_file, &actual_file).unwrap();

        let mut candidate = Recorder::new(candidate_config, "scenario").unwrap();
        assert!(candidate.handle_frame("home", &frame(), &original).is_err());
        assert_eq!(fs::read(&original_file).unwrap(), b"frozen approval");
        assert_eq!(fs::metadata(original_file).unwrap().nlink(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn original_approval_metadata_rejects_directories_and_dangling_symlinks() {
        use std::os::unix::fs::symlink;

        let scratch = Scratch::new("approval-metadata");
        let missing = scratch.path("missing.ansi");
        assert!(!original_approval_present(&missing).unwrap());

        let directory = scratch.path("directory.ansi");
        fs::create_dir_all(&directory).unwrap();
        assert!(original_approval_present(&directory).is_err());

        let dangling = scratch.path("dangling.ansi");
        symlink(scratch.path("absent-target"), &dangling).unwrap();
        assert!(original_approval_present(&dangling).is_err());

        let regular = scratch.path("regular.ansi");
        fs::write(&regular, b"ansi").unwrap();
        assert!(original_approval_present(&regular).unwrap());
    }

    #[test]
    fn checkpoint_identity_binds_case_order_and_logical_name() {
        let first = checkpoint_name("case/one", 0, "gate/home");
        let next = checkpoint_name("case/one", 1, "gate/home");
        let other = checkpoint_name("case/two", 0, "gate/home");
        let other_gate = checkpoint_name("case/one", 0, "gate/other");
        assert_ne!(first, next);
        assert_ne!(first, other);
        assert_ne!(first, other_gate);
        assert!(!first.contains('/'));
    }

    #[test]
    fn long_case_and_gate_names_use_filesystem_safe_checkpoint_paths() {
        let scratch = Scratch::new("long-checkpoint-name");
        let case = "holla/concept/activities-multi-with-history/120x40/truecolor";
        let logical = "holla/concept/activities-multi-with-history/120x40/truecolor";
        let mut recorder = Recorder::new(config(Phase::BaselineCapture, &scratch), case).unwrap();
        recorder
            .handle_frame(logical, &frame(), &scratch.path("original"))
            .unwrap();

        let expected_name = format!("{}.ansi", checkpoint_name(case, 0, logical));
        let receipt_name = format!("{}.json", checkpoint_name(case, 0, logical));
        assert!(expected_name.len() < 255);
        assert!(receipt_name.len() < 255);
        assert!(scratch.path(&format!("expected/{expected_name}")).is_file());
        assert!(
            scratch
                .path(&format!("receipts/{}/{receipt_name}", case_directory(case)))
                .is_file()
        );
        assert_eq!(checkpoint_identity(case, 0, logical).len(), 64);
    }
}
