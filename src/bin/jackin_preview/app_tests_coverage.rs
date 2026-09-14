//! Additional deterministic screen coverage for Jackin Preview.
//!
//! These tests deliberately stay on the in-process TestBackend.  PTY and
//! snapshot coverage belongs to `tests/visual_baseline`; this module proves
//! the state transitions that can be made stable without a terminal process.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;

use crate::app::Route;
use crate::app_tests::H;
use crate::domain::agent::Agent;
use crate::domain::instance::InstanceStatus;
use crate::domain::usage::Freshness;
use crate::scenario::{Motion, Scenario};
use crate::screens::Go;
use crate::screens::manager::RowKey;
use crate::sim::launch::LaunchPlan;

fn status(h: &H) -> &str {
    h.app
        .status
        .as_ref()
        .map(|(s, _, _)| s.as_str())
        .unwrap_or("")
}

fn outside(h: &H) -> Position {
    let area = h.term.backend().buffer().area;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let pos = Position::new(x, y);
            if h.app.hits.hit(pos).is_none() {
                return pos;
            }
        }
    }
    panic!(
        "no outside hit-test coordinate in {}x{}",
        area.width, area.height
    );
}

fn hard_manager() -> H {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    if h.app.route == Route::Intro {
        for _ in 0..8 {
            h.ticks(3);
            if h.app.route == Route::Manager {
                break;
            }
            h.key(KeyCode::Enter);
        }
    }
    assert_eq!(h.app.route, Route::Manager, "{}", h.text());
    h
}

fn select_instance(h: &mut H, id: &str) {
    h.app.screens.manager.select_instance(id, &h.app.world);
    h.draw();
    assert!(
        h.text().contains(id.trim_start_matches("jk-")),
        "{}",
        h.text()
    );
}

fn select_workspace(h: &mut H, id: u32) {
    h.app.screens.manager.selected = RowKey::Workspace(id);
    h.draw();
}

fn editor(h: &mut H, workspace: u32) {
    h.app.go(Go::Editor {
        workspace: Some(workspace),
        pending: None,
    });
    h.draw();
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
}

fn launch(h: &mut H, plan: LaunchPlan) {
    h.app.go(Go::Launch {
        workspace: Some(1),
        role: "the-architect".into(),
        agent: Agent::Codex,
        account: Some("acct-codex-primary".into()),
        plan,
    });
    h.draw();
    assert_eq!(h.app.route, Route::Cockpit, "{}", h.text());
}

#[test]
fn startup_inventory_focus_and_resize_recovery_are_deterministic() {
    let cases = [
        (Scenario::FirstUse, Route::Intro, "jackin❯"),
        (Scenario::Returning, Route::Manager, "Current directory"),
        (Scenario::AccountsMixed, Route::Accounts, "Accounts"),
        (Scenario::LaunchRunning, Route::Cockpit, "Launch"),
        (Scenario::LaunchFailure, Route::Cockpit, "Launch"),
        (Scenario::CapsuleMulti, Route::Capsule, "File"),
        (Scenario::OutroLast, Route::Capsule, "File"),
        (Scenario::HardCases, Route::Manager, "Current directory"),
    ];

    for (scenario, route, marker) in cases {
        let h = if scenario == Scenario::HardCases {
            hard_manager()
        } else {
            H::new(scenario, Motion::Paused, 0, 120, 40)
        };
        assert_eq!(h.app.route, route, "{scenario:?}: {}", h.text());
        assert!(h.text().contains(marker), "{scenario:?}: {}", h.text());
        match route {
            Route::Manager => assert_eq!(
                h.app.focus.current(),
                Some(crate::screens::manager::TREE),
                "{scenario:?}"
            ),
            Route::Accounts => assert_eq!(
                h.app.focus.current(),
                Some(crate::screens::accounts::TREE),
                "{scenario:?}"
            ),
            Route::Cockpit => assert_eq!(
                h.app.focus.current(),
                Some(crate::screens::cockpit::RAIL),
                "{scenario:?}"
            ),
            Route::Capsule => assert_eq!(
                h.app.focus.current(),
                Some(crate::screens::capsule::PANES),
                "{scenario:?}"
            ),
            Route::Intro | Route::Outro | Route::Handoff => {}
            _ => panic!("unexpected startup route {route:?}"),
        }
    }

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    for (width, height, marker) in [
        (71, 19, "Terminal too small"),
        (72, 20, "Current directory"),
        (80, 24, "Current directory"),
        (160, 50, "Current directory"),
        (60, 18, "Terminal too small"),
        (80, 24, "Current directory"),
    ] {
        h.resize(width, height);
        assert_eq!(h.app.size, (width, height));
        assert!(h.text().contains(marker), "{width}x{height}: {}", h.text());
    }
    assert_eq!(h.app.focus.current(), Some(crate::screens::manager::TREE));
}

#[test]
fn manager_scrollbar_click_and_drag_change_the_rendered_tree() {
    let mut h = hard_manager();
    h.key(KeyCode::Char('*'));
    h.resize(72, 20);
    let scrollbar = junie_tui::widgets::scrollbar::id_for(crate::screens::manager::TREE);
    let area = h
        .app
        .hits
        .area_of(scrollbar)
        .expect("hard manager tree must render a scrollbar");
    let bottom = Position::new(area.x, area.bottom().saturating_sub(1));
    assert_eq!(h.app.hits.hit(bottom), Some(scrollbar));
    let before_click = h.text();
    h.click(bottom.x, bottom.y);
    assert_ne!(
        h.text(),
        before_click,
        "scrollbar click did not repaint the tree"
    );

    let mut drag = hard_manager();
    drag.key(KeyCode::Char('*'));
    drag.resize(72, 20);
    let drag_area = drag
        .app
        .hits
        .area_of(scrollbar)
        .expect("hard manager drag tree must render a scrollbar");
    let start_y = (drag_area.top()..drag_area.bottom())
        .find(|y| drag.app.hits.hit(Position::new(drag_area.x, *y)) == Some(scrollbar))
        .expect("scrollbar thumb must be hittable");
    let end_y = (drag_area.top()..drag_area.bottom())
        .rev()
        .find(|y| drag.app.hits.hit(Position::new(drag_area.x, *y)) == Some(scrollbar))
        .expect("scrollbar track must be hittable");
    assert!(start_y < end_y);
    let before_drag = drag.text();
    drag.mouse(MouseKind::Down, drag_area.x, start_y);
    drag.mouse(MouseKind::Drag, drag_area.x, end_y);
    drag.mouse(MouseKind::Up, drag_area.x, end_y);
    assert_ne!(
        drag.text(),
        before_drag,
        "scrollbar drag did not repaint the tree"
    );
}

#[test]
fn modal_outside_click_traps_focus_and_restores_the_owner() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    let owner = h.app.focus.current();
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
    assert_ne!(h.app.focus.current(), owner);
    h.key(KeyCode::Tab);
    assert_ne!(h.app.focus.current(), owner);
    let p = outside(&h);
    h.mouse(MouseKind::Down, p.x, p.y);
    h.mouse(MouseKind::Up, p.x, p.y);
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "help survived outside click"
    );
    assert_eq!(h.app.focus.current(), owner);
    assert!(h.text().contains("Current directory"), "{}", h.text());

    let mut c = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let owner = c.app.focus.current();
    c.ctrl('\\');
    assert!(c.text().contains("Command palette"), "{}", c.text());
    assert_ne!(c.app.focus.current(), owner);
    let p = outside(&c);
    c.mouse(MouseKind::Down, p.x, p.y);
    c.mouse(MouseKind::Up, p.x, p.y);
    assert!(
        !c.text().contains("Command palette"),
        "picker survived outside click"
    );
    assert_eq!(c.app.focus.current(), owner);
    assert_eq!(c.app.route, Route::Capsule);

    c.ctrl('q');
    let choice = c.text();
    assert!(
        choice.contains("Unsaved work") || choice.contains("Exit"),
        "{choice}"
    );
    let p = outside(&c);
    c.mouse(MouseKind::Down, p.x, p.y);
    c.mouse(MouseKind::Up, p.x, p.y);
    assert!(
        c.text().contains("Unsaved work") || c.text().contains("Exit"),
        "destructive choice dismissed outside: {}",
        c.text()
    );
    c.key(KeyCode::Esc);
    assert_eq!(c.app.focus.current(), owner);
}

#[test]
fn manager_reconnect_lifecycle_states_are_explicit() {
    for id in ["jk-a1c0", "jk-c41e", "jk-04d7"] {
        let mut h = hard_manager();
        select_instance(&mut h, id);
        h.key(KeyCode::Char('r'));
        assert_eq!(h.app.route, Route::Capsule, "{id}: {}", h.text());
        assert_eq!(
            h.app.world.instance(id).map(|i| i.status),
            Some(InstanceStatus::Running),
            "{id}: {}",
            h.text()
        );
        assert!(h.app.world.daemons.contains_key(id), "{id}: {}", h.text());
    }

    for (id, message) in [
        ("jk-12ee", "crashed"),
        ("jk-5e5e", "never reached the Capsule"),
    ] {
        let mut h = hard_manager();
        select_instance(&mut h, id);
        h.key(KeyCode::Char('r'));
        assert_eq!(h.app.route, Route::Manager, "{id}: {}", h.text());
        assert!(status(&h).contains(message), "{id}: {}", status(&h));
    }

    let mut h = hard_manager();
    select_instance(&mut h, "jk-f1f1");
    h.key(KeyCode::Char('r'));
    assert_eq!(h.app.route, Route::Manager);
    assert!(status(&h).contains("exited cleanly"), "{}", status(&h));
}

#[test]
fn manager_stop_purge_delete_and_refresh_keep_fixture_state_coherent() {
    let mut h = hard_manager();
    select_instance(&mut h, "jk-7f3a");
    h.key(KeyCode::Char('t'));
    assert!(h.text().contains("Stop instance"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(
        h.app.world.instance("jk-7f3a").unwrap().status,
        InstanceStatus::Running
    );

    h.key(KeyCode::Char('t'));
    h.key(KeyCode::Char('y'));
    for _ in 0..40 {
        if h.app.world.instance("jk-7f3a").unwrap().status == InstanceStatus::PreservedDirty {
            break;
        }
        h.ticks(1);
    }
    assert_eq!(
        h.app.world.instance("jk-7f3a").unwrap().status,
        InstanceStatus::PreservedDirty
    );
    assert!(!h.app.world.daemons.contains_key("jk-7f3a"));
    assert!(status(&h).contains("stopped"), "{}", status(&h));

    let mut h = hard_manager();
    select_instance(&mut h, "jk-c41e");
    h.key(KeyCode::Char('p'));
    assert!(h.text().contains("Purge instance"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.type_str("c41e");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    for _ in 0..50 {
        if h.app.world.instance("jk-c41e").unwrap().status == InstanceStatus::Purged {
            break;
        }
        h.ticks(1);
    }
    assert_eq!(
        h.app.world.instance("jk-c41e").unwrap().status,
        InstanceStatus::Purged
    );
    assert!(!h.app.world.daemons.contains_key("jk-c41e"));
    assert!(status(&h).contains("Purged"), "{}", status(&h));

    let mut h = hard_manager();
    select_workspace(&mut h, 1);
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("Delete workspace"), "{}", h.text());
    h.key(KeyCode::Char('y'));
    assert!(!h.app.world.workspaces.iter().any(|w| w.id == 1));
    assert!(h.app.world.instance("jk-7f3a").unwrap().workspace.is_none());
    assert!(
        status(&h).contains("instances and files kept"),
        "{}",
        status(&h)
    );

    let mut h = hard_manager();
    h.key(KeyCode::F(5));
    h.ticks(4);
    assert!(status(&h).contains("Refresh failed"), "{}", status(&h));
    assert_eq!(
        h.app.world.daemon_health,
        crate::sim::world::DaemonHealth::Stale
    );
}

#[test]
fn manager_prewarm_github_and_shell_session_are_wired() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    select_workspace(&mut h, 1);
    h.key(KeyCode::Char('w'));
    assert!(status(&h).contains("Prewarming"), "{}", status(&h));
    h.ticks(40);
    assert!(
        status(&h).contains("derived image cached"),
        "{}",
        status(&h)
    );
    h.key(KeyCode::Char('o'));
    assert!(
        status(&h).contains("Opened https://github.com"),
        "{}",
        status(&h)
    );

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    select_instance(&mut h, "jk-7f3a");
    let before = h.app.world.daemons["jk-7f3a"].tabs.len();
    h.key(KeyCode::Char('a'));
    assert!(
        h.text().contains("New session · choose Agent"),
        "{}",
        h.text()
    );
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    assert_eq!(h.app.world.daemons["jk-7f3a"].tabs.len(), before + 1);
    let panes = h.app.world.daemons["jk-7f3a"]
        .tabs
        .last()
        .map(|t| t.root.leaves())
        .unwrap_or_default();
    assert!(panes.iter().any(|p| {
        h.app.world.daemons["jk-7f3a"]
            .pane(*p)
            .is_some_and(|pane| pane.proc.agent.is_none())
    }));
}

#[test]
fn cockpit_credentials_retry_plain_and_cancel_are_recoverable() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.world.op.session = crate::sim::onepassword::OpSession::Locked;
    launch(&mut h, LaunchPlan::CredentialsLocked);
    h.ticks(110);
    assert!(h.app.screens.cockpit.as_ref().unwrap().credential_hold);
    assert!(
        h.text().contains("Credential source unavailable"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert!(!h.app.screens.cockpit.as_ref().unwrap().credential_hold);
    assert_eq!(
        h.app.world.op.session,
        crate::sim::onepassword::OpSession::SignedIn
    );

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.world.op.session = crate::sim::onepassword::OpSession::Locked;
    launch(&mut h, LaunchPlan::CredentialsLocked);
    h.ticks(110);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let c = h.app.screens.cockpit.as_ref().unwrap();
    assert!(!c.credential_hold);
    assert!(c.account.is_none());
    assert!(
        h.text().contains("plain text") && h.text().contains("masked"),
        "{}",
        h.text()
    );

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.world.op.session = crate::sim::onepassword::OpSession::Locked;
    launch(&mut h, LaunchPlan::CredentialsLocked);
    h.ticks(110);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert!(status(&h).contains("Launch failed"), "{}", status(&h));
}

#[test]
fn cockpit_blocked_sidecar_abort_log_and_info_paths_are_screen_level() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    launch(&mut h, LaunchPlan::BlockedSidecar);
    h.ticks(110);
    let c = h.app.screens.cockpit.as_ref().unwrap();
    assert_eq!(c.run.blocked_at, Some(crate::sim::launch::Stage::Sidecar));
    assert!(c.activity.contains("Blocked"), "{}", h.text());
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("Blocked"), "{}", h.text());
    h.ctrl('q');
    assert!(h.text().contains("Exit jackin"), "{}", h.text());
    h.key(KeyCode::Char('y'));
    assert!(h.app.quit);

    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 110, 120, 40);
    h.key(KeyCode::Char('b'));
    assert!(h.app.screens.cockpit.as_ref().unwrap().log_open);
    assert!(h.text().contains("Docker build"), "{}", h.text());
    h.key(KeyCode::PageUp);
    h.key(KeyCode::End);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('d'));
    h.key(KeyCode::Char('i'));
    assert!(h.text().contains("Debug info"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.ctrl('c');
    assert_eq!(h.app.route, Route::Manager);
    assert!(status(&h).contains("Launch failed"), "{}", status(&h));
    assert!(
        h.app
            .world
            .instances
            .iter()
            .any(|i| i.status == InstanceStatus::FailedSetup)
    );
}

#[test]
fn capsule_palette_prefix_usage_and_close_commands_mutate_the_daemon() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let instance = h.app.screens.capsule.as_ref().unwrap().instance.clone();
    let before = h.app.world.daemons[&instance].tabs.len();

    h.ctrl('b');
    h.key(KeyCode::Char('r'));
    assert!(status(&h).contains("Redrawn"), "{}", status(&h));
    h.ctrl('b');
    h.key(KeyCode::Char('z'));
    assert!(
        h.app.world.daemons[&instance]
            .active_tab()
            .unwrap()
            .zoomed
            .is_some()
    );
    h.ctrl('b');
    h.key(KeyCode::Char('z'));
    assert!(
        h.app.world.daemons[&instance]
            .active_tab()
            .unwrap()
            .zoomed
            .is_none()
    );

    h.ctrl('b');
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("New tab"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Account for"), "{}", h.text());
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.world.daemons[&instance].tabs.len(), before + 1);

    h.ctrl('\\');
    h.type_str("clear pane");
    assert!(h.text().contains("Clear pane"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(!h.app.screens.capsule.as_ref().unwrap().dialog_open);

    h.ctrl('b');
    h.key(KeyCode::Char('u'));
    assert!(h.text().contains("Usage"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::PageDown);
    h.key(KeyCode::Char('r'));
    h.ticks(6);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert_eq!(h.app.route, Route::Capsule);
}

#[test]
fn capsule_prefix_tabs_splits_clear_and_close_keep_rendered_state_in_sync() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let instance = h.app.screens.capsule.as_ref().unwrap().instance.clone();
    let initial_tabs = h.app.world.daemons[&instance].tabs.len();
    let initial_panes = h.app.world.daemons[&instance].panes.len();
    let initial_text = h.text();

    h.ctrl('b');
    h.key(KeyCode::Char('n'));
    assert_eq!(h.app.world.daemons[&instance].active, 1);
    assert_ne!(h.text(), initial_text);
    h.ctrl('b');
    h.key(KeyCode::Char('p'));
    assert_eq!(h.app.world.daemons[&instance].active, 0);

    h.ctrl('b');
    h.key(KeyCode::Char('%'));
    assert!(h.text().contains("Split → Right"), "{}", h.text());
    h.key(KeyCode::Enter);
    if h.text().contains("Account for") {
        h.key(KeyCode::Down);
        h.key(KeyCode::Enter);
    }
    assert_eq!(
        h.app.world.daemons[&instance].panes.len(),
        initial_panes + 1
    );

    h.ctrl('b');
    h.key(KeyCode::Char('"'));
    assert!(h.text().contains("Split ↓ Below"), "{}", h.text());
    h.key(KeyCode::Enter);
    if h.text().contains("Account for") {
        h.key(KeyCode::Down);
        h.key(KeyCode::Enter);
    }
    assert_eq!(
        h.app.world.daemons[&instance].panes.len(),
        initial_panes + 2
    );

    {
        let daemon = h.app.world.daemons.get_mut(&instance).unwrap();
        daemon.active = 0;
        daemon.active_tab_mut().unwrap().focused = 3;
    }
    h.draw();
    let before_clear = h.app.world.daemons[&instance].pane(3).unwrap().term.len();
    assert!(
        before_clear > 100,
        "fixture lost scrollback: {before_clear}"
    );
    h.ctrl('b');
    h.ctrl('l');
    let after_clear = h.app.world.daemons[&instance].pane(3).unwrap().term.len();
    assert!(
        after_clear < before_clear,
        "clear pane did not change model"
    );
    assert!(h.text().contains("Shell"), "{}", h.text());

    h.ctrl('b');
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("Close pane?"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(
        h.app.world.daemons[&instance].panes.len(),
        initial_panes + 1
    );

    h.ctrl('b');
    h.key(KeyCode::Char('&'));
    assert!(h.text().contains("Close tab?"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.world.daemons[&instance].tabs.len(), initial_tabs - 1);
    assert!(!h.text().contains("Close tab?"), "{}", h.text());
}

#[test]
fn paste_updates_editor_model_and_filters_newlines_in_capsule_input() {
    let mut ed = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    editor(&mut ed, 1);
    ed.key(KeyCode::Enter);
    ed.key(KeyCode::Enter);
    ed.ctrl('l');
    assert_eq!(
        ed.paste("workspace-paste"),
        junie_tui::core::event::Outcome::Changed
    );
    assert!(ed.text().contains("workspace-paste"), "{}", ed.text());
    assert_eq!(
        ed.app.screens.editor.as_ref().unwrap().pending.name,
        "workspace-paste"
    );
    ed.key(KeyCode::Enter);
    assert_eq!(
        ed.app.screens.editor.as_ref().unwrap().pending.name,
        "workspace-paste"
    );

    let mut capsule = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let instance = capsule
        .app
        .screens
        .capsule
        .as_ref()
        .unwrap()
        .instance
        .clone();
    let pane = capsule.app.world.daemons[&instance].focused_pane().unwrap();
    assert_eq!(
        capsule.paste("paste-one\npaste-two\r"),
        junie_tui::core::event::Outcome::Changed
    );
    assert_eq!(
        capsule.app.world.daemons[&instance]
            .pane(pane)
            .unwrap()
            .input,
        "paste-onepaste-two"
    );
    assert!(
        capsule.text().contains("paste-onepaste-two"),
        "{}",
        capsule.text()
    );
    assert!(!capsule.text().contains("paste-one\npaste-two"));
}

#[test]
fn paste_reaches_form_and_file_browser_modal_fields() {
    let mut accounts = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    accounts.key(KeyCode::Char('a'));
    assert!(
        accounts.text().contains("New account"),
        "{}",
        accounts.text()
    );
    accounts.key(KeyCode::Enter);
    accounts.paste("Pasted account");
    assert!(
        accounts.text().contains("Pasted account"),
        "{}",
        accounts.text()
    );

    let mut prelude = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    prelude.app.go(Go::Prelude);
    prelude.draw();
    prelude.key(KeyCode::Char('g'));
    prelude.paste("github.com/chainargos/payments-platform");
    assert!(
        prelude.text().contains("chainargos/payments-platform"),
        "{}",
        prelude.text()
    );
    prelude.key(KeyCode::Enter);
    assert!(
        prelude.text().contains("Mount destination"),
        "{}",
        prelude.text()
    );
}

#[test]
fn attach_restores_the_requested_pane_focus() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let instance = h.app.screens.capsule.as_ref().unwrap().instance.clone();
    let target = h.app.world.daemons[&instance].tabs[0].leaves()[1];

    h.app.go(Go::Attach {
        instance: instance.clone(),
        pane: Some(target),
    });
    h.draw();

    assert_eq!(h.app.world.daemons[&instance].focused_pane(), Some(target));
}

#[test]
fn capsule_dirty_exit_covers_new_agent_inspect_keep_and_discard() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('q');
    h.key(KeyCode::Enter);
    assert!(h.text().contains("New tab"), "{}", h.text());
    h.key(KeyCode::Esc);

    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('q');
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Inspect changes"), "{}", h.text());
    h.key(KeyCode::Esc);

    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('q');
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert!(status(&h).contains("Still inside") || h.text().contains("Still inside"));

    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl('q');
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Exit and discard changes"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    h.type_str("payments-platform");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert_eq!(h.app.world.instance("jk-7f3a").unwrap().uncommitted, 0);
    assert!(status(&h).contains("Still inside") || h.text().contains("Still inside"));
}

#[test]
fn editor_validation_and_save_failure_preserve_pending_changes() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    editor(&mut h, 1);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("infra-control-plane");
    h.key(KeyCode::Enter);
    h.ctrl('s');
    assert!(status(&h).contains("already exists"), "{}", status(&h));
    assert_eq!(h.app.route, Route::Editor);

    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    if h.app.route == Route::Intro {
        h.ticks(3);
        h.key(KeyCode::Enter);
    }
    editor(&mut h, 1);
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('r'));
    assert!(h.app.screens.editor.as_ref().unwrap().change_count() > 0);
    h.ctrl('s');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(15);
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
    assert!(h.text().contains("Save failed"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.screens.editor.as_ref().unwrap().change_count() > 0);
}

#[test]
fn editor_environment_validation_and_masking_stay_local() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    editor(&mut h, 1);
    h.key(KeyCode::Char('4'));
    h.key(KeyCode::Enter);
    h.app
        .screens
        .editor
        .as_mut()
        .unwrap()
        .cfg
        .pending
        .env
        .push(crate::domain::workspace::EnvVar::plain("BAD-KEY", "bad"));
    h.draw();
    h.ctrl('s');
    assert!(
        status(&h).contains("environment key is invalid"),
        "{}",
        status(&h)
    );
    assert_eq!(h.app.route, Route::Editor);

    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('a'));
    h.key(KeyCode::Enter);
    h.type_str("DEPLOY_TOKEN");
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("sk-live-test-secret-1234");
    h.key(KeyCode::Tab);
    h.tab_to(WidgetId::of("editor.cfg").sub("form").sub("save"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("DEPLOY_TOKEN"), "{}", h.text());
    assert!(h.text().contains("1234"), "{}", h.text());
    assert!(
        !h.text().contains("test-secret"),
        "secret leaked: {}",
        h.text()
    );
}

#[test]
fn accounts_edit_toggle_validate_and_refresh_update_only_the_selected_account() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    h.app.screens.accounts.selected =
        crate::screens::accounts::Sel::Account("acct-claude-work".into());
    h.draw();
    h.key(KeyCode::Char('e'));
    h.tab_to(crate::screens::accounts::FORM.sub("purpose"));
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("coverage");
    h.key(KeyCode::Enter);
    h.tab_to(crate::screens::accounts::FORM.sub("save"));
    h.key(KeyCode::Enter);
    assert_eq!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .purpose
            .as_deref(),
        Some("coverage")
    );

    h.key(KeyCode::Char('d'));
    assert!(
        !h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .enabled
    );
    h.key(KeyCode::Char('d'));
    assert!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .enabled
    );
    h.key(KeyCode::Char('v'));
    assert!(h.text().contains("Validating"), "{}", h.text());
    h.ticks(15);
    assert!(!matches!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .validation,
        crate::domain::account::ValidationState::Validating { .. }
    ));
    h.key(KeyCode::Char('r'));
    assert_eq!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .usage
            .freshness
            .phase,
        Freshness::Refreshing
    );
    for _ in 0..40 {
        if h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .usage
            .freshness
            .phase
            != Freshness::Refreshing
        {
            break;
        }
        h.ticks(1);
    }
    assert_ne!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .usage
            .freshness
            .phase,
        Freshness::Refreshing
    );
    assert!(status(&h).contains("Refreshed"), "{}", status(&h));
}

#[test]
fn usage_refresh_detail_and_scroll_are_read_only() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.go(Go::Usage {
        select: Some("acct-claude-work".into()),
    });
    h.draw();
    let before = h.app.world.last_refresh_secs;
    h.key(KeyCode::Enter);
    assert!(h.app.screens.usage.show_detail);
    assert!(
        h.text().contains("Provider") || h.text().contains("Account"),
        "{}",
        h.text()
    );
    h.key(KeyCode::PageDown);
    h.mouse(MouseKind::WheelDown, 95, 20);
    h.key(KeyCode::Char('r'));
    assert!(h.app.screens.usage.refreshing);
    assert_eq!(
        h.app
            .world
            .accounts
            .get("acct-claude-work")
            .unwrap()
            .usage
            .freshness
            .phase,
        Freshness::Refreshing
    );
    assert_eq!(h.app.world.last_refresh_secs, before);
    h.key(KeyCode::Esc);
    assert!(!h.app.screens.usage.show_detail);
    h.key(KeyCode::Esc);
    assert_eq!(h.app.route, Route::Manager);
}

#[test]
fn settings_cancel_keeps_edits_and_discard_drops_them() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.go(Go::Settings);
    h.draw();
    let original = h.app.screens.settings.as_ref().unwrap().pending.clone();
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));
    assert_ne!(h.app.screens.settings.as_ref().unwrap().pending, original);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Settings);
    assert_ne!(h.app.screens.settings.as_ref().unwrap().pending, original);

    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.go(Go::Settings);
    h.draw();
    let original = h.app.screens.settings.as_ref().unwrap().pending.clone();
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    h.tab_to(WidgetId::of("settings.exit.discard"));
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert!(h.app.screens.settings.is_none());
    assert_eq!(h.app.world.global, original);
}

#[test]
fn prelude_invalid_destination_and_backtracking_do_not_create_workspace() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.app.go(Go::Prelude);
    h.draw();
    h.key(KeyCode::Backspace);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("relative/path");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("absolute path"), "{}", h.text());
    assert_eq!(h.app.route, Route::Prelude);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("step 2 of 5"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.text().contains("step 1 of 5"), "{}", h.text());
    assert_eq!(h.app.world.workspaces.len(), 4);
}

#[test]
fn launch_failure_screen_exposes_stage_detail_and_ack_route() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Reduced, 0, 120, 40);
    for _ in 0..120 {
        if h.app
            .screens
            .cockpit
            .as_ref()
            .is_some_and(|c| c.run.failure.is_some())
        {
            break;
        }
        h.ticks(3);
    }
    let c = h.app.screens.cockpit.as_ref().unwrap();
    let f = c.run.failure.as_ref().unwrap();
    assert_eq!(f.stage, crate::sim::launch::Stage::Network);
    assert!(
        f.detail
            .iter()
            .any(|line| line.contains("network jackin-net"))
    );
    assert!(h.text().contains("Launch failed"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app.route, Route::Manager);
    assert!(status(&h).contains("still running"), "{}", status(&h));
}
