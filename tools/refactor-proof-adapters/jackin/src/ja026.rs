//! JA-026: settings keys, attempted tab clicks, and focus ring.
//!
//! Source audit: the pinned settings screen is a single page (runtime mode,
//! workspace, DCO, secret policy, trust row, save). It has no tab strip, tab
//! labels, or tab controls; `5` focuses the trust row and the other aliases
//! are inert. The capture records that ground truth instead of inventing tabs.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA026_ID: &str = "JA-026";

/// Attempted settings tab labels, in contract order.
///
/// None of these labels is painted by the pinned source; the clicks resolve to
/// `None` and the frames prove the attempt changed nothing.
pub const JA026_TABS: [&str; 5] = ["General", "Mounts", "Environments", "Agents", "Trust"];

/// Tab alias keys in contract order.
pub const JA026_ALIASES: [KeyCode; 7] = [
    KeyCode::Char('1'),
    KeyCode::Char('2'),
    KeyCode::Char('3'),
    KeyCode::Char('4'),
    KeyCode::Char('5'),
    KeyCode::Char(']'),
    KeyCode::Char('['),
];

/// One size of JA-026.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja026Capture {
    /// After `s` opens settings.
    pub open: ObservedFrame,
    /// One frame per alias key.
    pub aliases: Vec<ObservedFrame>,
    /// One frame per attempted tab-label click.
    pub clicks: Vec<ObservedFrame>,
    /// Coordinates each tab label resolved to (all `None` in pinned source).
    pub click_at: Vec<Option<(u16, u16)>>,
    /// After `Enter,Tab,BackTab,Esc`.
    pub ring: ObservedFrame,
}

/// Capture JA-026 at every JA-001 size.
#[must_use]
pub fn ja026_settings_tabs() -> Vec<Ja026Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn open_settings(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA026_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('s'));
    session
}

fn capture_size(viewport: Viewport) -> Ja026Capture {
    let open = open_settings(viewport).observe("settings-open");

    let mut aliases = Vec::new();
    for (index, key) in JA026_ALIASES.into_iter().enumerate() {
        let mut session = open_settings(viewport);
        session.key(key);
        aliases.push(session.observe(&format!("alias-{index}")));
    }

    let mut clicks = Vec::new();
    let mut click_at = Vec::new();
    for tab in JA026_TABS {
        let mut session = open_settings(viewport);
        let at = session.find(tab);
        if let Some((x, y)) = at {
            session.click(x, y);
        }
        click_at.push(at);
        clicks.push(session.observe(&format!("click-{tab}")));
    }

    let mut ring_session = open_settings(viewport);
    ring_session.key(KeyCode::Enter);
    ring_session.key(KeyCode::Tab);
    ring_session.key_mod(KeyCode::Tab, junie_tui::KeyModifiers::SHIFT);
    ring_session.key(KeyCode::Esc);
    let ring = ring_session.observe("ring");

    Ja026Capture {
        open,
        aliases,
        clicks,
        click_at,
        ring,
    }
}
