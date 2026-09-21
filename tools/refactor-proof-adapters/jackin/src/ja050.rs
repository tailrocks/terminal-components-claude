//! JA-050: capsule typing, edit, paste, and submit per focus.
//!
//! Focus moves by clicking pane bodies (`h/j/k/l` chords cannot reach the
//! right-hand panes from the leftmost one).

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA050_ID: &str = "JA-050";

/// Focus variants in contract order: initial, right, back left.
pub const JA050_VARIANT_NAMES: [&str; 3] = ["initial", "right", "left"];

/// Left-pane transcript needles in preference order.
pub const JA050_LEFT_NEEDLES: [&str; 3] = ["Refactor", "Retries", "MAX_ATTEMPTS"];

/// Bottom-right pane transcript needles in preference order.
pub const JA050_RIGHT_NEEDLES: [&str; 2] = ["batch 4001", "0001-record"];

fn find_body(session: &DirectSession, needles: &[&str]) -> Option<(u16, u16)> {
    needles.iter().find_map(|needle| session.find(needle))
}

/// One size of JA-050.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja050Capture {
    /// One frame per variant after type/edit/paste, before submit.
    pub typed: Vec<ObservedFrame>,
    /// Whether the typed line echoed in each variant.
    pub echoed: Vec<bool>,
    /// One frame per variant after submit and `T(60)`.
    pub variants: Vec<ObservedFrame>,
    /// Selected pane per variant.
    pub panes: Vec<u64>,
}

/// Capture JA-050 at every JA-001 size.
#[must_use]
pub fn ja050_pane_typing() -> Vec<Ja050Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja050Capture {
    let mut typed = Vec::new();
    let mut echoed = Vec::new();
    let mut variants = Vec::new();
    let mut panes = Vec::new();
    for (index, name) in JA050_VARIANT_NAMES.into_iter().enumerate() {
        let mut session = DirectSession::fresh(
            JA050_ID,
            Scenario::CapsuleMulti,
            Motion::Full,
            0,
            viewport,
            CaptureColor::TrueColor,
        );
        session.ticks(60);
        if index == 1 {
            if let Some((x, y)) = find_body(&session, &JA050_RIGHT_NEEDLES) {
                session.click(x, y);
            }
        } else if index == 2 {
            if let Some((x, y)) = find_body(&session, &JA050_LEFT_NEEDLES) {
                session.click(x, y);
            }
        }
        session.type_str("hello");
        session.key(KeyCode::Backspace);
        session.paste("world");
        let frame = session.observe(&format!("typed-{name}"));
        echoed.push(frame.text.contains("hellworld"));
        typed.push(frame);
        session.key(KeyCode::Enter);
        session.ticks(60);
        variants.push(session.observe(&format!("variant-{name}")));
        panes.push(session.app().capsule.selected_pane);
    }
    Ja050Capture {
        typed,
        echoed,
        variants,
        panes,
    }
}
