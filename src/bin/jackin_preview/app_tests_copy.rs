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

fn footer(h: &H) -> String {
    h.text().lines().last().unwrap_or_default().to_owned()
}

/// One letter, one meaning across jackin's screens: `d` deletes or removes,
/// `x` closes or stops what is in front, `c` opens Accounts.
#[test]
fn jackin_letters_keep_one_meaning_across_screens() {
    // Accounts: d removes (as on the manager), t switches enabled
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 220, 40);
    h.app.screens.accounts.selected =
        crate::screens::accounts::Sel::Account("acct-claude-work".into());
    h.draw();
    let f = footer(&h);
    assert!(f.contains("d Remove…") && f.contains("t Disable"), "{f}");
    assert!(!f.contains("x Remove") && !f.contains("d Disable"), "{f}");
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("Remove account"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('t'));
    assert!(
        !h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .enabled
    );
    assert!(footer(&h).contains("t Enable"), "{}", footer(&h));

    // Cockpit: x cancels the launch, c stays Accounts' letter
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 110, 220, 40);
    let f = footer(&h);
    assert!(
        f.contains("x Cancel launch…") && !f.contains("c Cancel"),
        "{f}"
    );
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("Cancel the launch?"), "{}", h.text());

    // Manager: the shell is !, x never opens something
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 220, 40);
    let live = h
        .app
        .world
        .instances
        .iter()
        .find(|i| i.status.is_live())
        .map(|i| i.id.clone())
        .expect("a live instance");
    h.app.screens.manager.select_instance(&live, &h.app.world);
    h.draw();
    let f = footer(&h);
    assert!(
        f.contains("! Open shell") && !f.contains("x Open shell"),
        "{f}"
    );
    assert!(f.contains("c Accounts"), "{f}");
}

/// Ctrl+Q is Quit in every hint and menu, so every confirmation it opens
/// says Quit too: title, body and buttons.
#[test]
fn every_quit_confirmation_says_quit() {
    let cases = [
        H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40),
        H::new(Scenario::LaunchRunning, Motion::Paused, 110, 120, 40),
        H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40),
    ];
    for mut h in cases {
        h.ctrl('q');
        let t = h.text();
        assert!(t.contains("Quit"), "{t}");
        assert!(!t.contains("Exit"), "{t}");
    }
}
