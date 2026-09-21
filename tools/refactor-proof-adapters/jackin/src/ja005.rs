//! JA-005: returning manager tree navigation keys.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA005_ID: &str = "JA-005";

/// Shared navigation prefix `Home,Right,Down,Tab,Esc,*,-,End,Home`.
pub const JA005_PREFIX: [KeyCode; 9] = [
    KeyCode::Home,
    KeyCode::Right,
    KeyCode::Down,
    KeyCode::Tab,
    KeyCode::Esc,
    KeyCode::Char('*'),
    KeyCode::Char('-'),
    KeyCode::End,
    KeyCode::Home,
];

/// Variant keys run from a fresh world after Home.
pub const JA005_VARIANTS: [KeyCode; 8] = [
    KeyCode::Char('j'),
    KeyCode::Char('k'),
    KeyCode::Char('l'),
    KeyCode::Char('h'),
    KeyCode::Char('g'),
    KeyCode::Char('G'),
    KeyCode::PageDown,
    KeyCode::PageUp,
];

/// Prefix plus one variant sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja005Capture {
    /// Frames after each prefix key.
    pub prefix: Vec<ObservedFrame>,
    /// Frames after each listed variant key on a fresh Home selection.
    pub variants: Vec<ObservedFrame>,
}

/// Capture JA-005 at every JA-001 size.
#[must_use]
pub fn ja005_returning_manager_keys() -> Vec<Ja005Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja005Capture {
    let mut prefix_session = DirectSession::fresh(
        JA005_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let mut prefix = Vec::new();
    for (index, key) in JA005_PREFIX.into_iter().enumerate() {
        prefix_session.key(key);
        prefix.push(prefix_session.observe(&format!("prefix-{index}")));
    }

    let mut variants = Vec::new();
    for (index, key) in JA005_VARIANTS.into_iter().enumerate() {
        let mut session = DirectSession::fresh(
            JA005_ID,
            Scenario::Returning,
            Motion::Full,
            0,
            viewport,
            CaptureColor::TrueColor,
        );
        session.key(KeyCode::Home);
        session.key(key);
        variants.push(session.observe(&format!("variant-{index}")));
    }
    Ja005Capture { prefix, variants }
}
