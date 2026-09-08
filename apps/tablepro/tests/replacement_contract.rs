//! Replacement entry points refuse pending work before changing state.
use junie_tui::{App, GridEditor, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{QueryOutcome, Tab, TableProApp};
fn pending_query() -> TableProApp {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT id, currency FROM orders LIMIT 1");
    let Some(Tab::Query(tab)) = app.workbench.active_mut() else {
        unreachable!("query")
    };
    let Some(grid) = tab.result.as_mut() else {
        unreachable!("result")
    };
    assert!(grid.model.commit_cell(0, 1, "EUR").is_ok());
    app
}
#[test]
fn public_reconnect_refuses_dirty_work_before_replacing_tabs() {
    let mut app = pending_query();
    let key = app.workbench.active_key();
    assert!(!app.connect(0));
    assert_eq!(app.workbench.active_key(), key);
    assert_eq!(app.result().pending_total(), 1);
    let connection = app.workbench.connection.clone();
    let catalog = app.workbench.catalog.clone();
    app.workbench.reconnect(connection, catalog);
    assert_eq!(app.workbench.active_key(), key);
    assert_eq!(app.result().pending_total(), 1);
}
#[test]
fn all_query_entrypoints_preserve_pending_results_and_sql() {
    let mut app = pending_query();
    let sql = app.query().to_owned();
    assert!(matches!(
        app.run_query("SELECT * FROM orders LIMIT 2"),
        QueryOutcome::Rejected { .. }
    ));
    assert_eq!(app.query(), sql);
    assert_eq!(app.result().pending_total(), 1);
    assert!(matches!(app.execute_query(), QueryOutcome::Rejected { .. }));
    let catalog = app.workbench.catalog.clone();
    let Some(Tab::Query(tab)) = app.workbench.active_mut() else {
        unreachable!("query")
    };
    assert!(tab.execute(&catalog).is_err());
    assert!(app.workbench.execute_active().is_err());
    assert_eq!(app.result().pending_total(), 1);
}

#[test]
fn result_confirmation_cancel_preserves_inline_draft_and_confirm_targets_captured_tab() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT id, currency FROM orders LIMIT 1");
    let Some(key) = app.workbench.active_key() else {
        unreachable!("key")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(grid) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(grid));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    let _ = h.type_str(" draft");
    let _ = h.ctrl('r');
    assert!(h.find("Replace results with unsaved edits?").is_some());
    let _ = h.key(KeyCode::Esc);
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    assert_eq!(view.state.edit_draft(), Some("USD draft"));
    assert_eq!(view.model.pending_total(), 0);
    let _ = h.ctrl('r');
    assert!(h.find("Replace results with unsaved edits?").is_some());
    let Some(neighbor) = h
        .app_mut()
        .workbench
        .new_query("SELECT * FROM customers LIMIT 2")
    else {
        unreachable!("neighbor")
    };
    h.draw();
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), Some(neighbor));
    let Some(Tab::Query(tab)) = h.app().workbench.tab(key) else {
        unreachable!("target")
    };
    let Some(view) = tab.result.as_ref() else {
        unreachable!("result")
    };
    assert_eq!(view.model.row_count(), 1);
    assert_eq!(view.pending_total(), 0);
    assert!(view.state.edit_draft().is_none());
    assert!(
        matches!(h.app().workbench.tab(neighbor),Some(Tab::Query(tab)) if tab.result.is_none())
    );
    assert!(!h.app().should_quit());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn result_confirmation_uses_captured_sql_and_vanished_target_is_noop() {
    let app = pending_query();
    let Some(key) = app.workbench.active_key() else {
        unreachable!("key")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('r');
    assert!(h.find("Replace results with unsaved edits?").is_some());
    let Some(Tab::Query(tab)) = h.app_mut().workbench.tab_mut(key) else {
        unreachable!("query")
    };
    tab.query = "SELECT * FROM customers LIMIT 2".to_owned();
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().result().row_count(), 1);
    // A new pending intent cannot be redirected to a fresh record after removal.
    let Some(Tab::Query(tab)) = h.app_mut().workbench.tab_mut(key) else {
        unreachable!("query")
    };
    let Some(view) = tab.result.as_mut() else {
        unreachable!("result")
    };
    assert!(view.model.commit_cell(0, 1, "EUR").is_ok());
    let _ = h.ctrl('r');
    let Some(payload) = h.app_mut().workbench.tab_mut(key) else {
        unreachable!("payload")
    };
    *payload = Tab::Query(tablepro_app::QueryTab::new(20, ""));
    let Some(index) = h.app().workbench.active_index() else {
        unreachable!("index")
    };
    assert!(h.app_mut().workbench.close_tab(index));
    let Some(next) = h
        .app_mut()
        .workbench
        .new_query("SELECT * FROM orders LIMIT 4")
    else {
        unreachable!("next")
    };
    h.draw();
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), Some(next));
    assert!(matches!(h.app().workbench.active(),Some(Tab::Query(tab)) if tab.result.is_none()));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn reconnect_dialog_cancel_preserves_and_confirm_replaces_once() {
    let mut app = pending_query();
    let key = app.workbench.active_key();
    app.screen = tablepro_app::Screen::Connections;
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.click_id(junie_tui::Id::root("tablepro.connections.details"));
    assert!(h.find("Reconnect with unsaved work?").is_some());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().workbench.active_key(), key);
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.click_id(junie_tui::Id::root("tablepro.connections.details"));
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_ne!(h.app().workbench.active_key(), key);
    assert_eq!(h.app().workbench.tabs().len(), 1);
    assert_eq!(h.app().screen, tablepro_app::Screen::Workbench);
    let fresh = h.app().workbench.active_key();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), fresh);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn save_connect_from_workbench_form_requires_guard_and_cancel_keeps_work() {
    let mut h = Harness::new(pending_query(), Theme::junie(), 120, 40);
    let key = h.app().workbench.active_key();
    let _ = h.ctrl('n');
    assert!(h.app().connection_form_open());
    let save = junie_tui::Id::root("tablepro.connections.form")
        .part(junie_tui::Part::ACTIONS)
        .index(3);
    assert!(h.tab_to(save));
    let _ = h.key(KeyCode::Enter);
    assert!(h.find("Reconnect with unsaved work?").is_some());
    assert!(!h.app().connection_form_open());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().workbench.active_key(), key);
    assert_eq!(h.app().result().pending_total(), 1);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
