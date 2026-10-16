//! Pointer text selection in holla's fields through the real App: click
//! counts, press and drag reach the argument fields (interaction #16).

use junie_tui::core::event::MouseKind;
use junie_tui::theme::Theme;
use ratatui::crossterm::event::KeyCode;

use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

/// The selected cells of row `y`, as drawn.
fn selected_on_row(h: &H, y: u16) -> String {
    let sel = Theme::junie().selection().bg.expect("a selection colour");
    let buf = h.term.backend().buffer();
    (0..buf.area.width)
        .filter(|&x| buf[(x, y)].bg == sel)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

/// The arguments page with "alpha beta gamma" in its first field.
fn args() -> (H, u16, u16) {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    let item = h
        .app
        .world
        .items()
        .into_iter()
        .find(|item| !item.args.is_empty())
        .expect("an argument-bearing fixture item")
        .id;
    h.app.go(Go::Push(Page::Args { item }));
    h.draw();
    h.key(KeyCode::Enter);
    for _ in 0..40 {
        h.key(KeyCode::Backspace);
    }
    h.type_str("alpha beta gamma");
    h.key(KeyCode::Tab);
    let (x, y) = h
        .find("alpha beta gamma")
        .unwrap_or_else(|| panic!("{}", h.text()));
    (h, x, y)
}

#[test]
fn a_double_click_in_an_argument_field_selects_the_word() {
    let (mut h, x, y) = args();
    h.click(x + 7, y);
    h.click(x + 7, y);
    assert_eq!(selected_on_row(&h, y), "beta", "{}", h.text());
}

#[test]
fn dragging_in_an_argument_field_selects_and_a_double_press_drags_by_words() {
    let (mut h, x, y) = args();
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 3, y);
    h.mouse(MouseKind::Up, x + 3, y);
    assert_eq!(selected_on_row(&h, y), "alp", "{}", h.text());
    h.click(x + 1, y);
    h.mouse(MouseKind::Down, x + 1, y);
    h.mouse(MouseKind::Drag, x + 7, y);
    h.mouse(MouseKind::Up, x + 7, y);
    assert_eq!(selected_on_row(&h, y), "alpha beta", "{}", h.text());
}
