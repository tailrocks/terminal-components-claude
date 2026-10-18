use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use termrock_visibility_publisher::api::{
    API_ORIGIN, ApiClient, ApiError, HttpRequest, HttpResponse, ReadOnlyTransport,
    Repository, RunSelector, TargetResolution, TransportError,
};
use termrock_visibility_publisher::observer::{ObserverError, decode_observer_zip};
use termrock_visibility_publisher::{
    CallerIdentity, IngestError, IngestRequest, MAX_OBSERVER_JSON_BYTES,
    MAX_OBSERVER_ZIP_BYTES, RunIdentity, ingest,
};

const OWNER: &str = "tailrocks";
const REPO: &str = "terminal-components-claude";
const FULL_REPO: &str = "tailrocks/terminal-components-claude";
const BRANCH: &str = "termrock-implementation";
const TOKEN: &str = "test-token-not-a-real-credential";
const CALLER_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TARGET_SHA: &str = CALLER_SHA;
const OBSERVER_SHA: &str = CALLER_SHA;
const WORKFLOW_SHA: &str = "dddddddddddddddddddddddddddddddddddddddd";
const CALLER_PATH: &str = ".github/workflows/publish.yml";
const TARGET_PATH: &str = ".github/workflows/ci.yml";
const OBSERVER_PATH: &str = ".github/workflows/ci-observer.yml";
const ARTIFACT_NAME: &str = "ci-observer-report-300-attempt-2";
const CDN_URL: &str = "https://objects.githubusercontent.com/artifacts/900.zip?token=signed";

#[derive(Debug)]
struct ExpectedRequest {
    url: String,
    authorization: Option<String>,
    result: Result<HttpResponse, TransportError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SeenRequest {
    url: String,
    authorization: Option<String>,
    max_bytes: usize,
}

#[derive(Clone, Default)]
struct FakeTransport {
    expected: Arc<Mutex<VecDeque<ExpectedRequest>>>,
    seen: Arc<Mutex<Vec<SeenRequest>>>,
}

impl FakeTransport {
    fn with_requests(requests: Vec<ExpectedRequest>) -> Self {
        Self {
            expected: Arc::new(Mutex::new(requests.into())),
            seen: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn assert_drained(&self) {
        assert!(self.expected.lock().unwrap().is_empty(), "fake response queue was not drained");
    }

    fn seen(&self) -> Vec<SeenRequest> {
        self.seen.lock().unwrap().clone()
    }
}

impl ReadOnlyTransport for FakeTransport {
    fn get(
        &self,
        request: HttpRequest,
        deadline: Instant,
        max_bytes: usize,
    ) -> Result<HttpResponse, TransportError> {
        assert!(Instant::now() < deadline, "client supplied an expired deadline");
        assert!(max_bytes > 0, "client must supply a positive response cap");
        let actual = SeenRequest {
            url: request.url().to_owned(),
            authorization: request.authorization().map(str::to_owned),
            max_bytes,
        };
        let expected = self.expected.lock().unwrap().pop_front().expect("unexpected GET");
        assert_eq!(actual.url, expected.url, "unexpected request URL");
        assert_eq!(actual.authorization, expected.authorization, "unexpected credential scope");
        self.seen.lock().unwrap().push(actual);
        expected.result
    }
}

fn api_request(url: impl Into<String>) -> ExpectedRequest {
    ExpectedRequest {
        url: url.into(),
        authorization: Some(TOKEN.to_owned()),
        result: Ok(HttpResponse::new(200, Vec::<u8>::new())),
    }
}

fn json_request(url: impl Into<String>, value: Value) -> ExpectedRequest {
    let mut request = api_request(url);
    request.result = Ok(HttpResponse::new(200, serde_json::to_vec(&value).unwrap()));
    request
}

fn client(transport: FakeTransport) -> ApiClient<FakeTransport> {
    ApiClient::new(transport, TOKEN).unwrap()
}

fn repository() -> Repository {
    Repository::new(OWNER, REPO).unwrap()
}

fn request() -> IngestRequest {
    let repository = repository();
    IngestRequest {
        caller: CallerIdentity {
            run: RunIdentity {
                repository: repository.clone(),
                run_id: 100,
                attempt: 1,
                workflow_id: 50,
                workflow_path: CALLER_PATH.to_owned(),
                event: "push".to_owned(),
                branch: BRANCH.to_owned(),
                head_sha: CALLER_SHA.to_owned(),
            },
            workflow_ref: format!("{FULL_REPO}/{CALLER_PATH}@refs/heads/{BRANCH}"),
            workflow_sha: WORKFLOW_SHA.to_owned(),
        },
        target: RunSelector {
            repository: repository.clone(),
            workflow_id: 7,
            workflow_path: TARGET_PATH.to_owned(),
            head_sha: TARGET_SHA.to_owned(),
            branch: BRANCH.to_owned(),
            event: "push".to_owned(),
        },
        observer: RunIdentity {
            repository,
            run_id: 300,
            attempt: 2,
            workflow_id: 90,
            workflow_path: OBSERVER_PATH.to_owned(),
            event: "push".to_owned(),
            branch: BRANCH.to_owned(),
            head_sha: OBSERVER_SHA.to_owned(),
        },
        observer_artifact_name: ARTIFACT_NAME.to_owned(),
    }
}

fn run_json(
    id: u64,
    run_number: u64,
    attempt: u32,
    workflow_id: u64,
    workflow_path: &str,
    head_sha: &str,
    status: &str,
    conclusion: Option<&str>,
) -> Value {
    let branch = BRANCH;
    json!({
        "id": id,
        "run_number": run_number,
        "run_attempt": attempt,
        "workflow_id": workflow_id,
        "path": format!("{workflow_path}@refs/heads/{branch}"),
        "event": "push",
        "status": status,
        "conclusion": conclusion,
        "head_sha": head_sha,
        "head_branch": branch,
        "repository": { "full_name": FULL_REPO }
    })
}

fn caller_run_json() -> Value {
    run_json(100, 12, 1, 50, CALLER_PATH, CALLER_SHA, "completed", Some("success"))
}

fn target_run_json(id: u64, run_number: u64, attempt: u32, status: &str) -> Value {
    run_json(id, run_number, attempt, 7, TARGET_PATH, TARGET_SHA, status,
        if status == "completed" { Some("success") } else { None })
}

fn observer_run_json() -> Value {
    run_json(300, 18, 2, 90, OBSERVER_PATH, OBSERVER_SHA, "completed", Some("success"))
}

fn workflow_contents_json(bytes: &[u8]) -> Value {
    json!({
        "path": CALLER_PATH,
        "type": "file",
        "encoding": "base64",
        "size": bytes.len(),
        "sha": "0123456789012345678901234567890123456789",
        "content": STANDARD.encode(bytes)
    })
}

fn observer_json() -> Value {
    json!({
        "schema_version": 2,
        "observer_run_id": 300,
        "observer_run_attempt": 2,
        "observer_state": "measured",
        "target_run_state": "completed",
        "run_id": 200,
        "run_attempt": 1,
        "event": "push",
        "branch": BRANCH,
        "event_sha": OBSERVER_SHA,
        "run_source_sha": TARGET_SHA,
        "conclusion": "success",
        "jobs_count": 1,
        "jobs_scope": "run_attempt",
        "artifacts_count": 2,
        "artifacts_scope": "whole_run",
        "workflow_source_blob_sha": "0123456789012345678901234567890123456789",
        "workflow_source_bytes": 234,
        "workflow_source_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "source_state": "fetched",
        "product_execution": "NOT_RUN",
        "error": null
    })
}

fn target_job_json() -> Value {
    json!({
        "id": 800,
        "name": "test / linux",
        "status": "completed",
        "conclusion": "success"
    })
}

fn artifacts_json(archive: &[u8], override_fields: Value) -> Value {
    let mut artifact = json!({
        "id": 900,
        "name": ARTIFACT_NAME,
        "size_in_bytes": archive.len(),
        "expired": false,
        "digest": format!("sha256:{}", sha256_hex(archive)),
        "workflow_run": {
            "id": 300,
            "head_branch": BRANCH,
            "head_sha": OBSERVER_SHA
        }
    });
    if let (Some(target), Some(source)) = (artifact.as_object_mut(), override_fields.as_object()) {
        for (key, value) in source { target.insert(key.clone(), value.clone()); }
    }
    json!({ "total_count": 1, "artifacts": [artifact] })
}

fn success_requests(target_runs: Vec<Value>, archive: &[u8], artifact_listing: Value) -> Vec<ExpectedRequest> {
    success_requests_with_jobs(target_runs, archive, artifact_listing, vec![target_job_json()])
}

fn success_requests_with_jobs(
    target_runs: Vec<Value>,
    archive: &[u8],
    artifact_listing: Value,
    target_jobs: Vec<Value>,
) -> Vec<ExpectedRequest> {
    let workflow_bytes = b"name: read-only publisher\n";
    let target_runs_url = format!(
        "{API_ORIGIN}/repos/{FULL_REPO}/actions/workflows/7/runs?head_sha={TARGET_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"
    );
    let mut requests = vec![
        json_request(format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/100"), caller_run_json()),
        json_request(
            format!("{API_ORIGIN}/repos/{FULL_REPO}/contents/{CALLER_PATH}?ref={WORKFLOW_SHA}"),
            workflow_contents_json(workflow_bytes),
        ),
        json_request(target_runs_url, json!({ "total_count": target_runs.len(), "workflow_runs": target_runs })),
    ];
    requests.push(json_request(format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/300"), observer_run_json()));
    requests.push(json_request(
        format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/200/attempts/1/jobs?per_page=100&page=1"),
        {
            let count = target_jobs.len();
            json!({ "total_count": count, "jobs": target_jobs })
        },
    ));
    requests.push(json_request(
        format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/300/artifacts?per_page=100&page=1"),
        artifact_listing,
    ));
    requests.push(ExpectedRequest {
        url: format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/artifacts/900/zip"),
        authorization: Some(TOKEN.to_owned()),
        result: Ok(HttpResponse::new(302, Vec::<u8>::new()).with_header("Location", CDN_URL)),
    });
    requests.push(ExpectedRequest {
        url: CDN_URL.to_owned(),
        authorization: None,
        result: Ok(HttpResponse::new(200, archive.to_vec())),
    });
    requests
}

fn api_only_success_requests(target_runs: Vec<Value>) -> Vec<ExpectedRequest> {
    let archive = stored_zip(&serde_json::to_vec(&observer_json()).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    success_requests(target_runs, &archive, listing)
}

fn stored_zip(json_bytes: &[u8]) -> Vec<u8> {
    stored_zip_members(&[("ci-observer-report.json", json_bytes)], 0, 0)
}

fn deflated_zip(json_bytes: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(json_bytes).unwrap();
    let compressed = encoder.finish().unwrap();
    zip_members(&[("ci-observer-report.json", json_bytes, &compressed, 8)], 0, 0)
}

fn stored_zip_members(members: &[(&str, &[u8])], flags: u16, unix_mode: u32) -> Vec<u8> {
    let entries: Vec<_> = members.iter().map(|(name, data)| (*name, *data, *data, 0)).collect();
    zip_members(&entries, flags, unix_mode)
}

fn zip_members(members: &[(&str, &[u8], &[u8], u16)], flags: u16, unix_mode: u32) -> Vec<u8> {
    let mut archive = Vec::new();
    let mut central_entries = Vec::new();
    for (name, data, compressed, method) in members {
        let local_offset = archive.len() as u32;
        let name_bytes = name.as_bytes();
        let crc = crc32fast::hash(data);
        put_u32(&mut archive, 0x0403_4b50);
        put_u16(&mut archive, 20);
        put_u16(&mut archive, flags);
        put_u16(&mut archive, *method);
        put_u16(&mut archive, 0);
        put_u16(&mut archive, 0);
        put_u32(&mut archive, crc);
        put_u32(&mut archive, compressed.len() as u32);
        put_u32(&mut archive, data.len() as u32);
        put_u16(&mut archive, name_bytes.len() as u16);
        put_u16(&mut archive, 0);
        archive.extend_from_slice(name_bytes);
        archive.extend_from_slice(compressed);

        let mut central = Vec::new();
        put_u32(&mut central, 0x0201_4b50);
        put_u16(&mut central, (3 << 8) | 20);
        put_u16(&mut central, 20);
        put_u16(&mut central, flags);
        put_u16(&mut central, *method);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, crc);
        put_u32(&mut central, compressed.len() as u32);
        put_u32(&mut central, data.len() as u32);
        put_u16(&mut central, name_bytes.len() as u16);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, unix_mode << 16);
        put_u32(&mut central, local_offset);
        central.extend_from_slice(name_bytes);
        central_entries.push(central);
    }
    let central_offset = archive.len() as u32;
    for entry in &central_entries { archive.extend_from_slice(entry); }
    let central_size = archive.len() as u32 - central_offset;
    put_u32(&mut archive, 0x0605_4b50);
    put_u16(&mut archive, 0);
    put_u16(&mut archive, 0);
    put_u16(&mut archive, members.len() as u16);
    put_u16(&mut archive, members.len() as u16);
    put_u32(&mut archive, central_size);
    put_u32(&mut archive, central_offset);
    put_u16(&mut archive, 0);
    archive
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) { bytes.extend_from_slice(&value.to_le_bytes()); }
fn put_u32(bytes: &mut Vec<u8>, value: u32) { bytes.extend_from_slice(&value.to_le_bytes()); }

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn ingest_keeps_caller_target_and_observer_identities_separate_and_reads_inert_zip() {
    let observation = observer_json();
    let archive = stored_zip(&serde_json::to_vec(&observation).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let transport = FakeTransport::with_requests(success_requests(vec![target_run_json(200, 20, 1, "completed")], &archive, listing));
    let ingested = ingest(&client(transport.clone()), &request()).unwrap();
    transport.assert_drained();

    assert_eq!(ingested.caller.id, 100);
    assert_eq!(ingested.target.id, 200);
    assert_eq!(ingested.target.run_attempt, 1);
    assert_eq!(ingested.observer.id, 300);
    assert_eq!(ingested.observer.run_attempt, 2);
    assert_eq!(ingested.artifact.id, 900);
    assert_eq!(ingested.observer_report.observer_run_id, 300);
    assert_eq!(ingested.observer_report.observer_run_attempt, 2);
    assert_eq!(ingested.caller_workflow_bytes, b"name: read-only publisher\n");
    assert_eq!(ingested.caller_workflow_sha256, sha256_hex(&ingested.caller_workflow_bytes));
    assert_eq!(ingested.archive_sha256, sha256_hex(&archive));
    assert_eq!(ingested.target_jobs.len(), 1);

    let calls = transport.seen();
    assert_eq!(calls.len(), 8);
    assert_eq!(calls[0].authorization.as_deref(), Some(TOKEN));
    assert_eq!(calls[6].authorization.as_deref(), Some(TOKEN));
    assert_eq!(calls[7].url, CDN_URL);
    assert_eq!(calls[7].authorization, None, "bearer token must not follow the artifact redirect");
    assert!(calls.iter().all(|call| call.max_bytes > 0));

    assert_failed_and_zero_job_observations();
}

fn assert_failed_and_zero_job_observations() {
    let mut failed_report = observer_json();
    failed_report["conclusion"] = json!("failure");
    let mut failed_run = target_run_json(200, 20, 1, "completed");
    failed_run["conclusion"] = json!("failure");
    let mut failed_job = target_job_json();
    failed_job["conclusion"] = json!("failure");
    let failed_archive = stored_zip(&serde_json::to_vec(&failed_report).unwrap());
    let failed_listing = artifacts_json(&failed_archive, Value::Null);
    let failed_transport = FakeTransport::with_requests(success_requests_with_jobs(
        vec![failed_run], &failed_archive, failed_listing, vec![failed_job],
    ));
    let failed = ingest(&client(failed_transport.clone()), &request()).unwrap();
    failed_transport.assert_drained();
    assert_eq!(failed.observer_report.conclusion.as_deref(), Some("failure"));
    assert_eq!(failed.observer_report.jobs_count, Some(1));
    assert_eq!(failed.observer_report.product_execution, "NOT_RUN");

    let mut empty_report = observer_json();
    empty_report["jobs_count"] = json!(0);
    let empty_archive = stored_zip(&serde_json::to_vec(&empty_report).unwrap());
    let empty_listing = artifacts_json(&empty_archive, Value::Null);
    let empty_transport = FakeTransport::with_requests(success_requests_with_jobs(
        vec![target_run_json(200, 20, 1, "completed")], &empty_archive, empty_listing, Vec::new(),
    ));
    let empty = ingest(&client(empty_transport.clone()), &request()).unwrap();
    empty_transport.assert_drained();
    assert_eq!(empty.observer_report.jobs_count, Some(0));
    assert_eq!(empty.observer_report.product_execution, "NOT_RUN");
}

#[test]
fn invalid_caller_observer_identity_rejected_before_any_request() {
    let mut input = request();
    input.observer.run_id = input.caller.run.run_id;
    let transport = FakeTransport::default();
    assert!(matches!(ingest(&client(transport.clone()), &input), Err(IngestError::DuplicateRunIds)));
    assert!(transport.seen().is_empty());

    let mut mixed_source = request();
    mixed_source.observer.head_sha = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_owned();
    let transport = FakeTransport::default();
    assert!(matches!(ingest(&client(transport.clone()), &mixed_source), Err(IngestError::InvalidInput(_))));
    assert!(transport.seen().is_empty());
}

#[test]
fn newest_in_progress_target_blocks_older_completed_run() {
    let older = target_run_json(200, 20, 1, "completed");
    let newer = target_run_json(201, 21, 1, "in_progress");
    let transport = FakeTransport::with_requests(api_only_success_requests(vec![older, newer]));
    let result = ingest(&client(transport.clone()), &request());
    assert!(matches!(result, Err(IngestError::TargetInProgress { run_id: 201, .. })));
    assert_eq!(transport.seen().len(), 3, "do not read observer data for an incomplete newest target");
}

#[test]
fn absent_target_is_a_measurement_gap_not_an_api_error() {
    let transport = FakeTransport::with_requests(api_only_success_requests(Vec::new()));
    assert!(matches!(ingest(&client(transport.clone()), &request()), Err(IngestError::TargetMissing)));
    assert_eq!(transport.seen().len(), 3);
}

#[test]
fn api_classifies_not_found_rate_limit_transport_and_deadline_separately() {
    let url = format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/100");
    let missing = FakeTransport::with_requests(vec![ExpectedRequest {
        url: url.clone(), authorization: Some(TOKEN.to_owned()),
        result: Ok(HttpResponse::new(404, b"{}".to_vec())),
    }]);
    assert_eq!(client(missing).get_run(&repository(), 100), Err(ApiError::NotFound));

    let limited = FakeTransport::with_requests(vec![ExpectedRequest {
        url: url.clone(), authorization: Some(TOKEN.to_owned()),
        result: Ok(HttpResponse::new(429, b"{}".to_vec()).with_header("Retry-After", "2")),
    }]);
    assert_eq!(client(limited).get_run(&repository(), 100), Err(ApiError::RateLimited { retry_after_seconds: Some(2) }));

    let unavailable = FakeTransport::with_requests(vec![ExpectedRequest {
        url: url.clone(), authorization: Some(TOKEN.to_owned()), result: Err(TransportError::Unavailable),
    }]);
    assert_eq!(client(unavailable).get_run(&repository(), 100), Err(ApiError::Transport(TransportError::Unavailable)));

    let deadline = FakeTransport::with_requests(vec![ExpectedRequest {
        url, authorization: Some(TOKEN.to_owned()), result: Err(TransportError::Deadline),
    }]);
    assert_eq!(client(deadline).get_run(&repository(), 100), Err(ApiError::Deadline));
}

#[test]
fn jobs_pagination_requires_all_pages_and_rejects_duplicate_items() {
    let url = format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/200/attempts/1/jobs");
    let first_jobs: Vec<Value> = (1..=100).map(|id| json!({
        "id": id, "name": format!("job-{id}"), "status": "completed", "conclusion": "success"
    })).collect();
    let second_jobs = vec![json!({ "id": 101, "name": "job-101", "status": "completed", "conclusion": "success" })];
    let transport = FakeTransport::with_requests(vec![
        json_request(format!("{url}?per_page=100&page=1"), json!({ "total_count": 101, "jobs": first_jobs })),
        json_request(format!("{url}?per_page=100&page=2"), json!({ "total_count": 101, "jobs": second_jobs })),
    ]);
    let jobs = client(transport.clone()).list_jobs(&repository(), 200, 1).unwrap();
    assert_eq!(jobs.len(), 101);
    assert_eq!(transport.seen().len(), 2);

    let duplicate = FakeTransport::with_requests(vec![json_request(
        format!("{url}?per_page=100&page=1"),
        json!({ "total_count": 2, "jobs": [
            { "id": 1, "name": "a", "status": "completed", "conclusion": "success" },
            { "id": 1, "name": "b", "status": "completed", "conclusion": "failure" }
        ] }),
    )]);
    assert_eq!(client(duplicate).list_jobs(&repository(), 200, 1), Err(ApiError::DuplicatePageItem));
}

#[test]
fn workflow_runs_and_artifacts_pagination_collects_every_page() {
    let runs_url = format!(
        "{API_ORIGIN}/repos/{FULL_REPO}/actions/workflows/7/runs?head_sha={TARGET_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"
    );
    let first_runs: Vec<Value> = (1..=100).map(|id| {
        run_json(id, id, 1, 8, TARGET_PATH, TARGET_SHA, "completed", Some("success"))
    }).collect();
    let second_runs = vec![target_run_json(200, 200, 1, "completed")];
    let runs_transport = FakeTransport::with_requests(vec![
        json_request(runs_url.clone(), json!({ "total_count": 101, "workflow_runs": first_runs })),
        json_request(runs_url.replace("&page=1", "&page=2"), json!({ "total_count": 101, "workflow_runs": second_runs })),
    ]);
    let resolution = client(runs_transport.clone()).resolve_target(&request().target).unwrap();
    assert!(matches!(resolution, TargetResolution::Found(run) if run.id == 200));
    assert_eq!(runs_transport.seen().len(), 2);

    let artifacts_url = format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/300/artifacts?per_page=100&page=1");
    let artifact = |id| json!({
        "id": id, "name": format!("artifact-{id}"), "size_in_bytes": 10,
        "expired": false, "digest": null, "workflow_run": null
    });
    let first_artifacts: Vec<Value> = (1..=100).map(artifact).collect();
    let second_artifacts = vec![artifact(101)];
    let artifact_transport = FakeTransport::with_requests(vec![
        json_request(artifacts_url.clone(), json!({ "total_count": 101, "artifacts": first_artifacts })),
        json_request(artifacts_url.replace("&page=1", "&page=2"), json!({ "total_count": 101, "artifacts": second_artifacts })),
    ]);
    assert_eq!(client(artifact_transport.clone()).list_artifacts(&repository(), 300).unwrap().len(), 101);
    assert_eq!(artifact_transport.seen().len(), 2);
}

#[test]
fn pagination_cap_and_changed_total_fail_closed() {
    let url = format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/runs/200/attempts/1/jobs?per_page=100&page=1");
    let capped = FakeTransport::with_requests(vec![json_request(url.clone(), json!({ "total_count": 2001, "jobs": [] }))]);
    assert_eq!(client(capped).list_jobs(&repository(), 200, 1), Err(ApiError::PaginationLimit));

    let first_jobs: Vec<Value> = (1..=100).map(|id| json!({
        "id": id, "name": format!("job-{id}"), "status": "completed", "conclusion": "success"
    })).collect();
    let changed = FakeTransport::with_requests(vec![
        json_request(url.clone(), json!({ "total_count": 101, "jobs": first_jobs })),
        json_request(url.replace("&page=1", "&page=2"), json!({ "total_count": 102, "jobs": [] })),
    ]);
    assert_eq!(client(changed).list_jobs(&repository(), 200, 1), Err(ApiError::IncompletePagination));
}

#[test]
fn artifact_digest_and_workflow_source_are_bound_to_downloaded_bytes_and_exact_sha() {
    let archive = stored_zip(&serde_json::to_vec(&observer_json()).unwrap());
    let mut listing = artifacts_json(&archive, Value::Null);
    listing["artifacts"][0]["digest"] = json!(format!("sha256:{}", "0".repeat(64)));
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport.clone()), &request()), Err(IngestError::ArtifactDigestMismatch)));
    let calls = transport.seen();
    assert!(calls.iter().any(|call| call.url.ends_with(&format!("?ref={WORKFLOW_SHA}"))));
}

#[test]
fn artifact_run_association_is_checked_before_download() {
    let archive = stored_zip(&serde_json::to_vec(&observer_json()).unwrap());
    let listing = artifacts_json(&archive, json!({ "workflow_run": {
        "id": 301, "head_branch": BRANCH, "head_sha": OBSERVER_SHA
    }}));
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport.clone()), &request()), Err(IngestError::ArtifactMismatch("run association"))));
    assert_eq!(transport.seen().len(), 6, "mismatched artifact association must stop before download");
}

#[test]
fn artifact_missing_ambiguous_expired_and_oversize_states_are_distinct() {
    let archive = stored_zip(&serde_json::to_vec(&observer_json()).unwrap());
    let target_runs = vec![target_run_json(200, 20, 1, "completed")];

    let missing = FakeTransport::with_requests(success_requests(
        target_runs.clone(), &archive, json!({ "total_count": 0, "artifacts": [] }),
    ));
    assert!(matches!(ingest(&client(missing.clone()), &request()), Err(IngestError::ArtifactMissing)));
    assert_eq!(missing.seen().len(), 6);

    let one = artifacts_json(&archive, Value::Null)["artifacts"][0].clone();
    let mut duplicate = one.clone();
    duplicate["id"] = json!(901);
    let ambiguous = FakeTransport::with_requests(success_requests(
        target_runs.clone(), &archive, json!({ "total_count": 2, "artifacts": [one.clone(), duplicate] }),
    ));
    assert!(matches!(ingest(&client(ambiguous.clone()), &request()), Err(IngestError::ArtifactAmbiguous)));
    assert_eq!(ambiguous.seen().len(), 6);

    for (override_fields, expected) in [
        (json!({ "expired": true }), "expired"),
        (json!({ "size_in_bytes": (MAX_OBSERVER_ZIP_BYTES as u64) + 1 }), "archive size bound"),
    ] {
        let listing = artifacts_json(&archive, override_fields);
        let transport = FakeTransport::with_requests(success_requests(target_runs.clone(), &archive, listing));
        assert!(matches!(
            ingest(&client(transport.clone()), &request()),
            Err(IngestError::ArtifactMismatch(field)) if field == expected
        ));
        assert_eq!(transport.seen().len(), 6);
    }
}

#[test]
fn unsafe_redirect_never_receives_credentials_or_archive_processing() {
    let archive = stored_zip(&serde_json::to_vec(&observer_json()).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let mut requests = success_requests(vec![target_run_json(200, 20, 1, "completed")], &archive, listing);
    requests[6].result = Ok(HttpResponse::new(302, Vec::<u8>::new()).with_header("Location", "http://objects.example/archive.zip"));
    requests.pop();
    let transport = FakeTransport::with_requests(requests);
    assert!(matches!(ingest(&client(transport.clone()), &request()), Err(IngestError::Api(ApiError::UnsafeRedirect))));
    assert_eq!(transport.seen().len(), 7);

    let download_url = format!("{API_ORIGIN}/repos/{FULL_REPO}/actions/artifacts/900/zip");
    let oversized_redirect = FakeTransport::with_requests(vec![ExpectedRequest {
        url: download_url,
        authorization: Some(TOKEN.to_owned()),
        result: Ok(HttpResponse::new(302, b"oversized".to_vec()).with_header("Location", CDN_URL)),
    }]);
    assert_eq!(client(oversized_redirect.clone()).download_artifact(&repository(), 900, 4), Err(ApiError::BodyLimit));
    assert_eq!(oversized_redirect.seen().len(), 1, "oversized redirect body must stop before following Location");
}

#[test]
fn observer_zip_rejects_oversize_traversal_extra_member_encryption_symlink_and_crc_corruption() {
    let valid_json = br#"{"schema_version":2}"#;
    let valid = stored_zip(valid_json);
    assert_eq!(decode_observer_zip(&valid, valid.len(), MAX_OBSERVER_JSON_BYTES).unwrap(), valid_json);
    let compressed = deflated_zip(valid_json);
    assert_eq!(decode_observer_zip(&compressed, compressed.len(), MAX_OBSERVER_JSON_BYTES).unwrap(), valid_json);
    assert_eq!(decode_observer_zip(&valid, valid.len() - 1, MAX_OBSERVER_JSON_BYTES), Err(ObserverError::ArchiveTooLarge));
    assert_eq!(decode_observer_zip(&valid, valid.len(), 1), Err(ObserverError::MemberTooLarge));

    let traversal = stored_zip_members(&[("../ci-observer-report.json", valid_json)], 0, 0);
    assert_eq!(decode_observer_zip(&traversal, traversal.len(), MAX_OBSERVER_JSON_BYTES), Err(ObserverError::UnsafeMemberPath));

    let extra = stored_zip_members(&[("ci-observer-report.json", valid_json), ("ci-observer-report.json", b"x")], 0, 0);
    assert_eq!(decode_observer_zip(&extra, extra.len(), MAX_OBSERVER_JSON_BYTES), Err(ObserverError::MultipleMembers));

    let encrypted = stored_zip_members(&[("ci-observer-report.json", valid_json)], 1, 0);
    assert_eq!(decode_observer_zip(&encrypted, encrypted.len(), MAX_OBSERVER_JSON_BYTES), Err(ObserverError::EncryptedMember));

    let symlink = stored_zip_members(&[("ci-observer-report.json", valid_json)], 0, 0o120777);
    assert_eq!(decode_observer_zip(&symlink, symlink.len(), MAX_OBSERVER_JSON_BYTES), Err(ObserverError::SymlinkMember));

    let mut corrupt = valid.clone();
    let data_offset = 30 + b"ci-observer-report.json".len();
    corrupt[data_offset] ^= 1;
    assert_eq!(decode_observer_zip(&corrupt, corrupt.len(), MAX_OBSERVER_JSON_BYTES), Err(ObserverError::CrcMismatch));
}

#[test]
fn observer_schema_rejects_unknown_fields_and_wrong_target_attempt() {
    let mut observation = observer_json();
    observation["unexpected"] = json!("not accepted");
    let archive = stored_zip(&serde_json::to_vec(&observation).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport), &request()), Err(IngestError::Observer(ObserverError::InvalidJson))));

    let v1_report = json!({
        "observer_state": "measured", "target_run_state": "completed", "run_id": 200,
        "run_attempt": 1, "event": "push", "branch": BRANCH, "event_sha": TARGET_SHA,
        "run_source_sha": TARGET_SHA, "conclusion": "success", "jobs_count": 1,
        "jobs_scope": "run_attempt", "artifacts_count": 2, "artifacts_scope": "whole_run",
        "workflow_source_blob_sha": "0123456789012345678901234567890123456789",
        "workflow_source_bytes": 234,
        "workflow_source_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "source_state": "fetched", "product_execution": "NOT_RUN"
    });
    let archive = stored_zip(&serde_json::to_vec(&v1_report).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport), &request()), Err(IngestError::Observer(ObserverError::InvalidJson))));

    let mut wrong_version = observer_json();
    wrong_version["schema_version"] = json!(1);
    let archive = stored_zip(&serde_json::to_vec(&wrong_version).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport), &request()), Err(IngestError::ObserverMismatchData("schema version"))));

    let mut observation = observer_json();
    observation["run_attempt"] = json!(2);
    let archive = stored_zip(&serde_json::to_vec(&observation).unwrap());
    let listing = artifacts_json(&archive, Value::Null);
    let transport = FakeTransport::with_requests(success_requests(
        vec![target_run_json(200, 20, 1, "completed")], &archive, listing,
    ));
    assert!(matches!(ingest(&client(transport), &request()), Err(IngestError::ObserverMismatchData("target run facts"))));
}

#[test]
fn invalid_artifact_name_run_attempt_binding_is_rejected_before_network() {
    let mut input = request();
    input.observer_artifact_name = "ci-observer-report".to_owned();
    let transport = FakeTransport::default();
    assert!(matches!(ingest(&client(transport.clone()), &input), Err(IngestError::InvalidInput(_))));
    assert!(transport.seen().is_empty());

    input.observer_artifact_name = "observer-300-attempt-300".to_owned();
    let transport = FakeTransport::default();
    assert!(matches!(ingest(&client(transport.clone()), &input), Err(IngestError::InvalidInput(_))));
    assert!(transport.seen().is_empty());
}

#[test]
fn target_resolution_selects_highest_attempt_and_rejects_tied_newest_attempts() {
    let selector = request().target;
    let first_attempt = target_run_json(200, 20, 1, "completed");
    let second_attempt = target_run_json(201, 20, 2, "completed");
    let url = format!(
        "{API_ORIGIN}/repos/{FULL_REPO}/actions/workflows/7/runs?head_sha={TARGET_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"
    );
    let transport = FakeTransport::with_requests(vec![json_request(url, json!({
        "total_count": 2, "workflow_runs": [first_attempt, second_attempt]
    }))]);
    let result = client(transport).resolve_target(&selector).unwrap();
    assert!(matches!(result, TargetResolution::Found(run) if run.id == 201 && run.run_attempt == 2));

    let tied = FakeTransport::with_requests(vec![json_request(
        format!(
            "{API_ORIGIN}/repos/{FULL_REPO}/actions/workflows/7/runs?head_sha={TARGET_SHA}&branch={BRANCH}&event=push&per_page=100&page=1"
        ),
        json!({ "total_count": 2, "workflow_runs": [
            target_run_json(202, 21, 2, "completed"),
            target_run_json(203, 21, 2, "completed")
        ] }),
    )]);
    assert_eq!(client(tied).resolve_target(&selector), Ok(TargetResolution::Ambiguous));
}
