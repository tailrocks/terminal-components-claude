use std::collections::VecDeque;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use termrock_visibility_publisher::api::{
    API_ORIGIN, ApiClient, HttpRequest, HttpResponse, ReadOnlyTransport, Repository, RunSelector,
    TransportError,
};
use termrock_visibility_publisher::{ingest, CallerIdentity, IngestRequest, RunIdentity};

#[path = "../src/bin/observer-ingest.rs"]
#[allow(dead_code)]
mod cli;

const REPO: &str = "tailrocks/terminal-components-claude";
const BRANCH: &str = "termrock-implementation";
const TOKEN: &str = "test-token-never-a-real-credential";
const CALLER_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WORKFLOW_SHA: &str = "dddddddddddddddddddddddddddddddddddddddd";
const CALLER_PATH: &str = ".github/workflows/publish.yml";
const TARGET_PATH: &str = ".github/workflows/ci.yml";
const OBSERVER_PATH: &str = ".github/workflows/ci-observer.yml";
const WORKFLOW_BYTES: &[u8] = b"name: trusted publisher\n";
const CDN_URL: &str = "https://objects.githubusercontent.com/artifacts/900.zip?sig=test";

struct Expected {
    url: String,
    authorization: Option<String>,
    response: Result<HttpResponse, TransportError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Seen {
    url: String,
    authorization: Option<String>,
    max_bytes: usize,
}

#[derive(Clone, Default)]
struct FakeTransport {
    expected: Arc<Mutex<VecDeque<Expected>>>,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl FakeTransport {
    fn new(expected: Vec<Expected>) -> Self {
        Self { expected: Arc::new(Mutex::new(expected.into())), seen: Arc::default() }
    }

    fn requests(&self) -> Vec<Seen> { self.seen.lock().unwrap().clone() }

    fn assert_drained(&self) { assert!(self.expected.lock().unwrap().is_empty()); }
}

impl ReadOnlyTransport for FakeTransport {
    fn get(&self, request: HttpRequest, deadline: Instant, max_bytes: usize) -> Result<HttpResponse, TransportError> {
        assert!(Instant::now() < deadline);
        assert!(max_bytes > 0);
        let seen = Seen {
            url: request.url().to_owned(),
            authorization: request.authorization().map(str::to_owned),
            max_bytes,
        };
        let expected = self.expected.lock().unwrap().pop_front().expect("unexpected GET");
        assert_eq!(seen.url, expected.url);
        assert_eq!(seen.authorization, expected.authorization);
        self.seen.lock().unwrap().push(seen);
        expected.response
    }
}

fn context() -> cli::InvocationContext {
    cli::InvocationContext {
        repository: REPO.to_owned(), run_id: 100, attempt: 1, event: "push".to_owned(),
        github_ref: format!("refs/heads/{BRANCH}"), github_sha: CALLER_SHA.to_owned(),
        workflow_ref: format!("{REPO}/{CALLER_PATH}@refs/heads/{BRANCH}"),
        workflow_sha: WORKFLOW_SHA.to_owned(), token: TOKEN.to_owned(),
    }
}

fn policy() -> cli::TrustedPolicy {
    let pin = |workflow_id, path: &str, content_sha256| cli::WorkflowPin {
        workflow_id, path: path.to_owned(), content_sha256,
    };
    cli::TrustedPolicy {
        repository: REPO.to_owned(),
        caller: pin(50, CALLER_PATH, sha256(WORKFLOW_BYTES)),
        target: pin(7, TARGET_PATH, String::new()),
        observer: pin(90, OBSERVER_PATH, String::new()),
        triggers: vec![cli::TriggerPolicy {
            event: "push".to_owned(), github_ref: format!("refs/heads/{BRANCH}"),
            workflow_ref: format!("{REPO}/{CALLER_PATH}@refs/heads/{BRANCH}"),
            branch: BRANCH.to_owned(), github_sha_binding: cli::GithubShaBinding::RunHead,
        }],
        artifact_name_prefix: "ci-observer-report".to_owned(),
    }
}

fn run(id: u64, attempt: u32, workflow_id: u64, path: &str, sha: &str, conclusion: &str) -> Value {
    json!({
        "id": id, "run_number": id / 10, "run_attempt": attempt, "workflow_id": workflow_id,
        "path": format!("{path}@refs/heads/{BRANCH}"), "event": "push", "status": "completed",
        "conclusion": conclusion, "head_sha": sha, "head_branch": BRANCH,
        "repository": { "full_name": REPO }
    })
}

fn caller_run() -> Value { run(100, 1, 50, CALLER_PATH, CALLER_SHA, "success") }
fn target_run(sha: &str) -> Value { run(42, 2, 7, TARGET_PATH, sha, "success") }
fn observer_run(attempt: u32) -> Value { run(300, attempt, 90, OBSERVER_PATH, CALLER_SHA, "success") }

fn api_get(url: impl Into<String>, response: HttpResponse) -> Expected {
    Expected { url: url.into(), authorization: Some(TOKEN.to_owned()), response: Ok(response) }
}

fn json_get(url: impl Into<String>, value: Value) -> Expected {
    api_get(url, HttpResponse::new(200, serde_json::to_vec(&value).unwrap()))
}

fn workflow_contents() -> Value {
    json!({
        "path": CALLER_PATH, "type": "file", "encoding": "base64", "size": WORKFLOW_BYTES.len(),
        "sha": "0123456789012345678901234567890123456789",
        "content": base64::engine::general_purpose::STANDARD.encode(WORKFLOW_BYTES)
    })
}

fn report() -> Value {
    json!({
        "schema_version": 2, "observer_run_id": 300, "observer_run_attempt": 3,
        "observer_state": "measured", "target_run_state": "completed", "run_id": 42,
        "run_attempt": 2, "event": "push", "branch": BRANCH, "event_sha": CALLER_SHA,
        "run_source_sha": CALLER_SHA, "conclusion": "success", "jobs_count": 1,
        "jobs_scope": "run_attempt", "artifacts_count": 2, "artifacts_scope": "whole_run",
        "workflow_source_blob_sha": "0123456789012345678901234567890123456789",
        "workflow_source_bytes": 234,
        "workflow_source_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "source_state": "fetched", "product_execution": "NOT_RUN", "error": null
    })
}

fn expected_requests(target_sha: &str) -> Vec<Expected> {
    let archive = stored_zip(&serde_json::to_vec(&report()).unwrap());
    let artifact_name = "ci-observer-report-300-attempt-3";
    let artifact_listing = json!({
        "total_count": 1, "artifacts": [{
            "id": 900, "name": artifact_name, "size_in_bytes": archive.len(), "expired": false,
            "digest": format!("sha256:{}", sha256(&archive)),
            "workflow_run": { "id": 300, "head_branch": BRANCH, "head_sha": CALLER_SHA }
        }]
    });
    let target_url = format!("{API_ORIGIN}/repos/{REPO}/actions/workflows/7/runs?head_sha={CALLER_SHA}&branch={BRANCH}&event=push&per_page=100&page=1");
    let observer_url = format!("{API_ORIGIN}/repos/{REPO}/actions/workflows/90/runs?head_sha={CALLER_SHA}&branch={BRANCH}&event=push&per_page=100&page=1");
    let contents_url = format!("{API_ORIGIN}/repos/{REPO}/contents/{CALLER_PATH}?ref={WORKFLOW_SHA}");
    let runs = |item: Value| json!({ "total_count": 1, "workflow_runs": [item] });
    let mut requests = vec![
        json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/100"), caller_run()),
        json_get(contents_url, workflow_contents()),
        json_get(target_url.clone(), runs(target_run(target_sha))),
        json_get(observer_url, runs(observer_run(3))),
        json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/100"), caller_run()),
        json_get(format!("{API_ORIGIN}/repos/{REPO}/contents/{CALLER_PATH}?ref={WORKFLOW_SHA}"), workflow_contents()),
        json_get(target_url, runs(target_run(CALLER_SHA))),
        json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/300"), observer_run(3)),
        json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/42/attempts/2/jobs?per_page=100&page=1"), json!({
            "total_count": 1, "jobs": [{ "id": 800, "name": "test / linux", "status": "completed", "conclusion": "success" }]
        })),
        json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/300/artifacts?per_page=100&page=1"), artifact_listing),
        api_get(format!("{API_ORIGIN}/repos/{REPO}/actions/artifacts/900/zip"), HttpResponse::new(302, Vec::<u8>::new()).with_header("Location", CDN_URL)),
        Expected { url: CDN_URL.to_owned(), authorization: None, response: Ok(HttpResponse::new(200, archive)) },
    ];
    // The mismatch case returns a different SHA only on the first target lookup.
    if target_sha != CALLER_SHA { requests[2] = json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/workflows/7/runs?head_sha={CALLER_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"), runs(target_run(target_sha))); }
    requests
}

fn ingestion_request() -> IngestRequest {
    let context = context();
    let trusted = policy();
    let repository = Repository::new("tailrocks", "terminal-components-claude").unwrap();
    let caller = RunIdentity {
        repository: repository.clone(), run_id: context.run_id, attempt: context.attempt,
        workflow_id: trusted.caller.workflow_id, workflow_path: trusted.caller.path,
        event: context.event.clone(), branch: BRANCH.to_owned(), head_sha: context.github_sha.clone(),
    };
    let target = RunSelector {
        repository: repository.clone(), workflow_id: trusted.target.workflow_id,
        workflow_path: trusted.target.path, head_sha: CALLER_SHA.to_owned(),
        branch: BRANCH.to_owned(), event: "push".to_owned(),
    };
    let observer = RunIdentity {
        repository, run_id: 300, attempt: 3, workflow_id: trusted.observer.workflow_id,
        workflow_path: trusted.observer.path, event: "push".to_owned(),
        branch: BRANCH.to_owned(), head_sha: CALLER_SHA.to_owned(),
    };
    let observer_artifact_name = format!("{}-300-attempt-3", trusted.artifact_name_prefix);
    IngestRequest {
        caller: CallerIdentity {
            run: caller, workflow_ref: context.workflow_ref, workflow_sha: context.workflow_sha,
        },
        target, observer, observer_artifact_name,
    }
}

fn direct_ingest_diagnostic() -> String {
    let responses = expected_requests(CALLER_SHA).into_iter().skip(4).collect();
    let transport = FakeTransport::new(responses);
    let observed_transport = transport.clone();
    let client = match ApiClient::new(transport, TOKEN) {
        Ok(client) => client,
        Err(error) => return format!("collector diagnostic setup failed: {error:?}"),
    };
    let result = match ingest(&client, &ingestion_request()) {
        Ok(_) => "public ingest succeeded".to_owned(),
        Err(error) => format!("public ingest failed: {error:?}"),
    };
    let seen = observed_transport.requests();
    let last_endpoint = seen.last().map(|request| {
        request.url.split_once('?').map_or(request.url.as_str(), |(path, _)| path).to_owned()
    });
    format!("{result}; synthetic_gets={}; last_endpoint={last_endpoint:?}", seen.len())
}

fn stored_zip(data: &[u8]) -> Vec<u8> {
    let name = b"ci-observer-report.json";
    let crc = crc32fast::hash(data);
    let mut zip = Vec::new();
    let offset = 0_u32;
    put32(&mut zip, 0x0403_4b50); put16(&mut zip, 20); put16(&mut zip, 0); put16(&mut zip, 0);
    put16(&mut zip, 0); put16(&mut zip, 0); put32(&mut zip, crc); put32(&mut zip, data.len() as u32);
    put32(&mut zip, data.len() as u32); put16(&mut zip, name.len() as u16); put16(&mut zip, 0);
    zip.extend_from_slice(name); zip.extend_from_slice(data);
    let central_offset = zip.len() as u32;
    put32(&mut zip, 0x0201_4b50); put16(&mut zip, 20); put16(&mut zip, 20); put16(&mut zip, 0);
    put16(&mut zip, 0); put16(&mut zip, 0); put16(&mut zip, 0); put32(&mut zip, crc); put32(&mut zip, data.len() as u32);
    put32(&mut zip, data.len() as u32); put16(&mut zip, name.len() as u16); put16(&mut zip, 0);
    put16(&mut zip, 0); put16(&mut zip, 0); put16(&mut zip, 0); put32(&mut zip, 0); put32(&mut zip, offset);
    zip.extend_from_slice(name);
    let central_size = zip.len() as u32 - central_offset;
    put32(&mut zip, 0x0605_4b50); put16(&mut zip, 0); put16(&mut zip, 0); put16(&mut zip, 1);
    put16(&mut zip, 1); put32(&mut zip, central_size); put32(&mut zip, central_offset); put16(&mut zip, 0);
    zip
}

fn put16(bytes: &mut Vec<u8>, value: u16) { bytes.extend_from_slice(&value.to_le_bytes()); }
fn put32(bytes: &mut Vec<u8>, value: u32) { bytes.extend_from_slice(&value.to_le_bytes()); }
fn sha256(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect() }

#[test]
fn handler_returns_sanitized_receipt_with_distinct_run_attempts_and_observer_artifact_name() {
    let transport = FakeTransport::new(expected_requests(CALLER_SHA));
    let receipt = match cli::run_with_transport(context(), &policy(), transport.clone()) {
        Ok(receipt) => receipt,
        Err(error) => panic!("handler failed: {error}; {}", direct_ingest_diagnostic()),
    };
    transport.assert_drained();
    let receipt = serde_json::to_value(receipt).unwrap();
    assert_eq!(receipt["caller"]["run_id"], 100);
    assert_eq!(receipt["caller"]["attempt"], 1);
    assert_eq!(receipt["target"]["run_id"], 42);
    assert_eq!(receipt["target"]["attempt"], 2);
    assert_eq!(receipt["observer"]["run_id"], 300);
    assert_eq!(receipt["observer"]["attempt"], 3);
    assert_eq!(receipt["observer_report_schema_version"], 2);
    assert!(receipt.get("observer_report").is_none(), "receipt must not expose report contents");
    let requests = transport.requests();
    assert_eq!(requests.len(), 12);
    assert!(requests.iter().all(|request| request.max_bytes > 0));
    assert_eq!(requests[11].url, CDN_URL);
    assert_eq!(requests[11].authorization, None, "artifact CDN redirect must not receive the API token");
}

#[test]
fn workflow_ref_or_workflow_source_mismatch_stops_before_target_and_observer_lookup() {
    let mut wrong_ref = context();
    wrong_ref.workflow_ref.push_str("-untrusted");
    let untouched = FakeTransport::default();
    assert!(cli::run_with_transport(wrong_ref, &policy(), untouched.clone()).is_err());
    assert!(untouched.requests().is_empty());

    let mut responses = expected_requests(CALLER_SHA);
    let bad_workflow = b"name: changed workflow\n";
    responses[1] = json_get(
        format!("{API_ORIGIN}/repos/{REPO}/contents/{CALLER_PATH}?ref={WORKFLOW_SHA}"),
        json!({
            "path": CALLER_PATH, "type": "file", "encoding": "base64", "size": bad_workflow.len(),
            "sha": "0123456789012345678901234567890123456789",
            "content": base64::engine::general_purpose::STANDARD.encode(bad_workflow)
        }),
    );
    let transport = FakeTransport::new(responses);
    assert!(cli::run_with_transport(context(), &policy(), transport.clone()).is_err());
    assert_eq!(transport.requests().len(), 2);
}

#[test]
fn caller_api_identity_mismatches_stop_before_workflow_or_target_lookup() {
    for field in ["id", "run_attempt", "workflow_id", "path", "repository", "event", "head_branch", "head_sha"] {
        let mut responses = expected_requests(CALLER_SHA);
        let mut caller: Value = serde_json::from_slice(&responses[0].response.as_ref().unwrap().body).unwrap();
        match field {
            "id" => caller["id"] = json!(101),
            "run_attempt" => caller["run_attempt"] = json!(2),
            "workflow_id" => caller["workflow_id"] = json!(51),
            "path" => caller["path"] = json!(".github/workflows/other.yml@refs/heads/termrock-implementation"),
            "repository" => caller["repository"]["full_name"] = json!("tailrocks/untrusted"),
            "event" => caller["event"] = json!("workflow_dispatch"),
            "head_branch" => caller["head_branch"] = json!("other-branch"),
            "head_sha" => caller["head_sha"] = json!(OTHER_SHA),
            _ => unreachable!(),
        }
        responses[0] = json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/100"), caller);
        let transport = FakeTransport::new(responses);
        assert!(cli::run_with_transport(context(), &policy(), transport.clone()).is_err(), "{field}");
        assert_eq!(transport.requests().len(), 1, "{field}");
    }
}

#[test]
fn github_sha_pull_request_and_target_sha_mismatches_fail_before_observer_lookup() {
    let mut wrong_sha = context();
    wrong_sha.github_sha = OTHER_SHA.to_owned();
    let transport = FakeTransport::new(expected_requests(CALLER_SHA));
    assert!(cli::run_with_transport(wrong_sha, &policy(), transport.clone()).is_err());
    assert_eq!(transport.requests().len(), 1);

    let mut pull_request = context();
    pull_request.event = "pull_request".to_owned();
    pull_request.github_ref = "refs/pull/19/merge".to_owned();
    let transport = FakeTransport::default();
    assert!(cli::run_with_transport(pull_request, &policy(), transport.clone()).is_err());
    assert!(transport.requests().is_empty(), "unsupported PR mapping must fail closed");

    let transport = FakeTransport::new(expected_requests(OTHER_SHA));
    assert!(cli::run_with_transport(context(), &policy(), transport.clone()).is_err());
    let calls = transport.requests();
    assert_eq!(calls.len(), 3);
    assert!(!calls.iter().any(|request| request.url.contains("workflows/90/runs")));
}

#[test]
fn observer_api_attempt_mismatch_stops_before_jobs_artifacts_and_receipt() {
    let mut requests = expected_requests(CALLER_SHA);
    requests[7] = json_get(format!("{API_ORIGIN}/repos/{REPO}/actions/runs/300"), observer_run(4));
    let transport = FakeTransport::new(requests);
    assert!(cli::run_with_transport(context(), &policy(), transport.clone()).is_err());
    let calls = transport.requests();
    assert_eq!(calls.len(), 8);
    assert!(!calls.iter().any(|request| request.url.contains("/jobs") || request.url.contains("/artifacts")));
}

#[test]
fn incomplete_observer_run_stops_before_observer_record_or_artifact_lookup() {
    let mut requests = expected_requests(CALLER_SHA);
    let mut page: Value = serde_json::from_slice(&requests[3].response.as_ref().unwrap().body).unwrap();
    page["workflow_runs"][0]["status"] = json!("in_progress");
    requests[3] = json_get(
        format!("{API_ORIGIN}/repos/{REPO}/actions/workflows/90/runs?head_sha={CALLER_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"),
        page,
    );
    let transport = FakeTransport::new(requests);
    assert!(cli::run_with_transport(context(), &policy(), transport.clone()).is_err());
    let calls = transport.requests();
    assert_eq!(calls.len(), 4);
    assert!(!calls.iter().any(|request| request.url.contains("/actions/runs/300") || request.url.contains("/artifacts")));
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_observer-ingest"));
    command.env_clear().env("GITHUB_REPOSITORY", REPO).env("GITHUB_RUN_ID", "100")
        .env("GITHUB_RUN_ATTEMPT", "1").env("GITHUB_EVENT_NAME", "push")
        .env("GITHUB_REF", format!("refs/heads/{BRANCH}"))
        .env("GITHUB_SHA", CALLER_SHA).env("GITHUB_WORKFLOW_REF", format!("{REPO}/{CALLER_PATH}@refs/heads/{BRANCH}"))
        .env("GITHUB_WORKFLOW_SHA", WORKFLOW_SHA).env("GITHUB_TOKEN", TOKEN);
    command
}

fn output(command: &mut Command) -> Output { command.output().unwrap() }

#[test]
fn process_help_and_fail_closed_policy_are_observable_without_network_access() {
    let help = output(Command::new(env!("CARGO_BIN_EXE_observer-ingest")).env_clear().arg("--help"));
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("read-only API requests"));

    let missing = output(Command::new(env!("CARGO_BIN_EXE_observer-ingest")).env_clear());
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("required environment value is missing"));

    let configured = output(&mut command());
    assert!(!configured.status.success());
    assert!(String::from_utf8_lossy(&configured.stderr).contains("trusted workflow policy is not pinned"));
    assert!(configured.stdout.is_empty(), "fail-closed process emits no success receipt");
}
