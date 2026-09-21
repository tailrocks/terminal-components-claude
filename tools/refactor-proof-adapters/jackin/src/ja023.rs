//! JA-023: replay the editor accounts tab journey, then a no-match filter.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA023_ID: &str = "JA-023";

/// One size of JA-023.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja023Capture {
    /// Accounts tab with inherited defaults.
    pub open: ObservedFrame,
    /// After switching the default off for this workspace.
    pub disabled: ObservedFrame,
    /// After switching it back on.
    pub reenabled: ObservedFrame,
    /// After enabling the Experiments account.
    pub experiments: ObservedFrame,
    /// After preferring it and saving back to the manager.
    pub saved: ObservedFrame,
    /// Coordinate where the Experiments row resolved, when found.
    pub experiments_at: Option<(u16, u16)>,
    /// Accounts tab re-entered after the save.
    pub reentered: ObservedFrame,
    /// After `/`, which has no filter binding in the pinned source.
    pub after_slash: ObservedFrame,
    /// After typing the no-match query (global-letter fallthrough).
    pub after_type: ObservedFrame,
    /// After `Esc` from wherever typing landed.
    pub filter_closed: ObservedFrame,
}

/// Replay JA-023 at every JA-001 size.
#[must_use]
pub fn ja023_accounts_tab_replay() -> Vec<Ja023Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn open_accounts_tab(session: &mut DirectSession) {
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('5'));
    session.key(KeyCode::Enter);
}

fn capture_size(viewport: Viewport) -> Ja023Capture {
    let mut session = DirectSession::fresh(
        JA023_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    open_accounts_tab(&mut session);
    let open = session.observe("accounts-open");
    session.key(KeyCode::Char(' '));
    let disabled = session.observe("disabled");
    session.key(KeyCode::Char(' '));
    let reenabled = session.observe("reenabled");
    let experiments_at = session.find("Experiments");
    if let Some((x, y)) = experiments_at {
        session.click(x, y);
    }
    session.key(KeyCode::Char(' '));
    let experiments = session.observe("experiments");
    session.key(KeyCode::Char('p'));
    session.ctrl('s');
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
    session.ticks(20);
    let saved = session.observe("saved");

    open_accounts_tab(&mut session);
    let reentered = session.observe("reentered");
    session.key(KeyCode::Char('/'));
    let after_slash = session.observe("after-slash");
    session.type_str("no-such-account");
    let after_type = session.observe("after-type");
    session.key(KeyCode::Esc);
    let filter_closed = session.observe("filter-closed");

    Ja023Capture {
        open,
        disabled,
        reenabled,
        experiments,
        saved,
        experiments_at,
        reentered,
        after_slash,
        after_type,
        filter_closed,
    }
}
