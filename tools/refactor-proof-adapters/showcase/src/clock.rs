//! Logical Instant clock seam for oracle deadline rows.
//!
//! Oracle source stored status/flash/busy/submit deadlines on
//! [`std::time::Instant`]. Production runtime time is [`junie_tui::Moment`].
//! This adapter owns an Instant origin and maps elapsed Instant time onto
//! `Runtime::advance_to` without treating `Input::Tick` as elapsed time and
//! without locating widgets via buffer text search.

use std::time::{Duration, Instant};

use junie_tui::Moment;

/// Controllable Instant origin plus logical now.
#[derive(Clone, Copy, Debug)]
pub struct InstantClock {
    origin: Instant,
    now: Instant,
}

impl InstantClock {
    /// Capture a real Instant origin; logical time starts at elapsed zero.
    #[must_use]
    pub fn new() -> Self {
        let origin = Instant::now();
        Self {
            origin,
            now: origin,
        }
    }

    /// The Instant origin for this capture session.
    #[must_use]
    pub const fn origin(self) -> Instant {
        self.origin
    }

    /// Logical Instant now (not wall time during the session).
    #[must_use]
    pub const fn logical_now(self) -> Instant {
        self.now
    }

    /// Elapsed Instant time from the session origin.
    #[must_use]
    pub fn elapsed(self) -> Duration {
        self.now.saturating_duration_since(self.origin)
    }

    /// Set logical Instant now to `origin + elapsed`.
    pub fn set_elapsed(&mut self, elapsed: Duration) {
        self.now = self.origin.checked_add(elapsed).unwrap_or(self.origin);
    }

    /// Runtime moment matching the logical Instant elapsed duration.
    #[must_use]
    pub fn moment(self) -> Moment {
        Moment::from_duration(self.elapsed())
    }
}

impl Default for InstantClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Oracle Instant deadline boundaries that the clock seam must express.
pub mod deadlines {
    use std::time::Duration;

    /// Button busy completion: Instant elapsed `>= 2200ms` plus a Tick.
    pub const BUSY: Duration = Duration::from_millis(2_200);
    /// Form submit completion: Instant elapsed `> 1800ms` plus a Tick.
    pub const SUBMIT: Duration = Duration::from_millis(1_800);
    /// Pressed flash: Instant elapsed `>= 140ms` on redraw.
    pub const FLASH: Duration = Duration::from_millis(140);
    /// Transient status: Instant elapsed `> 4000ms` plus a Tick.
    pub const STATUS: Duration = Duration::from_millis(4_000);
}
