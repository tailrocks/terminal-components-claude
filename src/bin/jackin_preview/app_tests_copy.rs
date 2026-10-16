//! Copy-audit regressions: what the app says to the operator is in their
//! words, never the preview's internal vocabulary.

use ratatui::crossterm::event::KeyCode;

use crate::app::Route;
use crate::app_tests::H;
use crate::domain::agent::Agent;
use crate::scenario::{Motion, Scenario};
use crate::screens::Go;
use crate::sim::launch::LaunchPlan;

fn status(h: &H) -> String {
    h.app
        .status
        .as_ref()
        .map(|(s, _, _)| s.clone())
        .unwrap_or_default()
}

/// Words that belong to the preview's implementation, not to the operator.
const JARGON: [&str; 4] = ["ritual", "modeled", "runtime producer", "prefix command"];

fn assert_plain(what: &str, text: &str) {
    for word in JARGON {
        assert!(!text.contains(word), "{what} says {word:?}: {text}");
    }
}

#[test]
fn an_unreadable_instance_index_is_reported_without_jargon() {
    let h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    let s = status(&h);
    assert!(s.contains("Could not confirm running instances"), "{s}");
    assert!(s.contains("list may be incomplete"), "{s}");
    assert_plain("entry status", &s);
}

#[test]
fn a_blocked_launch_stage_says_what_to_do() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.go(Go::Launch {
        workspace: Some(1),
        role: "the-architect".into(),
        agent: Agent::Codex,
        account: Some("acct-codex-primary".into()),
        plan: LaunchPlan::BlockedSidecar,
    });
    h.draw();
    assert_eq!(h.app.route, Route::Cockpit, "{}", h.text());
    h.ticks(110);
    let c = h.app.screens.cockpit.as_ref().unwrap();
    assert!(c.run.blocked_at.is_some());
    assert_plain("activity", &c.activity);
    h.key(KeyCode::Esc);
    let s = status(&h);
    assert!(s.contains("This stage is blocked"), "{s}");
    assert_plain("blocked status", &s);
}

#[test]
fn an_unknown_pane_command_points_at_the_command_list() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('b');
    h.key(KeyCode::Char('Q'));
    let s = status(&h);
    assert_eq!(s, "Ctrl+B Q does nothing · Ctrl+B Space lists commands");
    assert_plain("prefix status", &s);
}
