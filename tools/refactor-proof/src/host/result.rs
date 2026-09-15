//! Host stdout result envelope (`tc-proof-host-result/v1`).

use serde::{Deserialize, Serialize};

use super::HostOperation;

/// Host command outcome status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Passed,
    Rejected,
}

/// Single JSON object emitted on stdout per host invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostResult {
    pub schema: HostResultSchema,
    pub operation: HostOperationWire,
    pub status: HostStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Wire representation of [`HostOperation`] in host results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostOperationWire {
    Install,
    Prepare,
    Freeze,
    Verify,
    Seal,
    Integrate,
}

impl From<HostOperation> for HostOperationWire {
    fn from(value: HostOperation) -> Self {
        match value {
            HostOperation::Install => Self::Install,
            HostOperation::Prepare => Self::Prepare,
            HostOperation::Freeze => Self::Freeze,
            HostOperation::Verify => Self::Verify,
            HostOperation::Seal => Self::Seal,
            HostOperation::Integrate => Self::Integrate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostResultSchema {
    #[serde(rename = "tc-proof-host-result/v1")]
    V1,
}

impl HostResult {
    pub fn rejected(operation: HostOperation, category: impl Into<String>) -> Self {
        Self {
            schema: HostResultSchema::V1,
            operation: operation.into(),
            status: HostStatus::Rejected,
            category: Some(category.into()),
        }
    }
}
