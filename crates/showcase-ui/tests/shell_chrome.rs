use showcase_ui::{App, PageId};
use termrock::{Color, Id, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const HEADER_INSPECT: Id = Id::root("showcase_ui::app::header.inspect");

#[test]
fn inspector_dock_keeps_one_clear_column_after_the_main_area() {
    let mut harness = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Char('i'));
    let canvas = Color::Rgb(0, 0, 0);
    // The main area ends at column 88 and the dock starts at column 90, so the
    // column before the dock stays clear of both.
    assert_eq!(harness.cell(88, 4).bg, canvas, "column 88 stays clear");
    assert_eq!(harness.cell(89, 4).bg, canvas, "column 89 stays clear");
    assert_ne!(
        harness.cell(90, 4).bg,
        canvas,
        "the inspector dock starts at column 90"
    );
}

#[test]
fn shell_chrome_hit_regions_are_unchanged() {
    let harness = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    // The header actions keep their parts at the right edge of the header.
    let help = harness
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("help part");
    assert_eq!(help, ratatui::layout::Rect::new(111, 0, 8, 1));
    let inspect = harness
        .area_of_part(HEADER_INSPECT, PartRef::of(Part::LABEL))
        .expect("inspector part");
    assert_eq!(inspect, ratatui::layout::Rect::new(97, 0, 13, 1));
    // The navigation tree keeps the sidebar rail inside its one-column inset.
    let nav = harness.area_of(NAV).expect("navigation tree");
    assert_eq!(nav, ratatui::layout::Rect::new(1, 4, 23, 33));
}
