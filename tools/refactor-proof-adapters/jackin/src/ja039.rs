//! JA-039: replay the usage handoff journey, then scroll and pointer suffix.

use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA039_ID: &str = "JA-039";

/// One size of JA-039.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja039Capture {
    /// Usage overview after `u`.
    pub usage: ObservedFrame,
    /// After `Down` reaches Limits.
    pub limits: ObservedFrame,
    /// After `m` hands off to Accounts.
    pub handoff: ObservedFrame,
    /// After `Esc` returns to the manager.
    pub returned: ObservedFrame,
    /// After the verbatim `u,Down,Enter` (`Enter` departs usage).
    pub enter_departure: ObservedFrame,
    /// After the suffix keys without `Enter`, plus `T(60)`.
    pub suffix: ObservedFrame,
    /// After clicking an account row.
    pub clicked: ObservedFrame,
    /// Coordinate the account click resolved to.
    pub click_at: Option<(u16, u16)>,
    /// After `wheel(list,3)`.
    pub list_wheel: ObservedFrame,
    /// After `wheel(detail,100)`.
    pub detail_wheel: ObservedFrame,
    /// Coordinates the wheels resolved to.
    pub wheel_at: (Option<(u16, u16)>, Option<(u16, u16)>),
}

/// Replay JA-039 at every JA-001 size.
#[must_use]
pub fn ja039_usage_handoff() -> Vec<Ja039Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja039Capture {
    let mut session = DirectSession::fresh(
        JA039_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('u'));
    let usage = session.observe("usage");
    session.key(KeyCode::Down);
    let limits = session.observe("limits");
    session.key(KeyCode::Char('m'));
    let handoff = session.observe("handoff");
    session.key(KeyCode::Esc);
    let returned = session.observe("returned");

    let mut departure_session = DirectSession::fresh(
        JA039_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    departure_session.key(KeyCode::Char('u'));
    departure_session.key(KeyCode::Down);
    departure_session.key(KeyCode::Enter);
    let enter_departure = departure_session.observe("enter-departure");

    let mut suffix_session = DirectSession::fresh(
        JA039_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    suffix_session.key(KeyCode::Char('u'));
    suffix_session.key(KeyCode::Down);
    suffix_session.key(KeyCode::PageDown);
    suffix_session.key(KeyCode::PageUp);
    suffix_session.key(KeyCode::Char('r'));
    suffix_session.ticks(60);
    let suffix = suffix_session.observe("suffix");
    let click_at = suffix_session
        .find("Accounts ·")
        .or_else(|| suffix_session.find("Overview"));
    if let Some((x, y)) = click_at {
        suffix_session.click(x, y);
    }
    let clicked = suffix_session.observe("clicked");
    let list_at = suffix_session
        .find("Overview")
        .or_else(|| suffix_session.find("Limits"));
    if let Some((x, y)) = list_at {
        for _ in 0..3 {
            suffix_session.wheel(Axis::V, 3, x, y);
        }
    }
    let list_wheel = suffix_session.observe("list-wheel");
    let detail_at = suffix_session
        .find("Limits")
        .or_else(|| suffix_session.find("Overview"));
    if let Some((x, y)) = detail_at {
        for _ in 0..100 {
            suffix_session.wheel(Axis::V, 3, x, y);
        }
    }
    let detail_wheel = suffix_session.observe("detail-wheel");

    Ja039Capture {
        usage,
        limits,
        handoff,
        returned,
        enter_departure,
        suffix,
        clicked,
        click_at,
        list_wheel,
        detail_wheel,
        wheel_at: (list_at, detail_at),
    }
}
