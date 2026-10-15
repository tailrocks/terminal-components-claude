//! Route-owned compositions for Jackin overlays: host chrome screens
//! (manager, editor, accounts, cockpit, settings, usage).
//!
//! The application shell owns layer lifetime and focus.  These modules own
//! only the durable domain state needed by a modal, so a redraw cannot mutate
//! a selection or copy credential material out of a provider operation.

#![forbid(unsafe_code)]

pub mod accounts;
pub mod cockpit;
pub mod editor;
pub mod manager;
pub mod manager_actions;
pub mod settings;
pub mod usage;
