//! Guarded exit through the real runtime and production dialog.
use junie_tui::{
    App, Cx, Dialog, DialogState, GridEditor, Id, Input, KeyCode, KeyMap, LayerId, Response, Theme,
    Ui,
};
use junie_tui_testing::Harness;
use tablepro_app::{
    Catalog, ColType, QueryTab, ResultGrid, ResultSet, Surface, Tab, TableProApp, Value,
};

fn begin_query_edit(h: &mut Harness<TableProApp>) {
    let query = Id::root("tablepro.query");
    for _ in 0..20 {
        if h.focus() == Some(query) {
            break;
        }
        let _ = h.key(KeyCode::Tab);
    }
    assert_eq!(h.focus(), Some(query));
}

#[test]
fn plain_q_quits_the_connection_screen() {
    let mut h = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.app().should_quit());
}

#[test]
fn empty_query_is_clean_and_execution_does_not_save_edits() {
    let mut query = QueryTab::new(1, "");
    assert!(!query.dirty());
    query.query = "SELECT * FROM orders LIMIT 1".to_owned();
    assert!(query.execute(&Catalog::acme_prod()).is_ok());
    assert!(query.dirty());
}

#[test]
fn dirty_query_from_keyboard_prompts_and_escape_preserves_it() {
    let mut h = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('t');
    begin_query_edit(&mut h);
    let _ = h.key(KeyCode::Char('q'));
    assert!(!h.app().should_quit());
    let _ = h.ctrl('q');
    assert!(!h.app().should_quit());
    assert!(h.find("Quit TablePro?").is_some());
    assert!(h.find("1 unsaved query will be lost.").is_some());
    let _ = h.key(KeyCode::Esc);
    assert!(!h.app().should_quit());
    assert!(h.find("Quit TablePro?").is_none());
    let _ = h.ctrl('q');
    assert!(h.find("1 unsaved query will be lost.").is_some());
}

#[test]
fn pending_rows_require_explicit_confirmation_and_modal_blocks_commands() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let pending = h.app().result().pending_total();
    assert_eq!(pending, 1);
    let _ = h.ctrl('q');
    assert!(h.find("1 pending row change will be lost.").is_some());
    let tabs = h.app().workbench.tabs.len();
    for key in ['q', 'c', 't', 'r'] {
        let _ = h.ctrl(key);
        assert!(!h.app().should_quit());
        assert_eq!(h.app().workbench.tabs.len(), tabs);
        assert_eq!(h.app().result().pending_total(), pending);
    }
    let _ = h.key(KeyCode::Char('q'));
    assert!(!h.app().should_quit());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.runtime().top_layer(), LayerId::PAGE);
    assert_eq!(h.app().result().pending_total(), pending);
    let _ = h.ctrl('q');
    let confirm = Dialog::new(Id::root("tablepro.quit-dialog")).action_id(1);
    let _ = h.click_id(confirm);
    assert!(h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), pending);
}

#[test]
fn ctrl_c_cancels_declared_running_query_before_requesting_exit() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::QueryEditing);
    if let Some(Tab::Query(query)) = app.workbench.active_mut() {
        query.running = true;
    }
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('c');
    assert!(!h.app().should_quit());
    assert_eq!(h.app().status(), "Query cancelled");
    assert!(matches!(h.app().workbench.active(), Some(Tab::Query(query)) if !query.running));
    let _ = h.ctrl('c');
    assert!(h.find("Quit TablePro?").is_some());
}

#[test]
fn confirmation_draws_and_cancel_do_not_commit_query_draft() {
    let mut h = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('t');
    begin_query_edit(&mut h);
    let _ = h.key(KeyCode::Char('x'));
    let committed = h.app().query().to_owned();
    let _ = h.ctrl('q');
    for _ in 0..5 {
        h.draw();
        assert_eq!(h.app().query(), committed);
        assert!(!h.app().should_quit());
    }
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().query(), committed);
    let _ = h.ctrl('q');
    assert!(h.find("1 unsaved query will be lost.").is_some());
}

#[test]
fn pending_count_is_per_row_operation_not_per_changed_cell() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let mut grid = app.result().clone();
    assert!(grid.commit_cell(0, 6, "EUR").is_ok());
    assert!(grid.commit_cell(0, 5, "12.34").is_ok());
    assert_eq!(grid.pending_total(), 1);
    assert!(grid.insert_row().is_some());
    assert_eq!(grid.pending_total(), 2);
    assert!(grid.delete_row(1));
    assert_eq!(grid.pending_total(), 3);
    assert!(grid.delete_row(0));
    assert_eq!(
        grid.pending_total(),
        4,
        "reference counts update and deletion independently"
    );
}

#[test]
fn untouched_query_with_initial_text_is_clean() {
    let query = QueryTab::new(1, "SELECT * FROM orders");
    assert!(!query.dirty());
}

#[test]
fn query_debug_does_not_expose_saved_or_current_sql() {
    let mut query = QueryTab::new(1, "synthetic-saved-secret");
    query.query = "synthetic-current-secret".to_owned();
    query.error = Some("synthetic-error-secret".to_owned());
    query.result = Some(ResultGrid::from_result(&ResultSet {
        columns: vec![("value".to_owned(), ColType::Text)],
        rows: vec![vec![Value::Text("synthetic-result-secret".to_owned())]],
        total: 1,
        source: None,
        duration_ms: 0,
        editable: false,
    }));
    let mut app = TableProApp::default();
    app.workbench.tabs.push(Tab::Query(query));
    let debug = format!("{app:?} {:?}", app.workbench);
    for secret in [
        "synthetic-saved-secret",
        "synthetic-current-secret",
        "synthetic-error-secret",
        "synthetic-result-secret",
    ] {
        assert!(!debug.contains(secret));
    }
}

struct NestedDialogApp {
    inner: TableProApp,
    open_nested: bool,
    state: DialogState,
}

const NESTED: Id = Id::root("test.tablepro.nested");

impl App for NestedDialogApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self.inner.update(cx);
        let dialog = Dialog::info(NESTED, "Nested modal");
        if self.open_nested {
            self.open_nested = false;
            cx.open_layer(NESTED, dialog.layer(cx));
        }
        let action = dialog.update(cx, &mut self.state);
        if action.action_ref().is_some() {
            cx.close_layer(NESTED, None);
        }
        response | action.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.inner.draw(ui);
        ui.layer(NESTED, |ui, area| {
            Dialog::info(NESTED, "Nested modal").draw(ui, area, &self.state, |_, _| {});
        });
    }

    fn keymap(&self) -> &KeyMap {
        self.inner.keymap()
    }
}

#[test]
fn nested_modal_cannot_confirm_or_bypass_underlying_quit_guard() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let mut h = Harness::new(
        NestedDialogApp {
            inner: app,
            open_nested: false,
            state: DialogState::default(),
        },
        Theme::junie(),
        120,
        40,
    );
    let _ = h.ctrl('q');
    assert!(h.find("Quit TablePro?").is_some());
    h.app_mut().open_nested = true;
    let _ = h.handle(Input::Tick);
    assert!(h.find("Nested modal").is_some());
    for key in ['q', 'c'] {
        let _ = h.ctrl(key);
        assert!(!h.app().inner.should_quit());
    }
    let confirm = Dialog::new(Id::root("tablepro.quit-dialog")).action_id(1);
    let _ = h.click_id(confirm);
    assert!(!h.app().inner.should_quit());
    let _ = h.key(KeyCode::Esc);
    assert!(h.find("Quit TablePro?").is_some());
    assert!(!h.app().inner.should_quit());
}
