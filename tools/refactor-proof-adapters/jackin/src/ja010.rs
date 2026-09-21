//! JA-010: hard-cases discovery refresh and tree scroll edges.

use jackin_app::screens::manager::TREE;
use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA010_ID: &str = "JA-010";

/// One size of JA-010.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja010Capture {
    /// Initial Reduced hard-cases draw.
    pub initial: ObservedFrame,
    /// After F5.
    pub after_f5: ObservedFrame,
    /// After T(60).
    pub after_ticks: ObservedFrame,
    /// After End.
    pub after_end: ObservedFrame,
    /// After PageUp.
    pub after_page_up: ObservedFrame,
    /// After Home.
    pub after_home: ObservedFrame,
    /// After 100 tree wheel-down events.
    pub wheel_down: ObservedFrame,
    /// After 100 tree wheel-up events.
    pub wheel_up: ObservedFrame,
}

/// Capture JA-010 at every JA-001 size.
#[must_use]
pub fn ja010_hard_cases_refresh_scroll() -> Vec<Ja010Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja010Capture {
    let mut session = DirectSession::fresh(
        JA010_ID,
        Scenario::HardCases,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let initial = session.observe("initial");
    session.key(KeyCode::F(5));
    let after_f5 = session.observe("f5");
    session.ticks(60);
    let after_ticks = session.observe("t60");
    session.key(KeyCode::End);
    let after_end = session.observe("end");
    session.key(KeyCode::PageUp);
    let after_page_up = session.observe("page-up");
    session.key(KeyCode::Home);
    let after_home = session.observe("home");
    if let Some(area) = session.area_of(TREE) {
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        for _ in 0..100 {
            session.wheel(Axis::V, 1, x, y);
        }
    }
    let wheel_down = session.observe("wheel-down");
    if let Some(area) = session.area_of(TREE) {
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        for _ in 0..100 {
            session.wheel(Axis::V, -1, x, y);
        }
    }
    let wheel_up = session.observe("wheel-up");
    Ja010Capture {
        initial,
        after_f5,
        after_ticks,
        after_end,
        after_page_up,
        after_home,
        wheel_down,
        wheel_up,
    }
}
