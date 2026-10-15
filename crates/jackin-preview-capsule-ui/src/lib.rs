//! Route-owned compositions for Jackin overlays: capsule screens
//! (capsule, inspect, file browser, operation flow).

#![forbid(unsafe_code)]

pub mod capsule;
pub mod file_browser;
pub mod inspect;
pub mod op_flow;

pub use file_browser::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
pub use op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
