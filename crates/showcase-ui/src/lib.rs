//! Application shell for the showcase, split out of the legacy app crate.
//!
//! Keeping the application in a library gives integration tests the same
//! public entry point as downstream consumers and makes the `termrock`
//! facade boundary compiler-enforced.

#![expect(
    clippy::arithmetic_side_effects,
    clippy::too_many_lines,
    reason = "historical compatibility pages keep bounded fixture arithmetic and composition together"
)]

pub mod app;
pub use app::{App, NAV_ENTRIES, NavEntry, PageId};

/// Run the showcase with command-line theme and colour selection.
///
/// # Errors
///
/// Returns terminal setup or teardown errors from the `termrock` runtime.
pub fn run() -> std::io::Result<()> {
    app::run()
}
