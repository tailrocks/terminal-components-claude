//! Runtime support for paired captures of the frozen CLI and a candidate CLI.
//!
//! Copy this file into the frozen test harness as a module. It intentionally
//! depends only on `std`, pinned `tuiscotti`, and `sha2`.

use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
#[cfg(target_os = "linux")]
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use tuiscotti::grouped::{ArtifactSet, GroupedStore};
use tuiscotti::{Frame, Profile, Provenance, VENDORED_FACES};

const PHASE_ENV: &str = "TERMROCK_PAIR_PHASE";
const EXPECTED_ROOT_ENV: &str = "TERMROCK_PAIR_EXPECTED_ROOT";
const RECEIPT_ROOT_ENV: &str = "TERMROCK_PAIR_RECEIPT_ROOT";
const PROCESS_RECEIPT_KEY_ENV: &str = "TERMROCK_PAIR_PROCESS_RECEIPT_KEY";
const REFERENCE_CASE_ENV: &str = "TERMROCK_PAIR_REFERENCE_CASE";
const REFERENCE_ORDINAL_ENV: &str = "TERMROCK_PAIR_REFERENCE_ORDINAL";
const REFERENCE_LOGICAL_ENV: &str = "TERMROCK_PAIR_REFERENCE_LOGICAL";
const REFERENCE_ANSI_SHA256_ENV: &str = "TERMROCK_PAIR_REFERENCE_ANSI_SHA256";
const REFERENCE_SEMANTIC_SHA256_ENV: &str = "TERMROCK_PAIR_REFERENCE_SEMANTIC_FRAME_SHA256";
const REFERENCE_DECISION_SHA256_ENV: &str = "TERMROCK_PAIR_REFERENCE_DECISION_SHA256";

const REFERENCE_CASE: &str = "showcase/resize/overview_grown/120x40/truecolor";
const REFERENCE_ORDINAL: u64 = 0;
const REFERENCE_LOGICAL: &str = "showcase/resize/overview_grown/120x40/truecolor";
const REFERENCE_APPROVAL_ANSI_SHA256: &str =
    "7769a6686dfc5cb5c69facb03f54a0fa57e963b35fcf315b1c4625745e2b7708";
const REFERENCE_ACTUAL_ANSI_SHA256: &str =
    "f2e07ed284673b1d543441767460193e987034e163ca7ea7d3d4496473a0d442";
const REFERENCE_ACTUAL_SEMANTIC_SHA256: &str =
    "01e9f7d349fc8075df7b64188db97a05705792aac8c781184db14a6d1ecc253a";
const REFERENCE_DECISION_SHA256: &str =
    "2a43fb118c268300c8bbd6439ea11bb9a24ffff03dc3c9f4601893eee26a096b";

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferenceDecision {
    decision_sha256: String,
}

#[derive(Debug, Clone, Copy)]
struct OriginalApprovalMismatch<'a> {
    case_name: &'a str,
    ordinal: u64,
    logical_name: &'a str,
    approval_sha256: &'a str,
    actual_ansi_sha256: &'a str,
    actual_semantic_sha256: &'a str,
    status: tuiscotti::snapshot::Status,
    ansi_match: Option<bool>,
}

impl ReferenceDecision {
    fn from_env() -> Result<Option<Self>, Error> {
        Self::from_values(
            env_value(REFERENCE_CASE_ENV)?.as_deref(),
            env_value(REFERENCE_ORDINAL_ENV)?.as_deref(),
            env_value(REFERENCE_LOGICAL_ENV)?.as_deref(),
            env_value(REFERENCE_ANSI_SHA256_ENV)?.as_deref(),
            env_value(REFERENCE_SEMANTIC_SHA256_ENV)?.as_deref(),
            env_value(REFERENCE_DECISION_SHA256_ENV)?.as_deref(),
        )
    }

    fn from_values(
        case_name: Option<&str>,
        ordinal: Option<&str>,
        logical_name: Option<&str>,
        ansi_sha256: Option<&str>,
        semantic_sha256: Option<&str>,
        decision_sha256: Option<&str>,
    ) -> Result<Option<Self>, Error> {
        let values = [
            case_name,
            ordinal,
            logical_name,
            ansi_sha256,
            semantic_sha256,
            decision_sha256,
        ];
        if values.iter().all(Option::is_none) {
            return Ok(None);
        }
        let [
            Some(case_name),
            Some(ordinal),
            Some(logical_name),
            Some(ansi_sha256),
            Some(semantic_sha256),
            Some(decision_sha256),
        ] = values
        else {
            return Err(Error::new(
                "all six TERMROCK_PAIR_REFERENCE_* values must be set together",
            ));
        };
        let ordinal = ordinal
            .parse::<u64>()
            .map_err(|_| Error::new("invalid paired reference ordinal"))?;
        if case_name != REFERENCE_CASE
            || ordinal != REFERENCE_ORDINAL
            || logical_name != REFERENCE_LOGICAL
            || ansi_sha256 != REFERENCE_ACTUAL_ANSI_SHA256
            || semantic_sha256 != REFERENCE_ACTUAL_SEMANTIC_SHA256
            || decision_sha256 != REFERENCE_DECISION_SHA256
        {
            return Err(Error::new(
                "paired reference decision does not match the pinned frozen reference",
            ));
        }
        Ok(Some(Self {
            decision_sha256: decision_sha256.to_string(),
        }))
    }

    fn applies_to(&self, case_name: &str, ordinal: u64, logical_name: &str) -> bool {
        case_name == REFERENCE_CASE
            && ordinal == REFERENCE_ORDINAL
            && logical_name == REFERENCE_LOGICAL
    }

    fn authorizes_legacy_mismatch(&self, observed: &OriginalApprovalMismatch<'_>) -> bool {
        self.applies_to(observed.case_name, observed.ordinal, observed.logical_name)
            && self.decision_sha256 == REFERENCE_DECISION_SHA256
            && observed.approval_sha256 == REFERENCE_APPROVAL_ANSI_SHA256
            && observed.actual_ansi_sha256 == REFERENCE_ACTUAL_ANSI_SHA256
            && observed.actual_semantic_sha256 == REFERENCE_ACTUAL_SEMANTIC_SHA256
            && observed.status == tuiscotti::snapshot::Status::CellsDiffer
            && observed.ansi_match == Some(false)
    }
}

#[derive(Debug)]
struct Recorder {
    config: Config,
    case_name: String,
    reference_decision: Option<ReferenceDecision>,
    next_ordinal: u64,
    next_process_ordinal: u64,
    latest_process_receipt_sha256: Option<String>,
    process_receipts: BTreeMap<String, (String, String, u64)>,
}

impl Recorder {
    #[cfg(test)]
    fn new(config: Config, case_name: &str) -> Result<Self, Error> {
        Self::new_with_reference(config, case_name, None)
    }

    fn new_with_reference(
        config: Config,
        case_name: &str,
        reference_decision: Option<ReferenceDecision>,
    ) -> Result<Self, Error> {
        validate_logical_name(case_name)?;
        if reference_decision.is_some()
            && (config.phase != Phase::BaselineCapture || case_name != REFERENCE_CASE)
        {
            return Err(Error::new(
                "paired reference decision is valid only for its pinned baseline-capture case",
            ));
        }
        Ok(Self {
            config,
            case_name: case_name.to_string(),
            reference_decision,
            next_ordinal: 0,
            next_process_ordinal: 0,
            latest_process_receipt_sha256: None,
            process_receipts: BTreeMap::new(),
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
        let semantic_frame = semantic_frame_json(frame);
        let semantic_actual = artifact_root(&self.config.receipt_root, "semantic-actual")
            .join(format!("{store_name}.frame.json"));
        let receipt_path = self.receipt_path(ordinal, name);
        validate_concrete_path(&self.config.receipt_root, &receipt_path, "receipt", true)?;
        if receipt_path.exists() {
            return Err(Error::new(format!(
                "paired capture receipt already exists: {}",
                receipt_path.display()
            )));
        }
        validate_concrete_path(
            &artifact_root(&self.config.receipt_root, "semantic-actual"),
            &semantic_actual,
            "semantic actual frame",
            true,
        )?;
        if semantic_actual.exists() {
            return Err(Error::new(format!(
                "paired semantic actual frame already exists: {}",
                semantic_actual.display()
            )));
        }
        create_new_bytes(&semantic_actual, semantic_frame.as_bytes())?;

        let mut reference_decision_sha256 = None;
        match self.config.phase {
            Phase::BaselineCapture => {
                let expected = self.config.expected_root.join(format!("{store_name}.ansi"));
                let expected_frame = self
                    .config
                    .expected_root
                    .join(format!("{store_name}.frame.json"));
                validate_concrete_path(
                    &self.config.expected_root,
                    &expected,
                    "runtime expectation",
                    true,
                )?;
                validate_concrete_path(
                    &self.config.expected_root,
                    &expected_frame,
                    "semantic runtime expectation",
                    true,
                )?;
                if expected.exists() {
                    return Err(Error::new(format!(
                        "paired capture expectation already exists: {}",
                        expected.display()
                    )));
                }
                if expected_frame.exists() {
                    return Err(Error::new(format!(
                        "paired semantic expectation already exists: {}",
                        expected_frame.display()
                    )));
                }
                reference_decision_sha256 = check_original_if_present(
                    original_expected_root,
                    name,
                    frame,
                    &self.config.receipt_root,
                    &self.case_name,
                    ordinal,
                    self.reference_decision.as_ref(),
                )?;
                create_new_bytes(&expected, ansi.as_bytes())?;
                create_new_bytes(&expected_frame, semantic_frame.as_bytes())?;
                self.check_runtime(&store_name, frame)?;
            }
            Phase::BaselineCheck | Phase::CandidateCheck => {
                self.check_runtime(&store_name, frame)?;
            }
        }

        self.write_receipt(
            ordinal,
            name,
            &ansi,
            &semantic_frame,
            reference_decision_sha256.as_deref(),
        )
    }

    fn check_runtime(&self, name: &str, frame: &Frame) -> Result<(), Error> {
        let expected_frame = self.config.expected_root.join(format!("{name}.frame.json"));
        validate_concrete_path(
            &self.config.expected_root,
            &expected_frame,
            "semantic runtime expectation",
            true,
        )?;
        let expected_semantics = fs::read(&expected_frame).map_err(|error| {
            Error::new(format!(
                "cannot read semantic runtime expectation {}: {error}",
                expected_frame.display()
            ))
        })?;
        let actual_semantics = semantic_frame_json(frame);
        if expected_semantics != actual_semantics.as_bytes() {
            return Err(Error::new(format!(
                "semantic runtime frame mismatch: {}",
                expected_frame.display()
            )));
        }
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

    fn write_receipt(
        &self,
        ordinal: u64,
        name: &str,
        ansi: &str,
        semantic_frame: &str,
        reference_decision_sha256: Option<&str>,
    ) -> Result<(), Error> {
        let path = self.receipt_path(ordinal, name);
        validate_concrete_path(&self.config.receipt_root, &path, "receipt", true)?;
        let hash = Sha256::digest(ansi.as_bytes());
        let semantic_hash = Sha256::digest(semantic_frame.as_bytes());
        let mut record = format!(
            "{{\n  \"case_id\": {},\n  \"ordinal\": {ordinal},\n  \"logical_name\": {},\n  \"sha256\": \"{}\",\n  \"semantic_frame_sha256\": \"{}\"",
            json_string(&self.case_name),
            json_string(name),
            hex(&hash),
            hex(&semantic_hash),
        );
        if let Some(process_digest) = self.latest_process_receipt_sha256.as_deref() {
            use std::fmt::Write as _;
            write!(
                record,
                ",\n  \"process_receipt_sha256\": {}",
                json_string(process_digest)
            )
            .map_err(|error| Error::new(format!("cannot format process receipt link: {error}")))?;
        }
        if let Some(decision_sha256) = reference_decision_sha256 {
            use std::fmt::Write as _;
            write!(
                record,
                ",\n  \"reference_decision_sha256\": {}",
                json_string(decision_sha256)
            )
            .map_err(|error| Error::new(format!("cannot format receipt: {error}")))?;
        }
        record.push_str("\n}\n");
        create_new_bytes(&path, record.as_bytes())
    }

    fn receipt_path(&self, ordinal: u64, name: &str) -> PathBuf {
        let case_id = case_directory(&self.case_name);
        let checkpoint = format!("{}.json", checkpoint_name(&self.case_name, ordinal, name));
        self.config.receipt_root.join(case_id).join(checkpoint)
    }

    fn record_process_launch(
        &mut self,
        intent: ProcessLaunchIntent,
        pid: u32,
    ) -> Result<ProcessLaunchReceipt, Error> {
        let (
            observed_path,
            observed_sha256,
            observed_file_identity,
            process_start_identity,
            image_hash_source,
        ) = process_image_for_expected(pid, &intent.expected_path)?;
        let expected_sha256_after = sha256_file(&intent.expected_path)?;
        if observed_path != intent.expected_path
            || observed_sha256 != intent.expected_sha256
            || expected_sha256_after != intent.expected_sha256
            || observed_file_identity != intent.expected_file_identity
            || file_identity(&intent.expected_path)? != intent.expected_file_identity
        {
            return Err(Error::new(
                "running process image differs from the phase binary",
            ));
        }
        let ordinal = self.next_process_ordinal;
        self.next_process_ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| Error::new("paired process ordinal overflow"))?;
        let mut fields = BTreeMap::<String, String>::new();
        fields.insert("app".into(), json_string(&intent.app));
        fields.insert(
            "argv_provenance".into(),
            json_string("tuiscotti-builder-input"),
        );
        fields.insert(
            "constructed_argv_hmac_sha256".into(),
            json_string(&intent.argv_sha256),
        );
        fields.insert("case_id".into(), json_string(&self.case_name));
        fields.insert(
            "constructed_env_keys".into(),
            json_string_array(&intent.env_keys),
        );
        fields.insert(
            "constructed_env_hmac_sha256".into(),
            json_string(&intent.env_sha256),
        );
        fields.insert(
            "expected_path".into(),
            json_string(&intent.expected_path.to_string_lossy()),
        );
        fields.insert(
            "expected_file_identity".into(),
            json_string(&intent.expected_file_identity),
        );
        fields.insert(
            "expected_sha256".into(),
            json_string(&intent.expected_sha256),
        );
        fields.insert("image_hash_source".into(), json_string(image_hash_source));
        fields.insert(
            "observed_file_identity".into(),
            json_string(&observed_file_identity),
        );
        fields.insert(
            "observed_path".into(),
            json_string(&observed_path.to_string_lossy()),
        );
        fields.insert("observed_sha256".into(), json_string(&observed_sha256));
        fields.insert(
            "process_start_identity".into(),
            json_string(&process_start_identity),
        );
        fields.insert(
            "previous_receipt_sha256".into(),
            self.latest_process_receipt_sha256
                .as_ref()
                .map_or_else(|| "null".to_string(), |hash| json_string(hash)),
        );
        fields.insert("run_id".into(), json_string(&intent.run_id));
        fields.insert("phase".into(), json_string(&intent.phase));
        fields.insert("pid".into(), pid.to_string());
        fields.insert("ordinal".into(), ordinal.to_string());
        fields.insert("schema".into(), "1".into());
        let payload = json_object(&fields);
        let signature = hex(&hmac_sha256(&intent.hmac_key, payload.as_bytes()));
        fields.insert("hmac_sha256".into(), json_string(&signature));
        let record = json_object(&fields);
        let case_dir = case_directory(&self.case_name);
        let path = self
            .config
            .receipt_root
            .join("process")
            .join(case_dir)
            .join(format!("launch-{ordinal:08}.json"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Error::new(format!("cannot create process receipt directory: {error}"))
            })?;
        }
        validate_concrete_path(&self.config.receipt_root, &path, "process receipt", true)?;
        create_new_bytes(&path, record.as_bytes())?;
        let receipt_sha256 = hex(&Sha256::digest(record.as_bytes()));
        self.process_receipts.insert(
            receipt_sha256.clone(),
            (intent.phase, intent.case_name, ordinal),
        );
        self.latest_process_receipt_sha256 = Some(receipt_sha256.clone());
        Ok(ProcessLaunchReceipt {
            sha256: receipt_sha256,
            case_name: self.case_name.clone(),
            ordinal,
        })
    }

    fn complete_process_launch(&mut self, receipt: ProcessLaunchReceipt) -> Result<(), Error> {
        let Some((phase, case_name, ordinal)) = self.process_receipts.get(&receipt.sha256) else {
            return Err(Error::new(
                "process completion does not match a launch receipt",
            ));
        };
        if case_name != &receipt.case_name || ordinal != &receipt.ordinal {
            return Err(Error::new(
                "process completion identity differs from its launch receipt",
            ));
        }
        let mut fields = BTreeMap::<String, String>::new();
        fields.insert("case_id".into(), json_string(case_name));
        fields.insert("launch_receipt_sha256".into(), json_string(&receipt.sha256));
        fields.insert("ordinal".into(), ordinal.to_string());
        fields.insert("phase".into(), json_string(phase));
        fields.insert(
            "run_id".into(),
            json_string(
                &std::env::var("TERMROCK_PAIR_PROCESS_RECEIPT_RUN_ID")
                    .map_err(|_| Error::new("paired process receipt run id is missing"))?,
            ),
        );
        fields.insert("schema".into(), "1".into());
        let payload = json_object(&fields);
        let key = receipt_key()?;
        let signature = hex(&hmac_sha256(&key, payload.as_bytes()));
        fields.insert("hmac_sha256".into(), json_string(&signature));
        let record = json_object(&fields);
        let path = self
            .config
            .receipt_root
            .join("process")
            .join(case_directory(&receipt.case_name))
            .join(format!("closed-{:08}.json", receipt.ordinal));
        validate_concrete_path(
            &self.config.receipt_root,
            &path,
            "process completion receipt",
            true,
        )?;
        create_new_bytes(&path, record.as_bytes())
    }
}

/// The child launch receipt created by the shared frozen test-support adapter.
#[derive(Debug, Clone)]
pub struct ProcessLaunchReceipt {
    sha256: String,
    case_name: String,
    ordinal: u64,
}

/// Immutable launch inputs captured before calling pinned `Tui::spawn`.
#[derive(Debug, Clone)]
pub struct ProcessLaunchIntent {
    app: String,
    case_name: String,
    phase: String,
    run_id: String,
    expected_path: PathBuf,
    expected_sha256: String,
    expected_file_identity: String,
    argv_sha256: String,
    env_sha256: String,
    env_keys: Vec<String>,
    hmac_key: Vec<u8>,
}

/// Canonical observable frame, excluding capture provenance and retaining all
/// cells plus cursor position, visibility, shape, and blink intent.
fn semantic_frame_json(frame: &Frame) -> String {
    let mut frame = frame.clone();
    frame.provenance = Provenance {
        source: "screen".to_string(),
        ..Provenance::default()
    };
    frame.to_json()
}

fn receipt_key() -> Result<Vec<u8>, Error> {
    let value = std::env::var(PROCESS_RECEIPT_KEY_ENV)
        .map_err(|_| Error::new("paired process receipt authentication key is missing"))?;
    let key = decode_hex(&value)?;
    if key.len() != 32 {
        return Err(Error::new(
            "paired process receipt authentication key must be 32 bytes",
        ));
    }
    Ok(key)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, Error> {
    if value.len() % 2 != 0 {
        return Err(Error::new("paired receipt key has invalid hex length"));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |value: u8| match value {
                b'0'..=b'9' => Ok(value - b'0'),
                b'a'..=b'f' => Ok(value - b'a' + 10),
                b'A'..=b'F' => Ok(value - b'A' + 10),
                _ => Err(Error::new("paired receipt key contains non-hex data")),
            };
            Ok((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut normalized = [0_u8; BLOCK];
    if key.len() > BLOCK {
        normalized[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(BLOCK + message.len());
    let mut outer = Vec::with_capacity(BLOCK + 32);
    for byte in normalized {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    outer.extend_from_slice(&Sha256::digest(&inner));
    Sha256::digest(&outer).into()
}

fn json_object(fields: &BTreeMap<String, String>) -> String {
    let entries = fields
        .iter()
        .map(|(key, value)| format!("{}:{value}", json_string(key)))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(","))
}

fn json_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_environment(values: &BTreeMap<String, String>) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|(key, value)| format!("[{},{}]", json_string(key), json_string(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn file_identity(path: &Path) -> Result<String, Error> {
    let metadata = fs::metadata(path).map_err(|error| {
        Error::new(format!(
            "cannot stat process image {}: {error}",
            path.display()
        ))
    })?;
    metadata_identity(&metadata)
}

#[cfg(unix)]
fn metadata_identity(metadata: &fs::Metadata) -> Result<String, Error> {
    use std::os::unix::fs::MetadataExt as _;
    Ok(format!(
        "{}:{}:{}:{}:{}",
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec()
    ))
}

#[cfg(not(unix))]
fn metadata_identity(_metadata: &fs::Metadata) -> Result<String, Error> {
    Err(Error::new(
        "paired process identity probe is unsupported on this platform",
    ))
}

#[cfg(target_os = "linux")]
fn process_image(pid: u32) -> Result<(PathBuf, String, String, String, &'static str), Error> {
    use std::os::unix::fs::MetadataExt as _;
    let proc_exe = PathBuf::from(format!("/proc/{pid}/exe"));
    let before_start = process_start_time(pid)?;
    let before_path = fs::read_link(&proc_exe)
        .map_err(|error| Error::new(format!("cannot resolve /proc child image: {error}")))?;
    if before_path.to_string_lossy().ends_with(" (deleted)") {
        return Err(Error::new(
            "paired child executable was replaced or deleted after spawn",
        ));
    }
    let mut image = fs::File::open(&proc_exe)
        .map_err(|error| Error::new(format!("cannot open /proc child image: {error}")))?;
    let metadata = image
        .metadata()
        .map_err(|error| Error::new(format!("cannot stat /proc child image: {error}")))?;
    let identity = metadata_identity(&metadata)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = image
            .read(&mut buffer)
            .map_err(|error| Error::new(format!("cannot hash /proc child image: {error}")))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let after_path = fs::read_link(&proc_exe)
        .map_err(|error| Error::new(format!("child exited during image verification: {error}")))?;
    let after_start = process_start_time(pid)?;
    if before_path != after_path || before_start != after_start {
        return Err(Error::new(
            "paired child process identity changed during image verification",
        ));
    }
    let path = before_path.canonicalize().map_err(|error| {
        Error::new(format!("cannot canonicalize observed child image: {error}"))
    })?;
    let expected_metadata = fs::metadata(&path)
        .map_err(|error| Error::new(format!("cannot stat observed child image: {error}")))?;
    if metadata.dev() != expected_metadata.dev() || metadata.ino() != expected_metadata.ino() {
        return Err(Error::new(
            "observed process image inode differs from resolved path",
        ));
    }
    Ok((
        path,
        hex(&digest.finalize()),
        identity,
        before_start,
        "linux-proc-exe-open-fd",
    ))
}

#[cfg(target_os = "macos")]
fn process_image_for_expected(
    pid: u32,
    expected_path: &Path,
) -> Result<(PathBuf, String, String, String, &'static str), Error> {
    let initial_start = macos_process_start(pid)?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if let Ok(observed) = process_image(pid) {
            if observed.3 != initial_start {
                return Err(Error::new(
                    "paired child PID identity changed while waiting for its executable",
                ));
            }
            if observed.0 == expected_path {
                return Ok(observed);
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err(Error::new(
                "paired child executable did not match the phase binary before the deadline",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn process_image_for_expected(
    pid: u32,
    _expected_path: &Path,
) -> Result<(PathBuf, String, String, String, &'static str), Error> {
    process_image(pid)
}

#[cfg(target_os = "linux")]
fn process_start_time(pid: u32) -> Result<String, Error> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|error| Error::new(format!("cannot read child process identity: {error}")))?;
    let fields = stat
        .rsplit_once(')')
        .ok_or_else(|| Error::new("invalid child process stat"))?
        .1;
    fields
        .split_whitespace()
        .nth(19)
        .map(str::to_string)
        .ok_or_else(|| Error::new("child process stat has no start time"))
}

#[cfg(target_os = "macos")]
fn process_image(pid: u32) -> Result<(PathBuf, String, String, String, &'static str), Error> {
    let before_start = macos_process_start(pid)?;
    let first_path = macos_process_path(pid)?;
    let first_identity = file_identity(&first_path)?;
    let hash = sha256_file(&first_path)?;
    let second_path = macos_process_path(pid)?;
    let second_identity = file_identity(&second_path)?;
    let after_start = macos_process_start(pid)?;
    if first_path != second_path || first_identity != second_identity || before_start != after_start
    {
        return Err(Error::new(
            "paired macOS child path or file identity changed during verification",
        ));
    }
    Ok((
        first_path,
        hash,
        first_identity,
        before_start,
        "macos-proc-pidpath-file",
    ))
}

#[cfg(target_os = "macos")]
fn macos_process_start(pid: u32) -> Result<String, Error> {
    use std::ffi::{c_char, c_int, c_void};
    #[repr(C)]
    struct ProcBsdInfo {
        flags: u32,
        status: u32,
        xstatus: u32,
        pid: u32,
        ppid: u32,
        uid: u32,
        gid: u32,
        ruid: u32,
        rgid: u32,
        svuid: u32,
        svgid: u32,
        reserved: u32,
        command: [c_char; 16],
        name: [c_char; 32],
        nfiles: u32,
        pgid: u32,
        pjobc: u32,
        terminal_device: u32,
        terminal_pgid: u32,
        nice: i32,
        start_seconds: u64,
        start_microseconds: u64,
    }
    const PROC_PIDTBSDINFO: c_int = 3;
    #[link(name = "proc")]
    unsafe extern "C" {
        #[link_name = "proc_pidinfo"]
        fn process_info(
            pid: c_int,
            flavor: c_int,
            arg: u64,
            buffer: *mut c_void,
            size: c_int,
        ) -> c_int;
    }
    let mut info = std::mem::MaybeUninit::<ProcBsdInfo>::zeroed();
    let size = std::mem::size_of::<ProcBsdInfo>() as c_int;
    // SAFETY: `info` is writable for exactly its declared size and proc_pidinfo
    // initializes that entire structure when it reports success.
    let read = unsafe {
        process_info(
            pid as c_int,
            PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size,
        )
    };
    if read != size {
        return Err(Error::new("macOS process start identity is unavailable"));
    }
    // SAFETY: the exact structure size was returned above.
    let info = unsafe { info.assume_init() };
    if info.pid != pid {
        return Err(Error::new("macOS process identity does not match PID"));
    }
    Ok(format!(
        "{}:{}",
        info.start_seconds, info.start_microseconds
    ))
}

#[cfg(target_os = "macos")]
fn macos_process_path(pid: u32) -> Result<PathBuf, Error> {
    use std::ffi::c_void;
    const MAX: usize = 4096;
    #[link(name = "proc")]
    unsafe extern "C" {
        fn proc_pidpath(pid: i32, buffer: *mut c_void, buffersize: u32) -> i32;
    }
    let mut buffer = [0_u8; MAX];
    // SAFETY: `buffer` is writable for MAX bytes and remains alive through the call.
    let length = unsafe { proc_pidpath(pid as i32, buffer.as_mut_ptr().cast(), MAX as u32) };
    if length <= 0 || length as usize >= MAX {
        return Err(Error::new(
            "macOS proc_pidpath could not identify the paired child image",
        ));
    }
    let written = &buffer[..length as usize];
    let path_end = written
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(written.len());
    let path = std::str::from_utf8(&written[..path_end])
        .map_err(|_| Error::new("macOS process image path is not UTF-8"))?;
    PathBuf::from(path)
        .canonicalize()
        .map_err(|error| Error::new(format!("cannot canonicalize macOS process image: {error}")))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn process_image(_pid: u32) -> Result<(PathBuf, String, String, String, &'static str), Error> {
    Err(Error::new(
        "paired process image probe is unsupported on this platform",
    ))
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
    let reference_decision = ReferenceDecision::from_env()?;
    if config.is_none() && reference_decision.is_some() {
        return Err(Error::new(
            "paired reference decision requires paired capture to be enabled",
        ));
    }
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        *active = match config {
            Some(config) => Some(Recorder::new_with_reference(
                config,
                name,
                reference_decision,
            )?),
            None => None,
        };
        Ok(active.is_some())
    })
}

/// Capture the exact argv and allowlisted constructed child environment before spawn.
/// Values are hashed and never written to the receipt.
pub fn prepare_process_launch(
    case_name: &str,
    argv: &[String],
    environment: &BTreeMap<String, String>,
) -> Result<Option<ProcessLaunchIntent>, Error> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let Some(recorder) = active.as_ref() else {
            if Config::from_env()?.is_some() {
                return Err(Error::new("paired child spawn occurred without begin_case"));
            }
            return Ok(None);
        };
        if recorder.case_name != case_name {
            return Err(Error::new("spawn case differs from active paired scenario"));
        }
        if argv.is_empty() || environment.contains_key(PROCESS_RECEIPT_KEY_ENV) {
            return Err(Error::new(
                "invalid paired argv or receipt key leaked into constructed child environment",
            ));
        }
        let phase = std::env::var(PHASE_ENV)
            .map_err(|_| Error::new("paired process receipt has no phase"))?;
        let run_id = std::env::var("TERMROCK_PAIR_PROCESS_RECEIPT_RUN_ID")
            .map_err(|_| Error::new("paired process receipt run id is missing"))?;
        let bin_root = PathBuf::from(
            std::env::var("VISUAL_BIN_DIR")
                .map_err(|_| Error::new("paired process receipt has no VISUAL_BIN_DIR"))?,
        )
        .canonicalize()
        .map_err(|error| Error::new(format!("cannot resolve paired binary directory: {error}")))?;
        let app = PathBuf::from(&argv[0])
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::new("paired argv executable has no UTF-8 filename"))?
            .to_string();
        if !["showcase", "tablepro", "jackin-preview", "holla"].contains(&app.as_str()) {
            return Err(Error::new("paired argv names an unregistered application"));
        }
        let binary = bin_root.join(&app);
        let metadata = fs::symlink_metadata(&binary)
            .map_err(|error| Error::new(format!("cannot inspect paired binary: {error}")))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(Error::new(
                "paired binary is not a regular non-symlink file",
            ));
        }
        let expected_path = binary
            .canonicalize()
            .map_err(|error| Error::new(format!("cannot canonicalize paired binary: {error}")))?;
        if PathBuf::from(&argv[0]).canonicalize().ok().as_deref() != Some(expected_path.as_path()) {
            return Err(Error::new(
                "paired argv executable differs from the phase binary",
            ));
        }
        let expected_sha256 = sha256_file(&expected_path)?;
        let expected_file_identity = file_identity(&expected_path)?;
        let hmac_key = receipt_key()?;
        let argv_sha256 = hex(&hmac_sha256(
            &hmac_key,
            format!("termrock-tui-argv-v1\0{}", json_string_array(argv)).as_bytes(),
        ));
        let env_sha256 = hex(&hmac_sha256(
            &hmac_key,
            format!("termrock-child-env-v1\0{}", json_environment(environment)).as_bytes(),
        ));
        let env_keys = environment.keys().cloned().collect();
        Ok(Some(ProcessLaunchIntent {
            app,
            case_name: case_name.to_string(),
            phase,
            run_id,
            expected_path,
            expected_sha256,
            expected_file_identity,
            argv_sha256,
            env_sha256,
            env_keys,
            hmac_key,
        }))
    })
}

/// Record and verify the OS-observed image after the pinned Tuiscotti spawn.
pub fn record_process_launch(
    intent: Option<ProcessLaunchIntent>,
    pid: Option<u32>,
) -> Result<Option<ProcessLaunchReceipt>, Error> {
    let Some(intent) = intent else {
        return Ok(None);
    };
    let pid = pid.ok_or_else(|| Error::new("paired Tuiscotti session has no child PID"))?;
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let recorder = active
            .as_mut()
            .ok_or_else(|| Error::new("paired process launch has no active recorder"))?;
        if recorder.case_name != intent.case_name {
            return Err(Error::new(
                "paired process launch no longer matches active case",
            ));
        }
        recorder.record_process_launch(intent, pid).map(Some)
    })
}

/// Record that the support wrapper for a Tuiscotti session was dropped.
pub fn complete_process_launch(receipt: Option<ProcessLaunchReceipt>) -> Result<(), Error> {
    let Some(receipt) = receipt else {
        return Ok(());
    };
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let recorder = active
            .as_mut()
            .ok_or_else(|| Error::new("paired process completion has no active recorder"))?;
        recorder.complete_process_launch(receipt)
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
        if recorder.latest_process_receipt_sha256.is_none() {
            return Err(Error::new(
                "paired visual checkpoint has no process launch receipt",
            ));
        }
        recorder.handle_frame(name, frame, original_expected_root)?;
        Ok(true)
    })
}

fn check_original_if_present(
    root: &Path,
    name: &str,
    frame: &Frame,
    receipt_root: &Path,
    case_name: &str,
    ordinal: u64,
    reference_decision: Option<&ReferenceDecision>,
) -> Result<Option<String>, Error> {
    let original = root.join(format!("{name}.ansi"));
    validate_concrete_path(root, &original, "original approval", false)?;
    if !original_approval_present(&original)? {
        if reference_decision.is_some_and(|decision| decision.applies_to(case_name, ordinal, name))
        {
            return Err(Error::new("pinned stale original approval is missing"));
        }
        return Ok(None);
    }
    let original_sha256 = sha256_file(&original)?;
    let expected_root = artifact_root(receipt_root, "original-expected");
    let actual_root = artifact_root(receipt_root, "original-actual");
    let diff_root = artifact_root(receipt_root, "original-diff");
    validate_grouped_paths(root, &expected_root, &actual_root, &diff_root, name)?;
    let profile = Profile::default_profile();
    let mut renderer = profile
        .renderer(&VENDORED_FACES)
        .map_err(|error| Error::new(error.to_string()))?;
    let outcome = GroupedStore::new(root)
        .with_artifacts(ArtifactSet::AnsiCursor)
        .with_expected_root(&expected_root)
        .with_actual_root(&actual_root)
        .with_diff_root(&diff_root)
        .with_review_evidence(false)
        .check_with(&mut renderer, name, frame, 1.0)
        .map_err(|error| Error::new(error.to_string()))?;
    if let Some(decision) =
        reference_decision.filter(|decision| decision.applies_to(case_name, ordinal, name))
    {
        let ansi = tuiscotti::render::ansi_dump_with_cursor(frame);
        let ansi_sha256 = sha256_hex(ansi.as_bytes());
        let semantic_sha256 = sha256_hex(semantic_frame_json(frame).as_bytes());
        if decision.authorizes_legacy_mismatch(&OriginalApprovalMismatch {
            case_name,
            ordinal,
            logical_name: name,
            approval_sha256: &original_sha256,
            actual_ansi_sha256: &ansi_sha256,
            actual_semantic_sha256: &semantic_sha256,
            status: outcome.status(),
            ansi_match: outcome.ansi_match,
        }) {
            return Ok(Some(decision.decision_sha256.clone()));
        }
        return Err(Error::new(
            "pinned reference exception did not match the diagnosed original approval mismatch",
        ));
    }
    outcome
        .ensure_matched()
        .map_err(|error| Error::new(error.to_string()))?;
    Ok(None)
}

fn sha256_file(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path)
        .map_err(|error| Error::new(format!("cannot read {}: {error}", path.display())))?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
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
    validate_concrete_path(
        approved_root,
        &named_artifact(approved_root, name, "ansi"),
        "approved artifact",
        false,
    )?;
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
            "semantic actual frames",
            artifact_root(receipts, "semantic-actual"),
        ),
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
            '\u{8}' => output.push_str("\\b"),
            '\n' => output.push_str("\\n"),
            '\u{c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if u32::from(control) <= 0x1f => {
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
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    use tuiscotti::{CancelToken, Cursor, Frame, Provenance, Tui};

    type FrameMutation = Box<dyn Fn(&mut Frame)>;
    type NamedFrameMutation<'a> = (&'a str, FrameMutation);

    #[test]
    fn hmac_sha256_matches_the_rfc_4231_sha256_vector() {
        let key = [0x0b; 20];
        assert_eq!(
            hex(&hmac_sha256(&key, b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(decode_hex("00aF").unwrap(), [0x00, 0xaf]);
        assert!(decode_hex("0xz1").is_err());
    }

    #[test]
    fn json_string_uses_python_json_canonical_control_escapes() {
        assert_eq!(
            json_string("\u{8}\u{c}\u{1b}\u{7f}\u{85}"),
            "\"\\b\\f\\u001b\u{7f}\u{85}\""
        );
    }

    #[test]
    fn process_probe_reports_the_running_test_binary_and_hash() {
        let expected = std::env::current_exe().unwrap().canonicalize().unwrap();
        let (observed, digest, _identity, process_start, method) =
            process_image(std::process::id()).unwrap();
        assert_eq!(observed, expected);
        assert_eq!(digest, sha256_file(&expected).unwrap());
        assert!(!process_start.is_empty());
        assert!(!process_start.is_empty());
        #[cfg(target_os = "linux")]
        assert_eq!(method, "linux-proc-exe-open-fd");
        #[cfg(target_os = "macos")]
        assert_eq!(method, "macos-proc-pidpath-file");
    }

    #[cfg(unix)]
    #[test]
    fn tuiscotti_child_env_removal_and_close_are_observable() {
        const KEY: &str = "TERMROCK_PAIR_PROCESS_RECEIPT_KEY";
        let child = Tui::new([
            "/bin/sh",
            "-c",
            "if [ -z \"${TERMROCK_PAIR_PROCESS_RECEIPT_KEY-}\" ]; then printf CLEAN; else printf LEAKED; fi",
        ])
        .size(80, 24)
        .env_clear(true)
        .env(KEY, "receipt-secret-must-not-reach-child")
        .env_remove(KEY)
        .spawn()
        .expect("spawn isolated Tuiscotti child");
        let observation = child
            .wait_stable_quiet(
                Instant::now() + Duration::from_secs(5),
                Duration::from_millis(30),
                &CancelToken::new(),
            )
            .expect("child emitted and settled its environment result");
        let output = (0..observation.screen.rows())
            .map(|row| {
                (0..observation.screen.cols())
                    .filter_map(|col| observation.screen.get(col, row))
                    .filter(|cell| !cell.continuation)
                    .map(|cell| cell.symbol.as_str())
                    .collect::<String>()
            })
            .collect::<String>();
        assert!(
            output.contains("CLEAN"),
            "child environment was not clean: {output:?}"
        );
        assert!(
            !output.contains("LEAKED"),
            "receipt key leaked to child: {output:?}"
        );
        child.close().expect("close and reap Tuiscotti child");
        assert!(
            child.poll_exit().is_some(),
            "close must observe the reaped child"
        );
    }

    #[test]
    fn paired_gate_rejects_a_checkpoint_without_a_process_launch_receipt() {
        let scratch = Scratch::new("missing-process-receipt");
        let recorder = Recorder::new(config(Phase::BaselineCapture, &scratch), "scenario").unwrap();
        ACTIVE.with(|active| *active.borrow_mut() = Some(recorder));
        let result = handle_frame("home", &frame(), &scratch.path("original"));
        ACTIVE.with(|active| *active.borrow_mut() = None);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no process launch receipt")
        );
        assert!(!scratch.path("expected").exists());
    }

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
    fn reference_decision_is_all_or_none_and_every_value_is_pinned() {
        assert_eq!(
            ReferenceDecision::from_values(None, None, None, None, None, None).unwrap(),
            None
        );
        assert!(
            ReferenceDecision::from_values(
                Some(REFERENCE_CASE),
                Some("0"),
                Some(REFERENCE_LOGICAL),
                Some(REFERENCE_ACTUAL_ANSI_SHA256),
                Some(REFERENCE_ACTUAL_SEMANTIC_SHA256),
                None,
            )
            .is_err()
        );

        let decision = ReferenceDecision::from_values(
            Some(REFERENCE_CASE),
            Some("0"),
            Some(REFERENCE_LOGICAL),
            Some(REFERENCE_ACTUAL_ANSI_SHA256),
            Some(REFERENCE_ACTUAL_SEMANTIC_SHA256),
            Some(REFERENCE_DECISION_SHA256),
        )
        .unwrap()
        .unwrap();
        assert_eq!(decision.decision_sha256, REFERENCE_DECISION_SHA256);
        assert!(
            ReferenceDecision::from_values(
                Some(REFERENCE_CASE),
                Some("1"),
                Some(REFERENCE_LOGICAL),
                Some(REFERENCE_ACTUAL_ANSI_SHA256),
                Some(REFERENCE_ACTUAL_SEMANTIC_SHA256),
                Some(REFERENCE_DECISION_SHA256),
            )
            .is_err()
        );
        assert!(
            ReferenceDecision::from_values(
                Some(REFERENCE_CASE),
                Some("0"),
                Some(REFERENCE_LOGICAL),
                Some("0".repeat(64).as_str()),
                Some(REFERENCE_ACTUAL_SEMANTIC_SHA256),
                Some(REFERENCE_DECISION_SHA256),
            )
            .is_err()
        );
    }

    #[test]
    fn reference_exception_authorizes_only_the_pinned_mismatch() {
        let decision = ReferenceDecision {
            decision_sha256: REFERENCE_DECISION_SHA256.to_string(),
        };
        let expected = OriginalApprovalMismatch {
            case_name: REFERENCE_CASE,
            ordinal: REFERENCE_ORDINAL,
            logical_name: REFERENCE_LOGICAL,
            approval_sha256: REFERENCE_APPROVAL_ANSI_SHA256,
            actual_ansi_sha256: REFERENCE_ACTUAL_ANSI_SHA256,
            actual_semantic_sha256: REFERENCE_ACTUAL_SEMANTIC_SHA256,
            status: tuiscotti::snapshot::Status::CellsDiffer,
            ansi_match: Some(false),
        };
        assert!(decision.authorizes_legacy_mismatch(&expected));

        let mut changed = expected;
        changed.ordinal += 1;
        assert!(!decision.authorizes_legacy_mismatch(&changed));
        let mut changed = expected;
        changed.actual_ansi_sha256 = "0";
        assert!(!decision.authorizes_legacy_mismatch(&changed));
        let mut changed = expected;
        changed.actual_semantic_sha256 = "0";
        assert!(!decision.authorizes_legacy_mismatch(&changed));
        let mut changed = expected;
        changed.status = tuiscotti::snapshot::Status::PixelsDiffer;
        assert!(!decision.authorizes_legacy_mismatch(&changed));
        let mut changed = expected;
        changed.ansi_match = Some(true);
        assert!(!decision.authorizes_legacy_mismatch(&changed));
    }

    #[test]
    fn reference_decision_is_rejected_outside_its_capture_case_and_phase() {
        let scratch = Scratch::new("reference-scope");
        let decision = ReferenceDecision {
            decision_sha256: REFERENCE_DECISION_SHA256.to_string(),
        };
        assert!(
            Recorder::new_with_reference(
                config(Phase::BaselineCheck, &scratch),
                REFERENCE_CASE,
                Some(decision.clone()),
            )
            .is_err()
        );
        assert!(
            Recorder::new_with_reference(
                config(Phase::CandidateCheck, &scratch),
                REFERENCE_CASE,
                Some(decision.clone()),
            )
            .is_err()
        );
        assert!(
            Recorder::new_with_reference(
                config(Phase::BaselineCapture, &scratch),
                "showcase::pages::overview::120x40::truecolor",
                Some(decision.clone()),
            )
            .is_err()
        );
        assert!(
            Recorder::new_with_reference(
                config(Phase::BaselineCapture, &scratch),
                REFERENCE_CASE,
                Some(decision),
            )
            .is_ok()
        );
    }

    #[test]
    fn begin_case_accepts_the_slash_scenario_identity_from_the_frozen_harness() {
        struct RestoreEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

        impl Drop for RestoreEnv {
            fn drop(&mut self) {
                for (key, value) in &self.0 {
                    // SAFETY: this test serializes the only paired-env reader in this process.
                    unsafe {
                        match value {
                            Some(value) => std::env::set_var(key, value),
                            None => std::env::remove_var(key),
                        }
                    }
                }
            }
        }

        let scratch = Scratch::new("slash-reference-case");
        let values = [
            (PHASE_ENV, Some("baseline-capture".to_string())),
            (
                EXPECTED_ROOT_ENV,
                Some(scratch.path("expected").to_string_lossy().into_owned()),
            ),
            (
                RECEIPT_ROOT_ENV,
                Some(scratch.path("receipts").to_string_lossy().into_owned()),
            ),
            (REFERENCE_CASE_ENV, Some(REFERENCE_CASE.to_string())),
            (REFERENCE_ORDINAL_ENV, Some(REFERENCE_ORDINAL.to_string())),
            (REFERENCE_LOGICAL_ENV, Some(REFERENCE_LOGICAL.to_string())),
            (
                REFERENCE_ANSI_SHA256_ENV,
                Some(REFERENCE_ACTUAL_ANSI_SHA256.to_string()),
            ),
            (
                REFERENCE_SEMANTIC_SHA256_ENV,
                Some(REFERENCE_ACTUAL_SEMANTIC_SHA256.to_string()),
            ),
            (
                REFERENCE_DECISION_SHA256_ENV,
                Some(REFERENCE_DECISION_SHA256.to_string()),
            ),
        ];
        let restore = RestoreEnv(
            values
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect(),
        );
        for (key, value) in values {
            // SAFETY: this test is the only one that reads paired env variables.
            unsafe { std::env::set_var(key, value.unwrap()) };
        }

        assert!(begin_case(REFERENCE_CASE).unwrap());
        ACTIVE.with(|active| {
            assert_eq!(active.borrow().as_ref().unwrap().case_name, REFERENCE_CASE);
            *active.borrow_mut() = None;
        });
        drop(restore);
    }

    #[test]
    fn capture_creates_runtime_expectation_and_receipt_without_touching_original() {
        let scratch = Scratch::new("capture");
        let original = scratch.path("original");
        fs::create_dir_all(&original).unwrap();
        capture(&scratch, "dashboard/home", "home", &frame()).unwrap();

        assert!(!original.join("home.ansi").exists());
        assert_eq!(fs::read_dir(scratch.path("expected")).unwrap().count(), 2);
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
        assert!(receipt_text.contains("\"semantic_frame_sha256\": \""));
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
        let mut changes: Vec<NamedFrameMutation<'_>> = vec![
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
    fn hidden_cursor_position_shape_and_blink_are_compared_when_ansi_is_identical() {
        let scratch = Scratch::new("hidden-cursor-semantics");
        let mut expected = frame();
        expected.cursor.visible = false;
        capture(&scratch, "scenario", "home", &expected).unwrap();
        let checkpoint = checkpoint_name("scenario", 0, "home");
        let expected_path = scratch
            .path("expected")
            .join(format!("{checkpoint}.frame.json"));
        let expected_bytes = fs::read(&expected_path).unwrap();
        let expected_ansi = tuiscotti::render::ansi_dump_with_cursor(&expected);

        let changes: [NamedFrameMutation<'_>; 4] = [
            ("hidden-x", Box::new(|value| value.cursor.x += 1)),
            ("hidden-y", Box::new(|value| value.cursor.y -= 1)),
            (
                "hidden-shape",
                Box::new(|value| value.cursor.style = tuiscotti::CursorStyle::Bar),
            ),
            (
                "hidden-blink",
                Box::new(|value| value.cursor.blinking = true),
            ),
        ];
        for (label, change) in changes {
            let mut changed = expected.clone();
            change(&mut changed);
            assert_eq!(
                tuiscotti::render::ansi_dump_with_cursor(&changed),
                expected_ansi,
                "the pinned ANSI projection should hide {label}"
            );
            let mut candidate_config = config(Phase::CandidateCheck, &scratch);
            candidate_config.receipt_root = scratch.path(&format!("receipts-{label}"));
            let mut candidate = Recorder::new(candidate_config.clone(), "scenario").unwrap();
            assert!(
                candidate
                    .handle_frame("home", &changed, &scratch.path("original"))
                    .is_err(),
                "semantic comparison must reject {label}"
            );
            let actual_path = artifact_root(&candidate_config.receipt_root, "semantic-actual")
                .join(format!("{checkpoint}.frame.json"));
            let actual_bytes = fs::read(&actual_path).unwrap();
            assert_eq!(
                actual_bytes,
                semantic_frame_json(&changed).as_bytes(),
                "full actual semantic frame should be retained for {label}"
            );
            let actual_frame =
                Frame::from_json(std::str::from_utf8(&actual_bytes).unwrap()).unwrap();
            assert_eq!(actual_frame.cursor, changed.cursor, "{label}");
            assert_eq!(actual_frame.cells, changed.cells, "{label}");
            assert_eq!(fs::read(&expected_path).unwrap(), expected_bytes, "{label}");
            assert!(!candidate_config.receipt_root.exists(), "{label}");
            let receipt = candidate.receipt_path(0, "home");
            assert!(
                !receipt.exists(),
                "failed {label} must not have a success receipt"
            );
        }
    }

    #[test]
    fn semantic_frame_uses_the_reviewed_screen_provenance() {
        let semantic = semantic_frame_json(&frame());
        let decoded = Frame::from_json(&semantic).unwrap();
        assert_eq!(decoded.provenance.source, "screen");
        assert_eq!(semantic, semantic_frame_json(&decoded));
    }

    #[cfg(unix)]
    #[test]
    fn semantic_actual_destination_rejects_hardlink_before_comparison() {
        use std::os::unix::fs::MetadataExt as _;

        let scratch = Scratch::new("hardlinked-semantic-actual");
        let source = frame();
        capture(&scratch, "scenario", "home", &source).unwrap();
        let expected_frame = scratch.path("expected").join(format!(
            "{}.frame.json",
            checkpoint_name("scenario", 0, "home")
        ));
        let expected_bytes = fs::read(&expected_frame).unwrap();
        let receipt = Recorder::new(config(Phase::BaselineCapture, &scratch), "scenario")
            .unwrap()
            .receipt_path(0, "home");
        let receipt_bytes = fs::read(&receipt).unwrap();
        let candidate_config = config(Phase::CandidateCheck, &scratch);
        let diagnostic = artifact_root(&candidate_config.receipt_root, "semantic-actual").join(
            format!("{}.frame.json", checkpoint_name("scenario", 0, "home")),
        );
        fs::create_dir_all(diagnostic.parent().unwrap()).unwrap();
        fs::remove_file(&diagnostic).unwrap();
        fs::hard_link(&expected_frame, &diagnostic).unwrap();

        let mut candidate = Recorder::new(candidate_config.clone(), "scenario").unwrap();
        assert!(
            candidate
                .handle_frame("home", &source, &scratch.path("original"))
                .is_err()
        );
        assert_eq!(fs::metadata(&expected_frame).unwrap().nlink(), 2);
        assert_eq!(fs::read(&expected_frame).unwrap(), expected_bytes);
        assert_eq!(fs::read(receipt).unwrap(), receipt_bytes);
    }

    #[test]
    fn matching_baseline_frame_passes_capture_replay_and_candidate_phases() {
        let scratch = Scratch::new("semantic-positive-control");
        let source = frame();
        capture(&scratch, "scenario", "home", &source).unwrap();

        for (phase, root) in [
            (Phase::BaselineCheck, "baseline-receipts"),
            (Phase::CandidateCheck, "candidate-receipts"),
        ] {
            let mut phase_config = config(phase, &scratch);
            phase_config.receipt_root = scratch.path(root);
            let mut recorder = Recorder::new(phase_config, "scenario").unwrap();
            recorder
                .handle_frame("home", &source, &scratch.path("original"))
                .unwrap();
        }
    }

    #[test]
    fn missing_semantic_expectation_fails_closed() {
        let scratch = Scratch::new("missing-semantic-frame");
        let source = frame();
        capture(&scratch, "scenario", "home", &source).unwrap();
        let expected_frame = scratch.path("expected").join(format!(
            "{}.frame.json",
            checkpoint_name("scenario", 0, "home")
        ));
        fs::remove_file(expected_frame).unwrap();

        let mut candidate_config = config(Phase::CandidateCheck, &scratch);
        candidate_config.receipt_root = scratch.path("candidate-receipts");
        let mut candidate = Recorder::new(candidate_config.clone(), "scenario").unwrap();
        assert!(
            candidate
                .handle_frame("home", &source, &scratch.path("original"))
                .is_err()
        );
        assert!(!candidate_config.receipt_root.exists());
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
