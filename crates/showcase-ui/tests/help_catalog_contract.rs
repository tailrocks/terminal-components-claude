use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAVIGATION: Id = Id::root("showcase_ui::app::navigation");
const HELP: Id = Id::root("showcase_ui::app::help");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");

fn assert_help_content_is_visible(harness: &Harness<App>) {
    assert_eq!(NAV_ENTRIES.len(), 54);

    let expected = format!("Navigation: {} destinations.", NAV_ENTRIES.len());
    let screen = harness.text();
    assert!(screen.contains(&expected), "missing {expected:?}\n{screen}");
    for instruction in [
        "Tab / Shift+Tab   move keyboard focus",
        "↑ ↓ ← →           move inside the focused control",
        "Enter / Space     activate · start editing",
        "Esc               cancel editing · back to navigation",
        "[ ]               previous / next page",
        "0                 jump to navigation",
        "i                 toggle state inspector",
        "q                 quit",
    ] {
        assert!(
            screen.contains(instruction),
            "missing keyboard instruction {instruction:?}\n{screen}"
        );
    }
    for instruction in [
        "Mouse: hover to preview, click to focus and activate, wheel to",
        "scroll, drag the scrollbar thumb.",
        "Mouse: hover to preview",
        "click to focus and activate",
        "drag the scrollbar thumb.",
    ] {
        assert!(
            screen.contains(instruction),
            "missing mouse instruction segment {instruction:?}\n{screen}"
        );
    }
    assert!(
        screen.contains("scrollbar thumb."),
        "the final Help instruction remains visible\n{screen}"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn keyboard_help_shows_live_destination_count_and_instructions_at_supported_sizes() {
    for (width, height) in [(72, 20), (120, 40)] {
        let mut harness = Harness::new(App::new(), Theme::junie(), width, height);
        assert_eq!(harness.focus(), Some(NAVIGATION));
        let closed_screen = harness.text();
        let previous_focus = harness.focus();

        let _ = harness.key(KeyCode::Char('?'));

        assert!(harness.is_open(HELP), "Help opens at {width}x{height}");
        assert_help_content_is_visible(&harness);
        assert!(
            harness.text().contains("Keyboard & mouse"),
            "Help title is visible at {width}x{height}"
        );

        let _ = harness.key(KeyCode::Esc);

        assert!(!harness.is_open(HELP));
        assert_eq!(harness.focus(), previous_focus);
        assert_eq!(harness.text(), closed_screen);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn header_help_shows_live_destination_count_and_instructions_at_supported_sizes() {
    for (width, height) in [(72, 20), (120, 40)] {
        let mut harness = Harness::new(App::new(), Theme::junie(), width, height);
        assert_eq!(harness.focus(), Some(NAVIGATION));
        let closed_screen = harness.text();
        let previous_focus = harness.focus();
        let help = harness
            .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
            .expect("the header Help button remains available");

        let _ = harness.click(help.x.saturating_add(1), help.y);

        assert!(
            harness.is_open(HELP),
            "header Help opens at {width}x{height}"
        );
        assert_help_content_is_visible(&harness);

        let _ = harness.key(KeyCode::Esc);

        assert!(!harness.is_open(HELP));
        assert_eq!(harness.focus(), previous_focus);
        assert_eq!(harness.text(), closed_screen);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn help_count_stays_tied_to_the_catalog_after_query_and_route_changes() {
    let mut harness = Harness::new(App::new(), Theme::junie(), 120, 40);

    let _ = harness.key(KeyCode::Char('/'));
    let _ = harness.type_str("tables");
    assert!(
        format!("{:?}", harness.app()).contains("nav_query: Some(NavQuery { text: \"tables\""),
        "the test entered a real navigation query: {:?}",
        harness.app()
    );
    let help = harness
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("header Help remains available during a query");
    let _ = harness.click(help.x.saturating_add(1), help.y);
    assert!(harness.is_open(HELP));
    assert_help_content_is_visible(&harness);
    assert!(
        format!("{:?}", harness.app()).contains("nav_query: None"),
        "opening Help closes the active query before modal presentation"
    );
    let _ = harness.key(KeyCode::Esc);

    let _ = harness.key(KeyCode::Char(']'));
    assert_eq!(harness.app().page(), PageId::Chrome);
    let _ = harness.key(KeyCode::Char('?'));
    assert!(harness.is_open(HELP));
    assert_help_content_is_visible(&harness);
}
