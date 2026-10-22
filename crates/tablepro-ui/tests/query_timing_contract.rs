#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use std::time::Duration;

use tablepro_ui::{QueryOutcome, SafeMode, Tab, TableProApp};
use termrock::{GridEditor, KeyCode, Theme};
use termrock_test_support::Harness;

const FIXED_DURATION: Duration = Duration::from_millis(42);

fn query_status(app: &TableProApp) -> (String, bool, u32) {
    let Some(Tab::Query(tab)) = app.workbench.active() else {
        unreachable!("active query exists")
    };
    let Some(status) = tab.last_status.as_deref() else {
        unreachable!("executed query has status")
    };
    (status.summary.clone(), status.is_error, status.duration_ms)
}

#[test]
fn explicit_constructor_times_its_initial_query_before_first_render() {
    let app = TableProApp::new_with_query_duration(FIXED_DURATION);

    assert_eq!(query_status(&app), ("ok · 42 ms".to_owned(), false, 42));
    assert_eq!(app.result().row_count(), 20);
}

fn typed_query_frame() -> (termrock::Buffer, termrock::Buffer) {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    assert!(app.connect(0));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let Some(query_id) = harness.app().query_id() else {
        unreachable!("connected workbench has a query editor")
    };
    assert!(tab_navigation::tab_to(&mut harness, query_id));
    let _ = harness.type_str("SELECT * FROM customers LIMIT 3");
    let _ = harness.ctrl('r');

    assert_eq!(
        query_status(harness.app()),
        ("ok · 42 ms".to_owned(), false, 42)
    );
    assert_eq!(harness.app().result().row_count(), 3);
    let _ = harness.ctrl('g');
    assert!(harness.text().contains("ok · 42 ms"), "{}", harness.text());

    harness.draw();
    let wide = harness.buffer().clone();
    harness.draw();
    assert_eq!(harness.buffer(), &wide);

    let _ = harness.resize(100, 30);
    harness.draw();
    let narrow = harness.buffer().clone();
    harness.draw();
    assert_eq!(harness.buffer(), &narrow);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    (wide, narrow)
}

#[test]
fn typed_run_tab_status_redraw_and_resize_are_full_cell_repeatable() {
    assert_eq!(typed_query_frame(), typed_query_frame());
}

#[test]
fn errors_pending_denial_and_confirmation_keep_timing_contracts() {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT * FROM missing_table"),
        QueryOutcome::Rejected { .. }
    ));
    let error_status = query_status(&app);
    assert_eq!(error_status, ("error".to_owned(), true, 42));

    app.set_safe_mode(SafeMode::ReadOnly);
    assert!(matches!(
        app.run_query("UPDATE orders SET status = 'paid' WHERE id = 'x'"),
        QueryOutcome::Denied { .. }
    ));
    assert_eq!(query_status(&app), error_status);

    app.set_safe_mode(SafeMode::Safe);
    assert!(matches!(
        app.run_query("UPDATE orders SET status = 'paid' WHERE id = 'x'"),
        QueryOutcome::ConfirmationRequired { .. }
    ));
    assert_eq!(query_status(&app), error_status);

    app.set_safe_mode(SafeMode::Silent);
    assert!(matches!(
        app.run_query("SELECT id, currency FROM orders LIMIT 1"),
        QueryOutcome::Executed { rows: 1, .. }
    ));
    let previous_status = query_status(&app);
    let Some(Tab::Query(tab)) = app.workbench.active_mut() else {
        unreachable!("active query exists")
    };
    let Some(grid) = tab.result.as_mut() else {
        unreachable!("successful query has a result")
    };
    assert!(grid.model.commit_cell(0, 1, "EUR").is_ok());
    assert!(matches!(
        app.run_query("SELECT * FROM customers LIMIT 2"),
        QueryOutcome::Rejected { .. }
    ));
    assert_eq!(query_status(&app), previous_status);
    assert_eq!(app.result().pending_total(), 1);
}

#[test]
fn saved_owner_confirmation_uses_fixed_timing_for_captured_query() {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT id, currency FROM orders LIMIT 1"),
        QueryOutcome::Executed { rows: 1, .. }
    ));
    let Some(owner) = app.workbench.active_key() else {
        unreachable!("active query has a key")
    };
    let Some(Tab::Query(tab)) = app.workbench.tab_mut(owner) else {
        unreachable!("owner is a query")
    };
    let Some(grid) = tab.result.as_mut() else {
        unreachable!("successful query has a result")
    };
    assert!(grid.model.commit_cell(0, 1, "EUR").is_ok());
    tab.replace_query("SELECT * FROM missing_table");

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('r');
    assert!(
        harness
            .find("Replace results with unsaved edits?")
            .is_some()
    );
    let Some(neighbor) = harness
        .app_mut()
        .workbench
        .new_query("SELECT * FROM customers LIMIT 2")
    else {
        unreachable!("neighbor query can be created")
    };
    harness.draw();
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Enter);

    assert_eq!(harness.app().workbench.active_key(), Some(neighbor));
    let Some(Tab::Query(owner_tab)) = harness.app().workbench.tab(owner) else {
        unreachable!("captured owner remains a query")
    };
    let Some(status) = owner_tab.last_status.as_deref() else {
        unreachable!("confirmed saved query records status")
    };
    assert_eq!(
        (status.summary.as_str(), status.is_error, status.duration_ms),
        ("error", true, 42)
    );
    assert!(matches!(
        harness.app().workbench.tab(neighbor),
        Some(Tab::Query(tab)) if tab.last_status.is_none()
    ));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn unsupported_query_rejection_uses_explicit_timing() {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("UPDATE orders SET status = 'paid' WHERE id = 'x'"),
        QueryOutcome::Rejected { .. }
    ));
    assert_eq!(query_status(&app), ("error".to_owned(), true, 42));
}
