//! `jackin-preview-capsule-ui`
//!
//! Capsule-specific detail views, log stream viewer, metrics dashboard.

#![forbid(unsafe_code)]

pub mod capsule;
pub mod editor;
pub mod file_browser;
pub mod inspect;
pub mod op_flow;

pub use capsule::{
    CapsuleFocus, CapsuleInteraction, CapsuleLayer, CapsuleState,
    ExitDecision as CapsuleExitDecision, PrefixCommand,
};
pub use editor::{EditorState, PendingWorkspace, Tab as EditorTab};
pub use file_browser::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
pub use inspect::InspectState;
pub use op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
