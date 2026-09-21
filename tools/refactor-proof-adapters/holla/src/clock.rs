//! Oracle tick contract.
//!
//! The frozen oracle advances `App::for_scenario` by frame *ticks* through
//! its scripted timeline (`src/bin/holla/app.rs` in the `visual-baseline`
//! tag), with an 80 ms simulation quantum (`world.tick`, `now_ms / 80`,
//! 80 ms animating interval). Current production `App::for_scenario` instead
//! seeks a millisecond target (`world.seek(frame_ms)`, 80 ms rounding).
//! The adapter therefore converts every oracle tick ordinal with
//! [`tick_to_ms`] before construction. It never passes a tick ordinal as if
//! it were already milliseconds, and never treats a frame ordinal as
//! milliseconds.

/// Virtual milliseconds advanced by one oracle simulation tick.
pub const TICK_MS: u64 = 80;

/// Oracle fixture epoch: 2026-09-10 08:41:00 local (UTC+7).
///
/// This is the frozen oracle's `clock.rs::EPOCH_SECS`, recorded here as an
/// oracle identity. It is distinct from the current production epoch and is
/// never substituted into production construction.
pub const ORACLE_EPOCH_SECS: i64 = 1_789_004_460;

/// One admitted runtime tick advances this many simulation milliseconds,
/// even when the delivery cadence is 200 ms.
pub const ORACLE_ADMITTED_TICK_ADVANCE_MS: i64 = 80;

/// Convert an oracle tick ordinal into the millisecond argument that
/// production `App::for_scenario` interprets as a seek target.
#[must_use]
pub const fn tick_to_ms(tick: u64) -> u64 {
    tick.saturating_mul(TICK_MS)
}

/// Oracle tick ordinal implied by a virtual millisecond clock.
#[must_use]
pub fn ms_to_tick(now_ms: i64) -> u64 {
    u64::try_from(now_ms.max(0)).unwrap_or(0) / TICK_MS
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn tick_40_is_3200_ms_not_40_ms() {
        assert_eq!(tick_to_ms(0), 0);
        assert_eq!(tick_to_ms(1), 80);
        assert_eq!(tick_to_ms(40), 3_200);
        assert_ne!(tick_to_ms(40), 40);
        assert_eq!(ms_to_tick(3_200), 40);
        assert_eq!(ORACLE_EPOCH_SECS, 1_789_004_460);
        assert_eq!(ORACLE_ADMITTED_TICK_ADVANCE_MS, 80);
    }
}
