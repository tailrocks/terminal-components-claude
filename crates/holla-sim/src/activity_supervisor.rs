//! Deterministic stop lifecycle for modeled activities.
//!
//! This module records simulated stop requests and reaping milestones. It does
//! not start processes or send operating-system signals.

use crate::domain::activity::Activity;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const NORMAL_REAP_DELAY_MS: i64 = 240;
const RESISTANT_KILL_DELAY_MS: i64 = 800;
const RESISTANT_REAP_DELAY_MS: i64 = 960;
const MAX_STOP_MILESTONES: usize = 3;

/// Identity for one registered execution of an Activity snapshot.
///
/// Generations are unique and never reused within one supervisor. Keep this
/// identity with the execution owner; drawing must not reconstruct it from an
/// Activity's display ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActivityExecutionId {
    activity_id: u32,
    generation: u64,
}

impl ActivityExecutionId {
    /// Returns the display ID of the exact Activity snapshot.
    pub const fn activity_id(self) -> u32 {
        self.activity_id
    }

    /// Returns this execution's supervisor-local generation.
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// Initial live state for one registered Activity execution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityExecutionStart {
    /// The activity has not started and can be cancelled before launch.
    Queued,
    /// The activity is running.
    Running,
    /// The activity is waiting while it remains live.
    Waiting,
    /// The activity is detached and remains supervised.
    Detached,
}

/// Current simulated ownership phase for one Activity execution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityExecutionPhase {
    /// The activity has not started and remains owned by the supervisor.
    Queued,
    /// The activity is running.
    Running,
    /// The activity is waiting while it remains live.
    Waiting,
    /// The activity is detached and remains supervised.
    Detached,
    /// A stop request is waiting for its simulated exit or reap deadline.
    Cancelling,
    /// The modeled execution reached its terminal stop state.
    Stopped,
}

impl ActivityExecutionPhase {
    /// Returns whether the supervisor still owns this execution.
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

/// Stop timing profile for one modeled execution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopProfile {
    /// Simulated TERM completes and reaps after 240 ms.
    Responsive,
    /// Simulated KILL is due at 800 ms and reaping at 960 ms.
    Resistant,
}

/// Current signal/reaping stage for one execution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopPhase {
    /// No stop request has been accepted.
    NotRequested,
    /// A simulated TERM request was recorded.
    TermSent,
    /// The resistant execution reached its simulated KILL deadline.
    KillSent,
    /// The execution reached its simulated reaping deadline.
    Reaped,
    /// A queued execution was cancelled before it started.
    CancelledBeforeStart,
}

/// One finite, timestamped milestone in an execution's stop history.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopMilestone {
    /// A simulated TERM request was accepted.
    TermSent { at_ms: i64 },
    /// A resistant execution reached its simulated KILL deadline.
    KillSent { at_ms: i64 },
    /// The execution reached its simulated reaping deadline.
    Reaped { at_ms: i64 },
    /// A queued execution was cancelled without starting.
    CancelledBeforeStart { at_ms: i64 },
}

impl ActivityStopMilestone {
    /// Returns the scheduled simulation timestamp for this milestone.
    pub const fn at_ms(self) -> i64 {
        match self {
            ActivityStopMilestone::TermSent { at_ms }
            | ActivityStopMilestone::KillSent { at_ms }
            | ActivityStopMilestone::Reaped { at_ms }
            | ActivityStopMilestone::CancelledBeforeStart { at_ms } => at_ms,
        }
    }
}

/// Result committed by an admitted stop request.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStopOutcome {
    /// A queued execution was cancelled before it started.
    CancelledBeforeStart,
    /// A simulated TERM request began the graceful stop interval.
    TermSent,
}

/// Recoverable supervisor registration or stop-admission failure.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivitySupervisorError {
    /// An Activity display ID already has a registered execution.
    DuplicateActivityId { activity_id: u32 },
    /// No unused execution generation remains in this supervisor.
    GenerationExhausted,
    /// The requested execution is not registered.
    UnknownExecution { activity_id: u32, generation: u64 },
    /// The generation does not own the current registration for this display ID.
    StaleExecution { activity_id: u32, generation: u64 },
    /// A live execution cannot be replaced before its simulated reaping point.
    ReplacementBeforeReap { activity_id: u32 },
    /// The supplied Activity differs from the exact registered snapshot.
    StaleActivitySnapshot { activity_id: u32 },
    /// The execution is no longer in a stoppable live phase.
    NotLive { activity_id: u32 },
    /// A stop request is already in progress and its deadlines cannot be reset.
    AlreadyStopping { activity_id: u32 },
    /// A required stop deadline does not fit in the supported simulation clock.
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

/// One registered immutable Activity snapshot and its separate stop lifecycle.
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
    /// Returns the stable identity assigned at registration.
    pub const fn id(&self) -> ActivityExecutionId {
        self.id
    }

    /// Returns the exact Activity snapshot bound to this execution.
    pub const fn snapshot(&self) -> &Activity {
        &self.snapshot
    }

    /// Returns the current ownership phase.
    pub const fn phase(&self) -> ActivityExecutionPhase {
        self.phase
    }

    /// Returns whether the supervisor still owns this execution.
    pub const fn is_owned(&self) -> bool {
        self.phase.is_owned()
    }

    /// Returns this execution's stop timing profile.
    pub const fn profile(&self) -> ActivityStopProfile {
        self.profile
    }

    /// Returns the current signal/reaping stage.
    pub const fn stop_phase(&self) -> ActivityStopPhase {
        self.stop_phase
    }

    /// Returns the effective simulation time at which Stop was accepted.
    pub const fn stop_requested_ms(&self) -> Option<i64> {
        self.stop_requested_ms
    }

    /// Returns the resistant profile's simulated KILL deadline, if any.
    pub const fn kill_deadline_ms(&self) -> Option<i64> {
        self.kill_deadline_ms
    }

    /// Returns the simulated reaping deadline, if Stop was accepted.
    pub const fn reap_deadline_ms(&self) -> Option<i64> {
        self.reap_deadline_ms
    }

    /// Returns the finite stop history in scheduled-time order.
    pub fn milestones(&self) -> &[ActivityStopMilestone] {
        &self.milestones
    }

    fn record(&mut self, milestone: ActivityStopMilestone) {
        if !self.milestones.contains(&milestone) && self.milestones.len() < MAX_STOP_MILESTONES {
            self.milestones.push(milestone);
        }
    }
}

/// A prevalidated stop transition that borrows its execution until commit.
///
/// Dropping this admission leaves the execution phase unchanged. The mutable
/// borrow prevents replacement between validation and commit, so an adapter
/// can first perform another fallible cancellation and commit only on success.
pub struct ActivityStopAdmission<'a> {
    execution: &'a mut ActivityExecution,
    at_ms: i64,
    schedule: StopSchedule,
}

impl ActivityStopAdmission<'_> {
    /// Commits the prevalidated stop transition without another failure point.
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

/// Deterministic simulated stop supervisor for multiple Activity executions.
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
    /// Creates an empty supervisor. The first successful registration has generation 1.
    pub fn new() -> Self {
        Self {
            next_generation: Some(1),
            high_water_ms: 0,
            executions: BTreeMap::new(),
            current_by_activity_id: BTreeMap::new(),
        }
    }

    /// Registers one exact Activity snapshot and returns its non-reused identity.
    ///
    /// Duplicate display IDs fail without consuming a generation. A stopped
    /// execution can be replaced through [`Self::replace_stopped`] to allocate
    /// a new generation for the same Activity ID.
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

    /// Registers an exact Activity snapshot that has already reached a terminal state.
    ///
    /// This publishes a stopped execution without adding a simulated stop lifecycle.
    /// Duplicate display IDs and exhausted generations leave the supervisor unchanged.
    ///
    /// # Errors
    /// Returns [`ActivitySupervisorError::DuplicateActivityId`] when the display ID is current,
    /// or [`ActivitySupervisorError::GenerationExhausted`] when no generation is available.
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

    /// Replaces a stopped execution with a newly registered snapshot and generation.
    ///
    /// The old generation remains queryable for history, but it can no longer
    /// accept stop requests once the new generation becomes current. A live or
    /// stale prior identity fails without changing either registration.
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

    /// Returns the current generation for one display ID, if it is registered.
    ///
    /// Producers use this only while publishing the exact Activity snapshot that they just
    /// registered. A view must retain the returned identity; it must not reconstruct one at draw
    /// time.
    pub fn current_execution_id(&self, activity_id: u32) -> Option<ActivityExecutionId> {
        self.current_by_activity_id.get(&activity_id).copied()
    }

    /// Returns the execution for this exact identity, if it is still registered.
    pub fn execution(&self, id: ActivityExecutionId) -> Option<&ActivityExecution> {
        self.executions.get(&id)
    }

    /// Records a natural or immediate simulated terminal event without a stop lifecycle.
    ///
    /// This releases a live execution without TERM, KILL, or reaping milestones. An explicit stop
    /// remains owned by its scheduled reaping lifecycle, even if its linked TaskRun was cancelled
    /// immediately to freeze authored output.
    ///
    /// # Errors
    /// Returns an error if the identity is unknown or stale, the Activity snapshot changed, or a
    /// queued execution has not started.
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

    /// Returns the supervisor's monotonic simulation-time high-water mark.
    pub const fn high_water_ms(&self) -> i64 {
        self.high_water_ms
    }

    /// Advances all stop lifecycles to a nondecreasing simulation time.
    ///
    /// When a coarse advance crosses multiple deadlines, every due milestone is
    /// recorded once at its scheduled timestamp and in lifecycle order.
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

    /// Validates a stop request and reserves its transition without mutating the
    /// execution phase.
    ///
    /// The returned mutable borrow keeps this exact generation and snapshot
    /// fixed until commit or drop. A caller may therefore hold the admission
    /// while it cancels linked TaskRun output, commit on success, and drop it on
    /// failure. Deadlines are checked before the admission is returned.
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
        // All identity, snapshot, state, and deadline checks have succeeded.
        // Advancing this simulation time now cannot leave a failed admission
        // partially applied, and it does not touch the target's live phase.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activity::ActivityState;

    fn fixture_activity(id: u32, name: &str, state: ActivityState) -> Activity {
        Activity::new(id, name, "~/work", state, 0)
    }

    fn register(
        supervisor: &mut ActivitySupervisor,
        id: u32,
        state: ActivityExecutionStart,
        profile: ActivityStopProfile,
    ) -> Result<(Activity, ActivityExecutionId), ActivitySupervisorError> {
        let activity = fixture_activity(id, "worker", ActivityState::Running);
        let execution = supervisor.register(&activity, state, profile)?;
        Ok((activity, execution))
    }

    #[test]
    fn terminal_registration_publishes_exact_stopped_snapshot_without_a_stop_lifecycle()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        supervisor.advance_to(37);
        let snapshot = fixture_activity(1, "finished", ActivityState::Failed);
        let profile = ActivityStopProfile::Resistant;

        let id = supervisor.register_terminal(&snapshot, profile)?;
        let execution = supervisor
            .execution(id)
            .ok_or("terminal execution missing")?;
        assert_eq!(id.activity_id(), snapshot.id);
        assert_eq!(id.generation(), 1);
        assert_eq!(supervisor.current_execution_id(snapshot.id), Some(id));
        assert_eq!(execution.id(), id);
        assert_eq!(execution.snapshot(), &snapshot);
        assert_eq!(execution.profile(), profile);
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert!(!execution.is_owned());
        assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
        assert_eq!(execution.stop_requested_ms(), None);
        assert_eq!(execution.kill_deadline_ms(), None);
        assert_eq!(execution.reap_deadline_ms(), None);
        assert!(execution.milestones().is_empty());
        assert_eq!(supervisor.high_water_ms(), 37);

        let (_, next) = register(
            &mut supervisor,
            2,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        assert_eq!(next.generation(), 2);
        Ok(())
    }

    #[test]
    fn duplicate_terminal_registration_is_atomic_and_preserves_error_precedence()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (original, original_id) = register(
            &mut supervisor,
            5,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        supervisor.advance_to(73);
        let duplicate = fixture_activity(5, "replacement", ActivityState::Failed);
        let current_before = supervisor.current_by_activity_id.clone();
        let execution_count_before = supervisor.executions.len();
        let generation_before = supervisor.next_generation;

        assert_eq!(
            supervisor.register_terminal(&duplicate, ActivityStopProfile::Resistant),
            Err(ActivitySupervisorError::DuplicateActivityId { activity_id: 5 })
        );
        assert_eq!(supervisor.current_by_activity_id, current_before);
        assert_eq!(supervisor.executions.len(), execution_count_before);
        assert_eq!(supervisor.next_generation, generation_before);
        assert_eq!(supervisor.high_water_ms(), 73);
        let execution = supervisor
            .execution(original_id)
            .ok_or("original execution missing")?;
        assert_eq!(execution.snapshot(), &original);
        assert_eq!(execution.phase(), ActivityExecutionPhase::Running);
        assert_eq!(supervisor.current_execution_id(5), Some(original_id));

        supervisor.next_generation = None;
        assert_eq!(
            supervisor.register_terminal(&duplicate, ActivityStopProfile::Resistant),
            Err(ActivitySupervisorError::DuplicateActivityId { activity_id: 5 })
        );
        assert_eq!(supervisor.current_by_activity_id, current_before);
        assert_eq!(supervisor.executions.len(), execution_count_before);
        assert_eq!(supervisor.next_generation, None);
        assert_eq!(supervisor.high_water_ms(), 73);
        assert_eq!(
            supervisor
                .execution(original_id)
                .map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Running)
        );

        supervisor.next_generation = generation_before;
        let (_, next) = register(
            &mut supervisor,
            6,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        assert_eq!(next.generation(), 2);
        Ok(())
    }

    #[test]
    fn exhausted_terminal_registration_leaves_supervisor_unchanged() -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (original, original_id) = register(
            &mut supervisor,
            7,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        supervisor.advance_to(91);
        supervisor.next_generation = None;
        let current_before = supervisor.current_by_activity_id.clone();
        let execution_count_before = supervisor.executions.len();
        let terminal = fixture_activity(8, "finished", ActivityState::Failed);

        assert_eq!(
            supervisor.register_terminal(&terminal, ActivityStopProfile::Responsive),
            Err(ActivitySupervisorError::GenerationExhausted)
        );
        assert_eq!(supervisor.current_by_activity_id, current_before);
        assert_eq!(supervisor.executions.len(), execution_count_before);
        assert_eq!(supervisor.next_generation, None);
        assert_eq!(supervisor.high_water_ms(), 91);
        assert_eq!(supervisor.current_execution_id(8), None);
        let execution = supervisor
            .execution(original_id)
            .ok_or("original execution missing")?;
        assert_eq!(execution.snapshot(), &original);
        assert_eq!(execution.phase(), ActivityExecutionPhase::Running);
        Ok(())
    }

    #[test]
    fn terminal_observation_releases_naturally_without_stop_milestones()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            1,
            ActivityExecutionStart::Waiting,
            ActivityStopProfile::Responsive,
        )?;
        supervisor.observe_terminal_exit(id, &activity)?;
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
        assert_eq!(execution.stop_requested_ms(), None);
        assert_eq!(execution.kill_deadline_ms(), None);
        assert_eq!(execution.reap_deadline_ms(), None);
        assert!(execution.milestones().is_empty());

        supervisor.observe_terminal_exit(id, &activity)?;
        assert!(
            supervisor
                .execution(id)
                .ok_or("execution missing")?
                .milestones()
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn terminal_observation_does_not_release_an_explicit_stop() -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            1,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        supervisor.admit_stop(id, &activity, 0)?.commit();

        supervisor.observe_terminal_exit(id, &activity)?;
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
        assert_eq!(execution.stop_phase(), ActivityStopPhase::TermSent);
        assert_eq!(execution.reap_deadline_ms(), Some(240));

        supervisor.advance_to(240);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Stopped)
        );
        Ok(())
    }

    #[test]
    fn responsive_stop_reaps_at_240_ms_and_remains_owned_until_then() -> Result<(), Box<dyn Error>>
    {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            1,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        let outcome = supervisor.admit_stop(id, &activity, 0)?.commit();
        assert_eq!(outcome, ActivityStopOutcome::TermSent);
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Cancelling);
        assert!(execution.is_owned());
        assert_eq!(execution.reap_deadline_ms(), Some(240));

        supervisor.advance_to(239);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Cancelling)
        );
        supervisor.advance_to(240);
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert!(!execution.is_owned());
        assert_eq!(
            execution.milestones(),
            &[
                ActivityStopMilestone::TermSent { at_ms: 0 },
                ActivityStopMilestone::Reaped { at_ms: 240 },
            ]
        );
        Ok(())
    }

    #[test]
    fn resistant_coarse_advance_records_kill_and_reap_once_at_deadlines()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            2,
            ActivityExecutionStart::Detached,
            ActivityStopProfile::Resistant,
        )?;
        supervisor.admit_stop(id, &activity, 0)?.commit();
        supervisor.advance_to(799);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::stop_phase),
            Some(ActivityStopPhase::TermSent)
        );
        supervisor.advance_to(1_000);
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert_eq!(
            execution.milestones(),
            &[
                ActivityStopMilestone::TermSent { at_ms: 0 },
                ActivityStopMilestone::KillSent { at_ms: 800 },
                ActivityStopMilestone::Reaped { at_ms: 960 },
            ]
        );
        supervisor.advance_to(1_000);
        assert_eq!(
            supervisor
                .execution(id)
                .map(ActivityExecution::milestones)
                .map(|milestones| milestones.len()),
            Some(3)
        );

        Ok(())
    }

    #[test]
    fn resistant_kill_and_reap_deadlines_are_inclusive() -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            12,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Resistant,
        )?;
        supervisor.admit_stop(id, &activity, 0)?.commit();
        supervisor.advance_to(799);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::stop_phase),
            Some(ActivityStopPhase::TermSent)
        );
        supervisor.advance_to(800);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::stop_phase),
            Some(ActivityStopPhase::KillSent)
        );
        supervisor.advance_to(959);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Cancelling)
        );
        supervisor.advance_to(960);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Stopped)
        );
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(
            execution.milestones(),
            &[
                ActivityStopMilestone::TermSent { at_ms: 0 },
                ActivityStopMilestone::KillSent { at_ms: 800 },
                ActivityStopMilestone::Reaped { at_ms: 960 },
            ]
        );
        Ok(())
    }

    #[test]
    fn queued_stop_finishes_before_start_without_term_or_reap_deadline()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            3,
            ActivityExecutionStart::Queued,
            ActivityStopProfile::Resistant,
        )?;
        let outcome = supervisor.admit_stop(id, &activity, 50)?.commit();
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(outcome, ActivityStopOutcome::CancelledBeforeStart);
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert_eq!(
            execution.stop_phase(),
            ActivityStopPhase::CancelledBeforeStart
        );
        assert_eq!(execution.kill_deadline_ms(), None);
        assert_eq!(execution.reap_deadline_ms(), None);
        assert_eq!(
            execution.milestones(),
            &[ActivityStopMilestone::CancelledBeforeStart { at_ms: 50 }]
        );
        Ok(())
    }

    #[test]
    fn repeated_stop_does_not_restart_grace_and_backward_time_is_clamped()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            4,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        supervisor.advance_to(500);
        supervisor.admit_stop(id, &activity, 400)?.commit();
        assert_eq!(
            supervisor
                .execution(id)
                .map(ActivityExecution::stop_requested_ms),
            Some(Some(500))
        );
        assert_eq!(
            supervisor.admit_stop(id, &activity, 200).err(),
            Some(ActivitySupervisorError::AlreadyStopping { activity_id: 4 })
        );
        assert_eq!(
            supervisor
                .execution(id)
                .map(ActivityExecution::reap_deadline_ms),
            Some(Some(740))
        );
        supervisor.advance_to(200);
        assert_eq!(supervisor.high_water_ms(), 500);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Cancelling)
        );
        supervisor.advance_to(740);
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Stopped)
        );
        Ok(())
    }

    #[test]
    fn admission_rejects_duplicate_and_stale_snapshots_without_mutation()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            5,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        let replacement = fixture_activity(5, "replacement", ActivityState::Running);
        assert_eq!(
            supervisor.register(
                &replacement,
                ActivityExecutionStart::Running,
                ActivityStopProfile::Responsive,
            ),
            Err(ActivitySupervisorError::DuplicateActivityId { activity_id: 5 })
        );
        assert_eq!(
            supervisor.admit_stop(id, &replacement, 0).err(),
            Some(ActivitySupervisorError::StaleActivitySnapshot { activity_id: 5 })
        );
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Running)
        );
        {
            let _admission = supervisor.admit_stop(id, &activity, 0)?;
        }
        assert_eq!(
            supervisor.execution(id).map(ActivityExecution::phase),
            Some(ActivityExecutionPhase::Running)
        );
        Ok(())
    }

    #[test]
    fn replacement_requires_reaping_and_stales_the_prior_generation() -> Result<(), Box<dyn Error>>
    {
        let mut supervisor = ActivitySupervisor::new();
        let (original, old_id) = register(
            &mut supervisor,
            11,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        let replacement = fixture_activity(11, "replacement", ActivityState::Running);
        assert_eq!(
            supervisor.replace_stopped(
                old_id,
                &replacement,
                ActivityExecutionStart::Running,
                ActivityStopProfile::Resistant,
            ),
            Err(ActivitySupervisorError::ReplacementBeforeReap { activity_id: 11 })
        );

        supervisor.admit_stop(old_id, &original, 0)?.commit();
        supervisor.advance_to(240);
        let new_id = supervisor.replace_stopped(
            old_id,
            &replacement,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Resistant,
        )?;
        assert_eq!(new_id.activity_id(), old_id.activity_id());
        assert_eq!(new_id.generation(), 2);
        assert_ne!(new_id.generation(), old_id.generation());
        assert_eq!(
            supervisor
                .execution(old_id)
                .map(ActivityExecution::snapshot),
            Some(&original)
        );
        assert_eq!(
            supervisor
                .execution(new_id)
                .map(ActivityExecution::snapshot),
            Some(&replacement)
        );
        assert_eq!(
            supervisor.admit_stop(old_id, &original, 240).err(),
            Some(ActivitySupervisorError::StaleExecution {
                activity_id: 11,
                generation: old_id.generation(),
            })
        );
        Ok(())
    }

    #[test]
    fn generation_allocation_is_checked_and_failed_registration_does_not_consume_it()
    -> Result<(), Box<dyn Error>> {
        let mut supervisor = ActivitySupervisor::new();
        let (first_activity, first) = register(
            &mut supervisor,
            6,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        let duplicate = supervisor.register(
            &first_activity,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        );
        assert!(matches!(
            duplicate,
            Err(ActivitySupervisorError::DuplicateActivityId { activity_id: 6 })
        ));
        let (_, second) = register(
            &mut supervisor,
            7,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        assert_eq!(first.generation(), 1);
        assert_eq!(second.generation(), 2);

        supervisor.next_generation = Some(u64::MAX);
        let (_, last) = register(
            &mut supervisor,
            8,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        assert_eq!(last.generation(), u64::MAX);
        assert!(matches!(
            register(
                &mut supervisor,
                9,
                ActivityExecutionStart::Running,
                ActivityStopProfile::Responsive,
            ),
            Err(ActivitySupervisorError::GenerationExhausted)
        ));
        Ok(())
    }

    #[test]
    fn unreachable_deadline_rejects_admission_without_starting_stop() -> Result<(), Box<dyn Error>>
    {
        let mut supervisor = ActivitySupervisor::new();
        let (activity, id) = register(
            &mut supervisor,
            10,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Resistant,
        )?;
        assert_eq!(
            supervisor.admit_stop(id, &activity, i64::MAX).err(),
            Some(ActivitySupervisorError::UnreachableTime { activity_id: 10 })
        );
        assert_eq!(supervisor.high_water_ms(), 0);
        let execution = supervisor.execution(id).ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Running);
        assert_eq!(execution.stop_phase(), ActivityStopPhase::NotRequested);
        assert!(execution.milestones().is_empty());
        Ok(())
    }
}
