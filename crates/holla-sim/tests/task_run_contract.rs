use holla_sim::task_run::{PromptId, TaskRunError, TaskRunState, TaskRunStore, TaskScriptId};
use std::error::Error;

fn prompt_id(
    store: &TaskRunStore,
    run_id: holla_sim::task_run::TaskRunId,
) -> Result<PromptId, TaskRunError> {
    store
        .get(run_id)
        .and_then(|run| run.active_prompt())
        .map(|prompt| prompt.id())
        .ok_or(TaskRunError::NotWaiting)
}

fn output_texts(
    store: &TaskRunStore,
    run_id: holla_sim::task_run::TaskRunId,
) -> Result<Vec<&'static str>, TaskRunError> {
    store
        .get(run_id)
        .map(|run| run.output().iter().map(|line| line.text()).collect())
        .ok_or(TaskRunError::MissingRun)
}

fn completed_stop_duration(answer_at_ms: i64) -> Result<(u64, Vec<&'static str>), TaskRunError> {
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, 0)?;
    store.advance_to(100);
    let prompt_id = prompt_id(&store, run_id)?;
    store.answer_prompt(run_id, prompt_id, "y", answer_at_ms)?;
    store.advance_to(answer_at_ms + 100);
    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.state(), TaskRunState::Completed);
    Ok((run.duration_ms(), output_texts(&store, run_id)?))
}

#[test]
fn prompt_wait_is_excluded_and_active_duration_freezes() -> Result<(), Box<dyn Error>> {
    let mut waiting = TaskRunStore::default();
    let waiting_id = waiting.start(TaskScriptId::DevServerDemo, 0)?;
    waiting.advance_to(100);
    let prompt = waiting
        .get(waiting_id)
        .and_then(|run| run.active_prompt())
        .ok_or(TaskRunError::NotWaiting)?;
    assert_eq!(prompt.since_ms(), 100);
    assert_eq!(
        waiting.get(waiting_id).map(|run| run.duration_ms()),
        Some(100)
    );
    waiting.advance_to(500);
    assert_eq!(
        waiting.get(waiting_id).map(|run| run.duration_ms()),
        Some(100)
    );

    let (fast_duration, fast_output) = completed_stop_duration(101)?;
    let (slow_duration, slow_output) = completed_stop_duration(501)?;
    assert_eq!(fast_duration, 200);
    assert_eq!(slow_duration, fast_duration);
    assert_eq!(fast_output, slow_output);
    assert!(!slow_output.contains(&"serving at localhost:5199"));
    Ok(())
}

#[test]
fn cancel_during_open_prompt_settles_wait_once() -> Result<(), Box<dyn Error>> {
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, 0)?;
    store.advance_to(400);
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(100));
    assert_eq!(store.get(run_id).map(|run| run.closed_wait_ms()), Some(0));

    store.cancel(run_id, 700)?;
    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.state(), TaskRunState::Cancelled);
    assert_eq!(run.duration_ms(), 100);
    assert_eq!(run.closed_wait_ms(), 600);
    store.cancel(run_id, 900)?;
    store.advance_to(1_000);
    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.duration_ms(), 100);
    assert_eq!(run.closed_wait_ms(), 600);
    Ok(())
}

#[test]
fn continuing_branch_restores_suffix_in_order() -> Result<(), Box<dyn Error>> {
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, 0)?;
    store.advance_to(100);
    let prompt_id = prompt_id(&store, run_id)?;
    store.answer_prompt(run_id, prompt_id, "n", 130)?;
    store.advance_to(230);

    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.state(), TaskRunState::Running);
    assert_eq!(
        output_texts(&store, run_id)?,
        vec![
            "Run dev · modeled simulation · no process executed",
            "dev server remains available",
            "serving at localhost:5199",
        ]
    );
    Ok(())
}

#[test]
fn stale_cross_run_prompt_ids_do_not_mutate_either_run() -> Result<(), Box<dyn Error>> {
    let mut store = TaskRunStore::default();
    let first = store.start(TaskScriptId::DevServerDemo, 0)?;
    let second = store.start(TaskScriptId::DevServerDemo, 0)?;
    store.advance_to(100);
    let first_prompt = prompt_id(&store, first)?;
    let second_prompt = prompt_id(&store, second)?;
    assert_ne!(first_prompt, second_prompt);
    let before_first = store.get(first).ok_or(TaskRunError::MissingRun)?;
    let first_before = (
        before_first.state(),
        before_first.duration_ms(),
        before_first.closed_wait_ms(),
    );
    let before_second = store.get(second).ok_or(TaskRunError::MissingRun)?;
    let second_before = (
        before_second.state(),
        before_second.duration_ms(),
        before_second.closed_wait_ms(),
    );

    assert_eq!(
        store.answer_prompt(second, first_prompt, "y", 500),
        Err(TaskRunError::StalePrompt)
    );
    assert_eq!(
        store.answer_prompt(first, second_prompt, "y", 500),
        Err(TaskRunError::StalePrompt)
    );
    let after_first = store.get(first).ok_or(TaskRunError::MissingRun)?;
    let after_second = store.get(second).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(
        (
            after_first.state(),
            after_first.duration_ms(),
            after_first.closed_wait_ms()
        ),
        first_before
    );
    assert_eq!(
        (
            after_second.state(),
            after_second.duration_ms(),
            after_second.closed_wait_ms()
        ),
        second_before
    );
    Ok(())
}

#[test]
fn coarse_jump_opens_prompt_at_logical_boundary() -> Result<(), Box<dyn Error>> {
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, 0)?;
    store.advance_to(400);
    let prompt = store
        .get(run_id)
        .and_then(|run| run.active_prompt())
        .ok_or(TaskRunError::NotWaiting)?;
    assert_eq!(prompt.since_ms(), 100);
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(100));

    let mut incremental = TaskRunStore::default();
    let incremental_id = incremental.start(TaskScriptId::DevServerDemo, 0)?;
    for now_ms in [25, 50, 75, 100, 150, 200, 250, 300, 350, 400] {
        incremental.advance_to(now_ms);
    }
    let incremental_prompt = incremental
        .get(incremental_id)
        .and_then(|run| run.active_prompt())
        .ok_or(TaskRunError::NotWaiting)?;
    assert_eq!(incremental_prompt, prompt);
    assert_eq!(
        incremental.get(incremental_id).map(|run| run.duration_ms()),
        store.get(run_id).map(|run| run.duration_ms())
    );

    store.answer_prompt(run_id, prompt.id(), "y", 500)?;
    assert_eq!(store.get(run_id).map(|run| run.closed_wait_ms()), Some(400));
    store.advance_to(600);
    assert_eq!(
        store.get(run_id).map(|run| run.state()),
        Some(TaskRunState::Completed)
    );
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(200));
    Ok(())
}

#[test]
fn negative_and_backward_times_never_reduce_runtime() -> Result<(), Box<dyn Error>> {
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, -10)?;
    assert_eq!(store.get(run_id).map(|run| run.started_at_ms()), Some(0));
    store.advance_to(50);
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(50));
    store.advance_to(-100);
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(50));
    Ok(())
}

#[test]
fn clock_ceiling_reports_unreachable_deadline_and_still_allows_cancel() -> Result<(), Box<dyn Error>>
{
    let mut store = TaskRunStore::default();
    let run_id = store.start(TaskScriptId::DevServerDemo, i64::MAX - 100)?;
    store.advance_to(i64::MAX);
    let prompt = store
        .get(run_id)
        .and_then(|run| run.active_prompt())
        .ok_or(TaskRunError::NotWaiting)?;
    assert_eq!(store.get(run_id).map(|run| run.duration_ms()), Some(100));

    store.answer_prompt(run_id, prompt.id(), "n", i64::MAX)?;
    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.state(), TaskRunState::TimeExhausted);
    assert_eq!(run.duration_ms(), 100);
    store.cancel(run_id, i64::MAX)?;
    let run = store.get(run_id).ok_or(TaskRunError::MissingRun)?;
    assert_eq!(run.state(), TaskRunState::Cancelled);
    assert_eq!(run.duration_ms(), 100);
    Ok(())
}
