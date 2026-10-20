use tablepro_ui::{Surface, TableProApp};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const TAB_LIST: Id = Id::root("tablepro.tab-list");

fn connected() -> Harness<TableProApp> {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    Harness::new(app, Theme::junie(), 120, 40)
}

#[test]
fn choosing_a_query_tab_moves_focus_to_its_editor() {
    let mut harness = connected();
    let Some(first) = harness
        .app()
        .workbench
        .tabs()
        .first()
        .map(tablepro_ui::TabRecord::key)
    else {
        unreachable!("connected fixture has an initial tab")
    };
    harness.app_mut().workbench.new_query("SELECT 1");
    harness.draw();
    assert_ne!(harness.app().workbench.active_key(), Some(first));

    let _ = harness.ctrl('g');
    assert!(harness.layer_area(TAB_LIST).is_some());
    let _ = harness.key(KeyCode::Up);
    let _ = harness.key(KeyCode::Enter);

    assert_eq!(harness.app().workbench.active_key(), Some(first));
    assert_eq!(harness.focus(), Some(first.control("query")));
    assert_ne!(harness.app().surface(), Surface::TabListPicker);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn delete_closes_clean_tab_and_reopens_picker_with_empty_filter() {
    let mut harness = connected();
    let Some(removed) = harness.app_mut().workbench.new_query("SELECT 1") else {
        unreachable!("fixture has available tab identities")
    };
    harness.draw();

    let _ = harness.ctrl('g');
    let _ = harness.type_str("Query 2");
    assert!(harness.text().contains("Query 2"), "{}", harness.text());
    let _ = harness.key(KeyCode::Delete);

    assert!(
        !harness
            .app()
            .workbench
            .tabs()
            .iter()
            .any(|tab| tab.key() == removed)
    );
    assert!(harness.layer_area(TAB_LIST).is_some());
    assert!(harness.text().contains("Query 1"), "{}", harness.text());
    assert!(!harness.text().contains("Close tab with unsaved work?"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn query_tab_detail_uses_its_last_execution_status() {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT * FROM customers LIMIT 1"),
        tablepro_ui::QueryOutcome::Executed { .. }
    ));
    let Some(query_key) = app.workbench.active_key() else {
        unreachable!("query fixture has active tab")
    };
    let Some(query) = app.workbench.tab(query_key) else {
        unreachable!("active query exists")
    };
    let tablepro_ui::Tab::Query(query) = query else {
        unreachable!("query fixture")
    };
    let Some(status) = query.last_status.as_ref() else {
        unreachable!("executed query has a status")
    };
    assert!(!status.is_error);
    assert!(status.summary.starts_with("ok · "));
    assert!(status.duration_ms < 60_000);
    assert!(matches!(
        app.run_query("SELECT * FROM missing_table"),
        tablepro_ui::QueryOutcome::Rejected { .. }
    ));
    let Some(query) = app.workbench.tab(query_key) else {
        unreachable!("active query exists")
    };
    let tablepro_ui::Tab::Query(query) = query else {
        unreachable!("query fixture")
    };
    let Some(status) = query.last_status.as_ref() else {
        unreachable!("failed query has a status")
    };
    assert!(status.is_error);
    assert_eq!(status.summary, "error");

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('g');
    assert!(harness.text().contains("error"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
