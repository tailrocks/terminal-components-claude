use jackin_preview_app::screens::manager::{self, ManagerRowKey};
use jackin_preview_app::{
    ACCOUNTS_LIST, App, CAPSULE_PANES, MANAGER, MANAGER_LIST, Motion, Route, Scenario,
};
use termrock::{Id, ItemKey, KeyCode, Theme};
use termrock_test_support::Harness;

const MANAGER_DETAIL_ACTION: Id = manager::DETAIL.sub("activate-selected");

fn harness(theme: Theme, width: u16, height: u16) -> Harness<App> {
    Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Paused, 40),
        theme,
        width,
        height,
    )
}

fn focus_list_from_shell(h: &mut Harness<App>) {
    assert_eq!(h.focus(), Some(MANAGER));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
    assert!(h.app().manager.detail_open());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(MANAGER_LIST));
    assert!(!h.app().manager.detail_open());
}

fn assert_split_geometry(h: &Harness<App>) {
    let master = h.area_of(MANAGER_LIST);
    assert!(master.is_some(), "split registers its master list");
    let Some(master) = master else { return };
    let detail = h.area_of(manager::DETAIL);
    assert!(detail.is_some(), "split registers its detail owner");
    let Some(detail) = detail else { return };
    let action = h.area_of(MANAGER_DETAIL_ACTION);
    assert!(action.is_some(), "split registers its real detail action");
    let Some(action) = action else { return };
    assert!(master.width > 0 && detail.width > 0 && action.width > 0);
    assert!(
        master.right() <= detail.x,
        "master {master:?}, detail {detail:?}"
    );
    assert_eq!(detail.x, action.x);
}

fn assert_drawer_geometry(h: &Harness<App>, content_x: u16) {
    assert_eq!(
        h.area_of(MANAGER_LIST),
        None,
        "drawer hides its master owner"
    );
    let detail = h.area_of(manager::DETAIL);
    assert!(detail.is_some(), "drawer registers its detail owner");
    let Some(detail) = detail else { return };
    let action = h.area_of(MANAGER_DETAIL_ACTION);
    assert!(action.is_some(), "drawer registers its real detail action");
    let Some(action) = action else { return };
    assert_eq!(detail.x, content_x);
    assert_eq!(action.x, content_x);
    assert!(detail.width > 0 && action.width > 0);
}

fn assert_selected_row_is_painted(h: &Harness<App>) {
    let selected = h.app().manager.selected_row();
    let key = selected.stable_key();
    let text = h.text();
    assert!(text.contains(key.as_str()), "{text}");
    match selected {
        ManagerRowKey::Workspace(id) => {
            let workspace = h.app().world.workspace(*id);
            assert!(workspace.is_some(), "selected World workspace");
            let Some(workspace) = workspace else { return };
            assert!(
                text.contains(&format!("Workspaces › {}", workspace.name)),
                "{text}"
            );
            assert!(
                text.contains(&format!("Workdir · {}", workspace.workdir)),
                "{text}"
            );
        }
        ManagerRowKey::CurrentDirectory => {
            assert!(text.contains("Current directory"), "{text}");
            assert!(text.contains(&h.app().world.cwd), "{text}");
            assert!(text.contains(&h.app().world.home), "{text}");
        }
        ManagerRowKey::Instance(id) => {
            assert!(text.contains(&format!("Instance › {id}")), "{text}");
        }
        ManagerRowKey::NewWorkspace => {
            assert!(text.contains("New workspace"), "{text}");
        }
    }
}

#[test]
fn tab_from_shell_and_list_opens_live_detail_across_the_breakpoint() {
    for theme in [Theme::paper(), Theme::junie()] {
        let content_x = theme.design.space.card_inset.saturating_mul(2);
        for (width, height) in [(120, 40), (80, 24)] {
            let mut h = harness(theme.clone(), width, height);
            assert_eq!(h.focus(), Some(MANAGER));

            // The shell shortcut opens the same Manager-owned detail and
            // remembers MANAGER_LIST as its close destination.
            let _ = h.key(KeyCode::Tab);
            assert!(h.app().manager.detail_open());
            assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
            assert_selected_row_is_painted(&h);
            if width >= 100 {
                assert_split_geometry(&h);
            } else {
                assert_drawer_geometry(&h, content_x);
            }
            let _ = h.key(KeyCode::Esc);
            assert_eq!(h.focus(), Some(MANAGER_LIST));
            assert!(!h.app().manager.detail_open());

            // The actual List owner can open the detail through the same
            // MasterDetail action while the default frame remains closed.
            let _ = h.key(KeyCode::Home);
            let _ = h.key(KeyCode::Tab);
            assert!(h.app().manager.detail_open());
            assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
            assert_selected_row_is_painted(&h);
            if width >= 100 {
                assert_split_geometry(&h);
            } else {
                assert_drawer_geometry(&h, content_x);
            }
            assert_eq!(h.diagnostics(), []);
        }
    }
}

#[test]
fn detail_close_keys_restore_the_real_master_focus() {
    for theme in [Theme::paper(), Theme::junie()] {
        let content_x = theme.design.space.card_inset.saturating_mul(2);
        for (width, height) in [(120, 40), (80, 24)] {
            for key in [KeyCode::Esc, KeyCode::Left, KeyCode::BackTab] {
                let mut h = harness(theme.clone(), width, height);
                focus_list_from_shell(&mut h);
                let _ = h.key(KeyCode::Home);
                let _ = h.key(KeyCode::Tab);
                assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
                assert!(h.app().manager.detail_open());
                if width >= 100 {
                    assert_split_geometry(&h);
                } else {
                    assert_drawer_geometry(&h, content_x);
                }

                let _ = h.key(key);
                assert_eq!(h.focus(), Some(MANAGER_LIST));
                assert!(!h.app().manager.detail_open());
                assert_eq!(h.app().route(), Route::Manager);
                assert_eq!(h.diagnostics(), []);
            }
        }
    }
}

#[test]
fn resize_across_split_and_drawer_preserves_selected_detail() {
    let theme = Theme::junie();
    let content_x = theme.design.space.card_inset.saturating_mul(2);
    let mut h = harness(theme, 120, 40);
    focus_list_from_shell(&mut h);
    let _ = h.key(KeyCode::Home);
    let selected = h.app().manager.selected_row().clone();
    let _ = h.key(KeyCode::Tab);

    assert_selected_row_is_painted(&h);
    let _ = h.resize(80, 24);
    assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
    assert_eq!(h.app().manager.selected_row(), &selected);
    assert!(h.app().manager.detail_open());
    assert_selected_row_is_painted(&h);
    assert_drawer_geometry(&h, content_x);

    let _ = h.resize(120, 40);
    assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
    assert_eq!(h.app().manager.selected_row(), &selected);
    assert!(h.app().manager.detail_open());
    assert_selected_row_is_painted(&h);
    assert_split_geometry(&h);
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn detail_primary_action_reuses_the_selected_row_reconnect_path() {
    for theme in [Theme::paper(), Theme::junie()] {
        let mut h = harness(theme, 120, 40);
        focus_list_from_shell(&mut h);
        let _ = h.key(KeyCode::Tab);
        assert!(!h.is_open(manager::AGENT_PICKER));
        assert!(h.text().contains("Reconnect jk-7f3a"));
        let selected_row = h.app().manager.selected_row().clone();
        let pane = h
            .app()
            .world
            .daemons
            .get("jk-7f3a")
            .and_then(jackin_preview_app::sim::pty::Daemon::focused_pane);
        assert!(
            pane.is_some(),
            "the reconnect action's running instance has a focused pane"
        );
        let Some(pane) = pane else { return };
        let pane_owner = CAPSULE_PANES.item(ItemKey::num(pane));

        let _ = h.key(KeyCode::Enter);
        assert!(!h.is_open(manager::AGENT_PICKER));
        assert_eq!(h.app().route(), Route::Capsule);
        assert!(!h.app().manager.detail_open());
        assert_eq!(h.area_of(manager::DETAIL), None);
        assert_eq!(h.area_of(MANAGER_DETAIL_ACTION), None);
        assert!(h.area_of(jackin_preview_app::CAPSULE_TABS).is_some());
        assert!(h.area_of(pane_owner).is_some());
        assert_eq!(h.app().manager.selected_row(), &selected_row);
        assert_eq!(h.focus(), Some(pane_owner));
        assert!(h.cursor().is_some());
        assert_eq!(h.diagnostics(), []);

        h.draw();
        assert_eq!(h.focus(), Some(pane_owner));
        assert!(h.cursor().is_some());
        assert_eq!(h.diagnostics(), []);
    }
}

#[test]
fn accounts_navigation_remains_independent_while_manager_detail_is_open() {
    let mut h = harness(Theme::paper(), 120, 40);
    focus_list_from_shell(&mut h);
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Tab);
    assert!(h.app().manager.detail_open());

    let _ = h.key(KeyCode::Char('a'));
    assert_eq!(h.app().route(), Route::Accounts);
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
    assert!(!h.app().manager.detail_open());
    assert!(h.text().contains("Accounts"));
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn accounts_escape_returns_with_manager_detail_closed() {
    let mut h = harness(Theme::paper(), 120, 40);
    focus_list_from_shell(&mut h);
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(MANAGER_DETAIL_ACTION));
    assert!(h.app().manager.detail_open());

    let _ = h.key(KeyCode::Char('a'));
    assert_eq!(h.app().route(), Route::Accounts);
    assert_eq!(h.focus(), Some(ACCOUNTS_LIST));
    assert!(!h.app().manager.detail_open());
    assert_eq!(h.area_of(MANAGER_DETAIL_ACTION), None);
    assert_eq!(h.diagnostics(), []);

    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
    assert_eq!(h.focus(), Some(manager::TREE));
    assert!(!h.app().manager.detail_open());
    assert_eq!(h.area_of(MANAGER_DETAIL_ACTION), None);
    assert_eq!(h.area_of(manager::DETAIL), None);
    assert_eq!(h.diagnostics(), []);
}
