//! Fixture clock. The preview never reads wall-clock time: every duration,
//! timestamp and animation frame derives from virtual milliseconds that
//! advance only with runtime ticks, so a given `--scenario … --frame N`
//! renders the same picture every time.

/// Fixed epoch for fixture timestamps: 2026-09-03 09:14:00 local (UTC+7).
pub const EPOCH_SECS: i64 = 1_788_401_640;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    /// Virtual milliseconds since the fixture epoch.
    pub now_ms: i64,
    /// Whether ticks advance at all (`--motion paused` freezes them).
    pub running: bool,
}

impl Clock {
    pub const fn new() -> Self {
        Self {
            now_ms: 0,
            running: true,
        }
    }

    /// Advance by one runtime tick of `interval_ms`.
    pub fn advance(&mut self, interval_ms: i64) {
        if self.running {
            self.now_ms += interval_ms;
        }
    }

    /// Seconds since the fixture epoch.
    pub fn now_secs(&self) -> i64 {
        EPOCH_SECS + self.now_ms.div_euclid(1000)
    }

    /// `HH:MM` for a fixture instant, local to the fixture zone (UTC+7).
    pub fn hhmm(secs: i64) -> String {
        let local = secs + 7 * 3600;
        let day = local.rem_euclid(86_400);
        format!("{:02}:{:02}", day / 3600, (day % 3600) / 60)
    }

    /// `2026-09-03 09:14` for a fixture instant.
    pub fn stamp(secs: i64) -> String {
        let local = secs + 7 * 3600;
        let days = local.div_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!("{y:04}-{m:02}-{d:02} {}", Self::hhmm(secs))
    }

    /// Weekday short name for a fixture instant (`Mon`).
    pub fn weekday(secs: i64) -> &'static str {
        let days = (secs + 7 * 3600).div_euclid(86_400);
        // 1970-01-01 was a Thursday
        const NAMES: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
        NAMES[days.rem_euclid(7) as usize]
    }

    /// `just now`, `3 min ago`, `2 h ago`, `yesterday`, `3 d ago`.
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

    /// `in 2 h 14 min` for a future instant, `now` when passed.
    pub fn until(&self, then_secs: i64) -> String {
        let delta = then_secs - self.now_secs();
        if delta <= 0 {
            return "now".into();
        }
        format!("in {}", format_duration(delta as u64))
    }

    /// `resets in 2 h 14 min` / `resets Mon 09:00` for far instants.
    pub fn reset_label(&self, then_secs: i64) -> String {
        let delta = then_secs - self.now_secs();
        if delta <= 0 {
            "resets now".into()
        } else if delta < 86_400 {
            format!("resets in {}", format_duration(delta as u64))
        } else {
            format!(
                "resets {} {}",
                Self::weekday(then_secs),
                Self::hhmm(then_secs)
            )
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
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

/// Human duration with spaced units: `38 s`, `3 min 2 s`, `2 h 14 min`,
/// `1 d 2 h`. Two most significant units, never more.
pub fn format_duration(secs: u64) -> String {
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
        assert_eq!(Clock::stamp(EPOCH_SECS), "2026-09-03 09:14");
        assert_eq!(Clock::weekday(EPOCH_SECS), "Thu");
        assert_eq!(c.ago(EPOCH_SECS - 3 * 60), "3 min ago");
        assert_eq!(c.until(EPOCH_SECS + 8_040), "in 2 h 14 min");
        assert_eq!(c.reset_label(EPOCH_SECS + 8_040), "resets in 2 h 14 min");
        c.running = false;
        c.advance(33);
        assert_eq!(c.now_ms, 990);
    }
}
