//! JA-011: launch picker filtering and nested-picker Escape.

use jackin_app::{LAUNCH, Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA011_ID: &str = "JA-011";
/// JA-011 sizes.
pub const JA011_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// First-use launch-disabled replay plus returning picker variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja011Capture {
    /// First-use manager after intro, launch Enter refused.
    pub first_use_refused: ObservedFrame,
    /// Returning launch picker after clicking Launch.
    pub returning_open: ObservedFrame,
    /// After Esc on the returning picker.
    pub returning_esc: ObservedFrame,
}

/// Capture JA-011 at both listed sizes.
#[must_use]
pub fn ja011_launch_picker() -> Vec<Ja011Capture> {
    JA011_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja011Capture {
    let mut first = DirectSession::fresh(
        JA011_ID,
        Scenario::FirstUse,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    first.ticks(3);
    first.key(KeyCode::Enter);
    first.key(KeyCode::Enter);
    let first_use_refused = first.observe("first-use-launch-refused");

    let mut returning = DirectSession::fresh(
        JA011_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    returning.key(KeyCode::Home);
    returning.key(KeyCode::Down);
    if let Some((x, y)) = returning.find("payments-platform") {
        returning.click(x, y);
    }
    returning.click_id(LAUNCH);
    let returning_open = returning.observe("returning-launch-open");
    returning.key(KeyCode::Esc);
    let returning_esc = returning.observe("returning-launch-esc");
    Ja011Capture {
        first_use_refused,
        returning_open,
        returning_esc,
    }
}
