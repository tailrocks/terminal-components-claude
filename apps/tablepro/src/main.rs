//! `TablePro` binary entry point.

use tablepro_app::TableProApp;

fn main() -> std::io::Result<()> {
    tui_next::run(TableProApp::default(), tui_next::Theme::junie())
}
