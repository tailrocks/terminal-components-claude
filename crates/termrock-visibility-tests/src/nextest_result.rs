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
pub const SELECTION_SCHEMA: &str = "termrock-nextest-selection/v1";
pub const REPORT_SCHEMA: &str = "termrock-nextest-reader-report/v1";
pub const PINNED_NEXTEST_VERSION: &str = "0.9.146";
pub const PINNED_MESSAGE_FORMAT: &str = "libtest-json-plus";
pub const PINNED_MESSAGE_FORMAT_VERSION: &str = "0.1";
pub const EXPECTED_SELECTED_TESTS: usize = 9;
pub const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
pub const MAX_SELECTION_BYTES: u64 = 1024 * 1024;
pub const MAX_CAPTURE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_MACHINE_EVENTS: usize = 100_000;

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
    if request.invocation.filter_args != selection.filter_args
        || request.invocation.run_ignored != selection.run_ignored
    {
        return Err(ReaderError(
            "invocation filter policy does not match the trusted selection".into(),
        ));
    }
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
    if string_field(object, "schema", "request")? != REQUEST_SCHEMA {
        return Err(ReaderError("unsupported request schema".into()));
    }
    let invocation_object = object_with_keys(
        required(object, "invocation", "request")?,
        "invocation",
        &[
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
        ],
    )?;
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
    if filter_args.is_empty() || filter_args.iter().any(String::is_empty) {
        return Err(ReaderError(
            "invocation filter_args must be nonempty strings".into(),
        ));
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
    Ok(Request {
        invocation: Invocation {
            machine_stream,
            filter_args,
            run_ignored: run_ignored.to_owned(),
            run_id,
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
    let object = object_with_keys(
        &value,
        "selection",
        &["schema", "filter_args", "run_ignored", "tests"],
    )?;
    if string_field(object, "schema", "selection")? != SELECTION_SCHEMA {
        return Err(ReaderError("unsupported selection schema".into()));
    }
    let filter_args = string_array_field(object, "filter_args", "selection")?;
    if filter_args.is_empty() || filter_args.iter().any(String::is_empty) {
        return Err(ReaderError(
            "selection filter_args must be nonempty strings".into(),
        ));
    }
    let run_ignored = string_field(object, "run_ignored", "selection")?;
    if !matches!(run_ignored, "default" | "only" | "all") {
        return Err(ReaderError(
            "selection run_ignored policy is unsupported".into(),
        ));
    }
    let test_values = array_field(object, "tests", "selection")?;
    if test_values.len() != EXPECTED_SELECTED_TESTS {
        return Err(ReaderError(format!(
            "selection must contain exactly {EXPECTED_SELECTED_TESTS} test identities"
        )));
    }
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
    tests.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(Selection {
        filter_args,
        run_ignored: run_ignored.to_owned(),
        tests,
    })
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
    json!({
        "schema": REPORT_SCHEMA,
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
    })
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
    let mut total_cases = 0u64;
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
    Ok(ListResult {
        test_count,
        inventory,
        matched_selection: true,
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
                    "domain":if summary.child_harness {"child_harness"} else {"nextest"},
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
                let expected = selected_per_suite.get(&metadata.binary_id);
                let Some((expected_count, expected_package, expected_kind)) = expected else {
                    errors.push(format!(
                        "unexpected suite in run output: {}",
                        metadata.binary_id
                    ));
                    continue;
                };
                if metadata.test_count != Some(*expected_count)
                    || metadata.package != *expected_package
                    || metadata.kind != *expected_kind
                {
                    errors.push(format!(
                        "suite test_count mismatch for {}",
                        metadata.binary_id
                    ));
                    continue;
                }
                match suite_states.get_mut(&metadata.binary_id) {
                    Some(previous) if previous.closed => errors.push(format!(
                        "suite started after terminal snapshot: {}",
                        metadata.binary_id
                    )),
                    Some(previous) if Some(previous.test_count) == metadata.test_count => {}
                    Some(_) => errors.push(format!(
                        "conflicting suite start for {}",
                        metadata.binary_id
                    )),
                    None => {
                        suite_states.insert(
                            metadata.binary_id,
                            SuiteState {
                                test_count: *expected_count,
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
                let Some(expected) = expected_by_event.get(&name) else {
                    errors.push(format!("unexpected test start: {name}"));
                    continue;
                };
                if !suite_states.contains_key(expected.identity.suite_id()) {
                    errors.push(format!("test started before its suite: {name}"));
                    continue;
                }
                if let Some(state) = test_states.get_mut(&name) {
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
                let Some(expected) = expected_by_event.get(&name) else {
                    errors.push(format!("unexpected terminal test identity: {name}"));
                    continue;
                };
                let Some(state) = test_states.get_mut(&name) else {
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
                let Some(state) = suite_states.get_mut(&metadata.binary_id) else {
                    errors.push(format!(
                        "suite terminal occurred before start: {}",
                        metadata.binary_id
                    ));
                    continue;
                };
                if metadata
                    .test_count
                    .is_some_and(|test_count| state.test_count != test_count)
                {
                    errors.push(format!(
                        "suite terminal test_count mismatch for {}",
                        metadata.binary_id
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
                            "conflicting repeated suite snapshot for {}",
                            metadata.binary_id
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
    validate_suite_aggregates(&mut errors, &test_states, &suite_states, selection);
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
                if counts.filtered_out > *listed_count {
                    count_anomalies.push(format!(
                        "filtered_out {} exceeds listed inventory {} for {}",
                        counts.filtered_out, listed_count, binary_id
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
        if observed != [counts.passed, counts.failed, counts.ignored] || counts.measured != 0 {
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
    let child_summaries = result
        .diagnostics
        .iter()
        .filter(|summary| summary.child_harness)
        .collect::<Vec<_>>();
    if result.machine_state != "verified" {
        result.child_harness_summary_check = if child_summaries.is_empty() {
            ChildHarnessSummaryCheck::not_present()
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
    let expected_skipped = result
        .suites
        .values()
        .map(|(_, counts)| counts.filtered_out)
        .try_fold(0u64, u64::checked_add);
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
        result.child_harness_summary_check = ChildHarnessSummaryCheck::not_present();
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
    for line in lines {
        let trimmed = line.trim();
        if let Some((run_count, passed, failed, skipped)) = parse_nextest_summary(trimmed) {
            summaries.push(HumanSummary {
                stream,
                run_count: Some(run_count),
                passed: Some(passed),
                failed,
                skipped,
                child_harness: false,
                ignored: None,
                measured: None,
                filtered_out: None,
                source_line: trimmed.to_owned(),
            });
        } else if let Some((passed, failed, ignored, measured, filtered_out)) =
            parse_harness_summary(trimmed)
        {
            summaries.push(HumanSummary {
                stream,
                run_count: None,
                passed: Some(passed),
                failed: Some(failed),
                skipped: None,
                child_harness: true,
                ignored: Some(ignored),
                measured: Some(measured),
                filtered_out: Some(filtered_out),
                source_line: trimmed.to_owned(),
            });
        }
    }
    summaries
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
        }
    }

    fn fixture_list_result() -> ListResult {
        ListResult {
            test_count: 9,
            inventory: BTreeMap::from([("pkg::bin".to_owned(), 9)]),
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
