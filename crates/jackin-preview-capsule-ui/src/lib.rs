#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

pub mod screens;

pub use screens::op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
pub use screens::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
