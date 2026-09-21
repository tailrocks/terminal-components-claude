//! JA-044: launch cancel, quit-confirmation, and detach variants (ground truth).
//!
//! Source audit: `Ctrl-C` and `d` are inert in the cockpit — the launch run is
//! never cancelled and no detach occurs. `Ctrl-Q` is likewise inert, so the
//! `ctrl-q-esc` variant's `Esc` simply leaves the cockpit for the manager.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA044_ID: &str = "JA-044";
/// JA-044 sizes.
pub const JA044_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Variant names in contract order.
pub const JA044_VARIANT_NAMES: [&str; 4] = ["ctrl-c", "ctrl-q-esc", "ctrl-q-confirm", "detach"];

/// One size of JA-044.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja044Capture {
    /// One frame per variant after its keys and settling ticks.
    pub variants: Vec<ObservedFrame>,
    /// Whether each variant's launch run is cancelled.
    pub cancelled: Vec<bool>,
}

/// Variant control chord plus trailing keys.
fn variant_keys() -> Vec<(Option<char>, Vec<KeyCode>)> {
    vec![
        (Some('c'), Vec::new()),
        (Some('q'), vec![KeyCode::Esc]),
        (Some('q'), vec![KeyCode::Right, KeyCode::Enter]),
        (None, vec![KeyCode::Char('d')]),
    ]
}

/// Capture JA-044 at both listed sizes.
///
/// The `ctrl-c`/`ctrl-q` variants use control chords; the others are plain keys.
#[must_use]
pub fn ja044_launch_cancel() -> Vec<Ja044Capture> {
    JA044_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja044Capture {
    let mut variants = Vec::new();
    let mut cancelled = Vec::new();
    for (index, (chord, keys)) in variant_keys().into_iter().enumerate() {
        let mut session = DirectSession::fresh(
            JA044_ID,
            Scenario::LaunchRunning,
            Motion::Full,
            0,
            viewport,
            CaptureColor::TrueColor,
        );
        session.ticks(10);
        if let Some(c) = chord {
            session.ctrl(c);
        }
        for key in keys {
            session.key(key);
        }
        session.ticks(20);
        variants.push(session.observe(&format!("variant-{index}")));
        cancelled.push(session.app().launch().is_some_and(|run| run.cancelled));
    }
    Ja044Capture {
        variants,
        cancelled,
    }
}
