//! JA-036: replay the hard-cases refresh journey, then per-account refresh.

use jackin_app::{Motion, Route, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA036_ID: &str = "JA-036";

/// Refresh variant cursor depths: account rows only (Overview and provider
/// headers accept `r` without producing a refresh status).
pub const JA036_VARIANT_DOWNS: [usize; 6] = [2, 3, 4, 6, 7, 9];

/// One size of JA-036.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja036Capture {
    /// Accounts help (`Credential sources`).
    pub help: ObservedFrame,
    /// After refreshing with the broker unreachable.
    pub refresh: ObservedFrame,
    /// Usage help (`Reading meters`).
    pub usage_help: ObservedFrame,
    /// One refresh frame per cursor depth.
    pub variants: Vec<ObservedFrame>,
}

/// Replay JA-036 at every JA-001 size.
#[must_use]
pub fn ja036_hard_cases_refresh() -> Vec<Ja036Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn reach_manager(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA036_ID,
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
    session
}

fn capture_size(viewport: Viewport) -> Ja036Capture {
    let mut session = reach_manager(viewport);
    session.key(KeyCode::Char('c'));
    session.key(KeyCode::Char('?'));
    let help = session.observe("help");
    session.key(KeyCode::Esc);
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('r'));
    session.ticks(60);
    let refresh = session.observe("refresh");
    session.key(KeyCode::Char('u'));
    session.key(KeyCode::Char('?'));
    let usage_help = session.observe("usage-help");

    let mut variants = Vec::new();
    for (index, downs) in JA036_VARIANT_DOWNS.into_iter().enumerate() {
        let mut variant = reach_manager(viewport);
        variant.key(KeyCode::Char('c'));
        for _ in 0..downs {
            variant.key(KeyCode::Down);
        }
        variant.key(KeyCode::Char('r'));
        variant.ticks(60);
        variants.push(variant.observe(&format!("variant-{index}")));
    }

    Ja036Capture {
        help,
        refresh,
        usage_help,
        variants,
    }
}
