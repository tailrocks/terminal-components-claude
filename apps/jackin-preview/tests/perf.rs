//! Jackin application performance contracts.
//!
//! These are deliberately deterministic smoke measurements rather than a
//! second baseline writer.  The old app-level allocator rows are retired;
//! the tests keep their documented names while asserting the structural
//! properties that make the new facade renderer bounded.

use jackin_app::{APP, App, Motion, Route, Scenario};
use junie_tui::{KeyCode, Theme};
use junie_tui_testing::Harness;

#[test]
fn frame_jackin_capsule_4panes_120x40() {
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
    assert!(first.area_of(APP).is_some());
    assert_eq!(first.text(), second.text());
    assert!(first.text().contains("Capsule"));
}

#[test]
fn frame_jackin_manager_100rows_120x40() {
    let first = Harness::new(
        App::for_scenario(Scenario::HardCases, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    let second = Harness::new(
        App::for_scenario(Scenario::HardCases, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );

    assert_eq!(first.app().route(), Route::Manager);
    assert!(first.text().contains("preserved"));
    assert_eq!(first.text(), second.text());
    assert!(first.diagnostics().is_empty(), "{:?}", first.diagnostics());
}

#[test]
fn key_jackin_manager_move() {
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
