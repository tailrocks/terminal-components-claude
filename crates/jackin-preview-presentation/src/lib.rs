//! `jackin-preview-presentation`
//!
//! Formatting, badges, status colors, table projections, text transforms.

#![forbid(unsafe_code)]

pub mod prelude;
pub mod rain;

pub use prelude::{CONTINUE, DESTINATION, PreludeState, ROOT as PRELUDE_ROOT, SOURCE};
pub use rain::{
    GLITCH_PASS_TICKS, GLITCH_PASSES, HANDOFF_LEN, HandoffStage, INTRO_END, IntroPhase, IntroState,
    KNOCK_LEN, KNOCK_START, MOTION_SEED, OUT_CAPTION, OUT_WARP, OutroPhase, OutroState, P1_LEN,
    P2_LEN, P3_LEN, PHRASES, REDUCED_HOLD, Starfield, TICK_MS, Tone, WARP_START, WARP_TICKS,
    WarpCell, fill_canvas, glyph, handoff_stage, mix, pct, style,
};
