#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]

pub mod screens;

pub use screens::op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
pub use screens::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
