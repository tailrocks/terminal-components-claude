//! Bounded GitHub status publication over the Git database API.
//!
//! The read-only observer transport remains GET-only. This module owns the
//! separate write transport and is deliberately disabled by the command until
//! a trusted generated publisher workflow is accepted.

use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use termrock_visibility_publisher::api::{
    ApiClient, HttpRequest, HttpResponse, ReadOnlyTransport, Repository, RunSelector,
    TransportError,
};
use termrock_visibility_publisher::{
    CallerIdentity, IngestError, IngestRequest, IngestedObservation, RunIdentity, ingest,
};

pub const STATUS_PATH: &str = "STATUS.md";
pub const CURSOR_PATH: &str = "docs/implementation/visibility/evidence/status-publication.json";
pub const LIVE_PUBLISHING_ENABLED: bool = false;
const API_ORIGIN: &str = "https://api.github.com";
const API_HOST: &str = "api.github.com";
const CDN_HOST: &str = "objects.githubusercontent.com";
const MAX_INPUT_BYTES: usize = 2 * 1024 * 1024;
const MAX_STATUS_BYTES: usize = 1024 * 1024;
const MAX_CURSOR_BYTES: usize = 64 * 1024;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_TEST_CA_BYTES: usize = 64 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const USER_AGENT: &str = "termrock-visibility-publisher";
const TEST_MODE_ENV: &str = "TERMROCK_STATUS_PUBLISH_TEST_MODE";
const TEST_CA_ENV: &str = "TERMROCK_STATUS_PUBLISH_TEST_CA_PEM";
const TEST_PORT_ENV: &str = "TERMROCK_STATUS_PUBLISH_TEST_PORT";
const TEST_POLICY_ENV: &str = "TERMROCK_STATUS_PUBLISH_TEST_POLICY_JSON";
const PUBLISH_MARKER: &str = "Termrock-Status-Publish: v1";
const SOURCE_MARKER: &str = "Termrock-Status-Source: ";
const RUN_MARKER: &str = "Termrock-Status-Run: ";
const COMMIT_SUBJECT: &str = "docs(visibility): publish validated status [skip ci]";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedPublisherPolicy {
    pub repository: String,
    pub branch: String,
    pub publisher_event: String,
    pub publisher_workflow_id: u64,
    pub publisher_workflow_path: String,
    pub publisher_workflow_sha256: String,
    pub target_workflow_id: u64,
    pub target_workflow_path: String,
    pub target_event: String,
    pub observer_workflow_id: u64,
    pub observer_workflow_path: String,
    pub observer_event: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishInvocation {
    pub schema_version: u16,
    pub repository: String,
    pub branch: String,
    pub expected_parent_sha: String,
    pub caller: CallerInput,
    pub target: TargetInput,
    pub observer: RunInput,
    pub observer_artifact_name: String,
    pub status: StatusInput,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallerInput {
    pub run_id: u64,
    pub attempt: u32,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub event: String,
    pub branch: String,
    pub head_sha: String,
    pub workflow_sha: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub workflow_id: u64,
    pub workflow_path: String,
    pub event: String,
    pub branch: String,
    pub head_sha: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInput {
    pub run_id: u64,
    pub attempt: u32,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub event: String,
    pub branch: String,
    pub head_sha: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusInput {
    pub markdown: String,
    pub sha256: String,
    pub source_commit: String,
    pub source_facts_sha256: String,
    pub tasks_sha256: String,
    pub generator_sha256: String,
}

#[derive(Clone, Debug)]
pub struct GitHubContext {
    repository: String,
    event: String,
    reference: String,
    branch: String,
    head_sha: String,
    run_id: u64,
    run_attempt: u32,
    workflow_ref: String,
    workflow_sha: String,
}

impl GitHubContext {
    pub fn from_environment() -> Result<Self, PublishError> {
        let repository = required_env("GITHUB_REPOSITORY")?;
        let event = required_env("GITHUB_EVENT_NAME")?;
        let reference = required_env("GITHUB_REF")?;
        let branch = required_env("GITHUB_REF_NAME")?;
        let head_sha = required_env("GITHUB_SHA")?;
        let run_id = positive_env("GITHUB_RUN_ID")?;
        let run_attempt = u32::try_from(positive_env("GITHUB_RUN_ATTEMPT")?)
            .map_err(|_| PublishError::InvalidInput("GITHUB_RUN_ATTEMPT is out of range"))?;
        let workflow_ref = required_env("GITHUB_WORKFLOW_REF")?;
        let workflow_sha = required_env("GITHUB_WORKFLOW_SHA")?;
        Ok(Self {
            repository,
            event,
            reference,
            branch,
            head_sha,
            run_id,
            run_attempt,
            workflow_ref,
            workflow_sha,
        })
    }
}

#[derive(Clone)]
pub struct StatusTransport {
    client: Client,
    token: String,
}

impl StatusTransport {
    pub fn from_environment(token: impl Into<String>) -> Result<Self, PublishError> {
        let token = token.into();
        if token.is_empty() || token.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(PublishError::InvalidInput(
                "GitHub token is empty or malformed",
            ));
        }
        let mut builder = base_client();

        if std::env::var(TEST_MODE_ENV).ok().as_deref() == Some("1") {
            #[cfg(debug_assertions)]
            {
                let root_pem = bounded_file(&required_env(TEST_CA_ENV)?, MAX_TEST_CA_BYTES)?;
                let root = reqwest::Certificate::from_pem(&root_pem)
                    .map_err(|_| PublishError::InvalidInput("test TLS root is invalid"))?;
                let port = required_env(TEST_PORT_ENV)?
                    .parse::<u16>()
                    .map_err(|_| PublishError::InvalidInput("test TLS port is invalid"))?;
                if port == 0 {
                    return Err(PublishError::InvalidInput("test TLS port must be positive"));
                }
                let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
                builder = builder
                    .tls_certs_only(vec![root])
                    .resolve(API_HOST, address)
                    .resolve(CDN_HOST, address);
            }
            #[cfg(not(debug_assertions))]
            {
                return Err(PublishError::TestModeUnavailable);
            }
        } else if std::env::var_os(TEST_CA_ENV).is_some()
            || std::env::var_os(TEST_PORT_ENV).is_some()
        {
            return Err(PublishError::InvalidInput(
                "test TLS settings require the explicit test mode",
            ));
        }

        let client = builder
            .build()
            .map_err(|_| PublishError::Transport(TransportError::Unavailable))?;
        Ok(Self { client, token })
    }

    fn request(
        &self,
        method: Method,
        url: &str,
        authorization: Option<&str>,
        body: Option<&[u8]>,
        deadline: Instant,
        max_bytes: usize,
    ) -> Result<HttpResponse, TransportError> {
        let parsed = Url::parse(url).map_err(|_| TransportError::Unavailable)?;
        if !valid_api_url(&parsed, authorization.is_some()) {
            return Err(TransportError::Unavailable);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or(TransportError::Deadline)?;
        let mut request = self.client.request(method, parsed).timeout(remaining);
        if let Some(token) = authorization {
            let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| TransportError::Unavailable)?;
            value.set_sensitive(true);
            request = request
                .header(AUTHORIZATION, value)
                .header(ACCEPT, "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28");
        }
        if let Some(body) = body {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(body.to_vec());
        }
        let mut response = request.send().map_err(|error| {
            if error.is_timeout() || Instant::now() >= deadline {
                TransportError::Deadline
            } else {
                TransportError::Unavailable
            }
        })?;
        if Instant::now() >= deadline {
            return Err(TransportError::Deadline);
        }
        if response
            .content_length()
            .is_some_and(|length| length > max_bytes as u64)
        {
            return Err(TransportError::BodyLimit);
        }
        let status = response.status().as_u16();
        let headers = selected_headers(response.headers());
        let body = read_bounded(&mut response, deadline, max_bytes)?;
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    fn api_get(&self, path: &str, max_bytes: usize) -> Result<HttpResponse, PublishError> {
        self.request_json_method(Method::GET, path, None, max_bytes)
    }

    fn request_json_method(
        &self,
        method: Method,
        path: &str,
        body: Option<&[u8]>,
        max_bytes: usize,
    ) -> Result<HttpResponse, PublishError> {
        let url = format!("{API_ORIGIN}{path}");
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        self.request(method, &url, Some(&self.token), body, deadline, max_bytes)
            .map_err(PublishError::Transport)
    }

    fn json_response(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        expected_status: u16,
        stage: &'static str,
    ) -> Result<Value, PublishError> {
        let encoded = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|_| PublishError::InvalidResponse)?;
        let response =
            self.request_json_method(method, path, encoded.as_deref(), MAX_RESPONSE_BYTES)?;
        if response.status != expected_status {
            return Err(PublishError::HttpStatus {
                stage,
                status: response.status,
            });
        }
        serde_json::from_slice(&response.body).map_err(|_| PublishError::InvalidResponse)
    }
}

impl ReadOnlyTransport for StatusTransport {
    fn get(
        &self,
        request: HttpRequest,
        deadline: Instant,
        max_bytes: usize,
    ) -> Result<HttpResponse, TransportError> {
        self.request(
            Method::GET,
            request.url(),
            request.authorization(),
            None,
            deadline,
            max_bytes,
        )
    }
}

#[derive(Debug)]
pub enum PublishError {
    InvalidInput(&'static str),
    LivePublishingDisabled,
    TestModeUnavailable,
    Transport(TransportError),
    Ingest(IngestError),
    HttpStatus { stage: &'static str, status: u16 },
    InvalidResponse,
    StaleBranchTip,
    StaleSource,
    StaleRunCursor,
    ExistingCursorMismatch,
    UntrustedWorkflow,
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid status publication input: {message}"),
            Self::LivePublishingDisabled => f.write_str(
                "live status publication is disabled pending trusted workflow and branch authority",
            ),
            Self::TestModeUnavailable => {
                f.write_str("status publication test mode is unavailable in release builds")
            }
            Self::Transport(TransportError::Deadline) => {
                f.write_str("GitHub request deadline exceeded")
            }
            Self::Transport(TransportError::BodyLimit) => {
                f.write_str("GitHub response exceeded its byte limit")
            }
            Self::Transport(TransportError::Unavailable) => f.write_str("GitHub request failed"),
            Self::Ingest(error) => write!(f, "observer ingestion failed: {error}"),
            Self::HttpStatus { stage, status } => write!(f, "{stage} returned HTTP {status}"),
            Self::InvalidResponse => {
                f.write_str("GitHub response did not match the required schema")
            }
            Self::StaleBranchTip => {
                f.write_str("remote branch tip changed; publication rejected without retry")
            }
            Self::StaleSource => f.write_str("source commit is not the current branch source"),
            Self::StaleRunCursor => {
                f.write_str("target workflow run is duplicate or older than the publication cursor")
            }
            Self::ExistingCursorMismatch => {
                f.write_str("remote publication cursor does not match its commit")
            }
            Self::UntrustedWorkflow => {
                f.write_str("caller workflow is outside the accepted source pin")
            }
        }
    }
}

impl std::error::Error for PublishError {}

impl From<IngestError> for PublishError {
    fn from(value: IngestError) -> Self {
        Self::Ingest(value)
    }
}

#[derive(Clone, Debug, Serialize)]
struct PublicationCursor {
    record_type: &'static str,
    schema_version: u16,
    repository: String,
    branch: String,
    last_published: RunCursor,
    observer: ObserverCursor,
    product: ProductCursor,
    status: StatusCursor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunCursor {
    workflow_id: u64,
    run_id: u64,
    run_number: u64,
    run_attempt: u32,
    source_sha: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ObserverCursor {
    run_id: u64,
    run_attempt: u32,
    observer_state: String,
    source_state: String,
    artifact_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductCursor {
    execution: String,
    conclusion: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StatusCursor {
    path: String,
    sha256: String,
    source_facts_sha256: String,
    tasks_sha256: String,
    generator_sha256: String,
}

#[derive(Deserialize)]
struct RefResponse {
    object: RefObject,
}

#[derive(Deserialize)]
struct RefObject {
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct CommitResponse {
    sha: String,
    message: String,
    tree: CommitTree,
}

#[derive(Deserialize)]
struct CommitTree {
    sha: String,
}

#[derive(Deserialize)]
struct ContentsResponse {
    path: String,
    sha: String,
    size: usize,
    encoding: String,
    content: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct CreatedObject {
    sha: String,
}

#[derive(Deserialize)]
struct UpdatedRef {
    object: RefObject,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExistingCursor {
    record_type: String,
    schema_version: u16,
    repository: String,
    branch: String,
    last_published: RunCursor,
    observer: ObserverCursor,
    product: ProductCursor,
    status: StatusCursor,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublishResult {
    pub status: &'static str,
    pub branch: String,
    pub parent_sha: String,
    pub commit_sha: String,
    pub status_sha256: String,
    pub target_conclusion: Option<String>,
}

pub fn publish(
    input: PublishInvocation,
    policy: TrustedPublisherPolicy,
    context: GitHubContext,
    transport: StatusTransport,
) -> Result<PublishResult, PublishError> {
    validate_policy(&policy)?;
    validate_invocation(&input, &policy, &context)?;

    let repository = Repository::new("tailrocks", "terminal-components-claude")
        .map_err(|_| PublishError::InvalidInput("repository is invalid"))?;
    let api = ApiClient::new(transport.clone(), transport.token.clone())
        .map_err(|_| PublishError::InvalidInput("GitHub token is invalid"))?;
    let request = build_ingest_request(&input, repository)?;
    let observation = ingest(&api, &request)?;
    if observation.caller_workflow_sha256 != policy.publisher_workflow_sha256 {
        return Err(PublishError::UntrustedWorkflow);
    }
    validate_observation(&input, &policy, &observation)?;

    publish_git_tree(&transport, &input, &policy, &observation)
}

fn validate_policy(policy: &TrustedPublisherPolicy) -> Result<(), PublishError> {
    if policy.repository != "tailrocks/terminal-components-claude"
        || !safe_ref_name(&policy.branch)
        || !safe_workflow_path(&policy.publisher_workflow_path)
        || !safe_workflow_path(&policy.target_workflow_path)
        || !safe_workflow_path(&policy.observer_workflow_path)
        || !is_sha256(&policy.publisher_workflow_sha256)
        || policy.publisher_workflow_id == 0
        || policy.target_workflow_id == 0
        || policy.observer_workflow_id == 0
        || !valid_event(&policy.publisher_event)
        || !valid_event(&policy.target_event)
        || !valid_event(&policy.observer_event)
    {
        return Err(PublishError::InvalidInput(
            "trusted policy has an invalid identity pin",
        ));
    }
    Ok(())
}

fn validate_invocation(
    input: &PublishInvocation,
    policy: &TrustedPublisherPolicy,
    context: &GitHubContext,
) -> Result<(), PublishError> {
    if input.schema_version != 1 {
        return Err(PublishError::InvalidInput("unsupported request schema"));
    }
    if input.repository != policy.repository
        || input.branch != policy.branch
        || input.expected_parent_sha.len() != 40
        || !is_sha1(&input.expected_parent_sha)
    {
        return Err(PublishError::InvalidInput(
            "repository, branch, or expected parent does not match policy",
        ));
    }
    if context.repository != input.repository
        || context.event != policy.publisher_event
        || context.reference != format!("refs/heads/{}", policy.branch)
        || context.branch != policy.branch
        || context.head_sha != input.caller.head_sha
        || context.run_id != input.caller.run_id
        || context.run_attempt != input.caller.attempt
        || context.workflow_sha != input.caller.workflow_sha
        || context.workflow_ref
            != format!(
                "{}/{}@refs/heads/{}",
                input.repository, policy.publisher_workflow_path, policy.branch
            )
    {
        return Err(PublishError::InvalidInput(
            "GitHub event context does not match the trusted run",
        ));
    }
    if input.caller.run_id == 0
        || input.caller.attempt == 0
        || input.caller.workflow_id != policy.publisher_workflow_id
        || input.caller.workflow_path != policy.publisher_workflow_path
        || input.caller.event != policy.publisher_event
        || input.caller.branch != policy.branch
        || input.caller.head_sha != context.head_sha
        || !is_sha1(&input.caller.head_sha)
        || !is_sha1(&input.caller.workflow_sha)
    {
        return Err(PublishError::UntrustedWorkflow);
    }
    if input.target.workflow_id != policy.target_workflow_id
        || input.target.workflow_path != policy.target_workflow_path
        || input.target.event != policy.target_event
        || input.target.branch != policy.branch
        || input.target.head_sha != input.caller.head_sha
        || input.observer.workflow_id != policy.observer_workflow_id
        || input.observer.workflow_path != policy.observer_workflow_path
        || input.observer.event != policy.observer_event
        || input.observer.branch != policy.branch
        || input.observer.head_sha != input.caller.head_sha
        || input.observer.run_id == 0
        || input.observer.attempt == 0
    {
        return Err(PublishError::InvalidInput(
            "target or observer identity does not match policy",
        ));
    }
    if input.status.source_commit != input.caller.head_sha
        || input.status.markdown.is_empty()
        || input.status.markdown.len() > MAX_STATUS_BYTES
        || input.status.markdown.contains('\0')
        || !is_sha1(&input.status.source_commit)
        || !is_sha256(&input.status.sha256)
        || !is_sha256(&input.status.source_facts_sha256)
        || !is_sha256(&input.status.tasks_sha256)
        || !is_sha256(&input.status.generator_sha256)
        || sha256_hex(input.status.markdown.as_bytes()) != input.status.sha256
    {
        return Err(PublishError::InvalidInput(
            "generated status bytes or source pins are invalid",
        ));
    }
    if input.observer_artifact_name.is_empty()
        || input.observer_artifact_name.len() > 256
        || input.observer_artifact_name.contains('/')
        || input.observer_artifact_name.contains('\\')
    {
        return Err(PublishError::InvalidInput(
            "observer artifact name is unsafe",
        ));
    }
    Ok(())
}

fn validate_observation(
    input: &PublishInvocation,
    policy: &TrustedPublisherPolicy,
    observation: &IngestedObservation,
) -> Result<(), PublishError> {
    if observation.caller.repository != policy.repository
        || observation.caller.workflow_id != policy.publisher_workflow_id
        || observation.caller.workflow_path != policy.publisher_workflow_path
        || observation.caller.event != policy.publisher_event
        || observation.caller.head_branch != policy.branch
        || observation.caller.head_sha != input.status.source_commit
        || observation.target.workflow_id != policy.target_workflow_id
        || observation.target.workflow_path != policy.target_workflow_path
        || observation.target.event != policy.target_event
        || observation.target.head_branch != policy.branch
        || observation.target.head_sha != input.status.source_commit
        || observation.observer.workflow_id != policy.observer_workflow_id
        || observation.observer.workflow_path != policy.observer_workflow_path
        || observation.observer.event != policy.observer_event
        || observation.observer.head_branch != policy.branch
        || observation.observer.head_sha != input.status.source_commit
        || observation.target.status != "completed"
        || observation.observer_report.observer_state != "measured"
        || observation.observer_report.product_execution != "NOT_RUN"
    {
        return Err(PublishError::InvalidInput(
            "validated observer record does not match the accepted run",
        ));
    }
    // A product failure is data to publish. The observer measures the run and
    // never changes the product conclusion into a publisher success/failure.
    Ok(())
}

fn build_ingest_request(
    input: &PublishInvocation,
    repository: Repository,
) -> Result<IngestRequest, PublishError> {
    let caller = RunIdentity {
        repository: repository.clone(),
        run_id: input.caller.run_id,
        attempt: input.caller.attempt,
        workflow_id: input.caller.workflow_id,
        workflow_path: input.caller.workflow_path.clone(),
        event: input.caller.event.clone(),
        branch: input.caller.branch.clone(),
        head_sha: input.caller.head_sha.clone(),
    };
    let target = RunSelector {
        repository: repository.clone(),
        workflow_id: input.target.workflow_id,
        workflow_path: input.target.workflow_path.clone(),
        head_sha: input.target.head_sha.clone(),
        branch: input.target.branch.clone(),
        event: input.target.event.clone(),
    };
    let observer = RunIdentity {
        repository,
        run_id: input.observer.run_id,
        attempt: input.observer.attempt,
        workflow_id: input.observer.workflow_id,
        workflow_path: input.observer.workflow_path.clone(),
        event: input.observer.event.clone(),
        branch: input.observer.branch.clone(),
        head_sha: input.observer.head_sha.clone(),
    };
    Ok(IngestRequest {
        caller: CallerIdentity {
            workflow_ref: format!(
                "{}/{}@refs/heads/{}",
                input.repository, input.caller.workflow_path, input.caller.branch
            ),
            workflow_sha: input.caller.workflow_sha.clone(),
            run: caller,
        },
        target,
        observer,
        observer_artifact_name: input.observer_artifact_name.clone(),
    })
}

fn publish_git_tree(
    transport: &StatusTransport,
    input: &PublishInvocation,
    policy: &TrustedPublisherPolicy,
    observation: &IngestedObservation,
) -> Result<PublishResult, PublishError> {
    let branch_path = encode_ref_path(&policy.branch)?;
    let reference_path = format!("/repos/{}/git/ref/heads/{branch_path}", policy.repository);
    let update_reference_path =
        format!("/repos/{}/git/refs/heads/{branch_path}", policy.repository);
    let current_ref_value = get_json(transport, &reference_path, "branch reference")?;
    let current_ref: RefResponse =
        serde_json::from_value(current_ref_value).map_err(|_| PublishError::InvalidResponse)?;
    if current_ref.object.kind != "commit"
        || current_ref.object.sha != input.expected_parent_sha
        || !is_sha1(&current_ref.object.sha)
    {
        return Err(PublishError::StaleBranchTip);
    }

    let current_commit_path = format!(
        "/repos/{}/git/commits/{}",
        policy.repository, input.expected_parent_sha
    );
    let current_commit_value = get_json(transport, &current_commit_path, "branch commit")?;
    let current_commit: CommitResponse =
        serde_json::from_value(current_commit_value).map_err(|_| PublishError::InvalidResponse)?;
    if current_commit.sha != input.expected_parent_sha || !is_sha1(&current_commit.tree.sha) {
        return Err(PublishError::InvalidResponse);
    }

    let cursor_path = contents_path(policy, input.expected_parent_sha.as_str())?;
    let existing_cursor = read_cursor(transport, &cursor_path, policy)?;
    validate_source_head(
        &current_commit.message,
        existing_cursor.as_ref(),
        input,
        observation,
        policy,
        &current_ref.object.sha,
    )?;

    let target_tuple = RunCursor {
        workflow_id: observation.target.workflow_id,
        run_id: observation.target.id,
        run_number: observation.target.run_number,
        run_attempt: observation.target.run_attempt,
        source_sha: observation.target.head_sha.clone(),
    };
    if let Some(previous) = existing_cursor.as_ref() {
        if previous.last_published.workflow_id != target_tuple.workflow_id
            || !run_cursor_is_newer(&target_tuple, &previous.last_published)
        {
            return Err(PublishError::StaleRunCursor);
        }
    }

    let cursor = PublicationCursor {
        record_type: "termrock.visibility.status_publication_cursor.v1",
        schema_version: 1,
        repository: policy.repository.clone(),
        branch: policy.branch.clone(),
        last_published: target_tuple.clone(),
        observer: ObserverCursor {
            run_id: observation.observer.id,
            run_attempt: observation.observer.run_attempt,
            observer_state: observation.observer_report.observer_state.clone(),
            source_state: observation.observer_report.source_state.clone(),
            artifact_sha256: observation.archive_sha256.clone(),
        },
        product: ProductCursor {
            execution: observation.observer_report.product_execution.clone(),
            conclusion: observation.target.conclusion.clone(),
        },
        status: StatusCursor {
            path: STATUS_PATH.to_owned(),
            sha256: input.status.sha256.clone(),
            source_facts_sha256: input.status.source_facts_sha256.clone(),
            tasks_sha256: input.status.tasks_sha256.clone(),
            generator_sha256: input.status.generator_sha256.clone(),
        },
    };
    let cursor_bytes =
        serde_json::to_vec_pretty(&cursor).map_err(|_| PublishError::InvalidResponse)?;
    if cursor_bytes.len() > MAX_CURSOR_BYTES {
        return Err(PublishError::InvalidInput(
            "publication cursor exceeds its byte limit",
        ));
    }

    let status_blob = create_blob(
        transport,
        policy,
        input.status.markdown.as_bytes(),
        "status document blob",
    )?;
    let cursor_blob = create_blob(transport, policy, &cursor_bytes, "publication cursor blob")?;
    let tree_value = transport.json_response(
        Method::POST,
        &format!("/repos/{}/git/trees", policy.repository),
        Some(&json!({
            "base_tree": current_commit.tree.sha,
            "tree": [
                {"path": STATUS_PATH, "mode": "100644", "type": "blob", "sha": status_blob},
                {"path": CURSOR_PATH, "mode": "100644", "type": "blob", "sha": cursor_blob}
            ]
        })),
        201,
        "create publication tree",
    )?;
    let tree: CreatedObject =
        serde_json::from_value(tree_value).map_err(|_| PublishError::InvalidResponse)?;
    if !is_sha1(&tree.sha) {
        return Err(PublishError::InvalidResponse);
    }

    let commit_message = format!(
        "{COMMIT_SUBJECT}\n\n{PUBLISH_MARKER}\n{SOURCE_MARKER}{}\n{RUN_MARKER}{}/{}/{}",
        target_tuple.source_sha,
        target_tuple.workflow_id,
        target_tuple.run_number,
        target_tuple.run_attempt
    );
    let commit_value = transport.json_response(
        Method::POST,
        &format!("/repos/{}/git/commits", policy.repository),
        Some(&json!({
            "message": commit_message,
            "tree": tree.sha,
            "parents": [input.expected_parent_sha]
        })),
        201,
        "create publication commit",
    )?;
    let commit: CommitResponse =
        serde_json::from_value(commit_value).map_err(|_| PublishError::InvalidResponse)?;
    if !is_sha1(&commit.sha) || commit.tree.sha != tree.sha {
        return Err(PublishError::InvalidResponse);
    }

    let update_value = transport.json_response(
        Method::PATCH,
        &update_reference_path,
        Some(&json!({"sha": commit.sha, "force": false})),
        200,
        "update branch reference",
    )?;
    let updated: UpdatedRef =
        serde_json::from_value(update_value).map_err(|_| PublishError::InvalidResponse)?;
    if updated.object.sha != commit.sha || updated.object.kind != "commit" {
        return Err(PublishError::InvalidResponse);
    }

    Ok(PublishResult {
        status: "published",
        branch: policy.branch.clone(),
        parent_sha: input.expected_parent_sha.clone(),
        commit_sha: commit.sha,
        status_sha256: input.status.sha256.clone(),
        target_conclusion: observation.target.conclusion.clone(),
    })
}

fn validate_source_head(
    message: &str,
    cursor: Option<&ExistingCursor>,
    input: &PublishInvocation,
    observation: &IngestedObservation,
    policy: &TrustedPublisherPolicy,
    remote_head: &str,
) -> Result<(), PublishError> {
    let publish_markers = message
        .lines()
        .filter(|line| *line == PUBLISH_MARKER)
        .count();
    if publish_markers > 1 {
        return Err(PublishError::ExistingCursorMismatch);
    }
    let has_publish_marker = publish_markers == 1;
    if has_publish_marker {
        let Some(previous) = cursor else {
            return Err(PublishError::ExistingCursorMismatch);
        };
        let Some(source) = marker_value(message, SOURCE_MARKER) else {
            return Err(PublishError::ExistingCursorMismatch);
        };
        let run_marker = format!(
            "{}/{}/{}",
            previous.last_published.workflow_id,
            previous.last_published.run_number,
            previous.last_published.run_attempt
        );
        let expected_run = marker_value(message, RUN_MARKER);
        if source != previous.last_published.source_sha
            || expected_run != Some(run_marker.as_str())
            || cursor_identity_matches(previous, policy).is_err()
        {
            return Err(PublishError::ExistingCursorMismatch);
        }
        // Re-runs of the same source commit may publish a newer target attempt.
        // A different source cannot be current while the last publisher commit
        // remains the remote tip.
        if input.status.source_commit != previous.last_published.source_sha {
            return Err(PublishError::StaleSource);
        }
    } else if input.status.source_commit != remote_head {
        // Before the first publisher commit, the source event must still be the
        // branch tip. This rejects delayed runs after a newer source push.
        return Err(PublishError::StaleSource);
    }
    if observation.target.head_sha != input.status.source_commit
        || observation.caller.head_sha != input.status.source_commit
        || input.branch != policy.branch
    {
        return Err(PublishError::StaleSource);
    }
    Ok(())
}

fn cursor_identity_matches(
    cursor: &ExistingCursor,
    policy: &TrustedPublisherPolicy,
) -> Result<(), PublishError> {
    if cursor.record_type != "termrock.visibility.status_publication_cursor.v1"
        || cursor.schema_version != 1
        || cursor.repository != policy.repository
        || cursor.branch != policy.branch
        || cursor.last_published.workflow_id != policy.target_workflow_id
        || !is_sha1(&cursor.last_published.source_sha)
        || cursor.last_published.run_id == 0
        || cursor.last_published.run_number == 0
        || cursor.last_published.run_attempt == 0
        || cursor.observer.run_id == 0
        || cursor.observer.run_attempt == 0
        || cursor.observer.observer_state != "measured"
        || cursor.observer.source_state != "fetched"
        || !is_sha256(&cursor.observer.artifact_sha256)
        || cursor.product.execution != "NOT_RUN"
        || cursor
            .product
            .conclusion
            .as_deref()
            .is_some_and(|value| value.is_empty())
        || cursor.status.path != STATUS_PATH
        || !is_sha256(&cursor.status.sha256)
        || !is_sha256(&cursor.status.source_facts_sha256)
        || !is_sha256(&cursor.status.tasks_sha256)
        || !is_sha256(&cursor.status.generator_sha256)
    {
        return Err(PublishError::ExistingCursorMismatch);
    }
    Ok(())
}

fn run_cursor_is_newer(candidate: &RunCursor, current: &RunCursor) -> bool {
    (candidate.run_number, candidate.run_attempt) > (current.run_number, current.run_attempt)
}

fn marker_value<'a>(message: &'a str, marker: &str) -> Option<&'a str> {
    let mut matches = message.lines().filter_map(|line| line.strip_prefix(marker));
    let value = matches.next()?;
    if matches.next().is_some() || value.is_empty() {
        return None;
    }
    Some(value)
}

fn read_cursor(
    transport: &StatusTransport,
    path: &str,
    policy: &TrustedPublisherPolicy,
) -> Result<Option<ExistingCursor>, PublishError> {
    let response = transport.api_get(path, MAX_RESPONSE_BYTES)?;
    if response.status == 404 {
        return Ok(None);
    }
    if response.status != 200 {
        return Err(PublishError::HttpStatus {
            stage: "read publication cursor",
            status: response.status,
        });
    }
    let contents: ContentsResponse =
        serde_json::from_slice(&response.body).map_err(|_| PublishError::InvalidResponse)?;
    if contents.path != CURSOR_PATH
        || contents.kind != "file"
        || contents.encoding != "base64"
        || contents.size > MAX_CURSOR_BYTES
        || !is_sha1(&contents.sha)
    {
        return Err(PublishError::ExistingCursorMismatch);
    }
    let compact: String = contents
        .content
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let decoded = STANDARD
        .decode(compact.as_bytes())
        .map_err(|_| PublishError::ExistingCursorMismatch)?;
    if decoded.len() != contents.size || decoded.len() > MAX_CURSOR_BYTES {
        return Err(PublishError::ExistingCursorMismatch);
    }
    let cursor: ExistingCursor =
        serde_json::from_slice(&decoded).map_err(|_| PublishError::ExistingCursorMismatch)?;
    cursor_identity_matches(&cursor, policy)?;
    Ok(Some(cursor))
}

fn create_blob(
    transport: &StatusTransport,
    policy: &TrustedPublisherPolicy,
    bytes: &[u8],
    stage: &'static str,
) -> Result<String, PublishError> {
    let encoded = STANDARD.encode(bytes);
    let response = transport.json_response(
        Method::POST,
        &format!("/repos/{}/git/blobs", policy.repository),
        Some(&json!({"content": encoded, "encoding": "base64"})),
        201,
        stage,
    )?;
    let blob: CreatedObject =
        serde_json::from_value(response).map_err(|_| PublishError::InvalidResponse)?;
    if !is_sha1(&blob.sha) {
        return Err(PublishError::InvalidResponse);
    }
    Ok(blob.sha)
}

fn get_json(
    transport: &StatusTransport,
    path: &str,
    stage: &'static str,
) -> Result<Value, PublishError> {
    let response = transport.api_get(path, MAX_RESPONSE_BYTES)?;
    if response.status != 200 {
        return Err(PublishError::HttpStatus {
            stage,
            status: response.status,
        });
    }
    serde_json::from_slice(&response.body).map_err(|_| PublishError::InvalidResponse)
}

fn contents_path(policy: &TrustedPublisherPolicy, ref_sha: &str) -> Result<String, PublishError> {
    if !is_sha1(ref_sha) {
        return Err(PublishError::InvalidInput(
            "expected publication ref is invalid",
        ));
    }
    Ok(format!(
        "/repos/{}/contents/{}?ref={}",
        policy.repository,
        encode_path(CURSOR_PATH),
        ref_sha
    ))
}

fn encode_ref_path(value: &str) -> Result<String, PublishError> {
    if !safe_ref_name(value) {
        return Err(PublishError::InvalidInput("branch name is unsafe"));
    }
    Ok(value
        .split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/"))
}

fn encode_path(value: &str) -> String {
    value
        .split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn valid_api_url(url: &Url, has_authorization: bool) -> bool {
    let authority = url.host_str().map(str::to_ascii_lowercase);
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && match authority.as_deref() {
            Some(API_HOST) => url.port_or_known_default() == Some(443),
            Some(CDN_HOST) => !has_authorization && url.port_or_known_default() == Some(443),
            _ => false,
        }
}

fn base_client() -> ClientBuilder {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    Client::builder()
        .tls_backend_rustls()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .connect_timeout(CONNECT_TIMEOUT)
        .user_agent(USER_AGENT)
}

fn read_bounded(
    response: &mut impl Read,
    deadline: Instant,
    max_bytes: usize,
) -> Result<Vec<u8>, TransportError> {
    let allocation_limit = max_bytes.checked_add(1).ok_or(TransportError::BodyLimit)?;
    let mut body = Vec::with_capacity(allocation_limit.min(64 * 1024));
    let mut buffer = [0_u8; 8192];
    loop {
        if Instant::now() >= deadline {
            return Err(TransportError::Deadline);
        }
        let remaining = allocation_limit.saturating_sub(body.len());
        if remaining == 0 {
            return Err(TransportError::BodyLimit);
        }
        let read_size = remaining.min(buffer.len());
        let count = response.read(&mut buffer[..read_size]).map_err(|error| {
            if error.kind() == std::io::ErrorKind::TimedOut
                || error.kind() == std::io::ErrorKind::WouldBlock
                || Instant::now() >= deadline
            {
                TransportError::Deadline
            } else {
                TransportError::Unavailable
            }
        })?;
        if count == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..count]);
        if body.len() > max_bytes {
            return Err(TransportError::BodyLimit);
        }
    }
    if Instant::now() >= deadline {
        return Err(TransportError::Deadline);
    }
    Ok(body)
}

fn selected_headers(source: &HeaderMap) -> std::collections::BTreeMap<String, String> {
    ["location", "retry-after", "x-ratelimit-remaining"]
        .iter()
        .filter_map(|name| {
            source
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .map(|value| ((*name).to_owned(), value.to_owned()))
        })
        .collect()
}

fn valid_event(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn safe_workflow_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
}

fn safe_ref_name(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("..")
        && !value.contains("@{")
        && value.bytes().all(|byte| {
            !byte.is_ascii_control()
                && !matches!(byte, b' ' | b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\')
        })
}

fn is_sha1(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn required_env(name: &str) -> Result<String, PublishError> {
    let value = std::env::var(name)
        .map_err(|_| PublishError::InvalidInput("required GitHub context is missing"))?;
    if value.is_empty() || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(PublishError::InvalidInput(
            "required GitHub context is malformed",
        ));
    }
    Ok(value)
}

fn positive_env(name: &str) -> Result<u64, PublishError> {
    required_env(name)?
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(PublishError::InvalidInput(
            "GitHub run ID or attempt is invalid",
        ))
}

fn bounded_file(path: &str, cap: usize) -> Result<Vec<u8>, PublishError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| PublishError::InvalidInput("test TLS root cannot be read"))?;
    if !metadata.file_type().is_file() || metadata.len() > cap as u64 {
        return Err(PublishError::InvalidInput(
            "test TLS root is not a bounded regular file",
        ));
    }
    let bytes = std::fs::read(path)
        .map_err(|_| PublishError::InvalidInput("test TLS root cannot be read"))?;
    if bytes.len() > cap {
        return Err(PublishError::InvalidInput(
            "test TLS root exceeds its byte limit",
        ));
    }
    Ok(bytes)
}

pub fn parse_invocation(bytes: &[u8]) -> Result<PublishInvocation, PublishError> {
    if bytes.is_empty() || bytes.len() > MAX_INPUT_BYTES {
        return Err(PublishError::InvalidInput(
            "request JSON is empty or exceeds its byte limit",
        ));
    }
    serde_json::from_slice(bytes).map_err(|_| PublishError::InvalidInput("request JSON is invalid"))
}

pub fn parse_policy(bytes: &[u8]) -> Result<TrustedPublisherPolicy, PublishError> {
    if bytes.is_empty() || bytes.len() > 64 * 1024 {
        return Err(PublishError::InvalidInput(
            "test policy is empty or exceeds its byte limit",
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|_| PublishError::InvalidInput("test policy JSON is invalid"))
}

pub fn read_policy_from_test_environment() -> Result<TrustedPublisherPolicy, PublishError> {
    #[cfg(debug_assertions)]
    {
        if std::env::var(TEST_MODE_ENV).ok().as_deref() != Some("1") {
            return Err(PublishError::LivePublishingDisabled);
        }
        let bytes = bounded_file(&required_env(TEST_POLICY_ENV)?, 64 * 1024)?;
        return parse_policy(&bytes);
    }
    #[cfg(not(debug_assertions))]
    {
        Err(PublishError::TestModeUnavailable)
    }
}

pub fn read_stdin_bounded() -> Result<Vec<u8>, PublishError> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| PublishError::InvalidInput("cannot read request JSON"))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(PublishError::InvalidInput(
            "request JSON exceeds its byte limit",
        ));
    }
    Ok(bytes)
}

pub fn test_mode_enabled() -> bool {
    std::env::var(TEST_MODE_ENV).ok().as_deref() == Some("1")
}

pub fn run_context_from_environment() -> Result<GitHubContext, PublishError> {
    GitHubContext::from_environment()
}

pub fn token_from_environment() -> Result<String, PublishError> {
    required_env("GITHUB_TOKEN")
}
