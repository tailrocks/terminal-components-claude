//! JA-054: replay the palette wheel journey, plus query, rename, and chords.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode, MouseKind};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA054_ID: &str = "JA-054";
/// JA-054 sizes.
pub const JA054_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-054.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja054Capture {
    /// Palette after `Ctrl-\`.
    pub palette: ObservedFrame,
    /// Coordinate `New tab` resolved to.
    pub new_tab_at: Option<(u16, u16)>,
    /// After wheeling down two rows below `New tab`.
    pub wheel_down: ObservedFrame,
    /// After wheeling back up.
    pub wheel_up: ObservedFrame,
    /// After `Enter` runs the kept selection.
    pub selected: ObservedFrame,
    /// After typing a no-match query on a fresh palette.
    pub no_match: ObservedFrame,
    /// After `Backspace,Backspace` and typing `rename`.
    pub rename_query: ObservedFrame,
    /// After `Enter` on the rename command.
    pub rename_prompt: ObservedFrame,
    /// After completing the rename.
    pub renamed: ObservedFrame,
    /// After `Esc` closes the palette.
    pub closed: ObservedFrame,
    /// After `Ctrl-B,Space`.
    pub space_chord: ObservedFrame,
    /// After `Ctrl-B,colon`.
    pub colon_chord: ObservedFrame,
}

/// Replay JA-054 at both listed sizes.
#[must_use]
pub fn ja054_palette() -> Vec<Ja054Capture> {
    JA054_SIZES.into_iter().map(capture_size).collect()
}

fn open_palette(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA054_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ctrl('\\');
    session
}

fn capture_size(viewport: Viewport) -> Ja054Capture {
    let mut session = open_palette(viewport);
    let palette = session.observe("palette");
    let new_tab_at = session.find("New tab");
    if let Some((x, y)) = new_tab_at {
        session.mouse(MouseKind::Wheel(Axis::V, 1), x, y.saturating_add(2));
    }
    let wheel_down = session.observe("wheel-down");
    if let Some((x, y)) = new_tab_at {
        session.mouse(MouseKind::Wheel(Axis::V, -1), x, y.saturating_add(2));
    }
    let wheel_up = session.observe("wheel-up");
    session.key(KeyCode::Enter);
    let selected = session.observe("selected");

    let mut query_session = open_palette(viewport);
    query_session.type_str("zzz-no-such-command");
    let no_match = query_session.observe("no-match");
    query_session.key(KeyCode::Backspace);
    query_session.key(KeyCode::Backspace);
    for _ in 0..17 {
        query_session.key(KeyCode::Backspace);
    }
    query_session.type_str("rename");
    let rename_query = query_session.observe("rename-query");
    query_session.key(KeyCode::Enter);
    let rename_prompt = query_session.observe("rename-prompt");
    query_session.type_str("mix-ops");
    query_session.key(KeyCode::Enter);
    let renamed = query_session.observe("renamed");
    query_session.key(KeyCode::Esc);
    let closed = query_session.observe("closed");

    let mut space_session = DirectSession::fresh(
        JA054_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    space_session.ctrl('b');
    space_session.key(KeyCode::Char(' '));
    let space_chord = space_session.observe("space-chord");

    let mut colon_session = DirectSession::fresh(
        JA054_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    colon_session.ctrl('b');
    colon_session.key(KeyCode::Char(':'));
    let colon_chord = colon_session.observe("colon-chord");

    Ja054Capture {
        palette,
        new_tab_at,
        wheel_down,
        wheel_up,
        selected,
        no_match,
        rename_query,
        rename_prompt,
        renamed,
        closed,
        space_chord,
        colon_chord,
    }
}
