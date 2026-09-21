//! JA-017: editor General name editing and control activation.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA017_ID: &str = "JA-017";
/// JA-017 sizes.
pub const JA017_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-017.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja017Capture {
    /// After opening the editor.
    pub editor: ObservedFrame,
    /// After Enter to start name editing.
    pub after_enter: ObservedFrame,
    /// After typing `x`.
    pub after_type: ObservedFrame,
    /// After Left/Home/End/Backspace/Delete/Esc.
    pub after_edit_keys: ObservedFrame,
}

/// Capture JA-017 at both listed sizes.
#[must_use]
pub fn ja017_editor_name_edit() -> Vec<Ja017Capture> {
    JA017_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja017Capture {
    let mut session = DirectSession::fresh(
        JA017_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    let editor = session.observe("editor");
    session.key(KeyCode::Enter);
    let after_enter = session.observe("enter-name");
    session.type_str("x");
    let after_type = session.observe("type-x");
    for key in [
        KeyCode::Left,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Esc,
    ] {
        session.key(key);
    }
    let after_edit_keys = session.observe("edit-keys");
    Ja017Capture {
        editor,
        after_enter,
        after_type,
        after_edit_keys,
    }
}
