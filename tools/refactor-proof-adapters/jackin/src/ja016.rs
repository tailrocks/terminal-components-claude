//! JA-016: editor tab aliases, clicks, and focus ring.

use jackin_app::screens::editor::{
    TAB_ACCOUNTS, TAB_ENVIRONMENTS, TAB_GENERAL, TAB_MOUNTS, TAB_ROLES,
};
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA016_ID: &str = "JA-016";

/// One size of JA-016.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja016Capture {
    /// After Down, e.
    pub editor: ObservedFrame,
    /// After keys 1-5 and ][.
    pub after_aliases: ObservedFrame,
    /// After clicking each editor tab.
    pub after_clicks: ObservedFrame,
    /// After Enter then Tab/BackTab then Esc.
    pub after_ring: ObservedFrame,
}

/// Capture JA-016 at every JA-001 size.
#[must_use]
pub fn ja016_editor_tabs() -> Vec<Ja016Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja016Capture {
    let mut session = DirectSession::fresh(
        JA016_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    let editor = session.observe("editor");
    for key in [
        KeyCode::Char('1'),
        KeyCode::Char('2'),
        KeyCode::Char('3'),
        KeyCode::Char('4'),
        KeyCode::Char('5'),
        KeyCode::Char(']'),
        KeyCode::Char('['),
    ] {
        session.key(key);
    }
    let after_aliases = session.observe("aliases");
    for id in [
        TAB_GENERAL,
        TAB_MOUNTS,
        TAB_ROLES,
        TAB_ENVIRONMENTS,
        TAB_ACCOUNTS,
    ] {
        session.click_id(id);
    }
    let after_clicks = session.observe("clicks");
    session.key(KeyCode::Enter);
    session.key(KeyCode::Tab);
    session.key(KeyCode::BackTab);
    session.key(KeyCode::Esc);
    let after_ring = session.observe("ring");
    Ja016Capture {
        editor,
        after_aliases,
        after_clicks,
        after_ring,
    }
}
