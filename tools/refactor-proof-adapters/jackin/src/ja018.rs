//! JA-018: editor dirty count, leave confirm, save preview, and return.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA018_ID: &str = "JA-018";
/// JA-018 sizes.
pub const JA018_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-018.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja018Capture {
    /// After opening editor.
    pub editor: ObservedFrame,
    /// After readonly+isolation edits.
    pub after_edits: ObservedFrame,
    /// After leave confirmation cancelled.
    pub after_leave_cancel: ObservedFrame,
    /// After save preview.
    pub after_preview: ObservedFrame,
    /// After confirming save.
    pub after_confirm: ObservedFrame,
    /// After T(20) completion.
    pub after_saved: ObservedFrame,
}

/// Capture JA-018 at both listed sizes.
#[must_use]
pub fn ja018_editor_save_replay() -> Vec<Ja018Capture> {
    JA018_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja018Capture {
    let mut session = DirectSession::fresh(
        JA018_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    let editor = session.observe("editor");
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char('r'));
    session.key(KeyCode::Char('i'));
    let after_edits = session.observe("edits");
    session.key(KeyCode::Esc);
    session.key(KeyCode::Esc);
    session.key(KeyCode::Esc);
    let after_leave_cancel = session.observe("leave-cancel");
    session.ctrl('s');
    let after_preview = session.observe("preview");
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
    let after_confirm = session.observe("confirm");
    session.ticks(20);
    let after_saved = session.observe("saved");
    Ja018Capture {
        editor,
        after_edits,
        after_leave_cancel,
        after_preview,
        after_confirm,
        after_saved,
    }
}
