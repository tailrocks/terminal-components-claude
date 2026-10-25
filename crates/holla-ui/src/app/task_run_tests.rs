#![cfg(test)]

use super::{App, Motion, ReturnLocation, Route, TaskRunOverlayReturn, TaskRunOwner, home};
use crate::{
    dispatch::{self, Destination},
    domain::{
        action::{Action, Risk},
        activity::ActivityState,
        fixtures,
        mise::Trust,
        ranking::{Pin, Usage},
    },
    scenario::Scenario,
    screens::task_run::TaskRunScreen,
    sim::{
        catalogue,
        task_run::{TaskRun, TaskRunId, TaskRunState, TaskScriptId},
        world::World,
    },
};
use termrock::{FeedbackClock, KeyCode, Rect, SimulationMoment, Theme};
use termrock_test_support::Harness;

type TestResult = Result<(), String>;

fn dev_world(trust: Trust) -> World {
    let mut world = fixtures::world_for(Scenario::MonorepoChild);
    world.seek(4_000);
    if let Some(task) = world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = trust;
    }
    world
}

fn dev_action(world: &World) -> Result<Action, String> {
    catalogue::catalogue(world)
        .into_iter()
        .find(|action| action.id == "task:dev")
        .ok_or_else(|| "fixture dev action is missing".into())
}

fn trusted_app() -> App {
    let theme = Theme::junie();
    let mut app = App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000)
        .with_source_theme(theme);
    if let Some(task) = app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    app
}

fn native_fixture() -> Harness<App> {
    Harness::new_with_feedback_clock(
        trusted_app(),
        Theme::junie(),
        80,
        40,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4_000),
        },
    )
}

fn native_fixture_with_direct_scan_binding() -> Harness<App> {
    let mut app = trusted_app();
    let f9 = termrock::Key {
        code: KeyCode::F(9),
        mods: termrock::KeyModifiers::NONE,
    };
    assert_eq!(
        app.keymap.lookup(termrock::KeyPhase::Capture, &f9, false),
        None
    );
    assert_eq!(
        app.keymap.lookup(termrock::KeyPhase::Capture, &f9, true),
        None
    );
    app.keymap.add(
        termrock::KeyPhase::Capture,
        termrock::Chord::key(KeyCode::F(9)),
        super::DISK_SCAN,
    );
    assert_eq!(
        app.keymap.lookup(termrock::KeyPhase::Capture, &f9, false),
        Some(super::DISK_SCAN),
    );
    Harness::new_with_feedback_clock(
        app,
        Theme::junie(),
        80,
        40,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4_000),
        },
    )
}

struct QueuedTaskRunApp {
    inner: App,
    replace_with: Option<Action>,
    answer_before_update: Option<(TaskRunId, crate::sim::task_run::PromptId)>,
    prompt_transition_succeeded: bool,
}

impl termrock::App for QueuedTaskRunApp {
    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> termrock::Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Event {
            if let Some(action) = self.replace_with.take() {
                self.inner
                    .start_task_run(action, TaskScriptId::DevServerDemo, cx);
            }
            if let Some((run_id, prompt_id)) = self.answer_before_update.take() {
                let now_ms = self.inner.world.now_ms();
                self.prompt_transition_succeeded = self
                    .inner
                    .task_runs
                    .answer_prompt(run_id, prompt_id, "y", now_ms)
                    .is_ok();
                if self.prompt_transition_succeeded {
                    let next_ms = now_ms.saturating_add(100);
                    if let Ok(next_ms) = u64::try_from(next_ms) {
                        self.inner.world.seek(next_ms);
                        self.inner.advance_task_runs_to_world();
                        self.prompt_transition_succeeded = self
                            .inner
                            .task_runs
                            .get(run_id)
                            .and_then(TaskRun::active_prompt)
                            .is_none_or(|prompt| prompt.id() != prompt_id);
                    } else {
                        self.prompt_transition_succeeded = false;
                    }
                }
            }
        }
        termrock::App::update(&mut self.inner, cx)
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        termrock::App::draw(&self.inner, ui);
    }

    fn should_quit(&self) -> bool {
        termrock::App::should_quit(&self.inner)
    }

    fn keymap(&self) -> &termrock::KeyMap {
        termrock::App::keymap(&self.inner)
    }

    fn on_esc(&mut self, cx: &mut termrock::Cx<'_>) -> termrock::Response<()> {
        termrock::App::on_esc(&mut self.inner, cx)
    }
}

fn queued_fixture() -> Harness<QueuedTaskRunApp> {
    Harness::new_with_feedback_clock(
        QueuedTaskRunApp {
            inner: trusted_app(),
            replace_with: None,
            answer_before_update: None,
            prompt_transition_succeeded: false,
        },
        Theme::junie(),
        80,
        40,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4_000),
        },
    )
}

fn enter_dev_queued(harness: &mut Harness<QueuedTaskRunApp>) -> TestResult {
    let _ = harness.type_str("Run dev");
    let _ = harness.key(KeyCode::Enter);
    if !matches!(harness.app().inner.route, Route::TaskRun) {
        return Err(format!("dev route was not opened: {}", harness.text()));
    }
    Ok(())
}

fn enter_dev(harness: &mut Harness<App>) -> TestResult {
    let _ = harness.type_str("Run dev");
    let Some(selected) = harness.app().home.selected(&harness.app().world) else {
        return Err("dev query did not select a row".into());
    };
    assert_eq!(selected.id, "task:dev");
    let _ = harness.key(KeyCode::Enter);
    if !matches!(harness.app().route, Route::TaskRun) {
        return Err(format!("dev route was not opened: {}", harness.text()));
    }
    Ok(())
}

fn install_owner(app: &mut App) -> Result<(u64, TaskRunId), String> {
    let action = dev_action(&app.world)?;
    let activity_id = dispatch::next_activity_id(&app.world)
        .ok_or_else(|| "Activity identity unavailable".to_owned())?;
    let activity = dispatch::task_run_activity(&app.world, &action, activity_id);
    let now = app.world.now_ms();
    let run_id = app
        .task_runs
        .start(TaskScriptId::DevServerDemo, now)
        .map_err(|error| error.to_string())?;
    let _updates = app.task_runs.advance_to(now);
    app.world.activities.push(activity.clone());
    app.activities.show(activity_id);
    let route_id = app
        .next_task_run_route_id
        .ok_or_else(|| "TaskRun route identity unavailable".to_owned())?;
    app.next_task_run_route_id = route_id.checked_add(1);
    app.task_run_owners.insert(
        run_id,
        TaskRunOwner {
            route_id,
            scenario: app.world.scenario,
            cwd: app.world.cwd.clone(),
            effect_revision: app.world.effect_revision,
            action,
            activity,
            origin: ReturnLocation {
                route: Route::Home,
                focus: Some(home::QUERY),
            },
        },
    );
    app.task_run_screen = Some(TaskRunScreen::new(route_id, run_id, activity_id));
    app.route = Route::TaskRun;
    Ok((route_id, run_id))
}

fn assert_generic(world: &mut World, selected: &Action) -> TestResult {
    let expected = catalogue::resolve_intent(world, selected).map_err(|error| error.to_string())?;
    let expected_id = dispatch::next_activity_id(world)
        .ok_or_else(|| "generic Activity identity unavailable".to_owned())?;
    let count = world.activities.len();
    let host = world.host.clone();
    let cwd = world.cwd.clone();
    let revision = world.effect_revision;
    let destination = dispatch::invoke(world, selected);
    assert!(
        matches!(destination, Destination::Notice(ref message) if message.contains("Activity started:"))
    );
    assert_eq!(world.activities.len(), count + 1);
    let Some(activity) = world.activities.last() else {
        return Err("generic simulation omitted Activity".into());
    };
    assert_eq!(activity.id, expected_id);
    assert_eq!(activity.name, expected.title);
    assert_eq!(activity.scope, expected.workdir);
    assert!(matches!(activity.state, ActivityState::Running));
    assert_eq!(world.host, host);
    assert_eq!(world.cwd, cwd);
    assert_eq!(world.effect_revision, revision);
    Ok(())
}

#[test]
fn canonical_dev_is_previewed_then_started_only_as_registered_script() -> TestResult {
    let mut world = dev_world(Trust::Trusted);
    let action = dev_action(&world)?;
    assert!(matches!(
        dispatch::preview(&mut world, &action),
        Destination::Preview(_)
    ));
    assert!(matches!(
        dispatch::confirm_preview(&mut world, &action),
        Destination::TaskRunStart {
            script: TaskScriptId::DevServerDemo,
            ..
        }
    ));
    assert!(matches!(
        dispatch::invoke(&mut world, &action),
        Destination::TaskRunStart { .. }
    ));
    assert!(world.activities.is_empty());
    assert_eq!(world.effect_revision, 0);
    Ok(())
}

#[test]
fn risk_body_and_duplicate_source_changes_fail_closed() -> TestResult {
    let mut world = dev_world(Trust::Trusted);
    let mut risky = dev_action(&world)?;
    risky.risk = Risk::Bounded;
    assert!(matches!(
        dispatch::preview(&mut world, &risky),
        Destination::Notice(_)
    ));
    assert!(matches!(
        dispatch::invoke(&mut world, &risky),
        Destination::Notice(_)
    ));
    assert!(matches!(
        dispatch::confirm_preview(&mut world, &risky),
        Destination::Notice(_)
    ));

    let selected = dev_action(&world)?;
    let mut duplicate_world = dev_world(Trust::Trusted);
    let duplicate = duplicate_world
        .mise
        .as_ref()
        .and_then(|mise| mise.tasks.iter().find(|task| task.id == "dev"))
        .cloned()
        .ok_or("fixture dev task is missing")?;
    duplicate_world
        .mise
        .as_mut()
        .ok_or("Mise inventory is missing")?
        .tasks
        .push(duplicate);
    assert!(matches!(
        dispatch::invoke(&mut duplicate_world, &selected),
        Destination::Notice(_)
    ));

    let mut changed_world = dev_world(Trust::Trusted);
    let stale = dev_action(&changed_world)?;
    changed_world
        .mise
        .as_mut()
        .ok_or("Mise inventory is missing")?
        .tasks
        .iter_mut()
        .find(|task| task.id == "dev")
        .ok_or("fixture dev task is missing")?
        .command = "vite --port 5200".into();
    assert!(matches!(
        dispatch::invoke(&mut changed_world, &stale),
        Destination::Notice(_)
    ));
    assert!(matches!(
        dispatch::confirm_preview(&mut changed_world, &stale),
        Destination::Notice(_)
    ));
    assert!(changed_world.activities.is_empty());
    Ok(())
}

#[test]
fn generic_nonregistered_recent_and_pin_rows_keep_activity_behavior_and_alias_is_canonical()
-> TestResult {
    let mut changed = dev_world(Trust::Trusted);
    changed
        .mise
        .as_mut()
        .ok_or("Mise inventory is missing")?
        .tasks
        .iter_mut()
        .find(|task| task.id == "dev")
        .ok_or("fixture dev task is missing")?
        .command = "vite --port 5200".into();
    let fresh = dev_action(&changed)?;
    assert_generic(&mut changed, &fresh)?;

    let mut recent_world = dev_world(Trust::Trusted);
    let action = dev_action(&recent_world)?;
    recent_world.memory.usage.push(Usage {
        path: recent_world.cwd.clone(),
        command: action.command.clone(),
        count: 1,
    });
    let recent = catalogue::catalogue(&recent_world)
        .into_iter()
        .find(|action| action.id.starts_with("recent:") && action.intent_id() == Some("task:dev"))
        .ok_or("recent row missing")?;
    assert_generic(&mut recent_world, &recent)?;

    let mut pin_world = dev_world(Trust::Trusted);
    let action = dev_action(&pin_world)?;
    pin_world.memory.pins.push(Pin {
        path: pin_world.cwd.clone(),
        command: action.command.clone(),
    });
    let pin = catalogue::catalogue(&pin_world)
        .into_iter()
        .find(|action| action.id.starts_with("pin:") && action.intent_id() == Some("task:dev"))
        .ok_or("pin row missing")?;
    assert_generic(&mut pin_world, &pin)?;

    let mut alias_world = dev_world(Trust::Trusted);
    let mut alias = dev_action(&alias_world)?;
    alias.title.push_str(" · display alias");
    assert!(matches!(
        dispatch::invoke(&mut alias_world, &alias),
        Destination::TaskRunStart { .. }
    ));
    assert!(alias_world.activities.is_empty());
    Ok(())
}

#[test]
fn untrusted_file_trust_is_preserved_and_selected_body_is_revalidated() -> TestResult {
    let mut world = dev_world(Trust::Untrusted);
    let stale = dev_action(&world)?;
    assert!(matches!(
        dispatch::invoke(&mut world, &stale),
        Destination::Trust { args: None, .. }
    ));
    let task = world
        .mise
        .as_mut()
        .ok_or("Mise inventory missing")?
        .tasks
        .iter_mut()
        .find(|task| task.id == "dev")
        .ok_or("fixture dev task missing")?;
    task.trust = Trust::Trusted;
    task.command = "vite --port 5200".into();
    assert!(matches!(
        dispatch::confirm_preview(&mut world, &stale),
        Destination::Notice(_)
    ));
    assert!(world.activities.is_empty());
    assert_eq!(world.effect_revision, 0);
    Ok(())
}

#[test]
fn reserved_nextest_task_args_route_keeps_priority() -> TestResult {
    let mut world = crate::sim::fixtures::nextest_args_world();
    let action = catalogue::catalogue(&world)
        .into_iter()
        .find(|action| action.id == "task:test-nextest")
        .ok_or("nextest action missing")?;
    assert!(matches!(
        dispatch::invoke(&mut world, &action),
        Destination::TaskArgs(_)
    ));
    assert!(world.activities.is_empty());
    Ok(())
}

#[test]
fn covered_owner_return_is_exact_to_route_run_prompt_and_original_input() -> TestResult {
    let mut app = App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000);
    if let Some(task) = app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    let (route_id, run_id) = install_owner(&mut app)?;
    let activity = app
        .world
        .activities
        .last()
        .cloned()
        .ok_or("Activity missing")?;
    app.world.seek(4_100);
    app.advance_task_runs_to_world();
    let prompt_id = app
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("prompt missing")?;
    let input = TaskRunScreen::input_id_for(route_id, run_id.get(), prompt_id.get());
    assert_eq!(
        app.task_runs.get(run_id).map(TaskRun::duration_ms),
        Some(100)
    );

    app.disk_scan_task_run_return = Some(TaskRunOverlayReturn {
        route_id,
        run_id,
        prompt_id: Some(prompt_id),
    });
    app.route = Route::DiskScan;
    assert!(app.task_run_owner_binding_is_current(route_id, run_id));
    assert!(!app.task_run_route_is_current(route_id, run_id));
    assert!(!app.task_run_prompt_event_is_current(route_id, run_id, prompt_id));
    let location = ReturnLocation {
        route: Route::TaskRun,
        focus: Some(input),
    };
    assert_eq!(app.valid_disk_scan_return_focus(location), Some(input));
    app.disk_scan_task_run_return = Some(TaskRunOverlayReturn {
        route_id: route_id.saturating_add(1),
        run_id,
        prompt_id: Some(prompt_id),
    });
    assert_eq!(app.valid_disk_scan_return_focus(location), None);
    assert!(
        app.world
            .activities
            .last()
            .is_some_and(|current| current == &activity)
    );
    app.world.seek(4_090);
    app.advance_task_runs_to_world();
    assert_eq!(
        app.task_runs.get(run_id).map(TaskRun::duration_ms),
        Some(100)
    );
    Ok(())
}

#[test]
fn prompt_input_identity_and_short_area_contract_are_exact() {
    let first = TaskRunScreen::input_id_for(1, 10, 100);
    assert_ne!(first, TaskRunScreen::input_id_for(2, 10, 100));
    assert_ne!(first, TaskRunScreen::input_id_for(1, 11, 100));
    assert_ne!(first, TaskRunScreen::input_id_for(1, 10, 101));
    assert!(TaskRunScreen::input_area_for(Rect::new(3, 4, 30, 6), true).is_none());
    assert_eq!(
        TaskRunScreen::input_area_for(Rect::new(3, 4, 30, 7), true),
        Some(Rect::new(3, 9, 30, 1))
    );
    assert!(TaskRunScreen::input_area_for(Rect::new(3, 4, 30, 7), false).is_none());
}

#[test]
fn duplicate_activity_identity_and_effect_revision_invalidate_owner() -> TestResult {
    let mut app = App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000);
    if let Some(task) = app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    let (route_id, run_id) = install_owner(&mut app)?;
    let activity = app
        .world
        .activities
        .last()
        .cloned()
        .ok_or("Activity missing")?;
    app.world.activities.push(activity);
    assert!(!app.task_run_owner_is_live(route_id, run_id));

    let mut second =
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000);
    if let Some(task) = second
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    let (route_id, run_id) = install_owner(&mut second)?;
    second.world.effect_revision = second.world.effect_revision.saturating_add(1);
    assert!(!second.task_run_owner_is_live(route_id, run_id));
    Ok(())
}

#[test]
fn identity_exhaustion_does_not_create_an_activity_or_route() -> TestResult {
    let mut route_limit = native_fixture();
    route_limit.app_mut().next_task_run_route_id = None;
    let _ = route_limit.type_str("dev");
    let _ = route_limit.key(KeyCode::Enter);
    assert!(matches!(route_limit.app().route, Route::Home));
    assert!(route_limit.app().world.activities.is_empty());
    assert!(route_limit.app().task_run_screen.is_none());

    let mut activity_limit = native_fixture();
    let action = dev_action(&activity_limit.app().world)?;
    let last = dispatch::task_run_activity(&activity_limit.app().world, &action, u32::MAX);
    activity_limit.app_mut().world.activities.push(last);
    let before = activity_limit.app().world.activities.len();
    let _ = activity_limit.type_str("dev");
    let _ = activity_limit.key(KeyCode::Enter);
    assert!(matches!(activity_limit.app().route, Route::Home));
    assert_eq!(activity_limit.app().world.activities.len(), before);
    assert!(activity_limit.app().task_run_screen.is_none());
    Ok(())
}

#[test]
fn native_task_run_owner_roundtrips_through_scan_and_home_without_diagnostics() -> TestResult {
    let mut harness = native_fixture();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let prompt_id = harness
        .app()
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("prompt missing")?;
    let route_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::route_id)
        .ok_or("TaskRun screen missing")?;
    let input = TaskRunScreen::input_id_for(route_id, run_id.get(), prompt_id.get());
    assert!(harness.area_of(input).is_some());
    assert_eq!(harness.focus(), Some(input));

    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().route, Route::DiskScan));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(matches!(harness.app().route, Route::TaskRun));
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .map(TaskRunScreen::run_id),
        Some(run_id)
    );
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .and_then(TaskRunScreen::current_prompt_id),
        Some(prompt_id)
    );
    assert_eq!(harness.focus(), Some(input));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().route, Route::Home));
    assert!(harness.app().task_run_screen.is_none());
    for _ in 0..3 {
        let _ = harness.tick();
    }
    assert!(harness.app().retired_task_run_screens.is_empty());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn native_direct_scan_cover_drains_prompt_and_restores_the_same_input() -> TestResult {
    // The test-only F9 binding covers the direct command route; the stock menu journey has a separate control.
    let mut harness = native_fixture_with_direct_scan_binding();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let prompt_id = harness
        .app()
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("prompt missing")?;
    let route_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::route_id)
        .ok_or("TaskRun screen missing")?;
    let input = TaskRunScreen::input_id_for(route_id, run_id.get(), prompt_id.get());
    assert!(harness.area_of(input).is_some());
    assert_eq!(harness.focus(), Some(input));

    let _ = harness.type_str("draft");
    assert!(harness.text().contains("draft"), "{}", harness.text());
    assert!(!harness.app().menu.is_open());
    assert_eq!(harness.top_layer(), termrock::LayerId::PAGE);
    let _ = harness.key(KeyCode::F(9));
    assert!(matches!(harness.app().route, Route::DiskScan));
    assert!(!harness.app().menu.is_open());
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .map(TaskRunScreen::run_id),
        Some(run_id)
    );
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .and_then(TaskRunScreen::current_prompt_id),
        Some(prompt_id)
    );
    for _ in 0..2 {
        let _ = harness.tick();
    }
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(
        harness.app().task_runs.get(run_id).map(TaskRun::state),
        Some(TaskRunState::Waiting)
    ));

    let _ = harness.key(KeyCode::Esc);
    assert!(matches!(harness.app().route, Route::TaskRun));
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .map(TaskRunScreen::run_id),
        Some(run_id)
    );
    assert_eq!(
        harness
            .app()
            .task_run_screen
            .as_ref()
            .and_then(TaskRunScreen::current_prompt_id),
        Some(prompt_id)
    );
    assert_eq!(harness.focus(), Some(input));
    assert!(harness.text().contains("draft"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn queued_enter_cannot_answer_after_run_owner_replacement() -> TestResult {
    let mut harness = queued_fixture();
    enter_dev_queued(&mut harness)?;
    harness.app_mut().inner.world.seek(4_100);
    let _ = harness.tick();
    let old_run = harness
        .app()
        .inner
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("old TaskRun screen missing")?;
    let old_prompt = harness
        .app()
        .inner
        .task_runs
        .get(old_run)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("old prompt missing")?;
    let old_input = harness
        .app()
        .inner
        .task_run_screen
        .as_ref()
        .and_then(TaskRunScreen::current_input_id)
        .ok_or("old input owner missing")?;
    assert!(
        harness.area_of(old_input).is_some(),
        "old prompt input must be published before replacement"
    );
    assert_eq!(
        harness.focus(),
        Some(old_input),
        "Enter must route to the old prompt owner"
    );
    let action = dev_action(&harness.app().inner.world)?;
    harness.app_mut().replace_with = Some(action);

    // Harness routes Enter to the displayed old input before this wrapper replaces its owner.
    let _ = harness.key(KeyCode::Enter);
    let old = harness
        .app()
        .inner
        .task_runs
        .get(old_run)
        .ok_or("old run missing")?;
    assert!(matches!(old.state(), TaskRunState::Waiting));
    assert_eq!(
        old.active_prompt().map(|prompt| prompt.id()),
        Some(old_prompt)
    );
    let new_run = harness
        .app()
        .inner
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("replacement TaskRun screen missing")?;
    assert_ne!(new_run, old_run);
    assert!(harness.app().inner.task_runs.get(new_run).is_some());
    assert_eq!(harness.app().inner.world.activities.len(), 2);
    assert!(!harness.app().inner.retired_task_run_screens.is_empty());
    for _ in 0..3 {
        let _ = harness.tick();
    }
    assert!(harness.app().inner.retired_task_run_screens.is_empty());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn queued_enter_is_retired_when_its_prompt_owner_changes() -> TestResult {
    let mut harness = queued_fixture();
    enter_dev_queued(&mut harness)?;
    harness.app_mut().inner.world.seek(4_100);
    let _ = harness.tick();
    let run_id = harness
        .app()
        .inner
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    let prompt_id = harness
        .app()
        .inner
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("prompt missing")?;
    let old_input = harness
        .app()
        .inner
        .task_run_screen
        .as_ref()
        .and_then(TaskRunScreen::current_input_id)
        .ok_or("old input owner missing")?;
    assert!(
        harness.area_of(old_input).is_some(),
        "old prompt input must be published before answering"
    );
    assert_eq!(
        harness.focus(),
        Some(old_input),
        "Enter must route to the old prompt owner"
    );
    harness.app_mut().answer_before_update = Some((run_id, prompt_id));

    // The old slot receives Enter, then the modeled answer changes its owner before update.
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.app().prompt_transition_succeeded,
        "model prompt transition was not applied"
    );
    let run = harness
        .app()
        .inner
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(!matches!(run.state(), TaskRunState::Waiting));
    assert!(
        run.active_prompt()
            .is_none_or(|prompt| prompt.id() != prompt_id)
    );
    assert_eq!(
        harness
            .app()
            .inner
            .task_run_screen
            .as_ref()
            .map(TaskRunScreen::retired_input_count_for_test),
        Some(1)
    );
    for _ in 0..3 {
        let _ = harness.tick();
    }
    assert_eq!(
        harness
            .app()
            .inner
            .task_run_screen
            .as_ref()
            .map(TaskRunScreen::retired_input_count_for_test),
        Some(0)
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn missing_presented_input_area_cannot_accept_text_or_commit() -> TestResult {
    let mut harness = native_fixture();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let prompt_id = harness
        .app()
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("prompt missing")?;
    let input = harness
        .app()
        .task_run_screen
        .as_ref()
        .and_then(TaskRunScreen::current_input_id)
        .ok_or("input owner missing")?;
    let _ = harness.resize(40, 10);
    let _ = harness.tick();
    assert_eq!(harness.area_of(input), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.type_str("y");
    let _ = harness.key(KeyCode::Enter);
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(matches!(run.state(), TaskRunState::Waiting));
    assert_eq!(
        run.active_prompt().map(|prompt| prompt.id()),
        Some(prompt_id)
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn native_updates_advance_run_show_age_and_duration_without_world_effects() -> TestResult {
    let mut harness = native_fixture();
    let host = harness.app().world.host.clone();
    let cwd = harness.app().world.cwd.clone();
    enter_dev(&mut harness)?;
    let activity = harness
        .app()
        .world
        .activities
        .last()
        .cloned()
        .ok_or("Activity missing")?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    assert!(
        harness.text().contains("Stop modeled dev server?"),
        "{}",
        harness.text()
    );
    let input = harness
        .app()
        .task_run_screen
        .as_ref()
        .and_then(TaskRunScreen::current_input_id)
        .ok_or("prompt input owner missing")?;
    assert!(
        harness.area_of(input).is_some(),
        "first visible prompt input area"
    );
    assert_eq!(harness.focus(), Some(input), "first visible prompt focus");
    let _ = harness.type_str("y");
    assert_eq!(
        harness.focus(),
        Some(input),
        "the first character stays with the prompt input"
    );
    assert!(
        !harness.app().menu.is_open(),
        "the first character must not fall through to the menu"
    );
    let waiting = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(matches!(waiting.state(), TaskRunState::Waiting));
    assert!(waiting.active_prompt().is_some());
    let answer_at_ms = harness.app().world.now_ms();
    assert_eq!(
        answer_at_ms, 4_160,
        "the 80 ms fixture clock rounds this seek to 4160"
    );
    let _ = harness.key(KeyCode::Enter);
    let finish_at_ms = answer_at_ms.saturating_add(100);
    let finish_at_ms = u64::try_from(finish_at_ms)
        .map_err(|_| "TaskRun deadline must be nonnegative".to_owned())?;
    harness.app_mut().world.seek(finish_at_ms);
    let _ = harness.tick();
    assert_eq!(
        harness.app().world.now_ms(),
        4_320,
        "the finish deadline rounds to the next 80 ms frame"
    );
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(matches!(run.state(), TaskRunState::Completed));
    assert!(
        run.output()
            .iter()
            .any(|line| line.text() == "dev server stopped by operator")
    );
    assert!(
        !run.output()
            .iter()
            .any(|line| line.text() == "serving at localhost:5199")
    );
    assert!(
        harness
            .app()
            .world
            .activities
            .last()
            .is_some_and(|current| current == &activity)
    );
    assert_eq!(harness.app().world.cwd, cwd);
    assert_eq!(harness.app().world.host, host);
    assert_eq!(harness.app().world.effect_revision, 0);
    let age_ms = harness
        .app()
        .world
        .now_ms()
        .saturating_sub(activity.started_ms);
    assert_eq!(age_ms, 320);
    assert!(
        harness.text().contains(&format!("age {age_ms} ms")),
        "{}",
        harness.text()
    );
    assert!(
        harness.text().contains("TaskRun active duration: 200 ms"),
        "{}",
        harness.text()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn native_default_continues_and_visible_interrupt_cancels_only_the_model() -> TestResult {
    let mut harness = native_fixture();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let _ = harness.key(KeyCode::Enter);
    let serving_deadline_ms = harness.app().world.now_ms().saturating_add(100);
    let serving_deadline_ms = u64::try_from(serving_deadline_ms)
        .map_err(|_| "serving deadline must be nonnegative".to_owned())?;
    harness.app_mut().world.seek(serving_deadline_ms);
    let _ = harness.tick();
    assert_eq!(harness.app().world.now_ms(), 4_320);
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(
        run.output()
            .iter()
            .any(|line| line.text() == "dev server remains available")
    );
    assert!(
        run.output()
            .iter()
            .any(|line| line.text() == "serving at localhost:5199")
    );

    let mut harness = native_fixture();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let _ = harness.ctrl('c');
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert!(harness.app().world.activities.len() == 1);
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn interrupt_under_menu_does_not_cancel_hidden_prompt() -> TestResult {
    let mut harness = native_fixture();
    enter_dev(&mut harness)?;
    let run_id = harness
        .app()
        .task_run_screen
        .as_ref()
        .map(TaskRunScreen::run_id)
        .ok_or("TaskRun screen missing")?;
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    let _ = harness.key(KeyCode::F(10));
    assert!(harness.app().menu.is_open());
    let _ = harness.ctrl('c');
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun missing")?;
    assert!(matches!(run.state(), TaskRunState::Waiting));
    assert!(run.active_prompt().is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}
