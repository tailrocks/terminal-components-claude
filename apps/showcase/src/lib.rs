//! The public component showcase application.
//!
//! The binary is intentionally a thin wrapper. Keeping the application in a
//! library gives integration tests the same public entry point as downstream
//! consumers and makes the `junie-tui` facade boundary compiler-enforced.

#![expect(
    clippy::arithmetic_side_effects,
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::collapsible_if,
    clippy::elidable_lifetime_names,
    clippy::needless_update,
    clippy::obfuscated_if_else,
    clippy::redundant_closure_for_method_calls,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "historical compatibility pages keep bounded fixture arithmetic and composition together"
)]

mod app;
mod data;
mod pages;

pub use app::{App, NAV_ENTRIES, NavEntry, PageId};

/// Run the showcase with command-line theme and colour selection.
///
/// # Errors
///
/// Returns terminal setup or teardown errors from the `junie-tui` runtime.
pub fn run() -> std::io::Result<()> {
    app::run()
}
