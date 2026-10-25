#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::model::{HistoryEntry, HistorySource};
use tablepro_ui::{
    CONNECTION_NAME, Catalog, Decision, Filter, FilterOp, History, HistoryTab, PendingEdits,
    QueryOutcome, ResultGrid, SafeMode, Screen, Surface, Tab, TableProApp, Value, complete,
    form_fields, gate, parse, preview_for,
};
use termrock::{
    App, Axis, Button, Cx, Diagnostic, GlyphRole, GridEditor, Id, Intent, ItemKey, KeyCode, KeyMap,
    KeyModifiers, LayerOpening, LayerSize, LayerSpec, MouseKind, Part, PartRef, Response, Size,
    StateFlags, Theme, Ui,
};
use termrock_test_support::Harness;

fn connected() -> TableProApp {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    app
}

fn history_entry(sql: &str) -> HistoryEntry {
    HistoryEntry {
        id: 0,
        sql: sql.to_owned(),
        connection: "Production".to_owned(),
        database: "acme_prod".to_owned(),
        schema: "public".to_owned(),
        minutes_ago: 0,
        duration_ms: Some(1),
        rows: Some(1),
        error: None,
        source: HistorySource::Editor,
    }
}

fn history_entry_for(sql: &str, connection: &str, failed: bool) -> HistoryEntry {
    let mut entry = history_entry(sql);
    entry.connection = connection.to_owned();
    if failed {
        entry.error = Some("query failed".to_owned());
    }
    entry
}

fn history_row_marker<A: App>(harness: &Harness<A>, entry_id: usize) -> String {
    let list = Id::root("tablepro.workbench.history.list");
    let row = harness
        .area_of_part(
            list,
            PartRef::item(Part::ROW, ItemKey::num(entry_id as u64)),
        )
        .expect("the history entry has a registered row");
    harness
        .cell(row.x.saturating_add(1), row.y)
        .symbol()
        .to_owned()
}

fn history_chosen_marker_rows<A: App>(harness: &Harness<A>) -> Vec<u16> {
    let list = Id::root("tablepro.workbench.history.list");
    let Some(area) = harness.area_of(list) else {
        return Vec::new();
    };
    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Chosen);
    (area.y..area.bottom())
        .filter(|&y| harness.cell(area.x.saturating_add(1), y).symbol() == chosen)
        .collect()
}

struct ClipboardProbeApp {
    app: TableProApp,
    copies: Rc<RefCell<Vec<String>>>,
}

impl std::ops::Deref for ClipboardProbeApp {
    type Target = TableProApp;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl std::ops::DerefMut for ClipboardProbeApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
    }
}

impl App for ClipboardProbeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let copies = self.copies.clone();
        cx.set_clipboard_copy_hook(move |text| copies.borrow_mut().push(text.to_owned()));
        App::update(&mut self.app, cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        App::draw(&self.app, ui);
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        App::on_esc(&mut self.app, cx)
    }

    fn should_quit(&self) -> bool {
        App::should_quit(&self.app)
    }

    fn keymap(&self) -> &KeyMap {
        App::keymap(&self.app)
    }

    fn min_size(&self) -> Size {
        App::min_size(&self.app)
    }
}

fn clipboard_harness(
    app: TableProApp,
    width: u16,
    height: u16,
) -> (Harness<ClipboardProbeApp>, Rc<RefCell<Vec<String>>>) {
    let copies = Rc::new(RefCell::new(Vec::new()));
    let probe = ClipboardProbeApp {
        app,
        copies: copies.clone(),
    };
    (Harness::new(probe, Theme::junie(), width, height), copies)
}

#[test]
fn ctrl_t_focuses_the_empty_query_editor_at_120x40() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('t');
    let query = harness.app().query_id().expect("query tab");
    assert_eq!(harness.focus(), Some(query));
    assert_eq!(harness.app().surface(), Surface::QueryEditing);
    let Some(Tab::Query(tab)) = harness.app().workbench.active() else {
        panic!("the newly opened tab is a query")
    };
    assert!(tab.editor_state.editor().is_editing());
    assert_eq!(tab.editor_state.editor().text(), "");

    let _ = harness.key(KeyCode::Char('x'));
    let Some(Tab::Query(tab)) = harness.app().workbench.active() else {
        panic!("the query tab remains active")
    };
    assert!(tab.editor_state.editor().is_editing());
    assert_eq!(harness.app().query(), "x");
}

#[test]
fn preloaded_and_replaced_query_stays_unedited_until_component_focus() {
    let mut app = connected();
    let key = app
        .workbench
        .new_query("SELECT * FROM orders")
        .expect("query tab");
    let Some(Tab::Query(tab)) = app.workbench.tab(key) else {
        panic!("new query is present")
    };
    assert!(!tab.editor_state.editor().is_editing());
    assert_eq!(tab.editor_state.editor().text(), "SELECT * FROM orders");

    let Some(Tab::Query(tab)) = app.workbench.tab_mut(key) else {
        panic!("new query is present")
    };
    tab.replace_query("SELECT * FROM customers");
    assert!(!tab.editor_state.editor().is_editing());
    assert_eq!(tab.editor_state.editor().text(), "SELECT * FROM customers");

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = harness.app().query_id() else {
        panic!("preloaded query remains available")
    };
    assert!(tab_navigation::tab_to(&mut harness, id));
    let Some(Tab::Query(tab)) = harness.app().workbench.active() else {
        panic!("the preloaded query is active")
    };
    assert!(tab.editor_state.editor().is_editing());
}

#[test]
fn connections_screen_lists_and_connects_with_keyboard() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    assert!(harness.find("Connections").is_some());
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().screen(), Screen::Workbench);
    assert_eq!(harness.app().surface(), Surface::WorkbenchDefault);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn failed_connection_shows_error_and_retry() {
    let mut app = TableProApp::default();
    assert!(!app.connect(3));
    assert_eq!(app.surface(), Surface::ConnectionsFailed);
    assert!(app.status().contains("Connection failed"));
    assert!(app.connections_screen.error.is_some());
}
#[test]
fn explorer_opens_table_and_grid_navigates() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let tab = app.workbench.active_table().expect("table tab");
    assert_eq!(tab.result.row_count(), 500);
    assert_eq!(tab.table.name, "orders");
}

#[test]
fn unfocused_grid_cursor_does_not_style_the_cursor_row() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let harness = Harness::new(app, Theme::junie(), 120, 40);
    assert_eq!(harness.app().surface(), Surface::WorkbenchDefault);
    let (x, y) = harness
        .find("1d9f28fc-3e09-4488-bae7-d3aeb39b01c4")
        .expect("the first result row is rendered");
    assert!(
        !harness
            .cell(x, y)
            .modifier
            .contains(termrock::Modifier::BOLD),
        "unfocused result cells must not be presented as the cursor row"
    );
}

#[test]
fn vertically_scrolled_grid_does_not_repaint_top_rows_after_horizontal_scroll() {
    let mut harness = Harness::new(connected(), Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.text().contains("public › orders"));

    let _ = harness.key(KeyCode::PageDown);
    let _ = harness.key(KeyCode::End);

    let (grid_id, cursor_row, row_one_key, created_at, row_zero_prefix, row_one_prefix) = {
        let (grid_id, view) = harness
            .app()
            .workbench
            .active_grid()
            .expect("active orders grid");
        let (cursor_key, _) = view.state.cursor().expect("grid cursor");
        let cursor_row = (0..view.model.row_count())
            .find(|&row| termrock::GridModel::row_key(&view.model, row) == cursor_key)
            .expect("cursor row is in the model");
        assert!(cursor_row > 0, "PageDown moved to row {cursor_row}");
        assert!(view.state.col_offset() > 0, "End scrolled horizontally");

        let created_at = view
            .columns
            .iter()
            .position(|(name, _)| name == "created_at")
            .expect("created_at column");
        let row_zero = termrock::GridModel::cell(&view.model, 0, created_at)
            .expect("first row timestamp")
            .text;
        let row_one = termrock::GridModel::cell(&view.model, 1, created_at)
            .expect("second row timestamp")
            .text;
        (
            grid_id,
            cursor_row,
            termrock::GridModel::row_key(&view.model, 1),
            created_at,
            row_zero.chars().take(10).collect::<String>(),
            row_one.chars().take(10).collect::<String>(),
        )
    };
    let column_key = created_at.saturating_add(1);
    let header = harness
        .area_of_part(
            grid_id,
            PartRef::item(Part::HEADER, ItemKey::num(column_key as u64)),
        )
        .expect("visible created_at header hit region");
    let top_data_y = header.y.saturating_add(1);
    assert!(
        harness.find(&row_zero_prefix).is_none(),
        "row zero value {row_zero_prefix:?} was painted into a viewport whose cursor is row {cursor_row}:\n{}",
        harness.text()
    );
    assert_eq!(
        harness.find(&row_one_prefix).map(|(_, y)| y),
        Some(top_data_y),
        "the top cell's contents must come from the resolved first visible row:\n{}",
        harness.text()
    );

    let _ = harness.mouse(MouseKind::Down, header.x.saturating_add(1), top_data_y);
    let _ = harness.mouse(MouseKind::Up, header.x.saturating_add(1), top_data_y);
    let (_, view) = harness
        .app()
        .workbench
        .active_grid()
        .expect("active orders grid after pointer input");
    assert_eq!(
        view.state.cursor(),
        Some((
            row_one_key,
            termrock::ColumnKey::num((created_at as u16).saturating_add(1)),
        )),
        "the top cell hit region must stay on its real model row and column"
    );
}

#[test]
fn slash_filters_connections_and_keeps_the_open_detail() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Char('/'));
    for ch in ['s', 't', 'a', 'g'] {
        let _ = harness.key(KeyCode::Char(ch));
    }
    let text = harness.text();
    assert!(text.contains("stag"), "{text}");
    assert!(text.contains("Staging"), "{text}");
    assert!(text.contains("Acme"), "{text}");
    assert!(!text.contains("Production"), "{text}");
    assert!(text.contains("Local PostgreSQL"), "{text}");
    assert!(text.contains("Type Filter"), "{text}");
    assert!(text.contains("6 saved"), "{text}");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn alt_d_duplicates_the_selected_connection_and_keeps_the_cursor() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for _ in 0..7 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.alt('d');
    let text = harness.text();
    assert!(text.contains("7 saved"), "{text}");
    assert!(text.contains("Production (Copy)"), "{text}");
    assert!(text.contains("Duplicated"), "{text}");
    assert!(text.contains("✓ on"), "{text}");
    let cursor = text
        .lines()
        .find(|line| line.contains('▎') && line.contains("Production"))
        .expect("cursor row");
    assert!(
        !cursor.contains("(Copy)"),
        "cursor moved onto the copy: {cursor}"
    );
    assert_eq!(harness.app().connections_screen.connections.len(), 7);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn alt_d_duplicates_orders_and_scrolls_the_inserted_row_just_out_of_view() {
    let mut app = TableProApp::default();
    let index = app
        .connections_screen
        .connections
        .iter()
        .position(|connection| connection.name == "Production")
        .expect("production");
    assert!(app.connect(index));
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
    let _ = h.alt('d');
    let text = h.text();
    assert!(text.contains("rows 477–500 of 501"), "{text}");
    assert!(text.contains("1 insert"), "{text}");
    assert!(text.contains("92e3ff9a"), "{text}");
    assert!(!text.contains("DEFAULT"), "{text}");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn sort_and_filter_on_table_tab() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let filter = Filter {
        column: "status".to_owned(),
        op: FilterOp::Eq,
        value: "pending".to_owned(),
        value2: String::new(),
        enabled: true,
    };
    assert!(app.workbench.apply_filter(filter.clone()));
    assert_eq!(filter.to_sql(), "status = 'pending'");
    if let Some(tab) = app.workbench.active_table_mut() {
        tab.sort(0, termrock::SortDir::Desc);
        assert_eq!(tab.filters.len(), 1);
    }
}
#[test]
fn structure_view_toggle() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    assert!(app.workbench.toggle_structure());
    assert!(
        app.workbench
            .active_table()
            .is_some_and(|tab| tab.is_structure())
    );
    assert!(app.workbench.toggle_structure());
    assert!(
        !app.workbench
            .active_table()
            .is_some_and(|tab| tab.is_structure())
    );
}
#[test]
fn editor_completion_and_execution() {
    let mut app = connected();
    assert_eq!(
        app.run_query("SELECT * FROM orders LIMIT 25"),
        QueryOutcome::Executed {
            rows: 25,
            editable: true
        }
    );
    let completions = complete("SELECT * FROM ord", 16, &Catalog::acme_prod());
    assert!(completions.iter().any(|item| item.label == "orders"));
}
#[test]
fn execution_error_marks_editor_and_result() {
    let mut app = connected();
    assert!(matches!(
        app.run_query("SELECT nope FROM orders"),
        QueryOutcome::Rejected { .. }
    ));
    assert!(app.status().contains("nope"));
}
#[test]
fn cancel_running_query() {
    let mut app = connected();
    let index = app.workbench.new_query("SELECT * FROM orders");
    if let Some(Tab::Query(query)) = index.and_then(|key| app.workbench.tab_mut(key)) {
        query.running = true;
        query.running = false;
        assert!(!query.running);
    } else {
        panic!("new query must be active");
    }
    let mut history = HistoryTab::new(&app.workbench.history);
    history.search = "running".to_owned();
    history.filter(&app.workbench.history);
    assert!(history.selected_query().is_none());
}
#[test]
fn explain_opens_plan_tree() {
    let mut app = connected();
    let index = app.workbench.new_query("SELECT * FROM orders LIMIT 5");
    let catalog = app.workbench.catalog.clone();
    if let Some(Tab::Query(query)) = index.and_then(|key| app.workbench.tab_mut(key)) {
        assert!(query.explain(&catalog).is_ok());
    } else {
        panic!("new query must be active");
    }
}
#[test]
fn safety_gate_intercepts_dangerous_statement_on_production() {
    let mut app = connected();
    app.set_safe_mode(SafeMode::Silent);
    assert!(matches!(
        app.run_query("DELETE FROM orders"),
        QueryOutcome::ConfirmationRequired {
            deliberate: false,
            ..
        }
    ));
}
#[test]
fn safety_gate_typed_token_executes() {
    let mut app = connected();
    app.set_safe_mode(SafeMode::Safe);
    assert!(matches!(
        app.run_query("DELETE FROM orders"),
        QueryOutcome::ConfirmationRequired {
            deliberate: true,
            ..
        }
    ));
    assert!(matches!(
        app.run_query("SELECT * FROM orders LIMIT 1"),
        QueryOutcome::Executed { rows: 1, .. }
    ));
}
#[test]
fn read_only_connection_refuses_writes() {
    let mut app = connected();
    app.set_safe_mode(SafeMode::ReadOnly);
    assert!(matches!(
        app.run_query("UPDATE orders SET status = 'paid' WHERE id = 7"),
        QueryOutcome::Denied { .. }
    ));
}
#[test]
fn silent_level_runs_scoped_writes_but_confirms_destructive() {
    let mut app = connected();
    app.set_safe_mode(SafeMode::Silent);
    let statement = parse("UPDATE orders SET status = 'paid' WHERE id = 7").expect("write parses");
    assert_eq!(gate(SafeMode::Silent, &statement), Decision::Run);
    assert!(matches!(
        app.run_query("TRUNCATE orders"),
        QueryOutcome::ConfirmationRequired { .. }
    ));
}
#[test]
fn quick_switcher_opens_table() {
    let app = connected();
    assert!(
        app.workbench
            .switcher()
            .search("orders")
            .iter()
            .any(|item| item.label == "orders")
    );
}
#[test]
fn history_tab_reopens_query() {
    let mut app = connected();
    let index = app.workbench.open_history();
    assert!(matches!(
        index.and_then(|key| app.workbench.tab(key)),
        Some(Tab::History(_))
    ));
    assert!(!app.workbench.history.is_empty());
}

#[test]
fn reopening_history_reuses_tab_and_refreshes_current_filter() {
    let mut app = connected();
    let seeded = app.workbench.history.entries.clone();
    let history_key = app.workbench.open_history().expect("history tab opens");

    let Some(Tab::History(history)) = app.workbench.tab_mut(history_key) else {
        panic!("history tab is present")
    };
    history.search = "refresh marker".to_owned();
    history.selected = 1;

    app.workbench
        .history
        .push(history_entry("SELECT * FROM refresh_marker alpha"));
    app.workbench
        .history
        .push(history_entry("SELECT * FROM refresh_marker beta"));
    let query_key = app
        .workbench
        .new_query("SELECT 1")
        .expect("query tab opens");
    let tab_order: Vec<_> = app.workbench.tabs().iter().map(|tab| tab.key()).collect();
    let tab_count = app.workbench.tabs().len();
    let query_counter = app.workbench.query_counter;
    let history_count = app.workbench.history.entries.len();

    assert_eq!(app.workbench.open_history(), Some(history_key));
    assert_eq!(app.workbench.active_key(), Some(history_key));
    assert_eq!(app.workbench.tabs().len(), tab_count);
    assert_eq!(
        app.workbench
            .tabs()
            .iter()
            .map(|tab| tab.key())
            .collect::<Vec<_>>(),
        tab_order
    );
    assert_eq!(app.workbench.query_counter, query_counter);
    assert!(app.workbench.tab(query_key).is_some());
    assert_eq!(app.workbench.history.entries.len(), history_count);
    assert_eq!(&app.workbench.history.entries[2..], seeded.as_slice());

    let Some(Tab::History(history)) = app.workbench.tab(history_key) else {
        panic!("the existing history tab is reused")
    };
    assert_eq!(history.search, "refresh marker");
    assert_eq!(history.selected, 1);
    assert_eq!(history.entries.len(), 2);
    assert_eq!(history.entries[0].sql, "SELECT * FROM refresh_marker beta");
    assert_eq!(history.entries[1].sql, "SELECT * FROM refresh_marker alpha");

    let Some(Tab::History(history)) = app.workbench.tab_mut(history_key) else {
        panic!("the existing history tab is reused")
    };
    history.search = "no matching row".to_owned();
    history.selected = usize::MAX;
    assert_eq!(app.workbench.open_history(), Some(history_key));
    let Some(Tab::History(history)) = app.workbench.tab(history_key) else {
        panic!("the existing history tab is reused")
    };
    assert_eq!(history.search, "no matching row");
    assert!(history.entries.is_empty());
    assert_eq!(history.selected, 0);
    assert_eq!(app.workbench.tabs().len(), tab_count);
}

#[test]
fn ctrl_y_reuses_history_tab_and_keeps_list_focus() {
    let mut harness = Harness::new(connected(), Theme::junie(), 120, 40);
    let history_list = Id::root("tablepro.workbench.history.list");

    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("history tab is active");
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::History(_))
    ));
    assert_eq!(harness.focus(), Some(history_list));
    let history_area = harness
        .area_of(history_list)
        .expect("history list is laid out");
    assert!(history_area.width > 0 && history_area.height > 0);

    let _ = harness.ctrl('t');
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::Query(_))
    ));
    let tab_count = harness.app().workbench.tabs().len();
    let _ = harness.ctrl('y');

    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.tabs().len(), tab_count);
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::History(_))
    ));
    assert_eq!(harness.focus(), Some(history_list));
    assert_eq!(harness.area_of(history_list), Some(history_area));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_chosen_marker_tracks_unique_cursor_without_executing_sql() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT older_row"));
    app.workbench
        .history
        .push(history_entry("SELECT newer_row"));
    let query_count = app.workbench.query_counter;
    let tab_count = app.workbench.tabs().len();
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let (first_id, first_sql, second_id, second_sql) = {
        let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
            panic!("History remains active")
        };
        (
            history.entries[0].id,
            history.entries[0].sql.clone(),
            history.entries[1].id,
            history.entries[1].sql.clone(),
        )
    };
    let list = Id::root("tablepro.workbench.history.list");
    let first_row = harness
        .area_of_part(
            list,
            PartRef::item(Part::ROW, ItemKey::num(first_id as u64)),
        )
        .expect("the initial preview row is registered");
    let second_row = harness
        .area_of_part(
            list,
            PartRef::item(Part::ROW, ItemKey::num(second_id as u64)),
        )
        .expect("the next preview row is registered");
    let first_bg = harness.cell(first_row.x.saturating_add(3), first_row.y).bg;
    let second_bg = harness
        .cell(second_row.x.saturating_add(3), second_row.y)
        .bg;
    let chosen = Theme::junie().design.glyphs.get(GlyphRole::Chosen);
    assert_eq!(history_row_marker(&harness, first_id), chosen);
    assert_ne!(
        first_bg, second_bg,
        "the chosen preview has its selected tint"
    );
    assert_ne!(history_row_marker(&harness, second_id), chosen);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active")
    };
    assert_eq!(history.selected, 0);
    assert_eq!(
        history.selected_query().as_deref(),
        Some(first_sql.as_str())
    );

    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(history_row_marker(&harness, first_id), chosen);
    let _ = harness.key(KeyCode::Down);
    assert_ne!(history_row_marker(&harness, first_id), chosen);
    assert_eq!(history_row_marker(&harness, second_id), chosen);
    let first_bg_after_move = harness.cell(first_row.x.saturating_add(3), first_row.y).bg;
    let second_bg_after_move = harness
        .cell(second_row.x.saturating_add(3), second_row.y)
        .bg;
    assert_eq!(first_bg_after_move, second_bg);
    assert_eq!(second_bg_after_move, first_bg);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after moving the preview")
    };
    assert_eq!(history.selected, 1);
    assert_eq!(
        history.selected_query().as_deref(),
        Some(second_sql.as_str())
    );
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert_eq!(harness.app().workbench.tabs().len(), tab_count + 1);
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::History(_))
    ));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_native_down_enter_opens_the_selected_full_sql_without_execution() {
    for (width, height) in [(120, 40), (80, 24)] {
        let older_sql =
            "SELECT duplicate\nFROM older_source\nWHERE note = '尾部🙂' /* exact suffix */";
        let newer_sql = "SELECT duplicate\nFROM newer_source\nWHERE note = 'different'";
        let mut app = connected();
        app.workbench.history = History::default();
        app.workbench.history.push(history_entry(older_sql));
        app.workbench.history.push(history_entry(newer_sql));
        let before_queries = app.workbench.query_counter;
        let before_history = app.workbench.history.entries.len();
        let connection = app.workbench.connection.name.clone();

        let mut harness = Harness::new(app, Theme::junie(), width, height);
        let _ = harness.ctrl('y');
        let history_key = harness
            .app()
            .workbench
            .active_key()
            .expect("History is active");
        let history_list = Id::root("tablepro.workbench.history.list");
        assert_eq!(harness.focus(), Some(history_list));
        assert_eq!(harness.app().workbench.history.entries[0].sql, newer_sql);
        assert_eq!(harness.app().workbench.history.entries[1].sql, older_sql);

        let _ = harness.key(KeyCode::Down);
        let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
            panic!("History remains active before opening")
        };
        assert_eq!(history.selected, 1);
        let _ = harness.key(KeyCode::Enter);

        let query_key = harness
            .app()
            .workbench
            .active_key()
            .expect("opening allocates one new query tab");
        let Some(Tab::Query(query)) = harness.app().workbench.tab(query_key) else {
            panic!("the opened tab is an editable query")
        };
        assert_eq!(query.query, older_sql);
        assert_eq!(query.saved_text, older_sql);
        assert_eq!(query.editor_state.editor().text(), older_sql);
        assert!(query.result.is_none());
        assert!(query.error.is_none());
        assert!(query.plan.is_none());
        assert!(!query.running);
        assert!(query.last_status.is_none());
        assert_eq!(harness.app().workbench.query_counter, before_queries + 1);
        assert_eq!(
            harness.app().workbench.history.entries.len(),
            before_history
        );
        assert_eq!(harness.app().workbench.connection.name, connection);
        assert_eq!(harness.focus(), harness.app().query_id());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );

        let _ = harness.ctrl('y');
        assert_eq!(harness.app().workbench.active_key(), Some(history_key));
        assert_eq!(harness.focus(), Some(history_list));
        let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
            panic!("the existing History tab is retained")
        };
        assert_eq!(history.selected, 1);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn history_native_bootstrap_falls_back_to_the_last_current_entry() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT first\nFROM first_source"));
    app.workbench
        .history
        .push(history_entry("SELECT last\nFROM last_source"));
    let history_key = app.workbench.open_history().expect("History tab opens");
    let (last_id, last_sql, entry_count) = {
        let Some(Tab::History(history)) = app.workbench.tab_mut(history_key) else {
            panic!("the newly opened tab is History")
        };
        history.selected = usize::MAX;
        let last = history
            .entries
            .last()
            .expect("the current History source has entries");
        (last.id, last.sql.clone(), history.entries.len())
    };

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_list = Id::root("tablepro.workbench.history.list");
    assert_eq!(harness.focus(), Some(history_list));
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active")
    };
    assert_eq!(history.selected, entry_count - 1);
    assert_eq!(history.entries[history.selected].id, last_id);
    assert_eq!(history.selected_query().as_deref(), Some(last_sql.as_str()));
    assert!(
        harness
            .area_of_part(
                history_list,
                PartRef::item(Part::ROW, ItemKey::num(last_id as u64)),
            )
            .is_some()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_native_home_page_down_end_tracks_more_than_ten_rows() {
    let mut app = connected();
    app.workbench.history = History::default();
    for index in 0..32 {
        app.workbench.history.push(history_entry(&format!(
            "SELECT row_{index:02}\nFROM detail_{index:02}\nWHERE id = {index}; -- full suffix {index:02}"
        )));
    }
    let oldest_sql = app.workbench.history.entries[31].sql.clone();
    let before_queries = app.workbench.query_counter;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_list = Id::root("tablepro.workbench.history.list");
    assert_eq!(harness.focus(), Some(history_list));

    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::PageDown);
    let _ = harness.key(KeyCode::End);
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History remains active");
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after list navigation")
    };
    assert_eq!(history.selected, 31);
    assert!(harness.find("FROM detail_00").is_some());

    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("End opens the selected History query")
    };
    assert_eq!(query.query, oldest_sql);
    assert_eq!(harness.app().workbench.query_counter, before_queries + 1);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_pointer_selects_by_entry_id_and_real_open_button_opens_full_sql() {
    let older_sql = "SELECT duplicate\nFROM older_source\nWHERE id = 101; /* pointer target */";
    let newer_sql = "SELECT duplicate\nFROM newer_source\nWHERE id = 202; /* another target */";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(older_sql));
    app.workbench.history.push(history_entry(newer_sql));
    let older_id = app.workbench.history.entries[1].id;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let history_list = Id::root("tablepro.workbench.history.list");
    let older_row = PartRef::item(Part::ROW, ItemKey::num(older_id as u64));
    assert!(harness.area_of_part(history_list, older_row).is_some());
    let _ = harness.click_part(history_list, older_row);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after row click")
    };
    assert_eq!(history.selected, 1);

    let open = history_key.control("history.open");
    let open_area = harness.area_of(open).expect("the real Open button fits");
    assert!(open_area.width > 0 && open_area.height > 0);
    let _ = harness.click_id(open);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("the real Open button opens a query tab")
    };
    assert_eq!(query.query, older_sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_copy_button_uses_a_fully_visible_native_area_at_minimum_and_preferred_sizes() {
    for (width, height, row_offset) in [(72, 20, 5), (120, 40, 11)] {
        let sql =
            "SELECT visible_copy\nFROM warehouse.events\nWHERE note = '尾部🙂' /* full suffix */";
        let mut app = connected();
        app.workbench.history = History::default();
        app.workbench.history.push(history_entry(sql));
        let (mut harness, copies) = clipboard_harness(app, width, height);
        let _ = harness.ctrl('y');
        let history_key = harness
            .app()
            .workbench
            .active_key()
            .expect("History is active");
        let list_id = Id::root("tablepro.workbench.history.list");
        let list_area = harness
            .area_of(list_id)
            .expect("the History List is visible");
        let copy_id = history_key.control("history.copy");
        let copy_area = harness
            .area_of(copy_id)
            .expect("the native Copy owner is fully visible");

        assert_eq!(
            copy_area.width, 6,
            "Button intrinsic width includes its pads"
        );
        assert_eq!(copy_area.height, 1);
        assert_eq!(copy_area.x, list_area.x.saturating_add(44));
        assert_eq!(copy_area.y, list_area.y.saturating_add(row_offset));
        assert!(harness.row(copy_area.y).contains("Copy"));
        assert!(copy_area.intersection(list_area).is_empty());
        assert!(
            harness
                .area_of(history_key.control("history.search"))
                .is_some()
        );
        assert!(harness.text().contains("Connection"));
        assert!(harness.text().contains("Duration"));
        if width == 120 {
            assert!(harness.text().contains("Rows"));
        }
        if width == 120 {
            let open_area = harness
                .area_of(history_key.control("history.open"))
                .expect("the existing Open owner remains visible");
            assert!(copy_area.intersection(open_area).is_empty());
        }
        assert!(harness.tab_to(copy_id), "Copy is in the native focus ring");
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(copies.borrow().as_slice(), &[sql.to_owned()]);
        assert_eq!(harness.focus(), Some(copy_id));
        let _ = harness.click_id(copy_id);
        assert_eq!(
            copies.borrow().as_slice(),
            &[sql.to_owned(), sql.to_owned()]
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn history_copy_owner_is_absent_below_the_visible_workbench_minimum() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT hidden copy"));
    let (mut harness, copies) = clipboard_harness(app, 71, 20);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    assert!(
        harness
            .area_of(history_key.control("history.copy"))
            .is_none()
    );
    let _ = harness.key(KeyCode::Char('y'));
    assert!(copies.borrow().is_empty());
}

#[test]
fn history_y_and_native_copy_button_copy_the_selected_full_sql_once() {
    let target = "SELECT duplicate\nFROM older_source\nWHERE note = '尾部🙂' /* exact suffix beyond preview */";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(target));
    app.workbench
        .history
        .push(history_entry("SELECT duplicate\nFROM newer_source"));
    let query_count = app.workbench.query_counter;
    let history_count = app.workbench.history.entries.len();
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let list_id = Id::root("tablepro.workbench.history.list");
    assert_eq!(harness.focus(), Some(list_id));
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char('y'));

    assert_eq!(copies.borrow().as_slice(), &[target.to_owned()]);
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert_eq!(harness.app().workbench.history.entries.len(), history_count);
    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.focus(), Some(list_id));
    assert_eq!(harness.app().status(), "Copied full SQL");

    let copy_id = history_key.control("history.copy");
    assert!(
        harness.tab_to(copy_id),
        "the native Copy Button is focusable"
    );
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(copies.borrow().len(), 2);
    assert_eq!(copies.borrow()[1], target);

    let _ = harness.click_id(copy_id);
    assert_eq!(copies.borrow().len(), 3);
    assert_eq!(copies.borrow()[2], target);
    assert_eq!(harness.focus(), Some(copy_id));
    let _ = harness.tick();
    assert_eq!(copies.borrow().len(), 3, "a tick does not repeat the copy");
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert_eq!(harness.app().workbench.history.entries.len(), history_count);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("Copy leaves History active")
    };
    assert_eq!(history.selected_query().as_deref(), Some(target));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_y_does_not_copy_from_search_query_or_modal_focus() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT copy guard"));
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");

    let _ = harness.ctrl('o');
    let switcher = Id::root("tablepro.quick-switcher");
    assert!(harness.is_open(switcher));
    let _ = harness.key(KeyCode::Char('y'));
    assert!(
        copies.borrow().is_empty(),
        "modal input cannot copy History"
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.is_open(switcher),
        "the first Esc clears the modal query"
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(switcher));

    let search_id = history_key.control("history.search");
    assert!(harness.tab_to(search_id), "History search is focusable");
    let _ = harness.key(KeyCode::Char('y'));
    assert!(
        copies.borrow().is_empty(),
        "search text cannot copy History"
    );

    let _ = harness.ctrl('t');
    let query_id = harness.app().query_id().expect("a query tab opens");
    assert!(harness.tab_to(query_id), "the query editor is focusable");
    let _ = harness.key(KeyCode::Char('y'));
    assert!(copies.borrow().is_empty(), "query text cannot copy History");
}

#[test]
fn history_right_opens_scrollable_full_detail_without_history_side_effects() {
    let sql_tail = "SELECT_SQL_TAIL_SENTINEL WHERE note = '尾部🙂'";
    let error_tail = "ERROR_TAIL_SENTINEL: retained full error detail";
    let long_sql = (0..80)
        .map(|row| format!("FROM event_{row} WHERE marker = 'row_{row}'"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut entry = history_entry(&format!("SELECT common_prefix\n{long_sql}\n{sql_tail}"));
    entry.connection = "Production · 東京".to_owned();
    entry.database = "ledger_full_name".to_owned();
    entry.schema = "tenant_full_name".to_owned();
    entry.error = Some(format!("first error line\n{error_tail}"));
    entry.source = HistorySource::Explain;

    for (width, height) in [(72, 20), (120, 40)] {
        let mut app = connected();
        app.workbench.history = History::default();
        app.workbench.history.push(entry.clone());
        let query_count = app.workbench.query_counter;
        let history_count = app.workbench.history.entries.len();
        let (mut harness, copies) = clipboard_harness(app, width, height);
        let _ = harness.ctrl('y');
        let history_key = harness
            .app()
            .workbench
            .active_key()
            .expect("History is active");
        let list = Id::root("tablepro.workbench.history.list");
        assert_eq!(harness.focus(), Some(list));

        let _ = harness.key(KeyCode::Right);
        let detail = Id::root("tablepro.history-detail");
        let text = detail.sub("text");
        let detail_close = detail.part(Part::CLOSE);
        let detail_area = harness.area_of(detail).expect("native detail dialog opens");
        let text_area = harness
            .area_of(text)
            .expect("native wrapped viewport opens");
        assert!(!text_area.is_empty());
        assert_eq!(detail_area.intersection(text_area), text_area);
        assert_eq!(
            detail_area.intersection(termrock::Rect::new(0, 0, width, height)),
            detail_area
        );
        assert!(harness.text().contains("Connection: Production · 東京"));
        assert!(harness.text().contains("Database: ledger_full_name"));
        assert!(harness.text().contains("Ctrl+C"));
        assert!(
            !harness.text().contains(error_tail),
            "the first viewport page begins at the detail header"
        );

        let _ = harness.key(KeyCode::End);
        assert!(harness.text().contains(sql_tail));
        assert!(harness.text().contains(error_tail));
        assert_history_detail_copy_help_outside_text(&harness);
        let _ = harness.key(KeyCode::Home);
        assert!(harness.text().contains("Source: Explain"));
        assert!(!harness.text().contains(error_tail));

        if width == 120 {
            let _ = harness.resize(96, 32);
            let resized_detail = harness
                .area_of(detail)
                .expect("detail survives a fitting resize");
            let resized_text = harness
                .area_of(text)
                .expect("viewport survives a fitting resize");
            let resized_screen = termrock::Rect::new(0, 0, 96, 32);
            assert_eq!(resized_detail.intersection(resized_screen), resized_detail);
            assert_eq!(resized_text.intersection(resized_detail), resized_text);
            let _ = harness.key(KeyCode::End);
            assert!(harness.text().contains(sql_tail));
            assert!(harness.text().contains(error_tail));
            let _ = harness.resize(120, 40);
            let _ = harness.key(KeyCode::End);
            assert!(harness.text().contains(sql_tail));
            assert!(harness.text().contains(error_tail));

            let _ = harness.resize(71, 20);
            assert!(
                harness.area_of(detail).is_none(),
                "the viewport owner is dismissed below the app minimum"
            );
            assert_ne!(
                harness.focus(),
                Some(text),
                "below-min close cannot retain stale viewport focus",
            );
            assert_ne!(
                harness.focus(),
                Some(detail_close),
                "below-min close cannot retain stale close focus",
            );
            let _ = harness.resize(72, 20);
            assert!(
                harness.area_of(detail).is_none(),
                "the detail stays dismissed through minimum-size recovery"
            );
            assert_ne!(
                harness.focus(),
                Some(text),
                "minimum-size recovery cannot retain stale viewport focus",
            );
            assert_ne!(
                harness.focus(),
                Some(detail_close),
                "minimum-size recovery cannot retain stale close focus",
            );
            assert!(
                harness.tab_to(list),
                "native Tab navigation returns to the History list before reopen",
            );
            assert_eq!(harness.focus(), Some(list));
            let _ = harness.key(KeyCode::Right);
            let recovered_detail = harness
                .area_of(detail)
                .expect("Right reopens detail after recovering the minimum size");
            let recovered_text = harness
                .area_of(text)
                .expect("the full-text viewport is restored after reopening");
            let recovered_screen = termrock::Rect::new(0, 0, 72, 20);
            assert_eq!(
                recovered_detail.intersection(recovered_screen),
                recovered_detail
            );
            assert_eq!(
                recovered_text.intersection(recovered_detail),
                recovered_text
            );
            assert!(harness.text().contains("Connection: Production · 東京"));
            assert!(
                !harness.text().contains(error_tail),
                "reopened detail starts at its header after minimum-size recovery"
            );
            let _ = harness.key(KeyCode::End);
            assert!(harness.text().contains(sql_tail));
            assert!(harness.text().contains(error_tail));
        } else {
            let _ = harness.key(KeyCode::Esc);
            assert!(harness.area_of(detail).is_none());
            assert_eq!(harness.focus(), Some(list));
        }
        assert_eq!(harness.app().workbench.active_key(), Some(history_key));
        assert_eq!(harness.app().workbench.query_counter, query_count);
        assert_eq!(harness.app().workbench.history.entries.len(), history_count);
        assert!(copies.borrow().is_empty());
    }
}

const HISTORY_DETAIL_ID: Id = Id::root("tablepro.history-detail");
const HISTORY_DETAIL_TEXT: Id = HISTORY_DETAIL_ID.sub("text");
const HISTORY_DETAIL_CLOSE: Id = HISTORY_DETAIL_ID.part(Part::CLOSE);
const HISTORY_DETAIL_COPY_HELP: &str = "Ctrl+C Copy selection · y Copy selection";
const FOREIGN_DETAIL_FOCUS: Id = Id::root("tablepro.history-detail.foreign-focus");

#[test]
fn history_detail_first_end_after_open_scrolls_without_pointer_redraw() {
    let sql_tail = "DETAIL_FIRST_END_SQL_TAIL";
    let error_tail = "DETAIL_FIRST_END_ERROR_TAIL";
    let rows = (0..48)
        .map(|row| format!("FROM detail_end_row_{row}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = connected();
    app.workbench.history = History::default();
    let mut entry = history_entry(&format!("SELECT first page\n{rows}\n{sql_tail}"));
    entry.error = Some(format!("first error line\n{error_tail}"));
    app.workbench.history.push(entry);
    let (mut harness, copies) = clipboard_harness(app, 72, 20);
    let _ = harness.ctrl('y');
    let _ = harness.key(KeyCode::Right);
    assert!(harness.is_open(HISTORY_DETAIL_ID));

    // End is the first keyboard event after opening; no mouse, area probe, or redraw intervenes.
    let _ = harness.key(KeyCode::End);
    assert!(harness.text().contains(sql_tail));
    assert!(harness.text().contains(error_tail));
    assert!(copies.borrow().is_empty());
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

fn history_detail_harness(
    sql: &str,
    error: Option<&str>,
) -> (Harness<ClipboardProbeApp>, Rc<RefCell<Vec<String>>>, Id) {
    let mut app = connected();
    app.workbench.history = History::default();
    let mut entry = history_entry(sql);
    entry.error = error.map(str::to_owned);
    app.workbench.history.push(entry);
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let _ = harness.key(KeyCode::Right);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    let list = Id::root("tablepro.workbench.history.list");
    (harness, copies, list)
}

fn select_detail_word(harness: &mut Harness<ClipboardProbeApp>, word: &str) {
    let (x, y) = harness.find(word).expect("the detail word is visible");
    let _ = harness.double_click(x, y);
}

fn assert_history_detail_copy_help_outside_text(harness: &Harness<ClipboardProbeApp>) {
    let text_area = harness
        .area_of(HISTORY_DETAIL_TEXT)
        .expect("the native text viewport is present");
    assert!(!text_area.is_empty());
    let (_, help_y) = harness
        .find(HISTORY_DETAIL_COPY_HELP)
        .expect("persistent Copy help remains visible");
    let text_bottom = text_area.y.saturating_add(text_area.height);
    assert!(
        help_y < text_area.y || help_y >= text_bottom,
        "persistent Copy help is outside the scrollable text viewport"
    );
}

fn assert_history_detail_diagnostics_clean(diagnostics: &[Diagnostic]) {
    assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::UndeliveredIntent { owner }
                if *owner == HISTORY_DETAIL_ID
                    || *owner == HISTORY_DETAIL_TEXT
                    || *owner == HISTORY_DETAIL_CLOSE
        )),
        "detail lifecycle intents must be serviced by their exact owner: {diagnostics:?}"
    );
    assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::DuplicateLayerDraw { id } if *id == HISTORY_DETAIL_ID
        )),
        "a stale same-ID renderer must not claim the replacement draw slot: {diagnostics:?}"
    );
}

#[test]
fn history_detail_empty_ctrl_c_does_not_quit_or_copy() {
    let (mut harness, copies, _) = history_detail_harness("DETAIL_EMPTY_SELECTION", None);
    let _ = harness.ctrl('c');
    assert!(copies.borrow().is_empty());
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert!(!harness.app().should_quit());
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_detail_ctrl_c_copies_the_selected_sql_word_once() {
    let target = "DETAILSQLTARGET";
    let (mut harness, copies, _) = history_detail_harness(target, None);
    select_detail_word(&mut harness, target);
    let _ = harness.ctrl('c');
    assert_eq!(copies.borrow().as_slice(), &[target.to_owned()]);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_detail_native_y_copies_the_selected_error_word_once() {
    let target = "DETAILERRORTARGET";
    let (mut harness, copies, _) = history_detail_harness("SELECT detail error copy", Some(target));
    select_detail_word(&mut harness, target);
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(copies.borrow().as_slice(), &[target.to_owned()]);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_detail_selection_first_escape_clears_then_closes_and_restores_focus() {
    let target = "DETAILESCAPETARGET";
    for (width, height) in [(72, 20), (120, 40)] {
        let (mut harness, copies, list) = history_detail_harness(target, None);
        if (width, height) == (72, 20) {
            let _ = harness.resize(width, height);
        }
        select_detail_word(&mut harness, target);
        let selected_frame = harness.text().to_owned();
        if width == 72 {
            assert!(selected_frame.contains(HISTORY_DETAIL_COPY_HELP));
            assert_history_detail_copy_help_outside_text(&harness);
        }
        for hint in ["Ctrl+C", "Copy selection", "Esc", "Clear selection"] {
            assert!(
                selected_frame.contains(hint),
                "selected {hint:?} hint is fully visible at {width}x{height}:\n{selected_frame}"
            );
        }
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.is_open(HISTORY_DETAIL_ID));
        assert!(copies.borrow().is_empty());
        let cleared_frame = harness.text().to_owned();
        for hint in ["Ctrl+C", "Copy selection", "Esc", "Close"] {
            assert!(
                cleared_frame.contains(hint),
                "cleared {hint:?} hint is fully visible at {width}x{height}:\n{cleared_frame}"
            );
        }
        let _ = harness.key(KeyCode::Esc);
        assert!(!harness.is_open(HISTORY_DETAIL_ID));
        assert_eq!(harness.focus(), Some(list));
        assert_history_detail_diagnostics_clean(harness.diagnostics());
    }
}

#[test]
fn history_detail_native_close_restores_history_focus_without_cancel_leaks() {
    let (mut harness, copies, list) = history_detail_harness("SELECT native close", None);
    assert!(harness.area_of(HISTORY_DETAIL_CLOSE).is_some());
    let _ = harness.click_id(HISTORY_DETAIL_CLOSE);
    assert!(!harness.is_open(HISTORY_DETAIL_ID));
    let _ = harness.tick();
    assert_eq!(harness.focus(), Some(list));
    let _ = harness.ctrl('c');
    assert!(copies.borrow().is_empty());
    assert!(
        harness.app().should_quit(),
        "root Ctrl+C remains active after native detail close"
    );
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_detail_closes_when_owner_becomes_inactive_and_then_closes() {
    let old_sql = "SELECT INACTIVE_DETAIL_OWNER_A";
    let (mut harness, copies, _) = history_detail_harness(old_sql, None);
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History A is active");
    assert!(harness.is_open(HISTORY_DETAIL_ID));

    let query_key = harness
        .app_mut()
        .workbench
        .new_query("SELECT ACTIVE_OWNER_B")
        .expect("the replacement active Query tab opens");
    assert_ne!(query_key, history_key);
    let _ = harness.tick();
    assert_eq!(harness.app().workbench.active_key(), Some(query_key));
    assert!(harness.app().workbench.tab(query_key).is_some());
    assert!(
        !harness.is_open(HISTORY_DETAIL_ID),
        "detail owned by inactive History A closes after Query B becomes active"
    );
    assert!(
        harness.area_of(HISTORY_DETAIL_ID).is_none(),
        "no stale detail area remains"
    );
    assert_ne!(harness.focus(), Some(HISTORY_DETAIL_TEXT));
    assert!(copies.borrow().is_empty());

    let history_index = harness
        .app()
        .workbench
        .tabs()
        .iter()
        .position(|tab| tab.key() == history_key)
        .expect("inactive History A remains closable");
    assert!(
        harness.app_mut().workbench.close_tab(history_index),
        "closing inactive History A does not affect active Query B"
    );
    let _ = harness.tick();
    assert!(harness.app().workbench.tab(history_key).is_none());
    assert_eq!(harness.app().workbench.active_key(), Some(query_key));
    assert!(harness.area_of(HISTORY_DETAIL_ID).is_none());
    assert!(copies.borrow().is_empty());
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_detail_rejects_colliding_replacement_workbench_and_restores_quit() {
    let old_sql = "SELECT OLD_WORKBENCH_DETAIL_OWNER";
    let target = "NEW_WORKBENCH_DETAIL_COPY_TARGET";
    let replacement_sql = format!("SELECT {target} FROM replacement_owner");
    let (mut harness, copies, list) = history_detail_harness(old_sql, None);
    let old_key = harness
        .app()
        .workbench
        .active_key()
        .expect("old History owner is active");
    assert!(harness.is_open(HISTORY_DETAIL_ID));

    let mut replacement = connected();
    replacement.workbench.history = History::default();
    replacement
        .workbench
        .history
        .push(history_entry(&replacement_sql));
    let replacement_key = replacement
        .workbench
        .open_history()
        .expect("replacement History opens");
    assert_eq!(
        replacement_key, old_key,
        "a different Workbench generation deliberately reuses the same TabKey"
    );

    harness.app_mut().workbench = replacement.workbench;
    harness.draw();
    let _ = harness.tick();
    assert_eq!(harness.app().workbench.active_key(), Some(old_key));
    assert!(!harness.is_open(HISTORY_DETAIL_ID));
    assert!(
        harness.area_of(HISTORY_DETAIL_ID).is_none(),
        "the old generation's modal area must not survive the key collision"
    );
    assert!(!harness.text().contains("OLD_WORKBENCH_DETAIL_OWNER"));
    assert!(copies.borrow().is_empty());
    assert!(!harness.app().should_quit());

    assert!(
        harness.tab_to(list),
        "replacement History list can regain focus"
    );
    let _ = harness.key(KeyCode::Right);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert!(harness.text().contains(&replacement_sql));
    assert!(!harness.text().contains(old_sql));
    select_detail_word(&mut harness, target);
    let _ = harness.ctrl('c');
    assert_eq!(copies.borrow().as_slice(), &[target.to_owned()]);
    assert_eq!(copies.borrow().len(), 1, "the new generation copies once");

    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.is_open(HISTORY_DETAIL_ID),
        "Escape clears the new selection first"
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(HISTORY_DETAIL_ID));
    assert_eq!(harness.focus(), Some(list));
    let _ = harness.ctrl('c');
    assert!(
        harness.app().should_quit(),
        "root Ctrl+C is restored after the colliding Workbench detail closes"
    );
    assert_eq!(copies.borrow().len(), 1);
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

const COVERING_DETAIL_ID: Id = Id::root("tablepro.history-detail.cover-probe");
const COVERING_DETAIL_FOCUS: Id = Id::root("tablepro.history-detail.cover-probe.focus");

struct CoveringHistoryDetailProbeApp {
    app: ClipboardProbeApp,
    detail_seen: bool,
    cover_opening: Option<LayerOpening>,
}

impl App for CoveringHistoryDetailProbeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = App::update(&mut self.app, cx);
        if self.cover_opening.is_none() && cx.layer_opening(HISTORY_DETAIL_ID).is_some() {
            if self.detail_seen {
                self.cover_opening = Some(
                    cx.open_layer_with_token(
                        COVERING_DETAIL_ID,
                        LayerSpec::modal(COVERING_DETAIL_ID)
                            .size(LayerSize::Fixed(30, 8))
                            .initial_focus(COVERING_DETAIL_FOCUS)
                            .restore_focus(true),
                    )
                    .expect("the test cover receives its exact opening token"),
                );
                response |= Response::changed();
            } else {
                self.detail_seen = true;
            }
        }

        if let Some(opening) = self.cover_opening {
            if cx.layer_opening(COVERING_DETAIL_ID) == Some(opening) {
                let close_requested = cx.intents(COVERING_DETAIL_FOCUS).any(
                    |intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')),
                );
                if close_requested {
                    let _ = cx.close_layer(COVERING_DETAIL_ID, None);
                    response |= Response::changed();
                } else {
                    response |= Button::new(COVERING_DETAIL_FOCUS, "Cover")
                        .update(cx)
                        .erase();
                }
            }
            while cx.take_layer_intent_receipt_for(opening).is_some() {}
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        App::draw(&self.app, ui);
        let Some(opening) = self.cover_opening else {
            return;
        };
        let _ = ui.layer_for(opening, |ui, area| {
            if !area.is_empty() {
                let _ = Button::new(COVERING_DETAIL_FOCUS, "Cover").draw(ui, area);
            }
        });
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        App::on_esc(&mut self.app, cx)
    }

    fn should_quit(&self) -> bool {
        App::should_quit(&self.app)
    }

    fn keymap(&self) -> &KeyMap {
        App::keymap(&self.app)
    }

    fn min_size(&self) -> Size {
        App::min_size(&self.app)
    }
}

#[test]
fn history_detail_cover_preserves_scroll_copy_and_focus_ownership() {
    let target = "COVERED_DETAIL_COPY_TARGET";
    let sql = format!(
        "{}\n{target}",
        (0..80)
            .map(|line| format!("SELECT cover_line_{line} WHERE owner = 'detail'"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(&sql));
    let copies = Rc::new(RefCell::new(Vec::new()));
    let probe = CoveringHistoryDetailProbeApp {
        app: ClipboardProbeApp {
            app,
            copies: copies.clone(),
        },
        detail_seen: false,
        cover_opening: None,
    };
    let mut harness = Harness::new(probe, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let list = Id::root("tablepro.workbench.history.list");
    assert_eq!(harness.focus(), Some(list));
    let _ = harness.key(KeyCode::Right);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    let _ = harness.tick();
    assert!(harness.is_open(COVERING_DETAIL_ID));
    assert_eq!(harness.focus(), Some(COVERING_DETAIL_FOCUS));
    let cover_area = harness
        .layer_area(COVERING_DETAIL_ID)
        .expect("cover is open");
    assert_eq!((cover_area.width, cover_area.height), (30, 8));
    assert!(!harness.text().contains(target));

    let _ = harness.key(KeyCode::End);
    assert!(
        !harness.text().contains(target),
        "End is not routed into the covered detail viewport"
    );
    let _ = harness.ctrl('c');
    assert!(
        copies.borrow().is_empty(),
        "covered selection state cannot copy"
    );
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert!(harness.is_open(COVERING_DETAIL_ID));
    assert!(
        !harness.app().should_quit(),
        "covered Ctrl+C does not quit the app"
    );

    let _ = harness.key(KeyCode::Char('x'));
    assert!(!harness.is_open(COVERING_DETAIL_ID));
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    let _ = harness.tick();
    assert_eq!(harness.focus(), Some(HISTORY_DETAIL_TEXT));
    assert!(
        !harness.text().contains(target),
        "cover close preserves the pre-cover scroll"
    );
    let _ = harness.key(KeyCode::End);
    assert!(
        harness.text().contains(target),
        "uncovered detail scrolls normally"
    );
    let (x, y) = harness.find(target).expect("the detail word is visible");
    let _ = harness.double_click(x, y);
    let _ = harness.ctrl('c');
    assert_eq!(copies.borrow().as_slice(), &[target.to_owned()]);
    assert_eq!(copies.borrow().len(), 1);

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(HISTORY_DETAIL_ID));
    assert_eq!(harness.focus(), Some(list));
    let _ = harness.ctrl('c');
    assert!(
        harness.app().should_quit(),
        "root keymap is restored after uncover and close"
    );
    assert_eq!(copies.borrow().len(), 1);
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

struct ForeignHistoryDetailProbeApp {
    app: TableProApp,
    old_opening: Option<LayerOpening>,
    foreign_opening: Option<LayerOpening>,
    foreign_draws: Cell<usize>,
}

impl App for ForeignHistoryDetailProbeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = App::update(&mut self.app, cx);
        if self.foreign_opening.is_none()
            && let Some(old) = cx.layer_opening(HISTORY_DETAIL_ID)
        {
            let _ = cx.close_layer(HISTORY_DETAIL_ID, None);
            let foreign = cx
                .open_layer_with_token(
                    HISTORY_DETAIL_ID,
                    LayerSpec::modal(HISTORY_DETAIL_ID)
                        .size(LayerSize::Fixed(24, 7))
                        .initial_focus(FOREIGN_DETAIL_FOCUS)
                        .restore_focus(false),
                )
                .expect("the same-ID foreign opening replaces the stale detail");
            self.old_opening = Some(old);
            self.foreign_opening = Some(foreign);
            response |= Response::changed();
        }
        if let Some(foreign) = self.foreign_opening
            && cx.layer_opening(HISTORY_DETAIL_ID) == Some(foreign)
        {
            let close_requested = cx
                .intents(FOREIGN_DETAIL_FOCUS)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')));
            if close_requested {
                let _ = cx.close_layer(HISTORY_DETAIL_ID, None);
                response |= Response::changed();
            } else {
                response |= Button::new(FOREIGN_DETAIL_FOCUS, "Foreign")
                    .update(cx)
                    .erase();
            }
        }
        if let Some(foreign) = self.foreign_opening {
            // Retire only this fixture's token even after its layer has closed.
            while cx.take_layer_intent_receipt_for(foreign).is_some() {}
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        App::draw(&self.app, ui);
        let Some(foreign) = self.foreign_opening else {
            return;
        };
        let draws = &self.foreign_draws;
        let _ = ui.layer(HISTORY_DETAIL_ID, |ui, area| {
            if ui.layer_id() == foreign.layer_id() && !area.is_empty() {
                let _ = Button::new(FOREIGN_DETAIL_FOCUS, "Foreign").draw(ui, area);
                draws.set(draws.get().saturating_add(1));
            }
        });
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        App::on_esc(&mut self.app, cx)
    }
    fn should_quit(&self) -> bool {
        App::should_quit(&self.app)
    }
    fn keymap(&self) -> &KeyMap {
        App::keymap(&self.app)
    }
    fn min_size(&self) -> Size {
        App::min_size(&self.app)
    }
}

#[test]
fn history_detail_stale_same_id_draw_preserves_foreign_focus_size_and_close() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT foreign owner"));
    let mut harness = Harness::new(
        ForeignHistoryDetailProbeApp {
            app,
            old_opening: None,
            foreign_opening: None,
            foreign_draws: Cell::new(0),
        },
        Theme::junie(),
        120,
        40,
    );
    let _ = harness.ctrl('y');
    let _ = harness.key(KeyCode::Right);

    assert!(harness.is_open(HISTORY_DETAIL_ID));
    let probe = harness.app();
    let old = probe.old_opening.expect("product detail opening");
    let foreign = probe.foreign_opening.expect("foreign replacement opening");
    assert_ne!(old, foreign);
    assert!(
        probe.foreign_draws.get() > 0,
        "foreign renderer owns the draw slot"
    );
    assert_eq!(harness.focus(), Some(FOREIGN_DETAIL_FOCUS));
    let area = harness
        .layer_area(HISTORY_DETAIL_ID)
        .expect("foreign layer is live");
    assert_eq!((area.width, area.height), (24, 7));
    assert_history_detail_diagnostics_clean(harness.diagnostics());

    let _ = harness.resize(90, 30);
    let _ = harness.tick();
    assert!(harness.is_open(HISTORY_DETAIL_ID));
    assert_eq!(harness.focus(), Some(FOREIGN_DETAIL_FOCUS));
    let resized = harness
        .layer_area(HISTORY_DETAIL_ID)
        .expect("foreign layer survives resize");
    assert_eq!((resized.width, resized.height), (24, 7));
    assert_history_detail_diagnostics_clean(harness.diagnostics());

    let _ = harness.key(KeyCode::Char('x'));
    assert!(!harness.is_open(HISTORY_DETAIL_ID));
    let _ = harness.ctrl('c');
    assert!(
        harness.app().should_quit(),
        "root Ctrl+C remains active after foreign replacement closes"
    );
    assert_history_detail_diagnostics_clean(harness.diagnostics());
}

#[test]
fn history_copy_tracks_search_connection_and_failure_projections() {
    let mut app = connected();
    app.workbench.history = History::default();
    let current_connection = app.workbench.connection.name.clone();
    let stage_ok = "SELECT copy_projection_stage_ok";
    let production_ok = "SELECT copy_projection_production_ok";
    let production_failed = "SELECT copy_projection_production_failed";
    let stage_failed = "SELECT copy_projection_stage_failed";
    app.workbench
        .history
        .push(history_entry_for(stage_ok, "Staging", false));
    app.workbench
        .history
        .push(history_entry_for(production_ok, &current_connection, false));
    app.workbench.history.push(history_entry_for(
        production_failed,
        &current_connection,
        true,
    ));
    app.workbench
        .history
        .push(history_entry_for(stage_failed, "Staging", true));

    let source_count = app.workbench.history.entries.len();
    let query_count = app.workbench.query_counter;
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let search = history_key.control("history.search");
    let scope = history_key.control("history.connection");
    let status = history_key.control("history.status");
    let list = Id::root("tablepro.workbench.history.list");
    assert!(harness.tab_to(search));
    let _ = harness.paste("copy_projection");
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(
        harness.app().workbench.tab(history_key),
        Some(Tab::History(history)) if history.search == "copy_projection" && history.entries.len() == 4
    ));

    assert!(harness.tab_to(list));
    let _ = harness.key(KeyCode::End);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active in the All search projection")
    };
    assert_eq!(history.selected_query().as_deref(), Some(stage_ok));
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(copies.borrow().as_slice(), &[stage_ok.to_owned()]);

    assert!(harness.tab_to(scope));
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active in the Current connection projection")
    };
    assert_eq!(history.entries.len(), 2);
    assert!(
        history
            .entries
            .iter()
            .all(|entry| entry.connection == current_connection)
    );
    assert!(harness.tab_to(list));
    let _ = harness.key(KeyCode::End);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after selecting a current-connection row")
    };
    assert_eq!(history.selected_query().as_deref(), Some(production_ok));
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(
        copies.borrow().as_slice(),
        &[stage_ok.to_owned(), production_ok.to_owned()]
    );

    assert!(harness.tab_to(status));
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active in the failed-only projection")
    };
    assert_eq!(history.entries.len(), 1);
    assert_eq!(history.entries[0].sql, production_failed);
    assert!(history.entries[0].error.is_some());
    assert!(harness.tab_to(list));
    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(
        copies.borrow().as_slice(),
        &[
            stage_ok.to_owned(),
            production_ok.to_owned(),
            production_failed.to_owned()
        ]
    );

    assert!(harness.tab_to(scope));
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active in the All failed-only projection")
    };
    assert_eq!(history.entries.len(), 2);
    assert!(history.entries.iter().all(|entry| entry.error.is_some()));
    assert!(harness.tab_to(list));
    let _ = harness.key(KeyCode::Home);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after selecting the All failure")
    };
    assert_eq!(history.selected_query().as_deref(), Some(stage_failed));
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(
        copies.borrow().as_slice(),
        &[
            stage_ok.to_owned(),
            production_ok.to_owned(),
            production_failed.to_owned(),
            stage_failed.to_owned()
        ]
    );
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert_eq!(harness.app().workbench.history.entries.len(), source_count);
    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_copy_from_an_empty_projection_is_disabled_and_has_no_effect() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT empty_projection_source"));
    let query_count = app.workbench.query_counter;
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let search = history_key.control("history.search");
    assert!(harness.tab_to(search));
    let _ = harness.paste("no matching copy row");
    let _ = harness.key(KeyCode::Enter);

    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active with an empty filter")
    };
    assert!(history.entries.is_empty());
    assert!(history_chosen_marker_rows(&harness).is_empty());
    let copy = history_key.control("history.copy");
    assert!(harness.area_of(copy).is_some());
    assert!(harness.state_of(copy).contains(StateFlags::DISABLED));
    assert!(harness.tab_to(Id::root("tablepro.workbench.history.list")));
    let _ = harness.key(KeyCode::Char('y'));
    let _ = harness.click_id(copy);
    assert!(copies.borrow().is_empty());
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_copy_services_an_inactive_button_before_its_tab_closes() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT inactive_copy_owner"));
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let copy = history_key.control("history.copy");
    assert!(harness.tab_to(copy));
    let _ = harness.ctrl('t');
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::Query(_))
    ));
    assert!(harness.app().workbench.tab(history_key).is_some());
    assert!(harness.area_of(copy).is_none());
    assert!(copies.borrow().is_empty());
    let _ = harness.tick();
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.click_part(
        Id::root("tablepro.workbench.tab-strip"),
        PartRef::item(Part::CLOSE, ItemKey::num(history_key.get())),
    );
    let _ = harness.tick();
    assert!(harness.app().workbench.tab(history_key).is_none());
    assert!(harness.area_of(copy).is_none());
    assert!(copies.borrow().is_empty());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_copy_owner_retires_on_close_and_reopens_with_history() {
    let sql = "SELECT retained\nFROM history_copy_owner";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(sql));
    let query_count = app.workbench.query_counter;
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let copy_id = history_key.control("history.copy");
    assert!(harness.tab_to(copy_id), "Copy receives focus before close");
    let _ = harness.click_part(
        Id::root("tablepro.workbench.tab-strip"),
        PartRef::item(Part::CLOSE, ItemKey::num(history_key.get())),
    );
    let _ = harness.tick();
    assert!(harness.app().workbench.tab(history_key).is_none());
    assert!(harness.area_of(copy_id).is_none());
    assert_ne!(harness.focus(), Some(copy_id));
    assert!(copies.borrow().is_empty());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.ctrl('y');
    let reopened_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History reopens");
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::History(_))
    ));
    let reopened_copy = reopened_key.control("history.copy");
    assert!(harness.area_of(reopened_copy).is_some());
    let list_id = Id::root("tablepro.workbench.history.list");
    assert!(harness.tab_to(list_id));
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(copies.borrow().as_slice(), &[sql.to_owned()]);
    assert_eq!(harness.app().workbench.query_counter, query_count);

    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_copy_owner_rejects_replacement_workbench_with_a_colliding_tab_key() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT old\nFROM old_owner"));
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let old_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    assert!(harness.tab_to(old_key.control("history.copy")));

    let mut replacement = connected();
    replacement.workbench.history = History::default();
    replacement
        .workbench
        .history
        .push(history_entry("SELECT replacement\nFROM new_owner"));
    let replacement_key = replacement
        .workbench
        .open_history()
        .expect("replacement History opens");
    assert_eq!(replacement_key, old_key, "the tab key collides");
    harness.app_mut().workbench = replacement.workbench;
    harness.draw();
    let _ = harness.tick();
    assert!(
        copies.borrow().is_empty(),
        "replacement does not replay Copy"
    );
    assert_eq!(harness.app().workbench.active_key(), Some(old_key));
    let list_id = Id::root("tablepro.workbench.history.list");
    assert!(harness.tab_to(list_id));
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(copies.borrow().len(), 1);
    assert_eq!(copies.borrow()[0], "SELECT replacement\nFROM new_owner");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_native_double_click_activates_the_exact_row_entry() {
    let older_sql = "SELECT duplicate\nFROM older_source\nWHERE id = 303; /* exact row */";
    let newer_sql = "SELECT duplicate\nFROM newer_source\nWHERE id = 404; /* other row */";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(older_sql));
    app.workbench.history.push(history_entry(newer_sql));
    let older_id = app.workbench.history.entries[1].id;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_list = Id::root("tablepro.workbench.history.list");
    let older_row = PartRef::item(Part::ROW, ItemKey::num(older_id as u64));
    let area = harness
        .area_of_part(history_list, older_row)
        .expect("the exact History item owns a native row part");
    let x = area.x.saturating_add(area.width / 2);
    let y = area.y.saturating_add(area.height / 2);

    let _ = harness.double_click(x, y);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("native double-click opens the selected query")
    };
    assert_eq!(query.query, older_sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_activation_keeps_the_same_entry_after_source_reordering() {
    let selected_sql = "SELECT first\nFROM selected_entry\nWHERE id = 11";
    let other_sql = "SELECT second\nFROM other_entry\nWHERE id = 22";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(other_sql));
    app.workbench.history.push(history_entry(selected_sql));
    let selected = app.workbench.history.entries[0].clone();
    let other_id = app.workbench.history.entries[1].id;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let _ = harness.ctrl('y');
    harness.draw();
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("active History tab key");
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::History(_))
    ));
    harness.app_mut().workbench.history.entries.swap(0, 1);

    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("the same exact History record remains openable after reordering")
    };
    assert_eq!(query.query, selected.sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("the existing History tab remains in the workbench")
    };
    assert_eq!(history.entries[0].id, other_id);
    assert_eq!(history.entries[1].id, selected.id);
    assert_eq!(history.entries[history.selected].id, selected.id);
    assert_eq!(
        history.selected_query().as_deref(),
        Some(selected.sql.as_str())
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_removal_reconciles_before_opening_the_current_remaining_row() {
    let removed_sql = "SELECT displayed\nFROM removed_source\nWHERE id = 55";
    let remaining_sql = "SELECT remaining\nFROM remaining_source\nWHERE id = 66";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(remaining_sql));
    app.workbench.history.push(history_entry(removed_sql));
    let removed_id = app.workbench.history.entries[0].id;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let _ = harness.ctrl('y');
    harness.draw();
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let query_count = harness.app().workbench.query_counter;
    let tab_count = harness.app().workbench.tabs().len();
    let remaining_id = harness
        .app()
        .workbench
        .history
        .entries
        .get(1)
        .expect("the remaining source is present before removal")
        .id;
    harness.app_mut().workbench.history.entries.remove(0);
    let _ = harness.tick();
    harness.draw();

    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert_eq!(harness.app().workbench.tabs().len(), tab_count);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active when its displayed source is removed")
    };
    assert_eq!(history.entries.len(), 1);
    assert_ne!(history.entries[0].id, removed_id);
    assert_eq!(history.entries[0].sql, remaining_sql);
    assert_eq!(history.selected, 0);
    assert_eq!(history.selected_query().as_deref(), Some(remaining_sql));
    assert!(harness.find("FROM remaining_source").is_some());
    let history_list = Id::root("tablepro.workbench.history.list");
    assert!(
        harness
            .area_of_part(
                history_list,
                PartRef::item(Part::ROW, ItemKey::num(removed_id as u64)),
            )
            .is_none()
    );
    assert!(
        harness
            .area_of_part(
                history_list,
                PartRef::item(Part::ROW, ItemKey::num(remaining_id as u64)),
            )
            .is_some()
    );

    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("Enter opens the current, reconciled History row")
    };
    assert_eq!(query.query, remaining_sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(!query.running);
    assert_eq!(harness.app().workbench.query_counter, query_count + 1);
    assert_eq!(harness.app().workbench.tabs().len(), tab_count + 1);
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        1,
        "opening the reconciled row does not mutate history"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_reconciles_same_id_entry_replacement_before_activation() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench.history.push(history_entry(
        "SELECT stale\nFROM original_entry\nWHERE id = 1",
    ));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let _ = harness.ctrl('y');
    harness.draw();
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let query_count = harness.app().workbench.query_counter;
    let tab_count = harness.app().workbench.tabs().len();
    harness.app_mut().workbench.history.entries[0].sql =
        "SELECT replacement\nFROM changed_entry\nWHERE id = 1".to_owned();
    harness.draw();

    let _ = harness.key(KeyCode::Enter);
    assert_ne!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.query_counter, query_count + 1);
    assert_eq!(harness.app().workbench.tabs().len(), tab_count + 1);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("activation opens the current replacement entry after redraw")
    };
    assert_eq!(
        query.query,
        "SELECT replacement\nFROM changed_entry\nWHERE id = 1"
    );
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(!query.running);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_duplicate_entry_ids_cannot_activate_an_ambiguous_row() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT first\nFROM first_source"));
    app.workbench
        .history
        .push(history_entry("SELECT second\nFROM second_source"));
    let duplicate_id = app.workbench.history.entries[0].id;
    app.workbench.history.entries[1].id = duplicate_id;
    let before_queries = app.workbench.query_counter;
    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let before_tabs = harness.app().workbench.tabs().len();
    assert!(history_chosen_marker_rows(&harness).is_empty());
    let copy_id = history_key.control("history.copy");
    assert!(harness.area_of(copy_id).is_some());
    assert!(harness.state_of(copy_id).contains(StateFlags::DISABLED));
    let _ = harness.key(KeyCode::Char('y'));
    let _ = harness.click_id(copy_id);
    assert!(copies.borrow().is_empty());
    let _ = harness.key(KeyCode::Enter);

    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.query_counter, before_queries);
    assert_eq!(harness.app().workbench.tabs().len(), before_tabs);
    assert_eq!(
        harness.focus(),
        Some(Id::root("tablepro.workbench.history.list"))
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_open_with_exhausted_query_counter_keeps_history_active() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT preserved\nFROM history"));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let _ = harness.ctrl('y');
    harness.draw();
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let tab_count = harness.app().workbench.tabs().len();
    harness.app_mut().workbench.query_counter = usize::MAX;

    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.query_counter, usize::MAX);
    assert_eq!(harness.app().workbench.tabs().len(), tab_count);
    assert_eq!(
        harness.focus(),
        Some(Id::root("tablepro.workbench.history.list"))
    );
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("the selected History tab remains active")
    };
    assert_eq!(history.selected, 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_refresh_keeps_selected_query_in_sync_with_the_native_row() {
    let selected_sql = "SELECT stable\nFROM selected_history\nWHERE id = 17";
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT other\nFROM other_history"));
    app.workbench.history.push(history_entry(selected_sql));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History is active");
    let (prior_rows, prior_selected, selected_id, prior_selected_sql) =
        match harness.app().workbench.tab(history_key) {
            Some(Tab::History(history)) => {
                let selected = history
                    .entries
                    .get(history.selected)
                    .expect("the native selection is within the current projection");
                (
                    history
                        .entries
                        .iter()
                        .map(|entry| (entry.id, entry.sql.clone()))
                        .collect::<Vec<_>>(),
                    history.selected,
                    selected.id,
                    selected.sql.clone(),
                )
            }
            _ => panic!("History remains active"),
        };

    harness
        .app_mut()
        .workbench
        .history
        .push(history_entry("SELECT newest\nFROM inserted_history"));
    harness.draw();

    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("draw-only does not reconcile History's stored projection")
    };
    assert_eq!(
        history
            .entries
            .iter()
            .map(|entry| (entry.id, entry.sql.clone()))
            .collect::<Vec<_>>(),
        prior_rows
    );
    assert_eq!(history.selected, prior_selected);
    assert_eq!(history.entries[history.selected].id, selected_id);
    assert_eq!(
        history.selected_query().as_deref(),
        Some(prior_selected_sql.as_str())
    );

    let _ = harness.tick();
    harness.draw();

    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("History remains active after its source changes")
    };
    assert_eq!(
        history.entries[0].sql,
        "SELECT newest\nFROM inserted_history"
    );
    assert_eq!(history.selected, 1);
    assert_eq!(history.entries[history.selected].id, selected_id);
    assert_eq!(history.selected_query().as_deref(), Some(selected_sql));
    assert!(harness.find("FROM selected_history").is_some());

    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("the native selected History row opens as a query")
    };
    assert_eq!(query.query, selected_sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_open_focus_is_serviced_when_switching_and_closing_an_inactive_tab() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT first\nFROM first_history"));
    app.workbench
        .history
        .push(history_entry("SELECT second\nFROM second_history"));
    let first = app.workbench.open_history().expect("first History tab");
    let second_index = app.workbench.tabs().len();
    let second_history = HistoryTab::new(&app.workbench.history);
    let second = app
        .workbench
        .insert_tab(second_index, Tab::History(second_history))
        .expect("second History tab");
    assert!(app.workbench.activate(first));

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let open_first = first.control("history.open");
    assert!(
        harness.tab_to(open_first),
        "the first Open button is focusable"
    );
    let query_count = harness.app().workbench.query_counter;

    assert!(harness.app_mut().workbench.activate(second));
    harness.draw();

    assert_eq!(harness.app().workbench.active_key(), Some(second));
    assert_ne!(harness.focus(), Some(open_first));
    assert_eq!(harness.app().workbench.query_counter, query_count);
    let Some(Tab::History(history)) = harness.app().workbench.tab(first) else {
        panic!("the inactive History tab retains its model")
    };
    assert_eq!(
        history.selected_query().as_deref(),
        Some("SELECT second\nFROM second_history")
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    assert!(harness.app_mut().workbench.close_tab_from_picker(first));
    harness.draw();
    assert!(harness.app().workbench.tab(first).is_none());
    assert_eq!(harness.app().workbench.active_key(), Some(second));
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(harness.area_of(open_first).is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_list_focus_is_serviced_when_its_tab_closes() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT retained\nFROM closed_list_owner"));
    let history_key = app.workbench.open_history().expect("History tab");
    let query_count = app.workbench.query_counter;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_list = Id::root("tablepro.workbench.history.list");
    assert_eq!(harness.focus(), Some(history_list));

    assert!(
        harness
            .app_mut()
            .workbench
            .close_tab_from_picker(history_key)
    );
    harness.draw();

    assert!(harness.app().workbench.tab(history_key).is_none());
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(harness.area_of(history_list).is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_open_focus_is_serviced_when_its_active_tab_closes() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT retained\nFROM closed_open_owner"));
    let history_key = app.workbench.open_history().expect("History tab");
    let query_count = app.workbench.query_counter;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let open = history_key.control("history.open");
    assert!(harness.tab_to(open), "the active Open button is focusable");
    assert_eq!(harness.focus(), Some(open));

    assert!(
        harness
            .app_mut()
            .workbench
            .close_tab_from_picker(history_key)
    );
    harness.draw();

    assert!(harness.app().workbench.tab(history_key).is_none());
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(harness.area_of(open).is_none());
    assert_ne!(harness.focus(), Some(open));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_open_focus_is_serviced_when_tabs_strip_closes_its_active_tab() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT retained\nFROM closed_open_owner"));
    let history_key = app.workbench.open_history().expect("History tab");
    let query_count = app.workbench.query_counter;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let open = history_key.control("history.open");
    assert!(harness.tab_to(open), "the active Open button is focusable");
    assert_eq!(harness.focus(), Some(open));

    let _ = harness.click_part(
        Id::root("tablepro.workbench.tab-strip"),
        PartRef::item(Part::CLOSE, ItemKey::num(history_key.get())),
    );
    let _ = harness.tick();

    assert!(harness.app().workbench.tab(history_key).is_none());
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(harness.area_of(open).is_none());
    assert_ne!(harness.focus(), Some(open));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.ctrl('y');
    let reopened_key = harness
        .app()
        .workbench
        .active_key()
        .expect("History reopens in a current tab");
    assert!(
        matches!(harness.app().workbench.active(), Some(Tab::History(_))),
        "reopening History must not construct or execute a query"
    );
    assert_eq!(harness.app().workbench.query_counter, query_count);
    let reopened_list = Id::root("tablepro.workbench.history.list");
    assert!(harness.area_of(reopened_list).is_some());
    assert!(harness.tab_to(reopened_list));
    assert_eq!(harness.focus(), Some(reopened_list));
    let Some(Tab::History(history)) = harness.app().workbench.active() else {
        panic!("History remains active while its native List is focused")
    };
    assert_eq!(
        history.selected_query().as_deref(),
        Some("SELECT retained\nFROM closed_open_owner")
    );
    let reopened_open = reopened_key.control("history.open");
    assert!(harness.area_of(reopened_open).is_some());
    assert!(harness.tab_to(reopened_open));
    assert_eq!(harness.focus(), Some(reopened_open));
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_rows_reset_for_a_replacement_workbench_with_a_colliding_tab_key() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry("SELECT old\nFROM old_workbench"));
    app.workbench
        .history
        .push(history_entry("SELECT old selected\nFROM old_selected"));
    let old_key = app.workbench.open_history().expect("old History tab");
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let _ = harness.key(KeyCode::Down);
    assert!(
        harness
            .app()
            .workbench
            .active()
            .is_some_and(|tab| { matches!(tab, Tab::History(history) if history.selected == 1) })
    );

    let mut replacement = connected();
    replacement.workbench.history = History::default();
    replacement
        .workbench
        .history
        .push(history_entry("SELECT replacement\nFROM new_workbench"));
    replacement.workbench.history.entries[0].id = 9000;
    let new_key = replacement
        .workbench
        .open_history()
        .expect("replacement History tab");
    let replacement_tab_count = replacement.workbench.tabs().len();
    let replacement_query_counter = replacement.workbench.query_counter;
    assert_eq!(old_key, new_key);
    harness.app_mut().workbench = replacement.workbench;
    harness.draw();

    assert_eq!(harness.app().workbench.active_key(), Some(new_key));
    assert_eq!(harness.app().workbench.tabs().len(), replacement_tab_count);
    assert_eq!(
        harness.app().workbench.query_counter,
        replacement_query_counter
    );
    let Some(Tab::History(history)) = harness.app().workbench.tab(new_key) else {
        panic!("replacement History tab remains active")
    };
    assert_eq!(history.selected, 0);
    assert_eq!(history.entries[0].id, 9000);
    assert_eq!(
        history.selected_query().as_deref(),
        Some("SELECT replacement\nFROM new_workbench")
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn tab_strip_overflow_and_tab_list() {
    let mut app = connected();
    for _ in 0..12 {
        app.workbench.new_query("SELECT 1");
    }
    assert!(app.workbench.tabs().len() >= 12);
    assert!(app.workbench.close_tab(0));
}
#[test]
fn pending_edits_preview_and_save() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let tab = app.workbench.active_table_mut().expect("table tab");
    let edit = tab.result.model.commit_cell(0, 6, "EUR");
    assert!(edit.is_ok(), "edit result: {edit:?}");
    assert!(tab.result.pending_total() > 0);
    assert!(!tab.preview().is_empty());
    tab.result.model.discard();
    assert_eq!(tab.result.pending_total(), 0);
}
#[test]
fn safe_mode_picker_changes_level_and_strip() {
    let mut app = connected();
    app.set_safe_mode(SafeMode::SafeFull);
    assert_eq!(app.safe_mode(), SafeMode::SafeFull);
    assert_eq!(app.surface(), Surface::SafeModePicker);
}
#[test]
fn mouse_opens_table_and_switches_tabs() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let first = app.workbench.active_key();
    app.workbench.new_query("SELECT * FROM customers LIMIT 1");
    assert_ne!(first, app.workbench.active_key());
}
#[test]
fn every_screen_renders_at_representative_sizes() {
    let app = TableProApp::default();
    let size = <TableProApp as App>::min_size(&app);
    assert_eq!(size.min, (72, 20));
    assert!(size.preferred.0 >= size.min.0 && size.preferred.1 >= size.min.1);
}

#[test]
fn visual_surface_fixture_materializes_the_real_route() {
    for surface in Surface::ALL {
        let mut app = TableProApp::default();
        app.set_surface(surface);
        assert_eq!(app.surface(), surface);
        let harness = Harness::new(app, Theme::junie(), 120, 40);

        let expected_marker = match surface {
            Surface::Connections | Surface::ConnectionsFailed => "Connections",
            _ => "Production",
        };
        assert!(
            harness.text().contains(expected_marker),
            "{} fixture did not reach its named renderer",
            surface.label()
        );
        match surface {
            Surface::Connections | Surface::ConnectionsFailed => {
                assert_eq!(harness.app().screen(), Screen::Connections);
            }
            _ => assert_eq!(harness.app().screen(), Screen::Workbench),
        }
    }
}

#[test]
fn narrow_terminals_turn_the_explorer_into_a_drawer() {
    let mut app = connected();
    app.set_surface(Surface::ExplorerFocused);
    assert_eq!(app.surface(), Surface::ExplorerFocused);
    assert!(app.workbench.visible_explorer().len() >= app.workbench.table_count());
}
#[test]
fn keyboard_flow_full_journey() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('t');
    let query_id = harness.app().query_id().expect("query id");
    assert!(tab_navigation::tab_to(&mut harness, query_id));
    let _ = harness.type_str("SELECT * FROM customers LIMIT 3");
    let _ = harness.ctrl('r');
    assert!(harness.app().query().contains("customers"));
    assert!(harness.app().status().contains("Loaded 3 rows"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn query_editor_keeps_multiline_unicode_and_cursor_through_resize() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('t');
    let query_id = harness.app().query_id().expect("new query editor");
    assert!(tab_navigation::tab_to(&mut harness, query_id));

    let _ = harness.type_str("SELECT '界🙂'");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.type_str("FROM customers");
    assert_eq!(harness.app().query(), "SELECT '界🙂'\nFROM customers");
    assert!(harness.text().contains("界🙂"));
    assert!(
        harness.is_open(query_id.sub("completion")),
        "the live completion popup remains open for this checkpoint"
    );
    assert_eq!(harness.focus(), Some(query_id));
    assert!(
        harness.cursor().is_some(),
        "the nonmodal completion popup preserves the rendered editor caret"
    );
    assert!(
        harness.cursor_bar(),
        "completion retains the editor's frozen steady-bar cursor shape"
    );
    let popup_cursor = harness.cursor().expect("visible completion caret");
    assert!(
        harness
            .area_of(query_id)
            .is_some_and(|area| area.contains(popup_cursor)),
        "the retained caret is the live editor's resolved cursor"
    );
    let popup_area = harness.layer_area(query_id.sub("completion")).unwrap();
    let first_item_y = popup_area.y.saturating_add(1);
    let first_item = (popup_area.x..popup_area.right())
        .map(|x| harness.cell(x, first_item_y))
        .collect::<Vec<_>>();
    assert!(
        first_item.iter().any(|cell| cell.symbol() == "T"),
        "the shared completion row renders its live table-kind glyph"
    );
    let second_item_y = first_item_y.saturating_add(1);
    assert_eq!(
        harness
            .cell(popup_area.x.saturating_add(2), second_item_y)
            .symbol(),
        "V",
        "the unselected completion row renders its view-kind glyph"
    );
    assert_eq!(
        harness
            .cell(popup_area.x.saturating_add(4), second_item_y)
            .symbol(),
        "a"
    );
    assert_eq!(
        harness
            .cell(popup_area.x.saturating_add(11), second_item_y)
            .symbol(),
        "c"
    );
    assert!(
        !format!(
            "{:?}",
            harness
                .cell(popup_area.x.saturating_add(4), second_item_y)
                .modifier
        )
        .contains("BOLD"),
        "the second, unselected completion remains plain before its fuzzy match"
    );
    assert!(
        format!(
            "{:?}",
            harness
                .cell(popup_area.x.saturating_add(11), second_item_y)
                .modifier
        )
        .contains("BOLD"),
        "the provider's matched grapheme is emphasized in the unselected popup row"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "popup cursor retention must not create duplicate controls: {:?}",
        harness.diagnostics()
    );
    let _ = harness.resize(96, 28);
    assert!(
        harness.is_open(query_id.sub("completion")),
        "resizing preserves the open completion popup"
    );
    assert_eq!(harness.focus(), Some(query_id));
    assert!(harness.cursor_bar(), "resize preserves the bar caret");
    assert!(harness.cursor().is_some());
    assert!(harness.text().contains("customers"));
    let resized_popup_cursor = harness.cursor();
    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.cursor().is_some(),
        "the rendered editor owns its cursor"
    );
    assert_eq!(harness.cursor(), resized_popup_cursor);

    assert_eq!(harness.app().query(), "SELECT '界🙂'\nFROM customers");
    assert!(
        harness.cursor().is_some(),
        "resize preserves the visible editor cursor"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn mouse_flow_full_journey() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let (x, y) = harness.find("Local PostgreSQL").expect("connection row");
    let _ = harness.click(x, y);
    assert_eq!(harness.app().screen(), Screen::Workbench);
    let (x, y) = harness.find("orders").expect("explorer row");
    let _ = harness.double_click(x, y);
    assert!(harness.app().workbench().active_table().is_some());
    assert!(harness.diagnostics().is_empty());
}
#[test]
fn connection_form_keyboard_and_mouse_reach_every_field() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.ctrl('n');
    assert!(harness.app().connection_form_open());
    let fields = form_fields();
    assert_eq!(fields.len(), 15);

    for (index, field) in fields.iter().enumerate() {
        let mut reached = false;
        for _ in 0..32 {
            if tab_navigation::tab_to(&mut harness, field.id) {
                reached = true;
                break;
            }
            let _ = harness.wheel(Axis::V, 1, 60, 26);
        }
        assert!(reached, "field {:?} was unreachable", field.id);

        if index == 12 {
            let _ = harness.key(KeyCode::Char(' '));
        }
    }
    assert!(harness.diagnostics().is_empty());
}
#[test]
fn connection_form_focuses_the_first_invalid_field() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.ctrl('n');
    let _ = harness.ctrl('s');
    assert_eq!(harness.focus(), Some(CONNECTION_NAME));
}
#[test]
fn connection_password_is_masked_and_absent_from_the_frame() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.ctrl('n');
    let fields = form_fields();
    assert!(tab_navigation::tab_to(&mut harness, fields[6].id));
    let _ = harness.type_str("hunter2");
    assert!(!harness.text().contains("hunter2"));
    assert!(harness.diagnostics().is_empty());
}
#[test]
fn resize_across_every_supported_size() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for (width, height) in [(72, 20), (80, 24), (120, 40), (160, 50)] {
        let _ = harness.resize(width, height);
        assert_eq!(harness.buffer().area().width, width);
        assert_eq!(harness.buffer().area().height, height);
    }
    assert!(harness.diagnostics().is_empty());
}
#[test]
fn focus_is_restored_after_every_overlay_closes() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let opener = harness.focus();
    let _ = harness.ctrl('n');
    assert!(harness.app().connection_form_open());
    let _ = harness.type_str("temporary name");
    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.app().connection_form_open(),
        "first Esc cancels the focused field, matching reference794b095"
    );
    let cancel = Id::root("tablepro.connections.form")
        .part(Part::ACTIONS)
        .index(1);

    for _ in 0..64 {
        if harness.focus() == Some(cancel) {
            break;
        }
        let _ = harness.key(KeyCode::Tab);
    }
    assert_eq!(harness.focus(), Some(cancel));
    let _ = harness.key(KeyCode::Enter);
    assert!(!harness.app().connection_form_open());
    assert!(harness.app().connection_draft().is_none());
    assert!(harness.find("Connections").is_some());
    assert_eq!(harness.focus(), opener);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn no_diagnostics_are_emitted_during_the_journey() {
    let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('t');
    let query_id = harness.app().query_id().expect("query id");
    assert!(tab_navigation::tab_to(&mut harness, query_id));
    let _ = harness.type_str("SELECT * FROM orders LIMIT 1");
    let _ = harness.ctrl('r');
    let _ = harness.resize(96, 28);
    assert!(harness.diagnostics().is_empty());
}
#[test]
fn acceptance_flow_keyboard_only() {
    keyboard_flow_full_journey();
}
#[test]
fn acceptance_flow_mouse() {
    mouse_flow_full_journey();
}
#[test]
fn pending_edits_keep_original_keys() {
    let pending = PendingEdits::new(vec![vec![Value::Int(7)]]);
    assert_eq!(pending.value(0, 0), Some(&Value::Int(7)));
}
#[test]
fn preview_uses_application_grid_adapter() {
    let mut app = connected();
    assert!(matches!(
        app.run_query("SELECT * FROM orders LIMIT 1"),
        QueryOutcome::Executed { .. }
    ));
    let catalog = Catalog::acme_prod();
    let table = catalog.find(Some("public"), "orders").expect("orders");
    let grid = ResultGrid::empty();
    assert!(preview_for(table, &grid).is_empty());
    assert!(app.result().row_count() > 0);
}
#[test]
fn history_search_is_multi_term_and() {
    let seeded = History::seeded();
    let mut history = HistoryTab::new(&seeded);
    history.search = "orders pending".to_owned();
    history.filter(&seeded);
    assert_eq!(
        history.selected_query().as_deref(),
        Some("SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 200")
    );
}

#[test]
fn history_search_combines_terms_connection_status_and_keeps_legacy_tab_filter() {
    let mut source = History::default();
    source.push(history_entry_for(
        "SELECT alpha FROM beta",
        "Production",
        true,
    ));
    source.push(history_entry_for(
        "SELECT beta FROM alpha",
        "Production",
        true,
    ));
    source.push(history_entry_for("SELECT alpha FROM beta", "Staging", true));
    source.push(history_entry_for(
        "SELECT alpha FROM beta",
        "Production",
        false,
    ));

    let filtered = source.search("ALPHA beta", Some("pROdUcTiOn"), true);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].sql, "SELECT beta FROM alpha");
    assert_eq!(filtered[1].sql, "SELECT alpha FROM beta");
    assert_eq!(source.search("", Some("production"), false).len(), 3);
    assert!(source.search("no matching SQL", None, false).is_empty());

    let mut tab = HistoryTab::new(&source);
    tab.search = "alpha beta".to_owned();
    tab.selected = usize::MAX;
    tab.filter(&source);
    assert_eq!(
        tab.entries.len(),
        4,
        "legacy filter stays all-scope/any-status"
    );
    assert_eq!(tab.selected, 3);
    tab.search = "no matching SQL".to_owned();
    tab.selected = usize::MAX;
    tab.filter(&source);
    assert!(tab.entries.is_empty());
    assert_eq!(tab.selected, 0);
}

#[test]
fn history_native_filters_apply_search_scope_status_and_survive_reopen() {
    let mut app = connected();
    app.workbench.history = History::default();
    let current = app.workbench.connection.name.clone();
    app.workbench.history.push(history_entry_for(
        "SELECT alpha FROM beta\n-- Production failure",
        &current,
        true,
    ));
    app.workbench.history.push(history_entry_for(
        "SELECT beta FROM alpha\n-- Production success",
        &current,
        false,
    ));
    app.workbench.history.push(history_entry_for(
        "SELECT alpha FROM beta\n-- Staging failure",
        "Staging",
        true,
    ));
    app.workbench.history.push(history_entry_for(
        "SELECT alpha FROM beta\n-- Staging success",
        "Staging",
        false,
    ));

    let (mut harness, copies) = clipboard_harness(app, 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("history tab is active");
    let search = history_key.control("history.search");
    let connection = history_key.control("history.connection");
    let status = history_key.control("history.status");
    let history_list = Id::root("tablepro.workbench.history.list");

    assert_eq!(
        harness
            .app()
            .workbench
            .tab(history_key)
            .and_then(|tab| match tab {
                Tab::History(history) => Some(history.entries.len()),
                _ => None,
            }),
        Some(4)
    );
    let initial_last = match harness.app().workbench.tab(history_key) {
        Some(Tab::History(history)) => history
            .entries
            .last()
            .expect("the source has visible History entries")
            .clone(),
        _ => panic!("history remains active"),
    };
    assert!(harness.tab_to(history_list));
    let _ = harness.key(KeyCode::End);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active after native End")
    };
    assert_eq!(history.selected, history.entries.len() - 1);
    assert_eq!(history.entries[history.selected].id, initial_last.id);
    assert_eq!(
        history.selected_query().as_deref(),
        Some(initial_last.sql.as_str())
    );
    assert!(harness.find("All connections").is_some());
    assert!(harness.find("Any status").is_some());
    assert!(harness.tab_to(search));
    let _ = harness.paste("ALPHA beta");
    assert!(
        matches!(harness.app().workbench.tab(history_key), Some(Tab::History(history)) if history.search.is_empty())
    );
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(history.search, "ALPHA beta");
    assert_eq!(history.entries.len(), 4);
    assert_eq!(history.selected, 3);

    assert!(harness.tab_to(connection));
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(history.entries.len(), 2);
    assert_eq!(history.selected, 1);
    assert!(
        history
            .entries
            .iter()
            .all(|entry| entry.connection == current)
    );

    let _ = harness.key(KeyCode::Char(' '));
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(
        history.entries.len(),
        4,
        "the scope button returns to all connections"
    );

    let _ = harness.click_id(connection);
    assert!(harness.tab_to(status));
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(history.entries.len(), 1);
    assert!(history.entries[0].error.is_some());
    assert!(harness.find("SELECT alpha FROM beta").is_some());

    harness.app_mut().workbench.connection.name = "Staging".to_owned();
    let _ = harness.tick();
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(history.entries.len(), 1);
    assert_eq!(history.entries[0].connection, "Staging");
    assert!(history.entries[0].error.is_some());

    let _ = harness.click_id(status);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains active")
    };
    assert_eq!(history.entries.len(), 2);
    assert!(
        history
            .entries
            .iter()
            .all(|entry| entry.connection == "Staging")
    );
    assert_eq!(
        history
            .entries
            .iter()
            .filter(|entry| entry.error.is_some())
            .count(),
        1,
        "the status control exposes one failed and one successful Staging row"
    );
    assert!(harness.find("SELECT alpha FROM beta").is_some());

    assert!(harness.tab_to(history_list));
    let (filtered_id, filtered_sql) = match harness.app().workbench.tab(history_key) {
        Some(Tab::History(history)) => {
            let selected = &history.entries[history.selected];
            let selected_query = history.selected_query();
            assert_eq!(selected_query.as_deref(), Some(selected.sql.as_str()));
            assert_eq!(selected.connection, "Staging");
            assert!(selected.sql.contains("-- Staging"));
            (
                selected.id,
                selected_query.expect("the selected row is in the committed filter projection"),
            )
        }
        _ => panic!("history remains active"),
    };
    assert_eq!(
        history_row_marker(&harness, filtered_id),
        Theme::junie().design.glyphs.get(GlyphRole::Chosen),
        "the committed unique filter selection is visibly chosen"
    );
    let query_count = harness.app().workbench.query_counter;
    let _ = harness.key(KeyCode::Char('y'));
    assert_eq!(copies.borrow().as_slice(), &[filtered_sql.clone()]);
    assert_eq!(harness.app().workbench.query_counter, query_count);
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Query(query)) = harness.app().workbench.active() else {
        panic!("native Enter opens the selected filtered History row")
    };
    assert_eq!(query.query, filtered_sql);
    assert!(query.result.is_none());
    assert!(query.plan.is_none());
    assert_eq!(harness.app().workbench.query_counter, query_count + 1);
    let _ = harness.ctrl('y');
    assert_eq!(harness.app().workbench.active_key(), Some(history_key));

    let tab_count = harness.app().workbench.tabs().len();
    assert!(harness.tab_to(history_list));
    let _ = harness.ctrl('t');
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::Query(_))
    ));
    harness.app_mut().workbench.history.push(history_entry_for(
        "SELECT alpha FROM beta -- new",
        "Staging",
        false,
    ));
    let _ = harness.ctrl('y');

    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(harness.app().workbench.tabs().len(), tab_count + 1);
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("the same history tab is reused")
    };
    assert_eq!(history.search, "ALPHA beta");
    assert_eq!(history.entries.len(), 3);
    assert!(
        history
            .entries
            .iter()
            .all(|entry| entry.connection == "Staging")
    );
    assert!(harness.find("Current connection").is_some());
    assert!(harness.find("Any status").is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_native_search_accepts_unicode_paste_and_escape_discards_draft() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry_for("SELECT '界🙂'", "Production", false));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let key = harness
        .app()
        .workbench
        .active_key()
        .expect("history tab is active");
    let search = key.control("history.search");
    assert!(harness.tab_to(search));
    let _ = harness.paste("界🙂");
    let _ = harness.key(KeyCode::Enter);

    let Some(Tab::History(history)) = harness.app().workbench.tab(key) else {
        panic!("history remains active")
    };
    assert_eq!(history.search, "界🙂");
    assert_eq!(history.entries.len(), 1);
    let committed_rows = history.entries.clone();

    let _ = harness.click_id(search);
    let _ = harness.paste(" discarded");
    let _ = harness.key(KeyCode::Esc);
    let Some(Tab::History(history)) = harness.app().workbench.tab(key) else {
        panic!("history remains active")
    };
    assert_eq!(history.search, "界🙂");
    assert_eq!(history.entries, committed_rows);

    let _ = harness.click_id(search);
    let _ = harness.ctrl('l');
    let _ = harness.paste("no matching row");
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(key) else {
        panic!("history remains active")
    };
    assert_eq!(history.search, "no matching row");
    assert!(history.entries.is_empty());
    let history_list = Id::root("tablepro.workbench.history.list");
    let open = key.control("history.open");
    let query_count = harness.app().workbench.query_counter;
    assert!(
        harness.tab_to(history_list),
        "the empty native List stays routable"
    );
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.click_id(open);
    assert_eq!(harness.app().workbench.active_key(), Some(key));
    assert_eq!(harness.app().workbench.query_counter, query_count);
    assert!(
        harness.app().workbench.tab(key).is_some_and(|tab| {
            matches!(tab, Tab::History(history) if history.entries.is_empty())
        })
    );

    let _ = harness.click_id(search);
    let _ = harness.ctrl('l');
    let _ = harness.key(KeyCode::Backspace);
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::History(history)) = harness.app().workbench.tab(key) else {
        panic!("history remains active")
    };
    assert!(history.search.is_empty());
    assert_eq!(history.entries.len(), 1);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_search_draft_survives_switch_reopen_and_owner_retirement() {
    let mut app = connected();
    app.workbench.history = History::default();
    app.workbench
        .history
        .push(history_entry_for("SELECT retained", "Production", false));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('y');
    let history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("history tab is active");
    let search = history_key.control("history.search");
    let history_list = Id::root("tablepro.workbench.history.list");
    assert!(harness.tab_to(search));
    let _ = harness.paste("hidden draft");

    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("history remains open")
    };
    assert!(history.search.is_empty(), "typing stays a draft");
    assert_eq!(history.entries.len(), 1);

    let _ = harness.ctrl('t');
    assert!(matches!(
        harness.app().workbench.active(),
        Some(Tab::Query(_))
    ));
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("the inactive history tab remains present")
    };
    assert!(
        history.search.is_empty(),
        "hidden lifecycle cannot commit a draft"
    );
    assert_eq!(history.entries.len(), 1);

    let _ = harness.ctrl('y');
    assert_eq!(harness.app().workbench.active_key(), Some(history_key));
    assert_eq!(
        harness.focus(),
        Some(history_list),
        "reentry focuses the list"
    );
    assert!(
        harness.find("hidden draft").is_some(),
        "the native draft is retained"
    );
    let Some(Tab::History(history)) = harness.app().workbench.tab(history_key) else {
        panic!("the same history tab is reused")
    };
    assert!(history.search.is_empty());
    assert_eq!(history.entries.len(), 1);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.ctrl('w');
    assert!(harness.app().workbench.tab(history_key).is_none());
    let _ = harness.tick();
    assert!(harness.area_of(search).is_none());
    assert!(
        harness
            .area_of(history_key.control("history.connection"))
            .is_none()
    );
    assert!(
        harness
            .area_of(history_key.control("history.status"))
            .is_none()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.ctrl('y');
    let focused_history_key = harness
        .app()
        .workbench
        .active_key()
        .expect("a new history tab is active");
    let focused_search = focused_history_key.control("history.search");
    assert!(harness.tab_to(focused_search));
    assert_eq!(harness.focus(), Some(focused_search));
    let _ = harness.click_part(
        Id::root("tablepro.workbench.tab-strip"),
        PartRef::item(Part::CLOSE, ItemKey::num(focused_history_key.get())),
    );
    assert!(harness.app().workbench.tab(focused_history_key).is_none());
    let _ = harness.tick();
    assert!(harness.area_of(focused_search).is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn history_native_post_paint_preserves_panel_bottom_border() {
    for (width, height) in [(72, 20), (120, 40)] {
        let mut app = connected();
        app.workbench.history = History::default();
        app.workbench
            .history
            .push(history_entry_for("SELECT retained", "Production", false));
        let mut harness = Harness::new(app, Theme::junie(), width, height);
        let _ = harness.ctrl('y');
        assert!(harness.find("All connections").is_some());
        assert!(harness.find("Any status").is_some());

        let panel = harness
            .area_of(Id::root("tablepro.workbench.content.frame"))
            .expect("content frame is registered");
        let edge = panel.right().saturating_sub(3);
        let bottom = panel.bottom().saturating_sub(1);
        assert_eq!(
            harness.cell(edge, bottom).symbol(),
            "─",
            "the bounded history detail pad must leave the framed bottom border intact at {width}x{height}"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn history_native_controls_are_nonoverlapping_and_reachable_at_minimum_size() {
    let mut harness = Harness::new(connected(), Theme::junie(), 72, 20);
    let _ = harness.ctrl('y');
    let key = harness
        .app()
        .workbench
        .active_key()
        .expect("history tab is active");
    let search = key.control("history.search");
    let connection = key.control("history.connection");
    let status = key.control("history.status");
    let history_list = Id::root("tablepro.workbench.history.list");
    let areas = [
        harness.area_of(search).expect("search is registered"),
        harness
            .area_of(connection)
            .expect("connection filter is registered"),
        harness
            .area_of(status)
            .expect("status filter is registered"),
        harness
            .area_of(history_list)
            .expect("history list is registered"),
    ];
    for area in areas {
        assert!(area.x.saturating_add(area.width) <= 72, "{area:?}");
        assert!(area.y.saturating_add(area.height) <= 20, "{area:?}");
    }
    for left in 0..areas.len() {
        for right in left + 1..areas.len() {
            let a = areas[left];
            let b = areas[right];
            let overlap = a.x < b.x.saturating_add(b.width)
                && b.x < a.x.saturating_add(a.width)
                && a.y < b.y.saturating_add(b.height)
                && b.y < a.y.saturating_add(a.height);
            assert!(!overlap, "controls overlap: {a:?} and {b:?}");
        }
    }
    assert!(harness.tab_to(search));
    assert!(harness.tab_to(connection));
    assert!(harness.tab_to(status));
    let _ = harness.click_id(status);
    let feedback = harness
        .activation_feedback()
        .expect("the real status click publishes activation feedback");
    assert_eq!(feedback.owner, status);
    let _ = harness.advance(feedback.remaining + core::time::Duration::from_millis(1));
    assert!(!harness.state_of(status).contains(StateFlags::PRESSED));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.resize(120, 40);
    assert!(harness.area_of(search).is_some());
    assert!(harness.area_of(connection).is_some());
    assert!(harness.area_of(status).is_some());
    assert!(harness.area_of(history_list).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[cfg(test)]
mod cursor_explain_tests {
    use super::*;
    use termrock_test_support::Harness;

    const FIRST: &str = "SELECT * FROM customers WHERE id = '界;🙂'; -- ; comment";
    const SECOND: &str = "SELECT * FROM orders;";

    #[derive(Debug, PartialEq, Eq)]
    struct ExplainEditorSnapshot {
        app_query: String,
        query_text: String,
        editor_text: String,
        cursor: usize,
        selection: Option<String>,
        editing: bool,
        saved_text: String,
        dirty: bool,
        focus: Option<Id>,
    }

    fn query_editor(width: u16, height: u16) -> Harness<TableProApp> {
        let mut harness = Harness::new(connected(), Theme::junie(), width, height);
        let query_id = harness.app().query_id().expect("connected query tab");
        assert!(tab_navigation::tab_to(&mut harness, query_id));
        assert_eq!(harness.focus(), Some(query_id));
        harness
    }

    fn explain_editor_snapshot(harness: &Harness<TableProApp>) -> ExplainEditorSnapshot {
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        let editor = query.editor_state.editor();
        ExplainEditorSnapshot {
            app_query: harness.app().query().to_owned(),
            query_text: query.query.clone(),
            editor_text: editor.text().to_owned(),
            cursor: editor.cursor_offset(),
            selection: editor.selected_text().map(str::to_owned),
            editing: editor.is_editing(),
            saved_text: query.saved_text.clone(),
            dirty: query.dirty(),
            focus: harness.focus(),
        }
    }

    fn dismiss_completion_if_open(harness: &mut Harness<TableProApp>) {
        let query_id = harness.app().query_id().expect("connected query editor");
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        if !query.editor_state.is_completion_open() {
            return;
        }
        assert!(harness.is_open(query_id.sub("completion")));
        assert!(!query.editor_state.is_completion_details_open());
        let before = explain_editor_snapshot(harness);

        let _ = harness.key(KeyCode::Esc);

        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        assert!(!query.editor_state.is_completion_open());
        assert!(!harness.is_open(query_id.sub("completion")));
        assert_eq!(explain_editor_snapshot(harness), before);
    }

    fn type_two_statements(harness: &mut Harness<TableProApp>) -> String {
        let _ = harness.type_str(FIRST);
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str(SECOND);
        let sql = format!("{FIRST}\n{SECOND}");
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        assert_eq!(query.editor_state.editor().text(), sql);
        assert!(query.editor_state.editor().is_editing());
        sql
    }

    fn active_plan_relation(harness: &Harness<TableProApp>) -> Option<String> {
        fn plan_relation(plan: &tablepro_ui::sql::PlanNode) -> Option<String> {
            plan.relation
                .clone()
                .or_else(|| plan.children.iter().find_map(|child| plan_relation(child)))
        }

        match harness.app().workbench.active() {
            Some(Tab::Query(query)) => query.plan.as_ref().and_then(plan_relation),
            _ => None,
        }
    }

    fn assert_editor_snapshot(
        harness: &Harness<TableProApp>,
        expected_sql: &str,
        saved_text: &str,
        cursor: usize,
        selection: Option<&str>,
    ) {
        assert_eq!(harness.app().query(), expected_sql);
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        let editor = query.editor_state.editor();
        assert_eq!(editor.text(), expected_sql);
        assert_eq!(editor.cursor_offset(), cursor);
        assert_eq!(editor.selected_text(), selection);
        assert!(editor.is_editing());
        assert_eq!(harness.focus(), harness.app().query_id());
        assert_eq!(query.saved_text, saved_text);
        assert!(query.dirty());
    }

    fn assert_nonexecution_snapshot(
        harness: &Harness<TableProApp>,
        history: &[HistoryEntry],
        result: &Option<ResultGrid>,
        last_status: &Option<String>,
    ) {
        assert_eq!(harness.app().workbench.history.entries.as_slice(), history);
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        assert_eq!(
            query.result.as_ref().map(|grid| &grid.model),
            result.as_ref()
        );
        assert_eq!(
            query
                .last_status
                .as_ref()
                .map(|status| status.summary.as_str()),
            last_status.as_deref()
        );
    }

    #[test]
    fn captured_plan_facts_follow_native_tab_identity() {
        let mut harness = query_editor(120, 40);
        let first_key = harness
            .app()
            .workbench
            .active_key()
            .expect("connected query key");
        let _ = harness.type_str("SELECT * FROM customers WHERE id = 1");
        dismiss_completion_if_open(&mut harness);
        let _ = harness.ctrl('x');
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.customers")
        );
        assert!(
            harness.find("Planning 0.300 ms").is_some(),
            "{}",
            harness.text()
        );

        let _ = harness.ctrl('t');
        let second_key = harness.app().workbench.active_key().expect("new query key");
        assert_ne!(first_key, second_key);
        let _ = harness.type_str("SELECT * FROM orders");
        dismiss_completion_if_open(&mut harness);
        let _ = harness.ctrl('x');
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.orders")
        );
        assert!(
            harness.find("Planning 0.210 ms").is_some(),
            "{}",
            harness.text()
        );

        let tab_strip = Id::root("tablepro.workbench.tab-strip");
        let _ = harness.click_part(
            tab_strip,
            PartRef::item(Part::TAB, ItemKey::num(first_key.get())),
        );
        assert_eq!(harness.app().workbench.active_key(), Some(first_key));
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.customers")
        );
        assert!(
            harness.find("Planning 0.300 ms").is_some(),
            "{}",
            harness.text()
        );

        let _ = harness.click_part(
            tab_strip,
            PartRef::item(Part::TAB, ItemKey::num(second_key.get())),
        );
        assert_eq!(harness.app().workbench.active_key(), Some(second_key));
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.orders")
        );
        assert!(
            harness.find("Planning 0.210 ms").is_some(),
            "{}",
            harness.text()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn explain_uses_cursor_statement_and_analyze_aliases_keep_editor_live() {
        for (width, height) in [(120, 40), (80, 24)] {
            let mut harness = query_editor(width, height);
            let sql = type_two_statements(&mut harness);
            let query_id = harness.app().query_id().expect("query editor");
            let active_key = harness.app().workbench.active_key();
            let connection = harness.app().workbench.connection.clone();
            let catalog_snapshot = format!("{:?}", harness.app().workbench.catalog);
            let (saved_text, cursor, selection, history, result, last_status) = {
                let Some(Tab::Query(query)) = harness.app().workbench.active() else {
                    panic!("connected query remains active")
                };
                (
                    query.saved_text.clone(),
                    query.editor_state.editor().cursor_offset(),
                    query
                        .editor_state
                        .editor()
                        .selected_text()
                        .map(str::to_owned),
                    harness.app().workbench.history.entries.clone(),
                    query.result.as_ref().map(|grid| grid.model.clone()),
                    query
                        .last_status
                        .as_ref()
                        .map(|status| status.summary.clone()),
                )
            };
            assert_eq!(harness.focus(), Some(query_id));
            assert_eq!(
                harness.app().query(),
                sql,
                "the full editor buffer remains the query snapshot"
            );

            dismiss_completion_if_open(&mut harness);
            let _ = harness.ctrl('x');
            assert_eq!(
                active_plan_relation(&harness).as_deref(),
                Some("public.orders")
            );
            assert!(
                harness.find("Planning 0.210 ms").is_some(),
                "{}",
                harness.text()
            );
            assert!(harness.find("Planning").is_some(), "{}", harness.text());
            assert!(harness.find("Seq Scan").is_some(), "{}", harness.text());
            assert_eq!(harness.app().status(), "Explain plan ready");
            assert_editor_snapshot(&harness, &sql, &saved_text, cursor, selection.as_deref());
            let Some(Tab::Query(query)) = harness.app().workbench.active() else {
                panic!("connected query remains active")
            };
            assert!(
                query
                    .plan
                    .as_ref()
                    .is_some_and(|plan| plan.actual_ms.is_none())
            );
            assert_nonexecution_snapshot(&harness, &history, &result, &last_status);

            dismiss_completion_if_open(&mut harness);
            let _ = harness.alt('x');
            assert_eq!(
                active_plan_relation(&harness).as_deref(),
                Some("public.orders")
            );
            assert!(
                harness.find("Planning 0.210 ms").is_some(),
                "{}",
                harness.text()
            );
            assert!(harness.find("Planning").is_some(), "{}", harness.text());
            assert!(matches!(
                harness.app().workbench.active(),
                Some(Tab::Query(query))
                    if query.plan.as_ref().is_some_and(|plan| plan.actual_ms.is_some())
            ));
            assert_editor_snapshot(&harness, &sql, &saved_text, cursor, selection.as_deref());
            assert_nonexecution_snapshot(&harness, &history, &result, &last_status);

            dismiss_completion_if_open(&mut harness);
            let _ = harness.ctrl('x');
            assert!(matches!(
                harness.app().workbench.active(),
                Some(Tab::Query(query))
                    if query.plan.as_ref().is_some_and(|plan| plan.actual_ms.is_none())
            ));
            dismiss_completion_if_open(&mut harness);
            let _ = harness.key_mod(KeyCode::F(7), KeyModifiers::SHIFT);
            assert_eq!(
                active_plan_relation(&harness).as_deref(),
                Some("public.orders")
            );
            assert!(
                harness.find("Planning 0.210 ms").is_some(),
                "{}",
                harness.text()
            );
            assert!(matches!(
                harness.app().workbench.active(),
                Some(Tab::Query(query))
                    if query.plan.as_ref().is_some_and(|plan| plan.actual_ms.is_some())
            ));
            assert_editor_snapshot(&harness, &sql, &saved_text, cursor, selection.as_deref());
            assert_nonexecution_snapshot(&harness, &history, &result, &last_status);

            let _ = harness.key(KeyCode::Home);
            let _ = harness.key(KeyCode::Up);
            let _ = harness.key(KeyCode::Home);
            assert_eq!(
                active_plan_relation(&harness).as_deref(),
                Some("public.orders")
            );
            assert!(
                harness.find("Planning 0.210 ms").is_some(),
                "cursor movement retargeted stored plan facts: {}",
                harness.text()
            );
            let first_cursor = match harness.app().workbench.active() {
                Some(Tab::Query(query)) => query.editor_state.editor().cursor_offset(),
                _ => unreachable!("connected query remains active"),
            };
            dismiss_completion_if_open(&mut harness);
            let _ = harness.ctrl('x');
            assert_eq!(
                active_plan_relation(&harness).as_deref(),
                Some("public.customers")
            );
            assert!(
                harness.find("Planning 0.300 ms").is_some(),
                "{}",
                harness.text()
            );
            assert!(harness.find("Planning").is_some(), "{}", harness.text());
            assert!(harness.find("Index Scan").is_some(), "{}", harness.text());
            assert_editor_snapshot(&harness, &sql, &saved_text, first_cursor, None);
            let Some(Tab::Query(query)) = harness.app().workbench.active() else {
                panic!("connected query remains active")
            };
            assert!(
                query
                    .plan
                    .as_ref()
                    .is_some_and(|plan| plan.actual_ms.is_none())
            );
            assert_nonexecution_snapshot(&harness, &history, &result, &last_status);
            assert_eq!(harness.app().workbench.active_key(), active_key);
            assert_eq!(harness.app().workbench.connection, connection);
            assert_eq!(
                format!("{:?}", harness.app().workbench.catalog),
                catalog_snapshot
            );
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }

    #[test]
    fn selected_statement_overrides_cursor_and_selected_batch_keeps_previous_plan() {
        let mut harness = query_editor(120, 40);
        let sql = type_two_statements(&mut harness);
        let active_key = harness.app().workbench.active_key();
        let connection = harness.app().workbench.connection.clone();
        let catalog_snapshot = format!("{:?}", harness.app().workbench.catalog);
        let history = harness.app().workbench.history.entries.clone();
        dismiss_completion_if_open(&mut harness);
        let _ = harness.ctrl('x');
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.orders")
        );
        assert!(
            harness.find("Planning 0.210 ms").is_some(),
            "{}",
            harness.text()
        );

        let _ = harness.key(KeyCode::Home);
        let _ = harness.key(KeyCode::Up);
        let _ = harness.key(KeyCode::Home);
        let _ = harness.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
        let (cursor, selection) = match harness.app().workbench.active() {
            Some(Tab::Query(query)) => (
                query.editor_state.editor().cursor_offset(),
                query
                    .editor_state
                    .editor()
                    .selected_text()
                    .map(str::to_owned),
            ),
            _ => unreachable!("connected query remains active"),
        };
        let expected_first_selection = format!("{FIRST}\n");
        assert_eq!(
            selection.as_deref(),
            Some(expected_first_selection.as_str())
        );
        assert_eq!(cursor, FIRST.len() + 1);
        dismiss_completion_if_open(&mut harness);
        let _ = harness.ctrl('x');
        assert_eq!(
            active_plan_relation(&harness).as_deref(),
            Some("public.customers"),
            "the selected statement wins even though the cursor is at the second statement"
        );
        assert!(
            harness.find("Planning 0.300 ms").is_some(),
            "{}",
            harness.text()
        );
        assert_editor_snapshot(&harness, &sql, "", cursor, selection.as_deref());

        let _ = harness.key(KeyCode::Home);
        let _ = harness.key(KeyCode::Up);
        let _ = harness.key(KeyCode::Home);
        let _ = harness.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
        for _ in SECOND.chars() {
            let _ = harness.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
        }
        let (batch_cursor, batch) = match harness.app().workbench.active() {
            Some(Tab::Query(query)) => (
                query.editor_state.editor().cursor_offset(),
                query
                    .editor_state
                    .editor()
                    .selected_text()
                    .map(str::to_owned),
            ),
            _ => unreachable!("connected query remains active"),
        };
        assert_eq!(batch.as_deref(), Some(sql.as_str()));
        assert_eq!(batch_cursor, sql.len());
        let previous_plan = match harness.app().workbench.active() {
            Some(Tab::Query(query)) => query.plan.clone(),
            _ => unreachable!("connected query remains active"),
        };
        dismiss_completion_if_open(&mut harness);
        let _ = harness.ctrl('x');
        let Some(Tab::Query(query)) = harness.app().workbench.active() else {
            panic!("connected query remains active")
        };
        assert_eq!(query.plan, previous_plan);
        assert!(
            harness.find("Planning 0.300 ms").is_some(),
            "failed batch selection replaced the captured plan facts: {}",
            harness.text()
        );
        assert!(query.result.is_none());
        assert!(query.last_status.is_none());
        assert_eq!(harness.app().workbench.history.entries, history);
        assert_editor_snapshot(&harness, &sql, "", batch_cursor, batch.as_deref());
        assert_eq!(harness.app().status(), "Explain plan ready");
        assert_eq!(harness.app().workbench.active_key(), active_key);
        assert_eq!(harness.app().workbench.connection, connection);
        assert_eq!(
            format!("{:?}", harness.app().workbench.catalog),
            catalog_snapshot
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
