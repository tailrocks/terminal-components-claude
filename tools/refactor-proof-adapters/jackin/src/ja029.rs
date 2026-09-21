//! JA-029: replay `settings_trust_toggle_and_failed_save_keep_edits`.

use jackin_app::{Motion, Route, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA029_ID: &str = "JA-029";
/// JA-029 sizes.
pub const JA029_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-029.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja029Capture {
    /// Trust toggled: `• 1 change`.
    pub toggled: ObservedFrame,
    /// First save attempt failed.
    pub failed: ObservedFrame,
    /// Error dismissed, draft retained.
    pub retained: ObservedFrame,
    /// Second save attempt returned to the manager.
    pub saved: ObservedFrame,
    /// Persisted trust of the first global row after the save.
    pub trust0: bool,
}

/// Replay JA-029 at both listed sizes.
#[must_use]
pub fn ja029_trust_save_retry() -> Vec<Ja029Capture> {
    JA029_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja029Capture {
    let mut session = DirectSession::fresh(
        JA029_ID,
        Scenario::HardCases,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    for _ in 0..8 {
        session.ticks(3);
        if session.app().route() == Route::Manager {
            break;
        }
        session.key(KeyCode::Enter);
    }
    session.key(KeyCode::Char('s'));
    session.key(KeyCode::Char('5'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char(' '));
    let toggled = session.observe("toggled");
    session.ctrl('s');
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
    session.ticks(20);
    let failed = session.observe("failed");
    session.key(KeyCode::Esc);
    let retained = session.observe("retained");
    session.ctrl('s');
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
    session.ticks(20);
    let saved = session.observe("saved");
    let trust0 = session
        .app()
        .world
        .global
        .trust
        .first()
        .is_some_and(|row| row.trusted);
    Ja029Capture {
        toggled,
        failed,
        retained,
        saved,
        trust0,
    }
}
