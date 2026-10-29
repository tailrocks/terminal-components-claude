use core::time::Duration;

use super::Moment;
use crate::id::{Id, PartRef};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationMoment(Duration);
impl SimulationMoment {
    pub const ZERO: Self = Self(Duration::ZERO);
    pub const fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }
    pub const fn from_duration(value: Duration) -> Self {
        Self(value)
    }
    pub const fn as_duration(self) -> Duration {
        self.0
    }
    #[must_use]
    pub const fn saturating_add(self, delta: Duration) -> Self {
        Self(self.0.saturating_add(delta))
    }
    pub const fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.0.saturating_sub(earlier.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeedbackClock {
    #[default]
    Elapsed,
    Simulation {
        initial: SimulationMoment,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackClockError {
    WrongPolicy,
    Backwards {
        current: SimulationMoment,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivationFeedback {
    pub owner: Id,
    pub part: PartRef,
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
