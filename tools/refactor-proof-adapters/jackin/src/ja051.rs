//! JA-051: capsule scrollback, wheels, drag-select, and copy.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA051_ID: &str = "JA-051";

/// Copy/select needles in preference order (narrow panes clip the first).
pub const JA051_NEEDLES: [&str; 3] = ["Refactor", "Retries", "policy"];

/// One size of JA-051.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja051Capture {
    /// Needle used for the select/copy checks at this size.
    pub needle: Option<String>,
    /// After `PageUp,PageUp,Home`.
    pub scrolled_up: ObservedFrame,
    /// Active-pane wheel coordinate.
    pub active_at: Option<(u16, u16)>,
    /// Inactive-pane wheel coordinate.
    pub inactive_at: Option<(u16, u16)>,
    /// After `wheel(active,-100)`.
    pub active_wheel: ObservedFrame,
    /// After `wheel(inactive,3)`.
    pub inactive_wheel: ObservedFrame,
    /// After `PageDown,End`.
    pub scrolled_down: ObservedFrame,
    /// After dragging across `Refactor` eight cells right.
    pub dragged: ObservedFrame,
    /// Clipboard after the drag.
    pub drag_clipboard: String,
    /// After double-clicking the word.
    pub double_clicked: ObservedFrame,
    /// Clipboard after double-click plus `y,End`.
    pub word_clipboard: String,
}

/// Capture JA-051 at every JA-001 size.
#[must_use]
pub fn ja051_scrollback_copy() -> Vec<Ja051Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja051Capture {
    let mut session = DirectSession::fresh(
        JA051_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ticks(60);
    session.key(KeyCode::PageUp);
    session.key(KeyCode::PageUp);
    session.key(KeyCode::Home);
    let scrolled_up = session.observe("scrolled-up");
    let active_at = session
        .find("Refactor")
        .or_else(|| session.find("cargo test"));
    let inactive_at = session
        .find("batch 4001")
        .or_else(|| session.find("0001-record"));
    if let Some((x, y)) = active_at {
        for _ in 0..100 {
            session.wheel(Axis::V, -3, x, y);
        }
    }
    let active_wheel = session.observe("active-wheel");
    if let Some((x, y)) = inactive_at {
        for _ in 0..3 {
            session.wheel(Axis::V, 3, x, y);
        }
    }
    let inactive_wheel = session.observe("inactive-wheel");
    session.key(KeyCode::PageDown);
    session.key(KeyCode::End);
    let scrolled_down = session.observe("scrolled-down");

    let mut copy_session = DirectSession::fresh(
        JA051_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    copy_session.ticks(60);
    let needle = JA051_NEEDLES
        .into_iter()
        .find(|needle| copy_session.find(needle).is_some())
        .map(String::from);
    if let Some(word) = needle.as_deref() {
        if let Some((x, y)) = copy_session.find(word) {
            let width = u16::try_from(word.len()).unwrap_or(u16::MAX);
            copy_session.drag((x, y), (x.saturating_add(width), y));
        }
    }
    let dragged = copy_session.observe("dragged");
    let drag_clipboard = copy_session
        .app()
        .world
        .clipboard
        .clone()
        .unwrap_or_default();
    if let Some(word) = needle.as_deref() {
        if let Some((x, y)) = copy_session.find(word) {
            copy_session.double_click(x, y);
        }
    }
    let double_clicked = copy_session.observe("double-clicked");
    copy_session.key(KeyCode::Char('y'));
    copy_session.key(KeyCode::End);
    let word_clipboard = copy_session
        .app()
        .world
        .clipboard
        .clone()
        .unwrap_or_default();

    Ja051Capture {
        needle,
        scrolled_up,
        active_at,
        inactive_at,
        active_wheel,
        inactive_wheel,
        scrolled_down,
        dragged,
        drag_clipboard,
        double_clicked,
        word_clipboard,
    }
}
