#![doc = r" Application shell for the showcase, split out of the legacy app crate."]
#![doc = r""]
#![doc = r" Keeping the application in a library gives integration tests the same"]
#![doc = r" public entry point as downstream consumers and makes the `termrock`"]
#![doc = r" facade boundary compiler-enforced."]

pub mod app;
pub use app::{App, NAV_ENTRIES, NavEntry, PageId};

#[doc = r" Run the showcase with command-line theme and colour selection."]
#[doc = r""]
#[doc = r" # Errors"]
#[doc = r""]
#[doc = r" Returns terminal setup or teardown errors from the `termrock` runtime."]
pub fn run() -> std::io::Result<()> {
    app::run()
}
