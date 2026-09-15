//! Observer IPC client stub (Phase 3b adds transport over inherited FDs).

use std::io;

use super::ipc::{ObserverObservation, ObserverRequest, ObserverStep};

/// Observer transport is not yet wired; callers receive this instead of panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverUnavailable;

impl std::fmt::Display for ObserverUnavailable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("observer IPC transport not implemented (Phase 3b)")
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
}

impl ObserverClient {
    /// Open a client from observer-supplied environment.
    ///
    /// Returns [`ObserverUnavailable`] until Phase 3b implements FD transport.
    pub fn from_env() -> Result<Self, ObserverUnavailable> {
        let _ = (
            std::env::var(super::ipc::ObserverEnv::REQUEST_FD),
            std::env::var(super::ipc::ObserverEnv::RESPONSE_FD),
            std::env::var(super::ipc::ObserverEnv::NONCE),
        );
        Err(ObserverUnavailable)
    }

    /// Fail closed when verify requests observer execution before transport exists.
    pub fn require_transport() -> Result<(), ObserverUnavailable> {
        Err(ObserverUnavailable)
    }

    /// Nonce copied from `TC_PROOF_OBSERVER_NONCE`.
    pub fn nonce(&self) -> &str {
        &self.nonce
    }

    /// Encode one bounded request line for the observer.
    pub fn encode_request(&self, step: ObserverStep) -> io::Result<String> {
        let request = ObserverRequest {
            schema: super::ipc::ObserverRequestSchema::V1,
            nonce: self.nonce.clone(),
            step,
        };
        let line = serde_json::to_string(&request).map_err(io::Error::other)?;
        if line.len() >= 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "observer request exceeds 4096-byte bound",
            ));
        }
        Ok(format!("{line}\n"))
    }

    /// Issue the fixed verify sequence: build, test, taskfmt.
    pub fn execute_verify_sequence(&mut self) -> Result<Vec<ObserverObservation>, ObserverUnavailable> {
        let _ = self.encode_request(ObserverStep::Build).map_err(|_| ObserverUnavailable)?;
        Err(ObserverUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::ipc::OBSERVER_STEP_ORDER;

    #[test]
    fn request_encoding_respects_step_order_contract() {
        let client = ObserverClient {
            nonce: "test-nonce".into(),
        };
        for step in OBSERVER_STEP_ORDER {
            let line = client.encode_request(step).expect("encode request");
            assert!(line.ends_with('\n'));
            assert!(line.len() < 4096);
        }
    }
}
