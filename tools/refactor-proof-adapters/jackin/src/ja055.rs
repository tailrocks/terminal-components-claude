//! JA-055: capsule usage modal, chips, container info, and menus.
//!
//! Source audit: the status chips are painted but have no click hitboxes, so
//! chip clicks are recorded as inert. The usage modal opens via `Ctrl-B,u`
//! and container info via `Ctrl-B,i`.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA055_ID: &str = "JA-055";

/// Chip labels clicked in contract order.
///
/// The meter chip renders `usage ━ 72%` at every size except 120×40, where the
/// fixture status shows `Session ━ 76%` instead.
pub const JA055_CHIPS: [&str; 2] = ["72%", "PR #482"];

/// Meter chip label at 120×40.
pub const JA055_METER_WIDE: &str = "76%";

/// One size of JA-055.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja055Capture {
    /// Usage modal after `Ctrl-B,u`.
    pub usage: ObservedFrame,
    /// After `Down,PageDown,PageUp` in the modal.
    pub usage_scrolled: ObservedFrame,
    /// After `wheel(usage,100)`.
    pub usage_wheeled: ObservedFrame,
    /// Coordinate the usage wheel resolved to.
    pub usage_at: Option<(u16, u16)>,
    /// After `Esc` closes the modal.
    pub usage_closed: ObservedFrame,
    /// One frame per chip click attempt.
    pub chips: Vec<ObservedFrame>,
    /// Frames before each chip click (inertness baseline).
    pub chip_base: Vec<ObservedFrame>,
    /// Coordinates each chip resolved to.
    pub chip_at: Vec<Option<(u16, u16)>>,
    /// Container info after `Ctrl-B,i`.
    pub info: ObservedFrame,
    /// After `y` in the info view and `Esc`.
    pub info_copied: ObservedFrame,
    /// Clipboard after the info `y`.
    pub info_clipboard: Option<String>,
    /// Whether `GitHub` resolves in any menu.
    pub github_found: bool,
    /// Whether `About` resolves in any menu.
    pub about_found: bool,
}

/// Capture JA-055 at every JA-001 size.
#[must_use]
pub fn ja055_usage_chips_info() -> Vec<Ja055Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA055_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn capture_size(viewport: Viewport) -> Ja055Capture {
    let mut usage_session = fresh(viewport);
    usage_session.ctrl('b');
    usage_session.key(KeyCode::Char('u'));
    let usage = usage_session.observe("usage");
    usage_session.key(KeyCode::Down);
    usage_session.key(KeyCode::PageDown);
    usage_session.key(KeyCode::PageUp);
    let usage_scrolled = usage_session.observe("usage-scrolled");
    let usage_at = usage_session
        .find("Usage")
        .or_else(|| usage_session.find("Overview"));
    if let Some((x, y)) = usage_at {
        for _ in 0..100 {
            usage_session.wheel(Axis::V, 3, x, y);
        }
    }
    let usage_wheeled = usage_session.observe("usage-wheeled");
    usage_session.key(KeyCode::Esc);
    let usage_closed = usage_session.observe("usage-closed");

    let mut chips = Vec::new();
    let mut chip_base = Vec::new();
    let mut chip_at = Vec::new();
    let mut needles = JA055_CHIPS;
    if viewport.width == 120 {
        needles[0] = JA055_METER_WIDE;
    }
    for chip in needles {
        let mut session = fresh(viewport);
        chip_base.push(session.observe(&format!("chip-{chip}-base")));
        let at = session.find(chip);
        if let Some((x, y)) = at {
            session.click(x, y);
        }
        chip_at.push(at);
        chips.push(session.observe(&format!("chip-{chip}")));
    }

    let mut info_session = fresh(viewport);
    info_session.ctrl('b');
    info_session.key(KeyCode::Char('i'));
    let info = info_session.observe("info");
    info_session.key(KeyCode::Char('y'));
    let info_clipboard = info_session.app().world.clipboard.clone();
    info_session.key(KeyCode::Esc);
    let info_copied = info_session.observe("info-copied");

    let mut menus = fresh(viewport);
    menus.key(KeyCode::F(10));
    let mut github_found = menus.count("GitHub") > 0;
    let mut about_found = menus.count("About") > 0;
    for _ in 0..4 {
        menus.key(KeyCode::Right);
        github_found = github_found || menus.count("GitHub") > 0;
        about_found = about_found || menus.count("About") > 0;
    }

    Ja055Capture {
        usage,
        usage_scrolled,
        usage_wheeled,
        usage_at,
        usage_closed,
        chips,
        chip_base,
        chip_at,
        info,
        info_copied,
        info_clipboard,
        github_found,
        about_found,
    }
}
