//! Compatibility exports for the shared shell block.
//!
//! `termrock-shell` owns shell geometry and composition. The facade re-exports
//! those same types so consumers cannot accidentally select a second shell
//! implementation.

pub use termrock_shell::{AppShell, AppShellSlots, Chrome, EmptyLeaf, Regions, ShellSlots};
