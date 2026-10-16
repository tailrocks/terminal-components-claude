//! Clocks the shell reads time from: [`WallClock`] for live applications,
//! [`SimClock`] for deterministic previews whose time advances only with
//! runtime ticks (a given `--frame N` renders the same picture every
//! time). Both implement [`Clock`]; time-based shell behaviour (the status
//! line's expiry, [`crate::shell::StatusLine`]) reads a [`Moment`] from it.

use std::time::{Duration, Instant};

/// A point on a [`Clock`]: milliseconds since that clock's origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Moment(u64);

impl Moment {
    /// The clock's origin.
    pub const ZERO: Moment = Moment(0);

    pub const fn from_millis(ms: u64) -> Moment {
        Moment(ms)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }

    /// Milliseconds from `earlier` to `self`, zero if `earlier` is later.
    pub const fn since(self, earlier: Moment) -> u64 {
        self.0.saturating_sub(earlier.0)
    }

    /// `self` moved `ms` later.
    pub const fn after(self, ms: u64) -> Moment {
        Moment(self.0.saturating_add(ms))
    }
}

/// A source of [`Moment`]s.
pub trait Clock {
    fn now(&self) -> Moment;
}

/// Real time, measured from when the clock was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallClock {
    origin: Instant,
}

impl WallClock {
    /// A clock whose origin is now.
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    /// A clock whose origin is `origin` (tests rewind it).
    pub fn with_origin(origin: Instant) -> Self {
        Self { origin }
    }

    pub fn origin(&self) -> Instant {
        self.origin
    }

    /// Milliseconds since the origin.
    pub fn now_ms(&self) -> u64 {
        self.origin.elapsed().as_millis() as u64
    }

    /// Time since the origin.
    pub fn elapsed(&self) -> Duration {
        self.origin.elapsed()
    }
}

impl Default for WallClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for WallClock {
    fn now(&self) -> Moment {
        Moment(self.now_ms())
    }
}

/// Fixture time for deterministic previews: virtual milliseconds since a
/// fixed epoch that advance only with runtime ticks. Calendar helpers
/// format instants (Unix seconds) in the fixture's zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimClock {
    /// Virtual milliseconds since the epoch.
    pub now_ms: i64,
    /// Whether ticks advance at all (`--motion paused` freezes them).
    pub running: bool,
    /// Unix seconds of virtual millisecond 0.
    pub epoch_secs: i64,
    /// Offset of the fixture zone from UTC, in seconds.
    pub utc_offset_secs: i64,
}

impl SimClock {
    /// A running clock at `epoch_secs` (Unix seconds), formatting in a zone
    /// `utc_offset_secs` from UTC.
    pub const fn new(epoch_secs: i64, utc_offset_secs: i64) -> Self {
        Self {
            now_ms: 0,
            running: true,
            epoch_secs,
            utc_offset_secs,
        }
    }

    /// Advance by one runtime tick of `interval_ms`, unless stopped.
    pub fn advance(&mut self, interval_ms: i64) {
        if self.running {
            self.now_ms += interval_ms;
        }
    }

    /// Unix seconds now.
    pub fn now_secs(&self) -> i64 {
        self.epoch_secs + self.now_ms.div_euclid(1000)
    }

    fn local(&self, secs: i64) -> i64 {
        secs + self.utc_offset_secs
    }

    /// `HH:MM` for an instant, in the fixture zone.
    pub fn hhmm(&self, secs: i64) -> String {
        let day = self.local(secs).rem_euclid(86_400);
        format!("{:02}:{:02}", day / 3600, (day % 3600) / 60)
    }

    /// `2026-09-03 09:14` for an instant, in the fixture zone.
    pub fn stamp(&self, secs: i64) -> String {
        let days = self.local(secs).div_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!("{y:04}-{m:02}-{d:02} {}", self.hhmm(secs))
    }

    /// Weekday short name for an instant (`Mon`), in the fixture zone.
    pub fn weekday(&self, secs: i64) -> &'static str {
        let days = self.local(secs).div_euclid(86_400);
        // 1970-01-01 was a Thursday
        const NAMES: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
        NAMES[days.rem_euclid(7) as usize]
    }

    /// `12 s ago`, `3 min ago`, `2 h ago`, `yesterday`, `3 d ago`.
    pub fn ago(&self, then_secs: i64) -> String {
        let delta = (self.now_secs() - then_secs).max(0) as u64;
        match delta {
            0..=59 => format!("{delta} s ago"),
            60..=3_599 => format!("{} min ago", delta / 60),
            3_600..=86_399 => format!("{} h ago", delta / 3600),
            86_400..=172_799 => "yesterday".into(),
            _ => format!("{} d ago", delta / 86_400),
        }
    }

    /// `resets in 2 h 14 min` within a day, `resets Mon 09:00` beyond it,
    /// `resets now` once past.
    pub fn reset_label(&self, then_secs: i64) -> String {
        let delta = then_secs - self.now_secs();
        if delta <= 0 {
            "resets now".into()
        } else if delta < 86_400 {
            format!("resets in {}", crate::ui::text::duration_secs(delta as u64))
        } else {
            format!(
                "resets {} {}",
                self.weekday(then_secs),
                self.hhmm(then_secs)
            )
        }
    }
}

impl Clock for SimClock {
    fn now(&self) -> Moment {
        Moment(self.now_ms.max(0) as u64)
    }
}

/// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPOCH: i64 = 1_788_401_640; // 2026-09-03 09:14 UTC+7
    const ZONE: i64 = 7 * 3600;

    #[test]
    fn sim_clock_is_pure_over_ticks() {
        let mut c = SimClock::new(EPOCH, ZONE);
        for _ in 0..30 {
            c.advance(33);
        }
        assert_eq!(c.now_ms, 990);
        assert_eq!(c.now(), Moment::from_millis(990));
        assert_eq!(c.now_secs(), EPOCH);
        assert_eq!(c.stamp(EPOCH), "2026-09-03 09:14");
        assert_eq!(c.weekday(EPOCH), "Thu");
        assert_eq!(c.ago(EPOCH - 3 * 60), "3 min ago");
        assert_eq!(c.reset_label(EPOCH + 8_040), "resets in 2 h 14 min");
        assert_eq!(c.reset_label(EPOCH + 3 * 86_400), "resets Sun 09:14");
        assert_eq!(c.reset_label(EPOCH), "resets now");
        c.running = false;
        c.advance(33);
        assert_eq!(c.now_ms, 990);
    }

    #[test]
    fn the_zone_moves_the_calendar() {
        let utc = SimClock::new(EPOCH, 0);
        assert_eq!(utc.stamp(EPOCH), "2026-09-03 02:14");
    }

    #[test]
    fn moments_measure_forward_only() {
        let a = Moment::from_millis(100);
        assert_eq!(a.after(50).since(a), 50);
        assert_eq!(a.since(a.after(50)), 0);
        let wall = WallClock::with_origin(Instant::now() - Duration::from_millis(700));
        assert!(wall.now().as_millis() >= 700);
    }
}
