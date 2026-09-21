//! JA-004: reduced intro boundary and Full Ctrl-C quit.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA004_ID: &str = "JA-004";
/// JA-004 sizes.
pub const JA004_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Reduced-motion path `T(3); K(Enter)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja004Reduced {
    /// Initial Reduced draw.
    pub initial: ObservedFrame,
    /// After `T(3)`.
    pub after_ticks: ObservedFrame,
    /// After Enter.
    pub after_enter: ObservedFrame,
}

/// Full-motion Ctrl-C path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja004Quit {
    /// Initial Full draw.
    pub initial: ObservedFrame,
    /// After Ctrl-C.
    pub after_ctrl_c: ObservedFrame,
}

/// One size of JA-004.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja004Capture {
    /// Reduced intro then manager.
    pub reduced: Ja004Reduced,
    /// Full intro Ctrl-C.
    pub quit: Ja004Quit,
}

/// Capture JA-004 at both listed sizes.
#[must_use]
pub fn ja004_first_use_reduced_and_quit() -> Vec<Ja004Capture> {
    JA004_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja004Capture {
    Ja004Capture {
        reduced: capture_reduced(viewport),
        quit: capture_quit(viewport),
    }
}

fn capture_reduced(viewport: Viewport) -> Ja004Reduced {
    let mut session = DirectSession::fresh(
        JA004_ID,
        Scenario::FirstUse,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let initial = session.observe("reduced-initial");
    session.ticks(3);
    let after_ticks = session.observe("reduced-t3");
    session.key(KeyCode::Enter);
    let after_enter = session.observe("reduced-enter");
    Ja004Reduced {
        initial,
        after_ticks,
        after_enter,
    }
}

fn capture_quit(viewport: Viewport) -> Ja004Quit {
    let mut session = DirectSession::fresh(
        JA004_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let initial = session.observe("full-initial");
    session.ctrl('c');
    let after_ctrl_c = session.observe("full-ctrl-c");
    Ja004Quit {
        initial,
        after_ctrl_c,
    }
}
