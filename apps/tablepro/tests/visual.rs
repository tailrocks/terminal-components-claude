//! Deterministic facade rendering smoke test.

use tablepro_app::TableProApp;
use tui_next::Theme;
use tui_next_testing::Harness;

#[test]
fn tablepro_visual_baseline() {
    let first = Harness::new(TableProApp::default(), Theme::junie(), 120, 40)
        .snapshot()
        .key();
    let second = Harness::new(TableProApp::default(), Theme::junie(), 120, 40)
        .snapshot()
        .key();

    assert_eq!(first, second);
}
