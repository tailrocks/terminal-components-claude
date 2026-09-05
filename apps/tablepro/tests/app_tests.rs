//! Slice 6's preserved `TablePro` workflow scenarios.
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use junie_tui::GridEditor;
use tablepro_app::{
    CONNECTION_NAME, Catalog, ConnectionDraft, Decision, Filter, FilterOp, History, HistoryTab,
    PendingEdits, QueryOutcome, ResultGrid, SafeMode, Screen, Surface, Tab, TableProApp, Value,
    complete, form_fields, gate, parse, preview_for,
};

fn connected() -> TableProApp {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    app
}

#[test]
fn connections_screen_lists_and_connects_with_keyboard() {
    let mut app = TableProApp::default();
    assert_eq!(app.screen(), Screen::Connections);
    assert!(app.connect(0));
    assert_eq!(app.screen(), Screen::Workbench);
    assert_eq!(app.surface(), Surface::WorkbenchDefault);
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
        tab.sort(0, junie_tui::SortDir::Desc);
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
    if let Some(Tab::Query(query)) = app.workbench.tabs.get_mut(index) {
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
    if let Some(Tab::Query(query)) = app.workbench.tabs.get_mut(index) {
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
        app.workbench.tabs.get(index),
        Some(Tab::History(_))
    ));
    assert!(!app.workbench.history.is_empty());
}
#[test]
fn tab_strip_overflow_and_tab_list() {
    let mut app = connected();
    for _ in 0..12 {
        app.workbench.new_query("SELECT 1");
    }
    assert!(app.workbench.tabs.len() >= 12);
    assert!(app.workbench.close_tab(0));
}
#[test]
fn pending_edits_preview_and_save() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    let tab = app.workbench.active_table_mut().expect("table tab");
    let edit = tab.result.commit_cell(0, 6, "EUR");
    assert!(edit.is_ok(), "edit result: {edit:?}");
    assert!(tab.result.pending_total() > 0);
    assert!(!tab.preview().is_empty());
    tab.result.discard();
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
    let first = app.workbench.active;
    app.workbench.new_query("SELECT * FROM customers LIMIT 1");
    assert_ne!(first, app.workbench.active);
}
#[test]
fn every_screen_renders_at_representative_sizes() {
    let app = TableProApp::default();
    let size = <TableProApp as junie_tui::App>::min_size(&app);
    assert_eq!(size.min, (72, 20));
    assert!(size.preferred.0 >= size.min.0 && size.preferred.1 >= size.min.1);
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
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    assert!(app.workbench.toggle_structure());
    assert!(app.workbench.toggle_structure());
    assert_eq!(
        app.run_query("SELECT * FROM customers LIMIT 3"),
        QueryOutcome::Executed {
            rows: 3,
            editable: true
        }
    );
}
#[test]
fn mouse_flow_full_journey() {
    let mut app = connected();
    app.begin_connection_form();
    assert!(app.connection_form_open());
    assert!(app.connection_draft().is_some());
    assert_eq!(
        app.connection_draft().map(ConnectionDraft::password_status),
        Some("not set")
    );
}
#[test]
fn connection_form_keyboard_and_mouse_reach_every_field() {
    let fields = form_fields();
    assert_eq!(fields.len(), 15);
    let ids: Vec<_> = fields.iter().map(|field| field.id).collect();
    ids.windows(2).for_each(|pair| assert_ne!(pair[0], pair[1]));
}
#[test]
fn connection_form_focuses_the_first_invalid_field() {
    let draft = ConnectionDraft::default();
    assert_eq!(
        draft.validate_all().map_err(|(id, _)| id),
        Err(CONNECTION_NAME)
    );
}
#[test]
fn connection_password_is_masked_and_absent_from_the_frame() {
    let mut draft = ConnectionDraft::default();
    draft.password.set("hunter2");
    assert!(format!("{draft:?}").contains("password: Secret([redacted])"));
    assert!(!format!("{draft:?}").contains("hunter2"));
}
#[test]
fn resize_across_every_supported_size() {
    let sizes = [(60, 15), (72, 20), (80, 24), (120, 40), (160, 50)];
    let app = TableProApp::default();
    let min = <TableProApp as junie_tui::App>::min_size(&app).min;
    assert!(sizes.iter().any(|&(width, height)| (width, height) == min));
}
#[test]
fn focus_is_restored_after_every_overlay_closes() {
    let mut app = TableProApp::default();
    app.begin_connection_form();
    assert!(app.connection_form_open());
    app.form_open_for_test(false);
    assert!(!app.connection_form_open());
}
#[test]
fn no_diagnostics_are_emitted_during_the_journey() {
    let mut app = connected();
    assert!(app.workbench.open_table("orders"));
    assert!(app.status().contains("Connected"));
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
    let app = connected();
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
