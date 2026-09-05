//! Route-owned compositions for Jackin overlays.
//!
//! The application shell owns layer lifetime. Runtime-owned focus and dispatch
//! stay outside these modules. They own only the durable domain state needed by
//! a modal, so a redraw cannot mutate a selection or copy credential material
//! out of a provider operation.

pub(crate) mod file_browser;
pub(crate) mod op_flow;
