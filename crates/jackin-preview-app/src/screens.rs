//! Aggregated screen definitions from host-ui, capsule-ui, and presentation crates.

pub use jackin_preview_capsule_ui::{capsule, editor, file_browser, inspect, op_flow};
pub use jackin_preview_host_ui::{accounts, cockpit, manager, manager_actions, settings, usage};
pub use jackin_preview_presentation::prelude;

pub use file_browser::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
pub use op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
