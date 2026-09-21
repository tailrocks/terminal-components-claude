//! JA-028: settings agents/trust walk (ground truth).
//!
//! Source audit: the pinned settings screen has no Agents tab, account
//! handoff, or Trust tab strip, and `Enter` leaves settings for the manager,
//! so the contract's `4,Enter` and `5,Enter` walks depart immediately. The
//! capture records those departures verbatim, then covers the production
//! trust toggle (`5,Space`), source-open attempt (`o`), and `Ctrl-S`
//! preview → cancel path on fresh settings pages.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA028_ID: &str = "JA-028";
/// JA-028 sizes.
pub const JA028_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-028.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja028Capture {
    /// After the verbatim `s,4,Enter` agents attempt (departs settings).
    pub agents_attempt: ObservedFrame,
    /// After `Space,d,c` from wherever the attempt landed.
    pub agents_keys: ObservedFrame,
    /// After the verbatim `s,5,Enter` trust attempt (departs settings).
    pub trust_attempt: ObservedFrame,
    /// Trust toggled via `5,Space` on a fresh settings page.
    pub toggled: ObservedFrame,
    /// Whether the toggle dirtied the settings draft.
    pub toggled_dirty: bool,
    /// After the source-open `o` key.
    pub trust_walk: ObservedFrame,
    /// `Ctrl-S` preview.
    pub preview: ObservedFrame,
    /// After cancelling the preview.
    pub cancelled: ObservedFrame,
    /// Whether the cancelled draft is still dirty.
    pub cancelled_dirty: bool,
}

/// Capture JA-028 at both listed sizes.
#[must_use]
pub fn ja028_settings_trust_walk() -> Vec<Ja028Capture> {
    JA028_SIZES.into_iter().map(capture_size).collect()
}

fn open_settings(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA028_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('s'));
    session
}

fn capture_size(viewport: Viewport) -> Ja028Capture {
    let mut agents = open_settings(viewport);
    agents.key(KeyCode::Char('4'));
    agents.key(KeyCode::Enter);
    let agents_attempt = agents.observe("agents-attempt");
    agents.key(KeyCode::Char(' '));
    agents.key(KeyCode::Char('d'));
    agents.key(KeyCode::Char('c'));
    let agents_keys = agents.observe("agents-keys");

    let mut trust_attempt_session = open_settings(viewport);
    trust_attempt_session.key(KeyCode::Char('5'));
    trust_attempt_session.key(KeyCode::Enter);
    let trust_attempt = trust_attempt_session.observe("trust-attempt");

    let mut trust = open_settings(viewport);
    trust.key(KeyCode::Char('5'));
    trust.key(KeyCode::Char(' '));
    let toggled = trust.observe("toggled");
    let toggled_dirty = trust.app().settings.dirty;
    trust.key(KeyCode::Char('o'));
    let trust_walk = trust.observe("trust-walk");
    trust.ctrl('s');
    let preview = trust.observe("preview");
    trust.key(KeyCode::Esc);
    let cancelled = trust.observe("cancelled");
    let cancelled_dirty = trust.app().settings.dirty;

    Ja028Capture {
        agents_attempt,
        agents_keys,
        trust_attempt,
        toggled,
        toggled_dirty,
        trust_walk,
        preview,
        cancelled,
        cancelled_dirty,
    }
}
