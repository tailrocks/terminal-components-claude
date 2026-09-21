//! JA-047: capsule prefix capture, persistence, and follow-ups.
//!
//! Source audit: the prefix shows the row-0 `prefix…` marker and never times
//! out within 120 helper ticks in Full motion, so the capture records
//! persistence rather than a timeout. `Esc` cancels; `Ctrl-B,Ctrl-B` sends a
//! literal; `Ctrl-L` clears; `r` redraws; unknown follow-ups keep the prefix.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA047_ID: &str = "JA-047";
/// JA-047 sizes.
pub const JA047_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Ticks waited for a prefix timeout that never arrives.
pub const JA047_TIMEOUT_TICKS: usize = 120;

/// One size of JA-047.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja047Capture {
    /// After `Ctrl-B`: prefix marker visible.
    pub prefix: ObservedFrame,
    /// After `T(120)`: marker still visible.
    pub persisted: ObservedFrame,
    /// After `Ctrl-B,Esc`: prefix cancelled.
    pub cancelled: ObservedFrame,
    /// After `Ctrl-B,Ctrl-B`: literal prefix sent.
    pub literal: ObservedFrame,
    /// After `Ctrl-B,Ctrl-L`: screen cleared.
    pub cleared: ObservedFrame,
    /// After `Ctrl-B,?` (unknown follow-up).
    pub unknown: ObservedFrame,
    /// After `Ctrl-B,r` (redraw).
    pub redrawn: ObservedFrame,
}

/// Capture JA-047 at both listed sizes.
#[must_use]
pub fn ja047_prefix_keys() -> Vec<Ja047Capture> {
    JA047_SIZES.into_iter().map(capture_size).collect()
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA047_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn prefix_followup(viewport: Viewport, name: &str, then: KeyCode) -> ObservedFrame {
    let mut session = fresh(viewport);
    session.ctrl('b');
    session.key(then);
    session.observe(name)
}

fn capture_size(viewport: Viewport) -> Ja047Capture {
    let mut session = fresh(viewport);
    session.ctrl('b');
    let prefix = session.observe("prefix");
    session.ticks(JA047_TIMEOUT_TICKS);
    let persisted = session.observe("persisted");

    let cancelled = prefix_followup(viewport, "cancelled", KeyCode::Esc);
    let mut literal_session = fresh(viewport);
    literal_session.ctrl('b');
    literal_session.ctrl('b');
    let literal = literal_session.observe("literal");
    let mut clear_session = fresh(viewport);
    clear_session.ctrl('b');
    clear_session.ctrl('l');
    let cleared = clear_session.observe("cleared");
    let unknown = prefix_followup(viewport, "unknown", KeyCode::Char('?'));
    let redrawn = prefix_followup(viewport, "redrawn", KeyCode::Char('r'));

    Ja047Capture {
        prefix,
        persisted,
        cancelled,
        literal,
        cleared,
        unknown,
        redrawn,
    }
}
