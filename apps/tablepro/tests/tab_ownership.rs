//! Canonical tab state survives logical navigation through the real runtime.
use junie_tui::{App, GridEditor, GridModel, Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, Tab, TabKey, TableProApp, Value};

fn activate(h: &mut Harness<TableProApp>, key: TabKey) {
    assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
    for _ in 0..h.app().workbench.tabs().len() {
        if h.app().workbench.active_key() == Some(key) {
            break;
        }
        let _ = h.key(KeyCode::Left);
    }
    assert_eq!(h.app().workbench.active_key(), Some(key));
}

#[test]
fn keyboard_cell_edit_survives_tab_switch_with_pending_cursor_and_undo() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(key) = h.app().workbench.active_key() else {
        unreachable!("table fixture")
    };
    let Some(grid_id) = h.app().result_id() else {
        unreachable!("grid fixture")
    };
    assert!(h.tab_to(grid_id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str("EUR");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert_eq!(h.app().result().pending_total(), 1);
    let cursor = h
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor());
    let row_key = h.app().result().row_key(0);
    assert!(
        h.diagnostics().is_empty(),
        "before new {:?}",
        h.diagnostics()
    );
    let _ = h.ctrl('t');
    assert!(
        h.diagnostics().is_empty(),
        "after new {:?}",
        h.diagnostics()
    );
    assert_ne!(h.app().workbench.active_key(), Some(key));
    activate(&mut h, key);
    assert!(
        h.diagnostics().is_empty(),
        "after activate {:?}",
        h.diagnostics()
    );
    assert_eq!(h.app().result_id(), Some(grid_id));
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(h.app().result().row_key(0), row_key);
    assert_eq!(
        h.app()
            .workbench
            .active_grid()
            .and_then(|(_, grid)| grid.state.cursor()),
        cursor
    );
    let _ = h.ctrl('q');
    assert!(
        h.diagnostics().is_empty(),
        "after quit {:?}",
        h.diagnostics()
    );
    assert!(h.find("1 pending row change will be lost.").is_some());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending_total(), 1);
    let Some(table) = h.app_mut().workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.undo());
    assert_eq!(table.result.pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn active_key_and_owned_values_survive_earlier_close_and_reorder() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("orders"));
    let Some(key) = app.workbench.active_key() else {
        unreachable!("table")
    };
    let Some(table) = app.workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.commit_cell(0, 6, "EUR").is_ok());
    assert!(app.workbench.close_tab(0));
    assert_eq!(app.workbench.active_key(), Some(key));
    app.workbench.new_query("SELECT * FROM customers LIMIT 1");
    assert!(app.workbench.activate(key));
    let mut keys: Vec<_> = app
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect();
    keys.reverse();
    assert!(app.workbench.reorder_tabs(&keys));
    assert_eq!(app.workbench.active_key(), Some(key));
    assert_eq!(app.result().pending_total(), 1);
    assert_eq!(app.result_id(), Some(key.control("data")));
}

#[test]
fn query_headers_results_and_uncommitted_draft_survive_switch() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT id, email FROM customers LIMIT 3");
    let Some(key) = app.workbench.active_key() else {
        unreachable!("query")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(query_id) = h.app().query_id() else {
        unreachable!("editor")
    };
    assert!(h.tab_to(query_id));
    let _ = h.key(KeyCode::End);
    let _ = h.type_str(" -- draft");
    let draft = match h.app().workbench.active() {
        Some(Tab::Query(tab)) => tab.editor_state.draft_text().map(str::to_owned),
        _ => None,
    };
    assert!(
        draft
            .as_deref()
            .is_some_and(|text| text.ends_with(" -- draft"))
    );
    let _ = h.ctrl('t');
    let Some(other) = h.app().query_id() else {
        unreachable!("new editor")
    };
    assert_ne!(query_id, other);
    activate(&mut h, key);
    assert_eq!(h.app().result().row_count(), 3);
    let Some(Tab::Query(query)) = h.app().workbench.active() else {
        unreachable!("query")
    };
    assert_eq!(query.editor_state.draft_text(), draft.as_deref());
    let Some(result) = &query.result else {
        unreachable!("result")
    };
    assert!(result.columns.iter().any(|(name, _)| name == "email"));
    assert!(h.find("email").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn structure_grid_is_separate_and_data_pending_undo_survives_toggle() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let Some(data_id) = app.result_id() else {
        unreachable!("data")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    assert!(h.tab_to(data_id));
    let _ = h.key(KeyCode::Down);
    let cursor = h
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor());
    let _ = h.ctrl('d');
    assert_ne!(h.app().result_id(), Some(data_id));
    assert!(!h.app().result().is_editable());
    let Some(table) = h.app().workbench.active_table() else {
        unreachable!("table")
    };
    assert_eq!(table.result.pending_total(), 1);
    let _ = h.ctrl('d');
    assert_eq!(h.app().result_id(), Some(data_id));
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(
        h.app()
            .workbench
            .active_grid()
            .and_then(|(_, grid)| grid.state.cursor()),
        cursor
    );
    let Some(table) = h.app_mut().workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.undo());
    assert_eq!(table.result.pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn reconnect_never_reuses_old_editor_focus_or_pointer_capture() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(old) = h.app().query_id() else {
        unreachable!("editor")
    };
    assert!(h.tab_to(old));
    let Some(area) = h.area_of(old) else {
        unreachable!("editor area")
    };
    let _ = h.mouse(junie_tui::MouseKind::Down, area.x, area.y);
    assert!(h.app_mut().connect(0));
    let Some(new) = h.app().query_id() else {
        unreachable!("replacement")
    };
    assert_ne!(old, new);
    h.draw();
    let _ = h.mouse(junie_tui::MouseKind::Up, area.x, area.y);
    let _ = h.type_str("stale");
    assert_eq!(h.app().query(), "");
    let Some(Tab::Query(query)) = h.app().workbench.active() else {
        unreachable!("replacement")
    };
    assert!(query.editor_state.draft_text().is_none());
    assert_ne!(h.focus(), Some(old));
    assert_ne!(h.focus(), Some(new));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn focused_query_identity_and_draft_survive_earlier_insert_close_and_reorder() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    app.workbench.new_query("");
    let Some(key) = app.workbench.active_key() else {
        unreachable!("query")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().query_id() else {
        unreachable!("editor")
    };
    assert!(h.tab_to(id));
    let _ = h.type_str("draft survives identity changes");
    assert!(h.app_mut().workbench.close_tab(0));
    assert!(
        h.app_mut()
            .workbench
            .insert_tab(
                0,
                Tab::Query(tablepro_app::QueryTab::new(3, "inserted neighbor"))
            )
            .is_some()
    );
    assert!(h.app_mut().workbench.activate(key));
    h.draw();
    assert_eq!(h.app().query_id(), Some(id));
    assert_eq!(h.focus(), Some(id));
    let mut keys: Vec<_> = h
        .app()
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect();
    keys.reverse();
    assert!(h.app_mut().workbench.reorder_tabs(&keys));
    h.draw();
    assert_eq!(h.app().workbench.active_key(), Some(key));
    assert_eq!(h.focus(), Some(id));
    let Some(Tab::Query(query)) = h.app().workbench.active() else {
        unreachable!("query")
    };
    assert_eq!(
        query.editor_state.draft_text(),
        Some("draft survives identity changes")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inactive_query_result_pending_edits_are_counted_by_quit_guard() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT * FROM orders LIMIT 1");
    let Some(Tab::Query(query)) = app.workbench.active_mut() else {
        unreachable!("query")
    };
    let Some(grid) = &mut query.result else {
        unreachable!("result")
    };
    assert!(grid.model.commit_cell(0, 6, "EUR").is_ok());
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('t');
    let _ = h.ctrl('q');
    assert!(h.find("1 pending row change and 1 unsaved query").is_some());
    assert!(!h.app().should_quit());
    let _ = h.key(KeyCode::Esc);
    let first = h
        .app()
        .workbench
        .tabs()
        .first()
        .map(tablepro_app::TabRecord::payload);
    assert!(
        matches!(first, Some(Tab::Query(query)) if query.result.as_ref().is_some_and(|grid| grid.pending_total() == 1))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
