//! JA-009: instance session/inspect/stop/purge keys.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA009_ID: &str = "JA-009";
/// JA-009 sizes.
pub const JA009_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One key-sequence variant from a fresh world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja009Variant {
    /// Variant name.
    pub name: &'static str,
    /// Frames after each listed action.
    pub frames: Vec<ObservedFrame>,
}

/// One size of JA-009.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja009Capture {
    /// Instance action variants.
    pub variants: Vec<Ja009Variant>,
}

/// Capture JA-009 at both listed sizes.
#[must_use]
pub fn ja009_instance_actions() -> Vec<Ja009Capture> {
    JA009_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja009Capture {
    Ja009Capture {
        variants: vec![
            capture_keys(viewport, "new-session", &[KeyCode::Char('n')]),
            capture_keys(viewport, "shell", &[KeyCode::Char('a')]),
            capture_keys(viewport, "close", &[KeyCode::Char('x')]),
            capture_keys(viewport, "inspect", &[KeyCode::Char('i')]),
            capture_keys(viewport, "stop-cancel", &[KeyCode::Char('t'), KeyCode::Esc]),
            capture_keys(
                viewport,
                "purge-cancel",
                &[KeyCode::Char('p'), KeyCode::Esc],
            ),
            capture_keys(viewport, "reconnect", &[KeyCode::Char('r')]),
            capture_confirm(viewport, "stop-confirm", KeyCode::Char('t')),
            capture_confirm(viewport, "purge-confirm", KeyCode::Char('p')),
        ],
    }
}

fn select_instance(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA009_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    if let Some((x, y)) = session.find("7f3a") {
        session.click(x, y);
    } else {
        session.key(KeyCode::Down);
        session.key(KeyCode::Down);
    }
    session
}

fn capture_keys(viewport: Viewport, name: &'static str, keys: &[KeyCode]) -> Ja009Variant {
    let mut session = select_instance(viewport);
    let mut frames = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        session.key(*key);
        frames.push(session.observe(&format!("{name}-{index}")));
    }
    Ja009Variant { name, frames }
}

fn capture_confirm(viewport: Viewport, name: &'static str, key: KeyCode) -> Ja009Variant {
    let mut session = select_instance(viewport);
    session.key(key);
    let after_key = session.observe(&format!("{name}-key"));
    session.key(KeyCode::Right);
    let after_right = session.observe(&format!("{name}-right"));
    session.key(KeyCode::Enter);
    let after_enter = session.observe(&format!("{name}-enter"));
    session.ticks(20);
    let after_ticks = session.observe(&format!("{name}-t20"));
    Ja009Variant {
        name,
        frames: vec![after_key, after_right, after_enter, after_ticks],
    }
}
