#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "The test drives the public editor and launch journey"
)]

use jackin_preview_app::domain::instance::{Instance, InstanceStatus};
use jackin_preview_app::domain::workspace::AllowedRoles;
use jackin_preview_app::screens::manager::{self, ManagerRowKey};
use jackin_preview_app::{App, MANAGER_LIST, Motion, ROLE_PICKER, Route, Scenario};
use std::time::Duration;
use termrock::{Id, ItemKey, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const ROLE_EDIT: Id = Id::root("jackin.editor").sub("role-edit");
const SAVE_CONFIRM: Id = Id::root("jackin.editor").sub("save-confirm");

fn returning_harness(motion: Motion) -> Harness<App> {
    returning_harness_at_size(motion, 120, 40)
}

fn returning_harness_at_size(motion: Motion, width: u16, height: u16) -> Harness<App> {
    Harness::new(
        App::for_scenario(Scenario::Returning, motion),
        Theme::junie(),
        width,
        height,
    )
}

fn disable_reconnect_for_new_launch(h: &mut Harness<App>, workspace_id: u32) {
    let matching = h
        .app()
        .world
        .instances
        .iter()
        .filter(|instance| instance.workspace == Some(workspace_id))
        .count();
    assert!(matching > 0, "the launch fixture has a matching instance");
    for instance in h
        .app_mut()
        .world
        .instances
        .iter_mut()
        .filter(|instance| instance.workspace == Some(workspace_id))
    {
        instance.status = InstanceStatus::FailedSetup;
    }
    assert!(
        h.app()
            .world
            .instances
            .iter()
            .filter(|instance| instance.workspace == Some(workspace_id))
            .all(|instance| instance.status == InstanceStatus::FailedSetup),
        "the target has no reconnectable fixture instance"
    );
}

fn open_first_workspace_editor(h: &mut Harness<App>) -> u32 {
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char('e'));
    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    workspace_id
}

fn enter_roles(h: &mut Harness<App>) {
    let _ = h.key(KeyCode::Char('3'));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().editor.tab,
        jackin_preview_app::screens::editor::Tab::Roles
    );
}

fn open_default_picker(h: &mut Harness<App>) {
    assert!(h.tab_to(ROLE_EDIT), "Default role control is registered");
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(ROLE_PICKER), "{}", h.text());
    assert!(h.text().contains("Choose default role"), "{}", h.text());
}

fn save_editor(h: &mut Harness<App>) {
    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.preview_open, "{}", h.text());
    assert!(h.tab_to(SAVE_CONFIRM));
    let _ = h.key(KeyCode::Enter);
    for _ in 0..24 {
        let _ = h.advance(Duration::from_millis(200));
    }
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
}

fn select_sre_default(h: &mut Harness<App>) {
    open_default_picker(h);
    let _ = h.type_str("chainargos/sre");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some("chainargos/sre")
    );
}

fn launch_workspace(h: &mut Harness<App>, workspace_id: u32) -> Instance {
    let row = ManagerRowKey::Workspace(workspace_id);
    h.draw();
    let row_area = h
        .area_of_part(
            MANAGER_LIST,
            PartRef::item(Part::ROW, ItemKey::text(&row.stable_key())),
        )
        .expect("real Manager workspace row");
    assert!(h.click(row_area.x, row_area.y).is_changed());
    assert_eq!(h.app().manager.selected_row(), &row);
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(manager::AGENT_PICKER), "{}", h.text());

    h.draw();
    let agent = h
        .area_of_part(
            manager::AGENT_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("claude")),
        )
        .expect("configured Claude Agent row");
    assert!(h.click(agent.x, agent.y).is_changed());
    assert_eq!(h.app().route(), Route::Cockpit, "{}", h.text());
    for _ in 0..420 {
        if matches!(h.app().route(), Route::Handoff | Route::Capsule) {
            break;
        }
        let cadence = h.app().route_tick_ms();
        let _ = h.advance(Duration::from_millis(cadence));
    }
    assert!(matches!(h.app().route(), Route::Handoff | Route::Capsule));
    assert_eq!(h.diagnostics(), []);
    let run_id = h.app().launch().expect("real launch run").run_id;
    h.app()
        .world
        .instances
        .iter()
        .find(|instance| instance.run_id == run_id)
        .expect("the selected Agent materialized this launch")
        .clone()
}

fn assert_workspace_launch_uses_role(h: &mut Harness<App>, workspace_id: u32, role: &str) {
    let instance = launch_workspace(h, workspace_id);
    assert_eq!(instance.workspace, Some(workspace_id));
    assert_eq!(instance.role, role);
    assert!(
        instance
            .accounts
            .iter()
            .any(|account| account == "acct-claude-work")
    );
}

#[test]
fn selected_default_is_reviewed_saved_reopened_and_used_by_manager_launch() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    disable_reconnect_for_new_launch(&mut h, workspace_id);
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .expect("first workspace")
        .roles
        .allowed = AllowedRoles::All;
    let before_roles = h.app().world.workspace(workspace_id).unwrap().roles.clone();
    let before_registry = h.app().world.roles.clone();

    assert_eq!(open_first_workspace_editor(&mut h), workspace_id);
    enter_roles(&mut h);
    let before_pending = h.app().editor.pending.clone();
    assert!(h.text().contains("Default role ★ chainargos/the-architect"));
    assert!(
        !h.is_open(ROLE_PICKER),
        "selecting Roles must not choose a role"
    );
    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/s");
    assert!(h.text().contains("chainargos/sre"), "{}", h.text());
    assert!(h.text().contains("chainargos/svc-001"), "{}", h.text());
    assert!(
        h.area_of_part(
            ROLE_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("chainargos/sre"))
        )
        .is_some()
    );
    assert!(
        h.area_of_part(
            ROLE_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("chainargos/svc-001"))
        )
        .is_some()
    );
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);

    assert!(!h.is_open(ROLE_PICKER));
    assert!(h.app().editor.dirty);
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some("chainargos/svc-001")
    );
    assert_eq!(h.app().editor.pending.roles.allowed, before_roles.allowed);
    assert_eq!(h.app().editor.pending.roles.last, before_roles.last);
    assert_eq!(h.app().editor.pending.env, before_pending.env);
    assert_eq!(h.app().editor.pending.role_env, before_pending.role_env);
    assert_eq!(h.app().editor.pending.accounts, before_pending.accounts);
    assert_eq!(
        h.app().world.workspace(workspace_id).unwrap().roles,
        before_roles
    );
    assert_eq!(h.app().world.roles, before_registry);

    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.preview_open);
    assert!(
        h.text()
            .contains("~ default role chainargos/the-architect → chainargos/svc-001"),
        "{}",
        h.text()
    );
    assert!(h.tab_to(SAVE_CONFIRM));
    let _ = h.key(KeyCode::Enter);
    for _ in 0..24 {
        let _ = h.advance(Duration::from_millis(200));
    }
    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert_eq!(
        h.app()
            .world
            .workspace(workspace_id)
            .unwrap()
            .roles
            .default
            .as_deref(),
        Some("chainargos/svc-001")
    );

    let _ = h.key(KeyCode::Char('e'));
    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    enter_roles(&mut h);
    assert!(h.text().contains("Default role ★ chainargos/svc-001"));
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);

    assert_workspace_launch_uses_role(&mut h, workspace_id, "chainargos/svc-001");
}

#[test]
fn choosing_the_current_default_clears_it_and_manager_uses_last_role() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    disable_reconnect_for_new_launch(&mut h, workspace_id);
    {
        let workspace = h.app_mut().world.workspace_mut(workspace_id).unwrap();
        workspace.roles.allowed = AllowedRoles::All;
        workspace.roles.last = Some("chainargos/reviewer".into());
    }
    let before = h.app().world.workspace(workspace_id).unwrap().roles.clone();
    assert_eq!(before.default.as_deref(), Some("chainargos/the-architect"));

    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/the-architect");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().editor.pending.roles.default, None);
    assert_eq!(h.app().editor.pending.roles.last, before.last);
    assert_eq!(h.app().editor.pending.roles.allowed, before.allowed);
    assert!(h.app().editor.dirty);

    save_editor(&mut h);
    let _ = h.key(KeyCode::Char('e'));
    enter_roles(&mut h);
    assert!(h.text().contains("Default role ★ none"));
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);

    let instance = launch_workspace(&mut h, workspace_id);
    assert_eq!(instance.workspace, Some(workspace_id));
    assert_eq!(instance.role, "chainargos/reviewer");
}

#[test]
fn cancel_outside_dismissal_and_removed_workspace_never_change_the_default() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = open_first_workspace_editor(&mut h);
    let original = h.app().editor.pending.roles.clone();
    enter_roles(&mut h);
    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/s");
    let _ = h.key(KeyCode::Esc);
    assert!(h.is_open(ROLE_PICKER), "the first Esc clears the query");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROLE_PICKER));
    assert_eq!(h.focus(), Some(ROLE_EDIT));
    assert_eq!(h.app().editor.pending.roles, original);
    assert!(!h.app().editor.dirty);

    open_default_picker(&mut h);
    let _ = h.type_str("reviewer");
    let _ = h.click(0, 0);
    assert!(!h.is_open(ROLE_PICKER), "outside click dismisses the modal");
    assert_eq!(h.focus(), Some(ROLE_EDIT));
    assert_eq!(h.app().editor.pending.roles, original);
    assert!(!h.app().editor.dirty);

    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/sre");
    h.app_mut()
        .world
        .workspaces
        .retain(|workspace| workspace.id != workspace_id);
    let _ = h.key(KeyCode::Enter);
    assert!(!h.is_open(ROLE_PICKER));
    assert_eq!(h.app().editor.pending.roles, original);
    assert!(h.app().world.workspace(workspace_id).is_none());
}

#[test]
fn picker_uses_current_allowed_roles_and_supports_legacy_leaf_names() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::Custom(vec!["reviewer".into()]);
    let before = h.app().world.workspace(workspace_id).unwrap().roles.clone();
    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/sre");
    assert!(
        h.area_of_part(
            ROLE_PICKER,
            PartRef::item(Part::ROW, ItemKey::text("chainargos/sre"))
        )
        .is_none()
    );
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROLE_PICKER));
    assert_eq!(h.app().editor.pending.roles, before);
    assert!(!h.app().editor.dirty);

    open_default_picker(&mut h);
    let _ = h.type_str("reviewer");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some("chainargos/reviewer")
    );
    assert_eq!(h.app().editor.pending.roles.allowed, before.allowed);
}

#[test]
fn empty_allowlist_is_cancellable_and_does_not_mutate_the_draft() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::Custom(Vec::new());
    let before = h.app().world.workspace(workspace_id).unwrap().roles.clone();
    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    open_default_picker(&mut h);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().editor.pending.roles, before);
    assert!(!h.app().editor.dirty);
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROLE_PICKER));
    assert_eq!(h.focus(), Some(ROLE_EDIT));
}

#[test]
fn refreshed_policy_and_registry_reject_picker_rows_that_became_stale() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::All;
    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    let original = h.app().editor.pending.roles.clone();

    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/sre");
    let sre = PartRef::item(Part::ROW, ItemKey::text("chainargos/sre"));
    h.draw();
    assert!(h.area_of_part(ROLE_PICKER, sre).is_some());
    h.app_mut().editor.pending.roles.allowed =
        AllowedRoles::Custom(vec!["chainargos/backend".into()]);
    h.draw();
    assert!(h.area_of_part(ROLE_PICKER, sre).is_none());
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(ROLE_PICKER));
    assert_eq!(h.app().editor.pending.roles.default, original.default);
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().editor.pending.roles.default, original.default);
    assert!(!h.app().editor.dirty);

    h.app_mut().editor.pending.roles.allowed = AllowedRoles::All;
    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/sre");
    h.app_mut()
        .world
        .roles
        .retain(|role| role.full_name() != "chainargos/sre");
    h.draw();
    assert!(h.area_of_part(ROLE_PICKER, sre).is_none());
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(ROLE_PICKER));
    assert_eq!(h.app().editor.pending.roles, original);
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().editor.pending.roles, original);
    assert!(!h.app().editor.dirty);
}

#[test]
fn supported_minimum_roles_layout_keeps_native_edit_and_save_owners_visible() {
    let mut h = returning_harness_at_size(Motion::Paused, 120, 20);
    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    h.draw();
    assert!(h.text().contains("edit · Roles"), "{}", h.text());
    assert!(h.text().contains("Save workspace"), "{}", h.text());

    let role_area = h
        .area_of(ROLE_EDIT)
        .expect("the real default-role button is registered at 120×20");
    let save_area = h
        .area_of(jackin_preview_app::screens::editor::SAVE)
        .expect("the real Save button is registered at 120×20");
    assert!(role_area.width > 0 && role_area.height > 0, "{role_area:?}");
    assert!(save_area.width > 0 && save_area.height > 0, "{save_area:?}");
    assert!(
        role_area.bottom() <= 20,
        "default-role owner is clipped: {role_area:?}"
    );
    assert!(
        save_area.bottom() <= 20,
        "Save owner is clipped: {save_area:?}"
    );

    assert!(
        h.tab_to(ROLE_EDIT),
        "the compact default-role owner is focusable"
    );
    assert_eq!(h.focus(), Some(ROLE_EDIT));
    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::editor::SAVE));
    assert_eq!(h.diagnostics(), [], "{}", h.text());
}

#[test]
fn changed_target_after_default_save_admission_is_not_overwritten() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::All;
    let original = h.app().world.workspace(workspace_id).unwrap().clone();

    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    select_sre_default(&mut h);
    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.preview_open);
    assert!(h.tab_to(SAVE_CONFIRM));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.is_saving());

    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .keep_awake = !original.keep_awake;
    for _ in 0..24 {
        let _ = h.advance(Duration::from_millis(200));
    }

    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    assert!(h.text().contains("Workspace changed · nothing was written"));
    assert_eq!(
        h.app().world.workspace(workspace_id).unwrap().roles.default,
        original.roles.default
    );
    assert_eq!(
        h.app().world.workspace(workspace_id).unwrap().keep_awake,
        !original.keep_awake
    );
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some("chainargos/sre")
    );
    assert!(h.app().editor.dirty);
    assert!(!h.app().editor.preview_open);
    assert_eq!(h.diagnostics(), []);
}

#[test]
fn failed_default_save_keeps_the_draft_for_a_fresh_review_and_retry() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::All;
    h.app_mut().world.save_fails_once = true;

    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    select_sre_default(&mut h);
    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.preview_open);
    assert!(h.tab_to(SAVE_CONFIRM));
    let _ = h.key(KeyCode::Enter);
    for _ in 0..24 {
        let _ = h.advance(Duration::from_millis(200));
    }

    assert_eq!(h.app().route(), Route::Editor, "{}", h.text());
    assert!(h.text().contains("Save failed"));
    assert!(h.app().editor.dirty);
    assert!(!h.app().editor.preview_open);
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some("chainargos/sre")
    );
    assert_eq!(
        h.app()
            .world
            .workspace(workspace_id)
            .unwrap()
            .roles
            .default
            .as_deref(),
        Some("chainargos/the-architect")
    );

    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().editor.preview_open);
    assert!(
        h.text()
            .contains("~ default role chainargos/the-architect → chainargos/sre"),
        "{}",
        h.text()
    );
    assert!(h.tab_to(SAVE_CONFIRM));
    let _ = h.key(KeyCode::Enter);
    for _ in 0..24 {
        let _ = h.advance(Duration::from_millis(200));
    }

    assert_eq!(h.app().route(), Route::Manager, "{}", h.text());
    assert_eq!(
        h.app()
            .world
            .workspace(workspace_id)
            .unwrap()
            .roles
            .default
            .as_deref(),
        Some("chainargos/sre")
    );
}

#[test]
fn supported_compact_picker_scrolls_and_survives_resize_with_save_reachable() {
    let mut h = returning_harness(Motion::Reduced);
    let workspace_id = h
        .app()
        .world
        .workspaces
        .first()
        .expect("fixture workspace")
        .id;
    h.app_mut()
        .world
        .workspace_mut(workspace_id)
        .unwrap()
        .roles
        .allowed = AllowedRoles::All;
    let _ = open_first_workspace_editor(&mut h);
    enter_roles(&mut h);
    let _ = h.resize(80, 18);
    assert!(h.text().contains("Terminal too small"), "{}", h.text());
    assert!(h.area_of(ROLE_EDIT).is_none());
    let _ = h.resize(80, 20);
    assert!(!h.text().contains("Terminal too small"), "{}", h.text());
    h.draw();
    assert!(
        h.area_of(jackin_preview_app::screens::editor::SAVE)
            .is_some()
    );
    assert!(h.tab_to(jackin_preview_app::screens::editor::SAVE));

    open_default_picker(&mut h);
    let _ = h.type_str("chainargos/");
    let registry_keys: Vec<String> = h
        .app()
        .world
        .roles
        .iter()
        .filter(|role| role.in_registry && role.full_name().starts_with("chainargos/"))
        .map(jackin_preview_app::domain::workspace::RoleEntry::full_name)
        .collect();
    let target = "chainargos/svc-020";
    let target_index = registry_keys
        .iter()
        .position(|role| role == target)
        .expect("the visible test registry includes the selected deep role");
    assert!(
        target_index >= 8,
        "the target must begin beyond a compact page"
    );
    let item = PartRef::item(Part::ROW, ItemKey::text(target));
    h.draw();
    assert!(
        h.area_of_part(ROLE_PICKER, item).is_none(),
        "the deeper target starts outside the compact card viewport: {}",
        h.text()
    );
    for _ in 0..target_index {
        let _ = h.key(KeyCode::Down);
    }
    h.draw();
    assert!(
        h.area_of_part(ROLE_PICKER, item).is_some(),
        "down-navigation scrolls the real selected row into view: {}",
        h.text()
    );

    let _ = h.resize(120, 40);
    h.draw();
    assert!(h.area_of_part(ROLE_PICKER, item).is_some());
    assert_eq!(h.diagnostics(), []);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().editor.pending.roles.default.as_deref(),
        Some(target)
    );
    assert_eq!(h.diagnostics(), []);

    save_editor(&mut h);
    assert_eq!(
        h.app()
            .world
            .workspace(workspace_id)
            .unwrap()
            .roles
            .default
            .as_deref(),
        Some(target)
    );
}
