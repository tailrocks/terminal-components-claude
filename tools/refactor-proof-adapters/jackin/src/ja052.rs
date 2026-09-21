//! JA-052: replay the tab context menu journey, plus comma attempt.
//!
//! Source audit: `Ctrl-B,comma` is unbound — no rename prompt opens, so the
//! contract's rename typing falls through to global navigation.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA052_ID: &str = "JA-052";

/// One size of JA-052.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja052Capture {
    /// Tab context menu after secondary-clicking `Shell`.
    pub menu: ObservedFrame,
    /// Coordinate the `Shell` tab resolved to.
    pub shell_at: Option<(u16, u16)>,
    /// Rename prompt after `Enter`.
    pub prompt: ObservedFrame,
    /// While typing `ops` into the prompt.
    pub typing: ObservedFrame,
    /// After confirming the rename.
    pub renamed: ObservedFrame,
    /// Whether the prompt was still open after the first confirm.
    pub needed_second_confirm: bool,
    /// Close confirmation after `Ctrl-B,m,End,Enter`.
    pub close_ask: ObservedFrame,
    /// After `Esc` dismisses the close confirmation.
    pub close_dismissed: ObservedFrame,
    /// After `Ctrl-B,m,Esc` dismisses the menu.
    pub menu_dismissed: ObservedFrame,
    /// After `Ctrl-B,comma` (no prompt in the pinned source).
    pub comma_prompt: ObservedFrame,
    /// After the rename typing falls through to global navigation.
    pub comma_renamed: ObservedFrame,
}

/// Replay JA-052 at every JA-001 size.
#[must_use]
pub fn ja052_tab_menu() -> Vec<Ja052Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja052Capture {
    let mut session = DirectSession::fresh(
        JA052_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let shell_at = session.find("Shell");
    if let Some((x, y)) = shell_at {
        session.secondary(x, y);
    }
    let menu = session.observe("menu");
    session.key(KeyCode::Enter);
    let prompt = session.observe("prompt");
    session.key(KeyCode::Enter);
    session.type_str("ops");
    let typing = session.observe("typing");
    session.key(KeyCode::Enter);
    let needed_second_confirm = session.count("Change tab title") > 0;
    if needed_second_confirm {
        session.key(KeyCode::Enter);
    }
    let renamed = session.observe("renamed");
    session.ctrl('b');
    session.key(KeyCode::Char('m'));
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    let close_ask = session.observe("close-ask");
    session.key(KeyCode::Esc);
    let close_dismissed = session.observe("close-dismissed");
    session.ctrl('b');
    session.key(KeyCode::Char('m'));
    session.key(KeyCode::Esc);
    let menu_dismissed = session.observe("menu-dismissed");

    let mut comma = DirectSession::fresh(
        JA052_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    comma.ctrl('b');
    comma.key(KeyCode::Char(','));
    let comma_prompt = comma.observe("comma-prompt");
    comma.type_str("mix-ops");
    comma.key(KeyCode::Enter);
    let comma_renamed = comma.observe("comma-renamed");

    Ja052Capture {
        menu,
        shell_at,
        prompt,
        typing,
        renamed,
        needed_second_confirm,
        close_ask,
        close_dismissed,
        menu_dismissed,
        comma_prompt,
        comma_renamed,
    }
}
