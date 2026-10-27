#![cfg(test)]

use super::{App, Motion, ReturnLocation, Route, TaskRunOwner, home};
use crate::{
    dispatch,
    domain::{
        activity::{Activity, ActivityState},
        mise::Trust,
    },
    scenario::Scenario,
    screens::{
        activity::{self, StopEvent},
        task_run::TaskRunScreen,
    },
    sim::{
        activity_supervisor::{
            ActivityExecutionId, ActivityExecutionPhase, ActivityExecutionStart,
            ActivityStopMilestone, ActivityStopPhase, ActivityStopProfile,
        },
        catalogue,
        task_run::{TaskRun, TaskRunId, TaskRunState, TaskScriptId},
    },
};
use termrock::{FeedbackClock, Id, ItemKey, KeyCode, SimulationMoment, Theme};
use termrock_test_support::Harness;

type TestResult = Result<(), String>;

fn trusted_app() -> App {
    let mut app = App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000);
    if let Some(task) = app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    app.with_source_theme(Theme::junie())
}

fn harness(app: App) -> Harness<App> {
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

fn parity_activity_stop_app() -> App {
    App::for_scenario_without_history(Scenario::ParityActivityStop, Motion::Paused, 4_000)
        .with_source_theme(Theme::junie())
}

fn scenario_activity_execution(
    app: &App,
    activity_id: u32,
) -> Result<(Activity, ActivityExecutionId), String> {
    let activity = app
        .world
        .activities
        .iter()
        .find(|activity| activity.id == activity_id)
        .cloned()
        .ok_or_else(|| format!("scenario Activity {activity_id} is missing"))?;
    let execution_id = app
        .activity_executions
        .get(&activity_id)
        .copied()
        .ok_or_else(|| format!("scenario Activity {activity_id} has no captured execution"))?;
    Ok((activity, execution_id))
}

fn enter_dev(harness: &mut Harness<App>) {
    let _ = harness.type_str("Run dev");
    let _ = harness.key(KeyCode::Enter);
}

fn install_task_run(app: &mut App) -> Result<(TaskRunId, ActivityExecutionId, u64), String> {
    let action = catalogue::catalogue(&app.world)
        .into_iter()
        .find(|action| action.id == "task:dev")
        .ok_or_else(|| "fixture dev action is missing".to_owned())?;
    let activity_id =
        dispatch::next_activity_id(&app.world).ok_or_else(|| "Activity ID exhausted".to_owned())?;
    let route_id = app
        .next_task_run_route_id
        .ok_or_else(|| "TaskRun route ID exhausted".to_owned())?;
    let now_ms = app.world.now_ms();
    let run_id = app
        .task_runs
        .start(TaskScriptId::DevServerDemo, now_ms)
        .map_err(|error| error.to_string())?;
    let activity = dispatch::task_run_activity(&app.world, &action, activity_id);
    let execution_id = app
        .world
        .activity_supervisor
        .register(
            &activity,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Resistant,
        )
        .map_err(|error| error.to_string())?;
    app.next_task_run_route_id = route_id.checked_add(1);
    app.activity_executions.insert(activity_id, execution_id);
    app.world.activities.push(activity.clone());
    app.activities.show(activity_id);
    app.task_run_owners.insert(
        run_id,
        TaskRunOwner {
            route_id,
            execution_id,
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
    let mut screen = TaskRunScreen::new(route_id, run_id, activity_id);
    screen.bind_execution(execution_id);
    app.task_run_screen = Some(screen);
    app.route = Route::TaskRun;
    app.advance_task_runs_to_world();
    Ok((run_id, execution_id, route_id))
}

fn activity_stop_id(activity_id: u32, view_token: u64, execution_id: ActivityExecutionId) -> Id {
    Id::root("holla.activity.stop")
        .item(ItemKey::pair(view_token, execution_id.generation()))
        .item(ItemKey::num(u64::from(activity_id)))
}

fn task_run_stop_id(
    route_id: u64,
    run_id: TaskRunId,
    activity_id: u32,
    execution_id: ActivityExecutionId,
) -> Id {
    Id::root("holla.task-run.stop")
        .item(ItemKey::pair(route_id, run_id.get()))
        .item(ItemKey::pair(
            u64::from(activity_id),
            execution_id.generation(),
        ))
}

fn update_detail_route(app: &mut App, activity_id: u32) -> Result<u64, String> {
    app.route = Route::Activity;
    app.activities
        .show_with_token(activity_id)
        .ok_or_else(|| "Activity view token exhausted".to_owned())
}

#[test]
fn task_run_producer_and_task_run_stop_button_keep_output_and_grace_ownership() -> TestResult {
    let mut harness = harness(trusted_app());
    enter_dev(&mut harness);
    assert!(harness.text().contains("Execution: running"));
    let (run_id, execution_id, route_id) = {
        let app = harness.app();
        let screen = app
            .task_run_screen
            .as_ref()
            .ok_or("TaskRun screen was not published")?;
        assert!(matches!(app.route, Route::TaskRun));
        assert_eq!(app.world.activities.len(), 1);
        let activity = app
            .world
            .activities
            .iter()
            .find(|activity| activity.id == screen.activity_id())
            .ok_or("TaskRun Activity was not published")?;
        let execution_id = app
            .activity_executions
            .get(&activity.id)
            .copied()
            .ok_or("TaskRun supervisor registration is missing")?;
        assert_eq!(
            app.world
                .activity_supervisor
                .execution(execution_id)
                .map(|execution| execution.snapshot()),
            Some(activity)
        );
        assert_eq!(screen.execution_id(), Some(execution_id));
        let owner = app
            .task_run_owners
            .get(&screen.run_id())
            .ok_or("TaskRun owner was not published")?;
        assert_eq!(owner.execution_id, execution_id);
        assert!(app.task_run_owner_is_live(screen.route_id(), screen.run_id()));
        assert!(app.task_run_owner_binding_is_current(screen.route_id(), screen.run_id()));
        (screen.run_id(), execution_id, screen.route_id())
    };
    let activity_id = execution_id.activity_id();
    let other_activity = Activity::new(
        activity_id.checked_add(1).ok_or("Activity ID exhausted")?,
        "other execution",
        &harness.app().world.cwd,
        ActivityState::Running,
        harness.app().world.now_ms(),
    );
    let other_execution_id = harness
        .app_mut()
        .world
        .activity_supervisor
        .register(
            &other_activity,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )
        .map_err(|error| error.to_string())?;
    harness
        .app_mut()
        .task_run_screen
        .as_mut()
        .ok_or("TaskRun screen disappeared")?
        .bind_execution(other_execution_id);
    assert!(
        !harness
            .app()
            .task_run_owner_binding_is_current(route_id, run_id)
    );
    harness
        .app_mut()
        .task_run_screen
        .as_mut()
        .ok_or("TaskRun screen disappeared")?
        .bind_execution(execution_id);
    assert!(
        harness
            .app()
            .task_run_owner_binding_is_current(route_id, run_id)
    );
    harness.app_mut().world.seek(4_100);
    let _ = harness.tick();
    assert!(harness.text().contains("Execution: waiting"));
    let before_output: Vec<_> = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing")?
        .output()
        .iter()
        .map(|line| line.text().to_owned())
        .collect();
    assert!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .and_then(TaskRun::active_prompt)
            .is_some(),
        "the setup must exercise immediate prompt-wait settlement"
    );
    let active_duration = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing")?
        .duration_ms();
    let stop_id = task_run_stop_id(route_id, run_id, activity_id, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "TaskRun Stop is visible"
    );
    let _ = harness.click_id(stop_id);

    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing after Stop")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        before_output,
        "explicit Stop freezes authored output immediately"
    );
    assert_eq!(harness.app().world.effect_revision, 0);
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor execution disappeared before reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    let stop_at = execution
        .stop_requested_ms()
        .ok_or("stop time is missing")?;
    assert_eq!(
        execution.reap_deadline_ms(),
        stop_at.checked_add(960),
        "the resistant profile retains its modeled 800 ms KILL and 960 ms reap contract"
    );
    assert!(
        harness.text().contains("TaskRun active duration:"),
        "active TaskRun duration remains a separate UI value"
    );
    assert!(harness.text().contains("Execution: stopping"));

    harness
        .app_mut()
        .world
        .seek(u64::try_from(stop_at + 800).map_err(|error| error.to_string())?);
    let _ = harness.tick();
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| (execution.phase(), execution.stop_phase())),
        Some((
            ActivityExecutionPhase::Cancelling,
            ActivityStopPhase::KillSent
        ))
    );
    assert!(harness.text().contains("Execution: stopping"));
    harness
        .app_mut()
        .world
        .seek(u64::try_from(stop_at + 960).map_err(|error| error.to_string())?);
    let _ = harness.tick();
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| (execution.phase(), execution.stop_phase())),
        Some((ActivityExecutionPhase::Stopped, ActivityStopPhase::Reaped))
    );
    assert_eq!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .ok_or("TaskRun is missing after reap")?
            .output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        before_output,
        "later reaping cannot resume authored output"
    );
    assert!(harness.text().contains("Execution: stopped"));
    Ok(())
}

#[test]
fn activity_detail_stop_button_freezes_linked_taskrun_but_keeps_supervisor_owner() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, _) = install_task_run(&mut app)?;
    app.world.seek(4_100);
    app.advance_task_runs_to_world();
    let activity_id = execution_id.activity_id();
    assert!(
        app.task_runs
            .get(run_id)
            .and_then(TaskRun::active_prompt)
            .is_some(),
        "Activity detail Stop must exercise prompt-wait settlement"
    );
    let active_duration = app
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing before Activity Stop")?
        .duration_ms();
    let output_before_stop = app
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing before Activity Stop")?
        .output()
        .iter()
        .map(|line| line.text().to_owned())
        .collect::<Vec<_>>();
    let view_token = update_detail_route(&mut app, activity_id)?;
    let mut harness = harness(app);
    let _ = harness.tick();
    let stop_id = activity_stop_id(activity_id, view_token, execution_id);
    assert!(harness.area_of(stop_id).is_some(), "detail Stop is visible");
    let _ = harness.click_id(stop_id);

    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun is missing after Activity Stop")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before_stop,
        "Activity Stop freezes linked authored output immediately"
    );
    assert!(run.active_prompt().is_none());
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor execution was released early")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    let projection = activity::project(
        &execution.snapshot().clone(),
        Some(execution),
        Some(run),
        harness.app().world.now_ms(),
    );
    assert_eq!(projection.state_label, "stopping");
    assert_eq!(projection.stop_grace_ms, Some(0));
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(
        !harness.tab_to(stop_id),
        "a Cancelling Activity Stop button is disabled in draw and focus traversal"
    );
    let _ = harness.click_id(stop_id);
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor execution disappeared after disabled click")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.milestones().len(), 1);
    Ok(())
}

#[test]
fn stale_view_snapshot_and_generation_events_do_not_stop_a_replacement() -> TestResult {
    let mut app = trusted_app();
    let activity_id =
        dispatch::next_activity_id(&app.world).ok_or_else(|| "Activity ID exhausted".to_owned())?;
    let snapshot = Activity::new(
        activity_id,
        "modeled log follower",
        &app.world.cwd,
        ActivityState::Running,
        app.world.now_ms(),
    );
    let old_execution = app
        .world
        .activity_supervisor
        .register(
            &snapshot,
            ActivityExecutionStart::Detached,
            ActivityStopProfile::Responsive,
        )
        .map_err(|error| error.to_string())?;
    app.activity_executions.insert(activity_id, old_execution);
    app.world.activities.push(snapshot.clone());
    let first_token = update_detail_route(&mut app, activity_id)?;
    let stale_view = StopEvent {
        view_token: first_token,
        execution_id: old_execution,
        snapshot: snapshot.clone(),
    };
    let second_token = app
        .activities
        .show_with_token(activity_id)
        .ok_or_else(|| "Activity view token exhausted".to_owned())?;
    app.handle_activity_stop_event(stale_view, true);
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(old_execution)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Detached)
    );

    let duplicate = app
        .world
        .activities
        .iter()
        .find(|activity| activity.id == activity_id)
        .cloned()
        .ok_or("Activity missing")?;
    app.world.activities.push(duplicate);
    app.handle_activity_stop_event(
        StopEvent {
            view_token: second_token,
            execution_id: old_execution,
            snapshot: snapshot.clone(),
        },
        true,
    );
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(old_execution)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Detached),
        "duplicate display IDs fail closed"
    );
    app.world.activities.pop();

    app.world
        .activity_supervisor
        .observe_terminal_exit(old_execution, &snapshot)
        .map_err(|error| error.to_string())?;
    let mut replacement = snapshot.clone();
    replacement.name = "replacement log follower".into();
    let new_execution = app
        .world
        .activity_supervisor
        .replace_stopped(
            old_execution,
            &replacement,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )
        .map_err(|error| error.to_string())?;
    if let Some(current) = app
        .world
        .activities
        .iter_mut()
        .find(|activity| activity.id == activity_id)
    {
        *current = replacement;
    } else {
        return Err("Activity replacement snapshot is missing".into());
    }
    app.activity_executions.insert(activity_id, new_execution);
    app.handle_activity_stop_event(
        StopEvent {
            view_token: second_token,
            execution_id: old_execution,
            snapshot,
        },
        true,
    );
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(new_execution)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Running),
        "a retired execution generation cannot stop its replacement"
    );
    Ok(())
}

#[test]
fn task_run_route_and_run_bindings_reject_stale_stop_events() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    app.handle_task_run_stop_event(route_id.saturating_add(1), run_id, execution_id, true);
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Running)
    );
    let stale_run_id = app
        .task_runs
        .start(TaskScriptId::DevServerDemo, app.world.now_ms())
        .map_err(|error| error.to_string())?;
    app.handle_task_run_stop_event(route_id, stale_run_id, execution_id, true);
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Running)
    );
    Ok(())
}

#[test]
fn old_task_run_generation_cannot_observe_or_stop_identical_replacement() -> TestResult {
    let mut app = trusted_app();
    let (run_id, old_execution_id, route_id) = install_task_run(&mut app)?;
    let snapshot = app
        .task_run_owners
        .get(&run_id)
        .ok_or("TaskRun owner is missing")?
        .activity
        .clone();

    app.world.seek(4_100);
    app.advance_task_runs_to_world();
    let prompt_id = app
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("TaskRun prompt is missing")?;
    let update = app
        .task_runs
        .answer_prompt(run_id, prompt_id, "y", app.world.now_ms())
        .map_err(|error| error.to_string())?;
    app.observe_task_run_updates(&[update], false);
    app.world.seek(4_320);
    let completed_updates = app.task_runs.advance_to(app.world.now_ms());
    app.observe_task_run_updates(&completed_updates, false);
    assert!(matches!(
        app.task_runs
            .get(run_id)
            .ok_or("TaskRun is missing")?
            .state(),
        TaskRunState::Completed
    ));
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(old_execution_id)
            .map(|execution| (execution.phase(), execution.stop_phase())),
        Some((
            ActivityExecutionPhase::Stopped,
            ActivityStopPhase::NotRequested
        ))
    );

    let replacement_id = app
        .world
        .activity_supervisor
        .replace_stopped(
            old_execution_id,
            &snapshot,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )
        .map_err(|error| error.to_string())?;
    assert_ne!(replacement_id, old_execution_id);
    assert!(!app.task_run_owner_is_live(route_id, run_id));
    assert!(!app.task_run_owner_binding_is_current(route_id, run_id));
    app.observe_task_run_updates(&completed_updates, false);
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(replacement_id)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Running),
        "the owner generation is stale even while the App map still holds its old ID"
    );
    app.activity_executions.insert(snapshot.id, replacement_id);
    assert_eq!(
        app.task_run_owners
            .get(&run_id)
            .map(|owner| owner.execution_id),
        Some(old_execution_id)
    );
    assert!(!app.task_run_owner_is_live(route_id, run_id));
    assert!(!app.task_run_owner_binding_is_current(route_id, run_id));

    app.observe_task_run_updates(&completed_updates, false);
    assert_eq!(
        app.world
            .activity_supervisor
            .current_execution_id(snapshot.id),
        Some(replacement_id)
    );
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(replacement_id)
            .map(|execution| (execution.phase(), execution.milestones().len())),
        Some((ActivityExecutionPhase::Running, 0)),
        "a replayed old terminal observation cannot stop an identical replacement"
    );

    app.handle_task_run_stop_event(route_id, run_id, replacement_id, true);
    let view_token = update_detail_route(&mut app, snapshot.id)?;
    app.handle_activity_stop_event(
        StopEvent {
            view_token,
            execution_id: replacement_id,
            snapshot,
        },
        true,
    );
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(replacement_id)
            .map(|execution| (execution.phase(), execution.milestones().len())),
        Some((ActivityExecutionPhase::Running, 0)),
        "Activity Stop cannot link a replacement generation to an old TaskRun"
    );
    Ok(())
}

#[test]
fn natural_completion_during_stop_and_offroute_completion_do_not_record_signals() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, _) = install_task_run(&mut app)?;
    let activity_id = execution_id.activity_id();
    let view_token = update_detail_route(&mut app, activity_id)?;
    app.world.seek(4_100);
    app.advance_task_runs_to_world();
    let prompt_id = app
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("TaskRun prompt is missing")?;
    let waiting_projection = app
        .activity_projections()
        .get(&activity_id)
        .cloned()
        .ok_or("waiting Activity projection is missing")?;
    assert_eq!(waiting_projection.state_label, "waiting");
    assert_eq!(
        activity::strip_status(&waiting_projection).as_deref(),
        Some("waiting")
    );
    assert!(waiting_projection.stop_enabled);
    let answer_time = app.world.now_ms();
    let update = app
        .task_runs
        .answer_prompt(run_id, prompt_id, "y", answer_time)
        .map_err(|error| error.to_string())?;
    app.observe_task_run_updates(&[update], false);
    app.world.seek(4_320);
    app.handle_activity_stop_event(
        StopEvent {
            view_token,
            execution_id,
            snapshot: app
                .task_run_owners
                .get(&run_id)
                .ok_or("TaskRun owner is missing")?
                .activity
                .clone(),
        },
        true,
    );
    let run = app.task_runs.get(run_id).ok_or("TaskRun is missing")?;
    assert!(matches!(run.state(), TaskRunState::Completed));
    assert_eq!(run.duration_ms(), 200);
    let execution = app
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor execution is missing")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
    assert!(
        execution.milestones().is_empty(),
        "the completion race must not commit a TERM or reap grace"
    );

    let completed_projection = app
        .activity_projections()
        .get(&activity_id)
        .cloned()
        .ok_or("completed Activity projection is missing")?;
    assert_eq!(completed_projection.state_label, "completed");
    assert_eq!(
        activity::strip_status(&completed_projection).as_deref(),
        Some("completed")
    );
    assert!(!completed_projection.stop_enabled);

    let mut terminal_harness = harness(app);
    terminal_harness.app_mut().route = Route::TaskRun;
    let _ = terminal_harness.tick();
    assert!(terminal_harness.text().contains("Execution: completed"));
    terminal_harness.app_mut().route = Route::Activity;
    let _ = terminal_harness.tick();
    assert!(terminal_harness.text().contains("completed"));
    let stop_id = activity_stop_id(activity_id, view_token, execution_id);
    assert!(terminal_harness.area_of(stop_id).is_some());
    assert!(
        !terminal_harness.tab_to(stop_id),
        "a naturally completed Activity Stop button is disabled in draw and focus traversal"
    );
    let _ = terminal_harness.click_id(stop_id);
    assert_eq!(
        terminal_harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| (
                execution.phase(),
                execution.stop_phase(),
                execution.milestones().len()
            )),
        Some((
            ActivityExecutionPhase::Stopped,
            ActivityStopPhase::NotRequested,
            0
        ))
    );

    let mut offroute = trusted_app();
    let (offroute_run, offroute_execution, _) = install_task_run(&mut offroute)?;
    offroute.route = Route::Home;
    offroute.world.seek(4_100);
    offroute.advance_task_runs_to_world();
    let prompt_id = offroute
        .task_runs
        .get(offroute_run)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("off-route TaskRun prompt is missing")?;
    let update = offroute
        .task_runs
        .answer_prompt(offroute_run, prompt_id, "y", offroute.world.now_ms())
        .map_err(|error| error.to_string())?;
    offroute.observe_task_run_updates(&[update], false);
    offroute.world.seek(4_320);
    offroute.advance_task_runs_to_world();
    assert!(matches!(
        offroute
            .task_runs
            .get(offroute_run)
            .ok_or("off-route TaskRun is missing")?
            .state(),
        TaskRunState::Completed
    ));
    let execution = offroute
        .world
        .activity_supervisor
        .execution(offroute_execution)
        .ok_or("off-route supervisor execution is missing")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
    assert!(execution.milestones().is_empty());
    assert_eq!(
        offroute
            .activity_projections()
            .get(&offroute_execution.activity_id())
            .map(|projection| projection.state_label.as_str()),
        Some("completed")
    );
    assert_eq!(offroute.world.effect_revision, 0);
    Ok(())
}

#[test]
fn focused_prompt_ctrl_c_completes_without_supervisor_grace() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, _) = install_task_run(&mut app)?;
    app.world.seek(4_100);
    app.advance_task_runs_to_world();
    let mut harness = harness(app);
    let _ = harness.tick();
    let _ = harness.ctrl('c');
    assert!(matches!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .ok_or("TaskRun is missing")?
            .state(),
        TaskRunState::Cancelled
    ));
    let projection = harness
        .app()
        .activity_projections()
        .get(&execution_id.activity_id())
        .cloned()
        .ok_or("cancelled Activity projection is missing")?;
    assert_eq!(projection.state_label, "cancelled");
    assert_eq!(
        activity::strip_status(&projection).as_deref(),
        Some("cancelled")
    );
    assert!(!projection.stop_enabled);
    assert!(harness.text().contains("Execution: cancelled"));
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor execution is missing")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
    assert!(execution.milestones().is_empty());
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn time_exhausted_projection_reaches_activity_strip_detail_and_task_run() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    app.world
        .seek(u64::try_from(i64::MAX).map_err(|error| error.to_string())?);
    app.advance_task_runs_to_world();
    let prompt_id = app
        .task_runs
        .get(run_id)
        .and_then(TaskRun::active_prompt)
        .map(|prompt| prompt.id())
        .ok_or("TaskRun prompt is missing before time exhaustion")?;
    let now_ms = app.world.now_ms();
    let update = app
        .task_runs
        .answer_prompt(run_id, prompt_id, "n", now_ms)
        .map_err(|error| error.to_string())?;
    app.observe_task_run_updates(&[update], false);
    assert!(matches!(
        app.task_runs
            .get(run_id)
            .ok_or("TaskRun is missing")?
            .state(),
        TaskRunState::TimeExhausted
    ));
    let projection = app
        .activity_projections()
        .get(&execution_id.activity_id())
        .cloned()
        .ok_or("time-exhausted Activity projection is missing")?;
    assert_eq!(projection.state_label, "time exhausted");
    assert_eq!(
        activity::strip_status(&projection).as_deref(),
        Some("time exhausted")
    );
    assert!(projection.stop_enabled);

    let mut harness = harness(app);
    let _ = harness.tick();
    let task_run_stop =
        task_run_stop_id(route_id, run_id, execution_id.activity_id(), execution_id);
    assert!(harness.text().contains("Execution: time exhausted"));
    assert!(harness.area_of(task_run_stop).is_some());
    assert!(
        harness.tab_to(task_run_stop),
        "TimeExhausted remains stoppable in TaskRun"
    );

    let view_token = {
        let app = harness.app_mut();
        update_detail_route(app, execution_id.activity_id())?
    };
    let _ = harness.tick();
    assert!(harness.text().contains("time exhausted"));
    let activity_stop = activity_stop_id(execution_id.activity_id(), view_token, execution_id);
    assert!(harness.area_of(activity_stop).is_some());
    assert!(
        harness.tab_to(activity_stop),
        "TimeExhausted remains stoppable in Activity detail"
    );
    Ok(())
}

#[test]
fn task_run_registration_failure_cancels_the_unpublished_run() -> TestResult {
    let mut app = trusted_app();
    let activity_id =
        dispatch::next_activity_id(&app.world).ok_or_else(|| "Activity ID exhausted".to_owned())?;
    let existing = Activity::new(
        activity_id,
        "existing registration",
        &app.world.cwd,
        ActivityState::Running,
        app.world.now_ms(),
    );
    let existing_id = app
        .world
        .activity_supervisor
        .register(
            &existing,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )
        .map_err(|error| error.to_string())?;
    let original_activity_count = app.world.activities.len();
    let mut harness = harness(app);
    enter_dev(&mut harness);
    assert!(matches!(harness.app().route, Route::Home));
    assert_eq!(
        harness.app().world.activities.len(),
        original_activity_count
    );
    assert!(harness.app().activity_executions.is_empty());
    assert!(harness.app().task_run_owners.is_empty());
    assert!(harness.app().task_run_screen.is_none());
    assert_eq!(harness.app().task_runs.len(), 1);
    assert!(
        format!("{:?}", harness.app().task_runs).contains("Cancelled"),
        "registration compensation retains a cancelled record and may consume IDs"
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(existing_id)
            .map(|execution| execution.snapshot().name.as_str()),
        Some("existing registration")
    );
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn generic_dispatch_registration_is_bound_to_the_exact_published_snapshot() -> TestResult {
    let mut app = trusted_app();
    let Some(task) = app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    else {
        return Err("fixture dev task is missing".into());
    };
    task.command = "vite --port 5200".into();
    let action = catalogue::catalogue(&app.world)
        .into_iter()
        .find(|action| action.id == "task:dev")
        .ok_or("generic dev action is missing")?;
    let (activity_id, message, execution_id) = match dispatch::invoke(&mut app.world, &action) {
        dispatch::Destination::ActivityNotice {
            id,
            message,
            execution_id,
        } => (id, message, execution_id),
        _ => return Err("changed generic task must keep its Activity notice route".into()),
    };
    assert!(message.starts_with("Activity started:"));
    let snapshot = app
        .world
        .activities
        .iter()
        .find(|activity| activity.id == activity_id)
        .cloned()
        .ok_or("generic Activity was not published")?;
    app.bind_activity_execution(&snapshot, execution_id);
    assert_eq!(
        app.activity_executions.get(&snapshot.id).copied(),
        Some(execution_id)
    );
    assert_eq!(
        app.world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| execution.snapshot()),
        Some(&snapshot)
    );
    assert!(
        app.activity_projections()
            .get(&snapshot.id)
            .is_some_and(|projection| projection.stop_enabled)
    );
    assert_eq!(app.world.effect_revision, 0);
    Ok(())
}

#[test]
fn failed_activity_is_registered_as_a_terminal_record_without_stop_grace() -> TestResult {
    let app = App::for_scenario_without_history(Scenario::LaunchFailure, Motion::Paused, 4_000)
        .with_source_theme(Theme::junie());
    let mut harness = harness(app);
    let _ = harness.key(KeyCode::Esc);
    let _ = harness.type_str("mise run test");
    let _ = harness.key(KeyCode::Enter);

    let app = harness.app();
    let snapshot = app
        .world
        .activities
        .last()
        .ok_or("failed Activity was not published")?;
    assert_eq!(snapshot.state, ActivityState::Failed);
    let execution_id = app
        .activity_executions
        .get(&snapshot.id)
        .copied()
        .ok_or("failed Activity supervisor identity was not bound")?;
    let execution = app
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("failed Activity execution was not registered")?;
    assert_eq!(execution.snapshot(), snapshot);
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
    assert!(execution.milestones().is_empty());
    let projection = app
        .activity_projections()
        .get(&snapshot.id)
        .cloned()
        .ok_or("failed Activity projection is missing")?;
    assert_eq!(projection.state_label, "failed");
    assert!(!projection.stop_enabled);
    assert_eq!(app.world.effect_revision, 0);
    Ok(())
}

#[test]
fn task_run_stop_button_also_works_without_an_open_prompt() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    let activity_id = execution_id.activity_id();
    let mut harness = harness(app);
    let stop_id = task_run_stop_id(route_id, run_id, activity_id, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "TaskRun Stop is visible"
    );
    assert!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .and_then(TaskRun::active_prompt)
            .is_none(),
        "the initial Stop path must not require a prompt"
    );
    let _ = harness.click_id(stop_id);
    assert!(matches!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .ok_or("TaskRun is missing")?
            .state(),
        TaskRunState::Cancelled
    ));
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Cancelling)
    );
    Ok(())
}

#[test]
fn task_run_stop_button_keyboard_space_without_prompt_preserves_resistant_grace() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    let activity_id = execution_id.activity_id();
    let run = app.task_runs.get(run_id).ok_or("TaskRun is missing")?;
    assert!(run.active_prompt().is_none());
    let active_duration = run.duration_ms();
    let output_before: Vec<_> = run
        .output()
        .iter()
        .map(|line| line.text().to_owned())
        .collect();
    assert!(!output_before.is_empty(), "the fixture has authored output to freeze");

    let mut harness = harness(app);
    let stop_id = task_run_stop_id(route_id, run_id, activity_id, execution_id);
    assert!(harness.area_of(stop_id).is_some(), "TaskRun Stop is visible");
    assert!(
        harness.tab_to(stop_id),
        "real Tab traversal reaches TaskRun Stop without a prompt"
    );
    assert_eq!(harness.focus(), Some(stop_id));
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(harness.app().world.now_ms(), 4_000);

    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after keyboard Stop")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert!(run.active_prompt().is_none());
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before,
        "keyboard Stop freezes authored output immediately"
    );
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("exact supervisor generation disappeared")?;
    assert_eq!(execution.id(), execution_id);
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(execution_id)
    );
    assert_eq!(
        harness.app().activity_executions.get(&activity_id),
        Some(&execution_id)
    );
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.stop_requested_ms(), Some(4_000));
    assert_eq!(execution.kill_deadline_ms(), Some(4_800));
    assert_eq!(execution.reap_deadline_ms(), Some(4_960));
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_000 }]
    );
    assert!(!harness.app().quit, "keyboard Stop does not quit the app");
    assert_eq!(harness.app().world.effect_revision, 0);

    harness.app_mut().world.seek(4_720);
    assert_eq!(harness.app().world.now_ms(), 4_720);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared before KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_000 }]
    );

    harness.app_mut().world.seek(4_800);
    assert_eq!(harness.app().world.now_ms(), 4_800);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared at KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::KillSent { at_ms: 4_800 },
        ]
    );

    harness.app_mut().world.seek(4_880);
    assert_eq!(harness.app().world.now_ms(), 4_880);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared before reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::KillSent { at_ms: 4_800 },
        ]
    );

    harness.app_mut().world.seek(4_960);
    assert_eq!(harness.app().world.now_ms(), 4_960);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert!(!execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::KillSent { at_ms: 4_800 },
            ActivityStopMilestone::Reaped { at_ms: 4_960 },
        ]
    );
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after reap")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before,
        "reaping does not resume authored output"
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn task_run_text_input_consumes_space_but_enter_stops_waiting_prompt() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    let activity_id = execution_id.activity_id();
    app.world.seek(4_100);
    assert_eq!(app.world.now_ms(), 4_160);
    app.advance_task_runs_to_world();
    let run = app.task_runs.get(run_id).ok_or("TaskRun is missing")?;
    assert_eq!(run.state(), TaskRunState::Waiting);
    let prompt_id = run
        .active_prompt()
        .ok_or("the fixture is missing its waiting prompt")?
        .id();
    let active_duration = run.duration_ms();
    let output_before: Vec<_> = run
        .output()
        .iter()
        .map(|line| line.text().to_owned())
        .collect();

    let mut harness = harness(app);
    let _ = harness.tick();
    assert!(harness.text().contains("Execution: waiting"));
    let stop_id = task_run_stop_id(route_id, run_id, activity_id, execution_id);
    assert!(harness.area_of(stop_id).is_some(), "TaskRun Stop is visible");
    assert!(
        harness.tab_to(stop_id),
        "real Tab traversal moves from the prompt to TaskRun Stop"
    );
    assert_eq!(harness.focus(), Some(stop_id));
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared before Space")?;
    assert_eq!(run.state(), TaskRunState::Waiting);
    assert_eq!(run.active_prompt().map(|prompt| prompt.id()), Some(prompt_id));
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(harness.app().world.now_ms(), 4_160);
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after text input")?;
    assert_eq!(run.state(), TaskRunState::Waiting);
    assert_eq!(
        run.active_prompt().map(|prompt| prompt.id()),
        Some(prompt_id),
        "Space leaves the same prompt waiting"
    );
    assert_eq!(run.duration_ms(), active_duration);
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("exact supervisor generation disappeared after Space")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Running);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
    assert_eq!(execution.stop_requested_ms(), None);
    assert!(execution.milestones().is_empty());
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    assert_eq!(harness.focus(), Some(stop_id));
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().world.now_ms(), 4_160);

    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after keyboard Stop")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert!(run.active_prompt().is_none());
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before,
        "keyboard Stop settles the prompt and freezes authored output immediately"
    );
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("exact supervisor generation disappeared")?;
    assert_eq!(execution.id(), execution_id);
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(execution_id)
    );
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.stop_requested_ms(), Some(4_160));
    assert_eq!(execution.kill_deadline_ms(), Some(4_960));
    assert_eq!(execution.reap_deadline_ms(), Some(5_120));
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_160 }]
    );
    assert!(!harness.app().quit, "keyboard Stop does not quit the app");
    assert_eq!(harness.app().world.effect_revision, 0);

    harness.app_mut().world.seek(4_880);
    assert_eq!(harness.app().world.now_ms(), 4_880);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared before KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_160 }]
    );

    harness.app_mut().world.seek(4_960);
    assert_eq!(harness.app().world.now_ms(), 4_960);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared at KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_160 },
            ActivityStopMilestone::KillSent { at_ms: 4_960 },
        ]
    );

    harness.app_mut().world.seek(5_040);
    assert_eq!(harness.app().world.now_ms(), 5_040);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared before reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_160 },
            ActivityStopMilestone::KillSent { at_ms: 4_960 },
        ]
    );

    harness.app_mut().world.seek(5_120);
    assert_eq!(harness.app().world.now_ms(), 5_120);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("supervisor generation disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert!(!execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_160 },
            ActivityStopMilestone::KillSent { at_ms: 4_960 },
            ActivityStopMilestone::Reaped { at_ms: 5_120 },
        ]
    );
    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after reap")?;
    assert!(matches!(run.state(), TaskRunState::Cancelled));
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before,
        "reaping does not resume authored output"
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn covered_task_run_stop_button_cannot_activate_from_keyboard() -> TestResult {
    let mut app = trusted_app();
    let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
    let activity_id = execution_id.activity_id();
    let mut harness = harness(app);
    let stop_id = task_run_stop_id(route_id, run_id, activity_id, execution_id);
    assert!(harness.tab_to(stop_id), "Tab traversal reaches visible TaskRun Stop");
    assert_eq!(harness.focus(), Some(stop_id));

    let _ = harness.key(KeyCode::F(1));
    assert!(harness.app().overlay.is_some(), "the Help dialog covers TaskRun");
    assert!(harness.text().contains("Key reference"));
    assert!(
        harness.area_of(stop_id).is_none(),
        "the covered TaskRun Stop has no published geometry"
    );
    assert!(
        !harness.ring().contains(stop_id),
        "the covered TaskRun Stop is absent from the published focus ring"
    );
    assert_ne!(harness.focus(), Some(stop_id));
    assert!(
        !harness.tab_to(stop_id),
        "Tab traversal cannot focus TaskRun Stop through the Help dialog"
    );
    let _ = harness.key(KeyCode::Enter);

    assert!(matches!(
        harness
            .app()
            .task_runs
            .get(run_id)
            .ok_or("TaskRun disappeared while covered")?
            .state(),
        TaskRunState::Running
    ));
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(execution_id)
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| {
                (execution.phase(), execution.stop_phase(), execution.milestones().len())
            }),
        Some((ActivityExecutionPhase::Running, ActivityStopPhase::NotRequested, 0))
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn task_run_route_invalidation_blocks_stale_keyboard_stop_generation() -> TestResult {
    let mut app = trusted_app();
    let (run_id, old_execution_id, route_id) = install_task_run(&mut app)?;
    let activity_id = old_execution_id.activity_id();
    let snapshot = app
        .task_run_owners
        .get(&run_id)
        .ok_or("TaskRun owner is missing")?
        .activity
        .clone();
    let run = app.task_runs.get(run_id).ok_or("TaskRun is missing")?;
    let active_duration = run.duration_ms();
    let output_before: Vec<_> = run
        .output()
        .iter()
        .map(|line| line.text().to_owned())
        .collect();

    let mut harness = harness(app);
    let stale_stop_id = task_run_stop_id(route_id, run_id, activity_id, old_execution_id);
    assert!(
        harness.tab_to(stale_stop_id),
        "real Tab traversal focuses the old TaskRun generation's Stop"
    );
    assert_eq!(harness.focus(), Some(stale_stop_id));

    let replacement_id = {
        let app = harness.app_mut();
        app.world
            .activity_supervisor
            .observe_terminal_exit(old_execution_id, &snapshot)
            .map_err(|error| error.to_string())?;
        let replacement_id = app
            .world
            .activity_supervisor
            .replace_stopped(
                old_execution_id,
                &snapshot,
                ActivityExecutionStart::Running,
                ActivityStopProfile::Resistant,
            )
            .map_err(|error| error.to_string())?;
        app.activity_executions.insert(activity_id, replacement_id);
        replacement_id
    };
    assert_ne!(replacement_id, old_execution_id);
    harness.draw();
    assert!(matches!(harness.app().route, Route::Home));
    assert!(harness.app().task_run_screen.is_none());
    assert_eq!(harness.focus(), Some(home::QUERY));
    let _ = harness.key(KeyCode::Char(' '));

    let run = harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun disappeared after route invalidation")?;
    assert!(matches!(run.state(), TaskRunState::Running));
    assert!(run.active_prompt().is_none());
    assert_eq!(run.duration_ms(), active_duration);
    assert_eq!(
        run.output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect::<Vec<_>>(),
        output_before,
        "stale keyboard input cannot change TaskRun output"
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(replacement_id)
    );
    assert_eq!(
        harness.app().activity_executions.get(&activity_id),
        Some(&replacement_id)
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(replacement_id)
            .map(|execution| {
                (
                    execution.phase(),
                    execution.stop_phase(),
                    execution.milestones().len(),
                )
            }),
        Some((ActivityExecutionPhase::Running, ActivityStopPhase::NotRequested, 0))
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(old_execution_id)
            .map(|execution| {
                (
                    execution.phase(),
                    execution.stop_phase(),
                    execution.milestones().len(),
                )
            }),
        Some((ActivityExecutionPhase::Stopped, ActivityStopPhase::NotRequested, 0))
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn parity_activity_stop_app_adopts_exact_setup_handles_without_task_runs() -> TestResult {
    let app = parity_activity_stop_app();
    assert!(matches!(app.route, Route::Activity));
    assert_eq!(app.world.effect_revision, 0);
    assert!(app.task_runs.is_empty());
    assert_eq!(app.activity_executions.len(), 4);
    assert_eq!(app.world.scenario_activity_execution_ids().len(), 4);

    for (activity, execution_id) in app
        .world
        .activities
        .iter()
        .zip(app.world.scenario_activity_execution_ids())
    {
        assert_eq!(execution_id.activity_id(), activity.id);
        assert_eq!(
            app.activity_executions.get(&activity.id),
            Some(execution_id)
        );
        assert_eq!(
            app.world
                .activity_supervisor
                .execution(*execution_id)
                .map(|execution| execution.snapshot()),
            Some(activity)
        );
    }
    Ok(())
}

#[test]
fn parity_activity_stop_responsive_button_reaps_at_240_ms_and_freezes_output() -> TestResult {
    let app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 1)?;
    let view_token = app
        .activities
        .current_view_token()
        .ok_or("ordinary Activity view token is missing")?;
    let mut harness = harness(app);
    let stop_id = activity_stop_id(1, view_token, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "rendered Stop button is visible"
    );
    let original_lines = snapshot.lines.clone();
    let _ = harness.click_id(stop_id);

    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary supervisor execution is missing")?;
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.stop_requested_ms(), Some(4_000));
    assert_eq!(execution.reap_deadline_ms(), Some(4_240));
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_000 }]
    );
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(harness.app().task_runs.is_empty());

    harness.app_mut().world.seek(4_160);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary execution disappeared during grace")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&1)
            .map(|projection| projection.lines.clone()),
        Some(original_lines.clone())
    );

    harness.app_mut().world.seek(4_240);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary execution disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::Reaped { at_ms: 4_240 },
        ]
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&1)
            .map(|projection| projection.lines.clone()),
        Some(original_lines)
    );
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn activity_detail_stop_button_keyboard_space_reaps_responsive_execution() -> TestResult {
    let app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 1)?;
    let activity_id = snapshot.id;
    let view_token = app
        .activities
        .current_view_token()
        .ok_or("ordinary Activity view token is missing")?;
    let original_lines = snapshot.lines.clone();
    let mut harness = harness(app);
    let stop_id = activity_stop_id(activity_id, view_token, execution_id);
    assert!(harness.area_of(stop_id).is_some(), "detail Stop is visible");
    assert!(
        harness.tab_to(stop_id),
        "real Tab traversal reaches Activity detail Stop"
    );
    assert_eq!(harness.focus(), Some(stop_id));
    let _ = harness.key(KeyCode::Char(' '));

    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("exact Activity supervisor generation disappeared")?;
    assert_eq!(execution.id(), execution_id);
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(execution_id)
    );
    assert_eq!(
        harness.app().activity_executions.get(&activity_id),
        Some(&execution_id)
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.stop_requested_ms(), Some(4_000));
    assert_eq!(execution.kill_deadline_ms(), None);
    assert_eq!(execution.reap_deadline_ms(), Some(4_240));
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::TermSent { at_ms: 4_000 }]
    );
    assert!(harness.app().task_runs.is_empty());
    assert!(!harness.app().quit, "keyboard Stop does not quit the app");
    assert_eq!(harness.app().world.effect_revision, 0);

    harness.app_mut().world.seek(4_160);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary execution disappeared during grace")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&activity_id)
            .map(|projection| projection.lines.clone()),
        Some(original_lines.clone())
    );

    harness.app_mut().world.seek(4_240);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary execution disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::Reaped { at_ms: 4_240 },
        ]
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&activity_id)
            .map(|projection| projection.lines.clone()),
        Some(original_lines)
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn activity_detail_retired_generation_ignores_keyboard_activation() -> TestResult {
    let app = parity_activity_stop_app();
    let (snapshot, old_execution_id) = scenario_activity_execution(&app, 1)?;
    let activity_id = snapshot.id;
    let view_token = app
        .activities
        .current_view_token()
        .ok_or("ordinary Activity view token is missing")?;
    let mut harness = harness(app);
    let stale_stop_id = activity_stop_id(activity_id, view_token, old_execution_id);
    assert!(
        harness.tab_to(stale_stop_id),
        "real Tab traversal focuses the old Activity generation's Stop"
    );
    assert_eq!(harness.focus(), Some(stale_stop_id));

    let replacement_id = {
        let app = harness.app_mut();
        app.world
            .activity_supervisor
            .observe_terminal_exit(old_execution_id, &snapshot)
            .map_err(|error| error.to_string())?;
        let replacement_id = app
            .world
            .activity_supervisor
            .replace_stopped(
                old_execution_id,
                &snapshot,
                ActivityExecutionStart::Running,
                ActivityStopProfile::Responsive,
            )
            .map_err(|error| error.to_string())?;
        app.activity_executions.insert(activity_id, replacement_id);
        replacement_id
    };
    assert_ne!(replacement_id, old_execution_id);
    let replacement_stop_id = activity_stop_id(activity_id, view_token, replacement_id);
    assert_ne!(stale_stop_id, replacement_stop_id);
    harness.draw();
    assert!(harness.area_of(replacement_stop_id).is_some());
    assert!(
        harness.area_of(stale_stop_id).is_none(),
        "the retired generation has no published Stop geometry"
    );
    assert!(
        !harness.ring().contains(stale_stop_id),
        "the retired generation is absent from the published focus ring"
    );
    assert_ne!(
        harness.focus(),
        Some(stale_stop_id),
        "focus no longer targets the retired generation"
    );
    assert_ne!(
        harness.focus(),
        Some(replacement_stop_id),
        "retirement does not transfer stale focus to replacement Stop"
    );
    let _ = harness.key(KeyCode::Enter);

    assert!(matches!(harness.app().route, Route::Activity));
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .current_execution_id(activity_id),
        Some(replacement_id)
    );
    assert_eq!(
        harness.app().activity_executions.get(&activity_id),
        Some(&replacement_id)
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(replacement_id)
            .map(|execution| {
                (
                    execution.phase(),
                    execution.stop_phase(),
                    execution.milestones().len(),
                )
            }),
        Some((ActivityExecutionPhase::Running, ActivityStopPhase::NotRequested, 0))
    );
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(old_execution_id)
            .map(|execution| {
                (
                    execution.phase(),
                    execution.stop_phase(),
                    execution.milestones().len(),
                )
            }),
        Some((ActivityExecutionPhase::Stopped, ActivityStopPhase::NotRequested, 0))
    );
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&activity_id)
            .map(|projection| projection.lines.clone()),
        Some(snapshot.lines)
    );
    assert!(!harness.app().quit);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn parity_activity_stop_queued_button_cancels_without_grace() -> TestResult {
    let mut app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 2)?;
    let view_token = update_detail_route(&mut app, snapshot.id)?;
    let mut harness = harness(app);
    let stop_id = activity_stop_id(snapshot.id, view_token, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "queued Stop button is visible"
    );
    assert!(
        harness
            .app()
            .activity_projections()
            .get(&snapshot.id)
            .is_some_and(|projection| projection.stop_enabled),
        "queued execution can be cancelled before start"
    );
    assert_eq!(
        harness
            .app()
            .activity_projections()
            .get(&snapshot.id)
            .map(|projection| projection.state_label.as_str()),
        Some("queued")
    );
    let _ = harness.click_id(stop_id);

    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("queued supervisor execution is missing")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(
        execution.stop_phase(),
        ActivityStopPhase::CancelledBeforeStart
    );
    assert_eq!(execution.stop_requested_ms(), Some(4_000));
    assert_eq!(execution.kill_deadline_ms(), None);
    assert_eq!(execution.reap_deadline_ms(), None);
    assert_eq!(
        execution.milestones(),
        &[ActivityStopMilestone::CancelledBeforeStart { at_ms: 4_000 }]
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(harness.app().task_runs.is_empty());
    Ok(())
}

#[test]
fn parity_activity_stop_detached_owner_reaps_at_240_ms() -> TestResult {
    let mut app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 3)?;
    let view_token = update_detail_route(&mut app, snapshot.id)?;
    let initial = app
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("detached supervisor execution is missing")?;
    assert_eq!(initial.phase(), ActivityExecutionPhase::Detached);
    assert!(initial.is_owned());
    assert!(
        app.activity_projections()
            .get(&snapshot.id)
            .is_some_and(|projection| projection.stop_enabled)
    );
    let original_lines = snapshot.lines.clone();
    let mut harness = harness(app);
    let stop_id = activity_stop_id(snapshot.id, view_token, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "detached Stop button is visible"
    );
    let _ = harness.click_id(stop_id);

    harness.app_mut().world.seek(4_160);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("detached execution disappeared during grace")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert!(execution.is_owned());
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.snapshot().lines, original_lines);

    harness.app_mut().world.seek(4_240);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("detached execution disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert!(!execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::Reaped { at_ms: 4_240 },
        ]
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}

#[test]
fn parity_activity_stop_resistant_button_sends_kill_at_800_and_reaps_at_960_ms() -> TestResult {
    let mut app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 4)?;
    let view_token = update_detail_route(&mut app, snapshot.id)?;
    let original_lines = snapshot.lines.clone();
    let mut harness = harness(app);
    let stop_id = activity_stop_id(snapshot.id, view_token, execution_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "resistant Stop button is visible"
    );
    let _ = harness.click_id(stop_id);

    harness.app_mut().world.seek(4_720);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("resistant execution disappeared before KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
    assert_eq!(execution.kill_deadline_ms(), Some(4_800));
    assert_eq!(execution.reap_deadline_ms(), Some(4_960));
    assert_eq!(execution.snapshot().lines, original_lines);

    harness.app_mut().world.seek(4_800);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("resistant execution disappeared at KILL")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::KillSent { at_ms: 4_800 },
        ]
    );
    assert_eq!(execution.snapshot().lines, original_lines);

    harness.app_mut().world.seek(4_880);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("resistant execution disappeared before reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::KillSent);
    assert!(execution.is_owned());
    assert_eq!(execution.snapshot().lines, original_lines);

    harness.app_mut().world.seek(4_960);
    harness.draw();
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("resistant execution disappeared after reap")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
    assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
    assert!(!execution.is_owned());
    assert_eq!(
        execution.milestones(),
        &[
            ActivityStopMilestone::TermSent { at_ms: 4_000 },
            ActivityStopMilestone::KillSent { at_ms: 4_800 },
            ActivityStopMilestone::Reaped { at_ms: 4_960 },
        ]
    );
    assert_eq!(execution.snapshot(), &snapshot);
    assert_eq!(execution.snapshot().lines, original_lines);
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(harness.app().task_runs.is_empty());
    Ok(())
}

#[test]
fn parity_activity_stop_rejects_stale_view_and_accepts_current_button() -> TestResult {
    let app = parity_activity_stop_app();
    let (snapshot, execution_id) = scenario_activity_execution(&app, 1)?;
    let mut harness = harness(app);
    let stale_view_token = harness
        .app()
        .activities
        .current_view_token()
        .ok_or("ordinary Activity view token is missing")?;
    let stale_stop_id = activity_stop_id(snapshot.id, stale_view_token, execution_id);
    assert!(
        harness.area_of(stale_stop_id).is_some(),
        "initial Stop button is rendered before reopening the same Activity"
    );
    let current_view_token = harness
        .app_mut()
        .activities
        .show_with_token(snapshot.id)
        .ok_or("ordinary Activity view token exhausted")?;
    assert_ne!(current_view_token, stale_view_token);
    harness.app_mut().handle_activity_stop_event(
        StopEvent {
            view_token: stale_view_token,
            execution_id,
            snapshot: snapshot.clone(),
        },
        true,
    );
    let execution = harness
        .app()
        .world
        .activity_supervisor
        .execution(execution_id)
        .ok_or("ordinary supervisor execution is missing")?;
    assert_eq!(execution.phase(), ActivityExecutionPhase::Running);
    assert!(execution.milestones().is_empty());

    let _ = harness.tick();
    let stop_id = activity_stop_id(snapshot.id, current_view_token, execution_id);
    assert_ne!(stop_id, stale_stop_id);
    assert!(
        harness.area_of(stop_id).is_some(),
        "current Stop button is visible"
    );
    let _ = harness.click_id(stop_id);
    assert_eq!(
        harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .map(|execution| execution.phase()),
        Some(ActivityExecutionPhase::Cancelling)
    );
    assert_eq!(harness.app().world.effect_revision, 0);
    Ok(())
}
