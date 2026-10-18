use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(super) const OBSERVER_STEP_ORDER: [ObserverStep; 3] = [
    ObserverStep::Build,
    ObserverStep::Test,
    ObserverStep::Taskfmt,
];

pub struct ObserverEnv;

impl ObserverEnv {
    pub const REQUEST_FD: &'static str = "TC_PROOF_OBSERVER_REQUEST_FD";
    pub const RESPONSE_FD: &'static str = "TC_PROOF_OBSERVER_RESPONSE_FD";
    pub const NONCE: &'static str = "TC_PROOF_OBSERVER_NONCE";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    #[serde(rename = "tc-proof-observer-request/v1")]
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverRequest {
    pub schema: ObserverRequestSchema,
    pub nonce: String,
    pub step: ObserverStep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "schema", rename_all = "kebab-case")]
pub enum ObserverResponse {
    #[serde(rename = "tc-proof-observation/v1")]
    Observation(ObserverObservation),
    #[serde(rename = "tc-proof-observer-error/v1")]
    Error(ObserverError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverObservation {
    pub step: ObserverStep,
    pub tree: String,
    pub argv: Vec<String>,
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverError {
    pub error: String,
}
