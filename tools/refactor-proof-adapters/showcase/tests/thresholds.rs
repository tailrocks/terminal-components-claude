//! Shell geometry boundaries: minimum size, sidebar, inspector, compact rows.
//!
//! Thresholds come from `docs/refactoring-plan/showcase.md` and are observed
//! through the production App: sidebar 19 columns below 110 else 24,
//! inspector 30 columns only when enabled and width >= 100, minimum 72x20,
//! compact navigation below height 32.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]

use junie_tui::KeyCode;
use oracle_showcase::{ColorSpec, OraclePageId, ShowcaseDriver, TerminalSize};

fn open(size: TerminalSize) -> ShowcaseDriver {
    ShowcaseDriver::open(OraclePageId::Overview, size, ColorSpec::TrueColor).unwrap()
}

fn rows(text: &str) -> Vec<&str> {
    text.lines().collect()
}

#[test]
fn minimum_size_shows_notice_below_72x20_only() {
    for size in [TerminalSize::new(71, 20), TerminalSize::new(72, 19)] {
        let mut driver = open(TerminalSize::new(80, 24));
        driver.resize(size).unwrap();
        let frame = driver.observe().unwrap();
        assert!(frame.is_complete());
        let text = frame.text();
        assert!(
            text.contains("Terminal too small"),
            "undersized {} must show the reduced notice\n{text}",
            size.token()
        );
        driver.key_code(KeyCode::Char('q')).unwrap();
        assert!(
            driver.observe().unwrap().quit,
            "q must quit from undersized {}",
            size.token()
        );
    }

    let mut driver = open(TerminalSize::new(80, 24));
    driver.resize(TerminalSize::new(72, 20)).unwrap();
    let frame = driver.observe().unwrap();
    assert!(frame.is_complete());
    assert!(!frame.quit);
    let text = frame.text();
    assert!(
        !text.contains("Terminal too small"),
        "72x20 is the minimum supported shell\n{text}"
    );
    assert_eq!(frame.production_page, Some(OraclePageId::Overview));
}

#[test]
fn sidebar_threshold_109_110_keeps_sidebar_and_shifts_main() {
    let narrow = open(TerminalSize::new(109, 30)).observe().unwrap();
    let wide = open(TerminalSize::new(110, 30)).observe().unwrap();
    assert!(narrow.is_complete());
    assert!(wide.is_complete());
    assert_eq!(narrow.production_page, Some(OraclePageId::Overview));
    assert_eq!(wide.production_page, Some(OraclePageId::Overview));

    let narrow_text = narrow.text();
    let wide_text = wide.text();
    let narrow_rows = rows(&narrow_text);
    let wide_rows = rows(&wide_text);
    assert_eq!(narrow_rows.len(), 30);
    assert_eq!(wide_rows.len(), 30);
    // Body rows keep the same 19-column sidebar prefix; the main area starts
    // at sidebar+2, so full rows differ once the sidebar widens to 24.
    for y in 2..28 {
        let left: String = narrow_rows[y].chars().take(19).collect();
        let right: String = wide_rows[y].chars().take(19).collect();
        assert_eq!(left, right, "sidebar prefix must match on body row {y}");
    }
    assert_ne!(
        narrow.text(),
        wide.text(),
        "109 vs 110 columns must change shell geometry"
    );
}

#[test]
fn inspector_threshold_99_100_applies_only_when_enabled() {
    let off_100 = open(TerminalSize::new(100, 30)).observe().unwrap().text();

    let mut toggled = open(TerminalSize::new(100, 30));
    toggled.key_code(KeyCode::Char('i')).unwrap();
    let on_100 = toggled.observe().unwrap().text();
    assert_ne!(
        off_100, on_100,
        "'i' must toggle the state inspector at width 100"
    );

    toggled.resize(TerminalSize::new(99, 30)).unwrap();
    let on_99 = toggled.observe().unwrap().text();
    let off_99 = open(TerminalSize::new(99, 30)).observe().unwrap().text();
    // Row 0 is the header hint chip, which shows the toggle state; the panel
    // geometry lives in the body and footer rows.
    assert_eq!(
        rows(&on_99)[1..],
        rows(&off_99)[1..],
        "inspector stays hidden below width 100 even when enabled"
    );

    toggled.resize(TerminalSize::new(100, 30)).unwrap();
    assert_eq!(
        toggled.observe().unwrap().text(),
        on_100,
        "inspector must reappear on resize back to 100"
    );
}

#[test]
fn compact_rows_31_32_keep_navigation_observable() {
    for rows in [31, 32] {
        let driver = open(TerminalSize::new(80, rows));
        let frame = driver.observe().unwrap();
        assert!(frame.is_complete());
        assert!(!frame.quit);
        assert_eq!(frame.production_page, Some(OraclePageId::Overview));
        assert!(
            !frame.hits.is_empty(),
            "shell must publish hit regions at height {rows}"
        );
    }
}
