pub mod app;
pub use app::{App, NAV_ENTRIES, NavEntry, PageId};

pub fn run() -> std::io::Result<()> {
    app::run()
}
