#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::model::{HistoryEntry, HistorySource};
use tablepro_ui::{
    CONNECTION_NAME, Catalog, Decision, Filter, FilterOp, History, HistoryTab, PendingEdits,
    QueryOutcome, ResultGrid, SafeMode, Screen, Surface, Tab, TableProApp, Value, complete,
    form_fields, gate, parse, preview_for,
};
use termrock::{
    Axis, GridEditor, Id, ItemKey, KeyCode, MouseKind, Part, PartRef, StateFlags, Theme,
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
    let size = <TableProApp as termrock::App>::min_size(&app);
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
    app.workbench
        .history
        .push(history_entry_for("SELECT alpha FROM beta", &current, true));
    app.workbench
        .history
        .push(history_entry_for("SELECT beta FROM alpha", &current, false));
    app.workbench
        .history
        .push(history_entry_for("SELECT alpha FROM beta", "Staging", true));
    app.workbench.history.push(history_entry_for(
        "SELECT alpha FROM beta",
        "Staging",
        false,
    ));

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
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
    if let Some(Tab::History(history)) = harness.app_mut().workbench.tab_mut(history_key) {
        history.selected = usize::MAX;
    }
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
    assert!(harness.find("SELECT alpha FROM beta").is_some());

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
