//! JA-046: capsule tab switching via prefix keys and clicks (ground truth).
//!
//! Source audit: only tab-strip clicks switch tabs. The `Ctrl-B,n/p` and
//! `Ctrl-B,digit` chords are unbound and leave the active tab unchanged.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA046_ID: &str = "JA-046";

/// Prefix digit variants in contract order.
pub const JA046_DIGITS: [char; 4] = ['1', '2', '0', '9'];

/// Tab labels clicked in contract order.
pub const JA046_TABS: [&str; 3] = ["Mix", "Shell", "docs"];

/// One size of JA-046.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja046Capture {
    /// Initial capsule frame.
    pub initial: ObservedFrame,
    /// After `Ctrl-B,n`.
    pub next: ObservedFrame,
    /// Active tab index after `Ctrl-B,n`.
    pub next_tab: u8,
    /// After `Ctrl-B,p`.
    pub prev: ObservedFrame,
    /// Active tab index after `Ctrl-B,p`.
    pub prev_tab: u8,
    /// One frame per prefix digit variant.
    pub digits: Vec<ObservedFrame>,
    /// Active tab index per digit variant.
    pub digit_tabs: Vec<u8>,
    /// One frame per tab-label click.
    pub clicks: Vec<ObservedFrame>,
    /// Coordinates each tab label resolved to.
    pub click_at: Vec<Option<(u16, u16)>>,
    /// Active tab index per click.
    pub click_tabs: Vec<u8>,
}

/// Capture JA-046 at every JA-001 size.
#[must_use]
pub fn ja046_tab_switch() -> Vec<Ja046Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn active_tab(session: &DirectSession) -> u8 {
    session.app().capsule.tab
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA046_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn capture_size(viewport: Viewport) -> Ja046Capture {
    let mut session = fresh(viewport);
    let initial = session.observe("initial");
    session.ctrl('b');
    session.key(KeyCode::Char('n'));
    let next = session.observe("next");
    let next_tab = active_tab(&session);
    session.ctrl('b');
    session.key(KeyCode::Char('p'));
    let prev = session.observe("prev");
    let prev_tab = active_tab(&session);

    let mut digits = Vec::new();
    let mut digit_tabs = Vec::new();
    for digit in JA046_DIGITS {
        let mut variant = fresh(viewport);
        variant.ctrl('b');
        variant.key(KeyCode::Char(digit));
        digits.push(variant.observe(&format!("digit-{digit}")));
        digit_tabs.push(active_tab(&variant));
    }

    let mut clicks = Vec::new();
    let mut click_at = Vec::new();
    let mut click_tabs = Vec::new();
    for tab in JA046_TABS {
        let mut variant = fresh(viewport);
        let at = variant.find(tab);
        if let Some((x, y)) = at {
            variant.click(x, y);
        }
        click_at.push(at);
        clicks.push(variant.observe(&format!("click-{tab}")));
        click_tabs.push(active_tab(&variant));
    }

    Ja046Capture {
        initial,
        next,
        next_tab,
        prev,
        prev_tab,
        digits,
        digit_tabs,
        clicks,
        click_at,
        click_tabs,
    }
}
