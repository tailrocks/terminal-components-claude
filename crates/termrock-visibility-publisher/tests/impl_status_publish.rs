use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rcgen::{
    BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const FULL_REPO: &str = "tailrocks/terminal-components-claude";
const BRANCH: &str = "main";
const CALLER_PATH: &str = ".github/workflows/status-publisher.yml";
const TARGET_PATH: &str = ".github/workflows/ci.yml";
const OBSERVER_PATH: &str = ".github/workflows/ci-observer.yml";
const API_HOST: &str = "api.github.com";
const CDN_HOST: &str = "objects.githubusercontent.com";
const TOKEN: &str = "test-token-never-a-real-credential";
const HEAD_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WORKFLOW_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PARENT_SHA: &str = HEAD_SHA;
const TREE_SHA: &str = "cccccccccccccccccccccccccccccccccccccccc";
const NEW_COMMIT_SHA: &str = "dddddddddddddddddddddddddddddddddddddddd";
const STATUS_TEXT: &str = "# Visibility status\n\nProduct: FAIL\nObserver: measured\n";
const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;
const MAX_COMMAND_LOG_BYTES: usize = 64 * 1024;
static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct ResponsePlan {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl ResponsePlan {
    fn json(status: u16, value: Value) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: serde_json::to_vec(&value).expect("fixture JSON serializes"),
        }
    }

    fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_owned(), content_type.to_owned())],
            body,
        }
    }
}

#[derive(Clone, Debug)]
struct CapturedRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

struct TlsMaterial {
    root_pem: String,
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
}

fn tls_material(server_names: &[&str]) -> TlsMaterial {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let mut root_params = CertificateParams::new(vec!["termrock-test-ca.invalid".to_owned()])
        .expect("static CA name is valid");
    root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let root_key = KeyPair::generate().expect("Ring generates a test CA key");
    let root_cert = root_params
        .self_signed(&root_key)
        .expect("test CA can be self-signed");
    let issuer = Issuer::new(root_params, root_key);

    let mut leaf_params =
        CertificateParams::new(server_names.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>())
            .expect("static server names are valid");
    leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let leaf_key = KeyPair::generate().expect("Ring generates a test server key");
    let leaf_cert = leaf_params
        .signed_by(&leaf_key, &issuer)
        .expect("test server certificate can be signed");

    let root_der = root_cert.der().as_ref();
    TlsMaterial {
        root_pem: pem_certificate(root_der),
        chain: vec![leaf_cert.der().clone()],
        key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
    }
}

fn pem_certificate(der: &[u8]) -> String {
    let encoded = STANDARD.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for line in encoded.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

struct TestServer {
    address: SocketAddr,
    root_pem: String,
    stopped: Arc<AtomicBool>,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
    worker: Option<JoinHandle<()>>,
}

impl TestServer {
    fn start(responses: Vec<ResponsePlan>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback bind succeeds");
        listener
            .set_nonblocking(true)
            .expect("listener is nonblocking");
        let address = listener.local_addr().expect("listener has a local address");
        let material = tls_material(&[API_HOST, CDN_HOST]);
        let config = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(material.chain, material.key)
                .expect("test server certificate is valid"),
        );
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = Arc::clone(&stopped);
        let captured = Arc::new(Mutex::new(Vec::new()));
        let worker_captured = Arc::clone(&captured);
        let worker = thread::Builder::new()
            .name("visibility-status-writer-tls-test".to_owned())
            .spawn(move || {
                serve(listener, config, responses, worker_stopped, worker_captured);
            })
            .expect("test server thread starts");
        Self {
            address,
            root_pem: material.root_pem,
            stopped,
            captured,
            worker: Some(worker),
        }
    }

    fn captured(&self) -> Vec<CapturedRequest> {
        self.captured
            .lock()
            .expect("request log is available")
            .clone()
    }

    fn finish(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            assert!(worker.join().is_ok(), "fake TLS server stopped cleanly");
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.finish();
    }
}

fn serve(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    responses: Vec<ResponsePlan>,
    stopped: Arc<AtomicBool>,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
) {
    for response in responses {
        let accept_deadline = Instant::now() + Duration::from_secs(15);
        let (stream, _) = loop {
            if stopped.load(Ordering::Acquire) || Instant::now() >= accept_deadline {
                return;
            }
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2));
                }
                Err(_) => return,
            }
        };
        let _ = handle_connection(stream, Arc::clone(&config), response, &captured);
    }
}

fn handle_connection(
    stream: TcpStream,
    config: Arc<ServerConfig>,
    response: ResponsePlan,
    captured: &Mutex<Vec<CapturedRequest>>,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let connection = ServerConnection::new(config)
        .map_err(|_| std::io::Error::other("test TLS server configuration failed"))?;
    let mut tls = StreamOwned::new(connection, stream);
    let request = read_request(&mut tls)?;
    captured.lock().unwrap().push(request);
    write_response(&mut tls, response)
}

fn read_request(stream: &mut impl Read) -> std::io::Result<CapturedRequest> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    let header_end;
    loop {
        if bytes.len() >= MAX_HEADER_BYTES {
            return Err(std::io::Error::other("test request headers exceeded bound"));
        }
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request closed",
            ));
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > MAX_HEADER_BYTES {
            return Err(std::io::Error::other("test request headers exceeded bound"));
        }
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = index + 4;
            break;
        }
    }
    let header_text = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| std::io::Error::other("request headers are not UTF-8"))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or_default().to_owned();
    if method.is_empty() || path.is_empty() {
        return Err(std::io::Error::other("request line is malformed"));
    }
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let body_len = headers
        .get("content-length")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| std::io::Error::other("content length is malformed"))?
        .unwrap_or(0);
    if body_len > MAX_REQUEST_BODY_BYTES {
        return Err(std::io::Error::other("request body exceeded bound"));
    }
    while bytes.len() < header_end + body_len {
        let remaining = (header_end + body_len - bytes.len()).min(buffer.len());
        let count = stream.read(&mut buffer[..remaining])?;
        if count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request body closed",
            ));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok(CapturedRequest {
        method,
        path,
        headers,
        body: bytes[header_end..header_end + body_len].to_vec(),
    })
}

fn write_response(stream: &mut impl Write, response: ResponsePlan) -> std::io::Result<()> {
    let reason = match response.status {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "Test Response",
    };
    write!(stream, "HTTP/1.1 {} {}\r\n", response.status, reason)?;
    for (name, value) in response.headers {
        write!(stream, "{name}: {value}\r\n")?;
    }
    write!(stream, "Content-Length: {}\r\n", response.body.len())?;
    write!(stream, "Connection: close\r\n\r\n")?;
    stream.write_all(&response.body)?;
    stream.flush()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "termrock-status-publish-test-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("unique test temp directory is created");
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("test fixture file is written");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct CommandResult {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_command(server: &TestServer, input: &Value, policy: &Value) -> CommandResult {
    let temp = TempDir::new();
    let root_pem = temp.write("test-ca.pem", server.root_pem.as_bytes());
    let policy_path = temp.write(
        "policy.json",
        &serde_json::to_vec(policy).expect("policy serializes"),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_status-publish"))
        .env_clear()
        .env("TERMROCK_STATUS_PUBLISH_TEST_MODE", "1")
        .env("TERMROCK_STATUS_PUBLISH_TEST_CA_PEM", root_pem)
        .env(
            "TERMROCK_STATUS_PUBLISH_TEST_PORT",
            server.address.port().to_string(),
        )
        .env("TERMROCK_STATUS_PUBLISH_TEST_POLICY_JSON", policy_path)
        .env("GITHUB_TOKEN", TOKEN)
        .env("GITHUB_REPOSITORY", FULL_REPO)
        .env("GITHUB_EVENT_NAME", "workflow_run")
        .env("GITHUB_REF", format!("refs/heads/{BRANCH}"))
        .env("GITHUB_REF_NAME", BRANCH)
        .env("GITHUB_SHA", HEAD_SHA)
        .env("GITHUB_RUN_ID", "100")
        .env("GITHUB_RUN_ATTEMPT", "1")
        .env(
            "GITHUB_WORKFLOW_REF",
            format!("{FULL_REPO}/{CALLER_PATH}@refs/heads/{BRANCH}"),
        )
        .env("GITHUB_WORKFLOW_SHA", WORKFLOW_SHA)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("status-publish process starts");
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || read_log_bounded(stdout));
    let stderr_reader = thread::spawn(move || read_log_bounded(stderr));
    let request = serde_json::to_vec(input).expect("request JSON serializes");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(&request)
        .expect("request JSON is sent");
    let status = wait_child(&mut child, Duration::from_secs(20));
    let stdout = stdout_reader.join().expect("stdout reader joins");
    let stderr = stderr_reader.join().expect("stderr reader joins");
    CommandResult {
        status,
        stdout,
        stderr,
    }
}

fn read_log_bounded(mut stream: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    stream
        .take((MAX_COMMAND_LOG_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .expect("child output is readable");
    bytes
}

fn wait_child(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().expect("child process status is readable") {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("status-publish exceeded its test deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn policy() -> Value {
    json!({
        "repository": FULL_REPO,
        "branch": BRANCH,
        "publisher_event": "workflow_run",
        "publisher_workflow_id": 50,
        "publisher_workflow_path": CALLER_PATH,
        "publisher_workflow_sha256": sha256_hex(b"trusted publisher workflow"),
        "target_workflow_id": 7,
        "target_workflow_path": TARGET_PATH,
        "target_event": "push",
        "observer_workflow_id": 90,
        "observer_workflow_path": OBSERVER_PATH,
        "observer_event": "push"
    })
}

fn input() -> Value {
    json!({
        "schema_version": 1,
        "repository": FULL_REPO,
        "branch": BRANCH,
        "expected_parent_sha": PARENT_SHA,
        "caller": {
            "run_id": 100,
            "attempt": 1,
            "workflow_id": 50,
            "workflow_path": CALLER_PATH,
            "event": "workflow_run",
            "branch": BRANCH,
            "head_sha": HEAD_SHA,
            "workflow_sha": WORKFLOW_SHA
        },
        "target": {
            "workflow_id": 7,
            "workflow_path": TARGET_PATH,
            "event": "push",
            "branch": BRANCH,
            "head_sha": HEAD_SHA
        },
        "observer": {
            "run_id": 102,
            "attempt": 1,
            "workflow_id": 90,
            "workflow_path": OBSERVER_PATH,
            "event": "push",
            "branch": BRANCH,
            "head_sha": HEAD_SHA
        },
        "observer_artifact_name": "ci-observer-report-102-attempt-1",
        "status": {
            "markdown": STATUS_TEXT,
            "sha256": sha256_hex(STATUS_TEXT.as_bytes()),
            "source_commit": HEAD_SHA,
            "source_facts_sha256": "1".repeat(64),
            "tasks_sha256": "2".repeat(64),
            "generator_sha256": "3".repeat(64)
        }
    })
}

fn publisher_workflow_bytes() -> &'static [u8] {
    b"trusted publisher workflow"
}

fn run_json(
    id: u64,
    run_number: u64,
    attempt: u32,
    workflow_id: u64,
    path: &str,
    event: &str,
    head_sha: &str,
    conclusion: &str,
) -> Value {
    json!({
        "id": id,
        "run_number": run_number,
        "run_attempt": attempt,
        "workflow_id": workflow_id,
        "path": format!("{path}@refs/heads/{BRANCH}"),
        "event": event,
        "status": "completed",
        "conclusion": conclusion,
        "head_sha": head_sha,
        "head_branch": BRANCH,
        "repository": { "full_name": FULL_REPO }
    })
}

fn observer_report_for_target(target_run_id: u64, target_run_attempt: u32) -> Value {
    json!({
        "schema_version": 2,
        "observer_run_id": 102,
        "observer_run_attempt": 1,
        "observer_state": "measured",
        "target_run_state": "completed",
        "run_id": target_run_id,
        "run_attempt": target_run_attempt,
        "event": "push",
        "branch": BRANCH,
        "event_sha": HEAD_SHA,
        "run_source_sha": HEAD_SHA,
        "conclusion": "failure",
        "jobs_count": 1,
        "jobs_scope": "run_attempt",
        "artifacts_count": 0,
        "artifacts_scope": "whole_run",
        "workflow_source_blob_sha": "e".repeat(40),
        "workflow_source_bytes": 128,
        "workflow_source_sha256": "4".repeat(64),
        "source_state": "fetched",
        "product_execution": "NOT_RUN",
        "error": null
    })
}

fn stored_zip(member_name: &str, json_bytes: &[u8]) -> Vec<u8> {
    let crc = crc32fast::hash(json_bytes);
    let name = member_name.as_bytes();
    let size = u32::try_from(json_bytes.len()).expect("fixture JSON fits ZIP32");
    let mut archive = Vec::new();
    push_u32(&mut archive, 0x0403_4b50);
    push_u16(&mut archive, 20);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u32(&mut archive, crc);
    push_u32(&mut archive, size);
    push_u32(&mut archive, size);
    push_u16(&mut archive, name.len() as u16);
    push_u16(&mut archive, 0);
    archive.extend_from_slice(name);
    archive.extend_from_slice(json_bytes);
    let central_offset = archive.len() as u32;
    push_u32(&mut archive, 0x0201_4b50);
    push_u16(&mut archive, 20);
    push_u16(&mut archive, 20);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u32(&mut archive, crc);
    push_u32(&mut archive, size);
    push_u32(&mut archive, size);
    push_u16(&mut archive, name.len() as u16);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u32(&mut archive, 0);
    push_u32(&mut archive, 0);
    archive.extend_from_slice(name);
    let central_size = archive.len() as u32 - central_offset;
    push_u32(&mut archive, 0x0605_4b50);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 0);
    push_u16(&mut archive, 1);
    push_u16(&mut archive, 1);
    push_u32(&mut archive, central_size);
    push_u32(&mut archive, central_offset);
    push_u16(&mut archive, 0);
    archive
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn base_responses(archive_name: &str) -> (Vec<ResponsePlan>, Vec<u8>) {
    base_responses_for_target(archive_name, 101, 11, 1)
}

fn base_responses_for_target(
    archive_name: &str,
    target_run_id: u64,
    target_run_number: u64,
    target_attempt: u32,
) -> (Vec<ResponsePlan>, Vec<u8>) {
    let archive = stored_zip(
        archive_name,
        &serde_json::to_vec(&observer_report_for_target(target_run_id, target_attempt)).unwrap(),
    );
    let archive_sha = sha256_hex(&archive);
    let workflow_b64 = STANDARD.encode(publisher_workflow_bytes());
    let mut responses = vec![
        ResponsePlan::json(
            200,
            run_json(
                100,
                10,
                1,
                50,
                CALLER_PATH,
                "workflow_run",
                HEAD_SHA,
                "success",
            ),
        ),
        ResponsePlan::json(
            200,
            json!({
                "path": CALLER_PATH,
                "sha": "5".repeat(40),
                "size": publisher_workflow_bytes().len(),
                "encoding": "base64",
                "content": workflow_b64,
                "type": "file"
            }),
        ),
        ResponsePlan::json(
            200,
            json!({
                "total_count": 1,
                "workflow_runs": [
                    run_json(
                        target_run_id,
                        target_run_number,
                        target_attempt,
                        7,
                        TARGET_PATH,
                        "push",
                        HEAD_SHA,
                        "failure",
                    )
                ]
            }),
        ),
        ResponsePlan::json(
            200,
            run_json(102, 12, 1, 90, OBSERVER_PATH, "push", HEAD_SHA, "success"),
        ),
        ResponsePlan::json(
            200,
            json!({
                "total_count": 1,
                "jobs": [{"id": 200, "name": "product", "status": "completed", "conclusion": "failure"}]
            }),
        ),
        ResponsePlan::json(
            200,
            json!({
                "total_count": 1,
                "artifacts": [{
                    "id": 900,
                    "name": "ci-observer-report-102-attempt-1",
                    "size_in_bytes": archive.len(),
                    "expired": false,
                    "digest": format!("sha256:{archive_sha}"),
                    "workflow_run": {"id": 102, "head_branch": BRANCH, "head_sha": HEAD_SHA}
                }]
            }),
        ),
        ResponsePlan::bytes(200, "application/zip", archive.clone()),
    ];
    responses.extend(write_responses(
        HEAD_SHA,
        None,
        target_run_number,
        target_attempt,
    ));
    (responses, archive)
}

fn write_responses(
    ref_sha: &str,
    existing_cursor: Option<Value>,
    target_run_number: u64,
    target_attempt: u32,
) -> Vec<ResponsePlan> {
    let commit_message = if let Some(cursor) = &existing_cursor {
        format!(
            "docs(visibility): publish validated status [skip ci]\n\n\
             Termrock-Status-Publish: v1\n\
             Termrock-Status-Source: {}\n\
             Termrock-Status-Run: 7/{}/{}",
            cursor["last_published"]["source_sha"].as_str().unwrap(),
            cursor["last_published"]["run_number"].as_u64().unwrap(),
            cursor["last_published"]["run_attempt"].as_u64().unwrap()
        )
    } else {
        "source commit".to_owned()
    };
    let mut responses = vec![
        ResponsePlan::json(200, json!({"object": {"sha": ref_sha, "type": "commit"}})),
        ResponsePlan::json(
            200,
            json!({"sha": ref_sha, "message": commit_message, "tree": {"sha": TREE_SHA}}),
        ),
    ];
    match existing_cursor {
        Some(cursor) => {
            let bytes = serde_json::to_vec(&cursor).unwrap();
            responses.push(ResponsePlan::json(
                200,
                json!({
                    "path": "docs/implementation/visibility/evidence/status-publication.json",
                    "sha": "6".repeat(40),
                    "size": bytes.len(),
                    "encoding": "base64",
                    "content": STANDARD.encode(bytes),
                    "type": "file"
                }),
            ));
        }
        None => responses.push(ResponsePlan::json(404, json!({"message":"Not Found"}))),
    }
    responses.extend([
        ResponsePlan::json(201, json!({"sha": "7".repeat(40)})),
        ResponsePlan::json(201, json!({"sha": "8".repeat(40)})),
        ResponsePlan::json(201, json!({"sha": "9".repeat(40)})),
        ResponsePlan::json(
            201,
            json!({
                "sha": NEW_COMMIT_SHA,
                "message": format!("{target_run_number}/{target_attempt}"),
                "tree": {"sha": "9".repeat(40)}
            }),
        ),
        ResponsePlan::json(
            200,
            json!({"object": {"sha": NEW_COMMIT_SHA, "type": "commit"}}),
        ),
    ]);
    responses
}

fn cursor(source_sha: &str, run_number: u64, attempt: u32) -> Value {
    json!({
        "record_type": "termrock.visibility.status_publication_cursor.v1",
        "schema_version": 1,
        "repository": FULL_REPO,
        "branch": BRANCH,
        "last_published": {
            "workflow_id": 7,
            "run_id": 101,
            "run_number": run_number,
            "run_attempt": attempt,
            "source_sha": source_sha
        },
        "observer": {
            "run_id": 102,
            "run_attempt": 1,
            "observer_state": "measured",
            "source_state": "fetched",
            "artifact_sha256": "a".repeat(64)
        },
        "product": {"execution":"NOT_RUN","conclusion":"failure"},
        "status": {
            "path":"STATUS.md",
            "sha256": "b".repeat(64),
            "source_facts_sha256": "c".repeat(64),
            "tasks_sha256": "d".repeat(64),
            "generator_sha256": "e".repeat(64)
        }
    })
}

#[test]
fn status_publish_command_commits_only_status_and_cursor_and_preserves_product_failure() {
    let (mut responses, _) = base_responses("ci-observer-report.json");
    let mut server = TestServer::start(std::mem::take(&mut responses));
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output: Value = serde_json::from_slice(&result.stdout).expect("success JSON is emitted");
    assert_eq!(output["status"], "published");
    assert_eq!(output["target_conclusion"], "failure");
    let requests = server.captured();
    let writes: Vec<_> = requests
        .iter()
        .filter(|request| request.method != "GET")
        .collect();
    assert_eq!(writes.len(), 5);
    assert!(requests.iter().any(|request| {
        request.method == "GET"
            && request.path == format!("/repos/{FULL_REPO}/git/ref/heads/{BRANCH}")
    }));
    assert_eq!(writes[0].path, format!("/repos/{FULL_REPO}/git/blobs"));
    assert_eq!(writes[1].path, format!("/repos/{FULL_REPO}/git/blobs"));
    let tree: Value = serde_json::from_slice(&writes[2].body).unwrap();
    let paths: Vec<_> = tree["tree"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "STATUS.md",
            "docs/implementation/visibility/evidence/status-publication.json"
        ]
    );
    let commit: Value = serde_json::from_slice(&writes[3].body).unwrap();
    assert_eq!(commit["parents"][0], HEAD_SHA);
    assert!(commit["message"].as_str().unwrap().contains("[skip ci]"));
    let update: Value = serde_json::from_slice(&writes[4].body).unwrap();
    assert_eq!(update["force"], false);
    assert_eq!(update["sha"], NEW_COMMIT_SHA);
    assert_eq!(
        writes[4].path,
        format!("/repos/{FULL_REPO}/git/refs/heads/{BRANCH}")
    );
    let expected_authorization = format!("Bearer {TOKEN}");
    assert!(requests.iter().all(|request| {
        request.headers.get("authorization").map(String::as_str)
            == Some(expected_authorization.as_str())
    }));
    let cursor_body: Value = serde_json::from_slice(&writes[1].body).unwrap();
    let cursor_bytes = STANDARD
        .decode(cursor_body["content"].as_str().unwrap())
        .unwrap();
    let published_cursor: Value = serde_json::from_slice(&cursor_bytes).unwrap();
    assert_eq!(published_cursor["observer"]["observer_state"], "measured");
    assert_eq!(published_cursor["observer"]["source_state"], "fetched");
    assert_eq!(published_cursor["product"]["execution"], "NOT_RUN");
    assert_eq!(published_cursor["product"]["conclusion"], "failure");
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"target_conclusion\":\"failure\""));
}

#[test]
fn untrusted_repository_event_ref_and_workflow_source_are_rejected_before_network() {
    for (field, bad_value) in [
        ("repository", json!("attacker/fork")),
        ("branch", json!("untrusted")),
        ("caller.event", json!("pull_request")),
        ("caller.workflow_path", json!(".github/workflows/evil.yml")),
        ("caller.workflow_sha", json!("f".repeat(40))),
        ("status.source_commit", json!("f".repeat(40))),
        ("status.sha256", json!("0".repeat(64))),
    ] {
        let server = TestServer::start(Vec::new());
        let mut request = input();
        set_nested(&mut request, field, bad_value);
        let result = run_command(&server, &request, &policy());
        let mut server = server;
        server.finish();
        assert!(!result.status.success(), "{field} unexpectedly accepted");
        assert!(server.captured().is_empty(), "{field} reached GitHub API");
    }
}

#[test]
fn stale_remote_branch_tip_fails_before_any_write() {
    let (base, _) = base_responses("ci-observer-report.json");
    let mut responses = base[..7].to_vec();
    responses.push(ResponsePlan::json(
        200,
        json!({"object": {"sha": "f".repeat(40), "type": "commit"}}),
    ));
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("remote branch tip changed"));
    assert!(
        server
            .captured()
            .iter()
            .all(|request| request.method == "GET")
    );
}

#[test]
fn stale_source_commit_fails_before_any_write() {
    let remote_head = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    let mut request = input();
    request["expected_parent_sha"] = json!(remote_head);
    let mut responses = base_responses("ci-observer-report.json").0;
    responses.truncate(7);
    responses.extend(
        write_responses(remote_head, None, 11, 1)
            .into_iter()
            .take(3),
    );
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &request, &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("source commit is not the current branch source")
    );
    assert!(
        server
            .captured()
            .iter()
            .all(|request| request.method == "GET")
    );
}

#[test]
fn stale_source_after_existing_publisher_commit_and_valid_cursor_fails_before_writes() {
    let publisher_commit = "dddddddddddddddddddddddddddddddddddddddd";
    let previous_source = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let mut request = input();
    request["expected_parent_sha"] = json!(publisher_commit);

    let previous_cursor = cursor(previous_source, 11, 1);
    let mut responses = base_responses_for_target("ci-observer-report.json", 103, 12, 1).0;
    responses.truncate(7);
    responses.extend(
        write_responses(publisher_commit, Some(previous_cursor), 12, 1)
            .into_iter()
            .take(3),
    );

    let mut server = TestServer::start(responses);
    let result = run_command(&server, &request, &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("source commit is not the current branch source")
    );
    let requests = server.captured();
    assert!(requests.iter().all(|request| request.method == "GET"));
    assert!(requests.iter().any(|request| {
        request.method == "GET"
            && request.path
                == format!("/repos/{FULL_REPO}/contents/docs/implementation/visibility/evidence/status-publication.json?ref={publisher_commit}")
    }));
}

#[test]
fn duplicate_or_older_target_run_cursor_is_rejected_before_writes() {
    for (run_id, run_number, attempt, existing_number, existing_attempt) in
        [(101, 11, 1, 11, 1), (99, 10, 1, 11, 1)]
    {
        let request = input();
        let old_cursor = cursor(HEAD_SHA, existing_number, existing_attempt);
        let mut responses =
            base_responses_for_target("ci-observer-report.json", run_id, run_number, attempt).0;
        responses.truncate(7);
        responses.extend(write_responses(
            HEAD_SHA,
            Some(old_cursor),
            run_number,
            attempt,
        ));
        let mut server = TestServer::start(responses);
        let result = run_command(&server, &request, &policy());
        server.finish();

        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate or older"));
        assert!(
            server
                .captured()
                .iter()
                .all(|captured| captured.method == "GET")
        );
    }
}

#[test]
fn newer_target_run_on_same_source_advances_cursor() {
    let old_cursor = cursor(HEAD_SHA, 11, 1);
    let mut responses = base_responses_for_target("ci-observer-report.json", 103, 12, 1).0;
    responses.truncate(7);
    responses.extend(write_responses(HEAD_SHA, Some(old_cursor), 12, 1));
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let writes: Vec<_> = server
        .captured()
        .into_iter()
        .filter(|request| request.method != "GET")
        .collect();
    assert_eq!(writes.len(), 5);
    let cursor_body: Value = serde_json::from_slice(&writes[1].body).unwrap();
    let cursor_bytes = STANDARD
        .decode(cursor_body["content"].as_str().unwrap())
        .unwrap();
    let published_cursor: Value = serde_json::from_slice(&cursor_bytes).unwrap();
    assert_eq!(published_cursor["last_published"]["run_id"], 103);
    assert_eq!(published_cursor["last_published"]["run_number"], 12);
}

#[test]
fn observer_report_digest_mismatch_never_reaches_the_writer() {
    let (mut responses, _) = base_responses("ci-observer-report.json");
    let artifact_response = &mut responses[5];
    let mut artifact: Value = serde_json::from_slice(&artifact_response.body).unwrap();
    artifact["artifacts"][0]["digest"] = json!(format!("sha256:{}", "0".repeat(64)));
    artifact_response.body = serde_json::to_vec(&artifact).unwrap();
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("digest"));
    assert!(
        server
            .captured()
            .iter()
            .all(|request| request.method == "GET")
    );
}

#[test]
fn failed_write_step_is_not_retried_and_does_not_update_the_ref() {
    let (mut responses, _) = base_responses("ci-observer-report.json");
    responses.truncate(10);
    responses.push(ResponsePlan::json(
        500,
        json!({"message":"injected blob failure"}),
    ));
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(!result.status.success());
    let requests = server.captured();
    let blobs: Vec<_> = requests
        .iter()
        .filter(|request| {
            request.method == "POST" && request.path == format!("/repos/{FULL_REPO}/git/blobs")
        })
        .collect();
    assert_eq!(blobs.len(), 1, "failed mutation was not retried");
    assert!(!requests.iter().any(|request| request.method == "PATCH"));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("published"));
}

#[test]
fn compare_and_swap_conflict_is_not_retried_or_reported_as_published() {
    let (mut responses, _) = base_responses("ci-observer-report.json");
    responses[14] = ResponsePlan::json(409, json!({"message": "reference changed"}));
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(!String::from_utf8_lossy(&result.stdout).contains("published"));
    let requests = server.captured();
    let updates: Vec<_> = requests
        .iter()
        .filter(|request| request.method == "PATCH")
        .collect();
    assert_eq!(
        updates.len(),
        1,
        "the compare-and-swap update is attempted once"
    );
    let update: Value = serde_json::from_slice(&updates[0].body).unwrap();
    assert_eq!(update["force"], false);
    assert_eq!(update["sha"], NEW_COMMIT_SHA);
    assert_eq!(
        updates[0].path,
        format!("/repos/{FULL_REPO}/git/refs/heads/{BRANCH}")
    );
}

#[test]
fn unsafe_zip_member_is_rejected_without_extraction_or_publication() {
    let (mut responses, _) = base_responses("../observer.sh");
    responses.truncate(7);
    let mut server = TestServer::start(responses);
    let result = run_command(&server, &input(), &policy());
    server.finish();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("path"));
    assert!(
        server
            .captured()
            .iter()
            .all(|request| request.method == "GET")
    );
}

fn set_nested(value: &mut Value, dotted: &str, replacement: Value) {
    let mut parts = dotted.split('.').peekable();
    let mut current = value;
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current[part] = replacement;
            return;
        }
        current = &mut current[part];
    }
}

