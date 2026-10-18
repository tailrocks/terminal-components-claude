use std::{cell::Cell, ops::Add, time::Duration};
thread_local! { static NOW: Cell<Duration> = const { Cell::new(Duration::ZERO) }; }
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(Duration);
impl Instant {
    pub fn now() -> Self { Self(NOW.with(Cell::get)) }
    pub fn elapsed(self) -> Duration { Self::now().0.saturating_sub(self.0) }
}
impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self { Self(self.0.checked_add(rhs).expect("clock overflow")) }
}
pub fn set_millis(value: u64) { NOW.with(|now| now.set(Duration::from_millis(value))); }

