//! Production manager selection follows durable rows across projection changes.

use jackin_app::{App, MANAGER_LIST, Motion, Scenario};
use junie_tui::{KeyCode, Theme};
use junie_tui_testing::Harness;

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
    assert_eq!(h.app().route(), jackin_app::Route::Capsule);
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
        jackin_app::screens::manager::ManagerRowKey::Instance(_)
    ));
    h.app_mut().world.workspaces.swap(0, 1);
    h.app_mut().manager.invalidate_rows();
    let _ = h.tick();
    assert_eq!(h.app().manager.selected_row(), &selected);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().route(), jackin_app::Route::Capsule);
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
    let child = jackin_app::screens::manager::ManagerRowKey::Instance(instance.id.clone());
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
        &jackin_app::screens::manager::ManagerRowKey::CurrentDirectory
    );
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.app().manager.selected_row(),
        &jackin_app::screens::manager::ManagerRowKey::CurrentDirectory
    );
    assert!(!h.app().manager.detail_open());
    Ok(())
}
