//! JA-038: 1Password item picker query, scroll, wheel, and resize.

use jackin_app::screens::accounts::{OP, PROVIDER};
use jackin_app::{ACCOUNT_PICKER, Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA038_ID: &str = "JA-038";
/// JA-038 sizes.
pub const JA038_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-038.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja038Capture {
    /// Whether the 1Password picker control resolves at this size.
    pub op_reachable: bool,
    /// Item picker after typing `Anthropic` and `T(4)`.
    pub queried: ObservedFrame,
    /// After `Down,PageDown,PageUp,Home,End`.
    pub scrolled: ObservedFrame,
    /// After typing a no-match query and `T(4)`.
    pub no_match: ObservedFrame,
    /// After `Esc` unwinds one picker step.
    pub stepped_back: ObservedFrame,
    /// Whether the picker is still open after the step back.
    pub picker_open: bool,
    /// Coordinate the picker wheel resolved to.
    pub wheel_at: Option<(u16, u16)>,
    /// After `wheel(picker,100)`.
    pub wheeled: ObservedFrame,
    /// After resizing to 80×24.
    pub small: ObservedFrame,
    /// After resizing to 120×40.
    pub large: ObservedFrame,
}

/// Capture JA-038 at both listed sizes.
#[must_use]
pub fn ja038_op_item_picker() -> Vec<Ja038Capture> {
    JA038_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja038Capture {
    let mut session = DirectSession::fresh(
        JA038_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("Team");
    let _ = session.tab_to(PROVIDER);
    session.key(KeyCode::Down);
    // The narrow form never paints the picker control; only continue where it
    // resolves, otherwise the contract keys would pinball across routes.
    let op_reachable = session.find("Choose 1Password reference").is_some() && session.tab_to(OP);
    if op_reachable {
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.key(KeyCode::Enter);
        session.ticks(4);
        session.type_str("Anthropic");
        session.ticks(4);
    }
    let queried = session.observe("queried");
    if op_reachable {
        for key in [
            KeyCode::Down,
            KeyCode::PageDown,
            KeyCode::PageUp,
            KeyCode::Home,
            KeyCode::End,
        ] {
            session.key(key);
        }
    }
    let scrolled = session.observe("scrolled");
    if op_reachable {
        session.type_str("no-match");
        session.ticks(4);
    }
    let no_match = session.observe("no-match");
    if op_reachable {
        session.key(KeyCode::Esc);
    }
    let stepped_back = session.observe("stepped-back");
    let picker_open = session.is_open(ACCOUNT_PICKER);
    let wheel_at = session
        .layer_area(ACCOUNT_PICKER)
        .map(|area| (area.x.saturating_add(2), area.y.saturating_add(2)));
    if let Some((x, y)) = wheel_at {
        for _ in 0..100 {
            session.wheel(Axis::V, 3, x, y);
        }
    }
    let wheeled = session.observe("wheeled");
    session.resize(80, 24);
    let small = session.observe("small");
    session.resize(120, 40);
    let large = session.observe("large");
    Ja038Capture {
        op_reachable,
        queried,
        scrolled,
        no_match,
        stepped_back,
        picker_open,
        wheel_at,
        wheeled,
        small,
        large,
    }
}
