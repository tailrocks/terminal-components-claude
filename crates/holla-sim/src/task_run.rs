//! Deterministic task-run model for authored simulated tasks.
//!
//! This module models scripted output and prompts. It never launches a process.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const MAX_RUNTIME_MS: u64 = i64::MAX as u64;
const DEV_PROMPT_AT_MS: u64 = 100;
const DEV_SERVING_AT_MS: u64 = 200;
const DEV_STOP_AFTER_MS: u64 = 100;

/// Identity for one modeled task run.
///
/// IDs are allocated by [`TaskRunStore`] and never wrap or get reused within a store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskRunId(u64);

impl TaskRunId {
    /// Returns this run's store-local numeric identity.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Identity for one prompt, unique across runs in one [`TaskRunStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PromptId(u64);

impl PromptId {
    /// Returns this prompt's store-local numeric identity.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Authored script identities supported by the bounded simulator model.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskScriptId {
    /// Small deterministic dev-server example used by the modeled UI route.
    DevServerDemo,
}

/// State of one simulated run.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunState {
    /// Script can advance on simulation time.
    Running,
    /// Script waits for its current typed prompt answer.
    Waiting,
    /// An authored terminal branch completed.
    Completed,
    /// The modeled run was canceled by its owner.
    Cancelled,
    /// A required future event cannot fit within the modeled clock range.
    ///
    /// A run in this state can still be canceled.
    TimeExhausted,
}

/// Recoverable TaskRun store or lifecycle failure.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunError {
    /// No unique run identity remains in this store.
    RunIdExhausted,
    /// No unique prompt identity remains for a new run.
    PromptIdExhausted,
    /// The requested run does not exist in this store.
    MissingRun,
    /// The run has no open prompt to answer.
    NotWaiting,
    /// The answer names a prompt other than the run's current prompt.
    StalePrompt,
    /// The authored prompt has no exact or default branch.
    BranchMissing,
    /// Wait accounting exceeded the finite supported duration range.
    WaitAccountingExhausted,
}

impl fmt::Display for TaskRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RunIdExhausted => "task run identity exhausted",
            Self::PromptIdExhausted => "task prompt identity exhausted",
            Self::MissingRun => "task run not found",
            Self::NotWaiting => "task run has no open prompt",
            Self::StalePrompt => "task prompt changed",
            Self::BranchMissing => "task prompt has no matching branch",
            Self::WaitAccountingExhausted => "task prompt wait accounting exhausted",
        })
    }
}

impl Error for TaskRunError {}

/// One immutable scripted output line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskRunOutput {
    runtime_ms: u64,
    text: &'static str,
}

impl TaskRunOutput {
    /// Returns the active script time when this line was emitted.
    pub const fn runtime_ms(self) -> u64 {
        self.runtime_ms
    }

    /// Returns the authored output text.
    pub const fn text(self) -> &'static str {
        self.text
    }
}

/// Current typed prompt exposed by a modeled run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskPrompt {
    id: PromptId,
    text: &'static str,
    since_ms: i64,
    secret: bool,
    branches: &'static [BranchSpec],
}

impl TaskPrompt {
    /// Returns this prompt's identity.
    pub const fn id(self) -> PromptId {
        self.id
    }

    /// Returns the authored prompt text.
    pub const fn text(self) -> &'static str {
        self.text
    }

    /// Returns the effective world-clock boundary where the prompt opened.
    pub const fn since_ms(self) -> i64 {
        self.since_ms
    }

    /// Returns whether a future UI must treat the answer as secret.
    pub const fn is_secret(self) -> bool {
        self.secret
    }
}

/// Observable event from one TaskRun update.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunEvent {
    /// Authored output became visible.
    Output(TaskRunOutput),
    /// A prompt became active and requires its matching identity to answer.
    PromptOpened(TaskPrompt),
    /// An authored terminal branch completed.
    Completed {
        /// Frozen active script duration.
        duration_ms: u64,
    },
    /// The modeled run was canceled.
    Cancelled {
        /// Frozen active script duration.
        duration_ms: u64,
    },
    /// A required next event is beyond the supported clock ceiling.
    TimeExhausted,
}

/// Events produced by one run operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunUpdate {
    run_id: TaskRunId,
    events: Vec<TaskRunEvent>,
}

impl TaskRunUpdate {
    /// Returns the run that produced these events.
    pub const fn run_id(&self) -> TaskRunId {
        self.run_id
    }

    /// Returns events in authored execution order.
    pub fn events(&self) -> &[TaskRunEvent] {
        &self.events
    }
}

/// Owns deterministic simulated runs and checked identity allocation.
#[derive(Debug)]
pub struct TaskRunStore {
    next_run_id: Option<u64>,
    next_prompt_id: Option<u64>,
    runs: BTreeMap<TaskRunId, TaskRun>,
}

impl Default for TaskRunStore {
    fn default() -> Self {
        Self {
            next_run_id: Some(1),
            next_prompt_id: Some(1),
            runs: BTreeMap::new(),
        }
    }
}

impl TaskRunStore {
    /// Starts one authored simulated script at the supplied world time.
    ///
    /// Negative time normalizes to zero. The call reserves run and prompt identities before it
    /// publishes the run, so identity exhaustion cannot leave a partial run.
    ///
    /// # Errors
    /// Returns an identity-exhaustion error without inserting a run or consuming another ID.
    pub fn start(
        &mut self,
        script_id: TaskScriptId,
        now_ms: i64,
    ) -> Result<TaskRunId, TaskRunError> {
        let run_number = self
            .next_run_id
            .filter(|number| *number != 0)
            .ok_or(TaskRunError::RunIdExhausted)?;
        let run_id = TaskRunId(run_number);
        if self.runs.contains_key(&run_id) {
            return Err(TaskRunError::RunIdExhausted);
        }
        let steps = script_steps(script_id);
        let prompt_count = steps
            .iter()
            .filter(|step| matches!(step.action, StepAction::Prompt { .. }))
            .count();
        let prompt_reservation = reserve_prompt_ids(self.next_prompt_id, prompt_count)
            .ok_or(TaskRunError::PromptIdExhausted)?;
        let next_run_id = run_number.checked_add(1);
        let run = TaskRun::new(run_id, script_id, now_ms, steps, prompt_reservation.ids);
        self.runs.insert(run_id, run);
        self.next_run_id = next_run_id;
        self.next_prompt_id = prompt_reservation.next;
        Ok(run_id)
    }

    /// Advances every stored run to a nondecreasing effective world time.
    ///
    /// A coarse jump stops at the first prompt's logical boundary. The remaining wall interval
    /// becomes prompt wait, not active script time.
    pub fn advance_to(&mut self, now_ms: i64) -> Vec<TaskRunUpdate> {
        let mut updates = Vec::new();
        for run in self.runs.values_mut() {
            let events = run.advance_to(now_ms);
            if !events.is_empty() {
                updates.push(TaskRunUpdate {
                    run_id: run.id,
                    events,
                });
            }
        }
        updates
    }

    /// Answers the run's current prompt with typed text.
    ///
    /// # Errors
    /// Returns [`TaskRunError::MissingRun`], [`TaskRunError::NotWaiting`],
    /// [`TaskRunError::StalePrompt`], [`TaskRunError::BranchMissing`], or a wait-accounting
    /// error without accepting a stale answer.
    pub fn answer_prompt(
        &mut self,
        run_id: TaskRunId,
        prompt_id: PromptId,
        answer: &str,
        now_ms: i64,
    ) -> Result<TaskRunUpdate, TaskRunError> {
        let run = self.runs.get_mut(&run_id).ok_or(TaskRunError::MissingRun)?;
        let events = run.answer_prompt(prompt_id, answer, now_ms)?;
        Ok(TaskRunUpdate { run_id, events })
    }

    /// Cancels one modeled run, settling an open prompt wait exactly once.
    ///
    /// Cancellation is immediate in this simulator stage; it does not model signals or processes.
    ///
    /// # Errors
    /// Returns [`TaskRunError::MissingRun`] or a wait-accounting error without partial settlement.
    pub fn cancel(
        &mut self,
        run_id: TaskRunId,
        now_ms: i64,
    ) -> Result<TaskRunUpdate, TaskRunError> {
        let run = self.runs.get_mut(&run_id).ok_or(TaskRunError::MissingRun)?;
        let events = run.cancel(now_ms)?;
        Ok(TaskRunUpdate { run_id, events })
    }

    /// Returns a stored run without changing its effective time.
    pub fn get(&self, run_id: TaskRunId) -> Option<&TaskRun> {
        self.runs.get(&run_id)
    }

    /// Returns the number of stored runs.
    pub fn len(&self) -> usize {
        self.runs.len()
    }

    /// Returns whether this store has no runs.
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }
}

/// A preflighted prompt-ID allocation that can be committed with its run.
#[derive(Debug)]
struct PromptIdReservation {
    ids: Vec<PromptId>,
    next: Option<u64>,
}

/// Reserves every prompt identity for a script before its run is published.
///
/// Reserving the complete range up front makes later prompt opening infallible and ensures an
/// exhausted allocator cannot leave a partially started run or reuse an earlier prompt ID.
fn reserve_prompt_ids(first: Option<u64>, count: usize) -> Option<PromptIdReservation> {
    if count == 0 {
        return Some(PromptIdReservation {
            ids: Vec::new(),
            next: first,
        });
    }
    let first = first.filter(|number| *number != 0)?;
    let last_offset = u64::try_from(count.checked_sub(1)?).ok()?;
    let last = first.checked_add(last_offset)?;
    let mut ids = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u64::try_from(offset).ok()?;
        ids.push(PromptId(first.checked_add(offset)?));
    }
    Some(PromptIdReservation {
        ids,
        next: last.checked_add(1),
    })
}

/// One deterministic simulated task run.
#[derive(Debug)]
pub struct TaskRun {
    id: TaskRunId,
    script_id: TaskScriptId,
    state: TaskRunState,
    started_at_ms: i64,
    high_water_ms: i64,
    runtime_ms: u64,
    closed_wait_ms: u64,
    prompt_ids: Vec<PromptId>,
    prompt_cursor: usize,
    active_prompt: Option<TaskPrompt>,
    steps: Vec<ScheduledStep>,
    cursor: usize,
    finish_at_ms: Option<u64>,
    output: Vec<TaskRunOutput>,
}

impl TaskRun {
    fn new(
        id: TaskRunId,
        script_id: TaskScriptId,
        now_ms: i64,
        steps: Vec<ScheduledStep>,
        prompt_ids: Vec<PromptId>,
    ) -> Self {
        let start = now_ms.max(0);
        Self {
            id,
            script_id,
            state: TaskRunState::Running,
            started_at_ms: start,
            high_water_ms: start,
            runtime_ms: 0,
            closed_wait_ms: 0,
            prompt_ids,
            prompt_cursor: 0,
            active_prompt: None,
            steps,
            cursor: 0,
            finish_at_ms: None,
            output: Vec::new(),
        }
    }

    /// Returns this run's identity.
    pub const fn id(&self) -> TaskRunId {
        self.id
    }

    /// Returns the authored script identity.
    pub const fn script_id(&self) -> TaskScriptId {
        self.script_id
    }

    /// Returns the run's current modeled state.
    pub const fn state(&self) -> TaskRunState {
        self.state
    }

    /// Returns the nonnegative world time recorded when the run started.
    pub const fn started_at_ms(&self) -> i64 {
        self.started_at_ms
    }

    /// Returns frozen-or-current active script duration, excluding every prompt wait.
    pub const fn duration_ms(&self) -> u64 {
        self.runtime_ms
    }

    /// Returns the total closed prompt wait, settled once per accepted answer or cancellation.
    pub const fn closed_wait_ms(&self) -> u64 {
        self.closed_wait_ms
    }

    /// Returns the active prompt, if this run is waiting for an answer.
    pub const fn active_prompt(&self) -> Option<TaskPrompt> {
        self.active_prompt
    }

    /// Returns output in authored execution order.
    pub fn output(&self) -> &[TaskRunOutput] {
        &self.output
    }

    fn effective_now(&self, now_ms: i64) -> i64 {
        now_ms.max(0).max(self.high_water_ms)
    }

    fn advance_to(&mut self, now_ms: i64) -> Vec<TaskRunEvent> {
        let mut events = Vec::new();
        let target = self.effective_now(now_ms);
        let previous = self.high_water_ms;
        self.high_water_ms = target;
        if self.state != TaskRunState::Running {
            return events;
        }
        let Some(wall_delta) = target
            .checked_sub(previous)
            .and_then(|delta| u64::try_from(delta).ok())
        else {
            self.mark_time_exhausted(&mut events);
            return events;
        };
        let Some(available) = self
            .runtime_ms
            .checked_add(wall_delta)
            .filter(|runtime| *runtime <= MAX_RUNTIME_MS)
        else {
            self.mark_time_exhausted(&mut events);
            return events;
        };
        let limit = self
            .finish_at_ms
            .map_or(available, |finish| available.min(finish));
        if limit < self.runtime_ms {
            self.mark_time_exhausted(&mut events);
            return events;
        }
        let runtime_at_call = self.runtime_ms;
        while let Some(step) = self.steps.get(self.cursor).copied() {
            let prompt_id = if matches!(step.action, StepAction::Prompt { .. }) {
                match self.prompt_ids.get(self.prompt_cursor).copied() {
                    Some(prompt_id) => Some(prompt_id),
                    None => {
                        self.mark_time_exhausted(&mut events);
                        return events;
                    }
                }
            } else {
                None
            };
            let due = step.at_ms.max(self.runtime_ms);
            if due > limit {
                break;
            }
            let Some(offset) = due.checked_sub(runtime_at_call) else {
                self.mark_time_exhausted(&mut events);
                return events;
            };
            let Some(offset_ms) = i64::try_from(offset).ok() else {
                self.mark_time_exhausted(&mut events);
                return events;
            };
            let Some(boundary_ms) = previous.checked_add(offset_ms) else {
                self.mark_time_exhausted(&mut events);
                return events;
            };
            self.runtime_ms = due;
            self.cursor += 1;
            match step.action {
                StepAction::Output(text) => {
                    let output = TaskRunOutput {
                        runtime_ms: due,
                        text,
                    };
                    self.output.push(output);
                    events.push(TaskRunEvent::Output(output));
                }
                StepAction::Prompt {
                    text,
                    secret,
                    branches,
                } => {
                    let Some(prompt_id) = prompt_id else {
                        self.mark_time_exhausted(&mut events);
                        return events;
                    };
                    self.prompt_cursor += 1;
                    let prompt = TaskPrompt {
                        id: prompt_id,
                        text,
                        since_ms: boundary_ms,
                        secret,
                        branches,
                    };
                    self.active_prompt = Some(prompt);
                    self.state = TaskRunState::Waiting;
                    events.push(TaskRunEvent::PromptOpened(prompt));
                    return events;
                }
            }
        }
        self.runtime_ms = limit;
        if self
            .finish_at_ms
            .is_some_and(|finish| self.runtime_ms >= finish)
        {
            self.state = TaskRunState::Completed;
            events.push(TaskRunEvent::Completed {
                duration_ms: self.runtime_ms,
            });
        } else if target == i64::MAX && self.has_future_deadline() {
            self.mark_time_exhausted(&mut events);
        }
        events
    }

    fn answer_prompt(
        &mut self,
        prompt_id: PromptId,
        answer: &str,
        now_ms: i64,
    ) -> Result<Vec<TaskRunEvent>, TaskRunError> {
        if self.state != TaskRunState::Waiting {
            return Err(TaskRunError::NotWaiting);
        }
        let prompt = self.active_prompt.ok_or(TaskRunError::NotWaiting)?;
        if prompt.id != prompt_id {
            return Err(TaskRunError::StalePrompt);
        }
        let branch = prompt
            .branches
            .iter()
            .find(|branch| branch.answer == Some(answer))
            .or_else(|| {
                prompt
                    .branches
                    .iter()
                    .find(|branch| branch.answer.is_none())
            })
            .ok_or(TaskRunError::BranchMissing)?;
        let prepared = self.prepare_branch(branch);
        let target = self.effective_now(now_ms);
        let wait_ms = target
            .checked_sub(prompt.since_ms)
            .and_then(|wait| u64::try_from(wait).ok())
            .ok_or(TaskRunError::WaitAccountingExhausted)?;
        let closed_wait_ms = self
            .closed_wait_ms
            .checked_add(wait_ms)
            .ok_or(TaskRunError::WaitAccountingExhausted)?;
        self.high_water_ms = target;
        self.closed_wait_ms = closed_wait_ms;
        self.active_prompt = None;
        let Some((branch_steps, finish_at_ms)) = prepared else {
            self.state = TaskRunState::TimeExhausted;
            return Ok(vec![TaskRunEvent::TimeExhausted]);
        };
        let suffix = self.steps.split_off(self.cursor);
        self.steps.extend(branch_steps);
        if finish_at_ms.is_none() {
            self.steps.extend(suffix);
        }
        self.finish_at_ms = finish_at_ms;
        self.state = TaskRunState::Running;
        Ok(self.advance_to(target))
    }

    fn prepare_branch(&self, branch: &BranchSpec) -> Option<(Vec<ScheduledStep>, Option<u64>)> {
        let mut steps = Vec::with_capacity(branch.outputs.len());
        for output in branch.outputs {
            let at_ms = self.runtime_ms.checked_add(output.after_ms)?;
            if at_ms > MAX_RUNTIME_MS {
                return None;
            }
            steps.push(ScheduledStep {
                at_ms,
                action: StepAction::Output(output.text),
            });
        }
        let finish_at_ms = match branch.finish_after_ms {
            Some(delay) => {
                let finish = self.runtime_ms.checked_add(delay)?;
                if finish > MAX_RUNTIME_MS || steps.iter().any(|step| step.at_ms > finish) {
                    return None;
                }
                Some(finish)
            }
            None => None,
        };
        Some((steps, finish_at_ms))
    }

    fn cancel(&mut self, now_ms: i64) -> Result<Vec<TaskRunEvent>, TaskRunError> {
        let mut events = self.advance_to(now_ms);
        if matches!(
            self.state,
            TaskRunState::Completed | TaskRunState::Cancelled
        ) {
            return Ok(events);
        }
        let target = self.effective_now(now_ms);
        let wait_ms = match self.active_prompt {
            Some(prompt) => target
                .checked_sub(prompt.since_ms)
                .and_then(|wait| u64::try_from(wait).ok())
                .ok_or(TaskRunError::WaitAccountingExhausted)?,
            None => 0,
        };
        let closed_wait_ms = self
            .closed_wait_ms
            .checked_add(wait_ms)
            .ok_or(TaskRunError::WaitAccountingExhausted)?;
        self.high_water_ms = target;
        self.closed_wait_ms = closed_wait_ms;
        self.active_prompt = None;
        self.state = TaskRunState::Cancelled;
        events.push(TaskRunEvent::Cancelled {
            duration_ms: self.runtime_ms,
        });
        Ok(events)
    }

    fn has_future_deadline(&self) -> bool {
        self.steps
            .get(self.cursor)
            .is_some_and(|step| step.at_ms.max(self.runtime_ms) > self.runtime_ms)
            || self
                .finish_at_ms
                .is_some_and(|finish| finish > self.runtime_ms)
    }

    fn mark_time_exhausted(&mut self, events: &mut Vec<TaskRunEvent>) {
        self.state = TaskRunState::TimeExhausted;
        events.push(TaskRunEvent::TimeExhausted);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScheduledStep {
    at_ms: u64,
    action: StepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepAction {
    Output(&'static str),
    Prompt {
        text: &'static str,
        secret: bool,
        branches: &'static [BranchSpec],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BranchOutput {
    after_ms: u64,
    text: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BranchSpec {
    answer: Option<&'static str>,
    outputs: &'static [BranchOutput],
    finish_after_ms: Option<u64>,
}

static STOP_OUTPUTS: &[BranchOutput] = &[BranchOutput {
    after_ms: 0,
    text: "dev server stopped by operator",
}];
static CONTINUE_OUTPUTS: &[BranchOutput] = &[BranchOutput {
    after_ms: 0,
    text: "dev server remains available",
}];
static DEV_BRANCHES: &[BranchSpec] = &[
    BranchSpec {
        answer: Some("y"),
        outputs: STOP_OUTPUTS,
        finish_after_ms: Some(DEV_STOP_AFTER_MS),
    },
    BranchSpec {
        answer: None,
        outputs: CONTINUE_OUTPUTS,
        finish_after_ms: None,
    },
];

fn script_steps(script_id: TaskScriptId) -> Vec<ScheduledStep> {
    match script_id {
        TaskScriptId::DevServerDemo => vec![
            ScheduledStep {
                at_ms: 0,
                action: StepAction::Output("Run dev · modeled simulation · no process executed"),
            },
            ScheduledStep {
                at_ms: DEV_PROMPT_AT_MS,
                action: StepAction::Prompt {
                    text: "Stop modeled dev server? [y/N]",
                    secret: false,
                    branches: DEV_BRANCHES,
                },
            },
            ScheduledStep {
                at_ms: DEV_SERVING_AT_MS,
                action: StepAction::Output("serving at localhost:5199"),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BranchOutput, BranchSpec, PromptId, ScheduledStep, StepAction, TaskRun, TaskRunError,
        TaskRunId, TaskRunState, TaskRunStore, TaskScriptId, reserve_prompt_ids,
    };

    static EMPTY_OUTPUTS: &[BranchOutput] = &[];
    static CONTINUE_BRANCHES: &[BranchSpec] = &[BranchSpec {
        answer: None,
        outputs: EMPTY_OUTPUTS,
        finish_after_ms: None,
    }];

    #[test]
    fn run_id_exhaustion_never_wraps_or_creates_a_partial_run() -> Result<(), TaskRunError> {
        let mut store = TaskRunStore {
            next_run_id: Some(u64::MAX),
            ..TaskRunStore::default()
        };
        let last_id = store.start(TaskScriptId::DevServerDemo, 0)?;
        assert_eq!(last_id.get(), u64::MAX);
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.start(TaskScriptId::DevServerDemo, 1),
            Err(TaskRunError::RunIdExhausted)
        );
        assert_eq!(store.len(), 1);
        Ok(())
    }

    #[test]
    fn prompt_id_exhaustion_fails_start_without_consuming_run_id() {
        let mut store = TaskRunStore {
            next_run_id: Some(9),
            next_prompt_id: None,
            ..TaskRunStore::default()
        };
        assert_eq!(
            store.start(TaskScriptId::DevServerDemo, 0),
            Err(TaskRunError::PromptIdExhausted)
        );
        assert_eq!(store.len(), 0);
        assert_eq!(store.next_run_id, Some(9));
    }

    #[test]
    fn prompt_id_range_reservation_is_checked_and_atomic() -> Result<(), TaskRunError> {
        let reservation =
            reserve_prompt_ids(Some(u64::MAX - 1), 2).ok_or(TaskRunError::PromptIdExhausted)?;
        assert_eq!(
            reservation.ids,
            vec![PromptId(u64::MAX - 1), PromptId(u64::MAX)]
        );
        assert_eq!(reservation.next, None);
        assert!(reserve_prompt_ids(Some(u64::MAX), 2).is_none());
        Ok(())
    }

    #[test]
    fn multiple_prompt_waits_settle_once_and_each_prompt_has_new_identity()
    -> Result<(), TaskRunError> {
        let steps = vec![
            ScheduledStep {
                at_ms: 10,
                action: StepAction::Prompt {
                    text: "first",
                    secret: false,
                    branches: CONTINUE_BRANCHES,
                },
            },
            ScheduledStep {
                at_ms: 20,
                action: StepAction::Prompt {
                    text: "second",
                    secret: false,
                    branches: CONTINUE_BRANCHES,
                },
            },
            ScheduledStep {
                at_ms: 30,
                action: StepAction::Output("after prompts"),
            },
        ];
        let mut run = TaskRun::new(
            TaskRunId(1),
            TaskScriptId::DevServerDemo,
            0,
            steps,
            vec![PromptId(11), PromptId(12)],
        );
        run.advance_to(10);
        let first = run.active_prompt().ok_or(TaskRunError::NotWaiting)?;
        assert_eq!(first.since_ms(), 10);
        run.advance_to(15);
        run.answer_prompt(first.id(), "", 15)?;
        assert_eq!(run.closed_wait_ms(), 5);
        assert_eq!(run.state(), TaskRunState::Running);

        run.advance_to(25);
        let second = run.active_prompt().ok_or(TaskRunError::NotWaiting)?;
        assert_eq!(second.since_ms(), 25);
        assert_ne!(first.id(), second.id());
        let before = (run.state(), run.duration_ms(), run.closed_wait_ms(), second);
        assert_eq!(
            run.answer_prompt(first.id(), "", 26),
            Err(TaskRunError::StalePrompt)
        );
        assert_eq!(
            (
                run.state(),
                run.duration_ms(),
                run.closed_wait_ms(),
                run.active_prompt().ok_or(TaskRunError::NotWaiting)?,
            ),
            before
        );

        run.advance_to(30);
        run.answer_prompt(second.id(), "", 30)?;
        assert_eq!(run.closed_wait_ms(), 10);
        assert_eq!(
            run.answer_prompt(second.id(), "", 30),
            Err(TaskRunError::NotWaiting)
        );
        run.advance_to(40);
        assert_eq!(run.duration_ms(), 30);
        assert_eq!(run.closed_wait_ms(), 10);
        assert_eq!(
            run.output()
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>(),
            vec!["after prompts"]
        );
        Ok(())
    }
}
