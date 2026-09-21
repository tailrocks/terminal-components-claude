//! JA-003: first-use Full intro skip guard, phase skip, returning join.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA003_ID: &str = "JA-003";
/// JA-003 sizes.
pub const JA003_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Guard sequence `K(Enter); T(45); K(Enter,Enter)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja003Guard {
    /// Immediate Enter on a fresh Full intro.
    pub stale_enter: ObservedFrame,
    /// After `T(45)`.
    pub after_ticks: ObservedFrame,
    /// First Enter after the ticks.
    pub first_skip: ObservedFrame,
    /// Second Enter after the ticks.
    pub second_skip: ObservedFrame,
}

/// Source replay of `first_use_plays_intro_then_manager_and_no_replay_when_returning`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja003Replay {
    /// After `T(45)`.
    pub after_ticks_45: ObservedFrame,
    /// After three more helper ticks.
    pub after_ticks_3: ObservedFrame,
    /// First Enter.
    pub first_enter: ObservedFrame,
    /// Second Enter.
    pub second_enter: ObservedFrame,
    /// Fresh Returning world.
    pub returning: ObservedFrame,
}

/// One size of JA-003.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja003Capture {
    /// Guard/skip sequence.
    pub guard: Ja003Guard,
    /// Intact source-test replay plus Returning join.
    pub replay: Ja003Replay,
}

/// Capture JA-003 at both listed sizes.
#[must_use]
pub fn ja003_first_use_full() -> Vec<Ja003Capture> {
    JA003_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja003Capture {
    Ja003Capture {
        guard: capture_guard(viewport),
        replay: capture_replay(viewport),
    }
}

fn capture_guard(viewport: Viewport) -> Ja003Guard {
    let mut session = DirectSession::fresh(
        JA003_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Enter);
    let stale_enter = session.observe("stale-enter");
    session.ticks(45);
    let after_ticks = session.observe("after-t45");
    session.key(KeyCode::Enter);
    let first_skip = session.observe("first-skip");
    session.key(KeyCode::Enter);
    let second_skip = session.observe("second-skip");
    Ja003Guard {
        stale_enter,
        after_ticks,
        first_skip,
        second_skip,
    }
}

fn capture_replay(viewport: Viewport) -> Ja003Replay {
    let mut session = DirectSession::fresh(
        JA003_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ticks(45);
    let after_ticks_45 = session.observe("replay-t45");
    session.ticks(3);
    let after_ticks_3 = session.observe("replay-t3");
    session.key(KeyCode::Enter);
    let first_enter = session.observe("replay-enter-1");
    session.key(KeyCode::Enter);
    let second_enter = session.observe("replay-enter-2");
    let returning = DirectSession::fresh(
        JA003_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
    .observe("replay-returning");
    Ja003Replay {
        after_ticks_45,
        after_ticks_3,
        first_enter,
        second_enter,
        returning,
    }
}
