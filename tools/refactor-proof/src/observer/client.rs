#![expect(
    unsafe_code,
    reason = "observer IPC wraps inherited anonymous pipe file descriptors"
)]

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use base64::Engine as _;

use super::ipc::{
    OBSERVER_STEP_ORDER, ObserverObservation, ObserverRequest, ObserverRequestSchema,
    ObserverResponse, ObserverStep,
};

const MAX_LINE_BYTES: usize = 4096;

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

pub struct ObserverClient {
    nonce: String,
    request: BufWriter<File>,
    response: BufReader<File>,
}

impl ObserverClient {
    pub fn from_env() -> Result<Self, ObserverUnavailable> {
        let request_fd = std::env::var(super::ipc::ObserverEnv::REQUEST_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        let response_fd = std::env::var(super::ipc::ObserverEnv::RESPONSE_FD)
            .map_err(|_| ObserverUnavailable)?
            .parse::<i32>()
            .map_err(|_| ObserverUnavailable)?;
        let nonce =
            std::env::var(super::ipc::ObserverEnv::NONCE).map_err(|_| ObserverUnavailable)?;
        if nonce.is_empty() {
            return Err(ObserverUnavailable);
        }
        Ok(Self {
            nonce,
            request: BufWriter::new(open_inherited_fd(request_fd)?),
            response: BufReader::new(open_inherited_fd(response_fd)?),
        })
    }

    pub fn require_transport() -> Result<(), ObserverUnavailable> {
        Self::from_env().map(|_| ())
    }

    pub fn nonce(&self) -> &str {
        &self.nonce
    }

    pub fn encode_request(&self, step: ObserverStep) -> io::Result<String> {
        encode_request_line(&self.nonce, step)
    }

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
        parse_observation(&response_line)
    }

    pub fn execute_verify_sequence(
        &mut self,
    ) -> Result<Vec<ObserverObservation>, ObserverUnavailable> {
        let mut observations = Vec::with_capacity(OBSERVER_STEP_ORDER.len());
        for step in OBSERVER_STEP_ORDER {
            observations.push(self.execute_step(step)?);
        }
        Ok(observations)
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

pub fn decode_file_payload(encoded: &str) -> Result<Vec<u8>, ObserverUnavailable> {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| ObserverUnavailable)
}

#[cfg(unix)]
fn open_inherited_fd(fd: i32) -> Result<File, ObserverUnavailable> {
    use std::os::fd::{FromRawFd, OwnedFd};

    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    Ok(File::from(owned))
}

#[cfg(not(unix))]
fn open_inherited_fd(_fd: i32) -> Result<File, ObserverUnavailable> {
    Err(ObserverUnavailable)
}
