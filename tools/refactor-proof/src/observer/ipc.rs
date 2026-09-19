//! Shared inherited-pipe observer ABI (`tc-proof-runner-observe/v1`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Fixed compatibility sequence used by the Rust observer client.
pub(super) const OBSERVER_STEP_ORDER: [ObserverStep; 3] = [
    ObserverStep::Build,
    ObserverStep::Test,
    ObserverStep::Taskfmt,
];

/// Environment keys supplied to a worker by the native launcher.
#[derive(Debug)]
pub struct ObserverEnv;

impl ObserverEnv {
    pub const REQUEST_FD: &'static str = "TC_PROOF_OBSERVER_REQUEST_FD";
    pub const RESPONSE_FD: &'static str = "TC_PROOF_OBSERVER_RESPONSE_FD";
    pub const NONCE: &'static str = "TC_PROOF_OBSERVER_NONCE";
    pub const RUN_ID: &'static str = "TC_PROOF_RUN_ID";
    pub const TASK_ID: &'static str = "TC_PROOF_TASK_ID";
    pub const CHECK_ID: &'static str = "TC_PROOF_CHECK_ID";
    pub const SOURCE_COMMIT: &'static str = "TC_PROOF_ORACLE_COMMIT";
    pub const SOURCE_TREE: &'static str = "TC_PROOF_SOURCE_TREE";
    pub const SOCKET: &'static str = "TC_PROOF_OBSERVER_SOCKET";
}

/// Fixed verify-phase operation names retained for the Rust client API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObserverStep {
    Build,
    Test,
    Taskfmt,
}

impl ObserverStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
            Self::Taskfmt => "taskfmt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObserverRequestSchema {
    #[serde(rename = "tc-proof-runner-observe/v1")]
    V1,
}

/// Newline-terminated host request. All binding fields are mandatory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverRequest {
    pub schema: ObserverRequestSchema,
    pub nonce: String,
    pub run_id: String,
    pub task_id: String,
    pub check_id: String,
    pub request_id: u64,
    pub operation: String,
    pub source_commit: String,
    pub tree: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "schema")]
pub enum ObserverResponse {
    #[serde(rename = "tc-proof-observation/v1")]
    Observation(Box<ObserverObservation>),
    #[serde(rename = "tc-proof-observer-error/v1")]
    Error(ObserverError),
}

/// Independently captured process evidence returned by the observer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverObservation {
    pub nonce: String,
    pub run_id: String,
    pub task_id: String,
    pub check_id: String,
    pub request_id: u64,
    pub operation: String,
    pub source_commit: String,
    pub tree: String,
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
    pub files: BTreeMap<String, String>,
    pub payload: Value,
    pub records: Vec<Value>,
}

/// Protocol violation reported instead of fabricated success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverError {
    pub error: String,
}
