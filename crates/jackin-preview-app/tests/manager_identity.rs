use jackin_preview_app::{ACCOUNTS, App, MANAGER_LIST, Motion, Scenario};
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

#[test]
fn tab_from_the_manager_paints_the_detail_drawer_hints() {
    let mut h = Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(h.text().contains("Tab Details"), "{}", h.text());
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(ACCOUNTS));
    let text = h.text();
    assert!(text.contains("Tab Actions"), "{text}");
    assert!(text.contains("Esc Back"), "{text}");
    assert!(!text.contains("Tab Details"), "{text}");
}

#[test]
fn detail_drawer_repaints_from_live_world_data() {
    let mut h = Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let _ = h.key(KeyCode::Tab);
    let text = h.text();
    assert!(
        text.contains("Current directory · payments-platform"),
        "{text}"
    );
    assert!(text.contains("↑↓"), "{text}");
    // The drawer composes over the live split: shifting the daemon age in
    // the world must reach the painted frame with no fixed-cell overlay.
    h.app_mut().world.last_refresh_secs -= 3600;
    let _ = h.tick();
    assert_ne!(text, h.text(), "drawer must repaint from live world data");
}

#[test]
fn inspect_shortcut_paints_the_container_dialog() {
    let mut h = Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char('i'));
    let text = h.text();
    assert!(text.contains("Container 7f3a"), "{text}");
    assert!(text.contains("read-only"), "{text}");
    assert!(text.contains("Copy"), "{text}");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("Container 7f3a"), "{}", h.text());
    assert!(!h.app().should_quit());
}

#[test]
fn selected_workspace_survives_world_reordering() {
    let mut h = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    assert!(h.app().world.workspaces.len() >= 2);
    assert!(h.tab_to(MANAGER_LIST));
    let _ = h.key(KeyCode::Home);
    let selected = h.app().manager.selected_row().clone();
    h.app_mut().world.workspaces.swap(0, 1);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
}

fn manager() -> Harness<App> {
    let mut h = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Paused),
        Theme::junie(),
        120,
        40,
    );
    assert!(h.tab_to(MANAGER_LIST));
    let _ = h.key(KeyCode::Home);
    h
}

#[test]
fn insertion_removal_and_rename_preserve_selected_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let mut h = manager();
    let selected = h.app().manager.selected_row().clone();
    let key = h.app().manager.list.cursor();
    let mut added = h
        .app()
        .world
        .workspaces
        .first()
        .ok_or("workspace fixture")?
        .clone();
    added.id = 999;
    added.name = "inserted above".into();
    h.app_mut().world.workspaces.insert(0, added);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    assert_eq!(h.app().manager.list.cursor(), key);
    h.app_mut()
        .world
        .workspaces
        .get_mut(1)
        .ok_or("workspace fixture")?
        .name = "renamed same object".into();
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.list.cursor(), key);
    h.app_mut().world.workspaces.remove(0);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    Ok(())
}

#[test]
fn expansion_above_selection_preserves_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = manager();
    let _ = h.key(KeyCode::Down);
    let selected = h.app().manager.selected_row().clone();
    let above = h
        .app()
        .world
        .workspaces
        .first()
        .ok_or("workspace fixture")?
        .id;
    h.app_mut().manager.toggle(above);
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    h.app_mut().manager.toggle(above);
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    Ok(())
}

#[test]
fn removed_selection_reconciles_to_an_existing_domain_record()
-> Result<(), Box<dyn std::error::Error>> {
    let mut h = manager();
    h.app_mut().world.workspaces.remove(0);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(
        h.app().manager.selected(),
        Some(
            h.app()
                .world
                .workspaces
                .first()
                .ok_or("workspace fixture")?
                .id
        )
    );
    Ok(())
}

#[test]
fn activation_after_reorder_uses_selected_workspace_daemon()
-> Result<(), Box<dyn std::error::Error>> {
    let mut h = manager();
    let workspace = h.app().manager.selected().ok_or("activation fixture")?;
    let instance = h
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.workspace == Some(workspace) && instance.status.reconnectable())
        .ok_or("activation fixture")?
        .id
        .clone();
    let pane = h
        .app()
        .world
        .daemons
        .get(&instance)
        .ok_or("activation fixture")?
        .focused_pane()
        .ok_or("activation fixture")?;
    h.app_mut().world.workspaces.swap(0, 1);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), jackin_preview_app::Route::Capsule);
    assert_eq!(h.app().capsule.selected_pane, pane);
    assert_eq!(h.app().manager.selected(), Some(workspace));
    Ok(())
}

#[test]
fn instance_identity_survives_expansion_and_reorder_above_it()
-> Result<(), Box<dyn std::error::Error>> {
    let mut h = manager();
    let workspace = h.app().manager.selected().ok_or("workspace fixture")?;
    h.app_mut().manager.toggle(workspace);
    let _ = h.tick();
    let _ = h.key(KeyCode::Down);
    let selected = h.app().manager.selected_row().clone();
    assert!(matches!(
        selected,
        jackin_preview_app::screens::manager::ManagerRowKey::Instance(_)
    ));
    h.app_mut().world.workspaces.swap(0, 1);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), jackin_preview_app::Route::Capsule);
    assert_eq!(h.app().manager.selected_row(), &selected);
    Ok(())
}

#[test]
fn right_expands_selected_workspace_then_enters_its_child() -> Result<(), Box<dyn std::error::Error>>
{
    let mut h = manager();
    let first = h
        .app()
        .world
        .workspaces
        .first()
        .ok_or("first workspace")?
        .id;
    let second = h
        .app()
        .world
        .workspaces
        .get(1)
        .ok_or("second workspace")?
        .id;
    let instance = h.app_mut().world.instances.first_mut().ok_or("instance")?;
    instance.workspace = Some(second);
    let child = jackin_preview_app::screens::manager::ManagerRowKey::Instance(instance.id.clone());
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().manager.selected(), Some(second));
    let _ = h.key(KeyCode::Right);
    assert!(h.app().manager.is_expanded(second));
    assert!(!h.app().manager.is_expanded(first));
    assert_eq!(h.app().manager.selected(), Some(second));
    assert!(!h.app().manager.detail_open());
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().manager.selected_row(), &child);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().manager.selected_row(), &child);
    assert!(h.app().manager.is_expanded(second));
    Ok(())
}

#[test]
fn right_on_current_directory_and_empty_workspace_is_noop() -> Result<(), Box<dyn std::error::Error>>
{
    let mut h = manager();
    h.app_mut().world.instances.clear();
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    let selected = h.app().manager.selected_row().clone();
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().manager.selected_row(), &selected);
    assert!(
        !h.app()
            .manager
            .is_expanded(h.app().manager.selected().ok_or("workspace")?)
    );
    for _ in 0..h.app().world.workspaces.len() {
        let _ = h.key(KeyCode::Down);
    }
    assert_eq!(
        h.app().manager.selected_row(),
        &jackin_preview_app::screens::manager::ManagerRowKey::CurrentDirectory
    );
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.app().manager.selected_row(),
        &jackin_preview_app::screens::manager::ManagerRowKey::CurrentDirectory
    );
    assert!(!h.app().manager.detail_open());
    Ok(())
}

#[test]
fn hidden_only_children_do_not_make_workspace_expandable() -> Result<(), Box<dyn std::error::Error>>
{
    use jackin_preview_app::domain::instance::InstanceStatus;
    for status in [InstanceStatus::Purged, InstanceStatus::Superseded] {
        let mut h = manager();
        let selected = h.app().manager.selected().ok_or("workspace")?;
        for instance in &mut h.app_mut().world.instances {
            instance.workspace = Some(selected);
            instance.status = status;
        }
        h.app_mut().manager.invalidate_rows();
        let _ = h.tick();
        let _ = h.key(KeyCode::Right);
        assert!(!h.app().manager.is_expanded(selected), "{status:?}");
        assert_eq!(h.app().manager.selected(), Some(selected));
    }
    Ok(())
}

#[test]
fn mixed_children_expand_and_enter_first_visible_record() -> Result<(), Box<dyn std::error::Error>>
{
    use jackin_preview_app::domain::instance::InstanceStatus;
    let mut h = manager();
    let selected = h.app().manager.selected().ok_or("workspace")?;
    let mut visible = h.app().world.instances.first().ok_or("instance")?.clone();
    visible.workspace = Some(selected);
    let expected =
        jackin_preview_app::screens::manager::ManagerRowKey::Instance(visible.id.clone());
    let mut hidden = visible.clone();
    hidden.id = "hidden-first".into();
    hidden.status = InstanceStatus::Purged;
    h.app_mut().world.instances = vec![hidden, visible];
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    let _ = h.key(KeyCode::Right);
    assert!(h.app().manager.is_expanded(selected));
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().manager.selected_row(), &expected);
    Ok(())
}
