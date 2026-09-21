//! JA-021: replay `editor_env_plain_value_stays_masked` with checkpoints.
//!
//! The typed secret is an obviously fake fixture value; the capture asserts it
//! never renders in any frame while its tail hint does.

use jackin_app::{Motion, Scenario};
use junie_tui::{Id, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA021_ID: &str = "JA-021";

/// Fake fixture secret typed through the env form (never a real credential).
pub const JA021_FIXTURE_SECRET: &str = "fixture-only-abcdefgh1234";

/// One size of JA-021.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja021Capture {
    /// Environments tab with the plain value masked.
    pub masked: ObservedFrame,
    /// After `m`: reveal request refused for plain values.
    pub after_reveal: ObservedFrame,
    /// New-key form while typing the secret.
    pub typing: ObservedFrame,
    /// After staging the new key (tail-only display).
    pub staged: ObservedFrame,
    /// After preview/save and the deterministic save ticks.
    pub saved: ObservedFrame,
    /// Whether the staged key reached durable world state.
    pub committed: bool,
}

/// Replay JA-021 at every JA-001 size.
#[must_use]
pub fn ja021_env_masked_replay() -> Vec<Ja021Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn cfg_save() -> Id {
    Id::root("editor.cfg").sub("form").sub("save")
}

fn capture_size(viewport: Viewport) -> Ja021Capture {
    let mut session = DirectSession::fresh(
        JA021_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('4'));
    session.key(KeyCode::Enter);
    let masked = session.observe("masked");
    session.key(KeyCode::Char('m'));
    let after_reveal = session.observe("after-reveal");
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("NEW_SECRET");
    session.key(KeyCode::Tab);
    session.key(KeyCode::Tab);
    session.key(KeyCode::Enter);
    session.type_str(JA021_FIXTURE_SECRET);
    let typing = session.observe("typing");
    session.key(KeyCode::Tab);
    let _ = session.tab_to(cfg_save());
    session.key(KeyCode::Enter);
    let staged = session.observe("staged");
    session.ctrl('s');
    let _ = session.tab_to(cfg_save());
    session.key(KeyCode::Enter);
    session.ticks(20);
    let saved = session.observe("saved");
    let committed = session
        .app()
        .world
        .workspace(1)
        .is_some_and(|ws| ws.env.iter().any(|env| env.key == "NEW_SECRET"));
    Ja021Capture {
        masked,
        after_reveal,
        typing,
        staged,
        saved,
        committed,
    }
}
