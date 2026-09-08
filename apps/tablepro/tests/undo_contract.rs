//! The pinned grid's plain-u undo remains scoped to a focused, nonediting grid.
use junie_tui::{GridEditor, Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, Tab, TableProApp, Value};

fn replace_currency(h: &mut Harness<TableProApp>, text: &str) {
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str(text);
    let _ = h.key(KeyCode::Enter);
}

#[test]
fn plain_u_after_keyboard_edit_new_tab_and_return_undoes_only_that_logical_table() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let neighbor_key = app.workbench.active_key();
    let Some(neighbor) = app.workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(neighbor.result.model.commit_cell(0, 6, "GBP").is_ok());
    assert!(app.workbench.open_table("orders"));
    let Some(active_key) = app.workbench.active_key() else {
        unreachable!("table")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    replace_currency(&mut h, "EUR");
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.ctrl('t');
    assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
    let _ = h.key(KeyCode::Left);
    assert_eq!(h.app().workbench.active_key(), Some(active_key));
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    let _ = h.key(KeyCode::Char('u'));
    assert_eq!(h.app().result().pending_total(), 0);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("USD".to_owned()))
    );
    let neighbor = h
        .app()
        .workbench
        .tabs()
        .iter()
        .find(|tab| Some(tab.key()) == neighbor_key)
        .map(tablepro_app::TabRecord::payload);
    assert!(
        matches!(neighbor, Some(Tab::Table(table)) if table.result.pending_total() == 1 && table.result.pending().value(0, 6) == Some(&Value::Text("GBP".to_owned())))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn plain_u_types_into_inline_editor_without_undoing_existing_changes() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Char('u'));
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    assert_eq!(view.state.edit_draft(), Some("EURu"));
    assert_eq!(h.app().result().pending_total(), 1);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn plain_u_on_tab_strip_or_modal_cannot_undo_background_grid() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
    let _ = h.key(KeyCode::Char('u'));
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.ctrl('q');
    let _ = h.key(KeyCode::Char('u'));
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending_total(), 1);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn plain_u_in_sql_editor_is_text_and_leaves_inactive_table_pending() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('t');
    let Some(id) = h.app().query_id() else {
        unreachable!("query")
    };
    assert!(h.tab_to(id));
    let _ = h.type_str("u");
    let Some(Tab::Query(query)) = h.app().workbench.active() else {
        unreachable!("query")
    };
    assert_eq!(query.editor_state.draft_text(), Some("u"));
    assert!(h.app().workbench.tabs().iter().any(
        |tab| matches!(tab.payload(), Tab::Table(table) if table.result.pending_total() == 1)
    ));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
