//! Row context menus on the plan, disk and activity screens, through the
//! real App (hit regions, focus and rendered cells).

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::app_tests_context::{
    assert_at_pointer, assert_below_row, choose, menu_area, right_click, shift_f10,
};
use crate::app_tests_parity::open;
use crate::domain::activity::ActivityState;
use crate::domain::plan::{PlanPhase, StepState};
use crate::scenario::{Motion, Scenario};
use crate::screens::{GateTarget, Go, Page};
use crate::screens::{activity, cleanup, disk, plan, review, snapshot};

fn until<F: Fn(&H) -> bool>(h: &mut H, max_ticks: usize, f: F) -> bool {
    for _ in 0..max_ticks {
        if f(h) {
            return true;
        }
        h.ticks(1);
    }
    f(h)
}

fn row(h: &H, id: junie_tui::core::id::WidgetId) -> Rect {
    h.app
        .hits
        .area_of(id)
        .unwrap_or_else(|| panic!("row {id:?} not drawn\n{}", h.text()))
}

// ------------------------------------------------------------------ plan

fn review() -> H {
    H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 120, 40)
}

#[test]
fn a_plan_step_right_click_opens_its_menu_at_the_pointer_and_toggles_it() {
    let mut h = review();
    let r = row(&h, plan::OUTLINE.child(5));
    let (x, y) = (r.x + 10, r.y);
    right_click(&mut h, x, y);
    assert_eq!(h.app.focus.current(), Some(plan::STEP_MENU));
    assert_at_pointer(&h, plan::STEP_MENU, x, y);
    let t = h.text();
    assert!(t.contains("06 "), "titled by the step\n{t}");
    assert!(
        t.contains("Exclude step") && t.contains("Discard plan"),
        "{t}"
    );
    assert!(h.app.world.plans[0].steps[5].included);
    choose(&mut h, "Exclude step");
    assert!(h.app.modals.is_empty());
    assert!(!h.app.world.plans[0].steps[5].included, "excluded");
    assert_eq!(h.app.focus.current(), Some(plan::OUTLINE));

    // a required step offers the toggle disabled, with its reason
    let r0 = row(&h, plan::OUTLINE.child(0));
    right_click(&mut h, r0.x + 10, r0.y);
    assert!(h.text().contains("Required by the plan"), "{}", h.text());
    choose(&mut h, "Exclude step");
    assert!(!h.app.modals.is_empty(), "a disabled row does not choose");
    assert!(h.app.world.plans[0].steps[0].included);
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn shift_f10_opens_the_step_menu_below_the_cursor_row() {
    let mut h = review();
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char(' '));
    assert!(!h.app.world.plans[0].steps[5].included);
    let r = row(&h, plan::OUTLINE.child(5));
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(plan::STEP_MENU), "{}", h.text());
    assert_below_row(&h, plan::STEP_MENU, r.y);
    choose(&mut h, "Undo last change");
    assert!(h.app.world.plans[0].steps[5].included, "undone");
}

#[test]
fn a_right_click_outside_the_step_menu_reopens_it_on_the_other_step() {
    let mut h = review();
    let r5 = row(&h, plan::OUTLINE.child(5));
    let r1 = row(&h, plan::OUTLINE.child(1));
    right_click(&mut h, r5.x + 10, r5.y);
    let first = menu_area(&h, plan::STEP_MENU);
    assert!(
        r1.y < first.y || r1.y >= first.bottom(),
        "step 02 is clear of the menu"
    );
    right_click(&mut h, r1.x + 10, r1.y);
    assert_eq!(h.app.modals.len(), 1);
    assert_at_pointer(&h, plan::STEP_MENU, r1.x + 10, r1.y);
    assert!(h.text().contains("02 "), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn a_running_plan_step_menu_retries_a_failed_step() {
    let mut h = review();
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(until(&mut h, 800, |h| h.app.world.plans[0].phase
        == PlanPhase::Done));
    assert_eq!(h.app.world.plans[0].steps[5].state, StepState::Failed);
    // a succeeded step: retry and skip are offered disabled with reasons
    let r3 = row(&h, plan::OUTLINE.child(3));
    right_click(&mut h, r3.x + 10, r3.y);
    assert_at_pointer(&h, plan::STEP_MENU, r3.x + 10, r3.y);
    let t = h.text();
    assert!(t.contains("Only a failed step can be retried"), "{t}");
    assert!(t.contains("The plan is not running"), "{t}");
    h.key(KeyCode::Esc);
    let r5 = row(&h, plan::OUTLINE.child(5));
    right_click(&mut h, r5.x + 10, r5.y);
    choose(&mut h, "Retry step");
    assert!(h.app.modals.is_empty());
    assert_eq!(h.app.world.plans[0].steps[5].state, StepState::Ready);
    assert_eq!(h.app.world.plans[0].steps[3].state, StepState::Succeeded);
}

// ------------------------------------------------------------------ disk

fn disk_tree() -> H {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 0, 120, 40);
    h.ticks(2);
    open(&mut h, "Analyze disk usage");
    h.ticks(40);
    assert!(h.text().contains("scan complete"), "{}", h.text());
    h
}

fn disk_selected(h: &mut H) -> Vec<String> {
    h.app.tabs[0]
        .stack
        .last_mut()
        .and_then(|s| s.as_disk())
        .map(|d| d.selected.iter().cloned().collect())
        .expect("the disk page")
}

#[test]
fn a_disk_tree_row_right_click_opens_its_menu_at_the_pointer_and_selects_it() {
    let mut h = disk_tree();
    let (x, y) = h.find("vm.img").expect("vm.img row");
    right_click(&mut h, x + 2, y);
    assert_eq!(h.app.focus.current(), Some(disk::ROW_MENU), "{}", h.text());
    assert_at_pointer(&h, disk::ROW_MENU, x + 2, y);
    let t = h.text();
    assert!(t.contains("Copy path") && t.contains("Rescan"), "{t}");
    assert!(t.contains("No scan is running"), "{t}");
    assert!(!disk_selected(&mut h).iter().any(|p| p.ends_with("vm.img")));
    choose(&mut h, "Select");
    assert!(h.app.modals.is_empty());
    assert!(
        disk_selected(&mut h).iter().any(|p| p.ends_with("vm.img")),
        "{:?}",
        disk_selected(&mut h)
    );
    assert_eq!(h.app.focus.current(), Some(disk::TREE));
}

#[test]
fn shift_f10_opens_the_disk_row_menu_below_the_cursor_row_and_reopens_outside() {
    let mut h = disk_tree();
    let (x, y) = h.find("vm.img").expect("vm.img row");
    h.click(x + 2, y);
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(disk::ROW_MENU), "{}", h.text());
    assert_below_row(&h, disk::ROW_MENU, y);
    h.key(KeyCode::Esc);
    // a right click on another row, outside the open menu, reopens there
    let (tx, ty) = h.find("target").expect("target row");
    right_click(&mut h, x + 2, y);
    let first = menu_area(&h, disk::ROW_MENU);
    assert!(ty < first.y || ty >= first.bottom(), "{}", h.text());
    right_click(&mut h, tx + 2, ty);
    assert_eq!(h.app.modals.len(), 1);
    assert_at_pointer(&h, disk::ROW_MENU, tx + 2, ty);
}

#[test]
fn a_disk_overview_row_menu_analyzes_the_row() {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 0, 120, 40);
    h.ticks(10);
    open(&mut h, "Disk overview");
    assert!(h.text().contains("Here › Disk › Overview"), "{}", h.text());
    let depth = h.app.tabs[0].stack.len();
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(disk::ROW_MENU), "{}", h.text());
    assert!(h.text().contains("Analyze"), "{}", h.text());
    choose(&mut h, "Analyze");
    assert!(h.app.modals.is_empty());
    assert!(
        h.app.tabs[0].stack.len() > depth,
        "Analyze opened the scan\n{}",
        h.text()
    );
}

// -------------------------------------------------------------- activity

fn activities() -> H {
    H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 120, 40)
}

fn activity_state(h: &H, name: &str) -> ActivityState {
    h.app
        .world
        .activities
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.state)
        .unwrap_or_else(|| panic!("no activity {name}"))
}

#[test]
fn an_activity_output_right_click_opens_its_menu_at_the_pointer_and_stops_it() {
    let mut h = activities();
    h.alt(KeyCode::Char('1'));
    let name = h.app.world.activities[0].name.clone();
    assert!(activity_state(&h, &name).live(), "{}", h.text());
    let v = row(&h, activity::VIEW);
    let (x, y) = (v.x + 4, v.y + 2);
    right_click(&mut h, x, y);
    assert_eq!(h.app.focus.current(), Some(activity::MENU), "{}", h.text());
    assert_at_pointer(&h, activity::MENU, x, y);
    let t = h.text();
    assert!(
        t.contains("Select text first"),
        "copy needs a selection\n{t}"
    );
    assert!(t.contains("Still running · s stops it first"), "{t}");
    choose(&mut h, "Stop");
    assert!(h.app.modals.is_empty());
    assert!(
        !matches!(activity_state(&h, &name), ActivityState::Running),
        "{:?}",
        activity_state(&h, &name)
    );
    assert_eq!(h.app.focus.current(), Some(activity::VIEW));
}

/// The activity tab's output viewport.
fn activity_view(h: &mut H) -> &junie_tui::widgets::viewport::TextViewport {
    let i = h.app.active;
    h.app.tabs[i]
        .stack
        .last_mut()
        .and_then(|s| s.as_activity())
        .map(|a| a.view())
        .expect("an activity tab")
}

#[test]
fn shift_f10_opens_the_activity_menu_at_the_followed_tail() {
    let mut h = activities();
    h.alt(KeyCode::Char('1'));
    let v = row(&h, activity::VIEW);
    assert_eq!(h.app.focus.current(), Some(activity::VIEW));
    let drawn = activity_view(&mut h).scroll.visible_range().len() as u16;
    assert!(drawn > 1, "output on screen\n{}", h.text());
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(activity::MENU), "{}", h.text());
    // no cursor row: the last drawn line of the followed output stands in
    let a = menu_area(&h, activity::MENU);
    assert!(
        a.y == v.y + drawn || a.bottom() < v.y + drawn,
        "below the tail line {} (or flipped above it): {a:?}\n{}",
        v.y + drawn - 1,
        h.text()
    );
    assert_ne!(a.y, v.y + 1, "not at the top line");
    choose(&mut h, "Find…");
    assert!(h.text().contains("Close find"), "{}", h.text());
}

#[test]
fn shift_f10_opens_the_activity_menu_at_the_selection() {
    use junie_tui::core::event::MouseKind;
    let mut h = activities();
    h.alt(KeyCode::Char('1'));
    let v = row(&h, activity::VIEW);
    // select from the first line into the second
    h.mouse(MouseKind::Down, v.x + 2, v.y);
    h.mouse(MouseKind::Drag, v.x + 6, v.y + 1);
    h.mouse(MouseKind::Up, v.x + 6, v.y + 1);
    assert!(activity_view(&mut h).has_selection(), "{}", h.text());
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(activity::MENU), "{}", h.text());
    assert_at_pointer(&h, activity::MENU, v.x + 6, v.y + 1);
    assert!(!h.text().contains("Select text first"), "{}", h.text());
}

#[test]
fn a_stream_chip_menu_hides_the_stream_and_reopens_from_the_output() {
    let mut h = activities();
    h.alt(KeyCode::Char('4'));
    assert!(h.text().contains("merged logs"), "{}", h.text());
    let a = h
        .app
        .world
        .activities
        .iter()
        .position(|a| a.services().len() > 1)
        .expect("a merged activity");
    let stream = h.app.world.activities[a].services()[0].clone();
    // open the output menu, then right-click the chip outside it
    let v = row(&h, activity::VIEW);
    right_click(&mut h, v.x + 30, v.y + 4);
    assert_eq!(h.app.focus.current(), Some(activity::MENU));
    let chip = row(&h, activity::CHIPS.child(0));
    let (cx, cy) = (chip.x, chip.y);
    right_click(&mut h, cx + 1, cy);
    assert_eq!(h.app.modals.len(), 1, "{}", h.text());
    assert_at_pointer(&h, activity::MENU, cx + 1, cy);
    assert!(h.text().contains("Hide stream"), "{}", h.text());
    choose(&mut h, "Hide stream");
    assert!(
        h.app.world.activities[a].hidden_services.contains(&stream),
        "{:?}",
        h.app.world.activities[a].hidden_services
    );
    assert_eq!(h.app.focus.current(), Some(activity::CHIPS));
}

// --------------------------------------------------------------- cleanup

fn cleanup_page() -> H {
    let mut h = H::new(Scenario::ParityInsights, Motion::Paused, 40, 120, 40);
    h.app.go(Go::Push(Page::Cleanup { category: None }));
    h.draw();
    h
}

#[test]
fn a_cleanup_row_right_click_opens_its_menu_at_the_pointer_and_unselects() {
    let mut h = cleanup_page();
    assert!(h.text().contains("19 selected"), "{}", h.text());
    // a selected category below the cursor row
    let (x, y) = h.find("Xcode device support").expect("the category row");
    right_click(&mut h, x + 2, y);
    assert_eq!(
        h.app.focus.current(),
        Some(cleanup::ROW_MENU),
        "{}",
        h.text()
    );
    assert_at_pointer(&h, cleanup::ROW_MENU, x + 2, y);
    let t = h.text();
    assert!(t.contains("Review deletion…"), "{t}");
    assert!(
        !t.contains("Select at least one eligible candidate first"),
        "items are selected\n{t}"
    );
    choose(&mut h, "Unselect category");
    assert!(h.app.modals.is_empty());
    assert!(!h.text().contains("19 selected"), "{}", h.text());
    assert_eq!(h.app.focus.current(), Some(cleanup::LIST));
}

#[test]
fn shift_f10_opens_the_cleanup_menu_below_the_cursor_row() {
    let mut h = cleanup_page();
    assert_eq!(h.app.focus.current(), Some(cleanup::LIST));
    shift_f10(&mut h);
    assert_eq!(
        h.app.focus.current(),
        Some(cleanup::ROW_MENU),
        "{}",
        h.text()
    );
    choose(&mut h, "Show details");
    assert_eq!(h.app.focus.current(), Some(cleanup::DETAIL));
}

// ------------------------------------------------------------ snapshot

#[test]
fn a_snapshot_body_menu_runs_the_page_keys() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    h.app.go(Go::Push(Page::Snapshot {
        kind: "port".into(),
    }));
    h.draw();
    let body = row(&h, snapshot::BODY);
    let (x, y) = (body.x + 3, body.y + 1);
    right_click(&mut h, x, y);
    assert_eq!(h.app.focus.current(), Some(snapshot::MENU), "{}", h.text());
    assert_at_pointer(&h, snapshot::MENU, x, y);
    choose(&mut h, "Refresh");
    assert!(h.text().contains("Refreshed · live"), "{}", h.text());
    // the keyboard opens it at the body's top line
    h.app.focus.focus(snapshot::BODY);
    h.draw();
    shift_f10(&mut h);
    assert_below_row(&h, snapshot::MENU, body.y);
}

#[test]
fn a_config_snapshot_menu_offers_revoking_trust() {
    let mut h = H::new(Scenario::ParityCustomActions, Motion::Paused, 40, 120, 40);
    h.app.go(Go::Push(Page::Config {
        path: "/Users/alex/work/team/.holla.toml".into(),
    }));
    h.draw();
    let body = row(&h, snapshot::BODY);
    right_click(&mut h, body.x + 3, body.y + 1);
    assert!(h.text().contains("Re-read configuration"), "{}", h.text());
    choose(&mut h, "Revoke trust");
    let t = h.text();
    assert!(
        t.contains("Trust revoked") || t.contains("Nothing to revoke"),
        "{t}"
    );
}

// ---------------------------------------------------------------- gate 1

#[test]
fn the_gate_sequence_menu_copies_or_cancels_but_never_continues() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Reduced, 0, 120, 40);
    let plan = h.app.world.plans.first().expect("plan fixture").id.clone();
    h.app.go(Go::Push(Page::Review(GateTarget::Plan(plan))));
    h.draw();
    assert!(h.text().contains("gate 1 of 2"));
    let seq = row(&h, review::SEQUENCE);
    h.app.focus.focus(review::SEQUENCE);
    h.draw();
    shift_f10(&mut h);
    assert_eq!(
        h.app.focus.current(),
        Some(review::SEQUENCE_MENU),
        "{}",
        h.text()
    );
    assert_below_row(&h, review::SEQUENCE_MENU, seq.y);
    let labels: Vec<String> = match h.app.modals.top().map(|m| &m.modal) {
        Some(crate::screens::Modal::Menu(m)) => m.items.iter().map(|i| i.label.clone()).collect(),
        _ => panic!("no menu"),
    };
    assert_eq!(labels, ["Copy sequence", "Cancel"], "never Continue");
    // the second item: Cancel
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.app.modals.is_empty());
    assert!(!h.text().contains("gate 1 of 2"), "{}", h.text());
}
