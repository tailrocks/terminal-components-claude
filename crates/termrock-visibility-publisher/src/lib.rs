//! Read-only ingestion of GitHub Actions run metadata and observer artifacts.
//!
//! The transport is deliberately GET-only. This crate never checks out source,
//! writes Git refs, executes an artifact, or writes downloaded bytes to disk.

pub mod api;
pub mod observer;

use api::{
    ApiClient, ApiError, ArtifactRecord, JobRecord, Repository, RunRecord, RunSelector,
    TargetResolution,
};
use observer::{ObserverError, decode_observer_zip};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_OBSERVER_ZIP_BYTES: usize = api::MAX_ARTIFACT_DOWNLOAD_BYTES;
pub const MAX_OBSERVER_JSON_BYTES: usize = 1024 * 1024;
pub const OBSERVER_REPORT_SCHEMA_VERSION: u16 = 2;

/// Expected identity of a workflow run, including the workflow source pin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunIdentity {
    pub repository: Repository,
    pub run_id: u64,
    pub attempt: u32,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub event: String,
    pub branch: String,
    pub head_sha: String,
}

/// Caller context supplied by the generated publisher workflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallerIdentity {
    pub run: RunIdentity,
    pub workflow_ref: String,
    pub workflow_sha: String,
}

/// The complete read-only input for one ingestion operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngestRequest {
    pub caller: CallerIdentity,
    pub target: RunSelector,
    pub observer: RunIdentity,
    /// The expected workflow artifact name. It must contain the observer run
    /// ID and attempt as distinct decimal tokens.
    pub observer_artifact_name: String,
}

/// Strict V2 form of the selected producer's root `ci-observer-report.json`.
/// V1 reports do not carry a version or observer identity and are deliberately
/// rejected rather than interpreted by guessing from their other fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverReportV2 {
    pub schema_version: u16,
    pub observer_run_id: u64,
    pub observer_run_attempt: u32,
    pub observer_state: String,
    pub target_run_state: String,
    pub run_id: Option<u64>,
    pub run_attempt: Option<u32>,
    pub event: String,
    pub branch: String,
    pub event_sha: String,
    pub run_source_sha: Option<String>,
    pub conclusion: Option<String>,
    pub jobs_count: Option<usize>,
    pub jobs_scope: String,
    pub artifacts_count: Option<usize>,
    pub artifacts_scope: String,
    pub workflow_source_blob_sha: Option<String>,
    pub workflow_source_bytes: Option<usize>,
    pub workflow_source_sha256: Option<String>,
    pub source_state: String,
    pub product_execution: String,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngestedObservation {
    pub caller: RunRecord,
    pub target: RunRecord,
    pub target_jobs: Vec<JobRecord>,
    pub observer: RunRecord,
    pub artifact: ArtifactRecord,
    pub archive_sha256: String,
    pub observer_report: ObserverReportV2,
    pub caller_workflow_sha256: String,
    pub caller_workflow_bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum IngestError {
    InvalidInput(&'static str),
    DuplicateRunIds,
    Api(ApiError),
    CallerMismatch(&'static str),
    TargetMissing,
    TargetAmbiguous,
    TargetInProgress { run_id: u64, status: String },
    ObserverMismatch(&'static str),
    ArtifactMissing,
    ArtifactAmbiguous,
    ArtifactMismatch(&'static str),
    ArtifactDigestMismatch,
    ObserverMismatchData(&'static str),
    Observer(ObserverError),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid ingestion input: {message}"),
            Self::DuplicateRunIds => f.write_str("caller, target, and observer run IDs must be distinct"),
            Self::Api(error) => write!(f, "GitHub API read failed: {error}"),
            Self::CallerMismatch(field) => write!(f, "caller run does not match expected {field}"),
            Self::TargetMissing => f.write_str("no matching target workflow run was found"),
            Self::TargetAmbiguous => f.write_str("multiple newest target workflow runs match"),
            Self::TargetInProgress { run_id, status } => {
                write!(f, "newest target run {run_id} is not complete (status {status})")
            }
            Self::ObserverMismatch(field) => write!(f, "observer run does not match expected {field}"),
            Self::ArtifactMissing => f.write_str("observer artifact is missing"),
            Self::ArtifactAmbiguous => f.write_str("multiple observer artifacts match"),
            Self::ArtifactMismatch(field) => write!(f, "observer artifact does not match expected {field}"),
            Self::ArtifactDigestMismatch => f.write_str("observer artifact digest does not match downloaded bytes"),
            Self::ObserverMismatchData(field) => write!(f, "observer JSON does not match resolved {field}"),
            Self::Observer(error) => write!(f, "invalid observer artifact: {error}"),
        }
    }
}

impl std::error::Error for IngestError {}

impl From<ApiError> for IngestError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
    }
}

impl From<ObserverError> for IngestError {
    fn from(value: ObserverError) -> Self {
        Self::Observer(value)
    }
}

/// Resolve the caller, target CI run, observer run, and the inert observer
/// artifact. All network access is mediated by the GET-only transport.
pub fn ingest<T: api::ReadOnlyTransport>(
    client: &ApiClient<T>,
    request: &IngestRequest,
) -> Result<IngestedObservation, IngestError> {
    validate_request(request)?;

    let caller = client.get_run(&request.caller.run.repository, request.caller.run.run_id)?;
    validate_run_identity(&caller, &request.caller.run, true)?;
    let workflow = client.get_workflow_file(
        &request.caller.run.repository,
        &request.caller.run.workflow_path,
        &request.caller.workflow_sha,
    )?;

    let target = match client.resolve_target(&request.target)? {
        TargetResolution::Missing => return Err(IngestError::TargetMissing),
        TargetResolution::Ambiguous => return Err(IngestError::TargetAmbiguous),
        TargetResolution::Found(run) => run,
    };
    if target.status != "completed" {
        return Err(IngestError::TargetInProgress {
            run_id: target.id,
            status: target.status.clone(),
        });
    }

    if target.id == caller.id || target.id == request.observer.run_id {
        return Err(IngestError::DuplicateRunIds);
    }
    let observer = client.get_run(&request.observer.repository, request.observer.run_id)?;
    validate_run_identity(&observer, &request.observer, false)
        .map_err(|_| IngestError::ObserverMismatch("run identity"))?;

    let target_jobs = client.list_jobs(&request.target.repository, target.id, target.run_attempt)?;
    let artifacts = client.list_artifacts(&request.observer.repository, observer.id)?;
    let matching: Vec<_> = artifacts
        .into_iter()
        .filter(|artifact| artifact.name == request.observer_artifact_name)
        .collect();
    let artifact = match matching.as_slice() {
        [] => return Err(IngestError::ArtifactMissing),
        [artifact] => artifact.clone(),
        _ => return Err(IngestError::ArtifactAmbiguous),
    };
    validate_artifact(&artifact, &request.observer, &request.observer_artifact_name)?;

    let archive = client.download_artifact(
        &request.observer.repository,
        artifact.id,
        MAX_OBSERVER_ZIP_BYTES,
    )?;
    if archive.len() as u64 != artifact.size_in_bytes {
        return Err(IngestError::ArtifactMismatch("archive byte count"));
    }
    let archive_sha256 = sha256_hex(&archive);
    let expected_digest = artifact
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .ok_or(IngestError::ArtifactMismatch("SHA-256 digest"))?;
    if expected_digest != archive_sha256 {
        return Err(IngestError::ArtifactDigestMismatch);
    }
    let json = decode_observer_zip(&archive, MAX_OBSERVER_ZIP_BYTES, MAX_OBSERVER_JSON_BYTES)?;
    let observer_report: ObserverReportV2 = serde_json::from_slice(&json)
        .map_err(|_| ObserverError::InvalidJson)?;
    validate_observer_report(&observer_report, request, &target, &target_jobs)?;

    Ok(IngestedObservation {
        caller,
        target,
        target_jobs,
        observer,
        artifact,
        archive_sha256,
        observer_report,
        caller_workflow_sha256: workflow.sha256,
        caller_workflow_bytes: workflow.bytes,
    })
}

fn validate_request(request: &IngestRequest) -> Result<(), IngestError> {
    let caller = &request.caller.run;
    let observer = &request.observer;
    let target = &request.target;
    if caller.run_id == 0 || caller.attempt == 0 || observer.run_id == 0 || observer.attempt == 0 {
        return Err(IngestError::InvalidInput("run IDs and attempts must be positive"));
    }
    if caller.workflow_id == 0 || observer.workflow_id == 0 || target.workflow_id == 0 {
        return Err(IngestError::InvalidInput("workflow IDs must be positive"));
    }
    if caller.run_id == observer.run_id {
        return Err(IngestError::DuplicateRunIds);
    }
    if caller.repository != observer.repository || caller.repository != target.repository {
        return Err(IngestError::InvalidInput("all identities must use the accepted repository"));
    }
    if !is_sha1(&caller.head_sha) || !is_sha1(&observer.head_sha)
        || !is_sha1(&target.head_sha) || !is_sha1(&request.caller.workflow_sha)
    {
        return Err(IngestError::InvalidInput("commit SHAs must be 40 lowercase hexadecimal characters"));
    }
    if !safe_workflow_path(&caller.workflow_path) || !safe_workflow_path(&observer.workflow_path)
        || !safe_workflow_path(&target.workflow_path)
    {
        return Err(IngestError::InvalidInput("workflow paths must be safe repository-relative paths"));
    }
    if !valid_ref_name(&caller.branch) || !valid_ref_name(&observer.branch)
        || !valid_ref_name(&target.branch) || caller.event.is_empty()
        || observer.event.is_empty() || target.event.is_empty()
    {
        return Err(IngestError::InvalidInput("branches and events must be non-empty and safe"));
    }
    if caller.branch != observer.branch || caller.branch != target.branch
        || caller.head_sha != observer.head_sha || caller.head_sha != target.head_sha
    {
        return Err(IngestError::InvalidInput("caller, target, and observer must refer to the same branch and commit"));
    }
    let expected_ref = format!(
        "{}/{}@refs/heads/{}",
        caller.repository.full_name(), caller.workflow_path, caller.branch
    );
    if request.caller.workflow_ref != expected_ref {
        return Err(IngestError::InvalidInput("caller workflow_ref does not match repository/path/branch"));
    }
    if request.observer_artifact_name.is_empty()
        || !artifact_name_binds_run(&request.observer_artifact_name, observer.run_id, observer.attempt)
    {
        return Err(IngestError::InvalidInput("artifact name must contain the observer run ID and attempt tokens"));
    }
    Ok(())
}

fn validate_run_identity(
    run: &RunRecord,
    expected: &RunIdentity,
    caller: bool,
) -> Result<(), IngestError> {
    let mismatch = |field| {
        if caller { IngestError::CallerMismatch(field) } else { IngestError::ObserverMismatch(field) }
    };
    if run.id != expected.run_id { return Err(mismatch("run ID")); }
    if run.run_attempt != expected.attempt { return Err(mismatch("attempt")); }
    if run.workflow_id != expected.workflow_id { return Err(mismatch("workflow ID")); }
    if run.repository != expected.repository.full_name() { return Err(mismatch("repository")); }
    if run.workflow_path != expected.workflow_path { return Err(mismatch("workflow path")); }
    if run.event != expected.event { return Err(mismatch("event")); }
    if run.head_branch != expected.branch { return Err(mismatch("branch")); }
    if run.head_sha != expected.head_sha { return Err(mismatch("head SHA")); }
    Ok(())
}

fn validate_artifact(
    artifact: &ArtifactRecord,
    observer: &RunIdentity,
    expected_name: &str,
) -> Result<(), IngestError> {
    if artifact.name != expected_name || !artifact_name_binds_run(&artifact.name, observer.run_id, observer.attempt) {
        return Err(IngestError::ArtifactMismatch("name/run/attempt binding"));
    }
    if artifact.expired { return Err(IngestError::ArtifactMismatch("expired")); }
    if artifact.size_in_bytes == 0 || artifact.size_in_bytes > MAX_OBSERVER_ZIP_BYTES as u64 {
        return Err(IngestError::ArtifactMismatch("archive size bound"));
    }
    let association = artifact.workflow_run.as_ref().ok_or(IngestError::ArtifactMismatch("run association"))?;
    if association.id != observer.run_id { return Err(IngestError::ArtifactMismatch("run association")); }
    if association.head_sha != observer.head_sha || association.head_branch != observer.branch {
        return Err(IngestError::ArtifactMismatch("source association"));
    }
    Ok(())
}

fn validate_observer_report(
    report: &ObserverReportV2,
    request: &IngestRequest,
    target: &RunRecord,
    target_jobs: &[JobRecord],
) -> Result<(), IngestError> {
    if report.schema_version != OBSERVER_REPORT_SCHEMA_VERSION {
        return Err(IngestError::ObserverMismatchData("schema version"));
    }
    if report.observer_run_id != request.observer.run_id
        || report.observer_run_attempt != request.observer.attempt
    {
        return Err(IngestError::ObserverMismatchData("observer run/attempt"));
    }
    if report.event != request.observer.event || report.branch != request.observer.branch
        || report.event_sha != request.observer.head_sha
    {
        return Err(IngestError::ObserverMismatchData("observer event identity"));
    }
    if report.product_execution != "NOT_RUN" {
        return Err(IngestError::ObserverMismatchData("product execution status"));
    }
    if report.jobs_scope != "run_attempt" || report.artifacts_scope != "whole_run" {
        return Err(IngestError::ObserverMismatchData("observation scope"));
    }
    if !matches!(report.observer_state.as_str(),
        "measured" | "pending_timeout" | "confirmed_missing_within_window" | "ambiguous" | "api_error")
    {
        return Err(IngestError::ObserverMismatchData("observer state"));
    }
    if !matches!(report.source_state.as_str(), "not_attempted" | "fetching" | "fetched" | "api_error") {
        return Err(IngestError::ObserverMismatchData("workflow source state"));
    }
    let target_identity = match (report.run_id, report.run_attempt) {
        (Some(run_id), Some(attempt)) => Some((run_id, attempt)),
        (None, None) => None,
        _ => return Err(IngestError::ObserverMismatchData("partial target run identity")),
    };
    if target_identity != Some((target.id, target.run_attempt))
        || report.target_run_state != target.status
        || report.run_source_sha.as_deref() != Some(target.head_sha.as_str())
        || report.conclusion != target.conclusion
    {
        return Err(IngestError::ObserverMismatchData("target run facts"));
    }
    if report.observer_state != "measured" {
        return Err(IngestError::ObserverMismatchData("resolved target must be measured"));
    }
    if report.jobs_count != Some(target_jobs.len()) {
        return Err(IngestError::ObserverMismatchData("job count"));
    }
    if report.source_state != "fetched"
        || report.workflow_source_blob_sha.as_deref().map_or(true, |value| !is_sha1(value))
        || report.workflow_source_bytes.map_or(true, |bytes| bytes == 0 || bytes > api::MAX_WORKFLOW_FILE_BYTES)
        || report.workflow_source_sha256.as_deref().map_or(true, |value| !is_sha256(value))
        || report.artifacts_count.is_none()
        || report.error.is_some()
    {
        return Err(IngestError::ObserverMismatchData("workflow source facts"));
    }
    Ok(())
}

fn artifact_name_binds_run(name: &str, run_id: u64, attempt: u32) -> bool {
    let numeric_tokens: Vec<&str> = name.split(|character: char| !character.is_ascii_digit())
        .filter(|token| !token.is_empty()).collect();
    let run = run_id.to_string();
    let attempt = attempt.to_string();
    let run_count = numeric_tokens.iter().filter(|token| **token == run.as_str()).count();
    let attempt_count = numeric_tokens.iter().filter(|token| **token == attempt.as_str()).count();
    if run == attempt { run_count >= 2 } else { run_count >= 1 && attempt_count >= 1 }
}

fn safe_workflow_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') { return false; }
    path.split('/').all(|segment| {
        !segment.is_empty() && segment != "." && segment != ".."
            && segment.chars().all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    })
}

fn is_sha1(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_ref_name(value: &str) -> bool {
    !value.is_empty() && !value.starts_with('/') && !value.ends_with('/')
        && !value.contains("..") && !value.contains("@{")
        && value.bytes().all(|byte| {
            !byte.is_ascii_control() && !matches!(byte, b' ' | b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\')
        })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
