//! JA-041: cockpit build log, info, and credential keys (ground truth).
//!
//! Source audit: the build log overlay is the only cockpit overlay in the
//! pinned source. `i` is inert (identical digest) and `c` falls through to
//! the global capsule route; there are no info/credential views to open.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA041_ID: &str = "JA-041";
/// JA-041 sizes.
pub const JA041_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-041.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja041Capture {
    /// Cockpit after `T(40)`.
    pub running: ObservedFrame,
    /// Build log after `b,PageUp,Home,PageDown,End`.
    pub log_scrolled: ObservedFrame,
    /// After `Esc` closes the log.
    pub log_closed: ObservedFrame,
    /// Coordinate the log click resolved to.
    pub log_at: Option<(u16, u16)>,
    /// After clicking the build log control.
    pub log_clicked: ObservedFrame,
    /// After `wheel(log,100)`.
    pub wheel_down: ObservedFrame,
    /// After `wheel(log,-100)`.
    pub wheel_up: ObservedFrame,
    /// After the inert `i` key.
    pub info: ObservedFrame,
    /// After `Esc` leaves the cockpit.
    pub info_closed: ObservedFrame,
    /// After `c` falls through to the global capsule route.
    pub credentials: ObservedFrame,
    /// After `Esc` in the capsule.
    pub closed: ObservedFrame,
}

/// Capture JA-041 at both listed sizes.
#[must_use]
pub fn ja041_cockpit_overlays() -> Vec<Ja041Capture> {
    JA041_SIZES.into_iter().map(capture_size).collect()
}

fn running_cockpit(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA041_ID,
        Scenario::LaunchRunning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ticks(40);
    session
}

fn capture_size(viewport: Viewport) -> Ja041Capture {
    let mut session = running_cockpit(viewport);
    let running = session.observe("running");
    session.key(KeyCode::Char('b'));
    session.key(KeyCode::PageUp);
    session.key(KeyCode::Home);
    session.key(KeyCode::PageDown);
    session.key(KeyCode::End);
    let log_scrolled = session.observe("log-scrolled");
    session.key(KeyCode::Esc);
    let log_closed = session.observe("log-closed");

    let mut clicked = running_cockpit(viewport);
    clicked.key(KeyCode::Char('b'));
    let log_at = clicked
        .find("Docker build")
        .or_else(|| clicked.find("build"));
    if let Some((x, y)) = log_at {
        clicked.click(x, y);
    }
    let log_clicked = clicked.observe("log-clicked");
    if let Some((x, y)) = log_at {
        for _ in 0..100 {
            clicked.wheel(Axis::V, 3, x, y);
        }
    }
    let wheel_down = clicked.observe("wheel-down");
    if let Some((x, y)) = log_at {
        for _ in 0..100 {
            clicked.wheel(Axis::V, -3, x, y);
        }
    }
    let wheel_up = clicked.observe("wheel-up");

    let mut info_session = running_cockpit(viewport);
    info_session.key(KeyCode::Char('i'));
    let info = info_session.observe("info");
    info_session.key(KeyCode::Esc);
    let info_closed = info_session.observe("info-closed");

    let mut cred_session = running_cockpit(viewport);
    cred_session.key(KeyCode::Char('c'));
    let credentials = cred_session.observe("credentials");
    cred_session.key(KeyCode::Esc);
    let closed = cred_session.observe("closed");

    Ja041Capture {
        running,
        log_scrolled,
        log_closed,
        log_at,
        log_clicked,
        wheel_down,
        wheel_up,
        info,
        info_closed,
        credentials,
        closed,
    }
}
