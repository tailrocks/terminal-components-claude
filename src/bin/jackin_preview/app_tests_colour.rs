//! Colour as meaning (design review 2026-10, colour follow-ups for
//! jackin): each status, count and health part takes the library role
//! that says what it is, checked on the rendered cells of the real App.

use ratatui::buffer::Cell;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use junie_tui::theme::{StatusKind, Theme};

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

fn theme() -> Theme {
    Theme::junie()
}

fn cell(h: &H, x: u16, y: u16) -> Cell {
    h.term.backend().buffer()[(x, y)].clone()
}

fn row(h: &H, y: u16) -> String {
    h.text().lines().nth(y as usize).unwrap_or("").to_owned()
}

/// The first row holding `needle`, and the cell column where it starts.
fn find(h: &H, needle: &str) -> (u16, u16) {
    let text = h.text();
    for (y, line) in text.lines().enumerate() {
        if let Some(i) = line.find(needle) {
            return (line[..i].chars().count() as u16, y as u16);
        }
    }
    panic!("{needle:?} not on screen\n{text}")
}

/// The column of `needle` on row `y`, searching from column `from`.
fn col(h: &H, y: u16, from: u16, needle: &str) -> u16 {
    let r = row(h, y);
    let skip: String = r.chars().take(from as usize).collect();
    let i = r[skip.len()..]
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not on row {y} after {from}: {r}"));
    from + r[skip.len()..skip.len() + i].chars().count() as u16
}

fn usage() -> H {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 140, 40);
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app.route, Route::Usage);
    h
}

#[test]
fn usage_provider_rows_tone_only_the_status_cell() {
    let t = theme();
    let h = usage();
    // a healthy provider: name primary, `✓ current` from Theme::status(Ok)
    let (x, y) = find(&h, "OpenAI       2");
    assert_eq!(cell(&h, x, y).fg, t.text_primary, "{}", row(&h, y));
    let (ok, ok_style) = t.status(StatusKind::Ok);
    let sx = col(&h, y, x + 40, ok);
    assert_eq!(Some(cell(&h, sx, y).fg), ok_style.fg, "{}", row(&h, y));
    assert_eq!(cell(&h, sx + 2, y).fg, t.success);
    // an exhausted provider: the name stays primary, only the status is red
    let (x, y) = find(&h, "Anthropic    2");
    assert_eq!(cell(&h, x, y).fg, t.text_primary, "{}", row(&h, y));
    let (fail, fail_style) = t.status(StatusKind::Fail);
    let sx = col(&h, y, x + 40, fail);
    assert_eq!(Some(cell(&h, sx, y).fg), fail_style.fg, "{}", row(&h, y));
    assert!(cell(&h, sx, y).modifier.contains(Modifier::BOLD));
    assert_eq!(cell(&h, sx + 2, y).fg, t.error);
    // the worst window is a meter, not text in the row's tone
    let filled = (x + 23..x + 50).any(|cx| cell(&h, cx, y).bg != t.canvas);
    assert!(filled, "the window is a filled block: {}", row(&h, y));
}

#[test]
fn the_usage_health_line_tones_each_part() {
    let t = theme();
    let h = usage();
    let (x, y) = find(&h, "Health       degraded");
    assert_eq!(cell(&h, x, y).fg, t.text_primary);
    let d = col(&h, y, x, "degraded");
    assert_eq!(cell(&h, d, y).fg, t.error);
    assert!(cell(&h, d, y).modifier.contains(Modifier::BOLD));
    assert_eq!(cell(&h, col(&h, y, x, "4 warnings"), y).fg, t.warning);
    assert_eq!(cell(&h, col(&h, y, x, "2 stale"), y).fg, t.warning);
    assert_eq!(cell(&h, col(&h, y, x, "1 refreshing"), y).fg, t.info);
    assert_eq!(cell(&h, col(&h, y, x, "3 failed"), y).fg, t.error);
}

#[test]
fn the_accounts_health_line_and_warnings_tone_each_part() {
    let t = theme();
    let h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 140, 40);
    assert_eq!(h.app.route, Route::Accounts);
    let (x, y) = find(&h, "Health        degraded");
    let d = col(&h, y, x, "degraded");
    assert_eq!(cell(&h, d, y).fg, t.error, "{}", row(&h, y));
    assert!(cell(&h, d, y).modifier.contains(Modifier::BOLD));
    assert_eq!(cell(&h, col(&h, y, x, "4 warnings"), y).fg, t.warning);
    assert_eq!(cell(&h, col(&h, y, x, "1 refreshing"), y).fg, t.info);
    // a warning row: the glyph and the phrase in warning, the account
    // primary, the reset time muted
    let (x, y) = find(&h, "Claude · Work   Session");
    let (warn, warn_style) = t.status(StatusKind::Warn);
    assert_eq!(cell(&h, x - 2, y).symbol(), warn, "{}", row(&h, y));
    assert_eq!(Some(cell(&h, x - 2, y).fg), warn_style.fg);
    assert_eq!(cell(&h, x, y).fg, t.text_primary, "{}", row(&h, y));
    assert_eq!(cell(&h, col(&h, y, x, "Session"), y).fg, t.warning);
    assert_eq!(cell(&h, col(&h, y, x, "resets"), y).fg, t.text_muted);
    // an error row: `✗`, the account primary, the message in error
    let (x, y) = find(&h, "Kimi · discovered   No credential");
    let (fail, fail_style) = t.status(StatusKind::Fail);
    assert_eq!(cell(&h, x - 2, y).symbol(), fail, "{}", row(&h, y));
    assert_eq!(Some(cell(&h, x - 2, y).fg), fail_style.fg);
    assert_eq!(cell(&h, x, y).fg, t.text_primary);
    assert_eq!(cell(&h, col(&h, y, x, "No credential"), y).fg, t.error);
}

#[test]
fn provider_group_labels_are_muted_not_faint() {
    let t = theme();
    let h = usage();
    // the usage tree's provider heading
    let (x, y) = find(&h, "Anthropic ");
    assert_eq!(cell(&h, x, y).fg, t.text_muted, "{}", row(&h, y));
    // the accounts tree's sentinel provider
    let h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 140, 40);
    let (x, y) = find(&h, "Unsupported");
    assert_eq!(cell(&h, x, y).fg, t.text_muted, "{}", row(&h, y));
}

#[test]
fn a_running_instance_shows_live_state_in_the_accent() {
    let t = theme();
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 140, 40);
    assert_eq!(h.app.route, Route::Manager);
    let (x, y) = find(&h, "◉ 7f3a");
    assert_eq!(cell(&h, x, y).fg, t.accent, "{}", row(&h, y));
    assert_eq!(cell(&h, x + 2, y).fg, t.text_primary);
    let r = col(&h, y, x, "running");
    assert_eq!(cell(&h, r, y).fg, t.accent, "{}", row(&h, y));
    // the tree row of the same instance, once its workspace is open
    let mut h = h;
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    let (x, y) = find(&h, "7f3a");
    assert_eq!(cell(&h, x - 2, y).symbol(), "◉", "{}", row(&h, y));
    assert_eq!(cell(&h, x - 2, y).fg, t.accent, "{}", row(&h, y));
    // a stopped one stays quiet
    let (x, y) = find(&h, "c41e");
    assert_eq!(cell(&h, x - 2, y).symbol(), "◌", "{}", row(&h, y));
    assert_eq!(cell(&h, x - 2, y).fg, t.text_muted);
}

#[test]
fn the_running_count_is_secondary_behind_a_live_dot() {
    let t = theme();
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 140, 40);
    let (x, y) = find(&h, " 2 running");
    assert_eq!(
        cell(&h, x - 1, y).symbol(),
        t.glyphs.dot_on,
        "{}",
        row(&h, y)
    );
    assert_eq!(cell(&h, x - 1, y).fg, t.accent);
    assert_eq!(cell(&h, x + 1, y).fg, t.text_secondary, "{}", row(&h, y));
}

#[test]
fn the_debug_run_id_chip_is_a_link_not_a_warning() {
    let t = theme();
    let mut h = H::new(Scenario::LaunchRunning, Motion::Reduced, 0, 140, 40);
    h.key(KeyCode::Char('t'));
    let (x, y) = find(&h, "run-2026");
    let c = cell(&h, x, y);
    assert_eq!(c.fg, t.legible(t.link, c.bg), "{}", row(&h, y));
    assert_ne!(c.fg, t.warning);
}

#[test]
fn a_finished_capsule_tab_is_done() {
    let t = theme();
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 140, 40);
    h.type_str("y");
    h.key(KeyCode::Enter);
    h.ticks(400);
    let (glyph, style) = t.status(StatusKind::Ok);
    let r = row(&h, 2);
    assert!(r.contains(&format!("Mix (3) {glyph}")), "{r}");
    let x = col(&h, 2, 0, glyph);
    assert_eq!(Some(cell(&h, x, 2).fg), style.fg, "{r}");
}
