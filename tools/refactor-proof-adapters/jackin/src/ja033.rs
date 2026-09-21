//! JA-033: replay the masked plain-key journey with checkpoints.
//!
//! The typed key is an obviously fake fixture value; the capture asserts it
//! never renders while its tail hint does. Source audit: the secret input is
//! painted and focusable only at wider sizes; at 80×24 the narrow form
//! exposes no secret control, which the capture records as data.

use jackin_app::screens::accounts::{FORM, SECRET, SOURCE};
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA033_ID: &str = "JA-033";

/// Fake fixture key typed through the form (never a real credential).
pub const JA033_FIXTURE_KEY: &str = "fixture-only-plain-abcdefgh1234";

/// One size of JA-033.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja033Capture {
    /// Source selector on `API key`.
    pub source: ObservedFrame,
    /// Selected source index (`API key` is 2).
    pub source_index: u8,
    /// Whether the secret input resolves at this size.
    pub secret_reachable: bool,
    /// While typing the key (must stay masked).
    pub typing: ObservedFrame,
    /// After leaving the secret field (tail hint only).
    pub tail: ObservedFrame,
    /// After saving the account.
    pub saved: ObservedFrame,
    /// Removal confirmation.
    pub remove_ask: ObservedFrame,
    /// After cancelling the removal.
    pub remove_cancelled: ObservedFrame,
    /// Whether the account survived the cancelled removal.
    pub retained: bool,
    /// Debug credential source (must not embed key material).
    pub source_debug: String,
}

/// Replay JA-033 at every JA-001 size.
#[must_use]
pub fn ja033_plain_key_replay() -> Vec<Ja033Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja033Capture {
    let mut session = DirectSession::fresh(
        JA033_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("Spare");
    let _ = session.tab_to(SOURCE);
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    let source = session.observe("source");
    let source_index = session.app().accounts.source_index;
    let secret_reachable = session.tab_to(SECRET);
    if !secret_reachable {
        let stall = |session: &DirectSession, name: &str| session.observe(name);
        return Ja033Capture {
            typing: stall(&session, "typing-unreachable"),
            tail: stall(&session, "tail-unreachable"),
            saved: stall(&session, "saved-unreachable"),
            remove_ask: stall(&session, "remove-ask-unreachable"),
            remove_cancelled: stall(&session, "remove-cancelled-unreachable"),
            retained: false,
            source_debug: String::from("unreachable"),
            source,
            source_index,
            secret_reachable,
        };
    }
    session.key(KeyCode::Enter);
    session.type_str(JA033_FIXTURE_KEY);
    let typing = session.observe("typing");
    session.key(KeyCode::Tab);
    let tail = session.observe("tail");
    let _ = session.tab_to(FORM.sub("save"));
    session.key(KeyCode::Enter);
    let saved = session.observe("saved");
    let source_debug = session
        .app()
        .world
        .accounts
        .get("acct-anthropic-spare")
        .map_or(String::from("missing"), |account| {
            format!("{:?}", account.source)
        });
    session.key(KeyCode::Char('x'));
    let remove_ask = session.observe("remove-ask");
    session.key(KeyCode::Esc);
    let remove_cancelled = session.observe("remove-cancelled");
    let retained = session
        .app()
        .world
        .accounts
        .get("acct-anthropic-spare")
        .is_some();
    Ja033Capture {
        source,
        source_index,
        secret_reachable,
        typing,
        tail,
        saved,
        remove_ask,
        remove_cancelled,
        retained,
        source_debug,
    }
}
