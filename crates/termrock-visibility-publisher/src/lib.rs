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

/// Concrete HTTPS transport for the existing bounded GET-only API client.
pub mod transport {
    use std::io::Read;
    use std::net::SocketAddr;
    use std::time::Instant;

    use reqwest::blocking::{Client, ClientBuilder};
    use reqwest::header::{
        ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue,
    };

    use crate::api::{
        HttpRequest, HttpResponse, MAX_ARTIFACT_DOWNLOAD_BYTES, ReadOnlyTransport,
        TransportError,
    };

    const USER_AGENT_VALUE: &str = "termrock-visibility-publisher";
    const RESPONSE_HEADERS: [&str; 3] = ["location", "retry-after", "x-ratelimit-remaining"];

    #[cfg(test)]
    #[derive(Clone, Debug, Default)]
    struct TestDiagnostic {
        wire: Option<TestWireUrl>,
        reqwest_error: Option<TestReqwestError>,
    }

    #[cfg(test)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestWireUrl {
        scheme: String,
        host: Option<String>,
        explicit_port: Option<u16>,
        effective_port: Option<u16>,
    }

    #[cfg(test)]
    #[derive(Clone, Debug, Default)]
    struct TestReqwestError {
        timeout: bool,
        connect: bool,
        dns: bool,
        request: bool,
        body: bool,
        decode: bool,
        redirect: bool,
        status: bool,
        builder: bool,
        source_kinds: Vec<&'static str>,
    }

    #[cfg(test)]
    impl TestWireUrl {
        fn from_url(url: &reqwest::Url) -> Self {
            Self {
                scheme: url.scheme().to_owned(),
                host: url.host_str().map(str::to_ascii_lowercase),
                explicit_port: url.port(),
                effective_port: url.port_or_known_default(),
            }
        }
    }

    #[cfg(test)]
    impl TestReqwestError {
        fn from_error(error: &reqwest::Error) -> Self {
            use std::error::Error as _;

            let mut source_kinds = Vec::new();
            let mut source = error.source();
            while let Some(current) = source {
                source_kinds.push(test_error_source_kind(current));
                source = current.source();
            }
            Self {
                timeout: error.is_timeout(),
                connect: error.is_connect(),
                dns: error.is_dns(),
                request: error.is_request(),
                body: error.is_body(),
                decode: error.is_decode(),
                redirect: error.is_redirect(),
                status: error.is_status(),
                builder: error.is_builder(),
                source_kinds,
            }
        }
    }

    #[cfg(test)]
    fn test_error_source_kind(error: &(dyn std::error::Error + 'static)) -> &'static str {
        if let Some(error) = error.downcast_ref::<std::io::Error>() {
            return match error.kind() {
                std::io::ErrorKind::ConnectionRefused => "io:connection-refused",
                std::io::ErrorKind::ConnectionReset => "io:connection-reset",
                std::io::ErrorKind::TimedOut => "io:timed-out",
                std::io::ErrorKind::UnexpectedEof => "io:unexpected-eof",
                std::io::ErrorKind::InvalidData => "io:invalid-data",
                _ => "io:other",
            };
        }
        if let Some(error) = error.downcast_ref::<rustls::Error>() {
            return match error {
                rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer) => "rustls:unknown-issuer",
                rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName) => "rustls:name-mismatch",
                rustls::Error::NoCertificatesPresented => "rustls:no-certificate",
                rustls::Error::AlertReceived(_) => "rustls:alert",
                rustls::Error::InvalidMessage(_) => "rustls:invalid-message",
                rustls::Error::General(_) => "rustls:general",
                _ => "rustls:other",
            };
        }
        "other"
    }

    /// A reusable reqwest client that only issues bounded HTTPS GET requests.
    #[derive(Clone)]
    pub struct ReqwestTransport {
        client: Client,
        #[cfg(test)]
        loopback_port: Option<u16>,
        #[cfg(test)]
        diagnostics: std::sync::Arc<std::sync::Mutex<TestDiagnostic>>,
    }

    impl ReqwestTransport {
        pub fn new() -> Result<Self, TransportError> {
            Ok(Self {
                client: build_client(None, &[])?,
                #[cfg(test)]
                loopback_port: None,
                #[cfg(test)]
                diagnostics: std::sync::Arc::default(),
            })
        }

        #[cfg(test)]
        fn with_client(client: Client) -> Self {
            Self {
                client,
                loopback_port: None,
                diagnostics: std::sync::Arc::default(),
            }
        }

        #[cfg(test)]
        fn with_loopback_client(client: Client, port: u16) -> Self {
            Self {
                client,
                loopback_port: Some(port),
                diagnostics: std::sync::Arc::default(),
            }
        }

        #[cfg(test)]
        fn test_diagnostic(&self) -> TestDiagnostic {
            self.diagnostics
                .lock()
                .map(|diagnostic| diagnostic.clone())
                .unwrap_or_default()
        }

        #[cfg(test)]
        fn record_test_wire_url(&self, url: &reqwest::Url) {
            if let Ok(mut diagnostic) = self.diagnostics.lock() {
                diagnostic.wire = Some(TestWireUrl::from_url(url));
                diagnostic.reqwest_error = None;
            }
        }

        #[cfg(test)]
        fn record_test_reqwest_error(&self, error: &reqwest::Error) {
            if let Ok(mut diagnostic) = self.diagnostics.lock() {
                diagnostic.reqwest_error = Some(TestReqwestError::from_error(error));
            }
        }

        fn execute_get(
            &self,
            url: &str,
            authorization: Option<&str>,
            deadline: Instant,
            max_bytes: usize,
        ) -> Result<HttpResponse, TransportError> {
            if max_bytes > MAX_ARTIFACT_DOWNLOAD_BYTES {
                return Err(TransportError::BodyLimit);
            }

            let parsed = reqwest::Url::parse(url).map_err(|_| TransportError::Unavailable)?;
            if parsed.scheme() != "https"
                || parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.fragment().is_some()
            {
                return Err(TransportError::Unavailable);
            }
            if authorization.is_some() && !is_api_origin(&parsed) {
                return Err(TransportError::Unavailable);
            }

            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|duration| !duration.is_zero())
                .ok_or(TransportError::Deadline)?;

            // Validate scheme, origin, and bearer authority against the
            // caller's original URL above. Tests may route only the two
            // HTTPS GitHub hosts to an ephemeral loopback listener after
            // those checks; production builds have no routing override.
            #[cfg(test)]
            let parsed = self.test_wire_url(parsed)?;
            // Record only the URL authority immediately before passing this
            // exact parsed value to Reqwest. Never retain a path or query.
            #[cfg(test)]
            self.record_test_wire_url(&parsed);
            let mut request = self.client.get(parsed);
            if let Some(token) = authorization {
                request = request
                    .header(AUTHORIZATION, sensitive_bearer(token)?)
                    .header(ACCEPT, "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2022-11-28");
            }

            let mut response = request
                .timeout(remaining)
                .send()
                .map_err(|error| {
                    #[cfg(test)]
                    self.record_test_reqwest_error(&error);
                    map_reqwest_error(&error, deadline)
                })?;
            if Instant::now() >= deadline {
                return Err(TransportError::Deadline);
            }

            let cap_as_u64 = u64::try_from(max_bytes).unwrap_or(u64::MAX);
            if response.content_length().is_some_and(|length| length > cap_as_u64) {
                return Err(TransportError::BodyLimit);
            }

            let status = response.status().as_u16();
            let headers = selected_headers(response.headers());
            let body = read_bounded(&mut response, deadline, max_bytes)?;
            Ok(HttpResponse { status, headers, body })
        }

        #[cfg(test)]
        fn test_wire_url(&self, mut url: reqwest::Url) -> Result<reqwest::Url, TransportError> {
            let Some(port) = self.loopback_port else {
                return Ok(url);
            };
            let is_test_host = url.host_str().is_some_and(|host| {
                host.eq_ignore_ascii_case("api.github.com")
                    || host.eq_ignore_ascii_case("objects.githubusercontent.com")
            });
            if is_test_host && url.port_or_known_default() == Some(443) {
                url.set_port(Some(port)).map_err(|_| TransportError::Unavailable)?;
            }
            Ok(url)
        }
    }

    impl ReadOnlyTransport for ReqwestTransport {
        fn get(
            &self,
            request: HttpRequest,
            deadline: Instant,
            max_bytes: usize,
        ) -> Result<HttpResponse, TransportError> {
            self.execute_get(request.url(), request.authorization(), deadline, max_bytes)
        }
    }

    fn build_client(
        test_roots: Option<Vec<reqwest::Certificate>>,
        dns_overrides: &[(String, SocketAddr)],
    ) -> Result<Client, TransportError> {
        ensure_default_provider()?;

        let mut builder: ClientBuilder = Client::builder()
            .tls_backend_rustls()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .user_agent(USER_AGENT_VALUE);

        if let Some(certificates) = test_roots {
            builder = builder.tls_certs_only(certificates);
        }
        for (host, address) in dns_overrides {
            builder = builder.resolve(host, *address);
        }

        builder.build().map_err(|_| TransportError::Unavailable)
    }

    /// Reqwest's `rustls-no-provider` mode needs a process default. Keep an
    /// existing host choice; install Ring only when no default exists, and
    /// accept another thread's successful one-time installation.
    fn ensure_default_provider() -> Result<(), TransportError> {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        rustls::crypto::CryptoProvider::get_default()
            .map(|_| ())
            .ok_or(TransportError::Unavailable)
    }

    fn is_api_origin(url: &reqwest::Url) -> bool {
        url.scheme() == "https"
            && url.host_str().is_some_and(|host| host.eq_ignore_ascii_case("api.github.com"))
            && url.port_or_known_default() == Some(443)
    }

    fn sensitive_bearer(token: &str) -> Result<HeaderValue, TransportError> {
        let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| TransportError::Unavailable)?;
        value.set_sensitive(true);
        Ok(value)
    }

    fn selected_headers(source: &HeaderMap) -> std::collections::BTreeMap<String, String> {
        RESPONSE_HEADERS
            .iter()
            .filter_map(|name| {
                source
                    .get(*name)
                    .and_then(|value| value.to_str().ok())
                    .map(|value| ((*name).to_owned(), value.to_owned()))
            })
            .collect()
    }

    fn map_reqwest_error(error: &reqwest::Error, deadline: Instant) -> TransportError {
        if error.is_timeout() || Instant::now() >= deadline {
            TransportError::Deadline
        } else {
            TransportError::Unavailable
        }
    }

    fn read_bounded(
        response: &mut impl Read,
        deadline: Instant,
        max_bytes: usize,
    ) -> Result<Vec<u8>, TransportError> {
        let allocation_limit = max_bytes
            .checked_add(1)
            .ok_or(TransportError::BodyLimit)?;
        let mut body = Vec::with_capacity(allocation_limit);
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
            let count = match response.read(&mut buffer[..read_size]) {
                Ok(count) => count,
                Err(error)
                    if error.kind() == std::io::ErrorKind::Interrupted =>
                {
                    continue;
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::TimedOut
                        || error.kind() == std::io::ErrorKind::WouldBlock
                        || Instant::now() >= deadline =>
                {
                    return Err(TransportError::Deadline);
                }
                Err(_) => return Err(TransportError::Unavailable),
            };
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

    #[cfg(test)]
    mod tests {
        use std::collections::BTreeMap;
        use std::io::{Read, Write};
        use std::net::{SocketAddr, TcpListener, TcpStream};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{Arc, Mutex, mpsc};
        use std::thread::{self, JoinHandle};
        use std::time::{Duration, Instant};

        use rcgen::{
            BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer,
            KeyPair, KeyUsagePurpose,
        };
        use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
        use rustls::{ServerConfig, ServerConnection, StreamOwned};

        use crate::api::{ApiClient, ApiError, Repository, ReadOnlyTransport, TransportError};

        use super::{ReqwestTransport, build_client, ensure_default_provider, sensitive_bearer};

        const TOKEN: &str = "test-token-never-a-real-secret";
        const API_HOST: &str = "api.github.com";
        const CDN_HOST: &str = "objects.githubusercontent.com";
        const MAX_TEST_HEADER_BYTES: usize = 16 * 1024;

        struct TlsMaterial {
            root_der: Vec<u8>,
            chain: Vec<CertificateDer<'static>>,
            key: PrivateKeyDer<'static>,
        }

        fn tls_material(server_names: &[&str]) -> TlsMaterial {
            ensure_default_provider().expect("Rustls provider should be available");

            let mut root_params = CertificateParams::new(vec!["termrock-test-ca.invalid".to_owned()])
                .expect("static CA name is valid");
            root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            root_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
            let root_key = KeyPair::generate().expect("Ring can generate a test CA key");
            let root_cert = root_params
                .self_signed(&root_key)
                .expect("test CA can be self-signed");
            let issuer = Issuer::new(root_params, root_key);

            let mut leaf_params = CertificateParams::new(
                server_names
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect::<Vec<_>>(),
            )
            .expect("static server names are valid");
            leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
            leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
            let leaf_key = KeyPair::generate().expect("Ring can generate a test server key");
            let leaf_cert = leaf_params
                .signed_by(&leaf_key, &issuer)
                .expect("test server certificate can be signed");

            TlsMaterial {
                root_der: root_cert.der().as_ref().to_vec(),
                chain: vec![leaf_cert.der().clone()],
                key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
            }
        }

        #[derive(Clone, Copy)]
        enum BodyWire {
            Fixed,
            Chunked,
            Stall { prefix_bytes: usize, duration: Duration },
        }

        struct ResponsePlan {
            status: u16,
            headers: Vec<(String, String)>,
            body: Vec<u8>,
            wire: BodyWire,
        }

        impl ResponsePlan {
            fn fixed(status: u16, body: impl Into<Vec<u8>>) -> Self {
                Self { status, headers: Vec::new(), body: body.into(), wire: BodyWire::Fixed }
            }

            fn header(mut self, name: &str, value: &str) -> Self {
                self.headers.push((name.to_owned(), value.to_owned()));
                self
            }

            fn chunked(mut self) -> Self {
                self.wire = BodyWire::Chunked;
                self
            }

            fn stall(mut self, prefix_bytes: usize, duration: Duration) -> Self {
                self.wire = BodyWire::Stall { prefix_bytes, duration };
                self
            }
        }

        struct CapturedRequest {
            line: String,
            headers: BTreeMap<String, String>,
        }

        impl CapturedRequest {
            fn header(&self, name: &str) -> Option<&str> {
                self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
            }

            fn path(&self) -> Option<&str> {
                self.line.split_whitespace().nth(1)
            }
        }

        struct TestServer {
            address: SocketAddr,
            root_der: Vec<u8>,
            stopped: Arc<AtomicBool>,
            accepted_connections: Arc<std::sync::atomic::AtomicUsize>,
            read_error_kind: Arc<Mutex<Option<std::io::ErrorKind>>>,
            requests: mpsc::Receiver<Option<CapturedRequest>>,
            worker: Option<JoinHandle<()>>,
        }

        impl TestServer {
            fn start(responses: Vec<ResponsePlan>, server_names: &[&str]) -> Self {
                let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback bind succeeds");
                listener.set_nonblocking(true).expect("loopback listener can be nonblocking");
                let address = listener.local_addr().expect("loopback listener has an address");
                let material = tls_material(server_names);
                let config = Arc::new(
                    ServerConfig::builder()
                        .with_no_client_auth()
                        .with_single_cert(material.chain, material.key)
                        .expect("test server certificate is valid"),
                );
                let stopped = Arc::new(AtomicBool::new(false));
                let worker_stopped = Arc::clone(&stopped);
                let accepted_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                let worker_accepted_connections = Arc::clone(&accepted_connections);
                let read_error_kind = Arc::new(Mutex::new(None));
                let worker_read_error_kind = Arc::clone(&read_error_kind);
                let (sender, requests) = mpsc::channel();
                let worker = thread::Builder::new()
                    .name("visibility-loopback-tls".to_owned())
                    .spawn(move || {
                        serve(
                            listener,
                            config,
                            responses,
                            worker_stopped,
                            sender,
                            worker_accepted_connections,
                            worker_read_error_kind,
                        );
                    })
                    .expect("test server thread starts");
                Self {
                    address,
                    root_der: material.root_der,
                    stopped,
                    accepted_connections,
                    read_error_kind,
                    requests,
                    worker: Some(worker),
                }
            }

            fn client(&self) -> ReqwestTransport {
                let root = reqwest::Certificate::from_der(&self.root_der)
                    .expect("generated test CA is valid DER");
                let dns = vec![
                    (API_HOST.to_owned(), self.address),
                    (CDN_HOST.to_owned(), self.address),
                ];
                let client = build_client(Some(vec![root]), &dns)
                    .expect("loopback client builds with only the generated CA");
                ReqwestTransport::with_loopback_client(client, self.address.port())
            }

            fn next_request(&self) -> Option<CapturedRequest> {
                self.requests
                    .recv_timeout(Duration::from_secs(3))
                    .expect("loopback server produced a request result")
            }

            fn diagnostic(&self) -> (usize, Option<std::io::ErrorKind>) {
                (
                    self.accepted_connections.load(Ordering::Acquire),
                    self.read_error_kind.lock().ok().and_then(|kind| *kind),
                )
            }

            fn finish(&mut self) {
                self.stopped.store(true, Ordering::Release);
                if let Some(worker) = self.worker.take() {
                    assert!(worker.join().is_ok(), "loopback server worker completed cleanly");
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
            requests: mpsc::Sender<Option<CapturedRequest>>,
            accepted_connections: Arc<std::sync::atomic::AtomicUsize>,
            read_error_kind: Arc<Mutex<Option<std::io::ErrorKind>>>,
        ) {
            for response in responses {
                let accept_deadline = Instant::now() + Duration::from_secs(3);
                let (stream, _) = loop {
                    if stopped.load(Ordering::Acquire) || Instant::now() >= accept_deadline {
                        return;
                    }
                    match listener.accept() {
                        Ok(connection) => {
                            accepted_connections.fetch_add(1, Ordering::AcqRel);
                            break connection;
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                        }
                        Err(_) => return,
                    }
                };
                if handle_connection(
                    stream,
                    Arc::clone(&config),
                    response,
                    &requests,
                    &read_error_kind,
                )
                .is_err()
                {
                    return;
                }
            }
        }

        fn handle_connection(
            stream: TcpStream,
            config: Arc<ServerConfig>,
            response: ResponsePlan,
            requests: &mpsc::Sender<Option<CapturedRequest>>,
            read_error_kind: &Mutex<Option<std::io::ErrorKind>>,
        ) -> std::io::Result<()> {
            stream.set_nonblocking(false)?;
            stream.set_read_timeout(Some(Duration::from_secs(2)))?;
            stream.set_write_timeout(Some(Duration::from_secs(2)))?;
            let connection = ServerConnection::new(config)
                .map_err(|_| std::io::Error::other("test TLS server config failed"))?;
            let mut tls = StreamOwned::new(connection, stream);
            let captured = match read_request(&mut tls) {
                Ok(request) => request,
                Err(error) => {
                    if let Ok(mut kind) = read_error_kind.lock() {
                        *kind = Some(error.kind());
                    }
                    let _ = requests.send(None);
                    return Ok(());
                }
            };
            if requests.send(Some(captured)).is_err() {
                return Ok(());
            }
            write_response(&mut tls, response)
        }

        fn read_request(stream: &mut impl Read) -> std::io::Result<CapturedRequest> {
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                if bytes.len() >= MAX_TEST_HEADER_BYTES {
                    return Err(std::io::Error::other("test request headers exceeded bound"));
                }
                let count = stream.read(&mut buffer)?;
                if count == 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "request closed"));
                }
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.len() > MAX_TEST_HEADER_BYTES {
                    return Err(std::io::Error::other("test request headers exceeded bound"));
                }
                if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let text = std::str::from_utf8(&bytes)
                .map_err(|_| std::io::Error::other("test request was not UTF-8"))?;
            let mut lines = text.split("\r\n");
            let line = lines.next().unwrap_or_default().to_owned();
            let mut headers = BTreeMap::new();
            for line in lines {
                if line.is_empty() { break; }
                if let Some((name, value)) = line.split_once(':') {
                    headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
                }
            }
            Ok(CapturedRequest { line, headers })
        }

        fn write_response(stream: &mut impl Write, response: ResponsePlan) -> std::io::Result<()> {
            let reason = match response.status {
                200 => "OK",
                302 => "Found",
                403 => "Forbidden",
                429 => "Too Many Requests",
                _ => "Test Response",
            };
            write!(stream, "HTTP/1.1 {} {}\r\n", response.status, reason)?;
            for (name, value) in response.headers {
                write!(stream, "{name}: {value}\r\n")?;
            }
            match response.wire {
                BodyWire::Chunked => write!(stream, "Transfer-Encoding: chunked\r\n")?,
                _ => write!(stream, "Content-Length: {}\r\n", response.body.len())?,
            }
            write!(stream, "Connection: close\r\n\r\n")?;

            match response.wire {
                BodyWire::Fixed => stream.write_all(&response.body)?,
                BodyWire::Chunked => {
                    write!(stream, "{:X}\r\n", response.body.len())?;
                    stream.write_all(&response.body)?;
                    write!(stream, "\r\n0\r\n\r\n")?;
                }
                BodyWire::Stall { prefix_bytes, duration } => {
                    let prefix_bytes = prefix_bytes.min(response.body.len());
                    stream.write_all(&response.body[..prefix_bytes])?;
                    stream.flush()?;
                    thread::sleep(duration);
                    let _ = stream.write_all(&response.body[prefix_bytes..]);
                }
            }
            stream.flush()
        }

        fn run_json() -> Vec<u8> {
            serde_json::to_vec(&serde_json::json!({
                "id": 41,
                "run_number": 9,
                "run_attempt": 1,
                "workflow_id": 7,
                "path": ".github/workflows/ci.yml@refs/heads/main",
                "event": "push",
                "status": "completed",
                "conclusion": "success",
                "head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "head_branch": "main",
                "repository": { "full_name": "owner/repo" }
            }))
            .expect("static run JSON serializes")
        }

        fn api_url(path: &str) -> String {
            format!("https://{API_HOST}{path}")
        }

        fn listener_has_no_connection(listener: &TcpListener) -> bool {
            matches!(
                listener.accept(),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
            )
        }

        fn direct_get(
            transport: &ReqwestTransport,
            url: &str,
            authorization: Option<&str>,
            max_bytes: usize,
            timeout: Duration,
        ) -> Result<crate::api::HttpResponse, TransportError> {
            transport.execute_get(url, authorization, Instant::now() + timeout, max_bytes)
        }

        #[derive(Clone)]
        struct RecordingTransport {
            inner: ReqwestTransport,
            calls: Arc<Mutex<Vec<(bool, bool)>>>,
        }

        impl RecordingTransport {
            fn new(inner: ReqwestTransport) -> Self {
                Self { inner, calls: Arc::new(Mutex::new(Vec::new())) }
            }

            fn count(&self) -> usize {
                self.calls.lock().expect("recording mutex is available").len()
            }

            fn calls(&self) -> Vec<(bool, bool)> {
                self.calls.lock().expect("recording mutex is available").clone()
            }
        }

        impl ReadOnlyTransport for RecordingTransport {
            fn get(
                &self,
                request: crate::api::HttpRequest,
                deadline: Instant,
                max_bytes: usize,
            ) -> Result<crate::api::HttpResponse, TransportError> {
                let parsed = reqwest::Url::parse(request.url())
                    .map_err(|_| TransportError::Unavailable)?;
                let is_api = super::is_api_origin(&parsed);
                let has_auth = request.authorization().is_some();
                self.calls
                    .lock()
                    .map_err(|_| TransportError::Unavailable)?
                    .push((is_api, has_auth));
                self.inner.get(request, deadline, max_bytes)
            }
        }

        #[test]
        fn verified_tls_get_forwards_bounded_response_and_expected_headers() {
            let body = b"opaque-archive-bytes".to_vec();
            let mut server = TestServer::start(
                vec![
                    ResponsePlan::fixed(200, body.clone())
                        .header("Content-Encoding", "gzip")
                        .header("Location", "https://objects.githubusercontent.com/a.zip?sig=hidden")
                        .header("Retry-After", "7")
                        .header("X-RateLimit-Remaining", "9")
                        .header("X-Not-Forwarded", "ignored"),
                ],
                &[API_HOST, CDN_HOST],
            );
            let transport = server.client();
            let response = direct_get(
                &transport,
                &api_url("/repos/owner/repo/actions/runs/41"),
                Some(TOKEN),
                128,
                Duration::from_secs(2),
            )
            .expect("trusted loopback TLS request succeeds");
            assert!(response.status == 200, "response status was preserved");
            assert!(response.body == body, "raw body bytes were preserved");
            assert!(response.headers.len() == 3, "only consumed response headers were returned");
            assert!(response.headers.get("location").is_some(), "Location was preserved");
            assert!(response.headers.get("retry-after").is_some(), "Retry-After was preserved");
            assert!(response.headers.get("x-ratelimit-remaining").is_some(), "rate limit header was preserved");

            let request = server.next_request().expect("TLS request reached the loopback server");
            assert!(request.path() == Some("/repos/owner/repo/actions/runs/41"), "GET path was preserved");
            assert!(request.line.starts_with("GET "), "only GET was issued");
            assert!(request.header("user-agent") == Some(super::USER_AGENT_VALUE), "fixed User-Agent was set");
            assert!(request.header("authorization") == Some("Bearer test-token-never-a-real-secret"), "API bearer header was set");
            assert!(request.header("accept") == Some("application/vnd.github+json"), "API Accept header was set");
            assert!(request.header("x-github-api-version") == Some("2022-11-28"), "API version header was set");
            server.finish();
        }

        #[test]
        fn diagnostic_loopback_tls_get_reports_sanitized_route_and_handshake_evidence() {
            let mut server = TestServer::start(
                vec![ResponsePlan::fixed(200, b"diagnostic-ok".to_vec())],
                &[API_HOST],
            );
            let transport = server.client();
            let url = api_url("/diagnostic?query-must-not-be-logged");
            let result = direct_get(
                &transport,
                &url,
                None,
                64,
                Duration::from_secs(2),
            );
            let (accepted_connections, _) = server.diagnostic();
            let request_observed = if accepted_connections > 0 {
                server.next_request().is_some()
            } else {
                false
            };
            let (_, server_read_error_kind) = server.diagnostic();
            let transport_diagnostic = transport.test_diagnostic();
            let expected_wire = super::TestWireUrl {
                scheme: "https".to_owned(),
                host: Some(API_HOST.to_owned()),
                explicit_port: Some(server.address.port()),
                effective_port: Some(server.address.port()),
            };
            let transport_result = result.as_ref().map(|response| (response.status, response.body.len()));
            assert!(
                result.is_ok()
                    && transport_diagnostic.wire.as_ref() == Some(&expected_wire)
                    && accepted_connections == 1
                    && request_observed,
                "diagnostic TLS evidence: transport_result={transport_result:?}; wire_authority={:?}; reqwest_error={:?}; accepted_connections={accepted_connections}; request_observed={request_observed}; server_read_error_kind={server_read_error_kind:?}",
                transport_diagnostic.wire,
                transport_diagnostic.reqwest_error,
            );
            server.finish();
        }

        #[test]
        fn bearer_header_is_sensitive_and_its_debug_form_is_redacted() {
            let header = sensitive_bearer(TOKEN).expect("test token is a valid header value");
            assert!(header.is_sensitive(), "bearer value is marked sensitive");
            assert!(!format!("{header:?}").contains(TOKEN), "header debug output hides the token");
            let malformed = "test-token-must-not-appear\ninvalid";
            let error = sensitive_bearer(malformed);
            assert!(error.is_err(), "invalid header characters are rejected");
            assert!(!format!("{error:?}").contains(malformed), "header error output hides input text");
        }

        #[test]
        fn provider_initialization_is_concurrent_idempotent_and_preserves_existing_choice() {
            let before = rustls::crypto::CryptoProvider::get_default().cloned();
            let barrier = Arc::new(std::sync::Barrier::new(9));
            let mut workers = Vec::new();
            for _ in 0..8 {
                let barrier = Arc::clone(&barrier);
                workers.push(
                    thread::Builder::new()
                        .name("visibility-rustls-provider".to_owned())
                        .spawn(move || {
                            barrier.wait();
                            ensure_default_provider().is_ok()
                        })
                        .expect("provider worker starts"),
                );
            }
            barrier.wait();
            let all_succeeded = workers
                .into_iter()
                .map(|worker| worker.join().is_ok_and(|succeeded| succeeded))
                .fold(true, |all_ok, succeeded| all_ok && succeeded);
            assert!(all_succeeded, "concurrent provider initialization succeeded");
            let after = rustls::crypto::CryptoProvider::get_default().cloned();
            assert!(after.is_some(), "a process provider is available after initialization");
            if let (Some(before), Some(after)) = (before, after) {
                assert!(Arc::ptr_eq(&before, &after), "an existing process provider was preserved");
            }
        }

        #[test]
        fn api_client_owns_same_origin_and_cross_origin_redirects() {
            let body = run_json();
            let mut same_origin = TestServer::start(
                vec![
                    ResponsePlan::fixed(302, Vec::<u8>::new())
                        .header("Location", "/repos/owner/repo/actions/runs/41?next=1"),
                    ResponsePlan::fixed(200, body),
                ],
                &[API_HOST, CDN_HOST],
            );
            let recorder = RecordingTransport::new(same_origin.client());
            let api = ApiClient::new(recorder.clone(), TOKEN).expect("test token is accepted");
            let repository = Repository::new("owner", "repo").expect("test repository is valid");
            let run = api.get_run(&repository, 41).expect("API client follows same-origin redirect");
            assert!(run.id == 41, "final same-origin response was parsed");
            assert!(recorder.count() == 2, "both redirect hops went through the API transport");
            assert!(recorder.calls() == vec![(true, true), (true, true)], "auth remained limited to API-origin GETs");
            let first = same_origin.next_request().expect("first same-origin request was observed");
            let second = same_origin.next_request().expect("second same-origin request was observed");
            assert!(first.header("authorization") == Some("Bearer test-token-never-a-real-secret"), "first API request was authenticated");
            assert!(second.header("authorization") == Some("Bearer test-token-never-a-real-secret"), "same-origin redirect retained auth");
            same_origin.finish();

            let artifact_bytes = b"opaque-zip-payload".to_vec();
            let location = "https://objects.githubusercontent.com/artifacts/77.zip?sig=private-query";
            let mut cross_origin = TestServer::start(
                vec![
                    ResponsePlan::fixed(302, Vec::<u8>::new()).header("Location", location),
                    ResponsePlan::fixed(200, artifact_bytes.clone()),
                ],
                &[API_HOST, CDN_HOST],
            );
            let recorder = RecordingTransport::new(cross_origin.client());
            let api = ApiClient::new(recorder.clone(), TOKEN).expect("test token is accepted");
            let repository = Repository::new("owner", "repo").expect("test repository is valid");
            let downloaded = api
                .download_artifact(&repository, 77, 1024)
                .expect("API client follows the safe artifact redirect");
            assert!(downloaded == artifact_bytes, "artifact bytes remained unchanged");
            assert!(recorder.count() == 2, "both redirect hops went through the API transport");
            assert!(recorder.calls() == vec![(true, true), (false, false)], "auth was removed before the cross-origin GET");
            let api_request = cross_origin.next_request().expect("API redirect request was observed");
            let cdn_request = cross_origin.next_request().expect("CDN artifact request was observed");
            assert!(api_request.header("authorization").is_some(), "API origin was authenticated");
            assert!(cdn_request.header("authorization").is_none(), "cross-origin request omitted auth");
            assert!(cdn_request.header("x-github-api-version").is_none(), "CDN request omitted API version header");
            assert!(cdn_request.header("accept") != Some("application/vnd.github+json"), "CDN request omitted API Accept header");
            assert!(cdn_request.header("referer").is_none(), "redirect did not add Referer");
            assert!(cdn_request.line.starts_with("GET "), "redirect remained GET-only");
            cross_origin.finish();
        }

        #[test]
        fn transport_returns_redirect_without_following_it() {
            let mut server = TestServer::start(
                vec![ResponsePlan::fixed(302, Vec::<u8>::new())
                    .header("Location", "https://objects.githubusercontent.com/a.zip?sig=opaque")],
                &[API_HOST, CDN_HOST],
            );
            let transport = server.client();
            let response = direct_get(
                &transport,
                &api_url("/repos/owner/repo/actions/artifacts/77/zip"),
                Some(TOKEN),
                128,
                Duration::from_secs(2),
            )
            .expect("redirect response is returned to ApiClient");
            assert!(response.status == 302, "transport returned the redirect status");
            assert!(response.headers.contains_key("location"), "transport returned Location");
            assert!(server.next_request().is_some(), "exactly one transport request reached the server");
            server.finish();
        }

        #[test]
        fn response_cap_accepts_exact_limit_and_rejects_declared_and_chunked_overflow() {
            let cap = 6;
            let mut exact = TestServer::start(
                vec![ResponsePlan::fixed(200, b"123456".to_vec())],
                &[API_HOST],
            );
            let response = direct_get(
                &exact.client(),
                &api_url("/exact"),
                None,
                cap,
                Duration::from_secs(2),
            )
            .expect("body exactly at the byte cap succeeds");
            assert!(response.body == b"123456", "exact-cap body is complete");
            assert!(exact.next_request().is_some(), "exact-cap request reached server");
            exact.finish();

            let mut declared = TestServer::start(
                vec![ResponsePlan::fixed(200, b"1234567".to_vec())],
                &[API_HOST],
            );
            let declared_result = direct_get(
                &declared.client(),
                &api_url("/declared-overflow"),
                None,
                cap,
                Duration::from_secs(2),
            );
            assert!(declared_result == Err(TransportError::BodyLimit), "declared over-cap body is rejected");
            assert!(declared.next_request().is_some(), "declared over-cap request reached server");
            declared.finish();

            let mut chunked = TestServer::start(
                vec![ResponsePlan::fixed(200, b"1234567".to_vec()).chunked()],
                &[API_HOST],
            );
            let chunked_result = direct_get(
                &chunked.client(),
                &api_url("/chunked-overflow"),
                None,
                cap,
                Duration::from_secs(2),
            );
            assert!(chunked_result == Err(TransportError::BodyLimit), "chunked over-cap body is rejected");
            assert!(chunked.next_request().is_some(), "chunked over-cap request reached server");
            chunked.finish();
        }

        #[test]
        fn expired_deadline_makes_no_connection_and_stalled_body_times_out() {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback bind succeeds");
            listener.set_nonblocking(true).expect("listener can check for a connection");
            let address = listener.local_addr().expect("listener has local address");
            let root = tls_material(&[API_HOST]);
            let cert = reqwest::Certificate::from_der(&root.root_der).expect("generated CA is valid");
            let client = build_client(
                Some(vec![cert]),
                &[(API_HOST.to_owned(), address)],
            )
            .expect("test client builds");
            let transport = ReqwestTransport::with_client(client);
            let expired = transport.execute_get(
                &api_url("/expired"),
                None,
                Instant::now() - Duration::from_millis(1),
                8,
            );
            assert!(expired == Err(TransportError::Deadline), "expired deadline is rejected");
            assert!(listener_has_no_connection(&listener), "expired deadline opened no TCP connection");

            let mut stalled = TestServer::start(
                vec![ResponsePlan::fixed(200, b"ab".to_vec()).stall(1, Duration::from_millis(350))],
                &[API_HOST],
            );
            let transport = stalled.client();
            let start = Instant::now();
            let result = direct_get(
                &transport,
                &api_url("/stall"),
                None,
                8,
                Duration::from_millis(120),
            );
            assert!(result == Err(TransportError::Deadline), "stalled body is bounded by the absolute deadline");
            assert!(start.elapsed() < Duration::from_secs(2), "body timeout returned promptly");
            assert!(stalled.next_request().is_some(), "stalled request reached the loopback server");
            stalled.finish();
        }

        #[test]
        fn test_client_trusts_only_ca_and_rejects_hostname_mismatch() {
            let mut trusted = TestServer::start(
                vec![ResponsePlan::fixed(200, b"ok".to_vec())],
                &[API_HOST],
            );
            let response = direct_get(
                &trusted.client(),
                &api_url("/trusted"),
                None,
                8,
                Duration::from_secs(2),
            )
            .expect("test-only CA permits trusted SAN");
            assert!(response.body == b"ok", "trusted SAN response was read");
            assert!(trusted.next_request().is_some(), "trusted request reached server");
            trusted.finish();

            let mut untrusted = TestServer::start(
                vec![ResponsePlan::fixed(200, b"must-not-be-read".to_vec())],
                &[API_HOST],
            );
            let other_root = tls_material(&["unrelated.invalid"]);
            let other_ca = reqwest::Certificate::from_der(&other_root.root_der)
                .expect("unrelated generated test CA is valid");
            let other_client = build_client(
                Some(vec![other_ca]),
                &[
                    (API_HOST.to_owned(), untrusted.address),
                    (CDN_HOST.to_owned(), untrusted.address),
                ],
            )
            .expect("client can be restricted to an unrelated test CA");
            let result = direct_get(
                &ReqwestTransport::with_loopback_client(other_client, untrusted.address.port()),
                &api_url("/untrusted-root"),
                None,
                64,
                Duration::from_secs(2),
            );
            assert!(result == Err(TransportError::Unavailable), "untrusted test CA is rejected");
            assert!(untrusted.next_request().is_none(), "untrusted CA reached no HTTP request");
            untrusted.finish();

            let mut wrong_name = TestServer::start(
                vec![ResponsePlan::fixed(200, b"must-not-be-read".to_vec())],
                &["wrong.invalid"],
            );
            let result = direct_get(
                &wrong_name.client(),
                &api_url("/wrong-name"),
                None,
                64,
                Duration::from_secs(2),
            );
            assert!(result == Err(TransportError::Unavailable), "trusted CA does not bypass hostname validation");
            assert!(wrong_name.next_request().is_none(), "hostname mismatch reached no HTTP request");
            wrong_name.finish();
        }

        #[test]
        fn https_only_builder_rejects_plain_http_before_connecting() {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback bind succeeds");
            listener.set_nonblocking(true).expect("listener can check for a connection");
            let address = listener.local_addr().expect("listener has local address");
            let transport = ReqwestTransport::new().expect("production client builds");
            let url = format!("http://127.0.0.1:{}/blocked", address.port());
            let result = direct_get(
                &transport,
                &url,
                None,
                8,
                Duration::from_secs(2),
            );
            assert!(result == Err(TransportError::Unavailable), "plain HTTP is rejected by shared builder");
            assert!(listener_has_no_connection(&listener), "plain HTTP opened no connection");
        }

        #[test]
        fn api_error_classification_receives_rate_limit_headers() {
            let mut server = TestServer::start(
                vec![ResponsePlan::fixed(429, b"{}".to_vec())
                    .header("Retry-After", "11")
                    .header("X-RateLimit-Remaining", "0")],
                &[API_HOST],
            );
            let transport = RecordingTransport::new(server.client());
            let client = ApiClient::new(transport.clone(), TOKEN).expect("test token is accepted");
            let repository = Repository::new("owner", "repo").expect("test repository is valid");
            let error = client.get_run(&repository, 41).expect_err("429 is not a successful run response");
            assert!(error == ApiError::RateLimited { retry_after_seconds: Some(11) }, "rate limit headers were preserved for API classification");
            assert!(transport.count() == 1, "429 did not trigger an automatic retry");
            assert!(transport.calls() == vec![(true, true)], "429 was one authenticated API GET");
            assert!(server.next_request().is_some(), "rate-limited request reached server");
            server.finish();
        }
    }
}

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
