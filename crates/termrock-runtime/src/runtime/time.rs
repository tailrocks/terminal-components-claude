use core::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment(Duration);

impl Moment {
    pub const ZERO: Self = Self(Duration::ZERO);
    pub const fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }
    pub const fn from_duration(elapsed: Duration) -> Self {
        Self(elapsed)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockError {
    pub current: Moment,
    pub requested: Moment,
}
impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("monotonic time cannot move backwards")
    }
}
impl core::error::Error for ClockError {}
