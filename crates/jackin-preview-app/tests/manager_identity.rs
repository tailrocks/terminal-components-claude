use jackin_preview_app::screens::manager::ManagerRowKey;
use jackin_preview_app::{ACCOUNTS, App, MANAGER_LIST, Motion, Scenario};
use termrock::{Axis, ItemKey, KeyCode, MouseKind, Part, PartRef, Rect, StateFlags, Theme};
use termrock_test_support::Harness;

fn hard_cases_manager() -> Harness<App> {
    let mut h = Harness::new(
        App::for_scenario_at(Scenario::HardCases, Motion::Paused, 0),
        Theme::paper(),
        100,
        30,
    );
    assert!(h.tab_to(MANAGER_LIST));
    let _ = h.key(KeyCode::Home);
    h
}

fn workspace_part(id: u32) -> PartRef {
    let key = ManagerRowKey::Workspace(id).stable_key();
    PartRef::item(Part::ROW, ItemKey::text(&key))
}

#[test]
fn ordinary_manager_pointer_chose_opens_live_detail() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = Harness::new(
        App::for_scenario_at(Scenario::Returning, Motion::Full, 0),
        Theme::junie(),
        120,
        40,
    );
    assert!(h.tab_to(MANAGER_LIST));
    let _ = h.key(KeyCode::Home);
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .ok_or("Returning second workspace")?;
    let target_id = target.id;
    let target_name = target.name.clone();
    let area = h
        .area_of_part(MANAGER_LIST, workspace_part(target_id))
        .ok_or("ordinary Manager workspace row")?;

    assert!(h.click(area.x, area.y).is_changed());
    assert_eq!(h.app().manager.selected(), Some(target_id));
    assert!(h.app().manager.detail_open());
    assert!(h.text().contains(&format!("Workspaces › {target_name}")));
    assert_eq!(h.diagnostics(), []);
    Ok(())
}

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
    assert!(matches!(selected, ManagerRowKey::Instance(_)));
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
    let child = ManagerRowKey::Instance(instance.id.clone());
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
        &ManagerRowKey::CurrentDirectory
    );
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.app().manager.selected_row(),
        &ManagerRowKey::CurrentDirectory
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
    let expected = ManagerRowKey::Instance(visible.id.clone());
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

#[test]
fn hard_cases_painted_workspace_row_matches_keyboard_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let mut pointer = hard_cases_manager();

    let (first_id, first_name, target_id, target_name) = {
        let workspaces = &pointer.app().world.workspaces;
        let first = workspaces.first().ok_or("HardCases first workspace")?;
        let target = workspaces.get(1).ok_or("HardCases second workspace")?;
        (first.id, first.name.clone(), target.id, target.name.clone())
    };

    // Keep this independent keyboard journey before pointer-specific assertions,
    // so it still provides a useful positive control if row dispatch is red.
    let mut keyboard = Harness::new(
        App::for_scenario_at(Scenario::HardCases, Motion::Paused, 0),
        Theme::paper(),
        100,
        30,
    );
    assert!(keyboard.tab_to(MANAGER_LIST));
    let _ = keyboard.key(KeyCode::Home);
    let keyboard_first = keyboard
        .app()
        .world
        .workspaces
        .first()
        .ok_or("HardCases first workspace")?
        .id;
    assert_eq!(keyboard.app().manager.selected(), Some(keyboard_first));
    let _ = keyboard.key(KeyCode::Down);
    assert_eq!(keyboard.app().manager.selected(), Some(target_id));
    assert_eq!(
        keyboard.app().manager.selected_row().workspace(),
        Some(target_id)
    );
    assert_eq!(keyboard.app().route(), jackin_preview_app::Route::Manager);
    assert!(!keyboard.app().manager.detail_open());
    assert_eq!(keyboard.focus(), Some(MANAGER_LIST));
    assert_eq!(keyboard.diagnostics(), []);

    assert_eq!(pointer.app().manager.selected(), Some(first_id));
    assert_eq!(pointer.focus(), Some(MANAGER_LIST));
    assert_eq!(pointer.diagnostics(), []);

    // The first name is also saved-target metadata on the selected detail row.
    // Assert the known leaf row directly instead of searching for its first
    // occurrence in the whole frame.
    assert!(
        pointer.row(4).contains(&first_name),
        "first live ManagerFrame leaf row 4: {:?}",
        pointer.row(4)
    );
    let target_row = pointer
        .find(&target_name)
        .ok_or("painted second workspace row")?;
    assert_eq!(target_row.1, 5, "second live ManagerFrame leaf row");
    assert!(pointer.row(target_row.1).contains(&target_name));
    assert_eq!(
        pointer.count(&target_name),
        1,
        "target identifies one painted row"
    );

    let target_part = workspace_part(target_id);
    let target_area = pointer
        .area_of_part(MANAGER_LIST, target_part)
        .ok_or("registered painted workspace row")?;
    assert_eq!(target_area, Rect::new(1, target_row.1, 98, 1));

    let press = pointer.mouse(MouseKind::Down, target_row.0, target_row.1);
    assert!(
        press.is_changed(),
        "press on the painted row must select its real model key"
    );
    assert_eq!(pointer.app().manager.selected(), Some(target_id));
    assert_eq!(
        pointer.app().manager.selected_row().workspace(),
        Some(target_id)
    );
    assert_eq!(pointer.app().route(), jackin_preview_app::Route::Manager);
    assert!(!pointer.app().manager.detail_open());
    assert!(pointer.state_of(MANAGER_LIST).contains(StateFlags::PRESSED));
    assert_eq!(pointer.runtime().capture_owner(), None);
    assert_eq!(
        pointer.area_of_part(MANAGER_LIST, target_part),
        Some(target_area),
        "pressed row keeps the same painted geometry through redraw"
    );

    let release = pointer.mouse(MouseKind::Up, target_row.0, target_row.1);
    assert!(
        release.is_changed(),
        "release on the pressed row chooses it"
    );
    assert_eq!(pointer.app().manager.selected(), Some(target_id));
    assert_eq!(
        pointer.app().manager.selected_row().workspace(),
        Some(target_id)
    );
    assert!(pointer.app().manager.detail_open());
    assert!(
        pointer
            .text()
            .contains(&format!("Workspaces › {target_name}")),
        "current List Chose action identifies its live target: {}",
        pointer.text()
    );
    assert_eq!(pointer.runtime().capture_owner(), None);
    assert_eq!(pointer.app().route(), jackin_preview_app::Route::Manager);
    assert_eq!(pointer.focus(), Some(MANAGER_LIST));
    assert_eq!(pointer.diagnostics(), []);
    Ok(())
}

#[test]
fn hard_cases_frame_chrome_has_no_workspace_row_hit() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = hard_cases_manager();
    let selected = h.app().manager.selected_row().clone();
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .ok_or("HardCases second workspace")?
        .id;
    let row_area = h
        .area_of_part(MANAGER_LIST, workspace_part(target))
        .ok_or("registered painted workspace row")?;
    let chrome = h.click(row_area.x.saturating_sub(1), row_area.y);

    assert!(!chrome.is_changed());
    assert_eq!(h.app().manager.selected_row(), &selected);
    assert!(!h.app().manager.detail_open());
    assert_eq!(h.app().route(), jackin_preview_app::Route::Manager);
    assert_eq!(h.focus(), Some(MANAGER_LIST));
    assert_eq!(
        h.area_of_part(MANAGER_LIST, workspace_part(target)),
        Some(row_area)
    );
    assert_eq!(h.diagnostics(), []);
    Ok(())
}

#[test]
fn hard_cases_row_release_outside_cancels_choose() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = hard_cases_manager();
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .ok_or("HardCases second workspace")?
        .id;
    let part = workspace_part(target);
    let area = h
        .area_of_part(MANAGER_LIST, part)
        .ok_or("registered painted workspace row")?;

    assert!(h.mouse(MouseKind::Down, area.x, area.y).is_changed());
    assert_eq!(h.area_of_part(MANAGER_LIST, part), Some(area));
    assert!(h.state_of(MANAGER_LIST).contains(StateFlags::PRESSED));
    assert_eq!(h.app().manager.selected(), Some(target));

    assert_eq!(h.runtime().capture_owner(), None);
    let release = h.mouse(MouseKind::Up, area.x.saturating_sub(1), area.y);

    assert!(!release.is_changed());
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(!h.app().manager.detail_open());
    assert_eq!(h.app().manager.selected(), Some(target));
    assert_eq!(h.app().route(), jackin_preview_app::Route::Manager);
    assert_eq!(h.focus(), Some(MANAGER_LIST));
    assert!(!h.state_of(MANAGER_LIST).contains(StateFlags::PRESSED));
    assert_eq!(h.diagnostics(), []);
    Ok(())
}

#[test]
fn hard_cases_painted_workspace_keys_survive_expanded_model_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let mut h = hard_cases_manager();
    let first = h
        .app()
        .world
        .workspaces
        .first()
        .ok_or("HardCases first workspace")?
        .id;
    let target = h
        .app()
        .world
        .workspaces
        .get(1)
        .ok_or("HardCases second workspace")?
        .id;
    let _ = h.key(KeyCode::Right);
    assert!(h.app().manager.is_expanded(first));

    let part = workspace_part(target);
    let area = h
        .area_of_part(MANAGER_LIST, part)
        .ok_or("mapped second workspace row")?;
    assert_eq!(area.y, 5);
    assert!(h.click(area.x, area.y).is_changed());
    assert_eq!(h.app().manager.selected(), Some(target));
    assert_eq!(h.app().manager.selected_row().workspace(), Some(target));
    assert!(h.app().manager.detail_open());
    assert_eq!(h.diagnostics(), []);
    Ok(())
}

#[test]
fn hard_cases_scroll_hands_off_to_stock_list_rows() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = hard_cases_manager();
    let template = h
        .app()
        .world
        .workspaces
        .first()
        .cloned()
        .ok_or("HardCases first workspace")?;
    let first_id = template.id;
    let next_id = h
        .app()
        .world
        .workspaces
        .iter()
        .map(|workspace| workspace.id)
        .max()
        .ok_or("HardCases workspaces")?
        .checked_add(1)
        .ok_or("HardCases workspace id space")?;
    {
        let app = h.app_mut();
        for index in 0..24_u32 {
            let mut workspace = template.clone();
            workspace.id = next_id
                .checked_add(index)
                .ok_or("HardCases workspace id space")?;
            workspace.name = format!("scroll-workspace-{index:02}");
            workspace.workdir = format!("/workspace/scroll-{index:02}");
            app.world.workspaces.push(workspace);
        }
        app.manager.invalidate_rows();
    }
    let _ = h.key(KeyCode::Home);
    let first_part = workspace_part(first_id);
    let first_area = h
        .area_of_part(MANAGER_LIST, first_part)
        .ok_or("first frame workspace row")?;
    assert_eq!(h.app().manager.list.scroll().offset(), 0);

    let wheel = h.wheel(Axis::V, 1, first_area.x, first_area.y);

    assert!(
        wheel.is_changed(),
        "wheel advances the real List scroll state"
    );
    assert!(h.app().manager.list.scroll().offset() > 0);
    assert_eq!(h.app().manager.selected(), Some(first_id));
    assert!(
        h.area_of_part(MANAGER_LIST, first_part).is_none(),
        "the scrolled stock List no longer publishes its offscreen first row"
    );
    assert_eq!(h.focus(), Some(MANAGER_LIST));
    assert_eq!(h.diagnostics(), []);
    Ok(())
}
