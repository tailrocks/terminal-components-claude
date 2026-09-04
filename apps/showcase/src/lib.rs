//! The public component showcase application.
//!
//! The binary is intentionally a thin wrapper. Keeping the application in a
//! library gives integration tests the same public entry point as downstream
//! consumers and makes the `tui-next` facade boundary compiler-enforced.

mod app;
mod data;
mod pages;

pub use app::{App, NAV_ENTRIES, NavEntry, PageId};

/// Run the showcase with command-line theme and colour selection.
///
/// # Errors
///
/// Returns terminal setup or teardown errors from the `tui-next` runtime.
pub fn run() -> std::io::Result<()> {
    app::run()
}
