//! Focused deterministic integration coverage for the Jackin Preview shell.

use jackin_app::{
    ACCOUNT_ADD, ACCOUNT_PICKER, APP, App, ENTER, LAUNCH, LAUNCH_DIALOG, Motion, ROLE_CHOOSE,
    ROLE_PICKER, Route, RunId, Scenario,
};
use junie_tui::{KeyCode, Theme};
use junie_tui_testing::Harness;

#[test]
fn first_use_flow_enters_the_manager() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::FirstUse, Motion::Paused),
        Theme::junie(),
        100,
        30,
    );
    assert_eq!(harness.app().route(), Route::Intro);
    assert!(harness.text().contains("No running instances found"));
    assert!(harness.area_of(ENTER).is_some());

    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().route(), Route::Manager);
    assert!(harness.text().contains("Workspaces & instances"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn product_routes_and_account_picker_render_through_the_facade() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        100,
        30,
    );
    assert!(harness.text().contains("Workspaces & instances"));

    let _ = harness.key(KeyCode::Char('a'));
    assert_eq!(harness.app().route(), Route::Accounts);
    assert!(harness.text().contains("Account & Usage Center"));
    let _ = harness.click_id(ACCOUNT_ADD);
    assert!(harness.is_open(ACCOUNT_PICKER));
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(ACCOUNT_PICKER));

    let _ = harness.key(KeyCode::Char('u'));
    assert_eq!(harness.app().route(), Route::Usage);
    assert!(harness.text().contains("Usage overview"));
    let _ = harness.key(KeyCode::Char('s'));
    assert_eq!(harness.app().route(), Route::Settings);
    assert!(harness.text().contains("Settings"));
    let _ = harness.key(KeyCode::Char('c'));
    assert_eq!(harness.app().route(), Route::Capsule);
    assert!(harness.text().contains("Capsule"));
    assert!(harness.area_of(APP).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn nested_overlay_picker_inside_dialog() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        100,
        30,
    );
    let _ = harness.click_id(LAUNCH);
    assert!(harness.is_open(LAUNCH_DIALOG));
    assert!(harness.area_of(LAUNCH_DIALOG).is_some());
    assert!(harness.area_of(ROLE_CHOOSE).is_some());

    let _ = harness.click_id(ROLE_CHOOSE);
    assert!(harness.is_open(LAUNCH_DIALOG));
    assert!(harness.is_open(ROLE_PICKER));
    assert!(harness.area_of(ROLE_PICKER).is_some());
    assert!(harness.text().contains("Choose a role"));

    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(ROLE_PICKER));
    assert!(harness.is_open(LAUNCH_DIALOG));
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(LAUNCH_DIALOG));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn launch_simulation_is_deterministic_and_run_id_is_typed() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::LaunchRunning, Motion::Full),
        Theme::junie(),
        100,
        30,
    );
    let run_id = harness.app().launch().map(|run| run.run_id);
    assert_eq!(run_id, Some(RunId::new(0x9c41_e2f0)));
    harness.ticks(320);
    assert_eq!(harness.app().route(), Route::Capsule);
    assert!(harness.app().launch().is_some_and(|run| run.done));

    let first = RunId::from_label("same fixture");
    let second = RunId::from_label("same fixture");
    let other = RunId::from_label("different fixture");
    assert_eq!(first, second);
    assert_ne!(first, other);
    assert_eq!(first.short().len(), 8);
    assert_eq!(format!("{first}"), format!("run-{}", first.short()));
}

#[test]
fn resize_preserves_focus_and_does_not_create_diagnostics() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        100,
        30,
    );
    assert!(harness.tab_to(LAUNCH));
    let before = harness.focus();
    let _ = harness.resize(84, 24);
    assert_eq!(harness.focus(), before);
    assert!(harness.area_of(APP).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn scenarios_and_references_are_stable_without_secret_material() {
    for scenario in Scenario::ALL {
        let first = jackin_app::sim::world::world_for(scenario);
        let second = jackin_app::sim::world::world_for(scenario);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
    }

    let world = jackin_app::sim::world::world_for(Scenario::AccountsMixed);
    let debug = format!("{world:?}");
    assert!(debug.contains("v_eng01"));
    assert!(debug.contains("it_cdx01"));
    assert!(!debug.contains("op://v_eng01/it_cdx01/credential"));
    assert!(!debug.contains("openai:valid-cdx01"));
    assert!(!debug.contains("anthropic:valid-ant01"));

    let harness = Harness::new(
        App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
        Theme::junie(),
        120,
        30,
    );
    let rendered = harness.text();
    assert!(rendered.contains("Engineering"));
    assert!(!rendered.contains("openai:valid-cdx01"));
    assert!(!rendered.contains("anthropic:valid-ant01"));
}

#[test]
fn pinned_frames_keep_each_ritual_route_reachable() {
    let first = App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 0);
    assert_eq!(first.route(), Route::Intro);
    let completed = App::for_scenario_at(
        Scenario::FirstUse,
        Motion::Paused,
        jackin_app::rain::INTRO_END,
    );
    assert_eq!(completed.route(), Route::Manager);

    let cockpit = App::for_scenario_at(Scenario::LaunchRunning, Motion::Paused, 0);
    assert_eq!(cockpit.route(), Route::Cockpit);
    assert_eq!(cockpit.route_tick_ms(), jackin_app::rain::TICK_MS);

    let capsule = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 0);
    assert_eq!(capsule.route(), Route::Capsule);
    assert_eq!(capsule.route_tick_ms(), 80);

    let outro = App::for_scenario_at(Scenario::OutroLast, Motion::Paused, 1);
    assert_eq!(outro.route(), Route::Outro);
    assert_eq!(outro.route_tick_ms(), jackin_app::rain::TICK_MS);
}

#[test]
fn paused_frames_freeze_the_virtual_clock() {
    let mut harness = Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Paused, 12),
        Theme::junie(),
        100,
        30,
    );
    let before = harness.app().world.now_ms();
    harness.ticks(20);
    assert_eq!(harness.app().world.now_ms(), before);
    assert_eq!(harness.app().frame(), 12);
}

#[test]
fn container_uid_is_total_for_every_fixture_scenario() {
    for scenario in Scenario::ALL {
        let world = jackin_app::sim::world::world_for(scenario);
        for instance in world.instances {
            let uid = instance.container_uid();
            assert_eq!(uid.len(), 16);
            assert!(uid.starts_with("3f9c"));
        }
    }
    let short = RunId::from_label("");
    assert_eq!(short.short().len(), 8);
    assert_eq!(short.container_uid().len(), 16);
}

#[test]
fn every_named_scenario_renders_a_deterministic_frame() {
    for scenario in Scenario::ALL {
        let first = Harness::new(
            App::for_scenario_at(scenario, Motion::Paused, 0),
            Theme::junie(),
            120,
            40,
        );
        let second = Harness::new(
            App::for_scenario_at(scenario, Motion::Paused, 0),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(first.text(), second.text(), "{}", scenario.name());
        assert!(first.diagnostics().is_empty(), "{}", scenario.name());
    }
}
