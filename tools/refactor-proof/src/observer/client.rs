//! Observer IPC client over inherited anonymous pipe descriptors.

#![expect(
    unsafe_code,
    reason = "observer IPC wraps inherited anonymous pipe file descriptors"
)]

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use base64::Engine as _;

use super::ipc::{
    OBSERVER_STEP_ORDER, ObserverEnv, ObserverObservation, ObserverRequest, ObserverRequestSchema,
    ObserverResponse, ObserverStep,
};

const MAX_LINE_BYTES: usize = 4096;

/// Observer transport is unavailable or the host is not running under the planner observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverUnavailable;

impl std::fmt::Display for ObserverUnavailable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("observer IPC transport unavailable")
    }
}

impl std::error::Error for ObserverUnavailable {}

impl From<ObserverUnavailable> for io::Error {
    fn from(value: ObserverUnavailable) -> Self {
        Self::new(io::ErrorKind::Unsupported, value)
    }
}

/// Host-side observer IPC client bound to inherited pipe descriptors.
#[derive(Debug)]
pub struct ObserverClient {
    nonce: String,
    run_id: String,
    task_id: String,
    check_id: String,
    source_commit: String,
    tree: String,
    request_id: u64,
    request: BufWriter<File>,
    response: BufReader<File>,
}

impl ObserverClient {
    /// Open a client from observer-supplied environment.
    pub fn from_env() -> Result<Self, ObserverUnavailable> {
        let request_fd = std::env::var(ObserverEnv::REQUEST_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        let response_fd = std::env::var(ObserverEnv::RESPONSE_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        if std::env::var_os(ObserverEnv::SOCKET).is_some() {
            return Err(ObserverUnavailable);
        }
        Ok(Self {
            nonce: required_env(ObserverEnv::NONCE)?,
            run_id: required_env(ObserverEnv::RUN_ID)?,
            task_id: required_env(ObserverEnv::TASK_ID)?,
            check_id: required_env(ObserverEnv::CHECK_ID)?,
            source_commit: required_env(ObserverEnv::SOURCE_COMMIT)?,
            tree: required_env(ObserverEnv::SOURCE_TREE)?,
            request_id: 0,
            request: BufWriter::new(open_inherited_fd(request_fd)?),
            response: BufReader::new(open_inherited_fd(response_fd)?),
        })
    }

    /// Fail closed when verify requests observer execution before transport exists.
    pub fn require_transport() -> Result<(), ObserverUnavailable> {
        Self::from_env().map(|_| ())
    }

    /// Nonce copied from `TC_PROOF_OBSERVER_NONCE`.
    pub fn nonce(&self) -> &str {
        &self.nonce
    }

    /// Encode one bounded request line for the shared observer ABI.
    pub fn encode_request(&self, step: ObserverStep) -> io::Result<String> {
        encode_request_line(
            &self.nonce,
            &self.run_id,
            &self.task_id,
            &self.check_id,
            self.request_id,
            step.as_str(),
            &self.source_commit,
            &self.tree,
        )
    }

    /// Issue one observer request and return the captured observation.
    pub fn execute_step(
        &mut self,
        step: ObserverStep,
    ) -> Result<ObserverObservation, ObserverUnavailable> {
        let line = self.encode_request(step).map_err(|_| ObserverUnavailable)?;
        self.request
            .write_all(line.as_bytes())
            .and_then(|_| self.request.flush())
            .map_err(|_| ObserverUnavailable)?;
        let response_line =
            read_response_line(&mut self.response).map_err(|_| ObserverUnavailable)?;
        let observation = parse_observation(&response_line)?;
        if observation.nonce != self.nonce
            || observation.run_id != self.run_id
            || observation.task_id != self.task_id
            || observation.check_id != self.check_id
            || observation.request_id != self.request_id
            || observation.operation != step.as_str()
            || observation.source_commit != self.source_commit
            || observation.tree != self.tree
        {
            return Err(ObserverUnavailable);
        }
        self.request_id += 1;
        Ok(observation)
    }

    /// Issue the fixed verify sequence: build, test, taskfmt.
    pub fn execute_verify_sequence(
        &mut self,
    ) -> Result<Vec<ObserverObservation>, ObserverUnavailable> {
        let mut observations = Vec::with_capacity(OBSERVER_STEP_ORDER.len());
        for step in OBSERVER_STEP_ORDER {
            observations.push(self.execute_step(step)?);
        }
        Ok(observations)
    }

    #[cfg(test)]
    fn with_io(
        nonce: impl Into<String>,
        run_id: impl Into<String>,
        task_id: impl Into<String>,
        check_id: impl Into<String>,
        source_commit: impl Into<String>,
        tree: impl Into<String>,
        request: File,
        response: File,
    ) -> Self {
        Self {
            nonce: nonce.into(),
            run_id: run_id.into(),
            task_id: task_id.into(),
            check_id: check_id.into(),
            source_commit: source_commit.into(),
            tree: tree.into(),
            request_id: 0,
            request: BufWriter::new(request),
            response: BufReader::new(response),
        }
    }
}

fn required_env(key: &str) -> Result<String, ObserverUnavailable> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or(ObserverUnavailable)
}

fn encode_request_line(
    nonce: &str,
    run_id: &str,
    task_id: &str,
    check_id: &str,
    request_id: u64,
    operation: &str,
    source_commit: &str,
    tree: &str,
) -> io::Result<String> {
    let request = ObserverRequest {
        schema: ObserverRequestSchema::V1,
        nonce: nonce.to_owned(),
        run_id: run_id.to_owned(),
        task_id: task_id.to_owned(),
        check_id: check_id.to_owned(),
        request_id,
        operation: operation.to_owned(),
        source_commit: source_commit.to_owned(),
        tree: tree.to_owned(),
    };
    let line = serde_json::to_string(&request).map_err(io::Error::other)?;
    if line.len() >= MAX_LINE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "observer request exceeds 4096-byte bound",
        ));
    }
    Ok(format!("{line}\n"))
}

fn read_response_line(reader: &mut impl BufRead) -> io::Result<String> {
    let mut buffer = Vec::new();
    let read = reader.read_until(b'\n', &mut buffer)?;
    if read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "observer response ended before newline",
        ));
    }
    if buffer.len() > MAX_LINE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "observer response exceeds 4096-byte bound",
        ));
    }
    if buffer.last() == Some(&b'\n') {
        buffer.pop();
    }
    String::from_utf8(buffer).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn parse_observation(line: &str) -> Result<ObserverObservation, ObserverUnavailable> {
    let response: ObserverResponse = serde_json::from_str(line).map_err(|_| ObserverUnavailable)?;
    match response {
        ObserverResponse::Observation(observation) => Ok(observation),
        ObserverResponse::Error(_) => Err(ObserverUnavailable),
    }
}

/// Decode one base64 file payload from an observer observation.
pub fn decode_file_payload(encoded: &str) -> Result<Vec<u8>, ObserverUnavailable> {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| ObserverUnavailable)
}

#[cfg(unix)]
fn open_inherited_fd(fd: i32) -> Result<File, ObserverUnavailable> {
    use std::os::fd::{FromRawFd, OwnedFd};
    // SAFETY: inherited anonymous pipe ends are owned by this host process for the fixture lifetime.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    Ok(File::from(owned))
}

#[cfg(not(unix))]
fn open_inherited_fd(_fd: i32) -> Result<File, ObserverUnavailable> {
    Err(ObserverUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::io::{BufReader, BufWriter};
    use std::os::fd::FromRawFd;
    use std::sync::mpsc;
    use std::thread;

    fn pipe_pair() -> io::Result<(File, File)> {
        let mut fds = [0_i32; 2];
        // SAFETY: libc fills two owned descriptors; both are immediately wrapped.
        let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: each descriptor is transferred exactly once to a File.
        Ok((unsafe { File::from_raw_fd(fds[0]) }, unsafe {
            File::from_raw_fd(fds[1])
        }))
    }

    #[test]
    fn request_encoding_uses_shared_binding_schema() {
        let line = encode_request_line(
            "test-nonce",
            "/run",
            "TASK-001",
            "CHK-001",
            0,
            "oracle",
            "source",
            "tree",
        )
        .expect("encode request");
        let request: ObserverRequest = serde_json::from_str(line.trim()).expect("parse request");
        assert_eq!(request.schema, ObserverRequestSchema::V1);
        assert_eq!(request.operation, "oracle");
    }

    #[test]
    fn from_env_fails_closed_without_observer() {
        let has_observer_env = [
            ObserverEnv::REQUEST_FD,
            ObserverEnv::RESPONSE_FD,
            ObserverEnv::NONCE,
        ]
        .into_iter()
        .any(|key| std::env::var(key).is_ok());
        if has_observer_env {
            return;
        }
        assert!(ObserverClient::from_env().is_err());
    }

    #[test]
    fn round_trip_verify_sequence_over_pipe_pair() {
        let (request_read, request_write) = pipe_pair().expect("request pipe");
        let (response_read, response_write) = pipe_pair().expect("response pipe");
        let nonce = "round-trip-nonce".to_owned();
        let server_nonce = nonce.clone();
        let (ready_tx, ready_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let mut requests = BufReader::new(request_read);
            let mut responses = BufWriter::new(response_write);
            ready_tx.send(()).expect("ready");
            for (request_id, step) in OBSERVER_STEP_ORDER.into_iter().enumerate() {
                let line = read_response_line(&mut requests).expect("read request");
                let request: ObserverRequest = serde_json::from_str(&line).expect("parse request");
                assert_eq!(request.nonce, server_nonce);
                assert_eq!(request.run_id, "/run");
                assert_eq!(request.task_id, "TASK-001");
                assert_eq!(request.check_id, "CHK-001");
                assert_eq!(request.request_id, request_id as u64);
                assert_eq!(request.operation, step.as_str());
                let observation = ObserverObservation {
                    nonce: request.nonce,
                    run_id: request.run_id,
                    task_id: request.task_id,
                    check_id: request.check_id,
                    request_id: request.request_id,
                    operation: request.operation,
                    source_commit: request.source_commit,
                    tree: request.tree,
                    exit: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                    files: BTreeMap::new(),
                    payload: Value::Object(Default::default()),
                    records: Vec::new(),
                };
                let payload = serde_json::to_string(&ObserverResponse::Observation(observation))
                    .expect("encode observation");
                responses
                    .write_all(format!("{payload}\n").as_bytes())
                    .expect("write response");
                responses.flush().expect("flush response");
            }
        });
        ready_rx.recv().expect("server ready");
        let mut client = ObserverClient::with_io(
            nonce,
            "/run",
            "TASK-001",
            "CHK-001",
            "source",
            "tree",
            request_write,
            response_read,
        );
        let observations = client.execute_verify_sequence().expect("verify sequence");
        assert_eq!(observations.len(), 3);
        server.join().expect("join server");
    }
}
