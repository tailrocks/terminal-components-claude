//! JA-014: prelude duplicate/empty name and cancel.

use jackin_app::{Motion, Route, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA014_ID: &str = "JA-014";
/// JA-014 sizes.
pub const JA014_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Duplicate-name replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja014Duplicate {
    /// After reaching the name step.
    pub name_step: ObservedFrame,
    /// After submitting the duplicate name.
    pub refused: ObservedFrame,
    /// After Esc rewind to Manager.
    pub cancelled: ObservedFrame,
}

/// Empty-name variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja014Empty {
    /// After clearing the name field.
    pub cleared: ObservedFrame,
    /// After submitting the empty name.
    pub submitted: ObservedFrame,
}

/// One size of JA-014.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja014Capture {
    /// Duplicate-name source replay.
    pub duplicate: Ja014Duplicate,
    /// Empty-name variant.
    pub empty: Ja014Empty,
}

/// Capture JA-014 at both listed sizes.
#[must_use]
pub fn ja014_prelude_name_validation() -> Vec<Ja014Capture> {
    JA014_SIZES.into_iter().map(capture_size).collect()
}

fn open_prelude(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA014_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    session
}

fn capture_size(viewport: Viewport) -> Ja014Capture {
    Ja014Capture {
        duplicate: capture_duplicate(viewport),
        empty: capture_empty(viewport),
    }
}

fn capture_duplicate(viewport: Viewport) -> Ja014Duplicate {
    let mut session = open_prelude(viewport);
    session.key(KeyCode::Backspace);
    session.key(KeyCode::Down);
    session.key(KeyCode::Char(' '));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    let name_step = session.observe("duplicate-name-step");
    session.key(KeyCode::Enter);
    let refused = session.observe("duplicate-refused");
    for _ in 0..8 {
        if session.app().route() != Route::Prelude {
            break;
        }
        session.key(KeyCode::Esc);
    }
    let cancelled = session.observe("duplicate-cancelled");
    Ja014Duplicate {
        name_step,
        refused,
        cancelled,
    }
}

fn capture_empty(viewport: Viewport) -> Ja014Empty {
    let mut session = open_prelude(viewport);
    session.key(KeyCode::Char(' '));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    for _ in 0..24 {
        session.key(KeyCode::Backspace);
    }
    let cleared = session.observe("empty-cleared");
    session.key(KeyCode::Enter);
    let submitted = session.observe("empty-submitted");
    Ja014Empty { cleared, submitted }
}
