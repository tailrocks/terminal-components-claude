//! Explicit elapsed time; no clock source or input cadence is embedded here.
use core::time::Duration;

/// Monotonic elapsed time from a driver- or fixture-owned origin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment(Duration);

impl Moment {
    /// The origin of elapsed time.
    pub const ZERO: Self = Self(Duration::ZERO);
    /// Construct elapsed milliseconds.
    pub const fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }
    /// Construct elapsed time without losing sub-millisecond precision.
    pub const fn from_duration(elapsed: Duration) -> Self {
        Self(elapsed)
    }
    /// Elapsed time from the origin.
    pub const fn as_duration(self) -> Duration {
        self.0
    }
    /// Add elapsed time, saturating at the representable maximum.
    #[must_use]
    pub const fn saturating_add(self, delta: Duration) -> Self {
        Self(self.0.saturating_add(delta))
    }
    /// Elapsed time since an earlier moment, or zero if it is later.
    pub const fn saturating_duration_since(self, earlier: Self) -> Duration {
        self.0.saturating_sub(earlier.0)
    }
}

/// A rejected backwards clock movement. Runtime state is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockError {
    /// The runtime's current elapsed time.
    pub current: Moment,
    /// The rejected earlier time.
    pub requested: Moment,
}
impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("monotonic time cannot move backwards")
    }
}
impl core::error::Error for ClockError {}
