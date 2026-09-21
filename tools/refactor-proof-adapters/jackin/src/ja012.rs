//! JA-012: prelude five-step create then pending editor.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA012_ID: &str = "JA-012";

/// One size of JA-012.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja012Capture {
    /// After Char('n').
    pub after_n: ObservedFrame,
    /// After End (production new-workspace chord).
    pub after_end: ObservedFrame,
    /// After Enter into the prelude.
    pub after_enter: ObservedFrame,
    /// After Space chooses a source.
    pub after_space: ObservedFrame,
    /// After first continue Enter.
    pub after_continue_1: ObservedFrame,
    /// After second continue Enter.
    pub after_continue_2: ObservedFrame,
    /// After third continue Enter (pending editor).
    pub after_continue_3: ObservedFrame,
}

/// Capture JA-012 at every JA-001 size.
#[must_use]
pub fn ja012_prelude_pending_editor() -> Vec<Ja012Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja012Capture {
    let mut session = DirectSession::fresh(
        JA012_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('n'));
    let after_n = session.observe("key-n");
    session.key(KeyCode::End);
    let after_end = session.observe("end");
    session.key(KeyCode::Enter);
    let after_enter = session.observe("enter-prelude");
    session.key(KeyCode::Char(' '));
    let after_space = session.observe("space");
    session.key(KeyCode::Enter);
    let after_continue_1 = session.observe("enter-1");
    session.key(KeyCode::Enter);
    let after_continue_2 = session.observe("enter-2");
    session.key(KeyCode::Enter);
    let after_continue_3 = session.observe("enter-3");
    Ja012Capture {
        after_n,
        after_end,
        after_enter,
        after_space,
        after_continue_1,
        after_continue_2,
        after_continue_3,
    }
}
