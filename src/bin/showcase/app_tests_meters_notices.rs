




use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;



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

    let rows = ["Session (5 h)", "Weekly", "Opus weekly", "Extra credits"];
    let (lx, _) = h.at("Session (5 h)");
    let starts: Vec<u16> = rows
        .iter()
        .map(|r| {
            let (x, y) = h.at(r);
            assert_eq!(x, lx, "labels align");

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

    h.focus(WidgetId::of("meters").sub("offline"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("offline · last-good values"));
    let (_, y) = h.at("Weekly");
    assert!(h.row(y).contains('%'));
    let (_, y) = h.at("/System/Volumes/Data");
    assert!(h.row(y).contains('╍'), "{}", h.row(y));

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



#[test]
fn notices_page_shows_every_kind_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        let h = Harness::new(w, h, PageId::Notices);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in ["! Authentication failed", "Test connection"] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let h = Harness::new(120, 40, PageId::Notices);
    let text = h.text();
    for needle in [
        "syntax error at or near",
        "! Port must be a number",
        "3 rows changed outside this session",
        "Read-only replica",
        "Saved 12 rows in 84 ms",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }

    let (x, y) = h.at("! Authentication failed");
    let t = Theme::junie();
    assert_eq!(h.cell(x, y).fg, t.error);
    assert!(
        h.cell(x, y)
            .modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
    assert!(h.row(y + 1).contains("FATAL"), "{}", h.row(y + 1));
    assert_eq!(h.at("FATAL").0, x + 2);
}

#[test]
fn notices_options_drive_the_notice_builders() {
    let mut h = Harness::new(120, 40, PageId::Notices);
    assert!(h.text().contains("Enter on the result tab"));
    h.opt("notices.hint=off");
    assert!(!h.text().contains("Enter on the result tab"));
    assert!(h.text().contains("LINE 1"));
    h.opt("notices.detail=off");
    assert!(!h.text().contains("LINE 1"));
    h.opt("notices.detail=on");

    h.opt("notices.width=narrow");
    let (_, y) = h.at("LINE 1");
    assert!(h.row(y + 1).trim_end().ends_with('…'), "{}", h.row(y + 1));
    h.opt("notices.mark_cut=off");
    assert!(!h.row(y + 1).trim_end().ends_with('…'), "{}", h.row(y + 1));
    h.opt("notices.max_lines=1");
    assert!(!h.row(y + 1).contains("created_at"), "{}", h.row(y + 1));
    h.opt("notices.detail_tone=muted");
    let (x, y) = h.at("LINE 1");
    assert_eq!(h.cell(x, y).fg, Theme::junie().text_muted);
    h.opt("notices.kind=warning");
    let (x, y) = h.at("syntax error");
    assert_eq!(h.cell(x, y).fg, Theme::junie().warning);
}

#[test]
fn test_connection_walks_the_outcomes() {
    let mut h = Harness::new(120, 40, PageId::Notices);
    h.click_id(WidgetId::of("notices").sub("test"));
    assert!(
        h.text().contains("Could not reach the host"),
        "{}",
        h.text()
    );
    h.focus(WidgetId::of("notices").sub("test"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("✓ Connected in 182 ms"), "{}", h.text());
    assert_eq!(h.status(), "Connected in 182 ms");
}

#[test]
fn notices_states_grid_shows_kinds_and_the_cut() {
    let mut h = Harness::new(120, 40, PageId::Notices);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "With detail",
        "With hint",
        "rest",
        "error",
        "overflow",
        "narrow",
        "…",
    ] {
        assert!(text.contains(label), "{label}: {text}");
    }
}
