//! Capsule screens owned by `jackin-preview-capsule-ui`.

pub mod capsule;
pub mod file_browser;
pub mod inspect;
pub mod op_flow;

pub use file_browser::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
pub use op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
