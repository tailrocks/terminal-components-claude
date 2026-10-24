#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use jackin_preview_app::domain::account::{Account, CredentialSource, DetectedKind, Lifecycle};
use jackin_preview_app::domain::agent::{Agent, Provider};
use jackin_preview_app::domain::instance::InstanceStatus;
use jackin_preview_app::domain::workspace::{Mount, WorkspaceId};
use jackin_preview_app::screens::manager::{self, ManagerRowKey};
use jackin_preview_app::{App, MANAGER_LIST, Motion, Route, Scenario};
use termrock::{ItemKey, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

fn harness() -> Harness<App> {
    Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Full),
        Theme::junie(),
        120,
        40,
    )
}

fn paused_harness() -> Harness<App> {
    Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        120,
        40,
    )
}

fn row_part(row: &ManagerRowKey) -> PartRef {
    PartRef::item(Part::ROW, ItemKey::text(&row.stable_key()))
}

fn choose_row(h: &mut Harness<App>, row: &ManagerRowKey) {
    h.draw();
    let area = h
        .area_of_part(MANAGER_LIST, row_part(row))
        .expect("real registered Manager row");
    assert!(h.click(area.x, area.y).is_changed());
    assert_eq!(h.app().manager.selected_row(), row);
    assert!(h.app().manager.detail_open());
}

fn open_agent_picker(h: &mut Harness<App>, row: &ManagerRowKey) {
    choose_row(h, row);
    assert!(h.key(KeyCode::Enter).is_changed());
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.is_open(manager::AGENT_PICKER), "{}", h.text());
}

fn choose_claude(h: &mut Harness<App>) {
    h.draw();
    let area = h
        .area_of_part(
            manager::AGENT_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("claude")),
        )
        .expect("Claude is a real registered picker item");
    assert!(h.click(area.x, area.y).is_changed());
}

fn set_only_global_claude_default(h: &mut Harness<App>, id: &str) {
    for account in &mut h.app_mut().world.accounts.accounts {
        if account.provider == Provider::Anthropic {
            account.default_for_provider = false;
        }
    }
    add_available_claude_account(h, id, true);
}

fn add_available_claude_account(h: &mut Harness<App>, id: &str, default: bool) {
    let mut account = Account::registered(
        id,
        id,
        Provider::Anthropic,
        CredentialSource::LocalFolder {
            path: "~/.claude".into(),
            detected: DetectedKind::ClaudeOAuthProfile,
        },
    );
    account.lifecycle = Lifecycle::Available;
    account.default_for_provider = default;
    h.app_mut().world.accounts.insert(account);
}

fn make_workspace_offer(h: &mut Harness<App>, workspace: WorkspaceId, account: &str) {
    let default_ids = h
        .app()
        .world
        .accounts
        .accounts
        .iter()
        .filter(|account| account.provider == Provider::Anthropic && account.default_for_provider)
        .map(|account| account.id.clone())
        .collect::<Vec<_>>();
    add_available_claude_account(h, account, false);
    let target = h
        .app_mut()
        .world
        .workspace_mut(workspace)
        .expect("workspace exists");
    target.accounts.disabled_defaults.extend(default_ids);
    target.accounts.enabled.insert(account.to_owned());
    target.accounts.role_preferred.insert(
        ("chainargos/the-architect".to_owned(), Provider::Anthropic),
        account.to_owned(),
    );
}

fn advance_until(h: &mut Harness<App>, done: impl Fn(Route) -> bool) {
    let initial_time = h.now();
    for _ in 0..420 {
        if done(h.app().route()) {
            break;
        }
        let cadence = h.app().route_tick_ms();
        let _ = h.advance(std::time::Duration::from_millis(cadence));
    }
    assert_ne!(h.now(), initial_time, "Harness clock must advance");
}

fn finish_launch(h: &mut Harness<App>) {
    advance_until(h, |route| matches!(route, Route::Handoff | Route::Capsule));
    assert!(
        matches!(h.app().route(), Route::Handoff | Route::Capsule),
        "stale launch must not hand off: {:?} · {}",
        h.app().route(),
        h.text()
    );
    assert_eq!(h.diagnostics(), []);
}

fn launched_instance(h: &Harness<App>) -> &jackin_preview_app::domain::instance::Instance {
    let run_id = h
        .app()
        .launch()
        .expect("launch run remains inspectable")
        .run_id;
    h.app()
        .world
        .instances
        .iter()
        .find(|instance| instance.run_id == run_id)
        .expect("Ready materialized exactly this run")
}

#[test]
fn current_directory_activation_precedes_returning_fixture_reconnect() {
    let mut h = paused_harness();
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();
    let initial_run_id = h.app().launch().map(|launch| launch.run_id);
    let current_directory = ManagerRowKey::CurrentDirectory;
    choose_row(&mut h, &current_directory);
    assert!(h.key(KeyCode::Enter).is_changed());
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.is_open(manager::AGENT_PICKER), "{}", h.text());
    assert_eq!(h.app().world.instances.len(), initial_instances);
    assert_eq!(h.app().world.daemons.len(), initial_daemons);
    assert_eq!(h.app().launch().map(|launch| launch.run_id), initial_run_id);
    assert!(!h.text().contains("Container ready"));
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn returning_paused_instance_reconnect_keeps_the_selected_instance_identity() {
    let mut h = paused_harness();
    let instance = h
        .app()
        .world
        .instance("jk-9b02")
        .expect("second workspace instance");
    let instance_id = instance.id.clone();
    let workspace = instance.workspace.expect("instance workspace");
    let pane = h
        .app()
        .world
        .daemons
        .get(&instance_id)
        .and_then(jackin_preview_app::sim::pty::Daemon::focused_pane)
        .expect("instance focused pane");
    h.app_mut()
        .manager
        .select_row(ManagerRowKey::Workspace(workspace));
    h.app_mut().manager.toggle(workspace);
    let _ = h.tick();

    let instance_row = ManagerRowKey::Instance(instance_id);
    choose_row(&mut h, &instance_row);
    assert!(h.key(KeyCode::Enter).is_changed());

    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().capsule.selected_pane, pane);
    assert_eq!(
        h.focus(),
        Some(jackin_preview_app::CAPSULE_PANES.item(ItemKey::num(pane)))
    );
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn current_directory_launch_uses_global_offer_and_raw_cwd() {
    let mut h = harness();
    let first = h
        .app()
        .world
        .workspaces
        .first()
        .expect("first workspace")
        .id;
    h.app_mut().world.cwd = "/tmp/unmounted/customer-app".into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, first, "first-workspace-only");
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();

    open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
    assert!(h.text().contains("Current directory · customer-app"));
    choose_claude(&mut h);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert_eq!(
        h.app().launch().map(|run| run.agent),
        Some(Agent::ClaudeCode)
    );
    finish_launch(&mut h);

    let instance = launched_instance(&h);
    assert_eq!(instance.workspace, None);
    assert_eq!(instance.container, "jackin-customer-app");
    assert_eq!(instance.workdir, "/workspace/customer-app");
    assert_eq!(instance.role, "chainargos/the-architect");
    assert_eq!(
        instance.accounts.first().map(String::as_str),
        Some("cwd-global")
    );
    assert_eq!(h.app().world.instances.len(), initial_instances + 1);
    assert_eq!(h.app().world.daemons.len(), initial_daemons + 1);
}

#[test]
fn matched_current_directory_keeps_nonfirst_workspace_and_account_after_reorder() {
    let mut h = harness();
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .expect("non-first workspace")
        .clone();
    let cwd = "/tmp/monorepo/customer-portal";
    {
        let workspace = h.app_mut().world.workspace_mut(target.id).expect("target");
        workspace.workdir = "/workspace/customer-portal".into();
        workspace.mounts = vec![Mount::host(cwd, "/workspace/customer-portal")];
    }
    h.app_mut().world.cwd = cwd.into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, target.id, "matched-workspace");
    let expected_workdir = h
        .app()
        .world
        .workspace(target.id)
        .expect("configured target remains available")
        .workdir
        .clone();
    let initial_instances = h.app().world.instances.len();

    open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
    h.app_mut().world.workspaces.reverse();
    choose_claude(&mut h);
    assert_eq!(h.app().route(), Route::Cockpit);
    finish_launch(&mut h);

    let instance = launched_instance(&h);
    assert_eq!(instance.workspace, Some(target.id));
    assert_eq!(instance.workdir, expected_workdir);
    assert_eq!(instance.container, "jackin-customer-portal");
    assert_eq!(
        instance.accounts.first().map(String::as_str),
        Some("matched-workspace")
    );
    assert_eq!(instance.role, "chainargos/the-architect");
    assert_eq!(h.app().world.instances.len(), initial_instances + 1);
}

#[test]
fn nonfirst_workspace_launch_uses_selected_workspace() {
    let mut h = harness();
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .expect("second workspace")
        .clone();
    for instance in &mut h.app_mut().world.instances {
        if instance.workspace == Some(target.id) {
            instance.status = InstanceStatus::FailedSetup;
        }
    }
    assert!(
        h.app()
            .world
            .instances_of(Some(target.id))
            .iter()
            .all(|instance| !instance.status.reconnectable())
    );
    h.app_mut().world.cwd = "/tmp/unmounted/current".into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, target.id, "selected-workspace");
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();

    open_agent_picker(&mut h, &ManagerRowKey::Workspace(target.id));
    assert!(
        h.text().contains("infra-control-plane › chainargos/sre"),
        "Workspace default role is captured in the real picker: {}",
        h.text()
    );
    assert!(h.text().contains("Claude Code"), "{}", h.text());
    assert!(
        h.text().contains("selected-workspace"),
        "Workspace account offer remains available: {}",
        h.text()
    );
    choose_claude(&mut h);
    finish_launch(&mut h);
    let instance = launched_instance(&h);
    assert_eq!(instance.workspace, Some(target.id));
    assert_eq!(instance.role, "chainargos/sre");
    assert_eq!(instance.workdir, target.workdir);
    assert_eq!(
        instance.accounts.first().map(String::as_str),
        Some("selected-workspace")
    );
    assert_eq!(h.app().world.instances.len(), initial_instances + 1);
    assert_eq!(h.app().world.daemons.len(), initial_daemons + 1);
}

#[test]
fn workspace_without_default_uses_its_last_role() {
    let mut h = harness();
    let target_id = 4;
    let target = h
        .app()
        .world
        .workspace(target_id)
        .expect("customer portal workspace")
        .clone();
    assert_eq!(target.roles.default, None);
    assert_eq!(target.roles.last.as_deref(), Some("local/writer"));
    for instance in &mut h.app_mut().world.instances {
        if instance.workspace == Some(target.id) {
            instance.status = InstanceStatus::FailedSetup;
        }
    }
    h.app_mut().world.cwd = "/tmp/unmounted/current".into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, target.id, "selected-workspace");
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();

    open_agent_picker(&mut h, &ManagerRowKey::Workspace(target.id));
    assert!(
        h.text().contains("customer-portal › local/writer"),
        "last role is captured in the real picker: {}",
        h.text()
    );
    assert!(h.text().contains("Claude Code"), "{}", h.text());
    assert!(
        h.text().contains("selected-workspace"),
        "Workspace account offer remains available: {}",
        h.text()
    );
    choose_claude(&mut h);
    finish_launch(&mut h);
    let instance = launched_instance(&h);
    assert_eq!(instance.workspace, Some(target.id));
    assert_eq!(instance.role, "local/writer");
    assert_eq!(instance.workdir, target.workdir);
    assert_eq!(
        instance.accounts.first().map(String::as_str),
        Some("selected-workspace")
    );
    assert_eq!(h.app().world.instances.len(), initial_instances + 1);
    assert_eq!(h.app().world.daemons.len(), initial_daemons + 1);
}

#[test]
fn workspace_without_role_history_uses_architect_fallback() {
    let mut h = harness();
    let target_id = 1;
    {
        let target = h
            .app_mut()
            .world
            .workspace_mut(target_id)
            .expect("payments workspace");
        target.roles.default = None;
        target.roles.last = None;
    }
    let target = h
        .app()
        .world
        .workspace(target_id)
        .expect("payments workspace")
        .clone();
    for instance in &mut h.app_mut().world.instances {
        if instance.workspace == Some(target.id) {
            instance.status = InstanceStatus::FailedSetup;
        }
    }
    h.app_mut().world.cwd = "/tmp/unmounted/current".into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, target.id, "selected-workspace");
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();

    open_agent_picker(&mut h, &ManagerRowKey::Workspace(target.id));
    assert!(
        h.text()
            .contains("payments-platform › chainargos/the-architect"),
        "architect fallback is captured in the real picker: {}",
        h.text()
    );
    assert!(
        h.area_of_part(
            manager::AGENT_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("claude")),
        )
        .is_some(),
        "Claude is a real registered picker item: {}",
        h.text()
    );
    assert!(
        h.text().contains("selected-workspace"),
        "Workspace account offer remains available: {}",
        h.text()
    );
    choose_claude(&mut h);
    finish_launch(&mut h);
    let instance = launched_instance(&h);
    assert_eq!(instance.workspace, Some(target.id));
    assert_eq!(instance.role, "chainargos/the-architect");
    assert_eq!(instance.workdir, target.workdir);
    assert_eq!(
        instance.accounts.first().map(String::as_str),
        Some("selected-workspace")
    );
    assert_eq!(h.app().world.instances.len(), initial_instances + 1);
    assert_eq!(h.app().world.daemons.len(), initial_daemons + 1);
}

#[test]
fn new_workspace_activation_keeps_the_prelude_route() {
    let mut h = harness();
    assert!(h.key(KeyCode::Char('n')).is_changed());
    assert_eq!(h.app().route(), Route::Prelude);
    assert!(!h.is_open(manager::AGENT_PICKER));
}

#[test]
fn cancel_and_stale_before_picker_confirmation_create_no_run_or_instance() {
    let mut h = harness();
    h.app_mut().world.cwd = "/tmp/first-target".into();
    set_only_global_claude_default(&mut h, "cwd-global");
    let initial_instances = h.app().world.instances.len();
    open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
    assert!(h.key(KeyCode::Esc).is_changed());
    assert!(!h.is_open(manager::AGENT_PICKER));
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.app().launch().is_none());
    assert_eq!(h.app().world.instances.len(), initial_instances);

    open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
    h.app_mut().world.cwd = "/tmp/changed-before-confirm".into();
    choose_claude(&mut h);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(!h.is_open(manager::AGENT_PICKER));
    assert!(h.app().launch().is_none());
    assert_eq!(h.app().world.instances.len(), initial_instances);
    assert!(h.text().contains("Launch target changed"));
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn changed_matched_workspace_policy_before_picker_confirmation_creates_no_run() {
    let mut h = harness();
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .expect("non-first workspace")
        .clone();
    let cwd = "/tmp/monorepo/customer-portal";
    {
        let workspace = h.app_mut().world.workspace_mut(target.id).expect("target");
        workspace.workdir = "/workspace/customer-portal".into();
        workspace.mounts = vec![Mount::host(cwd, "/workspace/customer-portal")];
    }
    h.app_mut().world.cwd = cwd.into();
    set_only_global_claude_default(&mut h, "cwd-global");
    make_workspace_offer(&mut h, target.id, "matched-workspace");
    let initial_instances = h.app().world.instances.len();
    let initial_daemons = h.app().world.daemons.len();

    open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
    h.app_mut()
        .world
        .workspace_mut(target.id)
        .expect("target")
        .accounts
        .enabled
        .remove("matched-workspace");
    choose_claude(&mut h);

    assert_eq!(h.app().route(), Route::Manager);
    assert!(!h.is_open(manager::AGENT_PICKER));
    assert!(h.app().launch().is_none());
    assert_eq!(h.app().world.instances.len(), initial_instances);
    assert_eq!(h.app().world.daemons.len(), initial_daemons);
    assert!(h.text().contains("Launch target changed"), "{}", h.text());
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn stale_target_role_or_account_before_ready_never_materializes_or_hands_off() {
    #[derive(Clone, Copy)]
    enum Mutation {
        Cwd,
        Role,
        Account,
    }

    for mutation in [Mutation::Cwd, Mutation::Role, Mutation::Account] {
        let mut h = harness();
        h.app_mut().world.cwd = "/tmp/target-before-ready".into();
        set_only_global_claude_default(&mut h, "cwd-global");
        let initial_instances = h.app().world.instances.len();
        let initial_daemons = h.app().world.daemons.len();

        open_agent_picker(&mut h, &ManagerRowKey::CurrentDirectory);
        choose_claude(&mut h);
        assert_eq!(h.app().route(), Route::Cockpit);
        match mutation {
            Mutation::Cwd => h.app_mut().world.cwd.push_str("/changed"),
            Mutation::Role => h
                .app_mut()
                .world
                .roles
                .retain(|role| role.full_name() != "chainargos/the-architect"),
            Mutation::Account => h
                .app_mut()
                .world
                .accounts
                .accounts
                .retain(|account| account.id != "cwd-global"),
        }
        advance_until(&mut h, |route| route == Route::Manager);
        assert_eq!(h.app().route(), Route::Manager);
        assert!(h.app().launch().is_none());
        assert_eq!(h.app().world.instances.len(), initial_instances);
        assert_eq!(h.app().world.daemons.len(), initial_daemons);
        assert!(!h.text().contains("Capsule ready"));
        let expected = match mutation {
            Mutation::Cwd => "Launch target changed",
            Mutation::Role => "Selected role is no longer available",
            Mutation::Account => "Selected agent account is no longer available",
        };
        assert!(h.text().contains(expected), "{}", h.text());
        assert_eq!(h.diagnostics(), []);
    }
}
