//! Regression cover for the Showcase shell chrome.
//!
//! The chrome is composed from shared Termrock blocks (`AppShell` for the
//! chrome regions, the shared layout plan for the body split). These tests
//! pin the rendered result so the composition cannot drift, and they pin the
//! routing so a screen is opened by its route instead of a display label.

use showcase_ui::{App, NAV_ENTRIES, PageId, Route};
use termrock::{Color, ColorLevel, KeyCode, Theme};
use termrock_test_support::Harness;

/// Frame digests captured from the app-owned shell chrome before the body
/// split moved to the shared layout plan.
///
/// The digest covers every cell's symbol, foreground, background and
/// modifiers, so an equality proves the migration rendered the same frame.
const CHROME_FRAMES: &[(PageId, u16, u16, bool, &str, u64)] = &[
    (
        PageId::Overview,
        80,
        24,
        false,
        "truecolor",
        0x80780cbb_ac55284d,
    ),
    (
        PageId::Overview,
        100,
        30,
        false,
        "truecolor",
        0x3cd4908e_316383aa,
    ),
    (
        PageId::Buttons,
        120,
        40,
        false,
        "truecolor",
        0xdcd0198f_4e49fff0,
    ),
    (
        PageId::Buttons,
        120,
        40,
        true,
        "truecolor",
        0xc4df8e1e_f62cea73,
    ),
    (PageId::Overview, 100, 30, false, "16", 0x297069b2_b57e2db6),
];

fn frame_digest(page: PageId, width: u16, height: u16, inspector: bool, level: ColorLevel) -> u64 {
    let mut harness =
        Harness::new(App::with_page(page), Theme::junie(), width, height).with_color(level);
    if inspector {
        let _ = harness.key(KeyCode::Char('i'));
    }
    harness.snapshot().named("shell-chrome").digest()
}

#[test]
fn shell_chrome_frames_are_unchanged_by_the_shared_layout() {
    for (page, width, height, inspector, level, expected) in CHROME_FRAMES {
        let level = match *level {
            "truecolor" => ColorLevel::TrueColor,
            "16" => ColorLevel::Ansi16,
            other => panic!("unknown colour level {other}"),
        };
        let digest = frame_digest(*page, *width, *height, *inspector, level);
        assert_eq!(
            digest, *expected,
            "{page:?} {width}x{height} inspector={inspector} {level:?}"
        );
    }
}

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
fn selection_screen_is_opened_by_its_route() {
    let selection = NAV_ENTRIES
        .iter()
        .find(|entry| entry.label == "Selection")
        .expect("the selection entry");
    assert_eq!(selection.route, Route::Selection);
    assert_eq!(selection.page(), None);

    let harness = Harness::new(App::for_page_name("selection"), Theme::junie(), 100, 30);
    assert!(
        harness.text().contains("Dana Whitfield"),
        "the selection screen renders its member table"
    );
}

#[test]
fn every_navigation_entry_draws_the_screen_its_route_names() {
    for (index, entry) in NAV_ENTRIES.iter().enumerate() {
        let harness = Harness::new(App::for_page_name(entry.label), Theme::junie(), 100, 30);
        let text = harness.text();
        match entry.route {
            Route::Page(page) => {
                assert!(
                    text.contains(page.title()),
                    "entry {index} ({}) draws the {} page",
                    entry.label,
                    page.title()
                );
            }
            Route::Selection => {
                assert!(
                    text.contains("Dana Whitfield"),
                    "entry {index} ({}) draws the selection screen",
                    entry.label
                );
            }
            Route::Placeholder => {
                assert!(
                    text.contains(entry.label),
                    "entry {index} draws its placeholder label"
                );
            }
        }
    }
}
