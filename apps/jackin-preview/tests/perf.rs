//! Jackin application performance contracts.
//!
//! These deterministic measurements exercise the migrated application's
//! public runtime facade. They keep the three documented Slice 7 rows while
//! making the deleted per-frame viewport clone an explicit acceptance check.

use std::hint::black_box;

use jackin_app::{APP, App, Motion, Route, Scenario};
use junie_tui::{Input, Key, KeyCode, KeyModifiers, Theme};
use junie_tui_testing::{Harness, perf};

#[global_allocator]
static GLOBAL: perf::Counting = perf::Counting;

const BASELINE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt");

/// The application baseline carries the three live Slice 7 measurements.
/// The old per-frame viewport clone is intentionally absent: its deletion is
/// an acceptance condition, not another benchmark row.
#[test]
fn jackin_perf_baseline_has_required_rows_and_no_deleted_clone() {
    const ROWS: [&str; 3] = [
        "frame_jackin_capsule_4panes_120x40",
        "frame_jackin_manager_100rows_120x40",
        "key_jackin_manager_move",
    ];
    let names: Vec<_> = include_str!("perf_baseline.txt")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| !name.starts_with('#'))
        .collect();
    assert_eq!(names, ROWS);
    assert!(!names.contains(&"capsule_pane_clone_4x2000"));
}

// ------------------------------------------------------------ A. frames

/// Manager render at the documented representative size.
#[test]
fn frame_jackin_manager_100rows_120x40() {
    let _guard = perf::lock();
    let mut harness = Harness::new(
        App::for_scenario(Scenario::HardCases, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    let stats = perf::bench(2, perf::iters(100), &mut || {
        harness.draw();
        black_box(harness.focus());
    });
    perf::report_to(BASELINE, "frame_jackin_manager_100rows_120x40", &stats);
    if perf::env_flag("PERF_STRICT") {
        assert!(
            stats.allocs < 60,
            "manager frame allocates {} times",
            stats.allocs
        );
    }
}

/// Capsule render at the documented representative size.
///
/// The current public fixture is deterministic and exercises the migrated
/// Capsule route. The removed legacy four-pane scrollback clone is checked by
/// the baseline-shape test above and is not recreated in the app test.
#[test]
fn frame_jackin_capsule_4panes_120x40() {
    let _guard = perf::lock();
    let mut harness = Harness::new(
        App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    assert_eq!(harness.app().route(), Route::Capsule);
    assert!(harness.area_of(APP).is_some());
    let stats = perf::bench(2, perf::iters(100), &mut || {
        harness.draw();
        black_box(harness.focus());
    });
    perf::report_to(BASELINE, "frame_jackin_capsule_4panes_120x40", &stats);
    if perf::env_flag("PERF_STRICT") {
        assert!(
            stats.allocs < 200,
            "capsule frame allocates {} times",
            stats.allocs
        );
    }
}

// ------------------------------------------------------------ B. events

/// Alternating manager movement through the runtime without a redraw.
#[test]
fn key_jackin_manager_move() {
    let _guard = perf::lock();
    let mut harness = Harness::new(
        App::for_scenario(Scenario::HardCases, Motion::Paused),
        Theme::junie(),
        120,
        40,
    )
    .with_auto_draw(false);
    let mut down = true;
    let stats = perf::bench(10, perf::iters(1000), &mut || {
        let code = if down { KeyCode::Down } else { KeyCode::Up };
        down = !down;
        let _ = black_box(harness.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })));
    });
    perf::report_to(BASELINE, "key_jackin_manager_move", &stats);
    if perf::env_flag("PERF_STRICT") {
        assert_eq!(
            stats.allocs, 0,
            "manager key path allocates {} times",
            stats.allocs
        );
    }
}

// ------------------------------------------------------------ C. facade smoke

#[test]
fn jackin_facade_frames_are_deterministic() {
    let first = Harness::new(
        App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    let second = Harness::new(
        App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );

    assert_eq!(first.app().route(), Route::Capsule);
    assert_eq!(first.text(), second.text());
    assert!(first.text().contains("Capsule"));
    assert!(first.diagnostics().is_empty(), "{:?}", first.diagnostics());
}

#[test]
fn jackin_manager_key_round_trip_preserves_focus() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    let before = harness.focus();
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Up);
    assert_eq!(harness.focus(), before);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
