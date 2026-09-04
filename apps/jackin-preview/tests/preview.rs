use jackin_app::{
    ACCOUNT_ADD, ACCOUNT_PICKER, APP, App, ENTER, LAUNCH, LAUNCH_DIALOG, Motion, ROLE_CHOOSE,
    ROLE_PICKER, Route, RunId, Scenario,
};
use tui_next::{KeyCode, Theme};
use tui_next_testing::Harness;

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
fn launch_dialog_contains_a_nested_role_picker() {
    let mut harness = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        100,
        30,
    );
    let _ = harness.click_id(LAUNCH);
    assert!(harness.is_open(LAUNCH_DIALOG));
    assert!(harness.area_of(ROLE_CHOOSE).is_some());

    let _ = harness.click_id(ROLE_CHOOSE);
    assert!(harness.is_open(LAUNCH_DIALOG));
    assert!(harness.is_open(ROLE_PICKER));

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
