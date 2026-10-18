use std::env;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::process::ExitCode;

use serde::Serialize;
use termrock_visibility_publisher::api::{
    ApiClient, ApiError, ReadOnlyTransport, Repository, RunRecord, RunSelector,
    TargetResolution,
};
use termrock_visibility_publisher::transport::ReqwestTransport;
use termrock_visibility_publisher::{
    ingest, CallerIdentity, IngestError, IngestRequest, IngestedObservation,
    RunIdentity,
};

const USAGE: &str = "Usage: observer-ingest [--help]\n\
Collect one trusted GitHub Actions observer report using read-only API requests.";
const RECEIPT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GithubShaBinding { RunHead }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TriggerPolicy {
    pub(crate) event: String,
    pub(crate) github_ref: String,
    pub(crate) workflow_ref: String,
    pub(crate) branch: String,
    pub(crate) github_sha_binding: GithubShaBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkflowPin {
    pub(crate) workflow_id: u64,
    pub(crate) path: String,
    pub(crate) content_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrustedPolicy {
    pub(crate) repository: String,
    pub(crate) caller: WorkflowPin,
    pub(crate) target: WorkflowPin,
    pub(crate) observer: WorkflowPin,
    pub(crate) triggers: Vec<TriggerPolicy>,
    pub(crate) artifact_name_prefix: String,
}

/// Environment values select a run; authenticated API records and a trusted
/// workflow policy establish its identity.
pub(crate) struct InvocationContext {
    pub(crate) repository: String,
    pub(crate) run_id: u64,
    pub(crate) attempt: u32,
    pub(crate) event: String,
    pub(crate) github_ref: String,
    pub(crate) github_sha: String,
    pub(crate) workflow_ref: String,
    pub(crate) workflow_sha: String,
    pub(crate) token: String,
}

impl InvocationContext {
    fn from_environment() -> Result<Self, CliError> {
        let context = Self {
            repository: required_environment("GITHUB_REPOSITORY")?,
            run_id: positive_u64_environment("GITHUB_RUN_ID")?,
            attempt: positive_u32_environment("GITHUB_RUN_ATTEMPT")?,
            event: required_environment("GITHUB_EVENT_NAME")?,
            github_ref: required_environment("GITHUB_REF")?,
            github_sha: required_environment("GITHUB_SHA")?,
            workflow_ref: required_environment("GITHUB_WORKFLOW_REF")?,
            workflow_sha: required_environment("GITHUB_WORKFLOW_SHA")?,
            token: required_environment("GITHUB_TOKEN")?,
        };
        if !is_sha1(&context.github_sha) || !is_sha1(&context.workflow_sha) {
            return Err(CliError("invalid commit SHA"));
        }
        if context.token.trim().is_empty() || context.token.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(CliError("invalid GitHub token"));
        }
        Ok(context)
    }
}

#[derive(Debug)]
pub(crate) struct CliError(&'static str);

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for CliError {}

impl From<ApiError> for CliError {
    fn from(_: ApiError) -> Self { Self("GitHub API read failed") }
}

impl From<IngestError> for CliError {
    fn from(_: IngestError) -> Self { Self("strict observer ingestion failed") }
}

#[derive(Serialize)]
pub(crate) struct Receipt {
    receipt_schema_version: u16,
    caller: RunSummary,
    target: RunSummary,
    observer: RunSummary,
    artifact_sha256: String,
    observer_report_schema_version: u16,
}

#[derive(Serialize)]
struct RunSummary {
    repository: String,
    run_id: u64,
    attempt: u32,
    workflow_id: u64,
    workflow_path: String,
}

impl Receipt {
    fn from_observation(observation: &IngestedObservation) -> Self {
        Self {
            receipt_schema_version: RECEIPT_SCHEMA_VERSION,
            caller: RunSummary::from_run(&observation.caller),
            target: RunSummary::from_run(&observation.target),
            observer: RunSummary::from_run(&observation.observer),
            artifact_sha256: observation.archive_sha256.clone(),
            observer_report_schema_version: observation.observer_report.schema_version,
        }
    }
}

impl RunSummary {
    fn from_run(run: &RunRecord) -> Self {
        Self {
            repository: run.repository.clone(),
            run_id: run.id,
            attempt: run.run_attempt,
            workflow_id: run.workflow_id,
            workflow_path: run.workflow_path.clone(),
        }
    }
}

fn main() -> ExitCode {
    match run_process() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("observer-ingest: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_process() -> Result<(), CliError> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    if !arguments.is_empty() { return Err(CliError("invalid arguments; use --help for usage")); }

    let context = InvocationContext::from_environment()?;
    let policy = production_policy().ok_or(CliError("trusted workflow policy is not pinned; collection is disabled"))?;
    let transport = ReqwestTransport::new().map_err(|_| CliError("HTTPS transport is unavailable"))?;
    write_receipt(&run_with_transport(context, &policy, transport)?)
}

/// Pending the exact generated workflow policy, the production entry point is
/// disabled. Environment and command-line values cannot provide this policy.
fn production_policy() -> Option<TrustedPolicy> { None }

pub(crate) fn run_with_transport<T: ReadOnlyTransport>(
    context: InvocationContext,
    policy: &TrustedPolicy,
    transport: T,
) -> Result<Receipt, CliError> {
    validate_policy_shape(policy)?;
    let trigger = validate_static_context(&context, policy)?;
    let repository = parse_policy_repository(&policy.repository)?;
    let client = ApiClient::new(transport, context.token.clone())?;

    let caller = client.get_run(&repository, context.run_id)?;
    validate_caller_record(&caller, &context, policy, trigger)?;
    validate_github_sha(&context, trigger, &caller)?;
    let caller_workflow = client.get_workflow_file(&repository, &policy.caller.path, &context.workflow_sha)?;
    if caller_workflow.path != policy.caller.path
        || caller_workflow.source_commit != context.workflow_sha
        || caller_workflow.sha256 != policy.caller.content_sha256
    {
        return Err(CliError("caller workflow source does not match its trusted pin"));
    }

    // Resolve target before observer so a mismatched target identity cannot
    // cause any observer lookup or produce a receipt.
    let target_selector = RunSelector {
        repository: repository.clone(), workflow_id: policy.target.workflow_id,
        workflow_path: policy.target.path.clone(), head_sha: caller.head_sha.clone(),
        branch: caller.head_branch.clone(), event: caller.event.clone(),
    };
    let target = match client.resolve_target(&target_selector)? {
        TargetResolution::Missing => return Err(CliError("no matching target workflow run was found")),
        TargetResolution::Ambiguous => return Err(CliError("target workflow run selection is ambiguous")),
        TargetResolution::Found(run) => run,
    };
    if !run_matches_selector(&target, &target_selector) {
        return Err(CliError("target workflow run does not match its selector"));
    }
    if target.id == caller.id { return Err(CliError("caller and target run IDs must be distinct")); }
    if target.status != "completed" { return Err(CliError("target workflow run is not complete")); }

    let observer_selector = RunSelector {
        repository: repository.clone(), workflow_id: policy.observer.workflow_id,
        workflow_path: policy.observer.path.clone(), head_sha: caller.head_sha.clone(),
        branch: caller.head_branch.clone(), event: caller.event.clone(),
    };
    let observer = match client.resolve_target(&observer_selector)? {
        TargetResolution::Missing => return Err(CliError("no matching observer workflow run was found")),
        TargetResolution::Ambiguous => return Err(CliError("observer workflow run selection is ambiguous")),
        TargetResolution::Found(run) => run,
    };
    if !run_matches_selector(&observer, &observer_selector) {
        return Err(CliError("observer workflow run does not match its selector"));
    }
    if observer.id == caller.id || observer.id == target.id {
        return Err(CliError("caller, target, and observer run IDs must be distinct"));
    }
    if observer.status != "completed" {
        return Err(CliError("observer workflow run is not complete"));
    }

    let request = IngestRequest {
        caller: CallerIdentity {
            run: run_identity_from_record(&caller, repository.clone()),
            workflow_ref: context.workflow_ref,
            workflow_sha: context.workflow_sha,
        },
        target: target_selector,
        observer: run_identity_from_record(&observer, repository),
        observer_artifact_name: format!("{}-{}-attempt-{}", policy.artifact_name_prefix, observer.id, observer.run_attempt),
    };
    let observation = ingest(&client, &request)?;
    if observation.caller_workflow_sha256 != policy.caller.content_sha256 {
        return Err(CliError("caller workflow source does not match its trusted pin"));
    }
    Ok(Receipt::from_observation(&observation))
}

fn validate_static_context<'a>(context: &InvocationContext, policy: &'a TrustedPolicy) -> Result<&'a TriggerPolicy, CliError> {
    if context.repository != policy.repository || context.run_id == 0 || context.attempt == 0
        || !is_sha1(&context.github_sha) || !is_sha1(&context.workflow_sha)
    {
        return Err(CliError("caller context does not match the trusted workflow policy"));
    }
    policy.triggers.iter().find(|trigger| {
        trigger.event == context.event && trigger.github_ref == context.github_ref
            && trigger.workflow_ref == context.workflow_ref
    }).ok_or(CliError("caller context does not match the trusted workflow policy"))
}

fn validate_policy_shape(policy: &TrustedPolicy) -> Result<(), CliError> {
    if policy.repository.split_once('/').is_none()
        || [policy.caller.workflow_id, policy.target.workflow_id, policy.observer.workflow_id].contains(&0)
        || policy.triggers.is_empty() || !safe_artifact_prefix(&policy.artifact_name_prefix)
        || !safe_workflow_path(&policy.caller.path) || !safe_workflow_path(&policy.target.path)
        || !safe_workflow_path(&policy.observer.path) || !is_sha256(&policy.caller.content_sha256)
    {
        return Err(CliError("trusted workflow policy is invalid"));
    }
    Ok(())
}

fn parse_policy_repository(value: &str) -> Result<Repository, CliError> {
    let Some((owner, name)) = value.split_once('/') else { return Err(CliError("trusted repository is invalid")); };
    if name.contains('/') { return Err(CliError("trusted repository is invalid")); }
    Repository::new(owner, name).map_err(CliError::from)
}

fn validate_caller_record(caller: &RunRecord, context: &InvocationContext, policy: &TrustedPolicy, trigger: &TriggerPolicy) -> Result<(), CliError> {
    if caller.id != context.run_id || caller.run_attempt != context.attempt
        || caller.workflow_id != policy.caller.workflow_id || caller.workflow_path != policy.caller.path
        || caller.repository != policy.repository || caller.event != trigger.event
        || caller.head_branch != trigger.branch || !is_sha1(&caller.head_sha)
    {
        return Err(CliError("caller run does not match the trusted workflow policy"));
    }
    Ok(())
}

fn validate_github_sha(context: &InvocationContext, trigger: &TriggerPolicy, caller: &RunRecord) -> Result<(), CliError> {
    match trigger.github_sha_binding {
        GithubShaBinding::RunHead if context.github_sha == caller.head_sha => Ok(()),
        GithubShaBinding::RunHead => Err(CliError("GITHUB_SHA does not match the caller run head")),
    }
}

fn run_matches_selector(run: &RunRecord, selector: &RunSelector) -> bool {
    run.workflow_id == selector.workflow_id && run.workflow_path == selector.workflow_path
        && run.repository == selector.repository.full_name() && run.head_sha == selector.head_sha
        && run.head_branch == selector.branch && run.event == selector.event
}

fn run_identity_from_record(run: &RunRecord, repository: Repository) -> RunIdentity {
    RunIdentity {
        repository, run_id: run.id, attempt: run.run_attempt, workflow_id: run.workflow_id,
        workflow_path: run.workflow_path.clone(), event: run.event.clone(),
        branch: run.head_branch.clone(), head_sha: run.head_sha.clone(),
    }
}

fn write_receipt(receipt: &Receipt) -> Result<(), CliError> {
    let mut output = io::stdout().lock();
    serde_json::to_writer(&mut output, receipt).map_err(|_| CliError("could not write the collection receipt"))?;
    output.write_all(b"\n").map_err(|_| CliError("could not write the collection receipt"))
}

fn required_environment(name: &'static str) -> Result<String, CliError> {
    env::var(name).map_err(|error| match error {
        env::VarError::NotPresent => CliError("required environment value is missing"),
        env::VarError::NotUnicode(_) => CliError("environment value is invalid"),
    })
}

fn positive_u64_environment(name: &'static str) -> Result<u64, CliError> {
    required_environment(name)?.parse::<u64>().ok().filter(|value| *value > 0)
        .ok_or(CliError("run ID is invalid"))
}

fn positive_u32_environment(name: &'static str) -> Result<u32, CliError> {
    required_environment(name)?.parse::<u32>().ok().filter(|value| *value > 0)
        .ok_or(CliError("run attempt is invalid"))
}

fn is_sha1(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_workflow_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains('\\')
        && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != ".."
            && segment.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)))
}

fn safe_artifact_prefix(prefix: &str) -> bool {
    !prefix.is_empty() && prefix.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}
