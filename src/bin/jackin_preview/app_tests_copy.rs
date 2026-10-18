use crate::pages::Pages;
use ratatui::crossterm::event::KeyCode;

use crate::app::Route;
use crate::app_tests::H;
use crate::domain::agent::Agent;
use crate::scenario::{Motion, Scenario};
use crate::screens::Go;
use crate::sim::launch::LaunchPlan;

fn status(h: &H) -> String {
    h.app
        .shell
        .status
        .text()
        .map(str::to_owned)
        .unwrap_or_default()
}

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
    assert_eq!(h.app.route(), Route::Cockpit, "{}", h.text());
    h.ticks(110);
    let c = h.app.router.cockpit().unwrap();
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

#[test]
fn jackin_letters_keep_one_meaning_across_screens() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 220, 40);
    h.app.router.accounts_mut().selected =
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

    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 110, 220, 40);
    let f = footer(&h);
    assert!(
        f.contains("x Cancel launch…") && !f.contains("c Cancel"),
        "{f}"
    );
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("Cancel the launch?"), "{}", h.text());

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 220, 40);
    let live = h
        .app
        .world
        .instances
        .iter()
        .find(|i| i.status.is_live())
        .map(|i| i.id.clone())
        .expect("a live instance");
    h.app
        .router
        .manager_mut()
        .select_instance(&live, &h.app.world);
    h.draw();
    let f = footer(&h);
    assert!(
        f.contains("! Open shell") && !f.contains("x Open shell"),
        "{f}"
    );
    assert!(f.contains("c Accounts"), "{f}");
}

#[test]
fn every_quit_confirmation_says_quit() {
    let mut clean = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    clean.ctrl('q');
    assert!(clean.app.shell.quit);
    let cases = [
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

#[test]
fn capsule_resize_is_hinted_only_where_the_terminal_delivers_alt() {
    use junie_tui::core::event::ClientPlatform;
    use junie_tui::core::keyboard::{self, AltKeys, KeyboardProtocol};
    keyboard::set_alt_keys(AltKeys::from_env(ClientPlatform::MacOs, |k| {
        (k == "TERM_PROGRAM").then(|| "Apple_Terminal".to_owned())
    }));
    let footer = |h: &H| h.text().lines().last().unwrap_or_default().to_owned();
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 200, 40);
    assert_eq!(h.app.route(), Route::Capsule);
    let f = footer(&h);
    assert!(f.contains("Ctrl+B"), "{f}");
    assert!(!f.contains("Alt+"), "{f}");
    h.ctrl('b');
    assert!(footer(&h).contains("H J K L Resize"), "{}", footer(&h));

    let before = h.text();
    h.key(KeyCode::Char('L'));
    if h.text() == before {
        h.ctrl('b');
        h.key(KeyCode::Char('H'));
    }
    assert_ne!(h.text(), before, "Ctrl+B then H or L resizes");

    keyboard::set_alt_keys(AltKeys::WithProtocol);
    keyboard::session::enter(KeyboardProtocol::On, || true);
    h.draw();
    assert!(
        footer(&h).contains("Alt+Shift+↑↓←→ Resize"),
        "{}",
        footer(&h)
    );
    keyboard::session::leave();
}

#[test]
fn a_capsule_copy_reports_what_was_copied() {
    use junie_tui::core::event::MouseKind;
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ticks(60);
    h.type_str("hello");
    let (x, y) = h.find("hello").expect("typed text visible");
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 5, y);
    h.mouse(MouseKind::Up, x + 5, y);
    assert_eq!(h.app.world.clipboard.as_deref(), Some("hello"));
    assert_eq!(status(&h), "Copied selection");

    h.app.shell.status.clear();
    let before = h.app.clipboard_gen;
    h.key(KeyCode::Char('y'));
    assert!(h.app.clipboard_gen > before, "{}", h.text());
    assert_eq!(status(&h), "Copied selection");

    h.key(KeyCode::Esc);
    h.app.shell.status.clear();
    let before = h.app.clipboard_gen;
    h.key(KeyCode::Char('y'));
    assert_eq!(h.app.clipboard_gen, before);
    assert_eq!(status(&h), "", "{}", h.text());
}

#[test]
fn ctrl_c_over_the_build_log_selection_copies_instead_of_aborting() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 110, 120, 40);
    h.key(KeyCode::Char('b'));
    assert!(h.app.router.cockpit().unwrap().log_open);
    h.ctrl('a');
    let selected = h.app.router.cockpit().unwrap().log.selected_text();
    assert!(selected.is_some(), "Ctrl+A selects the log: {}", h.text());
    junie_tui::shell::clipboard::take_output();
    h.ctrl('c');
    assert_eq!(h.app.route(), Route::Cockpit, "the launch goes on");
    assert_eq!(h.app.world.clipboard, selected);
    assert!(status(&h).starts_with("Copied "), "{}", status(&h));

    assert_eq!(
        junie_tui::shell::clipboard::take_output(),
        [junie_tui::shell::clipboard::osc52(
            selected.as_deref().unwrap()
        )]
    );

    h.key(KeyCode::Esc);
    h.ctrl('c');
    assert_eq!(h.app.route(), Route::Manager);
}
