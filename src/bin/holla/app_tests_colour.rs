//! Colour as meaning (design review 2026-10, colour follow-ups for holla):
//! each status, count and outcome takes the library role that says what it
//! is, checked on the rendered cells of the real App.

use ratatui::buffer::Cell;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};

use junie_tui::theme::{StatusKind, Theme};

use crate::app_tests::H;
use crate::app_tests_parity::{open, run};
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

fn theme() -> Theme {
    Theme::junie()
}

fn cell(h: &H, x: u16, y: u16) -> Cell {
    h.term.backend().buffer()[(x, y)].clone()
}

/// The cell at the first character of `needle` on row `y`.
fn at_row(h: &H, y: u16, needle: &str) -> (u16, Cell) {
    let row = h.row(y);
    let i = row
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not on row {y}: {row}"));
    let x = row[..i].chars().count() as u16;
    (x, cell(h, x, y))
}

fn until<F: Fn(&H) -> bool>(h: &mut H, max_ticks: usize, f: F) -> bool {
    for _ in 0..max_ticks {
        if f(h) {
            return true;
        }
        h.ticks(1);
    }
    f(h)
}

#[test]
fn a_batch_summary_colours_each_count_by_what_it_says() {
    let t = theme();
    let mut h = H::new(Scenario::ParityGitBatch, Motion::Reduced, 0, 120, 40);
    h.ticks(8);
    run(&mut h, "Pull 4 sibling repositories");
    assert!(
        until(&mut h, 80, |h| h.last_row().contains(" ok · ")),
        "{}",
        h.last_row()
    );
    let y = h.term.backend().buffer().area.height - 1;
    let (_, ok) = at_row(&h, y, " ok · ");
    let (x, _) = at_row(&h, y, " ok · ");
    // the digit before ` ok`
    assert_eq!(cell(&h, x - 1, y).fg, t.success, "{}", h.last_row());
    assert_eq!(ok.fg, t.success);
    let (x, _) = at_row(&h, y, " failed · ");
    assert_eq!(cell(&h, x - 1, y).fg, t.error, "{}", h.last_row());
    assert_eq!(cell(&h, x + 1, y).fg, t.error);
    let (_, cancelled) = at_row(&h, y, "0 cancelled");
    assert_eq!(cancelled.fg, t.text_muted);
    // the failure still marks the message, bold, so mono keeps the weight
    let (_, mark) = at_row(&h, y, "! ");
    assert_eq!(mark.fg, t.error);
    assert!(mark.modifier.contains(Modifier::BOLD));
}

#[test]
fn failure_output_is_primary_text_behind_a_red_gutter() {
    let t = theme();
    let h = H::new(Scenario::LaunchFailure, Motion::Reduced, 30, 120, 40);
    let (x, y) = h
        .find("holla: cannot start")
        .or_else(|| h.find("EADDRINUSE"))
        .unwrap_or_else(|| panic!("{}", h.text()));
    let body = cell(&h, x, y);
    assert_eq!(body.fg, t.text_primary, "{}", h.row(y));
    assert!(!body.modifier.contains(Modifier::BOLD));
    let row = h.row(y);
    let line_start = row[..row.find('!').unwrap_or_else(|| panic!("{row}"))]
        .chars()
        .count() as u16;
    let gutter = cell(&h, line_start, y);
    assert_eq!(gutter.symbol(), "!");
    assert_eq!(gutter.fg, t.error);
    assert!(gutter.modifier.contains(Modifier::BOLD));
}

#[test]
fn a_finished_plan_says_succeeded_in_success_and_failed_in_error() {
    let t = theme();
    let mut h = H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(until(&mut h, 800, |h| h.app.world.plans[0].phase
        == crate::domain::plan::PlanPhase::Done));
    let (x, y) = h
        .find(" failed · ")
        .unwrap_or_else(|| panic!("{}", h.text()));
    assert_eq!(cell(&h, x - 1, y).fg, t.error, "{}", h.row(y));
    let row = h.row(y);
    let i = row.find(" succeeded").unwrap_or_else(|| panic!("{row}"));
    let sx = row[..i].chars().count() as u16;
    assert_eq!(cell(&h, sx + 1, y).fg, t.success, "{row}");
    assert_eq!(cell(&h, sx - 1, y).fg, t.success, "{row}");
    let (nx, ny) = h.find("never started").unwrap();
    assert_eq!(cell(&h, nx, ny).fg, t.text_secondary);
}

#[test]
fn a_required_plan_step_says_why_it_cannot_be_toggled() {
    let t = theme();
    let h = H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 120, 40);
    // the faint marker stays on the row, the word is its lock
    let y = (0..40u16)
        .find(|&y| {
            let r = h.row(y);
            r.contains("[✓]") && r.contains("Preflight")
        })
        .unwrap_or_else(|| panic!("{}", h.text()));
    let row = h.row(y);
    let mark = row[..row.find("[✓]").unwrap()].chars().count() as u16;
    let tick = cell(&h, mark + 1, y).fg;
    assert_ne!(tick, t.accent, "{row}");
    assert_ne!(tick, t.text_primary, "{row}");
    let after = &row[row.find("[✓]").unwrap()..];
    assert!(after.contains("required"), "{row}");
}

#[test]
fn a_trashed_cleanup_record_carries_the_done_status() {
    let t = theme();
    let mut h = H::new(Scenario::ParityCleanupResults, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    open(&mut h, "Show cleanup history");
    let (x, y) = h
        .find("trashed")
        .unwrap_or_else(|| panic!("{}", h.text()));
    let (glyph, style) = t.status(StatusKind::Ok);
    let mark = cell(&h, x - 2, y);
    assert_eq!(mark.symbol(), glyph, "{}", h.row(y));
    assert_eq!(Some(mark.fg), style.fg);
    assert_eq!(cell(&h, x, y).fg, t.success);
}

#[test]
fn a_checked_cleanup_category_is_primary_until_its_list_has_focus() {
    let t = theme();
    let mut h = H::new(Scenario::ParityInsights, Motion::Paused, 40, 120, 40);
    h.app.go(Go::Push(Page::Cleanup { category: None }));
    h.draw();
    let list = crate::screens::cleanup::LIST;
    assert_eq!(h.app.focus.current(), Some(list));
    // a category with every eligible item checked
    let (x, y) = h.find("[✓]").unwrap_or_else(|| panic!("{}", h.text()));
    assert_eq!(cell(&h, x + 1, y).fg, t.accent, "focused: {}", h.row(y));
    h.key(KeyCode::Tab);
    assert_ne!(h.app.focus.current(), Some(list));
    assert_eq!(cell(&h, x + 1, y).fg, t.text_primary, "{}", h.row(y));
}

#[test]
fn the_git_state_in_the_status_bar_is_toned_per_part() {
    let t = theme();
    let h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    let y = h.term.backend().buffer().area.height - 2;
    let row = h.row(y);
    let (_, behind) = at_row(&h, y, "↓");
    assert_eq!(behind.fg, t.info, "{row}");
    let (_, dirty) = at_row(&h, y, "• ");
    assert_eq!(dirty.fg, t.warning, "{row}");
}

#[test]
fn section_headings_are_muted_not_faint() {
    let t = theme();
    let h = H::new(Scenario::DiskCleanup, Motion::Reduced, 0, 120, 40);
    let mut h = h;
    h.ticks(80);
    let (x, y) = h
        .find("Filesystems")
        .unwrap_or_else(|| panic!("{}", h.text()));
    assert_eq!(cell(&h, x, y).fg, t.text_muted);
}

#[test]
fn a_filesystem_meter_warns_at_its_own_thresholds() {
    // `/` is 90% full: attention for a disk (80 / 95), not danger as it
    // would be for a quota (60 / 85)
    let t = theme();
    let mut h = H::new(Scenario::DiskCleanup, Motion::Reduced, 0, 120, 40);
    h.ticks(80);
    let (_, y) = h
        .find("Filesystems")
        .unwrap_or_else(|| panic!("{}", h.text()));
    let row = h.row(y + 1);
    let (_, run) = at_row(&h, y + 1, "━");
    assert_eq!(run.fg, t.warning, "{row}");
    let errors: Vec<Color> = (0..120u16).map(|x| cell(&h, x, y + 1).fg).collect();
    assert!(!errors.contains(&t.error), "{row}");
}

#[test]
fn a_succeeded_activity_tab_is_done() {
    let t = theme();
    let h = H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 120, 40);
    let (glyph, style) = t.status(StatusKind::Ok);
    let row = h.row(2);
    let i = row
        .find(glyph)
        .unwrap_or_else(|| panic!("no finished tab: {row}"));
    let x = row[..i].chars().count() as u16;
    assert_eq!(Some(cell(&h, x, 2).fg), style.fg, "{row}");
}
