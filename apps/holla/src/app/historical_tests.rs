//! Pinned794b095 actual-App assertions restored through the shared production runtime.
//! Fixture selection, frames, dimensions and product assertions remain source-derived.
use super::{App, Motion, Scenario};
use junie_tui::{FeedbackClock, KeyCode, SimulationMoment, Theme};
use junie_tui_testing::Harness;
fn fixture(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    width: u16,
    height: u16,
) -> Harness<App> {
    let app = App::for_scenario(scenario, motion, frame);
    let frame = u64::try_from(app.fixture_time_ms());
    assert!(frame.is_ok(), "fixture time must be nonnegative");
    let initial = SimulationMoment::from_millis(frame.unwrap_or_default());
    Harness::new_with_feedback_clock(
        app,
        Theme::junie(),
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
}
#[test]
fn paused_frames_are_byte_identical() {
    let a = fixture(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    let b = fixture(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    assert_eq!(a.text(), b.text());
}

#[test]
fn paused_ticks_do_not_advance_the_world() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let before = h.text();
    for _ in 0..20 {
        let _ = h.advance(std::time::Duration::from_millis(200));
    }
    assert_eq!(before, h.text());
    // discovery is a virtual-clock event: paused means it never fires
    assert!(h.app().world.discovering());
}

#[test]
fn remote_production_host_is_unmistakable() {
    let h = fixture(Scenario::RemoteHost, Motion::Paused, 0, 120, 40);
    let t = h.text();
    assert!(t.contains("◆ prod-eu-1 · ssh · production"), "{t}");
    assert!(t.contains("/srv/payments"), "{t}");
}

#[test]
fn too_small_notice_below_minimum() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.resize(71, 19);
    let t = h.text();
    assert!(t.contains("Terminal too small"), "{t}");
    assert!(t.contains("Need 72×20, have 71×19"), "{t}");
    assert!(t.contains("holla❯"), "{t}");
    // q quits from the notice
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.app().quit);
}

#[test]
fn hard_cases_shows_docker_failure_and_detached_head() {
    let h = fixture(Scenario::HardCases, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    assert!(t.contains("docker: discovery failed"), "{t}");
    assert!(t.contains("detached at 9f3a21c"), "{t}");
}

#[test]
fn monorepo_child_shows_parent_ecosystem_with_scope_tags() {
    let h = fixture(Scenario::MonorepoChild, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    // local frontend task ranks; parent workspace rows are tagged
    assert!(t.contains("Run dev"), "{t}");
    assert!(t.contains("workspace"), "{t}");
}
