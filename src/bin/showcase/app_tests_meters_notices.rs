//! The Feedback pages Meters and Notices through the real app (plan §7
//! SC36): each page shows its library component, not the planned-page
//! stub; options set through `--opt` change the playground, the States
//! rows render, and the key interactions work.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

// ------------------------------------------------------------------ Meters

#[test]
fn meters_page_shows_its_lists_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        let h = Harness::new(w, h, PageId::Meters);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in ["Plan usage", "Session (5 h)", "Refresh", "Offline"] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let text = Harness::new(120, 40, PageId::Meters).text();
    for needle in ["Filesystems", "/Volumes/Backup", "Host", "Memory"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn meters_line_up_in_columns_and_keep_their_level_glyphs() {
    let h = Harness::new(120, 40, PageId::Meters);
    // every quota row's meter starts in the same column
    let rows = ["Session (5 h)", "Weekly", "Opus weekly", "Extra credits"];
    let (lx, _) = h.at("Session (5 h)");
    let starts: Vec<u16> = rows
        .iter()
        .map(|r| {
            let (x, y) = h.at(r);
            assert_eq!(x, lx, "labels align");
            // the block meter: the first cell with a different background
            (x..x + 40)
                .find(|&cx| h.cell(cx, y).bg != h.cell(x, y).bg)
                .expect("a filled meter")
        })
        .collect();
    assert!(starts.windows(2).all(|w| w[0] == w[1]), "{starts:?}");
    let (_, y) = h.at("Extra credits");
    assert!(
        h.row(y).contains('!'),
        "exhausted keeps its glyph: {}",
        h.row(y)
    );
    let (_, y) = h.at("/Volumes/Backup");
    assert!(h.row(y).contains("97% !"), "{}", h.row(y));
    assert!(h.row(y).contains("1.94 of 2 TB"));
}

#[test]
fn meters_options_drive_the_list_builders() {
    let mut h = Harness::new(120, 40, PageId::Meters);
    let (_, y) = h.at("Session (5 h)");
    assert!(h.row(y).contains("resets in 2 h"));
    h.opt("meters.detail=off");
    assert!(!h.text().contains("resets in 2 h"));
    h.opt("meters.detail=on");
    h.opt("meters.label_width=8");
    assert!(h.text().contains("Session…"), "{}", h.text());
    h.opt("meters.label_width=measured");
    let (x, y) = h.at("resets in 2 h");
    h.opt("meters.detail_tone=secondary");
    assert_eq!(h.cell(x, y).fg, Theme::junie().text_secondary);
    h.opt("meters.meter_width=14");
    assert_ne!(
        h.at("resets in 2 h").0,
        x,
        "a narrower meter moves the detail"
    );
    h.opt("meters.label_gap=3");
    h.opt("meters.visual=line");
    let (_, y) = h.at("Weekly");
    assert!(h.row(y).contains('━'), "{}", h.row(y));
}

#[test]
fn offline_keeps_last_good_values_and_refresh_shows_refreshing() {
    let mut h = Harness::new(120, 40, PageId::Meters);
    h.click_id(WidgetId::of("meters").sub("refresh"));
    assert!(h.text().contains("refreshing"), "{}", h.text());
    for _ in 0..30 {
        h.tick();
    }
    assert!(!h.text().contains("refreshing…"));
    assert_eq!(h.status(), "Usage refreshed ✓");
    // the keyboard toggles Offline: the values stay, drawn stale (dashed)
    h.focus(WidgetId::of("meters").sub("offline"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("offline · last-good values"));
    let (_, y) = h.at("Weekly");
    assert!(h.row(y).contains('%'));
    let (_, y) = h.at("/System/Volumes/Data");
    assert!(h.row(y).contains('╍'), "{}", h.row(y));
    // a refresh while offline says why nothing happens
    h.click_id(WidgetId::of("meters").sub("refresh"));
    assert!(h.status().starts_with("Offline"), "{}", h.status());
}

#[test]
fn meters_states_grid_shows_every_content_state() {
    let mut h = Harness::new(120, 40, PageId::Meters);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Quota list",
        "Disk list",
        "rest",
        "loading",
        "error",
        "overflow",
        "narrow",
        "read failed",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}
