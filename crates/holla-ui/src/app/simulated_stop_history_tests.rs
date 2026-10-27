#![cfg(test)]

use super::{App, Motion, ReturnLocation, Route, STRIP, TaskRunOwner, home};
use crate::{
    dispatch,
    domain::mise::Trust,
    scenario::Scenario,
    screens::{activity, task_run::TaskRunScreen},
    sim::{
        activity_supervisor::{
            ActivityExecutionId, ActivityExecutionStart, ActivityExecutionPhase,
            ActivityStopProfile, ActivityStopPhase,
        },
        catalogue,
        task_run::{TaskRunId, TaskScriptId, TaskRunState},
    },
};
use termrock::{FeedbackClock, Id, ItemKey, Part, PartRef, SimulationMoment, Theme};
use termrock_test_support::Harness;

type TestResult = Result<(), String>;

fn parity_activity_app() -> App {
    App::for_scenario_without_history(Scenario::ParityActivityStop, Motion::Paused, 4_000)
        .with_source_theme(Theme::junie())
}

fn strip_row(activity_id: u32) -> PartRef {
    PartRef::item(Part::ROW, ItemKey::num(u64::from(activity_id)))
}

fn activity_stop_id(
    activity_id: u32,
    view_token: u64,
    execution_id: ActivityExecutionId,
) -> Id {
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
    let mut activity = dispatch::task_run_activity(&app.world, &action, activity_id);
    activity.lines.push("authored Activity output".to_owned());
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

#[test]
fn completed_stop_preserves_multi_activity_strip_targets_at_three_sizes() -> TestResult {
    for (width, height) in [(80, 24), (120, 40), (72, 20)] {
        let mut app = parity_activity_app();
        let snapshot = app
            .world
            .activities
            .iter()
            .find(|activity| activity.id == 1)
            .cloned()
            .ok_or("responsive Activity fixture is missing")?;
        let execution_id = app
            .activity_executions
            .get(&snapshot.id)
            .copied()
            .ok_or("responsive execution fixture is missing")?;
        app.world
            .activity_supervisor
            .admit_stop(execution_id, &snapshot, 4_000)
            .map_err(|error| error.to_string())?
            .commit();
        app.world.seek(4_240);

        let mut harness = Harness::new_with_feedback_clock(
            app,
            Theme::junie(),
            width,
            height,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(4_240),
            },
        );
        let projection = harness
            .app()
            .activity_projections()
            .get(&snapshot.id)
            .cloned()
            .ok_or("completed Activity projection is missing")?;
        assert_eq!(
            activity::strip_status(&projection).as_deref(),
            Some("stopped"),
            "completed strip status at {width}x{height}"
        );
        let frame = harness.text();
        let strip_text = frame
            .lines()
            .nth(1)
            .ok_or("activity strip row is missing")?;
        assert!(strip_text.contains("stopped"));
        assert!(!strip_text.contains("SIMULATED"));

        let mut previous_right = 0;
        for activity_id in [1, 2, 3] {
            let area = harness
                .area_of_part(STRIP, strip_row(activity_id))
                .ok_or_else(|| {
                    format!(
                        "Activity {activity_id} strip target is missing at {width}x{height}"
                    )
                })?;
            assert!(area.width > 0, "Activity {activity_id} has a click cell");
            assert!(area.x >= previous_right, "strip targets remain ordered");
            previous_right = area.right().saturating_add(1);
        }

        let _ = harness.click_part(STRIP, strip_row(3));
        assert_eq!(harness.app().activities.current_id(), Some(3));
        assert!(harness.app().route == Route::Activity);
    }
    Ok(())
}

#[test]
fn below_minimum_keeps_execution_state_but_hides_normal_stop_controls() -> TestResult {
    const NOTICE: &str = "Terminal too small";
    const DETAILS: &str = "Need 72×20, have 71×19";
    const PROMPT: &str = "holla❯";

    let mut activity_app = parity_activity_app();
    let snapshot = activity_app
        .world
        .activities
        .iter()
        .find(|activity| activity.id == 1)
        .cloned()
        .ok_or("responsive Activity fixture is missing")?;
    let activity_execution = activity_app
        .activity_executions
        .get(&snapshot.id)
        .copied()
        .ok_or("responsive execution fixture is missing")?;
    activity_app
        .world
        .activity_supervisor
        .admit_stop(activity_execution, &snapshot, 4_000)
        .map_err(|error| error.to_string())?
        .commit();
    activity_app.world.seek(4_240);
    activity_app.activities.show(snapshot.id);
    activity_app.route = Route::Activity;
    let view_token = activity_app
        .activities
        .current_view_token()
        .ok_or("Activity view token is missing")?;
    let activity_button = activity_stop_id(snapshot.id, view_token, activity_execution);
    let activity_harness = Harness::new_with_feedback_clock(
        activity_app,
        Theme::junie(),
        71,
        19,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4_240),
        },
    );
    let activity_frame = activity_harness.text();
    assert!(
        activity_frame.contains(NOTICE),
        "Activity minimum notice: {activity_frame}"
    );
    assert!(
        activity_frame.contains(DETAILS),
        "Activity minimum dimensions: {activity_frame}"
    );
    assert!(
        activity_frame.contains(PROMPT),
        "Activity prompt remains visible: {activity_frame}"
    );
    assert!(activity_harness.area_of(STRIP).is_none());
    assert!(activity_harness
        .area_of_part(STRIP, strip_row(snapshot.id))
        .is_none());
    assert!(activity_harness.area_of(activity_button).is_none());
    assert_ne!(activity_harness.focus(), Some(activity_button));
    let activity_execution_state = activity_harness
        .app()
        .world
        .activity_supervisor
        .execution(activity_execution)
        .ok_or("completed Activity execution is missing below minimum")?;
    assert_eq!(activity_execution_state.phase(), ActivityExecutionPhase::Stopped);
    let activity_projection = activity_harness
        .app()
        .activity_projections()
        .get(&snapshot.id)
        .cloned()
        .ok_or("completed Activity projection is missing below minimum")?;
    assert_eq!(activity_projection.execution_id, Some(activity_execution));
    assert_eq!(activity_projection.state_label, "stopped");

    let mut task_run_app = App::for_scenario_without_history(
        Scenario::MonorepoChild,
        Motion::Paused,
        4_000,
    )
    .with_source_theme(Theme::junie());
    if let Some(task) = task_run_app
        .world
        .mise
        .as_mut()
        .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
    {
        task.trust = Trust::Trusted;
    }
    let (run_id, task_execution, route_id) = install_task_run(&mut task_run_app)?;
    let task_button = task_run_stop_id(
        route_id,
        run_id,
        task_execution.activity_id(),
        task_execution,
    );
    let task_run_harness = Harness::new_with_feedback_clock(
        task_run_app,
        Theme::junie(),
        71,
        19,
        FeedbackClock::Simulation {
            initial: SimulationMoment::from_millis(4_000),
        },
    );
    let task_run_frame = task_run_harness.text();
    assert!(
        task_run_frame.contains(NOTICE),
        "TaskRun minimum notice: {task_run_frame}"
    );
    assert!(
        task_run_frame.contains(DETAILS),
        "TaskRun minimum dimensions: {task_run_frame}"
    );
    assert!(
        task_run_frame.contains(PROMPT),
        "TaskRun prompt remains visible: {task_run_frame}"
    );
    assert!(task_run_harness.area_of(STRIP).is_none());
    assert!(task_run_harness
        .area_of_part(STRIP, strip_row(task_execution.activity_id()))
        .is_none());
    assert!(task_run_harness.area_of(task_button).is_none());
    assert_ne!(task_run_harness.focus(), Some(task_button));
    let task_run = task_run_harness
        .app()
        .task_runs
        .get(run_id)
        .ok_or("TaskRun remains present below minimum")?;
    assert_eq!(task_run.state(), TaskRunState::Running);
    let task_run_projection = task_run_harness
        .app()
        .activity_projections()
        .get(&task_execution.activity_id())
        .cloned()
        .ok_or("TaskRun Activity projection is missing below minimum")?;
    assert_eq!(task_run_projection.execution_id, Some(task_execution));
    Ok(())
}

#[test]
fn task_run_stop_detail_keeps_full_history_and_retained_output_without_row_overlap() -> TestResult {
    const FULL_EXECUTION: &str = "Execution: stopped · SIMULATED stop grace 960 ms";
    const HISTORY: &str = "SIMULATED stop · TERM +0 ms · KILL +800 ms · reap +960 ms";
    const TASK_OUTPUT: &str = "Run dev · modeled simulation · no process executed";

    for (width, height) in [(80, 24), (120, 40), (72, 20)] {
        let mut app = App::for_scenario_without_history(
            Scenario::MonorepoChild,
            Motion::Paused,
            4_000,
        )
        .with_source_theme(Theme::junie());
        if let Some(task) = app
            .world
            .mise
            .as_mut()
            .and_then(|mise| mise.tasks.iter_mut().find(|task| task.id == "dev"))
        {
            task.trust = Trust::Trusted;
        }
        let (run_id, execution_id, route_id) = install_task_run(&mut app)?;
        let authored_activity_lines = app
            .task_run_owners
            .get(&run_id)
            .ok_or("TaskRun Activity owner is missing")?
            .activity
            .lines
            .clone();
        app.world.seek(4_100);
        app.advance_task_runs_to_world();
        let before_output: Vec<_> = app
            .task_runs
            .get(run_id)
            .ok_or("TaskRun fixture is missing")?
            .output()
            .iter()
            .map(|line| line.text().to_owned())
            .collect();
        assert!(before_output.iter().any(|line| line == TASK_OUTPUT));
        let mut expected_activity_lines = authored_activity_lines;
        expected_activity_lines.extend(before_output.iter().cloned());
        assert_eq!(
            app.task_runs
                .get(run_id)
                .ok_or("TaskRun fixture is missing")?
                .duration_ms(),
            100
        );

        let mut harness = Harness::new_with_feedback_clock(
            app,
            Theme::junie(),
            width,
            height,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(4_100),
            },
        );
        let button_id = task_run_stop_id(
            route_id,
            run_id,
            execution_id.activity_id(),
            execution_id,
        );
        assert!(harness.area_of(button_id).is_some(), "Stop button at {width}x{height}");
        let _ = harness.click_id(button_id);
        assert_eq!(
            harness
                .app()
                .world
                .activity_supervisor
                .execution(execution_id)
                .map(|execution| execution.phase()),
            Some(ActivityExecutionPhase::Cancelling)
        );

        harness.app_mut().world.seek(5_060);
        harness.draw();
        let execution = harness
            .app()
            .world
            .activity_supervisor
            .execution(execution_id)
            .ok_or("execution is missing after simulated reap")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
        let run = harness
            .app()
            .task_runs
            .get(run_id)
            .ok_or("cancelled TaskRun is missing")?;
        assert_eq!(run.state(), TaskRunState::Cancelled);
        assert_eq!(run.duration_ms(), 100, "grace time does not extend runtime");
        assert_eq!(
            run.output()
                .iter()
                .map(|line| line.text().to_owned())
                .collect::<Vec<_>>(),
            before_output,
            "stop history does not mutate retained TaskRun output"
        );

        let projection = harness
            .app()
            .activity_projections()
            .get(&execution_id.activity_id())
            .cloned()
            .ok_or("stopped Activity projection is missing")?;
        assert_eq!(projection.stop_grace_ms, Some(960));
        assert_eq!(projection.simulated_stop_history.as_deref(), Some(HISTORY));
        assert_eq!(
            projection.lines,
            expected_activity_lines,
            "complete authored Activity prefix and retained TaskRun output remain unchanged"
        );

        let stop_area = harness
            .area_of(button_id)
            .ok_or("disabled Stop control remains rendered after reap")?;
        let body = super::shell_areas(*harness.buffer().area()).0;
        assert_eq!(stop_area.y, body.y.saturating_add(4));
        assert_eq!(stop_area.x, body.right().saturating_sub(22));
        let execution_label = FULL_EXECUTION;
        assert!(
            termrock::width(execution_label) <= stop_area.x.saturating_sub(body.x),
            "TaskRun execution summary ends before Stop control at {width}x{height}"
        );
        assert!(
            termrock::width(HISTORY) <= body.width,
            "complete history fits at {width}x{height}"
        );
        let frame = harness.text();
        assert!(frame.contains(execution_label), "execution summary at {width}x{height}");
        assert!(frame.contains(HISTORY), "complete lifecycle history at {width}x{height}");
        assert!(frame.contains("authored Activity output"));
        assert!(frame.contains(TASK_OUTPUT));
    }
    Ok(())
}
