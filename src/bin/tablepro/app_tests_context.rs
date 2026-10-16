//! Secondary clicks and context menus through the real App.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use ratatui::crossterm::event::KeyCode;

use crate::app::Modal;
use crate::app_tests::H;

/// Open `orders` from the explorer (its grid focused).
fn open_orders(h: &mut H) {
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}

fn filter_column_open(h: &H) -> Option<bool> {
    match h.app.modal() {
        Some(Modal::Filter(f)) => Some(f.column.open),
        _ => None,
    }
}

#[test]
fn a_right_click_outside_the_filter_editors_open_select_closes_it() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.key(KeyCode::Char('f'));
    // the column select, opened from the keyboard
    let column = WidgetId::of("filter-editor").sub("col");
    h.tab_to(column);
    h.key(KeyCode::Enter);
    assert_eq!(filter_column_open(&h), Some(true), "the select is open");
    // a right-click on the dimmed page, outside the popup
    h.mouse(MouseKind::Secondary, 3, 30);
    assert_eq!(
        filter_column_open(&h),
        Some(false),
        "the right-click outside closes the select, like a click"
    );
    assert!(
        matches!(h.app.modal(), Some(Modal::Filter(_))),
        "the form itself stays: only its Esc or Cancel discards it"
    );
}
