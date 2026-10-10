//! Strict reader for the pinned cargo-nextest list and libtest-json-plus output.
//!
//! The caller owns the trusted selection digest. This module never derives the
//! selected IDs from Node output or from the test list: it verifies a separate,
//! static selection file against the digest passed on the reader's command line.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

pub const REQUEST_SCHEMA: &str = "termrock-nextest-reader-request/v1";
pub const REQUEST_SCHEMA_V2: &str = "termrock-nextest-reader-request/v2";
pub const SELECTION_SCHEMA: &str = "termrock-nextest-selection/v1";
pub const SELECTION_SCHEMA_V2: &str = "termrock-nextest-selection/v2";
pub const REPORT_SCHEMA: &str = "termrock-nextest-reader-report/v1";
pub const REPORT_SCHEMA_V2: &str = "termrock-nextest-reader-report/v2";
pub const PINNED_NEXTEST_VERSION: &str = "0.9.146";
pub const PINNED_MESSAGE_FORMAT: &str = "libtest-json-plus";
pub const PINNED_MESSAGE_FORMAT_VERSION: &str = "0.1";
pub const EXPECTED_SELECTED_TESTS: usize = 9;
pub const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
pub const MAX_SELECTION_BYTES: u64 = 1024 * 1024;
pub const MAX_CAPTURE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_MACHINE_EVENTS: usize = 100_000;
const MAX_V2_SELECTED_TESTS: usize = 1024;
const MAX_V2_TARGETS: usize = 64;
const MAX_V2_OMITTED_IGNORED_TESTS: u64 = 1024;

/// A request, capture, or protocol error. Errors are deliberately plain text so
/// the thin CLI can report them without inventing another serialization layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderError(pub String);

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ReaderError {}

impl From<io::Error> for ReaderError {
    fn from(value: io::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileReference {
    path: PathBuf,
    byte_length: u64,
    sha256: String,
    truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Termination {
    Exited(i32),
    Signal(i32),
    Timeout,
    SpawnError(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessReference {
    stdout: FileReference,
    stderr: FileReference,
    termination: Termination,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Invocation {
    machine_stream: StreamName,
    filter_args: Vec<String>,
    run_ignored: String,
    run_id: Option<String>,
    protocol: ProtocolVersion,
    expected_test_count: Option<usize>,
    targets: Vec<TargetIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProtocolVersion {
    V1,
    V2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StreamName {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TestIdentity {
    package: String,
    binary_id: String,
    kind: String,
    test_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TargetIdentity {
    package: String,
    binary_id: String,
    kind: String,
}

impl TestIdentity {
    fn event_name(&self) -> String {
        format!("{}${}", self.binary_id, self.test_name)
    }

    fn suite_id(&self) -> &str {
        &self.binary_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedTest {
    identity: TestIdentity,
    ignored: bool,
    filter_match_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Selection {
    filter_args: Vec<String>,
    run_ignored: String,
    tests: Vec<ExpectedTest>,
    protocol: ProtocolVersion,
    expected_test_count: Option<usize>,
    targets: Vec<TargetIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureBytes {
    bytes: Vec<u8>,
    declared_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessBytes {
    stdout: CaptureBytes,
    stderr: CaptureBytes,
    termination: Termination,
}

/// Reads a request, selection file, and the exact raw process capture files.
///
/// `trusted_selection_sha256` must come from caller-owned static configuration;
/// a digest copied only from the request is not a trust anchor.
pub fn read_files(
    request_path: &Path,
    selection_path: &Path,
    trusted_selection_sha256: &str,
) -> Result<Value, ReaderError> {
    let request_bytes = read_regular_file(request_path, MAX_REQUEST_BYTES)?;
    let selection_bytes = read_regular_file(selection_path, MAX_SELECTION_BYTES)?;
    let request = parse_request(&request_bytes)?;
    let selection_hash = crate::sha256_hex(&selection_bytes);
    validate_sha256(trusted_selection_sha256, "trusted selection digest")?;
    if selection_hash != trusted_selection_sha256 {
        return Err(ReaderError(
            "selection bytes do not match the trusted selection digest".into(),
        ));
    }
    let selection = parse_selection(&selection_bytes)?;
    validate_request_selection(&request.invocation, &selection)?;
    let list = load_process(&request.list)?;
    let run = load_process(&request.run)?;
    let wrapper = load_process(&request.wrapper)?;
    Ok(evaluate(
        &request.invocation,
        &selection,
        &selection_hash,
        &list,
        &run,
        &wrapper,
    ))
}

fn validate_request_selection(
    invocation: &Invocation,
    selection: &Selection,
) -> Result<(), ReaderError> {
    if invocation.protocol != selection.protocol {
        return Err(ReaderError(
            "request and selection schema versions do not match".into(),
        ));
    }
    if invocation.filter_args != selection.filter_args
        || invocation.run_ignored != selection.run_ignored
    {
        return Err(ReaderError(
            "invocation filter policy does not match the trusted selection".into(),
        ));
    }
    if invocation.protocol == ProtocolVersion::V2
        && (invocation.expected_test_count != selection.expected_test_count
            || invocation.targets != selection.targets)
    {
        return Err(ReaderError(
            "invocation targets/count do not match the trusted selection".into(),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Request {
    invocation: Invocation,
    list: ProcessReference,
    run: ProcessReference,
    wrapper: ProcessReference,
}

fn parse_request(bytes: &[u8]) -> Result<Request, ReaderError> {
    if bytes.len() as u64 > MAX_REQUEST_BYTES {
        return Err(ReaderError("request exceeds the 1 MiB limit".into()));
    }
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| ReaderError(format!("invalid request JSON: {error}")))?;
    let object = object_with_keys(
        &value,
        "request",
        &[
            "schema",
            "invocation",
            "list_capture",
            "run_capture",
            "wrapper_capture",
        ],
    )?;
    let protocol = match string_field(object, "schema", "request")? {
        REQUEST_SCHEMA => ProtocolVersion::V1,
        REQUEST_SCHEMA_V2 => ProtocolVersion::V2,
        _ => return Err(ReaderError("unsupported request schema".into())),
    };
    let invocation_fields_v1 = [
        "nextest_version",
        "message_format",
        "message_format_version",
        "libtest_json_enabled",
        "retries",
        "stress",
        "partition",
        "machine_stream",
        "filter_args",
        "run_ignored",
        "run_id",
    ];
    let invocation_fields_v2 = [
        "nextest_version",
        "message_format",
        "message_format_version",
        "libtest_json_enabled",
        "retries",
        "stress",
        "partition",
        "machine_stream",
        "filter_args",
        "run_ignored",
        "run_id",
        "expected_test_count",
        "targets",
    ];
    let invocation_value = required(object, "invocation", "request")?;
    let invocation_allowed_fields: &[&str] = match protocol {
        ProtocolVersion::V1 => &invocation_fields_v1,
        ProtocolVersion::V2 => &invocation_fields_v2,
    };
    let invocation_object =
        object_with_keys(invocation_value, "invocation", invocation_allowed_fields)?;
    require_exact_string(
        invocation_object,
        "nextest_version",
        PINNED_NEXTEST_VERSION,
        "invocation",
    )?;
    require_exact_string(
        invocation_object,
        "message_format",
        PINNED_MESSAGE_FORMAT,
        "invocation",
    )?;
    require_exact_string(
        invocation_object,
        "message_format_version",
        PINNED_MESSAGE_FORMAT_VERSION,
        "invocation",
    )?;
    if bool_field(invocation_object, "libtest_json_enabled", "invocation")? != Some(true) {
        return Err(ReaderError(
            "invocation must enable NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1".into(),
        ));
    }
    if u64_field(invocation_object, "retries", "invocation")? != 0 {
        return Err(ReaderError("retries are unsupported; expected zero".into()));
    }
    require_exact_string(invocation_object, "stress", "none", "invocation")?;
    require_exact_string(invocation_object, "partition", "none", "invocation")?;
    let machine_stream = match string_field(invocation_object, "machine_stream", "invocation")? {
        "stdout" => StreamName::Stdout,
        "stderr" => StreamName::Stderr,
        _ => {
            return Err(ReaderError(
                "machine_stream must be stdout or stderr".into(),
            ));
        }
    };
    let filter_args = string_array_field(invocation_object, "filter_args", "invocation")?;
    if (protocol == ProtocolVersion::V1 && filter_args.is_empty())
        || filter_args.iter().any(String::is_empty)
    {
        let message = if protocol == ProtocolVersion::V1 {
            "invocation filter_args must be nonempty strings"
        } else {
            "invocation filter_args must contain only nonempty strings"
        };
        return Err(ReaderError(message.into()));
    }
    let run_ignored = string_field(invocation_object, "run_ignored", "invocation")?;
    if !matches!(run_ignored, "default" | "only" | "all") {
        return Err(ReaderError(
            "invocation run_ignored policy is unsupported".into(),
        ));
    }
    let run_id = optional_string_field(invocation_object, "run_id", "invocation")?;
    if run_id.as_deref().is_some_and(str::is_empty) {
        return Err(ReaderError("run_id must be nonempty when present".into()));
    }
    let (expected_test_count, targets) = match protocol {
        ProtocolVersion::V1 => (None, Vec::new()),
        ProtocolVersion::V2 => (
            Some(parse_expected_test_count(invocation_object, "invocation")?),
            parse_targets(invocation_object, "invocation")?,
        ),
    };
    Ok(Request {
        invocation: Invocation {
            machine_stream,
            filter_args,
            run_ignored: run_ignored.to_owned(),
            run_id,
            protocol,
            expected_test_count,
            targets,
        },
        list: parse_process_reference(
            required(object, "list_capture", "request")?,
            "list_capture",
        )?,
        run: parse_process_reference(required(object, "run_capture", "request")?, "run_capture")?,
        wrapper: parse_process_reference(
            required(object, "wrapper_capture", "request")?,
            "wrapper_capture",
        )?,
    })
}

fn parse_selection(bytes: &[u8]) -> Result<Selection, ReaderError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| ReaderError(format!("invalid selection JSON: {error}")))?;
    let value_object = value
        .as_object()
        .ok_or_else(|| ReaderError("selection must be an object".into()))?;
    let selection_fields_v1 = ["schema", "filter_args", "run_ignored", "tests"];
    let selection_fields_v2 = [
        "schema",
        "filter_args",
        "run_ignored",
        "expected_test_count",
        "targets",
        "tests",
    ];
    let schema = value_object.get("schema").and_then(Value::as_str);
    let protocol = if schema == Some(SELECTION_SCHEMA_V2) {
        ProtocolVersion::V2
    } else {
        ProtocolVersion::V1
    };
    let selection_allowed_fields: &[&str] = match protocol {
        ProtocolVersion::V1 => &selection_fields_v1,
        ProtocolVersion::V2 => &selection_fields_v2,
    };
    let object = object_with_keys(&value, "selection", selection_allowed_fields)?;
    let schema = string_field(object, "schema", "selection")?;
    if (protocol == ProtocolVersion::V1 && schema != SELECTION_SCHEMA)
        || (protocol == ProtocolVersion::V2 && schema != SELECTION_SCHEMA_V2)
    {
        return Err(ReaderError("unsupported selection schema".into()));
    }
    let filter_args = string_array_field(object, "filter_args", "selection")?;
    if (protocol == ProtocolVersion::V1 && filter_args.is_empty())
        || filter_args.iter().any(String::is_empty)
    {
        let message = if protocol == ProtocolVersion::V1 {
            "selection filter_args must be nonempty strings"
        } else {
            "selection filter_args must contain only nonempty strings"
        };
        return Err(ReaderError(message.into()));
    }
    let run_ignored = string_field(object, "run_ignored", "selection")?;
    if !matches!(run_ignored, "default" | "only" | "all") {
        return Err(ReaderError(
            "selection run_ignored policy is unsupported".into(),
        ));
    }
    let test_values = array_field(object, "tests", "selection")?;
    let (expected_test_count, targets) = match protocol {
        ProtocolVersion::V1 => {
            if test_values.len() != EXPECTED_SELECTED_TESTS {
                return Err(ReaderError(format!(
                    "selection must contain exactly {EXPECTED_SELECTED_TESTS} test identities"
                )));
            }
            (None, Vec::new())
        }
        ProtocolVersion::V2 => {
            let expected = parse_expected_test_count(object, "selection")?;
            if usize::try_from(expected).ok() != Some(test_values.len()) {
                return Err(ReaderError(format!(
                    "selection expected_test_count {expected} does not match {} test identities",
                    test_values.len()
                )));
            }
            let targets = parse_targets(object, "selection")?;
            (Some(expected), targets)
        }
    };
    let mut tests = Vec::with_capacity(test_values.len());
    let mut identities = BTreeSet::new();
    for (index, value) in test_values.iter().enumerate() {
        let context = format!("selection.tests[{index}]");
        let item = object_with_keys(
            value,
            &context,
            &[
                "package",
                "binary_id",
                "kind",
                "test_name",
                "ignored",
                "filter_match_status",
            ],
        )?;
        let identity = TestIdentity {
            package: nonempty_string_field(item, "package", &context)?,
            binary_id: nonempty_string_field(item, "binary_id", &context)?,
            kind: nonempty_string_field(item, "kind", &context)?,
            test_name: nonempty_string_field(item, "test_name", &context)?,
        };
        let prefix = format!("{}::", identity.package);
        if (!identity.binary_id.starts_with(&prefix) && identity.binary_id != identity.package)
            || (identity.binary_id.starts_with(&prefix)
                && identity.binary_id[prefix.len()..].is_empty())
            || identity.test_name.contains('$')
            || !matches!(identity.kind.as_str(), "test" | "lib" | "example" | "bench")
        {
            return Err(ReaderError(format!(
                "{context} has an invalid package/binary/test identity"
            )));
        }
        if protocol == ProtocolVersion::V2 {
            let target_matches = targets
                .iter()
                .filter(|target| {
                    target.package == identity.package
                        && target.binary_id == identity.binary_id
                        && target.kind == identity.kind
                })
                .count();
            if target_matches != 1 {
                return Err(ReaderError(format!(
                    "{context} must belong to exactly one declared target"
                )));
            }
        }
        let ignored = bool_field(item, "ignored", &context)?
            .ok_or_else(|| ReaderError(format!("{context}.ignored must be boolean")))?;
        let filter_match_status = nonempty_string_field(item, "filter_match_status", &context)?;
        if filter_match_status != "matches" {
            return Err(ReaderError(format!(
                "{context}.filter_match_status must be matches for a selected test"
            )));
        }
        if (run_ignored == "default" && ignored) || (run_ignored == "only" && !ignored) {
            return Err(ReaderError(format!(
                "{context} is inconsistent with the run_ignored selection policy"
            )));
        }
        if !identities.insert(identity.clone()) {
            return Err(ReaderError(format!(
                "{context} duplicates a selected identity"
            )));
        }
        tests.push(ExpectedTest {
            identity,
            ignored,
            filter_match_status,
        });
    }
    if protocol == ProtocolVersion::V2 {
        for target in &targets {
            if !tests.iter().any(|test| {
                test.identity.package == target.package
                    && test.identity.binary_id == target.binary_id
                    && test.identity.kind == target.kind
            }) {
                return Err(ReaderError(format!(
                    "declared target {} has no selected identities",
                    target.binary_id
                )));
            }
        }
    }
    tests.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(Selection {
        filter_args,
        run_ignored: run_ignored.to_owned(),
        tests,
        protocol,
        expected_test_count,
        targets,
    })
}

fn parse_expected_test_count(
    object: &Map<String, Value>,
    context: &str,
) -> Result<usize, ReaderError> {
    let count = u64_field(object, "expected_test_count", context)?;
    if !(1..=MAX_V2_SELECTED_TESTS as u64).contains(&count) {
        return Err(ReaderError(format!(
            "{context}.expected_test_count must be between 1 and {MAX_V2_SELECTED_TESTS}"
        )));
    }
    usize::try_from(count)
        .map_err(|_| ReaderError(format!("{context}.expected_test_count exceeds usize")))
}

fn parse_targets(
    object: &Map<String, Value>,
    context: &str,
) -> Result<Vec<TargetIdentity>, ReaderError> {
    let values = array_field(object, "targets", context)?;
    if values.is_empty() || values.len() > MAX_V2_TARGETS {
        return Err(ReaderError(format!(
            "{context}.targets must contain between 1 and {MAX_V2_TARGETS} entries"
        )));
    }
    let mut targets = Vec::with_capacity(values.len());
    let mut unique = BTreeSet::new();
    let mut binary_ids = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let target_context = format!("{context}.targets[{index}]");
        let item = object_with_keys(value, &target_context, &["package", "binary_id", "kind"])?;
        let target = TargetIdentity {
            package: nonempty_string_field(item, "package", &target_context)?,
            binary_id: nonempty_string_field(item, "binary_id", &target_context)?,
            kind: nonempty_string_field(item, "kind", &target_context)?,
        };
        let prefix = format!("{}::", target.package);
        if (!target.binary_id.starts_with(&prefix) && target.binary_id != target.package)
            || (target.binary_id.starts_with(&prefix)
                && target.binary_id[prefix.len()..].is_empty())
            || !matches!(target.kind.as_str(), "test" | "lib" | "example" | "bench")
        {
            return Err(ReaderError(format!(
                "{target_context} has an invalid package/binary/kind identity"
            )));
        }
        if !unique.insert(target.clone()) {
            return Err(ReaderError(format!(
                "{target_context} duplicates a declared target"
            )));
        }
        if !binary_ids.insert(target.binary_id.clone()) {
            return Err(ReaderError(format!(
                "{target_context} duplicates a target binary identity"
            )));
        }
        targets.push(target);
    }
    Ok(targets)
}

fn parse_process_reference(value: &Value, context: &str) -> Result<ProcessReference, ReaderError> {
    let object = object_with_keys(value, context, &["stdout", "stderr", "termination"])?;
    Ok(ProcessReference {
        stdout: parse_file_reference(
            required(object, "stdout", context)?,
            &format!("{context}.stdout"),
        )?,
        stderr: parse_file_reference(
            required(object, "stderr", context)?,
            &format!("{context}.stderr"),
        )?,
        termination: parse_termination(required(object, "termination", context)?, context)?,
    })
}

fn parse_file_reference(value: &Value, context: &str) -> Result<FileReference, ReaderError> {
    let object = object_with_keys(
        value,
        context,
        &["path", "byte_length", "sha256", "truncated"],
    )?;
    let path = PathBuf::from(nonempty_string_field(object, "path", context)?);
    if !path.is_absolute() {
        return Err(ReaderError(format!("{context}.path must be absolute")));
    }
    let byte_length = u64_field(object, "byte_length", context)?;
    if byte_length > MAX_CAPTURE_BYTES {
        return Err(ReaderError(format!(
            "{context} exceeds the 8 MiB capture limit"
        )));
    }
    let sha256 = nonempty_string_field(object, "sha256", context)?;
    validate_sha256(&sha256, context)?;
    let truncated = bool_field(object, "truncated", context)?
        .ok_or_else(|| ReaderError(format!("{context}.truncated must be boolean")))?;
    Ok(FileReference {
        path,
        byte_length,
        sha256,
        truncated,
    })
}

fn parse_termination(value: &Value, context: &str) -> Result<Termination, ReaderError> {
    let object = value
        .as_object()
        .ok_or_else(|| ReaderError(format!("{context}.termination must be an object")))?;
    let kind = string_field(object, "kind", &format!("{context}.termination"))?;
    match kind {
        "exited" => {
            ensure_keys(
                object,
                &["kind", "exit_code"],
                &format!("{context}.termination"),
            )?;
            let code = i32_field(object, "exit_code", &format!("{context}.termination"))?;
            Ok(Termination::Exited(code))
        }
        "signal" => {
            ensure_keys(
                object,
                &["kind", "signal"],
                &format!("{context}.termination"),
            )?;
            let signal = i32_field(object, "signal", &format!("{context}.termination"))?;
            if signal <= 0 {
                return Err(ReaderError(format!(
                    "{context}.termination.signal must be positive"
                )));
            }
            Ok(Termination::Signal(signal))
        }
        "timeout" => {
            ensure_keys(object, &["kind"], &format!("{context}.termination"))?;
            Ok(Termination::Timeout)
        }
        "spawn_error" => {
            ensure_keys(
                object,
                &["kind", "message"],
                &format!("{context}.termination"),
            )?;
            let message =
                nonempty_string_field(object, "message", &format!("{context}.termination"))?;
            Ok(Termination::SpawnError(message))
        }
        _ => Err(ReaderError(format!(
            "{context}.termination.kind is unsupported"
        ))),
    }
}

fn load_process(reference: &ProcessReference) -> Result<ProcessBytes, ReaderError> {
    Ok(ProcessBytes {
        stdout: load_capture(&reference.stdout)?,
        stderr: load_capture(&reference.stderr)?,
        termination: reference.termination.clone(),
    })
}

fn load_capture(reference: &FileReference) -> Result<CaptureBytes, ReaderError> {
    let bytes = read_regular_file(&reference.path, MAX_CAPTURE_BYTES)?;
    if bytes.len() as u64 != reference.byte_length {
        return Err(ReaderError(format!(
            "capture byte length changed for {}",
            reference.path.display()
        )));
    }
    if crate::sha256_hex(&bytes) != reference.sha256 {
        return Err(ReaderError(format!(
            "capture SHA-256 mismatch for {}",
            reference.path.display()
        )));
    }
    Ok(CaptureBytes {
        bytes,
        declared_truncated: reference.truncated,
    })
}

#[cfg(unix)]
fn read_regular_file(path: &Path, limit: u64) -> Result<Vec<u8>, ReaderError> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
    let mut file = options
        .open(path)
        .map_err(|error| ReaderError(format!("cannot open input {}: {error}", path.display())))?;
    let metadata = file.metadata().map_err(ReaderError::from)?;
    if !metadata.is_file() {
        return Err(ReaderError(format!(
            "input is not a regular file: {}",
            path.display()
        )));
    }
    if metadata.len() > limit {
        return Err(ReaderError(format!(
            "input exceeds byte limit: {}",
            path.display()
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(ReaderError::from)?;
    if bytes.len() as u64 > limit || bytes.len() as u64 != metadata.len() {
        return Err(ReaderError(format!(
            "input changed while reading: {}",
            path.display()
        )));
    }
    Ok(bytes)
}

#[cfg(not(unix))]
fn read_regular_file(_path: &Path, _limit: u64) -> Result<Vec<u8>, ReaderError> {
    Err(ReaderError(
        "capture reader requires Unix no-follow file opens".into(),
    ))
}

fn evaluate(
    invocation: &Invocation,
    selection: &Selection,
    selection_sha256: &str,
    list: &ProcessBytes,
    run: &ProcessBytes,
    wrapper: &ProcessBytes,
) -> Value {
    let list_result = evaluate_list(list, selection);
    let run_result = evaluate_run(invocation, selection, list_result.as_ref().ok(), run);
    let process_results = json!({
        "list": process_json(list),
        "run": process_json(run),
        "wrapper": process_json(wrapper),
    });
    let wrapper_ok = process_is_success(wrapper);
    let list_ok = list_result.is_ok() && process_is_success(list);
    let run_ok = run_result.machine_state == "verified"
        && process_is_success(run)
        && run_result.all_selected_passed
        && run_result.count_anomalies.is_empty()
        && run_result.contradictions.is_empty()
        && run_result
            .child_harness_summary_check
            .contradictions
            .is_empty();
    let accepted = list_ok && run_ok && wrapper_ok;
    let list_json = match list_result {
        Ok(result) => result.to_json(true),
        Err(error) => json!({"state":"invalid", "error":error.0}),
    };
    let mut run_json = run_result.to_json();
    if let Some(object) = run_json.as_object_mut() {
        object.insert(
            "process_success".into(),
            Value::Bool(process_is_success(run)),
        );
    }
    let mut report = json!({
        "schema": if invocation.protocol == ProtocolVersion::V1 { REPORT_SCHEMA } else { REPORT_SCHEMA_V2 },
        "nextest_version": PINNED_NEXTEST_VERSION,
        "message_format": PINNED_MESSAGE_FORMAT,
        "message_format_version": PINNED_MESSAGE_FORMAT_VERSION,
        "run_id": invocation.run_id,
        "selection_sha256": selection_sha256,
        "selection_count": selection.tests.len().to_string(),
        "processes": process_results,
        "list": list_json,
        "run": run_json,
        "acceptance": if accepted {"accepted"} else {"blocked"},
    });
    if invocation.protocol == ProtocolVersion::V2 {
        let object = report
            .as_object_mut()
            .expect("reader report root is always an object");
        object.insert(
            "expected_test_count".into(),
            json!(selection.expected_test_count.expect("v2 count is required")),
        );
        object.insert(
            "targets".into(),
            json!(
                selection
                    .targets
                    .iter()
                    .map(|target| json!({
                        "package":target.package,
                        "binary_id":target.binary_id,
                        "kind":target.kind,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    report
}

fn process_is_success(process: &ProcessBytes) -> bool {
    matches!(process.termination, Termination::Exited(0))
        && !process.stdout.declared_truncated
        && !process.stderr.declared_truncated
}

fn process_json(process: &ProcessBytes) -> Value {
    let termination = match &process.termination {
        Termination::Exited(code) => json!({"kind":"exited", "exit_code":code}),
        Termination::Signal(signal) => json!({"kind":"signal", "signal":signal}),
        Termination::Timeout => json!({"kind":"timeout"}),
        Termination::SpawnError(message) => json!({"kind":"spawn_error", "message":message}),
    };
    json!({
        "termination": termination,
        "stdout": capture_json(&process.stdout),
        "stderr": capture_json(&process.stderr),
        "success": process_is_success(process),
    })
}

fn capture_json(capture: &CaptureBytes) -> Value {
    json!({
        "byte_length": capture.bytes.len().to_string(),
        "sha256": crate::sha256_hex(&capture.bytes),
        "truncated": capture.declared_truncated,
    })
}

#[derive(Clone, Debug)]
struct ListResult {
    test_count: u64,
    inventory: BTreeMap<String, u64>,
    omitted_ignored_by_binary: BTreeMap<String, u64>,
    runtime_aliases: BTreeMap<String, TargetIdentity>,
    matched_selection: bool,
}

impl ListResult {
    fn to_json(&self, process_success: bool) -> Value {
        json!({
            "state": if process_success && self.matched_selection {"verified"} else {"process_failed"},
            "test_count": self.test_count.to_string(),
            "inventory_by_binary": self.inventory.iter().map(|(id,count)| json!({"binary_id":id,"test_count":count.to_string()})).collect::<Vec<_>>(),
            "selection_matches_list": self.matched_selection,
        })
    }
}

fn evaluate_list(process: &ProcessBytes, selection: &Selection) -> Result<ListResult, ReaderError> {
    if process.stdout.declared_truncated || process.stderr.declared_truncated {
        return Err(ReaderError("list capture is truncated".into()));
    }
    let stdout = parse_list_candidate(&process.stdout.bytes);
    let stderr = parse_list_candidate(&process.stderr.bytes);
    let value = match (stdout, stderr) {
        (Some(Ok(_)), Some(Ok(_))) => {
            return Err(ReaderError("list JSON appears in both streams".into()));
        }
        (Some(Err(error)), Some(_)) | (Some(_), Some(Err(error))) => {
            return Err(ReaderError(format!("ambiguous list streams: {error}")));
        }
        (Some(Err(error)), None) | (None, Some(Err(error))) => return Err(error),
        (Some(Ok(value)), None) | (None, Some(Ok(value))) => value,
        (None, None) => return Err(ReaderError("list JSON is unavailable".into())),
    };
    validate_list_value(&value, selection)
}

fn parse_list_candidate(bytes: &[u8]) -> Option<Result<Value, ReaderError>> {
    let trimmed = trim_ascii(bytes);
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with(b"{") {
        return None;
    }
    Some(
        serde_json::from_slice(trimmed)
            .map_err(|error| ReaderError(format!("invalid list JSON: {error}"))),
    )
}

fn validate_list_value(value: &Value, selection: &Selection) -> Result<ListResult, ReaderError> {
    let object = value
        .as_object()
        .ok_or_else(|| ReaderError("list JSON root must be an object".into()))?;
    let test_count = u64_from_value(required(object, "test-count", "list")?, "list.test-count")?;
    let suites = required(object, "rust-suites", "list")?
        .as_object()
        .ok_or_else(|| ReaderError("list.rust-suites must be an object".into()))?;
    let mut inventory = BTreeMap::new();
    let mut omitted_ignored_by_binary = BTreeMap::new();
    let mut runtime_aliases = BTreeMap::new();
    let mut total_cases = 0u64;
    let mut total_omitted_ignored = 0u64;
    for (suite_key, suite_value) in suites {
        let suite = suite_value
            .as_object()
            .ok_or_else(|| ReaderError(format!("list suite {suite_key} must be an object")))?;
        let package = string_field(suite, "package-name", &format!("list suite {suite_key}"))?;
        let binary_id = string_field(suite, "binary-id", &format!("list suite {suite_key}"))?;
        let kind = string_field(suite, "kind", &format!("list suite {suite_key}"))?;
        if binary_id != suite_key {
            return Err(ReaderError(format!(
                "list suite key does not match binary-id: {suite_key}"
            )));
        }
        if selection.protocol == ProtocolVersion::V2
            && !selection.targets.iter().any(|target| {
                target.package == package && target.binary_id == binary_id && target.kind == kind
            })
        {
            return Err(ReaderError(format!(
                "list contains an undeclared target: {binary_id}"
            )));
        }
        if selection.protocol == ProtocolVersion::V2 {
            let binary_name =
                nonempty_string_field(suite, "binary-name", &format!("list suite {suite_key}"))?;
            let runtime_id = format!("{package}::{binary_name}");
            let target = TargetIdentity {
                package: package.to_owned(),
                binary_id: binary_id.to_owned(),
                kind: kind.to_owned(),
            };
            if runtime_aliases.insert(runtime_id.clone(), target).is_some() {
                return Err(ReaderError(format!(
                    "duplicate runtime suite alias in list: {runtime_id}"
                )));
            }
        }
        let testcases = required(suite, "testcases", &format!("list suite {suite_key}"))?
            .as_object()
            .ok_or_else(|| {
                ReaderError(format!(
                    "list suite {suite_key}.testcases must be an object"
                ))
            })?;
        let case_count = u64::try_from(testcases.len())
            .map_err(|_| ReaderError("testcase inventory exceeds u64".into()))?;
        total_cases = total_cases
            .checked_add(case_count)
            .ok_or_else(|| ReaderError("testcase inventory total overflows u64".into()))?;
        if inventory.insert(binary_id.to_owned(), case_count).is_some() {
            return Err(ReaderError(format!(
                "duplicate binary identity in list: {binary_id}"
            )));
        }
        let selected_names = selection
            .tests
            .iter()
            .filter(|test| test.identity.binary_id == binary_id)
            .map(|test| test.identity.test_name.as_str())
            .collect::<BTreeSet<_>>();
        for expected in selection
            .tests
            .iter()
            .filter(|test| test.identity.binary_id == binary_id)
        {
            if expected.identity.package != package || expected.identity.kind != kind {
                return Err(ReaderError(format!(
                    "list suite identity mismatch for {}",
                    expected.identity.event_name()
                )));
            }
            let testcase = testcases.get(&expected.identity.test_name).ok_or_else(|| {
                ReaderError(format!(
                    "selected test is missing from list: {}",
                    expected.identity.event_name()
                ))
            })?;
            let testcase = testcase.as_object().ok_or_else(|| {
                ReaderError(format!(
                    "list testcase is not an object: {}",
                    expected.identity.event_name()
                ))
            })?;
            if let Some(case_kind) = testcase.get("kind") {
                if case_kind.as_str() != Some("test") {
                    return Err(ReaderError(format!(
                        "list testcase kind mismatch for {}",
                        expected.identity.event_name()
                    )));
                }
            }
            let ignored = bool_field(testcase, "ignored", "list testcase")?
                .ok_or_else(|| ReaderError("list testcase ignored must be boolean".into()))?;
            let filter = required(testcase, "filter-match", "list testcase")?
                .as_object()
                .ok_or_else(|| {
                    ReaderError("list testcase filter-match must be an object".into())
                })?;
            ensure_keys(filter, &["status"], "list testcase filter-match")?;
            let filter_status = string_field(filter, "status", "list testcase filter-match")?;
            if ignored != expected.ignored || filter_status != expected.filter_match_status {
                return Err(ReaderError(format!(
                    "list ignored/filter status mismatch for {}",
                    expected.identity.event_name()
                )));
            }
        }
        if selection.protocol == ProtocolVersion::V2 {
            let mut omitted_ignored = 0u64;
            for (test_name, testcase) in testcases {
                if selected_names.contains(test_name.as_str()) {
                    continue;
                }
                let context = format!("list testcase {binary_id}${test_name}");
                let testcase = testcase
                    .as_object()
                    .ok_or_else(|| ReaderError(format!("{context} must be an object")))?;
                if let Some(case_kind) = testcase.get("kind") {
                    if case_kind.as_str() != Some("test") {
                        return Err(ReaderError(format!("{context} kind must be test")));
                    }
                }
                let ignored = bool_field(testcase, "ignored", &context)?
                    .ok_or_else(|| ReaderError(format!("{context}.ignored must be boolean")))?;
                let filter_match = required(testcase, "filter-match", &context)?
                    .as_object()
                    .ok_or_else(|| {
                        ReaderError(format!("{context}.filter-match must be an object"))
                    })?;
                ensure_keys(
                    filter_match,
                    &["status", "reason"],
                    &format!("{context}.filter-match"),
                )?;
                let filter_status =
                    string_field(filter_match, "status", &format!("{context}.filter-match"))?;
                let filter_reason = filter_match
                    .get("reason")
                    .map(|value| {
                        value.as_str().ok_or_else(|| {
                            ReaderError(format!("{context}.filter-match.reason must be a string"))
                        })
                    })
                    .transpose()?;
                match filter_status {
                    "mismatch" if ignored && filter_reason == Some("ignored") => {
                        omitted_ignored = omitted_ignored.checked_add(1).ok_or_else(|| {
                            ReaderError("omitted ignored inventory count overflows u64".into())
                        })?;
                        total_omitted_ignored =
                            total_omitted_ignored.checked_add(1).ok_or_else(|| {
                                ReaderError("omitted ignored inventory count overflows u64".into())
                            })?;
                        if total_omitted_ignored > MAX_V2_OMITTED_IGNORED_TESTS {
                            return Err(ReaderError(format!(
                                "omitted ignored inventory exceeds {MAX_V2_OMITTED_IGNORED_TESTS} tests"
                            )));
                        }
                    }
                    "mismatch" if filter_reason == Some("ignored") => {
                        return Err(ReaderError(format!(
                            "{context} has ignored filter reason without ignored=true"
                        )));
                    }
                    "mismatch" => {}
                    "matches" => {
                        return Err(ReaderError(format!(
                            "unselected testcase matches the trusted filter: {binary_id}${test_name}"
                        )));
                    }
                    _ => {
                        return Err(ReaderError(format!(
                            "{context} has an unsupported filter-match status"
                        )));
                    }
                }
            }
            if omitted_ignored > 0 {
                omitted_ignored_by_binary.insert(binary_id.to_owned(), omitted_ignored);
            }
        }
    }
    if total_cases != test_count {
        return Err(ReaderError(format!(
            "list test-count {} does not match {} testcase entries",
            test_count, total_cases
        )));
    }
    for expected in &selection.tests {
        if !inventory.contains_key(expected.identity.suite_id()) {
            return Err(ReaderError(format!(
                "selected binary is missing from list: {}",
                expected.identity.binary_id
            )));
        }
    }
    if selection.protocol == ProtocolVersion::V2 {
        validate_runtime_aliases(&runtime_aliases)?;
        for target in &selection.targets {
            if !inventory.contains_key(&target.binary_id) {
                return Err(ReaderError(format!(
                    "declared target is missing from list: {}",
                    target.binary_id
                )));
            }
        }
        if inventory.len() != selection.targets.len() {
            return Err(ReaderError(
                "list target inventory does not match the trusted target set".into(),
            ));
        }
    }
    Ok(ListResult {
        test_count,
        inventory,
        omitted_ignored_by_binary,
        runtime_aliases,
        matched_selection: true,
    })
}

fn validate_runtime_aliases(
    runtime_aliases: &BTreeMap<String, TargetIdentity>,
) -> Result<(), ReaderError> {
    let aliases = runtime_aliases.keys().collect::<Vec<_>>();
    for (index, left) in aliases.iter().enumerate() {
        let left_prefix = format!("{left}$");
        for right in aliases.iter().skip(index + 1) {
            let right_prefix = format!("{right}$");
            if left_prefix.starts_with(&right_prefix) || right_prefix.starts_with(&left_prefix) {
                return Err(ReaderError(format!(
                    "ambiguous runtime event aliases in list: {left} and {right}"
                )));
            }
        }
    }
    Ok(())
}

fn admitted_suite_id(
    metadata: &SuiteMetadata,
    selection: &Selection,
    list: Option<&ListResult>,
) -> Result<String, ReaderError> {
    if selection.protocol == ProtocolVersion::V1 {
        return Ok(metadata.binary_id.clone());
    }
    let target = list
        .and_then(|result| result.runtime_aliases.get(&metadata.binary_id))
        .ok_or_else(|| {
            ReaderError(format!(
                "run suite alias is not admitted by the list: {}",
                metadata.binary_id
            ))
        })?;
    if metadata.package != target.package || metadata.kind != target.kind {
        return Err(ReaderError(format!(
            "run suite metadata does not match admitted alias: {}",
            metadata.binary_id
        )));
    }
    Ok(target.binary_id.clone())
}

fn admitted_test_event_name(
    event_name: &str,
    selection: &Selection,
    list: Option<&ListResult>,
) -> Option<String> {
    if selection.protocol == ProtocolVersion::V1 {
        return Some(event_name.to_owned());
    }
    let list = list?;
    list.runtime_aliases
        .iter()
        .find_map(|(runtime_id, target)| {
            let prefix = format!("{runtime_id}$");
            event_name
                .strip_prefix(prefix.as_str())
                .map(|test_name| format!("{}${test_name}", target.binary_id))
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SuiteCounts {
    passed: u64,
    failed: u64,
    ignored: u64,
    measured: u64,
    filtered_out: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SuiteState {
    test_count: u64,
    terminal: Option<(String, SuiteCounts)>,
    closed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestState {
    started: bool,
    terminal: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HumanSummary {
    stream: &'static str,
    run_count: Option<u64>,
    passed: Option<u64>,
    failed: Option<u64>,
    skipped: Option<u64>,
    child_harness: bool,
    nested_test_output: bool,
    malformed_output_context: bool,
    ignored: Option<u64>,
    measured: Option<u64>,
    filtered_out: Option<u64>,
    source_line: String,
}

#[derive(Clone, Debug)]
struct ChildHarnessSummaryCheck {
    state: &'static str,
    reason: Option<String>,
    contradictions: Vec<String>,
}

impl ChildHarnessSummaryCheck {
    fn not_present() -> Self {
        Self {
            state: "not_present",
            reason: None,
            contradictions: Vec::new(),
        }
    }

    fn unverified(reason: impl Into<String>) -> Self {
        Self {
            state: "unverified",
            reason: Some(reason.into()),
            contradictions: Vec::new(),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "state":self.state,
            "reason":self.reason,
            "contradictions":self.contradictions,
        })
    }
}

#[derive(Clone, Debug)]
struct RunResult {
    machine_state: &'static str,
    tests: BTreeMap<String, String>,
    suites: BTreeMap<String, (u64, SuiteCounts)>,
    omitted_ignored_count: u64,
    diagnostics: Vec<HumanSummary>,
    contradictions: Vec<String>,
    child_harness_summary_check: ChildHarnessSummaryCheck,
    count_anomalies: Vec<String>,
    errors: Vec<String>,
    all_selected_passed: bool,
}

impl RunResult {
    fn empty(state: &'static str, error: Option<String>) -> Self {
        Self {
            machine_state: state,
            tests: BTreeMap::new(),
            suites: BTreeMap::new(),
            omitted_ignored_count: 0,
            diagnostics: Vec::new(),
            contradictions: Vec::new(),
            child_harness_summary_check: ChildHarnessSummaryCheck::unverified(
                "structured run result is unavailable",
            ),
            count_anomalies: Vec::new(),
            errors: error.into_iter().collect(),
            all_selected_passed: false,
        }
    }

    fn to_json(&self) -> Value {
        let tests = self
            .tests
            .iter()
            .map(|(identity, outcome)| json!({"identity":identity, "outcome":outcome}))
            .collect::<Vec<_>>();
        let suites = self
            .suites
            .iter()
            .map(|(binary_id, (test_count, counts))| {
                json!({
                    "binary_id":binary_id,
                    "test_count":test_count.to_string(),
                    "passed":counts.passed.to_string(),
                    "failed":counts.failed.to_string(),
                    "ignored":counts.ignored.to_string(),
                    "measured":counts.measured.to_string(),
                    "filtered_out":counts.filtered_out.to_string(),
                })
            })
            .collect::<Vec<_>>();
        let diagnostics = self
            .diagnostics
            .iter()
            .map(|summary| {
                json!({
                    "stream":summary.stream,
                    "domain":if summary.malformed_output_context {"malformed_output_context"} else if summary.nested_test_output {"nested_child_harness"} else if summary.child_harness {"child_harness"} else {"nextest"},
                    "run_count":summary.run_count.map(|value|value.to_string()),
                    "passed":summary.passed.map(|value|value.to_string()),
                    "failed":summary.failed.map(|value|value.to_string()),
                    "skipped":summary.skipped.map(|value|value.to_string()),
                    "ignored":summary.ignored.map(|value|value.to_string()),
                    "measured":summary.measured.map(|value|value.to_string()),
                    "filtered_out":summary.filtered_out.map(|value|value.to_string()),
                    "source_line":summary.source_line,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "machine_state":self.machine_state,
            "selected_tests":tests,
            "suites":suites,
            "human_diagnostics":diagnostics,
            "same_domain_contradictions":self.contradictions,
            "child_harness_summary_check":self.child_harness_summary_check.to_json(),
            "count_anomalies":self.count_anomalies,
            "errors":self.errors,
            "all_selected_passed":self.all_selected_passed,
        })
    }
}

fn evaluate_run(
    invocation: &Invocation,
    selection: &Selection,
    list: Option<&ListResult>,
    process: &ProcessBytes,
) -> RunResult {
    let stdout_lines = split_lines(&process.stdout.bytes);
    let stderr_lines = split_lines(&process.stderr.bytes);
    let stdout_records = contains_machine_records(&stdout_lines);
    let stderr_records = contains_machine_records(&stderr_lines);
    let diagnostics = parse_human_diagnostics(&stdout_lines, "stdout")
        .into_iter()
        .chain(parse_human_diagnostics(&stderr_lines, "stderr"))
        .collect::<Vec<_>>();
    if process.stdout.declared_truncated || process.stderr.declared_truncated {
        let mut result = RunResult::empty("invalid", Some("run capture is truncated".into()));
        result.diagnostics = diagnostics;
        return result;
    }
    if stdout_records && stderr_records {
        let mut result = RunResult::empty(
            "invalid",
            Some(
                "structured run records appear in both streams; cross-stream order is unknown"
                    .into(),
            ),
        );
        result.diagnostics = diagnostics;
        return result;
    }
    let actual_stream = if stdout_records {
        Some(StreamName::Stdout)
    } else if stderr_records {
        Some(StreamName::Stderr)
    } else {
        None
    };
    if let Some(actual) = actual_stream {
        if actual != invocation.machine_stream {
            let mut result = RunResult::empty(
                "invalid",
                Some(
                    "structured records appeared on a stream other than the pinned machine stream"
                        .into(),
                ),
            );
            result.diagnostics = diagnostics;
            return result;
        }
    } else {
        let mut result = RunResult::empty("unavailable", None);
        result.diagnostics = diagnostics;
        return result;
    }
    let machine_bytes = match invocation.machine_stream {
        StreamName::Stdout => &process.stdout.bytes,
        StreamName::Stderr => &process.stderr.bytes,
    };
    if std::str::from_utf8(machine_bytes).is_err() {
        let mut result = RunResult::empty("invalid", Some("machine stream is not UTF-8".into()));
        result.diagnostics = diagnostics;
        return result;
    }
    let lines = match invocation.machine_stream {
        StreamName::Stdout => stdout_lines,
        StreamName::Stderr => stderr_lines,
    };
    let mut result = parse_machine_run(&lines, selection, list);
    result.diagnostics = diagnostics;
    compare_human_summaries(&mut result);
    result
}

fn parse_machine_run(
    lines: &[String],
    selection: &Selection,
    list: Option<&ListResult>,
) -> RunResult {
    let omitted_ignored_count = list
        .map(|result| result.omitted_ignored_by_binary.values().copied().sum())
        .unwrap_or(0);
    let expected_by_event: BTreeMap<String, &ExpectedTest> = selection
        .tests
        .iter()
        .map(|test| (test.identity.event_name(), test))
        .collect();
    let selected_per_suite = selection.tests.iter().fold(
        BTreeMap::<String, (u64, String, String)>::new(),
        |mut acc, test| {
            let entry = acc
                .entry(test.identity.binary_id.clone())
                .or_insert_with(|| (0, test.identity.package.clone(), test.identity.kind.clone()));
            entry.0 += 1;
            acc
        },
    );
    let mut suite_states = BTreeMap::<String, SuiteState>::new();
    let mut test_states = expected_by_event
        .keys()
        .cloned()
        .map(|name| {
            (
                name,
                TestState {
                    started: false,
                    terminal: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    let mut machine_event_count = 0usize;
    for (line_number, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }
        let parsed: Result<Value, _> = serde_json::from_str(trimmed);
        let value = match parsed {
            Ok(value) => value,
            Err(error) if trimmed.contains("\"type\"") => {
                errors.push(format!(
                    "malformed machine JSON on line {}: {error}",
                    line_number + 1
                ));
                continue;
            }
            Err(_) => continue,
        };
        let Some(object) = value.as_object() else {
            continue;
        };
        let Some(event_type) = object.get("type").and_then(Value::as_str) else {
            continue;
        };
        if event_type != "suite" && event_type != "test" {
            errors.push(format!(
                "unknown machine event type on line {}",
                line_number + 1
            ));
            continue;
        }
        machine_event_count += 1;
        if machine_event_count > MAX_MACHINE_EVENTS {
            errors.push("machine event count exceeds the configured limit".into());
            break;
        }
        let event = match object.get("event").and_then(Value::as_str) {
            Some(event) => event,
            None => {
                errors.push(format!(
                    "machine event lacks event name on line {}",
                    line_number + 1
                ));
                continue;
            }
        };
        let allowed_event_fields: &[&str] = match (event_type, event) {
            ("suite", "started") => &["type", "event", "test_count", "nextest"],
            ("suite", "ok" | "failed") => &[
                "type",
                "event",
                "test_count",
                "passed",
                "failed",
                "ignored",
                "measured",
                "filtered_out",
                "exec_time",
                "nextest",
            ],
            ("test", "started") => &["type", "event", "name"],
            ("test", "ok" | "failed" | "ignored") => {
                &["type", "event", "name", "exec_time", "stdout", "stderr"]
            }
            _ => {
                errors.push(format!(
                    "unsupported {event_type} event {event:?} on line {}",
                    line_number + 1
                ));
                continue;
            }
        };
        if let Err(error) = ensure_keys(object, allowed_event_fields, "machine event") {
            errors.push(format!("{} on line {}", error.0, line_number + 1));
            continue;
        }
        match (event_type, event) {
            ("suite", "started") => {
                let metadata = match parse_suite_metadata(object) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let binary_id = match admitted_suite_id(&metadata, selection, list) {
                    Ok(binary_id) => binary_id,
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let expected = selected_per_suite.get(&binary_id);
                let Some((expected_count, expected_package, expected_kind)) = expected else {
                    errors.push(format!("unexpected suite in run output: {binary_id}"));
                    continue;
                };
                let omitted_ignored = list
                    .and_then(|result| result.omitted_ignored_by_binary.get(&binary_id))
                    .copied()
                    .unwrap_or(0);
                let Some(expected_suite_count) = expected_count.checked_add(omitted_ignored) else {
                    errors.push(format!("suite test_count overflows for {binary_id}"));
                    continue;
                };
                if metadata.test_count != Some(expected_suite_count)
                    || metadata.package != *expected_package
                    || metadata.kind != *expected_kind
                {
                    errors.push(format!("suite test_count mismatch for {binary_id}"));
                    continue;
                }
                match suite_states.get_mut(&binary_id) {
                    Some(previous) if previous.closed => errors.push(format!(
                        "suite started after terminal snapshot: {binary_id}"
                    )),
                    Some(previous) if Some(previous.test_count) == metadata.test_count => {}
                    Some(_) => errors.push(format!("conflicting suite start for {binary_id}")),
                    None => {
                        suite_states.insert(
                            binary_id.clone(),
                            SuiteState {
                                test_count: expected_suite_count,
                                terminal: None,
                                closed: false,
                            },
                        );
                    }
                }
            }
            ("test", "started") => {
                let name = match string_field(object, "name", "test event") {
                    Ok(name) => name.to_owned(),
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let Some(stable_name) = admitted_test_event_name(&name, selection, list) else {
                    errors.push(format!("unexpected test start: {name}"));
                    continue;
                };
                let Some(expected) = expected_by_event.get(&stable_name) else {
                    errors.push(format!("unexpected test start: {name}"));
                    continue;
                };
                if !suite_states.contains_key(expected.identity.suite_id()) {
                    errors.push(format!("test started before its suite: {name}"));
                    continue;
                }
                if let Some(state) = test_states.get_mut(&stable_name) {
                    if state.started {
                        errors.push(format!("duplicate test start: {name}"));
                    } else {
                        state.started = true;
                    }
                }
            }
            ("test", "ok" | "failed" | "ignored") => {
                let name = match string_field(object, "name", "test event") {
                    Ok(name) => name.to_owned(),
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let Some(stable_name) = admitted_test_event_name(&name, selection, list) else {
                    errors.push(format!("unexpected terminal test identity: {name}"));
                    continue;
                };
                let Some(expected) = expected_by_event.get(&stable_name) else {
                    errors.push(format!("unexpected terminal test identity: {name}"));
                    continue;
                };
                let Some(state) = test_states.get_mut(&stable_name) else {
                    continue;
                };
                if !state.started {
                    errors.push(format!("test terminal occurred before start: {name}"));
                } else if state.terminal.is_some() {
                    errors.push(format!("duplicate terminal test identity: {name}"));
                } else if suite_states
                    .get(expected.identity.suite_id())
                    .is_some_and(|suite| suite.closed)
                {
                    errors.push(format!("test terminal occurred after suite close: {name}"));
                } else {
                    state.terminal = Some(event.to_owned());
                }
            }
            ("suite", "ok" | "failed") => {
                let metadata = match parse_suite_metadata(object) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let binary_id = match admitted_suite_id(&metadata, selection, list) {
                    Ok(binary_id) => binary_id,
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let Some(state) = suite_states.get_mut(&binary_id) else {
                    errors.push(format!("suite terminal occurred before start: {binary_id}"));
                    continue;
                };
                if metadata
                    .test_count
                    .is_some_and(|test_count| state.test_count != test_count)
                {
                    errors.push(format!(
                        "suite terminal test_count mismatch for {binary_id}"
                    ));
                    continue;
                }
                let counts = match parse_suite_counts(object) {
                    Ok(counts) => counts,
                    Err(error) => {
                        errors.push(error.0);
                        continue;
                    }
                };
                let snapshot = (event.to_owned(), counts);
                if let Some(previous) = &state.terminal {
                    if previous != &snapshot {
                        errors.push(format!(
                            "conflicting repeated suite snapshot for {binary_id}"
                        ));
                    }
                } else {
                    state.terminal = Some(snapshot);
                    state.closed = true;
                }
            }
            _ => unreachable!("event shape was validated above"),
        }
    }
    if machine_event_count == 0 {
        return RunResult::empty("unavailable", None);
    }
    for (name, state) in &test_states {
        if !state.started {
            errors.push(format!("selected test was not started: {name}"));
        }
        if state.terminal.is_none() {
            errors.push(format!("selected test has no terminal outcome: {name}"));
        }
    }
    for (binary_id, state) in &suite_states {
        if state.terminal.is_none() {
            errors.push(format!("suite has no terminal snapshot: {binary_id}"));
        }
    }
    for binary_id in selected_per_suite.keys() {
        if !suite_states.contains_key(binary_id) {
            errors.push(format!(
                "selected suite is missing from run output: {binary_id}"
            ));
        }
    }
    validate_suite_aggregates(&mut errors, &test_states, &suite_states, selection, list);
    let mut tests = BTreeMap::new();
    for (name, state) in &test_states {
        if let Some(outcome) = &state.terminal {
            tests.insert(name.clone(), outcome.clone());
        }
    }
    let mut suites = BTreeMap::new();
    let mut count_anomalies = Vec::new();
    for (binary_id, state) in &suite_states {
        if let Some((_, counts)) = &state.terminal {
            if let Some(listed_count) = list.and_then(|value| value.inventory.get(binary_id)) {
                let run_inventory_count = state.test_count.checked_add(counts.filtered_out);
                let inventory_mismatch = if selection.protocol == ProtocolVersion::V2 {
                    run_inventory_count != Some(*listed_count)
                } else {
                    counts.filtered_out > *listed_count
                };
                if inventory_mismatch {
                    count_anomalies.push(format!(
                        "run count {} plus filtered_out {} does not match listed inventory {} for {}",
                        state.test_count, counts.filtered_out, listed_count, binary_id,
                    ));
                }
            }
            suites.insert(binary_id.clone(), (state.test_count, counts.clone()));
        }
    }
    let all_selected_passed = errors.is_empty()
        && test_states
            .values()
            .all(|state| state.terminal.as_deref() == Some("ok"));
    RunResult {
        machine_state: if errors.is_empty() {
            "verified"
        } else {
            "invalid"
        },
        tests,
        suites,
        omitted_ignored_count,
        diagnostics: Vec::new(),
        contradictions: Vec::new(),
        child_harness_summary_check: ChildHarnessSummaryCheck::unverified(
            "human summaries have not been compared",
        ),
        count_anomalies,
        errors,
        all_selected_passed,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SuiteMetadata {
    binary_id: String,
    package: String,
    kind: String,
    test_count: Option<u64>,
}

fn parse_suite_metadata(object: &Map<String, Value>) -> Result<SuiteMetadata, ReaderError> {
    let test_count = object
        .get("test_count")
        .map(|value| u64_from_value(value, "suite.test_count"))
        .transpose()?;
    let nextest = required(object, "nextest", "suite event")?
        .as_object()
        .ok_or_else(|| ReaderError("suite.nextest must be an object".into()))?;
    ensure_keys(nextest, &["crate", "test_binary", "kind"], "suite.nextest")?;
    let crate_name = nonempty_string_field(nextest, "crate", "suite.nextest")?;
    let test_binary = nonempty_string_field(nextest, "test_binary", "suite.nextest")?;
    let kind = nonempty_string_field(nextest, "kind", "suite.nextest")?;
    if !matches!(kind.as_str(), "test" | "lib" | "example" | "bench") {
        return Err(ReaderError("suite.nextest.kind is not recognized".into()));
    }
    Ok(SuiteMetadata {
        binary_id: format!("{crate_name}::{test_binary}"),
        package: crate_name,
        kind,
        test_count,
    })
}

fn parse_suite_counts(object: &Map<String, Value>) -> Result<SuiteCounts, ReaderError> {
    Ok(SuiteCounts {
        passed: u64_field(object, "passed", "suite terminal")?,
        failed: u64_field(object, "failed", "suite terminal")?,
        ignored: u64_field(object, "ignored", "suite terminal")?,
        measured: u64_field(object, "measured", "suite terminal")?,
        filtered_out: u64_field(object, "filtered_out", "suite terminal")?,
    })
}

fn validate_suite_aggregates(
    errors: &mut Vec<String>,
    tests: &BTreeMap<String, TestState>,
    suites: &BTreeMap<String, SuiteState>,
    selection: &Selection,
    list: Option<&ListResult>,
) {
    for (binary_id, suite) in suites {
        let Some((event, counts)) = &suite.terminal else {
            continue;
        };
        let mut observed = [0u64; 3];
        for expected in selection
            .tests
            .iter()
            .filter(|test| test.identity.binary_id == *binary_id)
        {
            if let Some(outcome) = tests
                .get(&expected.identity.event_name())
                .and_then(|state| state.terminal.as_deref())
            {
                match outcome {
                    "ok" => observed[0] += 1,
                    "failed" => observed[1] += 1,
                    "ignored" => observed[2] += 1,
                    _ => {}
                }
            }
        }
        let terminal_total = counts
            .passed
            .checked_add(counts.failed)
            .and_then(|value| value.checked_add(counts.ignored))
            .and_then(|value| value.checked_add(counts.measured));
        if terminal_total != Some(suite.test_count) {
            errors.push(format!(
                "suite outcome counts do not equal test_count for {binary_id}"
            ));
        }
        let omitted_ignored = list
            .and_then(|result| result.omitted_ignored_by_binary.get(binary_id))
            .copied()
            .unwrap_or(0);
        let observed_ignored = observed[2].checked_add(omitted_ignored);
        if observed[0] != counts.passed
            || observed[1] != counts.failed
            || observed_ignored != Some(counts.ignored)
            || counts.measured != 0
        {
            errors.push(format!(
                "suite counts disagree with terminal test records for {binary_id}"
            ));
        }
        if (*event == "ok") != (counts.failed == 0) {
            errors.push(format!(
                "suite event label contradicts failure count for {binary_id}"
            ));
        }
    }
}

fn compare_human_summaries(result: &mut RunResult) {
    for summary in result
        .diagnostics
        .iter()
        .filter(|summary| summary.malformed_output_context)
    {
        result.contradictions.push(format!(
            "Nextest output block is malformed near: {}",
            summary.source_line
        ));
    }
    let child_summaries = result
        .diagnostics
        .iter()
        .filter(|summary| summary.child_harness && !summary.nested_test_output)
        .collect::<Vec<_>>();
    let has_nested_child_summaries = result
        .diagnostics
        .iter()
        .any(|summary| summary.nested_test_output);
    if result.machine_state != "verified" {
        result.child_harness_summary_check = if child_summaries.is_empty() {
            if has_nested_child_summaries {
                ChildHarnessSummaryCheck::unverified(
                    "nested child-harness summaries are test output, not suite summaries",
                )
            } else {
                ChildHarnessSummaryCheck::not_present()
            }
        } else {
            ChildHarnessSummaryCheck::unverified(
                "structured run results are unavailable for comparison",
            )
        };
        return;
    }
    let expected_passed = result
        .suites
        .values()
        .map(|(_, counts)| counts.passed)
        .try_fold(0u64, u64::checked_add);
    let expected_failed = result
        .suites
        .values()
        .map(|(_, counts)| counts.failed)
        .try_fold(0u64, u64::checked_add);
    let expected_filtered = result
        .suites
        .values()
        .map(|(_, counts)| counts.filtered_out)
        .try_fold(0u64, u64::checked_add);
    let expected_skipped =
        expected_filtered.and_then(|filtered| filtered.checked_add(result.omitted_ignored_count));
    let expected_run_count = u64::try_from(result.tests.len()).ok();
    for summary in result
        .diagnostics
        .iter()
        .filter(|summary| !summary.child_harness)
    {
        if let (Some(observed), Some(expected)) = (summary.run_count, expected_run_count) {
            if observed != expected {
                result.contradictions.push(format!(
                    "Nextest human run count {observed} contradicts structured test count {expected}"
                ));
            }
        }
        if let (Some(observed), Some(expected)) = (summary.passed, expected_passed) {
            if observed != expected {
                result.contradictions.push(format!(
                    "Nextest human passed count {observed} contradicts structured passed count {expected}"
                ));
            }
        }
        if let (Some(observed), Some(expected)) = (summary.failed, expected_failed) {
            if observed != expected {
                result.contradictions.push(format!(
                    "Nextest human failed count {observed} contradicts structured failed count {expected}"
                ));
            }
        }
        if let (Some(observed), Some(expected)) = (summary.skipped, expected_skipped) {
            if observed != expected {
                result.contradictions.push(format!(
                    "Nextest human skipped count {observed} contradicts structured filtered_out count {expected}"
                ));
            }
        }
    }

    if child_summaries.is_empty() {
        result.child_harness_summary_check = if has_nested_child_summaries {
            ChildHarnessSummaryCheck::unverified(
                "nested child-harness summaries are test output, not suite summaries",
            )
        } else {
            ChildHarnessSummaryCheck::not_present()
        };
        return;
    }
    if child_summaries.len() != 1 || result.suites.len() != 1 {
        result.child_harness_summary_check = ChildHarnessSummaryCheck::unverified(
            "child harness summaries cannot be bound one-to-one to a single suite",
        );
        return;
    }

    let summary = child_summaries[0];
    let (binary_id, (_, counts)) = result.suites.iter().next().expect("one suite was checked");
    let mut contradictions = Vec::new();
    for (name, observed, expected) in [
        ("passed", summary.passed, counts.passed),
        ("failed", summary.failed, counts.failed),
        ("ignored", summary.ignored, counts.ignored),
        ("measured", summary.measured, counts.measured),
        ("filtered_out", summary.filtered_out, counts.filtered_out),
    ] {
        if observed != Some(expected) {
            contradictions.push(format!(
                "child harness {name} count {:?} contradicts structured count {expected} for {binary_id}",
                observed
            ));
        }
    }
    if contradictions.is_empty() {
        result.child_harness_summary_check = ChildHarnessSummaryCheck {
            state: "matched",
            reason: None,
            contradictions,
        };
    } else {
        result.child_harness_summary_check = ChildHarnessSummaryCheck {
            state: "contradiction",
            reason: None,
            contradictions,
        };
    }
}

fn contains_machine_records(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            return false;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => value
                .as_object()
                .is_some_and(|object| object.contains_key("type")),
            Err(_) => trimmed.contains("\"type\""),
        }
    })
}

fn split_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn parse_human_diagnostics(lines: &[String], stream: &'static str) -> Vec<HumanSummary> {
    let mut summaries = Vec::new();
    let mut in_nextest_test_output = false;
    for line in lines {
        let trimmed = line.trim();
        if in_nextest_test_output
            && trimmed.starts_with("────────")
            && line.starts_with(trimmed)
        {
            in_nextest_test_output = false;
            continue;
        }
        if trimmed.starts_with("output ─") {
            in_nextest_test_output = true;
            continue;
        }
        if let Some((passed, failed, ignored, measured, filtered_out)) =
            parse_harness_summary(trimmed)
        {
            summaries.push(human_harness_summary(
                stream,
                trimmed,
                (passed, failed, ignored, measured, filtered_out),
                in_nextest_test_output,
            ));
            continue;
        }
        if let Some((run_count, passed, failed, skipped)) = parse_nextest_summary(trimmed) {
            if in_nextest_test_output {
                let mut summary = human_nextest_summary(
                    stream, trimmed, run_count, passed, failed, skipped,
                );
                summary.malformed_output_context = true;
                summaries.push(summary);
            } else {
                summaries.push(human_nextest_summary(
                    stream, trimmed, run_count, passed, failed, skipped,
                ));
            }
        }
    }
    if in_nextest_test_output {
        summaries.push(malformed_output_marker(
            stream,
            "unterminated Nextest test-output block",
        ));
    }
    summaries
}

fn human_nextest_summary(
    stream: &'static str,
    source_line: &str,
    run_count: u64,
    passed: u64,
    failed: Option<u64>,
    skipped: Option<u64>,
) -> HumanSummary {
    HumanSummary {
        stream,
        run_count: Some(run_count),
        passed: Some(passed),
        failed,
        skipped,
        child_harness: false,
        nested_test_output: false,
        malformed_output_context: false,
        ignored: None,
        measured: None,
        filtered_out: None,
        source_line: source_line.to_owned(),
    }
}

fn human_harness_summary(
    stream: &'static str,
    source_line: &str,
    counts: (u64, u64, u64, u64, u64),
    nested_test_output: bool,
) -> HumanSummary {
    let (passed, failed, ignored, measured, filtered_out) = counts;
    HumanSummary {
        stream,
        run_count: None,
        passed: Some(passed),
        failed: Some(failed),
        skipped: None,
        child_harness: true,
        nested_test_output,
        malformed_output_context: false,
        ignored: Some(ignored),
        measured: Some(measured),
        filtered_out: Some(filtered_out),
        source_line: source_line.to_owned(),
    }
}

fn malformed_output_marker(stream: &'static str, source_line: &str) -> HumanSummary {
    HumanSummary {
        stream,
        run_count: None,
        passed: None,
        failed: None,
        skipped: None,
        child_harness: false,
        nested_test_output: false,
        malformed_output_context: true,
        ignored: None,
        measured: None,
        filtered_out: None,
        source_line: source_line.to_owned(),
    }
}

fn parse_nextest_summary(line: &str) -> Option<(u64, u64, Option<u64>, Option<u64>)> {
    let marker = line.find(" tests run").or_else(|| line.find(" test run"))?;
    let prefix = &line[..marker];
    let run_count = prefix.split_whitespace().last()?.parse().ok()?;
    let colon = line[marker..].find(':')? + marker;
    let body = &line[colon + 1..];
    let passed = number_before_label(body, "passed")?;
    let failed = number_before_label(body, "failed");
    let skipped = number_before_label(body, "skipped");
    Some((run_count, passed, failed, skipped))
}

fn parse_harness_summary(line: &str) -> Option<(u64, u64, u64, u64, u64)> {
    if !line.starts_with("test result:") {
        return None;
    }
    Some((
        number_before_label(line, "passed")?,
        number_before_label(line, "failed")?,
        number_before_label(line, "ignored")?,
        number_before_label(line, "measured")?,
        number_before_label(line, "filtered out")?,
    ))
}

fn number_before_label(text: &str, label: &str) -> Option<u64> {
    let index = text.find(label)?;
    text[..index]
        .split_whitespace()
        .last()?
        .trim_matches(',')
        .parse()
        .ok()
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

fn object_with_keys<'a>(
    value: &'a Value,
    context: &str,
    allowed: &[&str],
) -> Result<&'a Map<String, Value>, ReaderError> {
    let object = value
        .as_object()
        .ok_or_else(|| ReaderError(format!("{context} must be an object")))?;
    ensure_keys(object, allowed, context)?;
    Ok(object)
}

fn ensure_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    context: &str,
) -> Result<(), ReaderError> {
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ReaderError(format!(
                "{context} contains unsupported field {key:?}"
            )));
        }
    }
    Ok(())
}

fn required<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a Value, ReaderError> {
    object
        .get(key)
        .ok_or_else(|| ReaderError(format!("{context} is missing required field {key:?}")))
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a str, ReaderError> {
    required(object, key, context)?
        .as_str()
        .ok_or_else(|| ReaderError(format!("{context}.{key} must be a string")))
}

fn nonempty_string_field(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<String, ReaderError> {
    let value = string_field(object, key, context)?;
    if value.is_empty() {
        return Err(ReaderError(format!("{context}.{key} must be nonempty")));
    }
    Ok(value.to_owned())
}

fn optional_string_field(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<Option<String>, ReaderError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ReaderError(format!(
            "{context}.{key} must be a string or null"
        ))),
    }
}

fn bool_field(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<Option<bool>, ReaderError> {
    match object.get(key) {
        None => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ReaderError(format!("{context}.{key} must be boolean"))),
    }
}

fn u64_field(object: &Map<String, Value>, key: &str, context: &str) -> Result<u64, ReaderError> {
    u64_from_value(required(object, key, context)?, &format!("{context}.{key}"))
}

fn u64_from_value(value: &Value, context: &str) -> Result<u64, ReaderError> {
    value
        .as_u64()
        .ok_or_else(|| ReaderError(format!("{context} must be a nonnegative u64")))
}

fn i32_field(object: &Map<String, Value>, key: &str, context: &str) -> Result<i32, ReaderError> {
    let value = required(object, key, context)?
        .as_i64()
        .ok_or_else(|| ReaderError(format!("{context}.{key} must be an integer")))?;
    i32::try_from(value).map_err(|_| ReaderError(format!("{context}.{key} is outside i32")))
}

fn string_array_field(
    object: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<Vec<String>, ReaderError> {
    array_field(object, key, context)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| ReaderError(format!("{context}.{key} must contain only strings")))
        })
        .collect()
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a Vec<Value>, ReaderError> {
    required(object, key, context)?
        .as_array()
        .ok_or_else(|| ReaderError(format!("{context}.{key} must be an array")))
}

fn require_exact_string(
    object: &Map<String, Value>,
    key: &str,
    expected: &str,
    context: &str,
) -> Result<(), ReaderError> {
    if string_field(object, key, context)? != expected {
        return Err(ReaderError(format!(
            "{context}.{key} must equal {expected:?}"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, context: &str) -> Result<(), ReaderError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ReaderError(format!(
            "{context} must be a lowercase SHA-256"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_selection_json() -> Value {
        let tests = (0..EXPECTED_SELECTED_TESTS)
            .map(|index| {
                json!({
                    "package":"pkg",
                    "binary_id":"pkg::bin",
                    "kind":"test",
                    "test_name":format!("case_{index}"),
                    "ignored": index == 0,
                    "filter_match_status":"matches",
                })
            })
            .collect::<Vec<_>>();
        json!({
            "schema":SELECTION_SCHEMA,
            "filter_args":["exact-selected-set"],
            "run_ignored":"all",
            "tests":tests,
        })
    }

    fn fixture_selection() -> Selection {
        parse_selection(&serde_json::to_vec(&fixture_selection_json()).unwrap()).unwrap()
    }

    fn fixture_list_value() -> Value {
        let testcases = (0..EXPECTED_SELECTED_TESTS)
            .map(|index| {
                (
                    format!("case_{index}"),
                    json!({
                        "kind":"test",
                        "ignored":index == 0,
                        "filter-match":{"status":"matches"},
                    }),
                )
            })
            .collect::<Map<_, _>>();
        json!({
            "test-count":EXPECTED_SELECTED_TESTS,
            "rust-suites":{
                "pkg::bin":{
                    "package-name":"pkg",
                    "binary-id":"pkg::bin",
                    "binary-name":"bin",
                    "kind":"test",
                    "status":"listed",
                    "testcases":testcases,
                }
            }
        })
    }

    fn fixture_run(filtered_out: u64) -> Vec<u8> {
        let mut events = vec![json!({
            "type":"suite",
            "event":"started",
            "test_count":EXPECTED_SELECTED_TESTS,
            "nextest":{"crate":"pkg","test_binary":"bin","kind":"test"},
        })];
        for index in 0..EXPECTED_SELECTED_TESTS {
            let name = format!("pkg::bin$case_{index}");
            events.push(json!({"type":"test","event":"started","name":name}));
            events.push(json!({
                "type":"test",
                "event":"ok",
                "name":name,
                "exec_time":0.001,
            }));
        }
        events.push(json!({
            "type":"suite",
            "event":"ok",
            "passed":EXPECTED_SELECTED_TESTS,
            "failed":0,
            "ignored":0,
            "measured":0,
            "filtered_out":filtered_out,
            "exec_time":0.01,
            "nextest":{"crate":"pkg","test_binary":"bin","kind":"test"},
        }));
        let mut bytes = Vec::new();
        for event in events {
            bytes.extend(serde_json::to_vec(&event).unwrap());
            bytes.push(b'\n');
        }
        bytes
    }

    fn v2_target(package: &str, binary_id: &str, kind: &str) -> Value {
        json!({"package":package,"binary_id":binary_id,"kind":kind})
    }

    fn v2_status_selection_json(count: usize, targets: Vec<Value>) -> Value {
        let tests = (0..count)
            .map(|index| {
                json!({
                    "package":"pkg",
                    "binary_id":"pkg::status",
                    "kind":"test",
                    "test_name":format!("status_case_{index}"),
                    "ignored":false,
                    "filter_match_status":"matches",
                })
            })
            .collect::<Vec<_>>();
        json!({
            "schema":SELECTION_SCHEMA_V2,
            "filter_args":[],
            "run_ignored":"default",
            "expected_test_count":count,
            "targets":targets,
            "tests":tests,
        })
    }

    fn v2_status_list_value(count: usize) -> Value {
        let testcases = (0..count)
            .map(|index| {
                (
                    format!("status_case_{index}"),
                    json!({
                        "kind":"test",
                        "ignored":false,
                        "filter-match":{"status":"matches"},
                    }),
                )
            })
            .collect::<Map<_, _>>();
        json!({
            "test-count":count,
            "rust-suites":{
                "pkg::status":{
                    "package-name":"pkg",
                    "binary-id":"pkg::status",
                    "binary-name":"status",
                    "kind":"test",
                    "status":"listed",
                    "testcases":testcases,
                }
            }
        })
    }

    fn v2_status_run_bytes(count: usize) -> Vec<u8> {
        let mut events = vec![json!({
            "type":"suite",
            "event":"started",
            "test_count":count,
            "nextest":{"crate":"pkg","test_binary":"status","kind":"test"},
        })];
        for index in 0..count {
            let name = format!("pkg::status$status_case_{index}");
            events.push(json!({"type":"test","event":"started","name":name}));
            events.push(json!({
                "type":"test",
                "event":"ok",
                "name":name,
                "exec_time":0.001,
            }));
        }
        events.push(json!({
            "type":"suite",
            "event":"ok",
            "passed":count,
            "failed":0,
            "ignored":0,
            "measured":0,
            "filtered_out":0,
            "exec_time":0.1,
            "nextest":{"crate":"pkg","test_binary":"status","kind":"test"},
        }));
        run_bytes(&events)
    }

    fn v2_ignored_inventory_fixture() -> (Selection, Request, Value, Vec<Value>) {
        let targets = vec![v2_target("pkg", "pkg::status", "test")];
        let mut selection_value = v2_status_selection_json(3, targets.clone());
        selection_value["run_ignored"] = json!("all");
        let selection = parse_selection(&serde_json::to_vec(&selection_value).unwrap()).unwrap();
        let mut request_value = v2_status_request_json(3, targets);
        request_value["invocation"]["run_ignored"] = json!("all");
        let request = parse_request(&serde_json::to_vec(&request_value).unwrap()).unwrap();

        let mut list = v2_status_list_value(3);
        let testcases = list["rust-suites"]["pkg::status"]["testcases"]
            .as_object_mut()
            .unwrap();
        testcases.insert(
            "yaml::share::tests::write_edge_corpus_for_external_psych_roundtrip".into(),
            json!({
                "kind":"test",
                "ignored":true,
                "filter-match":{"status":"mismatch","reason":"ignored"},
            }),
        );
        for index in 0..206 {
            testcases.insert(
                format!("filtered_{index}"),
                json!({
                    "kind":"test",
                    "ignored":false,
                    "filter-match":{"status":"mismatch","reason":"expression"},
                }),
            );
        }
        list["test-count"] = json!(210);

        let mut events = event_lines(&v2_status_run_bytes(3));
        events[0]["test_count"] = json!(4);
        let terminal = events.last_mut().unwrap();
        terminal["ignored"] = json!(1);
        terminal["filtered_out"] = json!(206);
        (selection, request, list, events)
    }

    fn v2_ignored_run_bytes(events: &[Value], skipped: u64) -> Vec<u8> {
        let mut bytes = run_bytes(events);
        bytes.extend_from_slice(
            format!("\nSummary [0.01s] 3 tests run: 3 passed, 0 failed, {skipped} skipped\n")
                .as_bytes(),
        );
        bytes.extend_from_slice(
            b"test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 206 filtered out; finished in 0.01s\n",
        );
        bytes
    }

    fn evaluate_v2_ignored_fixture(
        selection: &Selection,
        request: &Request,
        list: &Value,
        events: &[Value],
        skipped: u64,
    ) -> Value {
        evaluate(
            &request.invocation,
            selection,
            &"a".repeat(64),
            &success_process(serde_json::to_vec(list).unwrap(), Vec::new()),
            &success_process(v2_ignored_run_bytes(events, skipped), Vec::new()),
            &success_process(Vec::new(), Vec::new()),
        )
    }

    fn v2_capture_reference(name: &str) -> Value {
        json!({
            "stdout":{
                "path":format!("/tmp/{name}.stdout"),
                "byte_length":0,
                "sha256":crate::sha256_hex(&[]),
                "truncated":false,
            },
            "stderr":{
                "path":format!("/tmp/{name}.stderr"),
                "byte_length":0,
                "sha256":crate::sha256_hex(&[]),
                "truncated":false,
            },
            "termination":{"kind":"exited","exit_code":0},
        })
    }

    fn v2_capture_reference_for(
        stdout_path: &Path,
        stdout: &[u8],
        stderr_path: &Path,
        stderr: &[u8],
    ) -> Value {
        json!({
            "stdout":{
                "path":stdout_path.to_string_lossy(),
                "byte_length":stdout.len(),
                "sha256":crate::sha256_hex(stdout),
                "truncated":false,
            },
            "stderr":{
                "path":stderr_path.to_string_lossy(),
                "byte_length":stderr.len(),
                "sha256":crate::sha256_hex(stderr),
                "truncated":false,
            },
            "termination":{"kind":"exited","exit_code":0},
        })
    }

    fn v2_status_request_json(count: usize, targets: Vec<Value>) -> Value {
        json!({
            "schema":REQUEST_SCHEMA_V2,
            "invocation":{
                "nextest_version":PINNED_NEXTEST_VERSION,
                "message_format":PINNED_MESSAGE_FORMAT,
                "message_format_version":PINNED_MESSAGE_FORMAT_VERSION,
                "libtest_json_enabled":true,
                "retries":0,
                "stress":"none",
                "partition":"none",
                "machine_stream":"stdout",
                "filter_args":[],
                "run_ignored":"default",
                "run_id":"run-v2-test",
                "expected_test_count":count,
                "targets":targets,
            },
            "list_capture":v2_capture_reference("list"),
            "run_capture":v2_capture_reference("run"),
            "wrapper_capture":v2_capture_reference("wrapper"),
        })
    }

    fn v2_multi_target_fixture() -> (Vec<Value>, Value, Value, Vec<u8>) {
        let specs = [
            ("pkg", "pkg", "lib", "pkg", 35usize),
            ("pkg", "pkg::flood", "test", "flood", 35usize),
            ("pkg", "pkg::tui", "test", "tui", 36usize),
            ("pkg", "pkg::tui_shell", "test", "tui_shell", 36usize),
        ];
        let targets = specs
            .iter()
            .map(|(package, binary_id, kind, _, _)| v2_target(package, binary_id, kind))
            .collect::<Vec<_>>();
        let mut tests = Vec::new();
        let mut suites = Map::new();
        let mut events = Vec::new();
        for (target_index, (package, binary_id, kind, binary, count)) in specs.iter().enumerate() {
            let mut testcases = Map::new();
            for index in 0..*count {
                let test_name = format!("case_{target_index}_{index}");
                tests.push(json!({
                    "package":package,
                    "binary_id":binary_id,
                    "kind":kind,
                    "test_name":test_name,
                    "ignored":false,
                    "filter_match_status":"matches",
                }));
                testcases.insert(
                    test_name.clone(),
                    json!({
                        "kind":"test",
                        "ignored":false,
                        "filter-match":{"status":"matches"},
                    }),
                );
            }
            suites.insert(
                (*binary_id).into(),
                json!({
                    "package-name":package,
                    "binary-id":binary_id,
                    "binary-name":binary,
                    "kind":kind,
                    "status":"listed",
                    "testcases":testcases,
                }),
            );
            events.push(json!({
                "type":"suite",
                "event":"started",
                "test_count":count,
                "nextest":{"crate":package,"test_binary":binary,"kind":kind},
            }));
            for index in 0..*count {
                let name = format!("{package}::{binary}$case_{target_index}_{index}");
                events.push(json!({"type":"test","event":"started","name":name}));
                events.push(json!({
                    "type":"test",
                    "event":"ok",
                    "name":name,
                    "exec_time":0.001,
                }));
            }
            events.push(json!({
                "type":"suite",
                "event":"ok",
                "passed":count,
                "failed":0,
                "ignored":0,
                "measured":0,
                "filtered_out":0,
                "exec_time":0.1,
                "nextest":{"crate":package,"test_binary":binary,"kind":kind},
            }));
        }
        let selection = json!({
            "schema":SELECTION_SCHEMA_V2,
            "filter_args":[],
            "run_ignored":"default",
            "expected_test_count":142,
            "targets":targets,
            "tests":tests,
        });
        let list = json!({"test-count":142,"rust-suites":suites});
        (
            specs_to_targets(&specs),
            selection,
            list,
            run_bytes(&events),
        )
    }

    fn specs_to_targets(specs: &[(&str, &str, &str, &str, usize)]) -> Vec<Value> {
        specs
            .iter()
            .map(|(package, binary_id, kind, _, _)| v2_target(package, binary_id, kind))
            .collect()
    }

    fn process(stdout: Vec<u8>, stderr: Vec<u8>, termination: Termination) -> ProcessBytes {
        ProcessBytes {
            stdout: CaptureBytes {
                bytes: stdout,
                declared_truncated: false,
            },
            stderr: CaptureBytes {
                bytes: stderr,
                declared_truncated: false,
            },
            termination,
        }
    }

    fn success_process(stdout: Vec<u8>, stderr: Vec<u8>) -> ProcessBytes {
        process(stdout, stderr, Termination::Exited(0))
    }

    fn fixture_invocation(stream: StreamName) -> Invocation {
        Invocation {
            machine_stream: stream,
            filter_args: vec!["exact-selected-set".into()],
            run_ignored: "all".into(),
            run_id: None,
            protocol: ProtocolVersion::V1,
            expected_test_count: None,
            targets: Vec::new(),
        }
    }

    fn fixture_list_result() -> ListResult {
        ListResult {
            test_count: 9,
            inventory: BTreeMap::from([("pkg::bin".to_owned(), 9)]),
            omitted_ignored_by_binary: BTreeMap::new(),
            runtime_aliases: BTreeMap::new(),
            matched_selection: true,
        }
    }

    const CURSOR_LIB_TESTS: [&str; 4] = [
        "tui::cursor_projection_tests::hidden_cursor_preserves_raw_coordinates_style_and_blink",
        "tui::cursor_projection_tests::hidden_pending_wrap_preserves_column_equal_to_width",
        "tui::cursor_projection_tests::style_code_matrix_preserves_default_and_six_decscusr_shapes",
        "tui::cursor_projection_tests::visibility_requires_backend_and_bounds_without_clamping_raw_position",
    ];

    const CURSOR_SHELL_TESTS: [&str; 5] = [
        "replay::hidden_cup_cursor_coordinates_match_live_and_replay",
        "replay::hidden_pending_wrap_cursor_coordinates_match_live_and_replay",
        "replay::replay_chunk_invariance_all_split_points",
        "replay::replay_matches_live_session",
        "replay::replay_recorded_pty_bytes_chunk_invariant",
    ];

    fn cursor_selection() -> Selection {
        let tests: Vec<Value> = [
            (
                "tuiscotti-runtime::tuiscotti_runtime",
                "lib",
                &CURSOR_LIB_TESTS[..],
            ),
            (
                "tuiscotti-runtime::tui_shell",
                "test",
                &CURSOR_SHELL_TESTS[..],
            ),
        ]
        .into_iter()
        .flat_map(|(binary_id, kind, names)| {
            names.iter().map(move |test_name| {
                json!({
                    "package":"tuiscotti-runtime",
                    "binary_id":binary_id,
                    "kind":kind,
                    "test_name":test_name,
                    "ignored":false,
                    "filter_match_status":"matches",
                })
            })
        })
        .collect();
        let value = json!({
            "schema":SELECTION_SCHEMA,
            "filter_args":["exact-selected-set"],
            "run_ignored":"all",
            "tests":tests,
        });
        parse_selection(&serde_json::to_vec(&value).unwrap()).unwrap()
    }

    fn cursor_list_value() -> Value {
        let mut suites = Map::new();
        for (binary_id, binary_name, kind, names, unselected) in [
            (
                "tuiscotti-runtime::tuiscotti_runtime",
                "tuiscotti_runtime",
                "lib",
                &CURSOR_LIB_TESTS[..],
                68usize,
            ),
            (
                "tuiscotti-runtime::tui_shell",
                "tui_shell",
                "test",
                &CURSOR_SHELL_TESTS[..],
                20usize,
            ),
        ] {
            let mut testcases = Map::new();
            for name in names {
                testcases.insert(
                    (*name).into(),
                    json!({"kind":"test","ignored":false,"filter-match":{"status":"matches"}}),
                );
            }
            for index in 0..unselected {
                testcases.insert(
                    format!("unselected_{index}"),
                    json!({"kind":"test","ignored":false,"filter-match":{"status":"mismatch","reason":"string"}}),
                );
            }
            suites.insert(
                binary_id.into(),
                json!({
                    "package-name":"tuiscotti-runtime",
                    "binary-id":binary_id,
                    "binary-name":binary_name,
                    "kind":kind,
                    "status":"listed",
                    "testcases":testcases,
                }),
            );
        }
        json!({"test-count":97,"rust-suites":suites})
    }

    fn cursor_run_bytes() -> Vec<u8> {
        let mut events = Vec::new();
        for (binary, kind, names, filtered_out) in [
            ("tuiscotti_runtime", "lib", &CURSOR_LIB_TESTS[..], 68u64),
            ("tui_shell", "test", &CURSOR_SHELL_TESTS[..], 20u64),
        ] {
            events.push(json!({
                "type":"suite",
                "event":"started",
                "test_count":names.len(),
                "nextest":{"crate":"tuiscotti-runtime","test_binary":binary,"kind":kind},
            }));
            for name in names {
                let event_name = format!("tuiscotti-runtime::{binary}${name}");
                events.push(json!({"type":"test","event":"started","name":event_name}));
                events.push(json!({"type":"test","event":"ok","name":event_name,"exec_time":0.01}));
            }
            // The pinned Nextest 0.9.146 terminal suite record has no test_count.
            events.push(json!({
                "type":"suite",
                "event":"ok",
                "passed":names.len(),
                "failed":0,
                "ignored":0,
                "measured":0,
                "filtered_out":filtered_out,
                "exec_time":0.1,
                "nextest":{"crate":"tuiscotti-runtime","test_binary":binary,"kind":kind},
            }));
        }
        let mut bytes = run_bytes(&events);
        bytes.extend_from_slice(b"\nSummary [   0.178s] 9 tests run: 9 passed, 88 skipped\n");
        bytes
    }

    fn base_evaluation(run: &ProcessBytes) -> Value {
        let selection = fixture_selection();
        let list = success_process(
            serde_json::to_vec(&fixture_list_value()).unwrap(),
            Vec::new(),
        );
        let wrapper = success_process(Vec::new(), Vec::new());
        evaluate(
            &fixture_invocation(StreamName::Stdout),
            &selection,
            &"a".repeat(64),
            &list,
            run,
            &wrapper,
        )
    }

    fn event_lines(bytes: &[u8]) -> Vec<Value> {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn run_bytes(events: &[Value]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for event in events {
            bytes.extend(serde_json::to_vec(event).unwrap());
            bytes.push(b'\n');
        }
        bytes
    }

    #[test]
    fn v2_accepts_empty_filter_and_exact_59_case_status_selection() {
        let targets = vec![v2_target("pkg", "pkg::status", "test")];
        let selection_bytes =
            serde_json::to_vec(&v2_status_selection_json(59, targets.clone())).unwrap();
        let capture_dir = tempfile::Builder::new()
            .prefix("nextest-v2-captures-")
            .tempdir()
            .unwrap();
        let request_path = capture_dir.path().join("request.json");
        let selection_path = capture_dir.path().join("selection.json");
        let list_stdout = serde_json::to_vec(&v2_status_list_value(59)).unwrap();
        let run_stdout = v2_status_run_bytes(59);
        let empty = Vec::new();
        let files = [
            ("list.stdout", list_stdout.as_slice()),
            ("list.stderr", empty.as_slice()),
            ("run.stdout", run_stdout.as_slice()),
            ("run.stderr", empty.as_slice()),
            ("wrapper.stdout", empty.as_slice()),
            ("wrapper.stderr", empty.as_slice()),
        ];
        for (name, bytes) in files {
            std::fs::write(capture_dir.path().join(name), bytes).unwrap();
        }
        let mut request_value = v2_status_request_json(59, targets);
        request_value["list_capture"] = v2_capture_reference_for(
            &capture_dir.path().join("list.stdout"),
            &list_stdout,
            &capture_dir.path().join("list.stderr"),
            &empty,
        );
        request_value["run_capture"] = v2_capture_reference_for(
            &capture_dir.path().join("run.stdout"),
            &run_stdout,
            &capture_dir.path().join("run.stderr"),
            &empty,
        );
        request_value["wrapper_capture"] = v2_capture_reference_for(
            &capture_dir.path().join("wrapper.stdout"),
            &empty,
            &capture_dir.path().join("wrapper.stderr"),
            &empty,
        );
        std::fs::write(&selection_path, &selection_bytes).unwrap();
        std::fs::write(&request_path, serde_json::to_vec(&request_value).unwrap()).unwrap();
        let report = read_files(
            &request_path,
            &selection_path,
            &crate::sha256_hex(&selection_bytes),
        )
        .unwrap();
        assert_eq!(report["schema"], REPORT_SCHEMA_V2);
        assert_eq!(report["acceptance"], "accepted");
        assert_eq!(report["selection_count"], "59");
        assert_eq!(report["expected_test_count"], 59);
        assert_eq!(report["targets"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn v2_rejects_empty_selection() {
        let targets = vec![v2_target("pkg", "pkg::status", "test")];
        let empty = v2_status_selection_json(0, targets.clone());
        let error = parse_selection(&serde_json::to_vec(&empty).unwrap()).unwrap_err();
        assert!(error.to_string().contains("between 1 and 1024"));
    }

    #[test]
    fn v2_rejects_oversized_selection() {
        let targets = vec![v2_target("pkg", "pkg::status", "test")];
        let oversized = v2_status_selection_json(1025, targets.clone());
        let error = parse_selection(&serde_json::to_vec(&oversized).unwrap()).unwrap_err();
        assert!(error.to_string().contains("between 1 and 1024"));
    }

    #[test]
    fn v2_accepts_142_selected_identities_across_four_targets() {
        let (targets, selection_value, list_value, run_bytes) = v2_multi_target_fixture();
        let selection = parse_selection(&serde_json::to_vec(&selection_value).unwrap()).unwrap();
        assert_eq!(selection.tests.len(), 142);
        assert_eq!(selection.targets.len(), 4);
        let request =
            parse_request(&serde_json::to_vec(&v2_status_request_json(142, targets)).unwrap())
                .unwrap();
        validate_request_selection(&request.invocation, &selection).unwrap();
        let list = success_process(serde_json::to_vec(&list_value).unwrap(), Vec::new());
        assert_eq!(evaluate_list(&list, &selection).unwrap().inventory.len(), 4);
        let report = evaluate(
            &request.invocation,
            &selection,
            &"a".repeat(64),
            &list,
            &success_process(run_bytes, Vec::new()),
            &success_process(Vec::new(), Vec::new()),
        );
        assert_eq!(report["acceptance"], "accepted");
        assert_eq!(report["selection_count"], "142");
        assert_eq!(report["targets"].as_array().unwrap().len(), 4);
        let selected = report["run"]["selected_tests"].as_array().unwrap();
        assert_eq!(selected.len(), 142);
        assert!(selected
            .iter()
            .any(|test| { test["identity"] == "pkg$case_0_0" && test["outcome"] == "ok" }));
    }

    #[test]
    fn v2_reconciles_one_inventory_declared_ignored_test_without_run_event() {
        let (selection, request, list, events) = v2_ignored_inventory_fixture();
        let report = evaluate_v2_ignored_fixture(&selection, &request, &list, &events, 207);

        assert_eq!(report["acceptance"], "accepted");
        assert_eq!(report["list"]["state"], "verified");
        assert_eq!(report["run"]["machine_state"], "verified");
        assert_eq!(report["run"]["selected_tests"].as_array().unwrap().len(), 3);
        assert_eq!(report["run"]["suites"][0]["test_count"], "4");
        assert_eq!(report["run"]["suites"][0]["ignored"], "1");
        assert_eq!(report["run"]["suites"][0]["filtered_out"], "206");
        assert_eq!(report["run"]["human_diagnostics"][0]["run_count"], "3");
        assert_eq!(report["run"]["human_diagnostics"][0]["skipped"], "207");
        assert_eq!(
            report["run"]["child_harness_summary_check"]["state"],
            "matched"
        );
        assert_eq!(report["processes"]["run"]["success"], true);
    }

    #[test]
    fn v2_rejects_untrusted_ignored_inventory_rows_and_count_mismatch() {
        let (selection, _, list, _) = v2_ignored_inventory_fixture();

        let mut wrong_count = list.clone();
        wrong_count["test-count"] = json!(209);
        assert!(
            validate_list_value(&wrong_count, &selection)
                .unwrap_err()
                .to_string()
                .contains("does not match 210 testcase entries")
        );

        let mut nonignored_match = list.clone();
        nonignored_match["test-count"] = json!(211);
        nonignored_match["rust-suites"]["pkg::status"]["testcases"]
            .as_object_mut()
            .unwrap()
            .insert(
                "unselected_match".into(),
                json!({
                    "kind":"test",
                    "ignored":false,
                    "filter-match":{"status":"matches"},
                }),
            );
        assert!(
            validate_list_value(&nonignored_match, &selection)
                .unwrap_err()
                .to_string()
                .contains("unselected testcase matches the trusted filter")
        );

        let mut mislabeled_ignored = list.clone();
        mislabeled_ignored["rust-suites"]["pkg::status"]["testcases"]
            ["yaml::share::tests::write_edge_corpus_for_external_psych_roundtrip"]["ignored"] =
            json!(false);
        assert!(
            validate_list_value(&mislabeled_ignored, &selection)
                .unwrap_err()
                .to_string()
                .contains("ignored filter reason without ignored=true")
        );

        let mut too_many_ignored = list.clone();
        let testcases = too_many_ignored["rust-suites"]["pkg::status"]["testcases"]
            .as_object_mut()
            .unwrap();
        for index in 0..=MAX_V2_OMITTED_IGNORED_TESTS {
            testcases.insert(
                format!("ignored_{index}"),
                json!({
                    "kind":"test",
                    "ignored":true,
                    "filter-match":{"status":"mismatch","reason":"ignored"},
                }),
            );
        }
        too_many_ignored["test-count"] = json!(1235);
        assert!(validate_list_value(&too_many_ignored, &selection)
            .unwrap_err()
            .to_string()
            .contains("omitted ignored inventory exceeds 1024 tests"));

        assert_eq!(selection.run_ignored, "all");
    }

    #[test]
    fn v2_keeps_selected_event_set_exact_and_rejects_ignored_writer_event() {
        let (selection, _, list_value, events) = v2_ignored_inventory_fixture();
        let list = validate_list_value(&list_value, &selection).unwrap();

        let mut missing_selected = events.clone();
        missing_selected.retain(|event| event["name"] != "pkg::status$status_case_0");
        let missing = parse_machine_run(
            &split_lines(&run_bytes(&missing_selected)),
            &selection,
            Some(&list),
        );
        assert_eq!(missing.machine_state, "invalid");
        assert!(
            missing
                .errors
                .iter()
                .any(|error| error.contains("selected test was not started"))
        );

        let mut missing_terminal = events.clone();
        missing_terminal.retain(|event| {
            !(event["type"] == "test"
                && event["event"] == "ok"
                && event["name"] == "pkg::status$status_case_1")
        });
        let missing = parse_machine_run(
            &split_lines(&run_bytes(&missing_terminal)),
            &selection,
            Some(&list),
        );
        assert_eq!(missing.machine_state, "invalid");
        assert!(
            missing
                .errors
                .iter()
                .any(|error| error.contains("selected test has no terminal outcome"))
        );

        let mut unselected_writer = events.clone();
        unselected_writer.insert(
            unselected_writer.len() - 1,
            json!({"type":"test","event":"started","name":"pkg::status$yaml::share::tests::write_edge_corpus_for_external_psych_roundtrip"}),
        );
        let writer = parse_machine_run(
            &split_lines(&run_bytes(&unselected_writer)),
            &selection,
            Some(&list),
        );
        assert_eq!(writer.machine_state, "invalid");
        assert!(
            writer
                .errors
                .iter()
                .any(|error| error.contains(
                    "unexpected test start: pkg::status$yaml::share::tests::write_edge_corpus_for_external_psych_roundtrip"
                ))
        );
    }

    #[test]
    fn v2_blocks_suite_inventory_and_human_summary_count_mismatches() {
        let (selection, request, list_value, events) = v2_ignored_inventory_fixture();
        let list = validate_list_value(&list_value, &selection).unwrap();

        let mut wrong_suite_count = events.clone();
        wrong_suite_count[0]["test_count"] = json!(3);
        let mismatch = parse_machine_run(
            &split_lines(&run_bytes(&wrong_suite_count)),
            &selection,
            Some(&list),
        );
        assert_eq!(mismatch.machine_state, "invalid");
        assert!(
            mismatch
                .errors
                .iter()
                .any(|error| error.contains("suite test_count mismatch"))
        );

        let mut wrong_filter_count = events.clone();
        wrong_filter_count.last_mut().unwrap()["filtered_out"] = json!(205);
        let mismatch = parse_machine_run(
            &split_lines(&run_bytes(&wrong_filter_count)),
            &selection,
            Some(&list),
        );
        assert_eq!(mismatch.machine_state, "verified");
        assert!(!mismatch.count_anomalies.is_empty());

        let report = evaluate_v2_ignored_fixture(&selection, &request, &list_value, &events, 206);
        assert_eq!(report["run"]["machine_state"], "verified");
        assert_eq!(report["acceptance"], "blocked");
        assert!(
            !report["run"]["same_domain_contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn v2_rejects_missing_duplicate_and_ambiguous_runtime_aliases() {
        let (_, selection_value, list_value, _) = v2_multi_target_fixture();
        let selection = parse_selection(&serde_json::to_vec(&selection_value).unwrap()).unwrap();

        let mut missing = list_value.clone();
        missing["rust-suites"]["pkg"]
            .as_object_mut()
            .unwrap()
            .remove("binary-name");
        assert!(validate_list_value(&missing, &selection)
            .unwrap_err()
            .to_string()
            .contains("binary-name"));

        let mut duplicate = list_value.clone();
        duplicate["rust-suites"]["pkg::flood"]["binary-name"] = json!("pkg");
        assert!(validate_list_value(&duplicate, &selection)
            .unwrap_err()
            .to_string()
            .contains("duplicate runtime suite alias"));

        let mut ambiguous = list_value;
        ambiguous["rust-suites"]["pkg::flood"]["binary-name"] = json!("pkg$nested");
        assert!(validate_list_value(&ambiguous, &selection)
            .unwrap_err()
            .to_string()
            .contains("ambiguous runtime event aliases"));
    }

    #[test]
    fn v2_rejects_unadmitted_runtime_metadata_and_test_prefixes() {
        let (_, selection_value, list_value, run) = v2_multi_target_fixture();
        let selection = parse_selection(&serde_json::to_vec(&selection_value).unwrap()).unwrap();
        let list = validate_list_value(&list_value, &selection).unwrap();

        let mut wrong_crate = event_lines(&run);
        wrong_crate[0]["nextest"]["crate"] = json!("other-package");
        let wrong_crate = run_bytes(&wrong_crate);
        let lines = String::from_utf8(wrong_crate)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let result = parse_machine_run(&lines, &selection, Some(&list));
        assert!(result
            .errors
            .iter()
            .any(|error| { error.contains("run suite alias is not admitted by the list") }));

        let mut wrong_kind = event_lines(&run);
        wrong_kind[0]["nextest"]["kind"] = json!("test");
        let wrong_kind = run_bytes(&wrong_kind);
        let lines = String::from_utf8(wrong_kind)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let result = parse_machine_run(&lines, &selection, Some(&list));
        assert!(result
            .errors
            .iter()
            .any(|error| { error.contains("run suite metadata does not match admitted alias") }));

        let mut unknown_test_alias = event_lines(&run);
        unknown_test_alias[1]["name"] = json!("pkg::unlisted$case_0_0");
        let unknown_test_alias = run_bytes(&unknown_test_alias);
        let lines = String::from_utf8(unknown_test_alias)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let result = parse_machine_run(&lines, &selection, Some(&list));
        assert!(result
            .errors
            .iter()
            .any(|error| { error.contains("unexpected test start: pkg::unlisted$case_0_0") }));
    }

    #[test]
    fn v2_keeps_complete_failed_run_outcomes_separate_from_process_success() {
        let (targets, selection_value, list_value, run) = v2_multi_target_fixture();
        let selection = parse_selection(&serde_json::to_vec(&selection_value).unwrap()).unwrap();
        let request =
            parse_request(&serde_json::to_vec(&v2_status_request_json(142, targets)).unwrap())
                .unwrap();
        let mut events = event_lines(&run);
        events[2]["event"] = json!("failed");
        let library_terminal = events
            .iter_mut()
            .find(|event| {
                event["type"] == "suite"
                    && event["event"] == "ok"
                    && event["nextest"]["kind"] == "lib"
            })
            .unwrap();
        library_terminal["event"] = json!("failed");
        library_terminal["passed"] = json!(34);
        library_terminal["failed"] = json!(1);

        let report = evaluate(
            &request.invocation,
            &selection,
            &"a".repeat(64),
            &success_process(serde_json::to_vec(&list_value).unwrap(), Vec::new()),
            &process(run_bytes(&events), Vec::new(), Termination::Exited(100)),
            &success_process(Vec::new(), Vec::new()),
        );
        let run_report = &report["run"];
        assert_eq!(run_report["machine_state"], "verified");
        assert_eq!(run_report["selected_tests"].as_array().unwrap().len(), 142);
        assert_eq!(
            run_report["selected_tests"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|test| test["outcome"] == "failed")
                .count(),
            1
        );
        assert_eq!(run_report["all_selected_passed"], false);
        assert_eq!(run_report["process_success"], false);
        assert_eq!(report["acceptance"], "blocked");
    }

    #[test]
    fn v2_rejects_count_and_target_mismatches() {
        let selected_target = v2_target("pkg", "pkg::status", "test");
        let second_target = v2_target("pkg", "pkg::other", "test");

        let mut count_mismatch = v2_status_selection_json(59, vec![selected_target.clone()]);
        count_mismatch["expected_test_count"] = json!(60);
        let error = parse_selection(&serde_json::to_vec(&count_mismatch).unwrap()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("does not match 59 test identities")
        );

        let selection_bytes =
            serde_json::to_vec(&v2_status_selection_json(59, vec![selected_target.clone()]))
                .unwrap();
        let selection = parse_selection(&selection_bytes).unwrap();
        let request = parse_request(
            &serde_json::to_vec(&v2_status_request_json(59, vec![second_target.clone()])).unwrap(),
        )
        .unwrap();
        assert!(
            validate_request_selection(&request.invocation, &selection)
                .unwrap_err()
                .to_string()
                .contains("targets/count do not match")
        );

        let selection_with_empty_target =
            v2_status_selection_json(59, vec![selected_target, second_target]);
        let error = parse_selection(&serde_json::to_vec(&selection_with_empty_target).unwrap())
            .unwrap_err();
        assert!(error.to_string().contains("has no selected identities"));
    }

    #[test]
    fn validates_exact_nine_selection_and_list_identities() {
        let selection = fixture_selection();
        let list = success_process(
            serde_json::to_vec(&fixture_list_value()).unwrap(),
            Vec::new(),
        );
        let result = evaluate_list(&list, &selection).unwrap();
        assert_eq!(result.test_count, 9);
        assert!(result.matched_selection);
        assert_eq!(result.inventory.get("pkg::bin"), Some(&9));
    }

    #[test]
    fn reads_machine_records_from_the_configured_stderr_stream() {
        let selection = fixture_selection();
        let list = success_process(
            serde_json::to_vec(&fixture_list_value()).unwrap(),
            Vec::new(),
        );
        let run = success_process(Vec::new(), fixture_run(0));
        let wrapper = success_process(Vec::new(), Vec::new());
        let result = evaluate(
            &fixture_invocation(StreamName::Stderr),
            &selection,
            &"a".repeat(64),
            &list,
            &run,
            &wrapper,
        );
        assert_eq!(result["run"]["machine_state"], "verified");
        assert_eq!(result["acceptance"], "accepted");
    }

    #[test]
    fn verifies_nine_observed_tests_across_two_binaries_and_plural_summary() {
        let selection = cursor_selection();
        let list = success_process(
            serde_json::to_vec(&cursor_list_value()).unwrap(),
            Vec::new(),
        );
        let run = success_process(cursor_run_bytes(), Vec::new());
        let wrapper = success_process(Vec::new(), Vec::new());
        let result = evaluate(
            &fixture_invocation(StreamName::Stdout),
            &selection,
            &"a".repeat(64),
            &list,
            &run,
            &wrapper,
        );
        assert_eq!(result["acceptance"], "accepted");
        assert_eq!(result["run"]["selected_tests"].as_array().unwrap().len(), 9);
        assert_eq!(result["run"]["suites"].as_array().unwrap().len(), 2);
        assert_eq!(result["run"]["suites"][0]["filtered_out"], "20");
        assert_eq!(result["run"]["suites"][1]["filtered_out"], "68");
        assert_eq!(result["run"]["human_diagnostics"][0]["run_count"], "9");
        assert_eq!(result["run"]["human_diagnostics"][0]["skipped"], "88");
        assert!(
            result["run"]["same_domain_contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_structured_records_split_across_stdout_and_stderr() {
        let all = event_lines(&fixture_run(0));
        let stdout = run_bytes(&all[..4]);
        let stderr = run_bytes(&all[4..]);
        let run = success_process(stdout, stderr);
        let result = base_evaluation(&run);
        assert_eq!(result["run"]["machine_state"], "invalid");
        assert!(
            result["run"]["errors"][0]
                .as_str()
                .unwrap()
                .contains("both streams")
        );
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn list_ignored_and_filter_status_must_match_static_selection() {
        let selection = fixture_selection();
        let mut list_value = fixture_list_value();
        list_value["rust-suites"]["pkg::bin"]["testcases"]["case_0"]["filter-match"]["status"] =
            Value::String("mismatch".into());
        let list = success_process(serde_json::to_vec(&list_value).unwrap(), Vec::new());
        assert!(
            evaluate_list(&list, &selection)
                .unwrap_err()
                .to_string()
                .contains("ignored/filter status mismatch")
        );
    }

    #[test]
    fn selected_filter_and_ignored_status_must_match_run_policy() {
        let mut value = fixture_selection_json();
        value["tests"][1]["filter_match_status"] = Value::String("mismatch".into());
        assert!(
            parse_selection(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .to_string()
                .contains("must be matches")
        );

        let mut value = fixture_selection_json();
        value["run_ignored"] = Value::String("default".into());
        assert!(
            parse_selection(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .to_string()
                .contains("inconsistent with the run_ignored")
        );
    }

    #[test]
    fn rejects_missing_extra_duplicate_and_suffix_terminal_identities() {
        let selection = fixture_selection();
        let list_value = fixture_list_result();
        let list = Some(&list_value);
        let original = event_lines(&fixture_run(0));

        let mut missing = original.clone();
        missing.retain(|event| event["name"] != "pkg::bin$case_8");
        let missing_result =
            parse_machine_run(&split_lines(&run_bytes(&missing)), &selection, list);
        assert_eq!(missing_result.machine_state, "invalid");

        let mut extra = original.clone();
        extra.insert(
            extra.len() - 1,
            json!({"type":"test","event":"started","name":"pkg::bin$other"}),
        );
        let extra_bytes = run_bytes(&extra);
        let extra_result = parse_machine_run(&split_lines(&extra_bytes), &selection, list);
        assert_eq!(extra_result.machine_state, "invalid");

        let mut duplicate = original.clone();
        let terminal = duplicate
            .iter()
            .find(|event| event["event"] == "ok" && event["type"] == "test")
            .unwrap()
            .clone();
        duplicate.insert(duplicate.len() - 1, terminal);
        let duplicate_result =
            parse_machine_run(&split_lines(&run_bytes(&duplicate)), &selection, list);
        assert_eq!(duplicate_result.machine_state, "invalid");

        let mut suffix = original;
        for event in &mut suffix {
            if event["type"] == "test"
                && event["event"] == "ok"
                && event["name"] == "pkg::bin$case_0"
            {
                event["name"] = Value::String("pkg::bin$case_0-extra".into());
            }
        }
        let suffix_result = parse_machine_run(&split_lines(&run_bytes(&suffix)), &selection, list);
        assert_eq!(suffix_result.machine_state, "invalid");
    }

    #[test]
    fn terminal_identity_before_start_is_invalid() {
        let selection = fixture_selection();
        let list_value = fixture_list_result();
        let list = Some(&list_value);
        let mut events = event_lines(&fixture_run(0));
        events.swap(1, 2);
        let result = parse_machine_run(&split_lines(&run_bytes(&events)), &selection, list);
        assert_eq!(result.machine_state, "invalid");
    }

    #[test]
    fn identical_repeated_suite_snapshots_are_observations_not_extra_tests() {
        let selection = fixture_selection();
        let list_value = fixture_list_result();
        let list = Some(&list_value);
        let mut events = event_lines(&fixture_run(0));
        let start = events[0].clone();
        let terminal = events.last().unwrap().clone();
        events.insert(1, start);
        events.push(terminal);
        let result = parse_machine_run(&split_lines(&run_bytes(&events)), &selection, list);
        assert_eq!(result.machine_state, "verified");
        assert_eq!(result.tests.len(), 9);
    }

    #[test]
    fn rejects_suite_start_after_terminal_snapshot() {
        let selection = fixture_selection();
        let list_value = fixture_list_result();
        let list = Some(&list_value);
        let mut events = event_lines(&fixture_run(0));
        let start = events[0].clone();
        events.push(start);
        let result = parse_machine_run(&split_lines(&run_bytes(&events)), &selection, list);
        assert_eq!(result.machine_state, "invalid");
        assert!(
            result
                .errors
                .iter()
                .any(|error| { error.contains("suite started after terminal snapshot") })
        );
    }

    #[test]
    fn filtered_out_u64_values_are_preserved_and_inventory_anomalies_marked() {
        for filtered_out in [u64::MAX - 22, u64::MAX] {
            let run = success_process(fixture_run(filtered_out), Vec::new());
            let result = base_evaluation(&run);
            assert_eq!(
                result["run"]["suites"][0]["filtered_out"],
                filtered_out.to_string()
            );
            assert!(
                !result["run"]["count_anomalies"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(result["acceptance"], "blocked");
        }
    }

    #[test]
    fn list_ignored_does_not_become_run_filtered_out() {
        let selection = fixture_selection();
        let list_value = fixture_list_result();
        let list = Some(&list_value);
        let result = parse_machine_run(&split_lines(&fixture_run(0)), &selection, list);
        assert_eq!(result.machine_state, "verified");
        assert_eq!(result.suites["pkg::bin"].1.ignored, 0);
        assert_eq!(result.suites["pkg::bin"].1.filtered_out, 0);
    }

    #[test]
    fn human_nextest_contradiction_blocks_without_rewriting_machine_results() {
        let mut stdout = fixture_run(0);
        stdout.extend_from_slice(
            b"\n1 test run: 8 passed, 0 failed, 0 skipped\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s\n",
        );
        let run = success_process(stdout, Vec::new());
        let result = base_evaluation(&run);
        assert_eq!(result["run"]["machine_state"], "verified");
        assert_eq!(result["run"]["selected_tests"].as_array().unwrap().len(), 9);
        assert_eq!(
            result["run"]["human_diagnostics"].as_array().unwrap().len(),
            2
        );
        assert_eq!(result["run"]["human_diagnostics"][0]["domain"], "nextest");
        assert_eq!(
            result["run"]["human_diagnostics"][1]["domain"],
            "child_harness"
        );
        assert!(
            !result["run"]["same_domain_contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn nested_failed_test_summary_is_not_compared_to_nextest_suite_aggregate() {
        let mut events = event_lines(&fixture_run(0));
        for event in &mut events {
            match (event["type"].as_str(), event["event"].as_str()) {
                (Some("test"), Some("ok"))
                    if event["name"] == "pkg::bin$case_8" =>
                {
                    event["event"] = json!("failed");
                }
                (Some("suite"), Some("ok")) => {
                    event["event"] = json!("failed");
                    event["passed"] = json!(8);
                    event["failed"] = json!(1);
                }
                _ => {}
            }
        }
        let stderr = concat!(
            "FAIL [0.01s] pkg case_8\n  output ───\n\n",
            "    running 1 test\n",
            "    test result: FAILED. 0 passed; 1 failed; 0 ignored; ",
            "0 measured; 16 filtered out; finished in 0.00s\n\n",
            "────────────\n",
            "     Summary [0.01s] 9 tests run: 8 passed, 1 failed, 0 skipped\n",
        );
        let run = process(
            run_bytes(&events),
            stderr.as_bytes().to_vec(),
            Termination::Exited(100),
        );
        let result = base_evaluation(&run);

        assert_eq!(result["run"]["machine_state"], "verified");
        assert_eq!(result["run"]["selected_tests"].as_array().unwrap().len(), 9);
        assert_eq!(result["run"]["all_selected_passed"], false);
        assert_eq!(result["run"]["human_diagnostics"].as_array().unwrap().len(), 2);
        assert_eq!(result["run"]["human_diagnostics"][0]["domain"], "nested_child_harness");
        assert_eq!(result["run"]["human_diagnostics"][1]["domain"], "nextest");
        assert_eq!(result["run"]["same_domain_contradictions"].as_array().unwrap().len(), 0);
        assert_eq!(
            result["run"]["child_harness_summary_check"]["state"],
            "unverified"
        );
        assert_eq!(result["processes"]["run"]["termination"]["exit_code"], 100);
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn unclosed_nextest_output_block_cannot_hide_a_human_summary() {
        let stderr = concat!(
            "FAIL [0.01s] pkg case_8\n  output ───\n\n",
            "    test result: FAILED. 0 passed; 1 failed; 0 ignored; ",
            "0 measured; 16 filtered out; finished in 0.00s\n",
            "     Summary [0.01s] 9 tests run: 9 passed, 0 failed, 0 skipped\n",
        );
        let run = success_process(fixture_run(0), stderr.as_bytes().to_vec());
        let result = base_evaluation(&run);

        assert_eq!(result["run"]["machine_state"], "verified");
        assert_eq!(result["run"]["all_selected_passed"], true);
        assert!(
            result["run"]["human_diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|summary| summary["domain"] == "malformed_output_context")
        );
        assert!(
            !result["run"]["same_domain_contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn child_summary_is_checked_only_when_bound_to_one_suite() {
        let mut single_suite = fixture_run(0);
        single_suite.extend_from_slice(
            b"\ntest result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
        );
        let mismatch = base_evaluation(&success_process(single_suite, Vec::new()));
        assert_eq!(mismatch["run"]["machine_state"], "verified");
        assert_eq!(
            mismatch["run"]["child_harness_summary_check"]["state"],
            "contradiction"
        );
        assert!(
            !mismatch["run"]["child_harness_summary_check"]["contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(mismatch["acceptance"], "blocked");

        let mut multiple_suites = cursor_run_bytes();
        multiple_suites.extend_from_slice(
            b"\ntest result: ok. 400 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\ntest result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s\n",
        );
        let selection = cursor_selection();
        let list = success_process(
            serde_json::to_vec(&cursor_list_value()).unwrap(),
            Vec::new(),
        );
        let run = success_process(multiple_suites, Vec::new());
        let wrapper = success_process(Vec::new(), Vec::new());
        let unbound = evaluate(
            &fixture_invocation(StreamName::Stdout),
            &selection,
            &"a".repeat(64),
            &list,
            &run,
            &wrapper,
        );
        assert_eq!(unbound["run"]["machine_state"], "verified");
        assert_eq!(
            unbound["run"]["child_harness_summary_check"]["state"],
            "unverified"
        );
        assert!(
            unbound["run"]["child_harness_summary_check"]["contradictions"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(unbound["acceptance"], "accepted");
    }

    #[test]
    fn human_only_capture_is_unavailable_not_a_pass() {
        let run = success_process(
            Vec::new(),
            b"1 test run: 1 passed, 16 skipped\nrunning 1 test\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s\n".to_vec(),
        );
        let result = base_evaluation(&run);
        assert_eq!(result["run"]["machine_state"], "unavailable");
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn preserves_child_pass_and_wrapper_125_as_distinct_process_facts() {
        let selection = fixture_selection();
        let list = success_process(
            serde_json::to_vec(&fixture_list_value()).unwrap(),
            Vec::new(),
        );
        let run = success_process(fixture_run(0), Vec::new());
        let wrapper = process(
            Vec::new(),
            b"wrapper failed after child pass".to_vec(),
            Termination::Exited(125),
        );
        let result = evaluate(
            &fixture_invocation(StreamName::Stdout),
            &selection,
            &"a".repeat(64),
            &list,
            &run,
            &wrapper,
        );
        assert_eq!(result["run"]["machine_state"], "verified");
        assert_eq!(result["processes"]["run"]["termination"]["exit_code"], 0);
        assert_eq!(
            result["processes"]["wrapper"]["termination"]["exit_code"],
            125
        );
        assert_eq!(result["acceptance"], "blocked");
    }

    #[test]
    fn rejects_other_format_and_retry_stress_or_partition_invocations() {
        let mut request = json!({
            "schema":REQUEST_SCHEMA,
            "invocation":{
                "nextest_version":PINNED_NEXTEST_VERSION,
                "message_format":PINNED_MESSAGE_FORMAT,
                "message_format_version":PINNED_MESSAGE_FORMAT_VERSION,
                "libtest_json_enabled":true,
                "retries":0,
                "stress":"none",
                "partition":"none",
                "machine_stream":"stdout",
                "filter_args":["exact-selected-set"],
                "run_ignored":"all",
            },
            "list_capture":{},
            "run_capture":{},
            "wrapper_capture":{},
        });
        request["invocation"]["retries"] = json!(1);
        assert!(
            parse_request(&serde_json::to_vec(&request).unwrap())
                .unwrap_err()
                .to_string()
                .contains("retries")
        );
        request["invocation"]["retries"] = json!(0);
        request["invocation"]["message_format_version"] = json!("0.2");
        assert!(
            parse_request(&serde_json::to_vec(&request).unwrap())
                .unwrap_err()
                .to_string()
                .contains("message_format_version")
        );
    }

    #[test]
    fn request_file_capture_metadata_and_trusted_selection_hash_are_checked() {
        let directory = tempfile::tempdir().unwrap();
        let selection_bytes = serde_json::to_vec(&fixture_selection_json()).unwrap();
        let selection_hash = crate::sha256_hex(&selection_bytes);
        let selection_path = directory.path().join("selection.json");
        fs::write(&selection_path, &selection_bytes).unwrap();
        let list_bytes = serde_json::to_vec(&fixture_list_value()).unwrap();
        let run_bytes = fixture_run(0);
        let empty = Vec::new();
        let write_capture = |name: &str, bytes: &[u8]| {
            let path = directory.path().join(name);
            fs::write(&path, bytes).unwrap();
            json!({
                "path":path.to_string_lossy(),
                "byte_length":bytes.len(),
                "sha256":crate::sha256_hex(bytes),
                "truncated":false,
            })
        };
        let process_value = |stdout: Value, stderr: Value| {
            json!({
                "stdout":stdout,
                "stderr":stderr,
                "termination":{"kind":"exited","exit_code":0},
            })
        };
        let request = json!({
            "schema":REQUEST_SCHEMA,
            "invocation":{
                "nextest_version":PINNED_NEXTEST_VERSION,
                "message_format":PINNED_MESSAGE_FORMAT,
                "message_format_version":PINNED_MESSAGE_FORMAT_VERSION,
                "libtest_json_enabled":true,
                "retries":0,
                "stress":"none",
                "partition":"none",
                "machine_stream":"stdout",
                "filter_args":["exact-selected-set"],
                "run_ignored":"all",
            },
            "list_capture":process_value(write_capture("list.json", &list_bytes), write_capture("list.err", &empty)),
            "run_capture":process_value(write_capture("run.jsonl", &run_bytes), write_capture("run.err", &empty)),
            "wrapper_capture":process_value(write_capture("wrapper.out", &empty), write_capture("wrapper.err", &empty)),
        });
        let request_path = directory.path().join("request.json");
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let report = read_files(&request_path, &selection_path, &selection_hash).unwrap();
        assert_eq!(report["acceptance"], "accepted");
        assert!(read_files(&request_path, &selection_path, &"b".repeat(64)).is_err());
        let changed_capture = directory.path().join("run.jsonl");
        fs::write(changed_capture, b"tampered").unwrap();
        assert!(
            read_files(&request_path, &selection_path, &selection_hash)
                .unwrap_err()
                .to_string()
                .contains("byte length changed")
        );
    }
}
