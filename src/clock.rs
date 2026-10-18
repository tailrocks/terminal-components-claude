use crate::core::event::Outcome;

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Moment(u64);

impl Moment {
    pub const ZERO: Moment = Moment(0);

    pub const fn from_millis(ms: u64) -> Moment {
        Moment(ms)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }

    pub const fn since(self, earlier: Moment) -> u64 {
        self.0.saturating_sub(earlier.0)
    }

    pub const fn after(self, ms: u64) -> Moment {
        Moment(self.0.saturating_add(ms))
    }
}

pub trait Clock {
    fn now(&self) -> Moment;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallClock {
    origin: Instant,
}

impl WallClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    pub fn with_origin(origin: Instant) -> Self {
        Self { origin }
    }

    pub fn origin(&self) -> Instant {
        self.origin
    }

    pub fn now_ms(&self) -> u64 {
        self.origin.elapsed().as_millis() as u64
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimClock {
    pub now_ms: i64,

    pub running: bool,

    pub epoch_secs: i64,

    pub utc_offset_secs: i64,
}

impl SimClock {
    pub const fn new(epoch_secs: i64, utc_offset_secs: i64) -> Self {
        Self {
            now_ms: 0,
            running: true,
            epoch_secs,
            utc_offset_secs,
        }
    }

    pub fn advance(&mut self, interval_ms: i64) {
        if self.running {
            self.now_ms += interval_ms;
        }
    }

    pub fn now_secs(&self) -> i64 {
        self.epoch_secs + self.now_ms.div_euclid(1000)
    }

    fn local(&self, secs: i64) -> i64 {
        secs + self.utc_offset_secs
    }

    pub fn hhmm(&self, secs: i64) -> String {
        let day = self.local(secs).rem_euclid(86_400);
        format!("{:02}:{:02}", day / 3600, (day % 3600) / 60)
    }

    pub fn stamp(&self, secs: i64) -> String {
        let days = self.local(secs).div_euclid(86_400);
        let (y, m, d) = civil_from_days(days);
        format!("{y:04}-{m:02}-{d:02} {}", self.hhmm(secs))
    }

    pub fn weekday(&self, secs: i64) -> &'static str {
        let days = self.local(secs).div_euclid(86_400);

        const NAMES: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
        NAMES[days.rem_euclid(7) as usize]
    }

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

    pub fn reset_label(&self, then_secs: i64) -> String {
        let delta = then_secs - self.now_secs();
        if delta <= 0 {
            "resets now".into()
        } else if delta < 86_400 {
            format!(
                "resets in {}",
                crate::ui::text::duration(
                    std::time::Duration::from_secs(delta as u64),
                    crate::ui::text::DurationStyle::Spoken,
                )
            )
        } else {
            format!(
                "resets {} {}",
                self.weekday(then_secs),
                self.hhmm(then_secs)
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockStep;

impl ClockStep {
    pub const PREFIX: &'static str = "junie-tui:advance-clock:";

    pub const ENV: &'static str = "JUNIE_TUI_TEST_CLOCK";

    pub fn opted_in() -> bool {
        static OPTED_IN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *OPTED_IN.get_or_init(|| std::env::var_os(Self::ENV).is_some_and(|v| v == "1"))
    }

    pub fn request(ms: u64) -> String {
        format!("{}{ms}", Self::PREFIX)
    }

    pub fn parse(text: &str) -> Option<u64> {
        let digits = text.strip_prefix(Self::PREFIX)?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        digits.parse().ok().filter(|ms| *ms > 0)
    }
}

pub fn run_step<A: ?Sized>(
    app: &mut A,
    clock: fn(&mut A) -> &mut SimClock,
    ms: u64,
    mut tick: impl FnMut(&mut A) -> Outcome,
) -> Outcome {
    let c = clock(app);
    let was_running = c.running;
    let end = c
        .now_ms
        .saturating_add(i64::try_from(ms).unwrap_or(i64::MAX));
    c.running = true;
    let mut out = Outcome::Ignored;
    loop {
        let before = clock(app).now_ms;
        if before >= end {
            break;
        }
        out = out.or(tick(app));
        if clock(app).now_ms <= before {
            break;
        }
    }
    clock(app).running = was_running;
    out
}

impl Clock for SimClock {
    fn now(&self) -> Moment {
        Moment(self.now_ms.max(0) as u64)
    }
}

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

    const EPOCH: i64 = 1_788_401_640;
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
    fn a_clock_step_request_round_trips_and_nothing_else_parses() {
        assert_eq!(ClockStep::parse(&ClockStep::request(1_100)), Some(1_100));
        for text in [
            "",
            "junie-tui:advance-clock:",
            "junie-tui:advance-clock:0",
            "junie-tui:advance-clock:-5",
            "junie-tui:advance-clock:12ms",
            " junie-tui:advance-clock:12",
            "junie-tui:advance-clock:12\n",
            "hello",
        ] {
            assert_eq!(ClockStep::parse(text), None, "{text:?}");
        }
    }

    struct Ticking {
        clock: SimClock,
        ticks: u32,
    }

    fn ticking_clock(t: &mut Ticking) -> &mut SimClock {
        &mut t.clock
    }

    #[test]
    fn a_step_runs_a_stopped_clock_through_the_apps_ticks_and_stops_it_again() {
        let mut t = Ticking {
            clock: SimClock::new(EPOCH, ZONE),
            ticks: 0,
        };
        t.clock.running = false;
        let out = run_step(&mut t, ticking_clock, 1_100, |t| {
            t.ticks += 1;
            t.clock.advance(500);
            Outcome::Changed
        });
        assert_eq!(out, Outcome::Changed);
        assert_eq!((t.clock.now_ms, t.ticks), (1_500, 3));
        assert!(!t.clock.running, "the paused clock is stopped again");

        t.clock.advance(500);
        assert_eq!(t.clock.now_ms, 1_500);
    }

    #[test]
    fn a_tick_that_does_not_advance_the_clock_ends_the_step() {
        let mut t = Ticking {
            clock: SimClock::new(EPOCH, ZONE),
            ticks: 0,
        };
        let out = run_step(&mut t, ticking_clock, 1_000, |t| {
            t.ticks += 1;
            Outcome::Ignored
        });
        assert_eq!((out, t.ticks, t.clock.now_ms), (Outcome::Ignored, 1, 0));
        assert!(t.clock.running, "a running clock stays running");
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
