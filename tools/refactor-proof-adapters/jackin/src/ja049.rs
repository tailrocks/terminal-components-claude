//! JA-049: capsule pane focus, resize, zoom, and seam drags (ground truth).
//!
//! Source audit: of the `h/j/k/l` focus chords only `h` is bound, and from the
//! leftmost pane it stays put. Pane focus moves by clicking pane bodies, so
//! the capture records the inert chords and then focuses by click.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA049_ID: &str = "JA-049";

/// Focus variants in contract order.
pub const JA049_FOCUS: [char; 4] = ['h', 'j', 'k', 'l'];

/// Left-pane transcript needles in preference order.
///
/// Pane titles are not clickable, so the needles are transcript content.
/// `cargo test` also matches the right-hand pane at wider sizes, so the left
/// needle prefers left-only lines.
pub const JA049_LEFT_NEEDLES: [&str; 3] = ["Refactor", "Retries", "MAX_ATTEMPTS"];

/// Bottom-right pane transcript needles in preference order.
pub const JA049_RIGHT_NEEDLES: [&str; 2] = ["batch 4001", "0001-record"];

fn find_pane(session: &DirectSession, needles: &[&str]) -> Option<(u16, u16)> {
    needles.iter().find_map(|needle| session.find(needle))
}

/// One size of JA-049.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja049Capture {
    /// One frame per focus-chord variant.
    pub focus: Vec<ObservedFrame>,
    /// Selected pane per focus-chord variant.
    pub focus_panes: Vec<u64>,
    /// One frame per pane-body click.
    pub click_focus: Vec<ObservedFrame>,
    /// Selected pane per click.
    pub click_panes: Vec<u64>,
    /// Coordinates each pane click resolved to.
    pub click_at: Vec<Option<(u16, u16)>>,
    /// After `Ctrl-B,h` from the bottom-right pane.
    pub focus_left: ObservedFrame,
    /// Selected pane after the `h` step.
    pub focus_left_pane: u64,
    /// After Alt-Shift arrow resizes.
    pub resized: ObservedFrame,
    /// After `Ctrl-B,z` zoom.
    pub zoomed: ObservedFrame,
    /// After the second `Ctrl-B,z` unzoom.
    pub unzoomed: ObservedFrame,
    /// Vertical seam coordinate, when resolved.
    pub seam_v: Option<(u16, u16)>,
    /// Horizontal seam coordinate, when resolved.
    pub seam_h: Option<(u16, u16)>,
    /// After dragging the vertical seam four cells.
    pub dragged_v: ObservedFrame,
    /// After dragging the horizontal seam four cells.
    pub dragged_h: ObservedFrame,
}

/// Capture JA-049 at every JA-001 size.
#[must_use]
pub fn ja049_pane_geometry() -> Vec<Ja049Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA049_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn capture_size(viewport: Viewport) -> Ja049Capture {
    let mut focus = Vec::new();
    let mut focus_panes = Vec::new();
    for key in JA049_FOCUS {
        let mut session = fresh(viewport);
        session.ctrl('b');
        session.key(KeyCode::Char(key));
        focus.push(session.observe(&format!("focus-{key}")));
        focus_panes.push(session.app().capsule.selected_pane);
    }

    let mut click_focus = Vec::new();
    let mut click_panes = Vec::new();
    let mut click_at = Vec::new();
    for (position, name) in ["click-left", "click-right", "click-left-again"]
        .into_iter()
        .enumerate()
    {
        let mut session = fresh(viewport);
        let needles = if position == 1 {
            JA049_RIGHT_NEEDLES.as_slice()
        } else {
            JA049_LEFT_NEEDLES.as_slice()
        };
        let at = find_pane(&session, needles);
        if let Some((x, y)) = at {
            session.click(x, y);
        }
        click_at.push(at);
        click_focus.push(session.observe(name));
        click_panes.push(session.app().capsule.selected_pane);
    }

    let mut left_session = fresh(viewport);
    if let Some((x, y)) = find_pane(&left_session, &JA049_RIGHT_NEEDLES) {
        left_session.click(x, y);
    }
    left_session.ctrl('b');
    left_session.key(KeyCode::Char('h'));
    let focus_left = left_session.observe("focus-left");
    let focus_left_pane = left_session.app().capsule.selected_pane;

    let mut resized_session = fresh(viewport);
    for key in [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down] {
        resized_session.key_mod(
            key,
            junie_tui::KeyModifiers::ALT | junie_tui::KeyModifiers::SHIFT,
        );
    }
    let resized = resized_session.observe("resized");

    let mut zoom_session = fresh(viewport);
    zoom_session.ctrl('b');
    zoom_session.key(KeyCode::Char('z'));
    let zoomed = zoom_session.observe("zoomed");
    zoom_session.ctrl('b');
    zoom_session.key(KeyCode::Char('z'));
    let unzoomed = zoom_session.observe("unzoomed");

    let mut drag_session = fresh(viewport);
    let seam_v = drag_session.find("││").or_else(|| drag_session.find("╮│"));
    if let Some((x, y)) = seam_v {
        drag_session.drag(
            (x, y.saturating_add(2)),
            (x.saturating_add(4), y.saturating_add(2)),
        );
    }
    let dragged_v = drag_session.observe("dragged-v");
    let seam_h = drag_session.find("╰───");
    if let Some((x, y)) = seam_h {
        drag_session.drag(
            (x.saturating_add(4), y),
            (x.saturating_add(4), y.saturating_add(4)),
        );
    }
    let dragged_h = drag_session.observe("dragged-h");

    Ja049Capture {
        focus,
        focus_panes,
        click_focus,
        click_panes,
        click_at,
        focus_left,
        focus_left_pane,
        resized,
        zoomed,
        unzoomed,
        seam_v,
        seam_h,
        dragged_v,
        dragged_h,
    }
}
