//! Real keyboard switcher and current-schema selection contracts.
use junie_tui::{App, Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, Tab, TableProApp};
const PICKER: Id = Id::root("tablepro.quick-switcher");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");
fn connected() -> Harness<TableProApp> {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    Harness::new(app, Theme::junie(), 120, 40)
}
#[test]
fn source_quick_switcher_opens_table_and_escape_clears_then_closes() {
    let mut h = connected();
    let _ = h.ctrl('o');
    assert!(h.layer_area(PICKER).is_some());
    assert!(h.text().contains("Open Quickly"));
    let _ = h.type_str("cust");
    assert!(h.text().contains("customers"));
    let _ = h.key(KeyCode::Enter);
    assert!(h.layer_area(PICKER).is_none());
    assert!(
        matches!(h.app().workbench.active(), Some(Tab::Table(tab)) if tab.table.name == "customers")
    );
    assert_eq!(h.focus(), h.app().result_id());
    let before = h.focus();
    let _ = h.ctrl('o');
    let _ = h.type_str("x");
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(PICKER).is_some());
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(PICKER).is_none());
    assert_eq!(h.focus(), before);
    assert_ne!(h.app().surface(), Surface::QuickSwitcher);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn schema_target_changes_context_and_preserves_tab_identity_and_draft() {
    let mut h = connected();
    let before = h.app().workbench.active_key();
    let Some(editor) = h.app().query_id() else {
        unreachable!("query fixture")
    };
    assert!(h.tab_to(editor));
    let _ = h.type_str("SELECT secret_schema_draft");
    let query = h.app().query().to_owned();
    let count = h.app().workbench.tabs().len();
    h.app_mut().workbench.filter_explorer("orders");
    let _ = h.ctrl('o');
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Tab);
    let _ = h.type_str("analytics");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.current_schema(), "analytics");
    assert_eq!(h.app().workbench.explorer_filter, "");
    assert_eq!(h.app().workbench.active_key(), before);
    assert_eq!(h.app().workbench.tabs().len(), count);
    assert_eq!(h.app().query(), query);
    assert!(matches!(h.app().workbench.active(), Some(Tab::Query(tab))
        if tab.editor_state.draft_text() == Some("SELECT secret_schema_draft")));
    assert_eq!(h.focus(), Some(EXPLORER));
    assert!(h.text().contains("events"), "{}", h.text());
    assert!(!h.app_mut().select_schema("missing"));
    assert_eq!(h.app().workbench.current_schema(), "analytics");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn detail_queries_and_modal_typing_use_shared_input_without_quit_leak() {
    let mut h = connected();
    let _ = h.ctrl('o');
    let _ = h.type_str("public");
    assert!(h.text().contains("customers"));
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("q");
    let _ = h.ctrl('q');
    let _ = h.ctrl('c');
    assert!(!h.app().should_quit());
    assert!(h.layer_area(PICKER).is_some());
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(PICKER).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn reopening_existing_relation_preserves_the_owned_tab() {
    let mut h = connected();
    let _ = h.ctrl('o');
    let _ = h.type_str("cust");
    let _ = h.key(KeyCode::Enter);
    let key = h.app().workbench.active_key();
    let Some(grid) = h.app().result_id() else {
        unreachable!("table fixture")
    };
    assert!(h.tab_to(grid));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    let _ = h.type_str("retained");
    let draft = h
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.edit_draft())
        .map(str::to_owned);
    assert!(draft.is_some());
    let _ = h.ctrl('o');
    let _ = h.type_str("cust");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), key);
    assert_eq!(
        h.app()
            .workbench
            .active_grid()
            .and_then(|(_, grid)| grid.state.edit_draft()),
        draft.as_deref()
    );
    assert_eq!(h.app().workbench.tabs().len(), 2);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn open_tab_target_survives_reorder_and_refuses_removed_target() {
    let mut h = connected();
    let first = h.app().workbench.active_key();
    assert!(h.app_mut().workbench.new_query("").is_some());
    let _ = h.ctrl('o');
    let _ = h.type_str("Query 1");
    let order: Vec<_> = h
        .app()
        .workbench
        .tabs()
        .iter()
        .rev()
        .map(tablepro_app::TabRecord::key)
        .collect();
    assert!(h.app_mut().workbench.reorder_tabs(&order));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), first);
    assert!(
        h.diagnostics().is_empty(),
        "after reorder: {:?}",
        h.diagnostics()
    );
    let _ = h.ctrl('o');
    let _ = h.type_str("Query 1");
    let index = h.app().workbench.active_index();
    assert!(index.is_some_and(|index| h.app_mut().workbench.close_tab(index)));
    assert!(h.app_mut().workbench.new_query("").is_some());
    let replacement = h.app().workbench.active_key();
    h.draw();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), replacement);
    assert!(h.app().status().contains("Target unavailable"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn whole_workbench_replacement_refuses_captured_switcher_targets() {
    let mut h = connected();
    let _ = h.ctrl('o');
    let _ = h.type_str("cust");
    h.app_mut().workbench = TableProApp::new().workbench;
    let count = h.app().workbench.tabs().len();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.tabs().len(), count);
    assert!(h.app().status().contains("Workbench changed"));
    assert!(h.layer_area(PICKER).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn switcher_query_is_absent_from_app_debug() {
    let mut h = connected();
    let _ = h.ctrl('o');
    let _ = h.type_str("picker_secret_19");
    assert!(!format!("{:?}", h.app()).contains("picker_secret_19"));
    assert!(h.text().contains("picker_secret_19"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
