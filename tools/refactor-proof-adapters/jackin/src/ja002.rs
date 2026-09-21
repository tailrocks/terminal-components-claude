//! JA-002: first-use paused intro rain phases at listed frames.

use jackin_app::rain::{
    CAPTION, INTRO_END, IntroPhase, KNOCK_START, P1_LEN, P2_LEN, PHRASES, WARP_START,
};
use jackin_app::{Motion, Scenario};

use crate::observe::{CaptureColor, DirectSession};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA002_ID: &str = "JA-002";

/// JA-002 paused first-use frames in source variant order.
#[must_use]
pub fn ja002_frames() -> [u64; 10] {
    [
        0,
        45,
        P1_LEN.saturating_sub(1),
        P1_LEN,
        P1_LEN.saturating_add(P2_LEN),
        KNOCK_START,
        KNOCK_START.saturating_add(1),
        WARP_START,
        WARP_START.saturating_add(1),
        INTRO_END.saturating_sub(1),
    ]
}

/// Production intro line observed through `App::draw` at a paused frame.
#[must_use]
pub fn production_intro_message(frame: u64) -> &'static str {
    match IntroPhase::of(frame) {
        IntroPhase::Phrases => {
            let index = usize::try_from(frame / P1_LEN).unwrap_or(0);
            PHRASES
                .get(index)
                .map_or("Stand up, operator…", |(text, _, _)| *text)
        }
        IntroPhase::Warp => "Knock, knock, operator. · opening the Construct",
        IntroPhase::Done => "Construct ready. Choose a workspace to continue.",
    }
}

/// Capture first-use paused frames at every JA-001 size, truecolor.
#[must_use]
pub fn ja002_paused_first_use() -> Vec<ObservedFrame> {
    let mut out = Vec::new();
    for viewport in JA001_SIZES {
        for frame in ja002_frames() {
            out.push(
                DirectSession::fresh(
                    JA002_ID,
                    Scenario::FirstUse,
                    Motion::Paused,
                    frame,
                    viewport,
                    CaptureColor::TrueColor,
                )
                .observe("initial"),
            );
        }
    }
    out
}

/// Knock caption owned by the production rain module.
#[must_use]
pub const fn knock_caption() -> &'static str {
    CAPTION
}
