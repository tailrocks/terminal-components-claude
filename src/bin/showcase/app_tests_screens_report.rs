//! The Screens pages Read-only report (SC56), Launch cockpit (SC55) and
//! Accounts & usage (SC53) through the real app: each page shows its
//! composed library widgets, not the planned-page stub; options set
//! through `--opt` change the playground, and the key interactions work.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;
use junie_tui::core::id::WidgetId;

// --------------------------------------------------------- Read-only report

const REPORT: WidgetId = WidgetId::of("readonlyreport");

#[test]
fn report_shows_its_snapshot_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        let h = Harness::new(w, h, PageId::ReadOnlyReport);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in ["Host snapshot", "live · 12 s ago", "CPU", "Memory"] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let text = Harness::new(120, 40, PageId::ReadOnlyReport).text();
    for needle in [
        "8 cores · load",
        "Facts",
        "build-07.acme.internal",
        "/Volumes/Backup is 97 % full",
        "This report is read-only",
        "Copy report",
        "Disk cleanup →",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn report_r_refreshes_and_the_meta_says_so() {
    let mut h = Harness::new(120, 40, PageId::ReadOnlyReport);
    h.focus(REPORT.sub("facts"));
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("refreshing…"), "{}", h.text());
    for _ in 0..20 {
        h.tick();
    }
    assert!(h.text().contains("live · just now"), "{}", h.text());
    assert_eq!(h.status(), "Snapshot refreshed ✓");
}

#[test]
fn report_offline_keeps_last_good_values_and_refuses_refresh() {
    let mut h = Harness::new(120, 40, PageId::ReadOnlyReport);
    h.opt("readonlyreport.offline=on");
    let text = h.text();
    assert!(text.contains("offline · last-good 2 m ago"), "{text}");
    assert!(text.contains("Offline: showing the last-good snapshot"));
    h.click_id(REPORT.sub("refresh"));
    assert!(h.status().contains("does not answer"), "{}", h.status());
    assert!(!h.text().contains("refreshing…"));
}

#[test]
fn report_copies_a_fact_and_the_whole_report() {
    let mut h = Harness::new(120, 40, PageId::ReadOnlyReport);
    h.focus(REPORT.sub("facts"));
    h.key(KeyCode::Char('y'));
    assert_eq!(h.status(), "Copied Host");
    h.click_id(REPORT.sub("copy"));
    assert!(
        h.status().starts_with("Copied the report ("),
        "{}",
        h.status()
    );
    h.opt("readonlyreport.copyable=off");
    h.focus(REPORT.sub("facts"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Address: 10.4.12.37", "not copyable: shown");
}

#[test]
fn report_options_drive_the_widgets() {
    let mut h = Harness::new(120, 40, PageId::ReadOnlyReport);
    h.opt("readonlyreport.notices=off");
    assert!(!h.text().contains("97 % full"));
    h.opt("readonlyreport.visual=line");
    let (_, y) = h.at("Swap");
    assert!(h.row(y).contains('─'), "a line meter: {}", h.row(y));
    h.section(Section::States);
    let text = h.text();
    for needle in ["Meters", "Facts", "loading", "narrow"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}
