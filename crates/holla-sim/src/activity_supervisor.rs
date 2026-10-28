use crate::domain::activity::Activity;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const NORMAL_REAP_DELAY_MS: i64 = 240;
const RESISTANT_KILL_DELAY_MS: i64 = 800;
const RESISTANT_REAP_DELAY_MS: i64 = 960;
const MAX_STOP_MILESTONES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActivityExecutionId {
    activity_id: u32,
    generation: u64,
}

impl ActivityExecutionId {
    pub const fn activity_id(self) -> u32 {
        self.activity_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityExecutionStart {
    Queued,

    Running,

    Waiting,

    Detached,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityExecutionPhase {
    Queued,

    Running,

    Waiting,

    Detached,

    Cancelling,

    Stopped,
}

impl ActivityExecutionPhase {
    pub const fn is_owned(self) -> bool {
        matches!(
            self,
            ActivityExecutionPhase::Queued
                | ActivityExecutionPhase::Running
                | ActivityExecutionPhase::Waiting
                | ActivityExecutionPhase::Detached
                | ActivityExecutionPhase::Cancelling
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopProfile {
    Responsive,

    Resistant,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopPhase {
    NotRequested,

    TermSent,

    KillSent,

    Reaped,

    CancelledBeforeStart,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopMilestone {
    TermSent { at_ms: i64 },

    KillSent { at_ms: i64 },

    Reaped { at_ms: i64 },

    CancelledBeforeStart { at_ms: i64 },
}

impl ActivityStopMilestone {
    pub const fn at_ms(self) -> i64 {
        match self {
            ActivityStopMilestone::TermSent { at_ms }
            | ActivityStopMilestone::KillSent { at_ms }
            | ActivityStopMilestone::Reaped { at_ms }
            | ActivityStopMilestone::CancelledBeforeStart { at_ms } => at_ms,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopOutcome {
    CancelledBeforeStart,

    TermSent,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivitySupervisorError {
    DuplicateActivityId { activity_id: u32 },

    GenerationExhausted,

    UnknownExecution { activity_id: u32, generation: u64 },

    StaleExecution { activity_id: u32, generation: u64 },

    ReplacementBeforeReap { activity_id: u32 },

    StaleActivitySnapshot { activity_id: u32 },

    NotLive { activity_id: u32 },

    AlreadyStopping { activity_id: u32 },

    UnreachableTime { activity_id: u32 },
}

impl fmt::Display for ActivitySupervisorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivitySupervisorError::DuplicateActivityId { activity_id } => {
                write!(formatter, "activity ID {activity_id} is already registered")
            }
            ActivitySupervisorError::GenerationExhausted => {
                formatter.write_str("activity execution generations are exhausted")
            }
            ActivitySupervisorError::UnknownExecution {
                activity_id,
                generation,
            } => write!(
                formatter,
                "activity execution {activity_id}:{generation} is not registered"
            ),
            ActivitySupervisorError::StaleExecution {
                activity_id,
                generation,
            } => write!(
                formatter,
                "activity execution {activity_id}:{generation} is stale"
            ),
            ActivitySupervisorError::ReplacementBeforeReap { activity_id } => write!(
                formatter,
                "activity {activity_id} cannot be replaced before simulated reaping"
            ),
            ActivitySupervisorError::StaleActivitySnapshot { activity_id } => {
                write!(
                    formatter,
                    "activity {activity_id} no longer matches its snapshot"
                )
            }
            ActivitySupervisorError::NotLive { activity_id } => {
                write!(formatter, "activity {activity_id} is not live")
            }
            ActivitySupervisorError::AlreadyStopping { activity_id } => {
                write!(formatter, "activity {activity_id} is already stopping")
            }
            ActivitySupervisorError::UnreachableTime { activity_id } => write!(
                formatter,
                "activity {activity_id} stop deadline exceeds simulation time"
            ),
        }
    }
}

impl Error for ActivitySupervisorError {}

#[derive(Debug)]
pub struct ActivityExecution {
    id: ActivityExecutionId,
    snapshot: Activity,
    phase: ActivityExecutionPhase,
    profile: ActivityStopProfile,
    stop_phase: ActivityStopPhase,
    stop_requested_ms: Option<i64>,
    kill_deadline_ms: Option<i64>,
    reap_deadline_ms: Option<i64>,
    milestones: Vec<ActivityStopMilestone>,
}

impl ActivityExecution {
    pub const fn id(&self) -> ActivityExecutionId {
        self.id
    }

    pub const fn snapshot(&self) -> &Activity {
        &self.snapshot
    }

    pub const fn phase(&self) -> ActivityExecutionPhase {
        self.phase
    }

    pub const fn is_owned(&self) -> bool {
        self.phase.is_owned()
    }

    pub const fn profile(&self) -> ActivityStopProfile {
        self.profile
    }

    pub const fn stop_phase(&self) -> ActivityStopPhase {
        self.stop_phase
    }

    pub const fn stop_requested_ms(&self) -> Option<i64> {
        self.stop_requested_ms
    }

    pub const fn kill_deadline_ms(&self) -> Option<i64> {
        self.kill_deadline_ms
    }

    pub const fn reap_deadline_ms(&self) -> Option<i64> {
        self.reap_deadline_ms
    }

    pub fn milestones(&self) -> &[ActivityStopMilestone] {
        &self.milestones
    }

    fn record(&mut self, milestone: ActivityStopMilestone) {
        if !self.milestones.contains(&milestone) && self.milestones.len() < MAX_STOP_MILESTONES {
            self.milestones.push(milestone);
        }
    }
}

pub struct ActivityStopAdmission<'a> {
    execution: &'a mut ActivityExecution,
    at_ms: i64,
    schedule: StopSchedule,
}

impl ActivityStopAdmission<'_> {
    pub fn commit(self) -> ActivityStopOutcome {
        match self.schedule {
            StopSchedule::Queued => {
                self.execution.phase = ActivityExecutionPhase::Stopped;
                self.execution.stop_phase = ActivityStopPhase::CancelledBeforeStart;
                self.execution.stop_requested_ms = Some(self.at_ms);
                self.execution
                    .record(ActivityStopMilestone::CancelledBeforeStart { at_ms: self.at_ms });
                ActivityStopOutcome::CancelledBeforeStart
            }
            StopSchedule::Graceful {
                kill_deadline_ms,
                reap_deadline_ms,
            } => {
                self.execution.phase = ActivityExecutionPhase::Cancelling;
                self.execution.stop_phase = ActivityStopPhase::TermSent;
                self.execution.stop_requested_ms = Some(self.at_ms);
                self.execution.kill_deadline_ms = kill_deadline_ms;
                self.execution.reap_deadline_ms = Some(reap_deadline_ms);
                self.execution
                    .record(ActivityStopMilestone::TermSent { at_ms: self.at_ms });
                ActivityStopOutcome::TermSent
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StopSchedule {
    Queued,
    Graceful {
        kill_deadline_ms: Option<i64>,
        reap_deadline_ms: i64,
    },
}

#[derive(Debug)]
pub struct ActivitySupervisor {
    next_generation: Option<u64>,
    high_water_ms: i64,
    executions: BTreeMap<ActivityExecutionId, ActivityExecution>,
    current_by_activity_id: BTreeMap<u32, ActivityExecutionId>,
}

impl Default for ActivitySupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivitySupervisor {
    pub fn new() -> Self {
        Self {
            next_generation: Some(1),
            high_water_ms: 0,
            executions: BTreeMap::new(),
            current_by_activity_id: BTreeMap::new(),
        }
    }

    pub fn register(
        &mut self,
        snapshot: &Activity,
        start: ActivityExecutionStart,
        profile: ActivityStopProfile,
    ) -> Result<ActivityExecutionId, ActivitySupervisorError> {
        if self.current_by_activity_id.contains_key(&snapshot.id) {
            return Err(ActivitySupervisorError::DuplicateActivityId {
                activity_id: snapshot.id,
            });
        }
        let generation = self
            .next_generation
            .ok_or(ActivitySupervisorError::GenerationExhausted)?;
        let id = ActivityExecutionId {
            activity_id: snapshot.id,
            generation,
        };
        let execution = ActivityExecution::new(id, snapshot, start, profile);
        self.executions.insert(id, execution);
        self.current_by_activity_id.insert(snapshot.id, id);
        self.next_generation = generation.checked_add(1);
        Ok(id)
    }

    pub fn register_terminal(
        &mut self,
        snapshot: &Activity,
        profile: ActivityStopProfile,
    ) -> Result<ActivityExecutionId, ActivitySupervisorError> {
        if self.current_by_activity_id.contains_key(&snapshot.id) {
            return Err(ActivitySupervisorError::DuplicateActivityId {
                activity_id: snapshot.id,
            });
        }
        let generation = self
            .next_generation
            .ok_or(ActivitySupervisorError::GenerationExhausted)?;
        let next_generation = generation.checked_add(1);
        let id = ActivityExecutionId {
            activity_id: snapshot.id,
            generation,
        };
        let mut execution =
            ActivityExecution::new(id, snapshot, ActivityExecutionStart::Running, profile);
        execution.phase = ActivityExecutionPhase::Stopped;

        self.executions.insert(id, execution);
        self.current_by_activity_id.insert(snapshot.id, id);
        self.next_generation = next_generation;
        Ok(id)
    }

    pub fn replace_stopped(
        &mut self,
        previous_id: ActivityExecutionId,
        snapshot: &Activity,
        start: ActivityExecutionStart,
        profile: ActivityStopProfile,
    ) -> Result<ActivityExecutionId, ActivitySupervisorError> {
        if snapshot.id != previous_id.activity_id {
            return Err(ActivitySupervisorError::StaleActivitySnapshot {
                activity_id: previous_id.activity_id,
            });
        }
        if self
            .current_by_activity_id
            .get(&previous_id.activity_id)
            .copied()
            != Some(previous_id)
        {
            return Err(ActivitySupervisorError::StaleExecution {
                activity_id: previous_id.activity_id,
                generation: previous_id.generation,
            });
        }
        let previous =
            self.executions
                .get(&previous_id)
                .ok_or(ActivitySupervisorError::UnknownExecution {
                    activity_id: previous_id.activity_id,
                    generation: previous_id.generation,
                })?;
        if previous.phase != ActivityExecutionPhase::Stopped {
            return Err(ActivitySupervisorError::ReplacementBeforeReap {
                activity_id: previous_id.activity_id,
            });
        }
        let generation = self
            .next_generation
            .ok_or(ActivitySupervisorError::GenerationExhausted)?;
        let id = ActivityExecutionId {
            activity_id: snapshot.id,
            generation,
        };
        let execution = ActivityExecution::new(id, snapshot, start, profile);
        self.executions.insert(id, execution);
        self.current_by_activity_id.insert(snapshot.id, id);
        self.next_generation = generation.checked_add(1);
        Ok(id)
    }

    pub fn current_execution_id(&self, activity_id: u32) -> Option<ActivityExecutionId> {
        self.current_by_activity_id.get(&activity_id).copied()
    }

    pub fn execution(&self, id: ActivityExecutionId) -> Option<&ActivityExecution> {
        self.executions.get(&id)
    }

    pub fn observe_terminal_exit(
        &mut self,
        id: ActivityExecutionId,
        snapshot: &Activity,
    ) -> Result<(), ActivitySupervisorError> {
        let current = self.current_by_activity_id.get(&id.activity_id).copied();
        if current != Some(id) {
            return Err(if current.is_some() {
                ActivitySupervisorError::StaleExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                }
            } else {
                ActivitySupervisorError::UnknownExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                }
            });
        }
        let execution =
            self.executions
                .get_mut(&id)
                .ok_or(ActivitySupervisorError::UnknownExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                })?;
        if &execution.snapshot != snapshot {
            return Err(ActivitySupervisorError::StaleActivitySnapshot {
                activity_id: id.activity_id,
            });
        }
        match execution.phase {
            ActivityExecutionPhase::Queued => Err(ActivitySupervisorError::NotLive {
                activity_id: id.activity_id,
            }),
            ActivityExecutionPhase::Running
            | ActivityExecutionPhase::Waiting
            | ActivityExecutionPhase::Detached => {
                execution.phase = ActivityExecutionPhase::Stopped;
                Ok(())
            }
            ActivityExecutionPhase::Cancelling | ActivityExecutionPhase::Stopped => Ok(()),
        }
    }

    pub const fn high_water_ms(&self) -> i64 {
        self.high_water_ms
    }

    pub fn advance_to(&mut self, now_ms: i64) {
        let target_ms = now_ms.max(0).max(self.high_water_ms);
        self.high_water_ms = target_ms;
        for execution in self.executions.values_mut() {
            if execution.phase != ActivityExecutionPhase::Cancelling {
                continue;
            }
            if execution.stop_phase == ActivityStopPhase::TermSent
                && execution
                    .kill_deadline_ms
                    .is_some_and(|deadline| deadline <= target_ms)
            {
                if let Some(at_ms) = execution.kill_deadline_ms {
                    execution.record(ActivityStopMilestone::KillSent { at_ms });
                }
                execution.stop_phase = ActivityStopPhase::KillSent;
            }
            if execution
                .reap_deadline_ms
                .is_some_and(|deadline| deadline <= target_ms)
            {
                if let Some(at_ms) = execution.reap_deadline_ms {
                    execution.record(ActivityStopMilestone::Reaped { at_ms });
                }
                execution.stop_phase = ActivityStopPhase::Reaped;
                execution.phase = ActivityExecutionPhase::Stopped;
            }
        }
    }

    pub fn admit_stop<'a>(
        &'a mut self,
        id: ActivityExecutionId,
        snapshot: &Activity,
        now_ms: i64,
    ) -> Result<ActivityStopAdmission<'a>, ActivitySupervisorError> {
        let current = self.current_by_activity_id.get(&id.activity_id).copied();
        if current != Some(id) {
            return Err(if current.is_some() {
                ActivitySupervisorError::StaleExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                }
            } else {
                ActivitySupervisorError::UnknownExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                }
            });
        }
        let Some(execution) = self.executions.get(&id) else {
            return Err(ActivitySupervisorError::UnknownExecution {
                activity_id: id.activity_id,
                generation: id.generation,
            });
        };
        if &execution.snapshot != snapshot {
            return Err(ActivitySupervisorError::StaleActivitySnapshot {
                activity_id: id.activity_id,
            });
        }

        let at_ms = now_ms.max(0).max(self.high_water_ms);
        let execution =
            self.executions
                .get(&id)
                .ok_or(ActivitySupervisorError::UnknownExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                })?;
        let schedule = match execution.phase {
            ActivityExecutionPhase::Queued => StopSchedule::Queued,
            ActivityExecutionPhase::Running
            | ActivityExecutionPhase::Waiting
            | ActivityExecutionPhase::Detached => match execution.profile {
                ActivityStopProfile::Responsive => StopSchedule::Graceful {
                    kill_deadline_ms: None,
                    reap_deadline_ms: at_ms.checked_add(NORMAL_REAP_DELAY_MS).ok_or(
                        ActivitySupervisorError::UnreachableTime {
                            activity_id: id.activity_id,
                        },
                    )?,
                },
                ActivityStopProfile::Resistant => StopSchedule::Graceful {
                    kill_deadline_ms: Some(at_ms.checked_add(RESISTANT_KILL_DELAY_MS).ok_or(
                        ActivitySupervisorError::UnreachableTime {
                            activity_id: id.activity_id,
                        },
                    )?),
                    reap_deadline_ms: at_ms.checked_add(RESISTANT_REAP_DELAY_MS).ok_or(
                        ActivitySupervisorError::UnreachableTime {
                            activity_id: id.activity_id,
                        },
                    )?,
                },
            },
            ActivityExecutionPhase::Cancelling => {
                return Err(ActivitySupervisorError::AlreadyStopping {
                    activity_id: id.activity_id,
                });
            }
            ActivityExecutionPhase::Stopped => {
                return Err(ActivitySupervisorError::NotLive {
                    activity_id: id.activity_id,
                });
            }
        };

        self.advance_to(now_ms);
        let execution =
            self.executions
                .get_mut(&id)
                .ok_or(ActivitySupervisorError::UnknownExecution {
                    activity_id: id.activity_id,
                    generation: id.generation,
                })?;
        Ok(ActivityStopAdmission {
            execution,
            at_ms,
            schedule,
        })
    }
}

impl ActivityExecution {
    fn new(
        id: ActivityExecutionId,
        snapshot: &Activity,
        start: ActivityExecutionStart,
        profile: ActivityStopProfile,
    ) -> Self {
        let phase = match start {
            ActivityExecutionStart::Queued => ActivityExecutionPhase::Queued,
            ActivityExecutionStart::Running => ActivityExecutionPhase::Running,
            ActivityExecutionStart::Waiting => ActivityExecutionPhase::Waiting,
            ActivityExecutionStart::Detached => ActivityExecutionPhase::Detached,
        };
        Self {
            id,
            snapshot: snapshot.clone(),
            phase,
            profile,
            stop_phase: ActivityStopPhase::NotRequested,
            stop_requested_ms: None,
            kill_deadline_ms: None,
            reap_deadline_ms: None,
            milestones: Vec::with_capacity(MAX_STOP_MILESTONES),
        }
    }
}
