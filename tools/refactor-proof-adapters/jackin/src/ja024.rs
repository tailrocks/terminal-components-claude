//! JA-024: replay the hundred-role readability journey, then scroll/suffix.

use jackin_app::{Motion, Route, Scenario};
use junie_tui::{Axis, Id, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA024_ID: &str = "JA-024";

/// One size of JA-024.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja024Capture {
    /// Roles tab at `End`: the `+ Load role…` row.
    pub roles_end: ObservedFrame,
    /// Environments re-entered after `Esc,4,Enter`.
    pub reentered: ObservedFrame,
    /// After `End,PageUp,Home`.
    pub scrolled: ObservedFrame,
    /// After `wheel(config,100)`.
    pub wheel_down: ObservedFrame,
    /// After `wheel(config,-100)`.
    pub wheel_up: ObservedFrame,
    /// Coordinate the config wheel resolved to.
    pub wheel_at: Option<(u16, u16)>,
}

/// Replay JA-024 at every JA-001 size.
#[must_use]
pub fn ja024_hundred_roles_replay() -> Vec<Ja024Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn cfg_save() -> Id {
    Id::root("editor.cfg").sub("form").sub("save")
}

fn capture_size(viewport: Viewport) -> Ja024Capture {
    let mut session = DirectSession::fresh(
        JA024_ID,
        Scenario::HardCases,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    for _ in 0..8 {
        session.ticks(3);
        if session.app().route() == Route::Manager {
            break;
        }
        session.key(KeyCode::Enter);
    }
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('4'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    session.type_str("svc-01");
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    session.type_str("SVC_FLAG");
    session.key(KeyCode::Tab);
    session.key(KeyCode::Tab);
    session.key(KeyCode::Enter);
    session.type_str("on");
    session.key(KeyCode::Tab);
    let _ = session.tab_to(cfg_save());
    session.key(KeyCode::Enter);
    session.key(KeyCode::Esc);
    session.key(KeyCode::Char('3'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::End);
    let roles_end = session.observe("roles-end-load-row");
    session.key(KeyCode::Esc);
    session.key(KeyCode::Char('4'));
    session.key(KeyCode::Enter);
    let reentered = session.observe("environments-reentered");
    session.key(KeyCode::End);
    session.key(KeyCode::PageUp);
    session.key(KeyCode::Home);
    let scrolled = session.observe("scrolled");
    let wheel_at = session
        .find("Role:")
        .or_else(|| session.find("Environments"));
    if let Some((x, y)) = wheel_at {
        for _ in 0..100 {
            session.wheel(Axis::V, 3, x, y);
        }
    }
    let wheel_down = session.observe("wheel-down");
    if let Some((x, y)) = wheel_at {
        for _ in 0..100 {
            session.wheel(Axis::V, -3, x, y);
        }
    }
    let wheel_up = session.observe("wheel-up");
    Ja024Capture {
        roles_end,
        reentered,
        scrolled,
        wheel_down,
        wheel_up,
        wheel_at,
    }
}
