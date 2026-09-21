//! JA-015: prelude browser navigation, invalid path, paste, and wheel.

use jackin_app::screens::prelude::SOURCE;
use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA015_ID: &str = "JA-015";

/// One size of JA-015.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja015Capture {
    /// After opening prelude.
    pub prelude: ObservedFrame,
    /// After Down/Right/Left/Backspace/Home/End.
    pub after_browser_keys: ObservedFrame,
    /// After typing `/missing`.
    pub after_missing: ObservedFrame,
    /// After Enter commit.
    pub after_commit: ObservedFrame,
    /// After paste of `/work/payments-platform`.
    pub after_paste: ObservedFrame,
    /// After 100 browser wheel events.
    pub after_wheel: ObservedFrame,
    /// After Esc.
    pub after_esc: ObservedFrame,
}

/// Capture JA-015 at every JA-001 size.
#[must_use]
pub fn ja015_prelude_browser() -> Vec<Ja015Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja015Capture {
    let mut session = DirectSession::fresh(
        JA015_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('n'));
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    let prelude = session.observe("prelude");
    for key in [
        KeyCode::Down,
        KeyCode::Right,
        KeyCode::Left,
        KeyCode::Backspace,
        KeyCode::Home,
        KeyCode::End,
    ] {
        session.key(key);
    }
    let after_browser_keys = session.observe("browser-keys");
    session.type_str("/missing");
    let after_missing = session.observe("missing");
    session.key(KeyCode::Enter);
    let after_commit = session.observe("commit");
    session.paste("/work/payments-platform");
    let after_paste = session.observe("paste");
    if let Some(area) = session.area_of(SOURCE) {
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        for _ in 0..100 {
            session.wheel(Axis::V, 1, x, y);
        }
    }
    let after_wheel = session.observe("wheel");
    session.key(KeyCode::Esc);
    let after_esc = session.observe("esc");
    Ja015Capture {
        prelude,
        after_browser_keys,
        after_missing,
        after_commit,
        after_paste,
        after_wheel,
        after_esc,
    }
}
