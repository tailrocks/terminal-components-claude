//! Deterministic replacements for the historical `TablePro` performance entry points.
//!
//! These retain the named smoke-test contracts while exercising only the
//! public application facade and domain adapter. Allocation budgets belong to
//! a separate harness because the old backend is outside this package.

use junie_tui::{Axis, ColumnKey, GridModel, ItemKey, KeyCode, SortDir, Theme};
use junie_tui_testing::{Harness, perf};
use tablepro_app::{ColType, QueryOutcome, ResultGrid, ResultSet, TableProApp, Value};

#[global_allocator]
static GLOBAL: perf::Counting = perf::Counting;

fn full_result_app() -> TableProApp {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
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
    let mut harness =
        Harness::new(full_result_app(), Theme::junie(), 120, 40).with_auto_draw(false);

    let stats = perf::bench(2, perf::iters(100), &mut || harness.draw());
    assert!(
        stats.allocs < 100,
        "frame_tablepro_grid_500x12_120x40 exceeded 100 allocs: {}",
        stats.allocs
    );

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
fn grid_100k_local_sort() {
    let _guard = perf::lock();
    let n = perf::big(100_000);
    let result = ResultSet {
        columns: vec![("id".to_owned(), ColType::Int)],
        rows: (0..n)
            .rev()
            .map(|value| vec![Value::Int(value as i64)])
            .collect(),
        total: n,
        source: Some("public.orders".to_owned()),
        duration_ms: 0,
        editable: false,
    };
    let mut grid = ResultGrid::from_result(&result);
    let stats = perf::bench(0, perf::iters(2), &mut || {
        grid.sort(ColumnKey::num(1), SortDir::Asc);
        grid.sort(ColumnKey::num(1), SortDir::Desc);
    });
    perf::report_to(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt"),
        "grid_100k_local_sort",
        &stats,
    );
    assert_eq!(grid.row_key(0), ItemKey::num(1));
    assert_eq!(grid.row_count(), n);
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
    let result = ResultSet {
        columns: (0..12)
            .map(|column| (format!("column_{column}"), ColType::Text))
            .collect(),
        rows: (0..500)
            .map(|row| {
                (0..12)
                    .map(|column| Value::Text(format!("{row}-{column}")))
                    .collect()
            })
            .collect(),
        total: 500,
        source: Some("public.orders".to_owned()),
        duration_ms: 0,
        editable: false,
    };
    let _guard = perf::lock();
    let mut grid = ResultGrid::empty();
    let stats = perf::bench(0, perf::iters(8), &mut || {
        grid = ResultGrid::from_result(&result);
        std::hint::black_box(&grid);
    });
    assert!(
        stats.allocs < 8_000,
        "grid_500x12_load exceeded 8000 allocs: {}",
        stats.allocs
    );
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
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let stats = perf::bench(2, perf::iters(100), &mut || harness.draw());
    assert!(
        stats.allocs < 40,
        "frame_tablepro_connection_form_120x40 exceeded 40 allocs: {}",
        stats.allocs
    );
    assert!(harness.find("Connect to database").is_some());
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn frame_tablepro_query_editor_2k_lines() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let query = format!(
        "-- generated query\n{}",
        "SELECT * FROM orders\n".repeat(2_000)
    );
    let _ = app.run_query(query);
    let mut harness = Harness::new(app, Theme::junie(), 120, 40).with_auto_draw(false);
    let stats = perf::bench(2, perf::iters(100), &mut || harness.draw());
    assert!(
        stats.allocs < 40,
        "frame_tablepro_query_editor_2k_lines exceeded 40 allocs: {}",
        stats.allocs
    );
    assert!(harness.find("TablePro").is_some());
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn perf_tablepro_baseline() {
    let _guard = perf::lock();
    let mut harness = Harness::new(full_result_app(), Theme::junie(), 120, 40);
    let stats = perf::bench(2, perf::iters(100), &mut || {
        harness.draw();
        std::hint::black_box(harness.focus());
    });
    perf::report_to(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt"),
        "frame_tablepro_grid_500x12_120x40",
        &stats,
    );
}
