#![doc = r" One activation-feedback record, aged by an immutable selected clock domain."]
use core::time::Duration;

use super::Moment;
use crate::id::{Id, PartRef};

#[doc = r" Absolute monotonic simulation time, distinct from elapsed driver time."]
#[doc = r""]
#[doc = r" ```compile_fail"]
#[doc = r" use termrock_runtime::{Cx, Moment};"]
#[doc = r" fn synchronize(cx: &mut Cx<'_>, elapsed: Moment) {"]
#[doc = r"     let _ = cx.sync_feedback_time(elapsed);"]
#[doc = r" }"]
#[doc = r" ```"]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationMoment(Duration);
impl SimulationMoment {
    #[doc = r" Simulation origin."]
    pub const ZERO: Self = Self(Duration::ZERO);
    #[doc = r" Construct nonnegative simulation milliseconds, including fixture epochs."]
    pub const fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }
    #[doc = r" Construct simulation time without discarding fractional milliseconds."]
    pub const fn from_duration(value: Duration) -> Self {
        Self(value)
    }
    #[doc = r" Absolute time in the simulation domain."]
    pub const fn as_duration(self) -> Duration {
        self.0
    }
    #[doc = r" Add a simulation duration, saturating at the representable maximum."]
    #[must_use]
    pub const fn saturating_add(self, delta: Duration) -> Self {
        Self(self.0.saturating_add(delta))
    }
    #[doc = r" Simulation duration since an earlier moment, or zero if it is later."]
    pub const fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.0.saturating_sub(earlier.0)
    }
}

#[doc = r" Select activation-feedback timing once, before initialization."]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeedbackClock {
    #[doc = r" Driver elapsed time ages feedback and schedules its deadline."]
    #[default]
    Elapsed,
    #[doc = r" Domain-owned absolute simulation time ages feedback only when synchronized."]
    Simulation {
        #[doc = r" Already-seeked fixture time before the first activation."]
        initial: SimulationMoment,
    },
}

#[doc = r" A rejected feedback synchronization; the clock and record remain unchanged."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackClockError {
    #[doc = r" This runtime selected elapsed feedback at construction."]
    WrongPolicy,
    #[doc = r" The requested simulation time precedes the current time."]
    Backwards {
        #[doc = r" Current simulation time."]
        current: SimulationMoment,
        #[doc = r" Rejected earlier time."]
        requested: SimulationMoment,
    },
}
impl core::fmt::Display for FeedbackClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::WrongPolicy => "feedback uses elapsed time, not simulation time",
            Self::Backwards { .. } => "simulation feedback time cannot move backwards",
        })
    }
}
impl core::error::Error for FeedbackClockError {}

#[doc = r" Current activation feedback; remaining duration belongs to the selected clock."]
#[doc = r" It is never an elapsed driver deadline when simulation timing was selected."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivationFeedback {
    #[doc = r" Stable control identity, possibly awaiting a future route's publication."]
    pub owner: Id,
    #[doc = r" Stable sub-region, such as a collection item."]
    pub part: PartRef,
    #[doc = r" Time remaining in the selected domain."]
    pub remaining: Duration,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    owner: Id,
    part: PartRef,
    until: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FeedbackState {
    simulation: Option<SimulationMoment>,
    record: Option<Record>,
}
impl FeedbackState {
    pub(crate) const fn new(clock: FeedbackClock) -> Self {
        Self {
            simulation: match clock {
                FeedbackClock::Elapsed => None,
                FeedbackClock::Simulation { initial } => Some(initial),
            },
            record: None,
        }
    }
    fn now(self, elapsed: Moment) -> Duration {
        self.simulation
            .map_or(elapsed.as_duration(), SimulationMoment::as_duration)
    }
    pub(crate) fn activate(
        &mut self,
        owner: Id,
        part: PartRef,
        elapsed: Moment,
        duration: Duration,
    ) {
        let now = self.now(elapsed);
        let until = now.saturating_add(duration);

        self.record =
            (self.simulation.is_none() || until > now).then_some(Record { owner, part, until });
    }
    pub(crate) fn active(self, elapsed: Moment) -> Option<ActivationFeedback> {
        self.record.map(|r| ActivationFeedback {
            owner: r.owner,
            part: r.part,
            remaining: r.until.saturating_sub(self.now(elapsed)),
        })
    }
    pub(crate) fn pressed(self) -> Option<(Id, PartRef)> {
        self.record.map(|r| (r.owner, r.part))
    }
    pub(crate) fn elapsed_deadline(self) -> Option<Moment> {
        if self.simulation.is_some() {
            None
        } else {
            self.record.map(|r| Moment::from_duration(r.until))
        }
    }
    fn expire(&mut self, now: Duration) -> bool {
        if self.record.is_some_and(|r| now >= r.until) {
            self.record = None;
            true
        } else {
            false
        }
    }
    pub(crate) fn expire_elapsed(&mut self, now: Moment) -> bool {
        self.simulation.is_none() && self.expire(now.as_duration())
    }
    pub(crate) fn sync(&mut self, requested: SimulationMoment) -> Result<bool, FeedbackClockError> {
        let Some(current) = self.simulation else {
            return Err(FeedbackClockError::WrongPolicy);
        };
        if requested < current {
            return Err(FeedbackClockError::Backwards { current, requested });
        }
        if requested == current {
            return Ok(false);
        }
        self.simulation = Some(requested);
        Ok(self.expire(requested.as_duration()))
    }
}
