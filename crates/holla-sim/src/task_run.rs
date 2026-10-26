use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const MAX_RUNTIME_MS: u64 = i64::MAX as u64;
const DEV_PROMPT_AT_MS: u64 = 100;
const DEV_SERVING_AT_MS: u64 = 200;
const DEV_STOP_AFTER_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskRunId(u64);

impl TaskRunId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PromptId(u64);

impl PromptId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskScriptId {
    DevServerDemo,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunState {
    Running,

    Waiting,

    Completed,

    Cancelled,

    TimeExhausted,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunError {
    RunIdExhausted,

    PromptIdExhausted,

    MissingRun,

    NotWaiting,

    StalePrompt,

    BranchMissing,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskRunOutput {
    runtime_ms: u64,
    text: &'static str,
}

impl TaskRunOutput {
    pub const fn runtime_ms(self) -> u64 {
        self.runtime_ms
    }

    pub const fn text(self) -> &'static str {
        self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskPrompt {
    id: PromptId,
    text: &'static str,
    since_ms: i64,
    secret: bool,
    branches: &'static [BranchSpec],
}

impl TaskPrompt {
    pub const fn id(self) -> PromptId {
        self.id
    }

    pub const fn text(self) -> &'static str {
        self.text
    }

    pub const fn since_ms(self) -> i64 {
        self.since_ms
    }

    pub const fn is_secret(self) -> bool {
        self.secret
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunEvent {
    Output(TaskRunOutput),

    PromptOpened(TaskPrompt),

    Completed { duration_ms: u64 },

    Cancelled { duration_ms: u64 },

    TimeExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunUpdate {
    run_id: TaskRunId,
    events: Vec<TaskRunEvent>,
}

impl TaskRunUpdate {
    pub const fn run_id(&self) -> TaskRunId {
        self.run_id
    }

    pub fn events(&self) -> &[TaskRunEvent] {
        &self.events
    }
}

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

    pub fn cancel(
        &mut self,
        run_id: TaskRunId,
        now_ms: i64,
    ) -> Result<TaskRunUpdate, TaskRunError> {
        let run = self.runs.get_mut(&run_id).ok_or(TaskRunError::MissingRun)?;
        let events = run.cancel(now_ms)?;
        Ok(TaskRunUpdate { run_id, events })
    }

    pub fn get(&self, run_id: TaskRunId) -> Option<&TaskRun> {
        self.runs.get(&run_id)
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }
}

#[derive(Debug)]
struct PromptIdReservation {
    ids: Vec<PromptId>,
    next: Option<u64>,
}

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

    pub const fn id(&self) -> TaskRunId {
        self.id
    }

    pub const fn script_id(&self) -> TaskScriptId {
        self.script_id
    }

    pub const fn state(&self) -> TaskRunState {
        self.state
    }

    pub const fn started_at_ms(&self) -> i64 {
        self.started_at_ms
    }

    pub const fn duration_ms(&self) -> u64 {
        self.runtime_ms
    }

    pub const fn closed_wait_ms(&self) -> u64 {
        self.closed_wait_ms
    }

    pub const fn active_prompt(&self) -> Option<TaskPrompt> {
        self.active_prompt
    }

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
