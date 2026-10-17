//! Copy-audit regressions: the footer names each key the way every other
//! surface of the app names it.

use ratatui::crossterm::event::KeyCode;

use crate::app_tests::H;
use crate::app_tests_parity::open;
use crate::scenario::{Motion, Scenario};

fn footer(h: &H) -> String {
    h.last_row()
}

#[test]
fn alt_enter_is_alternatives_in_the_finder_and_the_files_find_page() {
    let mut h = H::new(Scenario::ParityFiles, Motion::Reduced, 0, 200, 40);
    h.ticks(4);
    let f = footer(&h);
    assert!(f.contains("Alt+Enter Alternatives…"), "finder: {f}");
    assert!(!f.contains("More"), "finder: {f}");

    open(&mut h, "Find files under home");
    h.ticks(8);
    let f = footer(&h);
    // `/` opens Find, so the mode names itself Find; Enter opens the same
    // menu as Alt+Enter; Delete and Esc no longer both say Clear
    assert!(f.contains("Type Find"), "find: {f}");
    assert!(f.contains("Enter Alternatives…"), "find: {f}");
    assert!(!f.contains("Actions"), "find: {f}");
    assert!(!f.contains("Delete"), "find: {f}");
    assert!(f.contains("Esc Back"), "find, empty query: {f}");
    h.type_str("todo");
    let f = footer(&h);
    assert!(f.contains("Esc Clear"), "find, with a query: {f}");
    h.key(KeyCode::Enter);
    assert!(!h.app.modals.is_empty(), "Enter still opens the menu");
}

#[test]
fn the_files_browser_names_alt_enter_alternatives() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 0, 200, 40);
    h.ticks(4);
    open(&mut h, "Browse ~/work/site");
    h.ticks(8);
    let f = footer(&h);
    assert!(f.contains("Alt+Enter Alternatives…"), "browse: {f}");
    assert!(!f.contains("Actions"), "browse: {f}");
}

/// SVg10: holla's hint bar is left-aligned like every other app's (it was
/// centred), so the first hint starts one cell in from the left edge.
#[test]
fn the_hint_bar_is_left_aligned() {
    let h = H::new(Scenario::ParityFiles, Motion::Reduced, 0, 200, 40);
    let f = footer(&h);
    assert!(f.starts_with(' ') && !f.starts_with("  "), "{f:?}");
}

/// CO5 (SV11): an error in the status line wins over every hint, the
/// pinned ones included: it is shown whole instead of being cut to leave
/// `F1 Help` and `Esc Back` their room.
#[test]
fn an_error_status_is_never_cut_for_the_hints() {
    let mut h = H::new(Scenario::ParityFiles, Motion::Reduced, 0, 120, 40);
    // an error as wide as the row allows (holla cut it 8 cells short)
    let msg = "Could not read the archive volume mounted at the backup path: permission denied by the file system driver, try again";
    h.app
        .shell
        .status
        .set_with(msg, junie_tui::theme::Tone::Error, vec![]);
    h.draw();
    let f = footer(&h);
    assert!(f.contains(msg), "{f}");
}
