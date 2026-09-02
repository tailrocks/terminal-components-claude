//! Interaction tests for the Capsule chrome (menu bar, tab context menu,
//! status bar), the shell hint bar, the inspector and palette scrolling.

use ratatui::crossterm::event::KeyCode;

use junie_tui::core::event::MouseKind;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

fn row(h: &H, y: u16) -> String {
    h.text().lines().nth(y as usize).unwrap_or("").to_owned()
}

fn last_row(h: &H) -> String {
    let t = h.text();
    t.lines().next_back().unwrap_or("").to_owned()
}

#[test]
fn capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line() {
    let h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Capsule);
    let top = row(&h, 0);
    assert!(!top.contains("inside the Construct"), "{top}");
    assert!(
        top.contains("jackin❯") && top.contains("File") && top.contains("Help"),
        "{top}"
    );
    assert!(row(&h, 2).contains("Shell"), "{}", row(&h, 2));
    let status = row(&h, 38);
    assert!(status.contains("payments-platform ›"), "{status}");
    assert!(status.contains('%'), "usage chip missing: {status}");
    assert!(last_row(&h).contains("Ctrl+B"), "{}", last_row(&h));
}

#[test]
fn menu_bar_opens_switches_and_runs_an_action() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::F(10));
    assert!(
        h.text().contains("New tab") && h.text().contains("Split right"),
        "{}",
        h.text()
    );
    assert!(
        last_row(&h).contains("Menu") && last_row(&h).contains("Choose"),
        "{}",
        last_row(&h)
    );
    h.key(KeyCode::Right);
    assert!(h.text().contains("Copy selection"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Copy selection"));
    h.key(KeyCode::F(10));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("New tab"), "spawn picker: {}", h.text());
    h.key(KeyCode::Esc);
    // mouse: click the View label, then Usage
    let (x, y) = h.find("View").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Zoom pane"), "{}", h.text());
    let (ux, uy) = h.find("Usage").unwrap();
    h.click(ux, uy);
    assert!(h.text().contains("Overview"), "usage dialog: {}", h.text());
}

#[test]
fn tab_context_menu_renames_and_closes_by_mouse_and_keyboard() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let (x, y) = h.find("Shell").unwrap();
    h.mouse(MouseKind::Secondary, x, y);
    assert!(
        h.text().contains("Change title…") && h.text().contains("Close tab"),
        "{}",
        h.text()
    );
    assert!(last_row(&h).contains("Choose"), "{}", last_row(&h));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Change tab title"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.type_str("ops");
    assert!(h.text().contains("ops"), "{}", h.text());
    h.key(KeyCode::Enter);
    if h.text().contains("Change tab title") {
        h.key(KeyCode::Enter);
    }
    assert!(row(&h, 2).contains("ops"), "{}", row(&h, 2));
    // keyboard path: prefix m opens the menu for the active tab; the last row is Close
    h.ctrl('b');
    h.key(KeyCode::Char('m'));
    assert!(h.text().contains("Change title…"), "{}", h.text());
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Close tab?"), "{}", h.text());
    h.key(KeyCode::Esc);
    // dismissal
    h.ctrl('b');
    h.key(KeyCode::Char('m'));
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Change title…"));
}

#[test]
fn hint_bar_stays_on_the_last_row_across_layers() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    let base = last_row(&h);
    assert!(base.contains("Enter"), "{base}");
    h.key(KeyCode::Char('?'));
    let help = last_row(&h);
    assert!(help.contains("Esc") && help.contains("Close"), "{help}");
    h.key(KeyCode::Esc);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    let picker = last_row(&h);
    assert!(picker.contains("Choose"), "{picker}");
    // the picker draws no hint row of its own
    let t = h.text();
    assert_eq!(t.matches("Enter Choose").count(), 1, "{t}");
    h.key(KeyCode::Esc);
    let mut c = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    c.ctrl('b');
    assert!(last_row(&c).contains("New tab"), "{}", last_row(&c));
    assert!(row(&c, 38).contains("prefix…"), "{}", row(&c, 38));
}

#[test]
fn inspect_changes_opens_from_the_view_menu_in_both_modes() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::F(10));
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Inspect changes ·"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("@@") || h.text().contains("│"),
        "diff shown: {}",
        h.text()
    );
    h.key(KeyCode::Char('m'));
    assert!(
        h.text().contains("src") && last_row(&h).contains("Tab"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('d'));
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Inspect changes ·"), "{}", h.text());
    assert_eq!(h.app.route, Route::Capsule);
}

#[test]
fn command_palette_scrolls_with_the_wheel_and_keeps_the_selection() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 24);
    h.ctrl('\\');
    assert!(h.text().contains("Command palette"));
    let (x, y) = h.find("New tab").unwrap();
    let before = h.text();
    h.mouse(MouseKind::WheelDown, x, y + 2);
    let after = h.text();
    assert_ne!(before, after, "wheel did not move the rows");
    assert!(
        !after.contains("New tab")
            || after
                .lines()
                .nth(y as usize)
                .is_some_and(|l| !l.contains("New tab")),
        "{after}"
    );
    h.mouse(MouseKind::WheelUp, x, y + 2);
    assert_eq!(h.text(), before, "wheel up did not restore the rows");
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("New tab"),
        "the selection stayed on the first item: {}",
        h.text()
    );
}
