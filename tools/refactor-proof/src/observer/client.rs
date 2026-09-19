//! Observer IPC client over inherited anonymous pipe descriptors.

#![expect(
    unsafe_code,
    reason = "observer IPC wraps inherited anonymous pipe file descriptors"
)]

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use base64::Engine as _;

use crate::json_util::parse_json_strict;

use super::ipc::{
    OBSERVER_STEP_ORDER, ObserverEnv, ObserverObservation, ObserverRequest, ObserverRequestSchema,
    ObserverResponse, ObserverStep,
};

const MAX_REQUEST_BYTES: usize = 4096;
const MAX_RESPONSE_BYTES: usize = 10_000_000;

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
    identity: ObserverIdentity,
    request_id: u64,
    request: BufWriter<File>,
    response: BufReader<File>,
}

#[derive(Debug)]
struct ObserverIdentity {
    nonce: String,
    run_id: String,
    task_id: String,
    check_id: String,
    source_commit: String,
    tree: String,
}

impl ObserverClient {
    /// Open a client from observer-supplied environment.
    ///
    /// # Errors
    ///
    /// Returns [`ObserverUnavailable`] when the required environment bindings
    /// are absent, invalid, or request the retired socket transport.
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
            identity: ObserverIdentity {
                nonce: required_env(ObserverEnv::NONCE)?,
                run_id: required_env(ObserverEnv::RUN_ID)?,
                task_id: required_env(ObserverEnv::TASK_ID)?,
                check_id: required_env(ObserverEnv::CHECK_ID)?,
                source_commit: required_env(ObserverEnv::SOURCE_COMMIT)?,
                tree: required_env(ObserverEnv::SOURCE_TREE)?,
            },
            request_id: 0,
            request: BufWriter::new(open_inherited_fd(request_fd)?),
            response: BufReader::new(open_inherited_fd(response_fd)?),
        })
    }

    /// Fail closed when verify requests observer execution before transport exists.
    ///
    /// # Errors
    ///
    /// Returns [`ObserverUnavailable`] when the inherited observer transport is
    /// not completely bound.
    pub fn require_transport() -> Result<(), ObserverUnavailable> {
        Self::from_env().map(|_| ())
    }

    /// Nonce copied from `TC_PROOF_OBSERVER_NONCE`.
    pub fn nonce(&self) -> &str {
        &self.identity.nonce
    }

    /// Encode one bounded request line for the shared observer ABI.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the request cannot be serialized or exceeds
    /// the protocol size bound.
    pub fn encode_request(&self, step: ObserverStep) -> io::Result<String> {
        encode_request_line(&self.identity, self.request_id, step.as_str())
    }

    /// Issue one observer request and return the captured observation.
    ///
    /// # Errors
    ///
    /// Returns [`ObserverUnavailable`] when transport I/O, response parsing,
    /// response validation, or request-id advancement fails.
    pub fn execute_step(
        &mut self,
        step: ObserverStep,
    ) -> Result<ObserverObservation, ObserverUnavailable> {
        let next_request_id = self.request_id.checked_add(1).ok_or(ObserverUnavailable)?;
        let line = self.encode_request(step).map_err(|_| ObserverUnavailable)?;
        self.request
            .write_all(line.as_bytes())
            .map_err(|_| ObserverUnavailable)?;
        self.request.flush().map_err(|_| ObserverUnavailable)?;
        let response_line =
            read_response_line(&mut self.response).map_err(|_| ObserverUnavailable)?;
        let observation = parse_observation(&response_line)?;
        if observation.nonce != self.identity.nonce
            || observation.run_id != self.identity.run_id
            || observation.task_id != self.identity.task_id
            || observation.check_id != self.identity.check_id
            || observation.request_id != self.request_id
            || observation.operation != step.as_str()
            || observation.source_commit != self.identity.source_commit
            || observation.tree != self.identity.tree
        {
            return Err(ObserverUnavailable);
        }
        self.request_id = next_request_id;
        Ok(observation)
    }

    /// Issue the fixed verify sequence: build, test, taskfmt.
    ///
    /// # Errors
    ///
    /// Returns [`ObserverUnavailable`] when any step cannot be completed or
    /// validated.
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
    fn with_io(identity: ObserverIdentity, request: File, response: File) -> Self {
        Self {
            identity,
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
    identity: &ObserverIdentity,
    request_id: u64,
    operation: &str,
) -> io::Result<String> {
    let request = ObserverRequest {
        schema: ObserverRequestSchema::V1,
        nonce: identity.nonce.clone(),
        run_id: identity.run_id.clone(),
        task_id: identity.task_id.clone(),
        check_id: identity.check_id.clone(),
        request_id,
        operation: operation.to_owned(),
        source_commit: identity.source_commit.clone(),
        tree: identity.tree.clone(),
    };
    let line = serde_json::to_string(&request).map_err(io::Error::other)?;
    if line.len() >= MAX_REQUEST_BYTES {
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
    if buffer.len() > MAX_RESPONSE_BYTES {
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
    let value = parse_json_strict(line).map_err(|_| ObserverUnavailable)?;
    let response: ObserverResponse =
        serde_json::from_value(value).map_err(|_| ObserverUnavailable)?;
    match response {
        ObserverResponse::Observation(observation) => {
            if observation.exit != 0
                || !observation.payload.is_object()
                || observation
                    .payload
                    .as_object()
                    .is_some_and(serde_json::Map::is_empty)
                || observation.records.is_empty()
            {
                return Err(ObserverUnavailable);
            }
            Ok(*observation)
        }
        ObserverResponse::Error(_) => Err(ObserverUnavailable),
    }
}

/// Decode one base64 file payload from an observer observation.
///
/// # Errors
///
/// Returns [`ObserverUnavailable`] when `encoded` is not valid standard
/// base64.
pub fn decode_file_payload(encoded: &str) -> Result<Vec<u8>, ObserverUnavailable> {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| ObserverUnavailable)
}

#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the non-Unix implementation reports an unavailable transport"
)]
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
    fn request_encoding_uses_shared_binding_schema()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let identity = ObserverIdentity {
            nonce: "test-nonce".to_owned(),
            run_id: "/run".to_owned(),
            task_id: "TASK-001".to_owned(),
            check_id: "CHK-001".to_owned(),
            source_commit: "source".to_owned(),
            tree: "tree".to_owned(),
        };
        let line = encode_request_line(&identity, 0, "oracle")?;
        let request: ObserverRequest = serde_json::from_str(line.trim())?;
        assert_eq!(request.schema, ObserverRequestSchema::V1);
        assert_eq!(request.operation, "oracle");
        Ok(())
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
    fn empty_observation_payload_and_records_are_rejected()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let observation = ObserverObservation {
            nonce: "nonce".to_string(),
            run_id: "/run".to_string(),
            task_id: "TASK-001".to_string(),
            check_id: "CHK-001".to_string(),
            request_id: 0,
            operation: "capture".to_string(),
            source_commit: "source".to_string(),
            tree: "tree".to_string(),
            exit: 0,
            stdout: String::new(),
            stderr: String::new(),
            files: BTreeMap::new(),
            payload: Value::Object(serde_json::Map::default()),
            records: Vec::new(),
        };
        let line = serde_json::to_string(&ObserverResponse::Observation(Box::new(observation)))?;
        assert!(parse_observation(&line).is_err());
        Ok(())
    }

    #[test]
    fn forged_observation_binding_is_rejected()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (request_read, request_write) = pipe_pair()?;
        let (response_read, mut response_write) = pipe_pair()?;
        let server = thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut requests = BufReader::new(request_read);
                let line = read_response_line(&mut requests)?;
                let request: ObserverRequest = serde_json::from_str(&line)?;
                let observation = ObserverObservation {
                    nonce: "forged-nonce".to_string(),
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
                    payload: serde_json::json!({"observed": true}),
                    records: vec![serde_json::json!({"observed": true})],
                };
                let line =
                    serde_json::to_string(&ObserverResponse::Observation(Box::new(observation)))?;
                response_write.write_all(format!("{line}\n").as_bytes())?;
                response_write.flush()?;
                Ok(())
            },
        );
        let mut client = ObserverClient::with_io(
            ObserverIdentity {
                nonce: "nonce".to_owned(),
                run_id: "/run".to_owned(),
                task_id: "TASK-001".to_owned(),
                check_id: "CHK-001".to_owned(),
                source_commit: "source".to_owned(),
                tree: "tree".to_owned(),
            },
            request_write,
            response_read,
        );
        assert!(client.execute_step(ObserverStep::Test).is_err());
        server
            .join()
            .map_err(|_| io::Error::other("server thread panicked"))??;
        Ok(())
    }

    #[test]
    fn round_trip_verify_sequence_over_pipe_pair()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (request_read, request_write) = pipe_pair()?;
        let (response_read, response_write) = pipe_pair()?;
        let nonce = "round-trip-nonce".to_owned();
        let server_nonce = nonce.clone();
        let (ready_tx, ready_rx) = mpsc::channel();
        let server = thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut requests = BufReader::new(request_read);
                let mut responses = BufWriter::new(response_write);
                ready_tx
                    .send(())
                    .map_err(|_| io::Error::other("ready receiver dropped"))?;
                for (request_id, step) in OBSERVER_STEP_ORDER.into_iter().enumerate() {
                    let line = read_response_line(&mut requests)?;
                    let request: ObserverRequest = serde_json::from_str(&line)?;
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
                        payload: serde_json::json!({"observed": true}),
                        records: vec![serde_json::json!({"request_id": request.request_id})],
                    };
                    let payload = serde_json::to_string(&ObserverResponse::Observation(Box::new(
                        observation,
                    )))?;
                    responses.write_all(format!("{payload}\n").as_bytes())?;
                    responses.flush()?;
                }
                Ok(())
            },
        );
        ready_rx
            .recv()
            .map_err(|_| io::Error::other("server did not become ready"))?;
        let mut client = ObserverClient::with_io(
            ObserverIdentity {
                nonce,
                run_id: "/run".to_owned(),
                task_id: "TASK-001".to_owned(),
                check_id: "CHK-001".to_owned(),
                source_commit: "source".to_owned(),
                tree: "tree".to_owned(),
            },
            request_write,
            response_read,
        );
        let observations = client.execute_verify_sequence()?;
        assert_eq!(observations.len(), 3);
        server
            .join()
            .map_err(|_| io::Error::other("server thread panicked"))??;
        Ok(())
    }
}
