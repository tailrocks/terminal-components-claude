//! JA-030: accounts tree navigation and filter attempts (ground truth).
//!
//! Source audit: fresh accounts starts host-focused, so the capture tabs into
//! the list before the contract keys (matching the post-navigation focus the
//! notation assumes). `Esc` leaves Accounts for the Manager, and the pinned
//! accounts screen has no `/` filter binding (`/` is inert; typed characters
//! act as navigation keys). The capture drives the contract prefix and records
//! the `Esc` departure, replays the navigation without `Esc` for the remaining
//! tree coverage, and attempts the filters on fresh sessions.

use jackin_app::screens::accounts::LIST;
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA030_ID: &str = "JA-030";

/// Verbatim navigation prefix in contract order.
pub const JA030_PREFIX: [KeyCode; 9] = [
    KeyCode::Home,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Tab,
    KeyCode::Esc,
    KeyCode::Char('*'),
    KeyCode::Char('-'),
    KeyCode::End,
    KeyCode::Home,
];

/// Prefix without the departing `Esc`.
pub const JA030_PREFIX_NO_ESC: [KeyCode; 8] = [
    KeyCode::Home,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Tab,
    KeyCode::Char('*'),
    KeyCode::Char('-'),
    KeyCode::End,
    KeyCode::Home,
];

/// One size of JA-030.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja030Capture {
    /// One frame per verbatim prefix key (`Esc` departs to the manager).
    pub prefix: Vec<ObservedFrame>,
    /// One frame per no-`Esc` navigation key, staying in accounts.
    pub navigation: Vec<ObservedFrame>,
    /// Fresh accounts session before the `/` attempt.
    pub filter_base: ObservedFrame,
    /// After `/` on a fresh accounts session (inert).
    pub after_slash: ObservedFrame,
    /// After typing `Work` (keys, not a filter query).
    pub after_type: ObservedFrame,
    /// After `Esc` from accounts.
    pub after_esc: ObservedFrame,
    /// Fresh accounts session before the no-match query.
    pub no_match_base: ObservedFrame,
    /// After typing an inert no-match query.
    pub no_match: ObservedFrame,
    /// After the final `Esc`.
    pub closed: ObservedFrame,
}

/// Capture JA-030 at every JA-001 size.
#[must_use]
pub fn ja030_accounts_navigation() -> Vec<Ja030Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn open_accounts(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA030_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let _ = session.tab_to(LIST);
    session
}

fn capture_size(viewport: Viewport) -> Ja030Capture {
    let mut verbatim = open_accounts(viewport);
    let mut prefix = Vec::new();
    for (index, key) in JA030_PREFIX.into_iter().enumerate() {
        verbatim.key(key);
        prefix.push(verbatim.observe(&format!("prefix-{index}")));
    }

    let mut navigate = open_accounts(viewport);
    let mut navigation = Vec::new();
    for (index, key) in JA030_PREFIX_NO_ESC.into_iter().enumerate() {
        navigate.key(key);
        navigation.push(navigate.observe(&format!("nav-{index}")));
    }

    let mut filter = open_accounts(viewport);
    let filter_base = filter.observe("filter-base");
    filter.key(KeyCode::Char('/'));
    let after_slash = filter.observe("after-slash");
    filter.type_str("Work");
    let after_type = filter.observe("after-type");
    filter.key(KeyCode::Esc);
    let after_esc = filter.observe("after-esc");

    let mut no_match_session = open_accounts(viewport);
    let no_match_base = no_match_session.observe("no-match-base");
    // Digit/dash query: every character is inert in the accounts list, so the
    // frame proves no filter field consumes text.
    no_match_session.type_str("000-111");
    let no_match = no_match_session.observe("no-match");
    no_match_session.key(KeyCode::Esc);
    let closed = no_match_session.observe("closed");

    Ja030Capture {
        prefix,
        navigation,
        filter_base,
        after_slash,
        after_type,
        after_esc,
        no_match_base,
        no_match,
        closed,
    }
}
