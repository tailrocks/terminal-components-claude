//! Bounded, GET-only GitHub REST API client primitives.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;

pub const API_ORIGIN: &str = "https://api.github.com";
pub const MAX_JSON_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_WORKFLOW_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_ARTIFACT_DOWNLOAD_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_PAGES: usize = 20;
pub const PAGE_SIZE: usize = 100;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_REDIRECTS: usize = 3;

/// The only transport operation exposed to this crate is a bounded GET.
/// Implementations must not follow redirects automatically, must enforce the
/// supplied deadline while reading the body, and must stop after `max_bytes`.
pub trait ReadOnlyTransport: Send + Sync {
    fn get(
        &self,
        request: HttpRequest,
        deadline: Instant,
        max_bytes: usize,
    ) -> Result<HttpResponse, TransportError>;
}

#[derive(Clone)]
pub struct HttpRequest {
    url: String,
    authorization: Option<String>,
}

impl HttpRequest {
    pub fn url(&self) -> &str { &self.url }
    pub fn authorization(&self) -> Option<&str> { self.authorization.as_deref() }
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &"GET")
            .field("url", &redact_query(&self.url))
            .field("authorization", &self.authorization.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    /// Header names are compared without case sensitivity.
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self { status, headers: BTreeMap::new(), body: body.into() }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|(key, value)| {
            key.eq_ignore_ascii_case(name).then_some(value.as_str())
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportError {
    Deadline,
    BodyLimit,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    InvalidInput(&'static str),
    Deadline,
    Transport(TransportError),
    BodyLimit,
    RateLimited { retry_after_seconds: Option<u64> },
    Unauthorized,
    Forbidden,
    NotFound,
    HttpStatus(u16),
    InvalidResponse,
    IncompletePagination,
    PaginationLimit,
    DuplicatePageItem,
    UnsafeRedirect,
    RedirectLimit,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid API input: {message}"),
            Self::Deadline => f.write_str("GitHub API request deadline exceeded"),
            Self::Transport(TransportError::Deadline) => f.write_str("GitHub API transport deadline exceeded"),
            Self::Transport(TransportError::BodyLimit) | Self::BodyLimit => f.write_str("GitHub API body exceeded its byte limit"),
            Self::Transport(TransportError::Unavailable) => f.write_str("GitHub API transport failed"),
            Self::RateLimited { .. } => f.write_str("GitHub API rate limit reached"),
            Self::Unauthorized => f.write_str("GitHub API rejected authentication"),
            Self::Forbidden => f.write_str("GitHub API denied the read request"),
            Self::NotFound => f.write_str("GitHub API resource was not found"),
            Self::HttpStatus(status) => write!(f, "GitHub API returned HTTP {status}"),
            Self::InvalidResponse => f.write_str("GitHub API response was invalid"),
            Self::IncompletePagination => f.write_str("GitHub API pagination was incomplete or changed during the read"),
            Self::PaginationLimit => f.write_str("GitHub API pagination exceeded its configured bound"),
            Self::DuplicatePageItem => f.write_str("GitHub API pagination repeated an item"),
            Self::UnsafeRedirect => f.write_str("GitHub API returned an unsafe redirect"),
            Self::RedirectLimit => f.write_str("GitHub API redirect limit exceeded"),
        }
    }
}

impl std::error::Error for ApiError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    owner: String,
    name: String,
}

impl Repository {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Result<Self, ApiError> {
        let repository = Self { owner: owner.into(), name: name.into() };
        if !valid_path_segment(&repository.owner) || !valid_path_segment(&repository.name) {
            return Err(ApiError::InvalidInput("repository owner and name must be safe path segments"));
        }
        Ok(repository)
    }

    pub fn full_name(&self) -> String { format!("{}/{}", self.owner, self.name) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSelector {
    pub repository: Repository,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub head_sha: String,
    pub branch: String,
    pub event: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRecord {
    pub id: u64,
    pub run_number: u64,
    pub run_attempt: u32,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub repository: String,
    pub head_sha: String,
    pub head_branch: String,
    pub event: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobRecord {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactRunAssociation {
    pub id: u64,
    pub head_branch: String,
    pub head_sha: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactRecord {
    pub id: u64,
    pub name: String,
    pub size_in_bytes: u64,
    pub expired: bool,
    pub digest: Option<String>,
    pub workflow_run: Option<ArtifactRunAssociation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetResolution {
    Missing,
    Ambiguous,
    Found(RunRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowFile {
    pub path: String,
    pub source_commit: String,
    pub git_blob_sha: String,
    pub sha256: String,
    pub bytes: Vec<u8>,
}

pub struct ApiClient<T> {
    transport: T,
    token: String,
    timeout: Duration,
}

impl<T: ReadOnlyTransport> ApiClient<T> {
    pub fn new(transport: T, token: impl Into<String>) -> Result<Self, ApiError> {
        let token = token.into();
        if token.is_empty() || token.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(ApiError::InvalidInput("bearer token is empty or contains whitespace"));
        }
        Ok(Self { transport, token, timeout: REQUEST_TIMEOUT })
    }

    pub fn transport(&self) -> &T { &self.transport }

    pub fn get_run(&self, repository: &Repository, run_id: u64) -> Result<RunRecord, ApiError> {
        if run_id == 0 { return Err(ApiError::InvalidInput("run ID must be positive")); }
        let url = format!("{}/repos/{}/actions/runs/{run_id}", API_ORIGIN, repository.full_name());
        let run: ApiRun = self.get_json(&url, MAX_JSON_RESPONSE_BYTES)?;
        Ok(run.into_record())
    }

    pub fn resolve_target(&self, selector: &RunSelector) -> Result<TargetResolution, ApiError> {
        if selector.workflow_id == 0 || !is_sha1(&selector.head_sha)
            || !safe_repository_path(&selector.workflow_path)
            || selector.branch.is_empty() || selector.event.is_empty()
        {
            return Err(ApiError::InvalidInput("target selector is incomplete"));
        }
        let runs = self.list_runs(selector)?;
        let mut matches: Vec<RunRecord> = runs.into_iter().map(ApiRun::into_record).filter(|run| {
            run.workflow_id == selector.workflow_id
                && run.workflow_path == selector.workflow_path
                && run.repository == selector.repository.full_name()
                && run.head_sha == selector.head_sha
                && run.head_branch == selector.branch
                && run.event == selector.event
        }).collect();
        if matches.is_empty() { return Ok(TargetResolution::Missing); }
        let newest = matches.iter().map(|run| (run.run_number, run.run_attempt)).max().unwrap_or((0, 0));
        matches.retain(|run| (run.run_number, run.run_attempt) == newest);
        if matches.len() != 1 { return Ok(TargetResolution::Ambiguous); }
        Ok(TargetResolution::Found(matches.remove(0)))
    }

    pub fn list_jobs(
        &self,
        repository: &Repository,
        run_id: u64,
        attempt: u32,
    ) -> Result<Vec<JobRecord>, ApiError> {
        if run_id == 0 || attempt == 0 { return Err(ApiError::InvalidInput("run ID and attempt must be positive")); }
        let path = format!("/repos/{}/actions/runs/{run_id}/attempts/{attempt}/jobs", repository.full_name());
        self.collect_pages(&path, "jobs", |item: &ApiJob| item.id)
            .map(|items: Vec<ApiJob>| items.into_iter().map(ApiJob::into_record).collect())
    }

    pub fn list_artifacts(&self, repository: &Repository, run_id: u64) -> Result<Vec<ArtifactRecord>, ApiError> {
        if run_id == 0 { return Err(ApiError::InvalidInput("run ID must be positive")); }
        let path = format!("/repos/{}/actions/runs/{run_id}/artifacts", repository.full_name());
        self.collect_pages(&path, "artifacts", |item: &ApiArtifact| item.id)
            .map(|items: Vec<ApiArtifact>| items.into_iter().map(ApiArtifact::into_record).collect())
    }

    pub fn download_artifact(
        &self,
        repository: &Repository,
        artifact_id: u64,
        max_bytes: usize,
    ) -> Result<Vec<u8>, ApiError> {
        if artifact_id == 0 || max_bytes == 0 || max_bytes > MAX_ARTIFACT_DOWNLOAD_BYTES {
            return Err(ApiError::InvalidInput("artifact ID and bounded byte cap must be positive"));
        }
        let url = format!("{}/repos/{}/actions/artifacts/{artifact_id}/zip", API_ORIGIN, repository.full_name());
        self.get_bytes(&url, max_bytes)
    }

    /// Read a workflow file through the Contents API at the exact commit SHA.
    /// The bytes are returned as data and are never checked out or executed.
    pub fn get_workflow_file(
        &self,
        repository: &Repository,
        path: &str,
        commit_sha: &str,
    ) -> Result<WorkflowFile, ApiError> {
        if !safe_repository_path(path) || !is_sha1(commit_sha) {
            return Err(ApiError::InvalidInput("workflow path or source commit is invalid"));
        }
        let encoded_path = path.split('/').map(percent_encode_path_segment).collect::<Vec<_>>().join("/");
        let url = format!("{}/repos/{}/contents/{encoded_path}?ref={commit_sha}", API_ORIGIN, repository.full_name());
        let response: ContentsResponse = self.get_json(&url, MAX_JSON_RESPONSE_BYTES)?;
        if response.path != path || response.kind != "file" || response.encoding != "base64"
            || response.size > MAX_WORKFLOW_FILE_BYTES || !is_sha1(&response.sha)
        {
            return Err(ApiError::InvalidResponse);
        }
        let compact: String = response.content.chars().filter(|character| !character.is_ascii_whitespace()).collect();
        let bytes = STANDARD.decode(compact.as_bytes()).map_err(|_| ApiError::InvalidResponse)?;
        if bytes.len() != response.size { return Err(ApiError::InvalidResponse); }
        let sha256 = sha256_hex(&bytes);
        Ok(WorkflowFile {
            path: response.path,
            source_commit: commit_sha.to_owned(),
            git_blob_sha: response.sha,
            sha256,
            bytes,
        })
    }

    fn list_runs(&self, selector: &RunSelector) -> Result<Vec<ApiRun>, ApiError> {
        let path = format!("/repos/{}/actions/workflows/{}/runs", selector.repository.full_name(), selector.workflow_id);
        let mut output = Vec::new();
        let mut seen = HashSet::new();
        let mut expected_total = None;
        for page in 1..=MAX_PAGES {
            let url = format!("{}{}?head_sha={}&branch={}&event={}&per_page={PAGE_SIZE}&page={page}", API_ORIGIN, path,
                percent_encode_query(&selector.head_sha), percent_encode_query(&selector.branch), percent_encode_query(&selector.event));
            let response: RunPage = self.get_json(&url, MAX_JSON_RESPONSE_BYTES)?;
            let total = usize::try_from(response.total_count).map_err(|_| ApiError::PaginationLimit)?;
            if total > MAX_PAGES * PAGE_SIZE { return Err(ApiError::PaginationLimit); }
            if expected_total.replace(total).is_some_and(|previous| previous != total) {
                return Err(ApiError::IncompletePagination);
            }
            let pages = total.max(1).div_ceil(PAGE_SIZE);
            if pages > MAX_PAGES { return Err(ApiError::PaginationLimit); }
            let expected_count = total.saturating_sub((page - 1) * PAGE_SIZE).min(PAGE_SIZE);
            if response.workflow_runs.len() != expected_count { return Err(ApiError::IncompletePagination); }
            for run in response.workflow_runs {
                if run.id == 0 || run.run_number == 0 || run.run_attempt == 0 || run.workflow_id == 0 {
                    return Err(ApiError::InvalidResponse);
                }
                if !seen.insert(run.id) { return Err(ApiError::DuplicatePageItem); }
                output.push(run);
            }
            if page >= pages { return Ok(output); }
        }
        Err(ApiError::PaginationLimit)
    }

    fn collect_pages<R, F>(&self, path: &str, collection: &str, id: F) -> Result<Vec<R>, ApiError>
    where
        R: for<'de> Deserialize<'de>,
        F: Fn(&R) -> u64,
    {
        let mut output = Vec::new();
        let mut seen = HashSet::new();
        let mut expected_total = None;
        for page in 1..=MAX_PAGES {
            let url = format!("{API_ORIGIN}{path}?per_page={PAGE_SIZE}&page={page}");
            let response: PageEnvelope<R> = self.get_json(&url, MAX_JSON_RESPONSE_BYTES)?;
            let total = usize::try_from(response.total_count).map_err(|_| ApiError::PaginationLimit)?;
            if total > MAX_PAGES * PAGE_SIZE { return Err(ApiError::PaginationLimit); }
            if expected_total.replace(total).is_some_and(|previous| previous != total) {
                return Err(ApiError::IncompletePagination);
            }
            let pages = total.max(1).div_ceil(PAGE_SIZE);
            if pages > MAX_PAGES { return Err(ApiError::PaginationLimit); }
            let expected_count = total.saturating_sub((page - 1) * PAGE_SIZE).min(PAGE_SIZE);
            let items = response.into_items(collection).ok_or(ApiError::InvalidResponse)?;
            if items.len() != expected_count { return Err(ApiError::IncompletePagination); }
            for item in items {
                let item_id = id(&item);
                if item_id == 0 { return Err(ApiError::InvalidResponse); }
                if !seen.insert(item_id) { return Err(ApiError::DuplicatePageItem); }
                output.push(item);
            }
            if page >= pages { return Ok(output); }
        }
        Err(ApiError::PaginationLimit)
    }

    fn get_json<R: for<'de> Deserialize<'de>>(&self, url: &str, max_bytes: usize) -> Result<R, ApiError> {
        let bytes = self.get_bytes(url, max_bytes)?;
        serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidResponse)
    }

    fn get_bytes(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, ApiError> {
        if !is_safe_https_url(url) { return Err(ApiError::UnsafeRedirect); }
        let deadline = Instant::now() + self.timeout;
        let mut current = url.to_owned();
        for redirect_count in 0..=MAX_REDIRECTS {
            if Instant::now() >= deadline { return Err(ApiError::Deadline); }
            let auth = same_origin(&current, API_ORIGIN).then(|| self.token.clone());
            let request = HttpRequest { url: current.clone(), authorization: auth };
            let response = self.transport.get(request, deadline, max_bytes).map_err(map_transport)?;
            if Instant::now() >= deadline { return Err(ApiError::Deadline); }
            if response.body.len() > max_bytes { return Err(ApiError::BodyLimit); }
            if (300..400).contains(&response.status) {
                if redirect_count == MAX_REDIRECTS { return Err(ApiError::RedirectLimit); }
                let location = response.header("location").ok_or(ApiError::UnsafeRedirect)?;
                current = resolve_redirect(&current, location)?;
                continue;
            }
            if !(200..300).contains(&response.status) { return Err(classify_http_error(&response)); }
            return Ok(response.body);
        }
        Err(ApiError::RedirectLimit)
    }
}

#[derive(Deserialize)]
struct ApiRepository { full_name: String }

#[derive(Deserialize)]
struct ApiRun {
    id: u64,
    run_number: u64,
    run_attempt: u32,
    workflow_id: u64,
    path: String,
    event: String,
    status: String,
    conclusion: Option<String>,
    head_sha: String,
    head_branch: String,
    repository: ApiRepository,
}

impl ApiRun {
    fn into_record(self) -> RunRecord {
        let workflow_path = self.path.split_once('@').map_or(self.path.as_str(), |(path, _)| path).to_owned();
        RunRecord {
            id: self.id,
            run_number: self.run_number,
            run_attempt: self.run_attempt,
            workflow_id: self.workflow_id,
            workflow_path,
            repository: self.repository.full_name,
            head_sha: self.head_sha,
            head_branch: self.head_branch,
            event: self.event,
            status: self.status,
            conclusion: self.conclusion,
        }
    }
}

#[derive(Deserialize)]
struct RunPage { total_count: u64, workflow_runs: Vec<ApiRun> }

#[derive(Deserialize)]
struct ApiJob { id: u64, name: String, status: String, conclusion: Option<String> }

impl ApiJob {
    fn into_record(self) -> JobRecord { JobRecord { id: self.id, name: self.name, status: self.status, conclusion: self.conclusion } }
}

#[derive(Deserialize)]
struct ApiArtifact {
    id: u64,
    name: String,
    size_in_bytes: u64,
    expired: bool,
    digest: Option<String>,
    workflow_run: Option<ApiArtifactRun>,
}

impl ApiArtifact {
    fn into_record(self) -> ArtifactRecord {
        ArtifactRecord {
            id: self.id,
            name: self.name,
            size_in_bytes: self.size_in_bytes,
            expired: self.expired,
            digest: self.digest,
            workflow_run: self.workflow_run.map(|run| ArtifactRunAssociation {
                id: run.id, head_branch: run.head_branch, head_sha: run.head_sha,
            }),
        }
    }
}

#[derive(Deserialize)]
struct ApiArtifactRun { id: u64, head_branch: String, head_sha: String }

// PageEnvelope is specialized through a small Value envelope so unexpected
// collection names fail closed without accepting arbitrary response shapes.
#[derive(Deserialize)]
struct PageEnvelope<T> {
    total_count: u64,
    #[serde(flatten)] collections: BTreeMap<String, serde_json::Value>,
    #[serde(skip)] marker: std::marker::PhantomData<T>,
}

impl<T: for<'de> Deserialize<'de>> PageEnvelope<T> {
    fn into_items(self, key: &str) -> Option<Vec<T>> {
        self.collections.get(key).cloned().and_then(|value| serde_json::from_value(value).ok())
    }
}

#[derive(Deserialize)]
struct ContentsResponse { path: String, #[serde(rename = "type")] kind: String, encoding: String, size: usize, sha: String, content: String }

fn map_transport(error: TransportError) -> ApiError {
    match error {
        TransportError::Deadline => ApiError::Deadline,
        TransportError::BodyLimit => ApiError::BodyLimit,
        TransportError::Unavailable => ApiError::Transport(error),
    }
}

fn classify_http_error(response: &HttpResponse) -> ApiError {
    match response.status {
        401 => ApiError::Unauthorized,
        403 if response.header("x-ratelimit-remaining") == Some("0") => ApiError::RateLimited {
            retry_after_seconds: response.header("retry-after").and_then(|value| value.parse().ok()),
        },
        429 => ApiError::RateLimited {
            retry_after_seconds: response.header("retry-after").and_then(|value| value.parse().ok()),
        },
        403 => ApiError::Forbidden,
        404 => ApiError::NotFound,
        status => ApiError::HttpStatus(status),
    }
}

fn is_sha1(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_repository_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains('\\')
        && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != ".."
            && segment.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)))
}

fn valid_path_segment(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

fn percent_encode_path_segment(value: &str) -> String {
    percent_encode(value, true)
}

fn percent_encode_query(value: &str) -> String {
    percent_encode(value, false)
}

fn percent_encode(value: &str, slash_safe: bool) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) || (slash_safe && byte == b'/') {
            output.push(byte as char);
        } else {
            use std::fmt::Write;
            let _ = write!(output, "%{byte:02X}");
        }
    }
    output
}

fn is_safe_https_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else { return false; };
    if rest.is_empty() || rest.contains(['\\', '\r', '\n', '#']) { return false; }
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    !authority.is_empty() && !authority.contains('@') && !authority.starts_with('.')
        && authority.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".-:".contains(&byte))
}

fn same_origin(left: &str, right: &str) -> bool {
    origin(left).zip(origin(right)).is_some_and(|(left, right)| left.eq_ignore_ascii_case(&right))
}

fn origin(url: &str) -> Option<String> {
    if !is_safe_https_url(url) { return None; }
    let rest = url.strip_prefix("https://")?;
    Some(format!("https://{}", rest.split(['/', '?']).next()?))
}

fn resolve_redirect(current: &str, location: &str) -> Result<String, ApiError> {
    let trimmed = location.trim();
    let resolved = if trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else if trimmed.starts_with('/') && !trimmed.starts_with("//") {
        format!("{}{}", origin(current).ok_or(ApiError::UnsafeRedirect)?, trimmed)
    } else {
        return Err(ApiError::UnsafeRedirect);
    };
    if !is_safe_https_url(&resolved) { return Err(ApiError::UnsafeRedirect); }
    Ok(resolved)
}

fn redact_query(url: &str) -> String {
    url.split_once('?').map_or(url.to_owned(), |(base, _)| format!("{base}?[redacted]"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
