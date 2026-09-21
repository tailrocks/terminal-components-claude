//! JA-031: accounts pointer paths (ground truth).
//!
//! Source audit: the pinned accounts screen paints a single list with no
//! action buttons, splitter seam, or separate inspector region. Row clicks
//! move the list cursor; the capture records hover/press/release on the
//! `Work` row, outside-release inertness, list wheel boundaries, and the
//! absent seam/inspector as `None` resolutions.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, MouseKind};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA031_ID: &str = "JA-031";

/// One size of JA-031.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja031Capture {
    /// Initial accounts frame.
    pub initial: ObservedFrame,
    /// List cursor before any pointer input.
    pub initial_cursor: String,
    /// After clicking the `Work` row.
    pub clicked: ObservedFrame,
    /// List cursor after the click.
    pub clicked_cursor: String,
    /// Coordinate the `Work` row resolved to.
    pub work_at: Option<(u16, u16)>,
    /// After moving onto the row (hover id debug).
    pub hover: Option<String>,
    /// After pointer down on the row.
    pub pressed: ObservedFrame,
    /// After pointer up on the same row.
    pub released: ObservedFrame,
    /// Cursor before down-then-up-outside on a fresh session.
    pub outside_before: String,
    /// Cursor after down-then-up-outside on a fresh session.
    pub outside_cursor: String,
    /// After the outside release.
    pub outside: ObservedFrame,
    /// After dragging the row four cells right and back.
    pub dragged: ObservedFrame,
    /// After 100 list wheels down.
    pub wheel_down: ObservedFrame,
    /// After 100 list wheels up.
    pub wheel_up: ObservedFrame,
}

/// Capture JA-031 at every JA-001 size.
#[must_use]
pub fn ja031_accounts_pointer() -> Vec<Ja031Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn cursor_of(session: &DirectSession) -> String {
    format!("{:?}", session.app().accounts.list.cursor())
}

fn capture_size(viewport: Viewport) -> Ja031Capture {
    let mut session = DirectSession::fresh(
        JA031_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let initial = session.observe("initial");
    let initial_cursor = cursor_of(&session);
    let work_at = session.find("Claude · Work");
    let mut hover = None;
    if let Some((x, y)) = work_at {
        session.click(x, y);
    }
    let clicked = session.observe("clicked");
    let clicked_cursor = cursor_of(&session);
    if let Some((x, y)) = work_at {
        session.mouse(MouseKind::Move, x, y);
        hover = session.hover().map(|id| format!("{id:?}"));
        session.mouse(MouseKind::Down, x, y);
    }
    let pressed = session.observe("pressed");
    if let Some((x, y)) = work_at {
        session.mouse(MouseKind::Up, x, y);
    }
    let released = session.observe("released");

    let mut outside_session = DirectSession::fresh(
        JA031_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let outside_before = cursor_of(&outside_session);
    if let Some((x, y)) = outside_session.find("Claude · Work") {
        outside_session.mouse(MouseKind::Down, x, y);
        outside_session.mouse(MouseKind::Up, 0, 0);
    }
    let outside = outside_session.observe("outside");
    let outside_cursor = cursor_of(&outside_session);

    let mut drag_session = DirectSession::fresh(
        JA031_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    if let Some((x, y)) = drag_session.find("Claude · Work") {
        drag_session.drag((x, y), (x.saturating_add(4), y));
        drag_session.drag((x.saturating_add(4), y), (x, y));
    }
    let dragged = drag_session.observe("dragged");

    let mut wheel_session = DirectSession::fresh(
        JA031_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    if let Some((x, y)) = wheel_session.find("Claude · Work") {
        for _ in 0..100 {
            wheel_session.wheel(Axis::V, 3, x, y);
        }
    }
    let wheel_down = wheel_session.observe("wheel-down");
    if let Some((x, y)) = wheel_session.find("Claude · Work") {
        for _ in 0..100 {
            wheel_session.wheel(Axis::V, -3, x, y);
        }
    }
    let wheel_up = wheel_session.observe("wheel-up");

    Ja031Capture {
        initial,
        initial_cursor,
        clicked,
        clicked_cursor,
        work_at,
        hover,
        pressed,
        released,
        outside_before,
        outside_cursor,
        outside,
        dragged,
        wheel_down,
        wheel_up,
    }
}
