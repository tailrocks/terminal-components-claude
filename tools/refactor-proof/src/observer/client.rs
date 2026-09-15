//! Observer IPC client stub (Phase 2 compile-only; transport in Phase 3).

use std::io;

use super::ipc::{ObserverObservation, ObserverRequest, ObserverStep};

/// Host-side observer IPC client bound to inherited pipe descriptors.
///
/// Phase 3 will implement newline-framed JSON over `TC_PROOF_OBSERVER_*_FD`.
pub struct ObserverClient {
    nonce: String,
}

impl ObserverClient {
    /// Open a client from observer-supplied environment (Phase 3).
    pub fn from_env() -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "observer IPC transport not implemented (Phase 3)",
        ))
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

    /// Issue the fixed verify sequence: build, test, taskfmt (Phase 3).
    pub fn execute_verify_sequence(&mut self) -> io::Result<Vec<ObserverObservation>> {
        let _ = self.encode_request(ObserverStep::Build)?;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "observer IPC transport not implemented (Phase 3)",
        ))
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
