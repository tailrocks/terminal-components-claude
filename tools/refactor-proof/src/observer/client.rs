//! Observer IPC client over inherited anonymous pipe descriptors.

#![expect(unsafe_code, reason = "observer IPC wraps inherited anonymous pipe file descriptors")]

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use base64::Engine as _;

use super::ipc::{ObserverObservation, ObserverRequest, ObserverRequestSchema, ObserverResponse, ObserverStep, OBSERVER_STEP_ORDER};

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
pub struct ObserverClient {
    nonce: String,
    request: BufWriter<File>,
    response: BufReader<File>,
}

impl ObserverClient {
    /// Open a client from observer-supplied environment.
    pub fn from_env() -> Result<Self, ObserverUnavailable> {
        let request_fd = std::env::var(super::ipc::ObserverEnv::REQUEST_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        let response_fd = std::env::var(super::ipc::ObserverEnv::RESPONSE_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        let nonce = std::env::var(super::ipc::ObserverEnv::NONCE).map_err(|_| ObserverUnavailable)?;
        if nonce.is_empty() {
            return Err(ObserverUnavailable);
        }
        Ok(Self {
            nonce,
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

    /// Encode one bounded request line for the observer.
    pub fn encode_request(&self, step: ObserverStep) -> io::Result<String> {
        encode_request_line(&self.nonce, step)
    }

    /// Issue one observer request and return the captured observation.
    pub fn execute_step(&mut self, step: ObserverStep) -> Result<ObserverObservation, ObserverUnavailable> {
        let line = self.encode_request(step).map_err(|_| ObserverUnavailable)?;
        self.request
            .write_all(line.as_bytes())
            .and_then(|_| self.request.flush())
            .map_err(|_| ObserverUnavailable)?;
        let response_line = read_response_line(&mut self.response).map_err(|_| ObserverUnavailable)?;
        parse_observation(&response_line)
    }

    /// Issue the fixed verify sequence: build, test, taskfmt.
    pub fn execute_verify_sequence(&mut self) -> Result<Vec<ObserverObservation>, ObserverUnavailable> {
        let mut observations = Vec::with_capacity(OBSERVER_STEP_ORDER.len());
        for step in OBSERVER_STEP_ORDER {
            observations.push(self.execute_step(step)?);
        }
        Ok(observations)
    }

    #[cfg(test)]
    fn with_io(nonce: impl Into<String>, request: File, response: File) -> Self {
        Self {
            nonce: nonce.into(),
            request: BufWriter::new(request),
            response: BufReader::new(response),
        }
    }
}

fn encode_request_line(nonce: &str, step: ObserverStep) -> io::Result<String> {
    let request = ObserverRequest {
        schema: ObserverRequestSchema::V1,
        nonce: nonce.to_owned(),
        step,
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
        ObserverResponse::Error(error) => {
            let _ = error;
            Err(ObserverUnavailable)
        }
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
    use crate::observer::ipc::OBSERVER_STEP_ORDER;
    use std::io::Write;
    use std::os::fd::FromRawFd;
    use std::sync::mpsc;
    use std::thread;

    fn pipe_pair() -> io::Result<(File, File)> {
        let mut fds = [0_i32; 2];
        let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((unsafe { File::from_raw_fd(fds[0]) }, unsafe { File::from_raw_fd(fds[1]) }))
    }

    #[test]
    fn request_encoding_respects_step_order_contract() {
        for step in OBSERVER_STEP_ORDER {
            let line = encode_request_line("test-nonce", step).expect("encode request");
            assert!(line.ends_with('\n'));
            assert!(line.len() <= MAX_LINE_BYTES);
        }
    }

    #[test]
    fn from_env_fails_closed_without_observer() {
        let has_observer_env = [
            super::super::ipc::ObserverEnv::REQUEST_FD,
            super::super::ipc::ObserverEnv::RESPONSE_FD,
            super::super::ipc::ObserverEnv::NONCE,
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
            for step in OBSERVER_STEP_ORDER {
                let line = read_response_line(&mut requests).expect("read request");
                let request: ObserverRequest = serde_json::from_str(&line).expect("parse request");
                assert_eq!(request.nonce, server_nonce);
                assert_eq!(request.step, step);
                let observation = ObserverObservation {
                    step,
                    tree: "abc123".into(),
                    argv: vec!["fixture".into()],
                    exit: 0,
                    stdout: base64::engine::general_purpose::STANDARD.encode(b"stdout"),
                    stderr: String::new(),
                    files: std::collections::BTreeMap::from([(
                        format!("{}.json", step.as_str()),
                        base64::engine::general_purpose::STANDARD.encode(b"{}"),
                    )]),
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
        let mut client = ObserverClient::with_io(nonce.clone(), request_write, response_read);
        let observations = client.execute_verify_sequence().expect("verify sequence");
        assert_eq!(observations.len(), 3);
        server.join().expect("join server");
    }
}
