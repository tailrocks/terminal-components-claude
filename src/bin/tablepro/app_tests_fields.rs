//! Pointer text selection in TablePro's fields through the real App:
//! click counts, press and drag reach the fields (interaction #16).

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use ratatui::crossterm::event::KeyCode;

use crate::app::Modal;
use crate::app_tests::H;
use crate::workbench::WorkTab;

fn history_search(h: &H) -> &junie_tui::widgets::input::TextInput {
    match h.wb().active_tab() {
        Some(WorkTab::History(t)) => &t.search,
        _ => panic!("expected the history tab"),
    }
}

/// The History tab with `text` typed into its search.
fn searched(text: &str) -> (H, u16, u16) {
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    h.key(KeyCode::Char('/'));
    h.type_str(text);
    h.key(KeyCode::Enter);
    let (x, y) = h.find(text).expect("the search text");
    (h, x, y)
}

#[test]
fn a_double_click_in_the_history_search_selects_the_word() {
    let (mut h, x, y) = searched("plan count");
    h.click(x + 1, y);
    h.click(x + 1, y);
    assert_eq!(history_search(&h).buffer.selected_text(), Some("plan"));
    h.click(x + 1, y);
    assert_eq!(
        history_search(&h).buffer.selected_text(),
        Some("plan count"),
        "triple click: the line"
    );
}

#[test]
fn dragging_in_the_explorer_filter_selects_what_it_crosses() {
    let mut h = H::connected(120, 40);
    let filter = h.wb().explorer_filter.id;
    h.tab_to(filter);
    h.key(KeyCode::Enter);
    h.type_str("orders");
    h.key(KeyCode::Enter);
    let (x, y) = h.find("orders").expect("the filter text");
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 3, y);
    h.mouse(MouseKind::Up, x + 3, y);
    assert_eq!(h.wb().explorer_filter.buffer.selected_text(), Some("ord"));
}

#[test]
fn a_drag_from_a_double_click_extends_the_history_search_by_words() {
    let (mut h, x, y) = searched("plan count");
    h.click(x + 1, y);
    h.mouse(MouseKind::Down, x + 1, y);
    h.mouse(MouseKind::Drag, x + 6, y);
    h.mouse(MouseKind::Up, x + 6, y);
    assert_eq!(
        history_search(&h).buffer.selected_text(),
        Some("plan count")
    );
}

#[test]
fn a_double_click_in_the_filter_editor_value_selects_the_word() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
    h.key(KeyCode::Char('f'));
    let value = WidgetId::of("filter-editor").sub("value");
    h.tab_to(value);
    h.key(KeyCode::Enter);
    for _ in 0..64 {
        h.key(KeyCode::Backspace);
    }
    h.type_str("abc def");
    // Tab applies the field and moves on (Enter would apply the filter)
    h.key(KeyCode::Tab);
    let (x, y) = h.find("abc def").expect("the value");
    h.click(x + 5, y);
    h.click(x + 5, y);
    let sel = match h.app.modal() {
        Some(Modal::Filter(f)) => f.value.buffer.selected_text().map(str::to_owned),
        _ => panic!("the filter editor is open"),
    };
    assert_eq!(sel.as_deref(), Some("def"));
}

#[test]
fn a_press_on_the_filter_editor_body_ends_the_value_edit() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
    h.key(KeyCode::Char('f'));
    let value = WidgetId::of("filter-editor").sub("value");
    h.tab_to(value);
    h.key(KeyCode::Enter);
    let editing = |h: &H| match h.app.modal() {
        Some(Modal::Filter(f)) => f.value.editing,
        _ => panic!("the filter editor is open"),
    };
    assert!(editing(&h));
    let (x, y) = h
        .find("Add filter")
        .or_else(|| h.find("Edit filter"))
        .expect("the editor title");
    h.click(x + 30, y);
    assert!(!editing(&h), "dead space ends the edit\n{}", h.text());
    assert!(
        matches!(h.app.modal(), Some(Modal::Filter(_))),
        "the editor stays open"
    );
}
