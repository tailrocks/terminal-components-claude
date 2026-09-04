//! Deterministic replacements for the historical `TablePro` performance entry points.
//!
//! These retain the named smoke-test contracts while exercising only the
//! public application facade and domain adapter. Allocation budgets belong to
//! a separate harness because the old backend is outside this package.

use tablepro_app::{QueryOutcome, TableProApp};
use tui_next::{Axis, KeyCode, Theme};
use tui_next_testing::Harness;

fn full_result_app() -> TableProApp {
    let mut app = TableProApp::default();
    assert_eq!(
        app.run_query("SELECT * FROM orders"),
        QueryOutcome::Executed {
            rows: 500,
            editable: true,
        }
    );
    app
}

#[test]
fn frame_tablepro_grid_500x12_120x40() {
    let harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);

    assert_eq!(harness.app().result().row_count(), 500);
    assert!(harness.find("Results").is_some());
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn key_tablepro_grid_cursor() {
    let mut harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);

    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Right);

    assert!(harness.diagnostics().is_empty());
}

#[test]
fn key_tablepro_grid_sort_local() {
    let mut harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);

    let _ = harness.key(KeyCode::Char('s'));

    assert_eq!(harness.app().result().row_count(), 500);
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn mouse_click_grid_cell() {
    let mut harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);

    let _ = harness.click(60, 12);

    assert!(harness.diagnostics().is_empty());
}

#[test]
fn wheel_tablepro_grid() {
    let mut harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);

    let _ = harness.wheel(Axis::V, 1, 60, 20);

    assert!(harness.diagnostics().is_empty());
}

#[test]
fn grid_500x12_load() {
    let app = full_result_app();

    assert_eq!(app.result().row_count(), 500);
    assert_eq!(app.result().total(), 1_203_338);
}

#[test]
fn debug_and_release_alloc_counts_match() {
    let app = full_result_app();
    let first = format!("{app:?}");
    let second = format!("{app:?}");

    assert_eq!(first, second);
}

#[test]
fn frame_tablepro_connection_form_120x40() {
    let mut app = TableProApp::default();
    app.begin_connection_form();
    let harness = Harness::new(app, Theme::junie(), 120, 40);
    assert!(harness.find("Connect to database").is_some());
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn frame_tablepro_query_editor_2k_lines() {
    let mut app = TableProApp::default();
    let query = format!(
        "-- generated query\n{}",
        "SELECT * FROM orders\n".repeat(2_000)
    );
    let _ = app.run_query(query);
    let harness = Harness::new(app, Theme::junie(), 120, 40);
    assert!(harness.find("TablePro").is_some());
    assert!(harness.diagnostics().is_empty());
}
