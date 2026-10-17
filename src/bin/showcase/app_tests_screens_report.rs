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

// ----------------------------------------------------------- Launch cockpit

const COCKPIT: WidgetId = WidgetId::of("launchcockpit");

fn ticks(h: &mut Harness, n: usize) {
    for _ in 0..n {
        h.tick();
    }
}

#[test]
fn cockpit_shows_rail_log_and_actions_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        let h = Harness::new(w, h, PageId::LaunchCockpit);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in [
            "Launch · payments-platform",
            "0 of 11 stages",
            "Credentials",
        ] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let text = Harness::new(120, 40, PageId::LaunchCockpit).text();
    for needle in [
        "Stages",
        "Derived Image",
        "Hardline",
        "Build log",
        "not started",
        "Launch",
        "Cancel launch…",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn cockpit_launch_runs_every_stage_to_a_ready_capsule() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    h.click_id(COCKPIT.sub("launch"));
    ticks(&mut h, 7);
    assert!(h.text().contains("Resolving credentials…"), "{}", h.text());
    ticks(&mut h, 80);
    let text = h.text();
    assert!(text.contains("capsule ready"), "{text}");
    assert!(text.contains("11 of 11 stages"), "{text}");
    assert!(
        text.contains("cached"),
        "the skipped stage says why: {text}"
    );
    assert!(text.contains("Launch again"), "{text}");
    assert_eq!(h.status(), "Launched ✓ capsule cap-payments-01");
}

#[test]
fn cockpit_credentials_failure_holds_until_retry() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    h.opt("launchcockpit.outcome=credentials");
    h.click_id(COCKPIT.sub("launch"));
    ticks(&mut h, 12);
    let text = h.text();
    assert!(text.contains("Credential source unavailable"), "{text}");
    assert!(text.contains("vault Engineering is locked"), "{text}");
    // held: ticks do not move the launch on
    ticks(&mut h, 10);
    assert!(h.text().contains("Credential source unavailable"));
    let (x, y) = h.at("Retry");
    h.click(x + 1, y);
    assert!(
        !h.text().contains("Credential source unavailable"),
        "{}",
        h.text()
    );
    ticks(&mut h, 80);
    assert!(h.text().contains("capsule ready"), "{}", h.text());
}

#[test]
fn cockpit_build_failure_opens_the_facts_dialog() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    h.opt("launchcockpit.outcome=build-failure");
    h.click_id(COCKPIT.sub("launch"));
    ticks(&mut h, 80);
    let text = h.text();
    assert!(text.contains("Docker build failed"), "{text}");
    assert!(text.contains("run-7f3a91c2"), "{text}");
    let (x, y) = h.at("Launch again");
    h.click(x + 1, y);
    assert!(!h.text().contains("Docker build failed"), "{}", h.text());
    assert!(h.text().contains("jackin launch backend"), "relaunched");
}

#[test]
fn cockpit_cancel_asks_then_stops_the_launch() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    h.click_id(COCKPIT.sub("launch"));
    ticks(&mut h, 4);
    h.click_id(COCKPIT.sub("cancel"));
    assert!(h.text().contains("Cancel launch?"), "{}", h.text());
    let (_, y) = h.at("Keep launching");
    let row = h.row(y);
    let i = row.rfind("Cancel launch").expect("the confirm button");
    let x = row[..i].chars().count() as u16;
    h.click(x + 1, y);
    assert!(!h.text().contains("Cancel launch?"), "{}", h.text());
    assert_eq!(h.status(), "Launch cancelled");
    assert!(h.text().contains("cancelled ·"), "{}", h.text());
}

/// The key path the overlay scenarios take: Launch is the fourth stop
/// (strip, facts, log, Launch) and Cancel launch… the fifth.
#[test]
fn cockpit_keyboard_reaches_launch_and_cancel() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    for _ in 0..4 {
        h.key(KeyCode::Tab);
    }
    assert!(h.app.shell.focus.is(COCKPIT.sub("launch")));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    assert!(h.app.shell.focus.is(COCKPIT.sub("cancel")));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Cancel launch?"), "{}", h.text());
}

#[test]
fn cockpit_options_drive_the_rail_and_states() {
    let mut h = Harness::new(120, 40, PageId::LaunchCockpit);
    h.opt("launchcockpit.numbered=on");
    assert!(h.text().contains("01 Identity"), "{}", h.text());
    h.section(Section::States);
    let text = h.text();
    for needle in ["Stage rail", "Failed launch", "Build log"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}
