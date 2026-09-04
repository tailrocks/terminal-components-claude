//! Route-owned compositions for Jackin overlays.
//!
//! The application shell owns layer lifetime and focus.  These modules own
//! only the durable domain state needed by a modal, so a redraw cannot mutate
//! a selection or copy credential material out of a provider operation.

pub mod file_browser;
pub mod op_flow;

pub use file_browser::{FileBrowserAction, FileBrowserEntry, FileBrowserState};
pub use op_flow::{OpFlowAction, OpFlowStage, OpFlowState, OpFlowStatus};
