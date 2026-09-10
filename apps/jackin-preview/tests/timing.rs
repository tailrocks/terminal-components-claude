//! Product ticks are coalesced wakes, not elapsed-time or input-count units.
use jackin_app::{App, Motion, Scenario};
use junie_tui::{KeyCode, Theme};
use junie_tui_testing::Harness;
use std::time::Duration;
fn session(scenario: Scenario, motion: Motion) -> Harness<App> {
    Harness::new(App::for_scenario(scenario, motion), Theme::junie(), 100, 30)
}
#[test]
fn initial_idle_wake_admits_one_product_step() {
    for (scenario, step) in [
        (Scenario::FirstUse, 33),
        (Scenario::Returning, 80),
        (Scenario::CapsuleMulti, 80),
        (Scenario::LaunchRunning, 33),
    ] {
        let mut h = session(scenario, Motion::Full);
        let _ = h.advance(Duration::from_millis(1800));
        assert_eq!(h.app().world.now_ms(), step, "{scenario:?}");
    }
}
#[test]
fn unchanged_time_ticks_do_not_age_world_or_launch() {
    for scenario in [
        Scenario::FirstUse,
        Scenario::Returning,
        Scenario::CapsuleMulti,
        Scenario::LaunchRunning,
    ] {
        let mut h = session(scenario, Motion::Full);
        h.ticks(3);
        assert_eq!(h.app().world.now_ms(), 0, "{scenario:?}");
        assert_eq!(h.app().frame(), 0, "{scenario:?}");
    }
}
#[test]
fn frame_seek_does_not_advance_fixture_world_clock() {
    for scenario in Scenario::ALL {
        let app = App::for_scenario_at(scenario, Motion::Paused, 7);
        assert_eq!(app.world.now_ms(), 0, "{scenario:?}");
    }
}
#[test]
fn early_input_and_draw_do_not_postpone_intro_deadline() {
    let mut h = session(Scenario::FirstUse, Motion::Full);
    let _ = h.advance(Duration::from_millis(32));
    let _ = h.key(KeyCode::Null);
    h.draw();
    let _ = h.tick();
    assert_eq!(h.app().world.now_ms(), 0);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().world.now_ms(), 33);
    assert_eq!(h.app().frame(), 1);
}
#[test]
fn paused_wakes_and_ticks_keep_animation_and_world_frozen() {
    for scenario in Scenario::ALL {
        let mut h = session(scenario, Motion::Paused);
        let frame = h.app().frame();
        let _ = h.advance(Duration::from_secs(5));
        h.ticks(3);
        assert_eq!(h.app().world.now_ms(), 0, "{scenario:?}");
        assert_eq!(h.app().frame(), frame, "{scenario:?}");
    }
}

#[test]
fn idle_manager_wakes_at_200_but_advances_80_virtual_milliseconds() {
    let mut app = App::for_scenario(Scenario::Returning, Motion::Full);
    app.world.daemons.clear();
    app.world.jobs.clear();
    let mut h = Harness::new(app, Theme::junie(), 100, 30);
    let _ = h.advance(Duration::from_millis(199));
    assert_eq!(h.app().world.now_ms(), 0);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().world.now_ms(), 80);
    let _ = h.advance(Duration::from_millis(1800));
    assert_eq!(h.app().world.now_ms(), 160);
}
#[test]
fn reduced_motion_keeps_fixed_virtual_steps_without_input_aging() {
    let mut h = session(Scenario::Returning, Motion::Reduced);
    let _ = h.advance(Duration::from_millis(1800));
    assert_eq!(h.app().world.now_ms(), 80);
    h.ticks(10);
    assert_eq!(h.app().world.now_ms(), 80);
}
#[test]
fn fractional_early_wakes_wait_for_the_exact_intro_deadline() {
    let mut h = session(Scenario::FirstUse, Motion::Full);
    let _ = h.advance(Duration::from_micros(32999));
    assert_eq!(h.app().frame(), 0);
    let _ = h.advance(Duration::from_micros(1));
    assert_eq!(h.app().frame(), 1);
    let _ = h.advance(Duration::ZERO);
    assert_eq!(h.app().frame(), 1);
}

#[test]
fn pending_job_accelerates_idle_wake_then_returns_to_idle_interval() {
    let mut app = App::for_scenario(Scenario::Returning, Motion::Full);
    app.world.daemons.clear();
    app.world.jobs.clear();
    let mut h = Harness::new(app, Theme::junie(), 100, 30);
    let _ = h.advance(Duration::from_millis(50));
    h.app_mut()
        .world
        .schedule(80, jackin_app::sim::world::Msg::Refreshed { ok: true });
    let _ = h.key(KeyCode::Null);
    let _ = h.advance(Duration::from_millis(29));
    assert_eq!(h.app().world.now_ms(), 0);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().world.now_ms(), 80);
    assert!(h.app().world.jobs.is_empty());
    let _ = h.advance(Duration::from_millis(199));
    assert_eq!(h.app().world.now_ms(), 80);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().world.now_ms(), 160);
}
#[test]
fn dismissed_animation_does_not_admit_the_old_early_deadline() {
    let mut app = App::for_scenario(Scenario::Returning, Motion::Full);
    app.world.daemons.clear();
    app.world.jobs.clear();
    app.world
        .schedule(800, jackin_app::sim::world::Msg::Refreshed { ok: true });
    let mut h = Harness::new(app, Theme::junie(), 100, 30);
    let _ = h.advance(Duration::from_millis(50));
    h.app_mut().world.jobs.clear();
    let _ = h.key(KeyCode::Null);
    let _ = h.advance(Duration::from_millis(30));
    assert_eq!(h.app().world.now_ms(), 0);
    let _ = h.advance(Duration::from_millis(120));
    assert_eq!(h.app().world.now_ms(), 80);
}

#[test]
fn account_validation_uses_animated_wake_without_elapsed_catchup() {
    let mut app = App::for_scenario(Scenario::AccountsMixed, Motion::Full);
    app.world.daemons.clear();
    app.world.jobs.clear();
    for account in &mut app.world.accounts.accounts {
        account.validation =
            jackin_app::domain::account::ValidationState::Validating { started_tick: 0 };
    }
    let mut h = Harness::new(app, Theme::junie(), 100, 30);
    let _ = h.advance(Duration::from_millis(79));
    assert_eq!(h.app().world.now_ms(), 0);
    let _ = h.advance(Duration::from_millis(1));
    assert_eq!(h.app().world.now_ms(), 80);
}
#[test]
fn active_modal_does_not_suspend_world_or_restart_its_deadline() {
    let mut h = session(Scenario::Returning, Motion::Full);
    let _ = h.click_id(jackin_app::LAUNCH);
    assert!(h.is_open(jackin_app::screens::manager::AGENT_PICKER));
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(h.app().world.now_ms(), 80);
    assert!(h.is_open(jackin_app::screens::manager::AGENT_PICKER));
    h.ticks(3);
    assert_eq!(h.app().world.now_ms(), 80);
    let _ = h.advance(Duration::from_millis(80));
    assert_eq!(h.app().world.now_ms(), 160);
}
