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
