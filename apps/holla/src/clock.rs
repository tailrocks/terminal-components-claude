//! Fixture clock. The launcher never reads wall-clock time: every duration,
//! timestamp, spinner frame and discovery step derives from virtual
//! milliseconds that advance in fixed steps at admitted runtime deadlines, so a given
//! `--scenario … --frame N` renders the same picture every time.

// Timestamp formatting (`ago`, `stamp`, …) serves the P1 fixture world.
#![allow(dead_code)]

/// Fixed epoch for fixture timestamps: 2026-09-06 09:14:00 local (UTC+7).
pub(crate) const EPOCH_SECS: i64 = 1_788_660_840;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clock {
    /// Virtual milliseconds since the fixture epoch.
    pub(crate) now_ms: i64,
    /// Whether ticks advance at all (`--motion paused` freezes them).
    pub(crate) running: bool,
}

impl Clock {
    pub(crate) const fn new() -> Self {
        Self {
            now_ms: 0,
            running: true,
        }
    }

    /// Advance by one admitted, fixed simulation interval of `interval_ms`.
    pub(crate) fn advance(&mut self, interval_ms: i64) {
        if self.running {
            self.now_ms = self.now_ms.saturating_add(interval_ms.max(0));
        }
    }

    /// Seconds since the fixture epoch.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Division by 1000 bounds every i64 clock value before adding the fixed epoch"
    )]
    pub(crate) fn now_secs(&self) -> i64 {
        EPOCH_SECS + self.now_ms.div_euclid(1000)
    }

    /// `HH:MM` for a fixture instant, local to the fixture zone (UTC+7).
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Widened i64 seconds plus a fixed timezone offset fit i128"
    )]
    pub(crate) fn hhmm(secs: i64) -> String {
        let local = i128::from(secs) + 7 * 3600;
        let day = local.rem_euclid(86_400);
        format!("{:02}:{:02}", day / 3600, (day % 3600) / 60)
    }

    /// `2026-09-06 09:14` for a fixture instant.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Widened i64 seconds and the fixed timezone offset fit i128"
    )]
    pub(crate) fn stamp(secs: i64) -> String {
        let local = i128::from(secs) + 7 * 3600;
        let days = local.div_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!("{y:04}-{m:02}-{d:02} {}", Self::hhmm(secs))
    }

    /// `9 s ago`, `3 min ago`, `2 h ago`, `yesterday`, `3 d ago`.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "The difference of two widened i64 timestamps fits i128"
    )]
    pub(crate) fn ago(&self, then_secs: i64) -> String {
        let delta = (i128::from(self.now_secs()) - i128::from(then_secs)).max(0);
        match delta {
            0..=59 => format!("{delta} s ago"),
            60..=3_599 => format!("{} min ago", delta / 60),
            3_600..=86_399 => format!("{} h ago", delta / 3600),
            86_400..=172_799 => "yesterday".into(),
            _ => format!("{} d ago", delta / 86_400),
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// Howard Hinnant's days-to-civil algorithm.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Private calendar algorithm receives only timezone-adjusted i64 seconds divided by 86400; all intermediates fit i128"
)]
fn civil_from_days(days: i128) -> (i128, u32, u32) {
    let shifted_days = days + 719_468;
    let era = shifted_days.div_euclid(146_097);
    let cycle_day = shifted_days.rem_euclid(146_097);
    let cycle_year =
        (cycle_day - cycle_day / 1460 + cycle_day / 36_524 - cycle_day / 146_096) / 365;
    let year = cycle_year + era * 400;
    let ordinal = cycle_day - (365 * cycle_year + cycle_year / 4 - cycle_year / 100);
    let march_month = (5 * ordinal + 2) / 153;
    let day = (ordinal - (153 * march_month + 2) / 5 + 1) as u32;
    let month = if march_month < 10 {
        march_month + 3
    } else {
        march_month - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Human duration with spaced units: `38 s`, `3 min 2 s`, `2 h 14 min`,
/// `1 d 2 h`. Two most significant units, never more.
pub(crate) fn format_duration(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if d > 0 {
        if h > 0 {
            format!("{d} d {h} h")
        } else {
            format!("{d} d")
        }
    } else if h > 0 {
        if m > 0 {
            format!("{h} h {m} min")
        } else {
            format!("{h} h")
        }
    } else if m > 0 {
        if s > 0 {
            format!("{m} min {s} s")
        } else {
            format!("{m} min")
        }
    } else {
        format!("{s} s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_use_two_units() {
        assert_eq!(format_duration(0), "0 s");
        assert_eq!(format_duration(38), "38 s");
        assert_eq!(format_duration(450), "7 min 30 s");
        assert_eq!(format_duration(8_040), "2 h 14 min");
        assert_eq!(format_duration(7_200), "2 h");
        assert_eq!(format_duration(97_200), "1 d 3 h");
        assert_eq!(format_duration(259_200), "3 d");
    }

    #[test]
    fn clock_is_pure_over_ticks() {
        let mut c = Clock::new();
        for _ in 0..30 {
            c.advance(33);
        }
        assert_eq!(c.now_ms, 990);
        assert_eq!(c.now_secs(), EPOCH_SECS);
        assert_eq!(Clock::stamp(EPOCH_SECS), "2026-09-06 09:14");
        assert_eq!(c.ago(EPOCH_SECS - 3 * 60), "3 min ago");
        c.running = false;
        c.advance(33);
        assert_eq!(c.now_ms, 990);
    }
    #[test]
    fn elapsed_intervals_never_rewind_or_overflow() {
        let mut clock = Clock::new();
        clock.advance(33);
        clock.advance(-80);
        assert_eq!(clock.now_ms, 33);
        clock.advance(i64::MAX);
        assert_eq!(clock.now_ms, i64::MAX);
    }
    #[test]
    fn extreme_timestamp_views_do_not_overflow() {
        assert_eq!(Clock::hhmm(i64::MAX), "22:30");
        assert_eq!(Clock::hhmm(i64::MIN), "15:29");
        assert!(!Clock::stamp(i64::MAX).is_empty());
        assert!(!Clock::stamp(i64::MIN).is_empty());
        let clock = Clock {
            now_ms: i64::MAX,
            running: false,
        };
        assert!(clock.ago(i64::MIN).ends_with(" d ago"));
    }
}
