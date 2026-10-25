#![cfg(test)]

use super::*;
use crate::{
    scenario::{Motion, Scenario},
    screens::{activity, disk_scan},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use termrock::{Chord, Dialog, Id, ItemKey, KeyCode, KeyPhase, MouseKind, Part, PartRef, Theme};
use termrock_test_support::Harness;

struct QueuedExhaustionApp {
    inner: App,
    queued: QueuedDiskScanInputs,
}

#[derive(Clone, Default)]
struct QueuedDiskScanInputs {
    exhaust_projection: Rc<Cell<bool>>,
    world_seek_ms: Rc<Cell<Option<i64>>>,
    effect_revision: Rc<Cell<Option<u64>>>,
    remove_activity: Rc<Cell<Option<u32>>>,
    replace_scan: Rc<Cell<Option<bool>>>,
    replacement_snapshot: Rc<RefCell<Option<ScanSnapshot>>>,
    retire_selection: Rc<Cell<bool>>,
    transitions: Rc<Cell<u8>>,
    succeeded: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct ScanSnapshot {
    generation: Option<u64>,
    status: Option<crate::sim::ScanStatus>,
    paths: Vec<String>,
    revealed_count: usize,
    selected: Option<String>,
    context: Option<String>,
}

impl termrock::App for QueuedExhaustionApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> termrock::Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Event {
            if let Some(now_ms) = self.queued.world_seek_ms.take() {
                self.inner
                    .world
                    .seek(u64::try_from(now_ms).unwrap_or_default());
                self.inner
                    .disk_scan
                    .refresh_at_ms_for_test(now_ms)
                    .expect("queued modeled scan refreshes");
            }
            if let Some(revision) = self.queued.effect_revision.take() {
                self.inner.world.effect_revision = revision;
            }
            if let Some(removed_id) = self.queued.remove_activity.take() {
                self.inner
                    .world
                    .activities
                    .retain(|activity| activity.id != removed_id);
                self.inner.world.effect_revision =
                    self.inner.world.effect_revision.saturating_add(1);
            }
            if let Some(leave_first) = self.queued.replace_scan.take() {
                if leave_first {
                    self.inner.disk_scan.leave(&self.inner.world);
                    self.inner.route = Route::Home;
                    self.inner.disk_scan_return = None;
                }
                self.inner
                    .disk_scan
                    .open(&self.inner.world)
                    .expect("queued replacement scan opens");
                self.inner.route = Route::DiskScan;
                self.queued.replacement_snapshot.replace(Some(ScanSnapshot {
                    generation: self.inner.disk_scan.generation(),
                    status: self.inner.disk_scan.status_for_test(),
                    paths: self.inner.disk_scan.projected_paths(),
                    revealed_count: self.inner.disk_scan.revealed_count_for_test(),
                    selected: self.inner.disk_scan.selected_path().map(str::to_owned),
                    context: self.inner.disk_scan.context_for_test(),
                }));
            }
            if self.queued.retire_selection.replace(false) {
                assert!(matches!(
                    self.inner.overlay,
                    Some(Overlay::DiskSelection(_))
                ));
                self.inner.retired_overlay = self.inner.overlay.take();
                cx.close_layer(disk::SELECTION_DIALOG, None);
            }
            if self.queued.exhaust_projection.replace(false) {
                let result = self.inner.disk_scan.exhaust_projection_for_test();
                self.queued
                    .transitions
                    .set(self.queued.transitions.get().saturating_add(1));
                self.queued
                    .succeeded
                    .set(result == Err("presentation identity exhausted".to_owned()));
            }
        }
        termrock::App::update(&mut self.inner, cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        termrock::App::draw(&self.inner, ui);
    }

    fn should_quit(&self) -> bool {
        termrock::App::should_quit(&self.inner)
    }

    fn keymap(&self) -> &termrock::KeyMap {
        termrock::App::keymap(&self.inner)
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> termrock::Response<()> {
        termrock::App::on_esc(&mut self.inner, cx)
    }
}

fn harness(width: u16, height: u16) -> Harness<App> {
    let app = App::for_scenario_without_history(Scenario::ParityHistory, Motion::Paused, 4_000);
    harness_for_app(app, width, height)
}

fn harness_for_app(app: App, width: u16, height: u16) -> Harness<App> {
    let initial = u64::try_from(app.fixture_time_ms()).unwrap_or_default();
    Harness::new_with_feedback_clock(
        app,
        Theme::junie(),
        width,
        height,
        termrock::FeedbackClock::Simulation {
            initial: termrock::SimulationMoment::from_millis(initial),
        },
    )
}

fn queued_harness(
    app: App,
    width: u16,
    height: u16,
    queued: QueuedDiskScanInputs,
) -> Harness<QueuedExhaustionApp> {
    let initial = u64::try_from(app.fixture_time_ms()).unwrap_or_default();
    Harness::new_with_feedback_clock(
        QueuedExhaustionApp { inner: app, queued },
        Theme::junie(),
        width,
        height,
        termrock::FeedbackClock::Simulation {
            initial: termrock::SimulationMoment::from_millis(initial),
        },
    )
}

fn open_presented_scan_with_rows(
    harness: &mut Harness<QueuedExhaustionApp>,
    now_ms: i64,
) -> (u64, Vec<String>) {
    let generation = {
        let app = &mut harness.app_mut().inner;
        app.disk_scan.open(&app.world)
    }
    .expect("modeled scan opens");
    harness.app_mut().inner.route = Route::DiskScan;
    harness
        .app_mut()
        .inner
        .world
        .seek(u64::try_from(now_ms).unwrap_or_default());
    harness
        .app_mut()
        .inner
        .disk_scan
        .refresh_at_ms_for_test(now_ms)
        .expect("modeled paths reveal");
    assert_eq!(
        harness.app().inner.disk_scan.status_for_test(),
        Some(crate::sim::ScanStatus::Scanning),
        "the old Cancel control must still be enabled"
    );
    let paths = harness.app().inner.disk_scan.projected_paths();
    assert!(paths.len() > 1, "fixture reveals a selectable tree row");
    harness.draw();
    (generation, paths)
}

fn context_snapshot(harness: &Harness<QueuedExhaustionApp>) -> Option<String> {
    harness.app().inner.disk_scan.context_for_test()
}

trait BrowserRouteState {
    fn browser_route(&self) -> Route;
}

impl BrowserRouteState for App {
    fn browser_route(&self) -> Route {
        self.route
    }
}

impl BrowserRouteState for QueuedExhaustionApp {
    fn browser_route(&self) -> Route {
        self.inner.route
    }
}

fn open_scan_from_go_menu<A: termrock::App + BrowserRouteState>(harness: &mut Harness<A>) {
    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().browser_route() == Route::DiskScan);
}

fn assert_inside(area: termrock::Rect, control: termrock::Rect) {
    assert!(control.x >= area.x && control.y >= area.y);
    assert!(control.right() <= area.right() && control.bottom() <= area.bottom());
}

fn overlaps(left: termrock::Rect, right: termrock::Rect) -> bool {
    left.x < right.right()
        && right.x < left.right()
        && left.y < right.bottom()
        && right.y < left.bottom()
}

#[test]
fn go_menu_keeps_home_first_opens_scan_and_escape_restores_origin_focus() {
    let mut harness = harness(100, 30);
    let origin_focus = harness.focus();
    let original_cwd = harness.app().world.cwd.clone();
    let original_revision = harness.app().world.effect_revision;
    let original_activities = harness.app().world.activities.clone();

    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let menu_text = harness.text();
    let home = menu_text.find("Home").expect("Go menu shows Home");
    let scan = menu_text
        .find("Modeled disk scan")
        .expect("Go menu shows modeled scan");
    assert!(home < scan, "Home must remain first in Go menu");

    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::DiskScan);
    assert_eq!(
        harness.focus(),
        Some(harness.app().disk_scan.owner_id_for_test(disk_scan::TREE))
    );
    assert!(harness.text().contains("Modeled snapshot"));
    assert!(harness.text().contains("Root: /home/dev/work/holla"));
    assert_eq!(harness.app().world.cwd, original_cwd);
    assert_eq!(harness.app().world.effect_revision, original_revision);
    assert_eq!(harness.app().world.activities, original_activities);

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.focus(), origin_focus);
    assert_eq!(harness.app().world.cwd, original_cwd);
    assert_eq!(harness.app().world.effect_revision, original_revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn go_menu_disk_scan_restores_nondefault_home_rows_focus() {
    let mut harness = harness(100, 30);
    harness.draw();
    assert!(harness.tab_to(crate::screens::home::ROWS));
    let origin_focus = harness.focus();
    assert_eq!(origin_focus, Some(crate::screens::home::ROWS));

    open_scan_from_go_menu(&mut harness);
    assert_eq!(
        harness.focus(),
        Some(harness.app().disk_scan.owner_id_for_test(disk_scan::TREE))
    );
    let _ = harness.key(KeyCode::Esc);

    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.focus(), origin_focus);
    assert!(harness.area_of(crate::screens::home::ROWS).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn go_menu_disk_scan_restores_root_menu_focus() {
    let mut harness = harness(100, 30);
    harness.draw();
    assert!(harness.tab_to(MENU));
    let origin_focus = harness.focus();
    assert_eq!(origin_focus, Some(MENU));

    open_scan_from_go_menu(&mut harness);
    let _ = harness.key(KeyCode::Esc);

    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.focus(), origin_focus);
    assert!(harness.area_of(MENU).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn go_menu_disk_scan_restores_current_activity_output_focus() {
    let mut app =
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
    let activity_id = app
        .world
        .activities
        .first()
        .expect("activity fixture has a current Activity")
        .id;
    app.activities.show(activity_id);
    app.route = Route::Activity;
    let mut harness = harness_for_app(app, 100, 30);
    let activity_owner = activity::output_id(activity_id);
    harness.draw();
    assert!(harness.tab_to(activity_owner));
    let origin_focus = harness.focus();
    assert_eq!(origin_focus, Some(activity_owner));

    open_scan_from_go_menu(&mut harness);
    assert_eq!(
        harness.focus(),
        Some(harness.app().disk_scan.owner_id_for_test(disk_scan::TREE))
    );
    let _ = harness.key(KeyCode::Esc);

    assert!(harness.app().route == Route::Activity);
    assert_eq!(harness.focus(), origin_focus);
    assert!(harness.area_of(activity_owner).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn failed_root_mapping_keeps_route_focus_and_generation_unchanged() {
    let mut harness = harness(100, 30);
    harness.app_mut().world.cwd = "relative/path".to_owned();
    let origin_focus = harness.focus();

    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);

    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.app().disk_scan.generation(), None);
    assert_eq!(harness.focus(), origin_focus);
    assert!(harness.text().contains("Modeled disk scan unavailable"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn stale_area_witness_rejects_browser_input_without_mutating_the_session() {
    let mut harness = harness(100, 30).with_auto_draw(false);
    let generation = {
        let app = harness.app_mut();
        app.disk_scan.open(&app.world)
    }
    .expect("modeled scan opens");
    let selected = harness.app().disk_scan.selected_path().map(str::to_owned);
    harness.app_mut().route = Route::DiskScan;

    // The last presented frame is still Home, so it has no current browser owner areas.
    let _ = harness.key(KeyCode::Down);

    assert!(harness.app().route == Route::DiskScan);
    assert_eq!(harness.app().disk_scan.generation(), Some(generation));
    assert_eq!(harness.app().disk_scan.selected_path(), selected.as_deref());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn tree_keyboard_pointer_sort_and_last_row_remain_path_keyed() {
    let mut harness = harness(100, 30);
    open_scan_from_go_menu(&mut harness);
    let generation = harness
        .app()
        .disk_scan
        .generation()
        .expect("scan has a generation");
    assert!(termrock::App::accept_paused_clock_step(
        harness.app_mut(),
        5_000
    ));
    // The clock step changes the model projection; publish that projection before
    // sending Tree input so the current dynamic owner has a matching witness.
    let _ = harness.tick();
    harness.draw();

    let _ = harness.key(KeyCode::Right);
    assert!(harness.text().contains("archive"));
    let archive = "/home/dev/work/holla/archive";
    let archive_row = PartRef::item(Part::ROW, ItemKey::text(archive));
    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    assert!(harness.area_of_part(tree, archive_row).is_some());
    let _ = harness.click_part(tree, archive_row);
    assert_eq!(harness.app().disk_scan.selected_path(), Some(archive));
    let details = harness
        .area_of(
            harness
                .app()
                .disk_scan
                .owner_id_for_test(disk_scan::DETAILS),
        )
        .expect("details pane has a current area");
    let mut detail_text = String::new();
    for y in details.y..details.bottom() {
        for x in details.x..details.right() {
            detail_text.push_str(
                harness
                    .buffer()
                    .cell((x, y))
                    .expect("details cell is inside the frame")
                    .symbol(),
            );
        }
    }
    let compact_details: String = detail_text
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '┃')
        .collect();
    let compact_archive: String = archive
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(
        compact_details.contains(&format!("Inspectedpath:{compact_archive}")),
        "{detail_text}"
    );
    let child_path = "/home/dev/work/holla/archive/2025.txt";
    assert!(harness.text().contains("2025.txt"));
    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    assert!(
        harness
            .area_of_part(tree, PartRef::item(Part::ROW, ItemKey::text(child_path)))
            .is_some()
    );

    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    let archive_disclosure = PartRef::item(Part::ICON, ItemKey::text(archive));
    assert!(harness.area_of_part(tree, archive_disclosure).is_some());
    let _ = harness.click_part(tree, archive_disclosure);
    assert_eq!(harness.app().disk_scan.selected_path(), Some(archive));
    assert!(!harness.text().contains("2025.txt"));

    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    assert!(
        harness
            .area_of_part(tree, PartRef::item(Part::ROW, ItemKey::text(child_path)))
            .is_none()
    );
    assert!(harness.area_of_part(tree, archive_disclosure).is_some());
    let _ = harness.click_part(tree, archive_disclosure);
    assert_eq!(harness.app().disk_scan.selected_path(), Some(archive));
    assert!(harness.text().contains("2025.txt"));

    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    assert!(
        harness
            .area_of_part(tree, PartRef::item(Part::ROW, ItemKey::text(child_path)))
            .is_some()
    );
    assert_eq!(harness.focus(), Some(tree));
    let _ = harness.key(KeyCode::End);
    let tree = harness.app().disk_scan.owner_id_for_test(disk_scan::TREE);
    let last_path = "/home/dev/work/holla/résumé.txt";
    assert_eq!(harness.app().disk_scan.selected_path(), Some(last_path));
    assert!(
        harness
            .area_of_part(tree, PartRef::item(Part::ROW, ItemKey::text(last_path)))
            .is_some()
    );

    let selected = harness.app().disk_scan.selected_path().map(str::to_owned);
    let sort = harness.app().disk_scan.owner_id_for_test(disk_scan::SORT);
    let _ = harness.click_id(sort);
    assert_eq!(
        harness.app().disk_scan.selected_path(),
        selected.as_deref(),
        "sorting keeps cursor identity by path"
    );
    assert!(harness.text().contains("Sort: Apparent descending"));
    assert_eq!(harness.app().disk_scan.generation(), Some(generation));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn route_bounds_survive_minimum_and_standard_resize() {
    let mut harness = harness(100, 30);
    open_scan_from_go_menu(&mut harness);

    for (width, height) in [(72, 20), (120, 40)] {
        let _ = harness.resize(width, height);
        harness.draw();
        let frame = *harness.buffer().area();
        let body = shell_areas(frame).0;
        let state = &harness.app().disk_scan;
        let controls = [
            disk_scan::RESCAN,
            disk_scan::CANCEL,
            disk_scan::SORT,
            disk_scan::BACK,
            disk_scan::TREE,
            disk_scan::DETAILS,
        ]
        .map(|base| state.owner_id_for_test(base));
        let rects: Vec<_> = controls
            .into_iter()
            .map(|id| {
                harness
                    .area_of(id)
                    .unwrap_or_else(|| panic!("missing owner {id:?}"))
            })
            .collect();
        for rect in &rects {
            assert_inside(body, *rect);
        }
        for (index, left) in rects.iter().enumerate() {
            for right in rects.iter().skip(index.saturating_add(1)) {
                assert!(
                    !overlaps(*left, *right),
                    "owners overlap: {left:?}, {right:?}"
                );
            }
        }
        assert!(harness.app().route == Route::DiskScan);
    }
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn repeated_open_and_covered_dialog_do_not_replace_scan_or_return_location() {
    let mut harness = harness(100, 30);
    open_scan_from_go_menu(&mut harness);
    let generation = harness.app().disk_scan.generation();
    let origin = harness
        .app()
        .disk_scan_return
        .expect("opening the scan records its return location");

    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::DiskScan);
    assert_eq!(harness.app().disk_scan.generation(), generation);
    let retained = harness
        .app()
        .disk_scan_return
        .expect("reopening the active route keeps its return location");
    assert!(retained.route == origin.route);
    assert_eq!(retained.focus, origin.focus);

    let _ = harness.key(KeyCode::F(1));
    assert!(harness.app().overlay.is_some());
    let covered_selection = harness.app().disk_scan.selected_path().map(str::to_owned);
    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.app().disk_scan.generation(), generation);
    assert_eq!(
        harness.app().disk_scan.selected_path(),
        covered_selection.as_deref()
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().overlay.is_none());
    assert!(harness.app().route == Route::DiskScan);
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Home);
    assert_eq!(
        harness.app().disk_scan.status_for_test(),
        Some(crate::sim::ScanStatus::Cancelled)
    );
    let selected_after_leave = harness.app().disk_scan.selected_path().map(str::to_owned);
    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.app().disk_scan.generation(), generation);
    assert_eq!(
        harness.app().disk_scan.selected_path(),
        selected_after_leave.as_deref()
    );
    open_scan_from_go_menu(&mut harness);
    assert!(
        harness
            .app()
            .disk_scan
            .generation()
            .is_some_and(|next| next > generation.expect("first scan generation"))
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

fn assert_stale_control_is_ignored_after_replacement(control: Id, leave_first: bool) {
    let queued = QueuedDiskScanInputs::default();
    let app = App::for_scenario_without_history(Scenario::ParityHistory, Motion::Paused, 4_000);
    let mut harness = queued_harness(app, 100, 30, queued.clone()).with_auto_draw(false);
    let (first_generation, paths) = open_presented_scan_with_rows(&mut harness, 4_160);
    assert!(paths.len() > 1);
    let old_owner = harness.app().inner.disk_scan.owner_id_for_test(control);
    assert!(harness.area_of(old_owner).is_some());
    assert!(
        harness.tab_to(old_owner),
        "old control is a real focused owner"
    );
    harness.draw();

    // The replacement is queued so the old native input is routed against the
    // published owner before the event update retires that presentation.
    queued.replace_scan.set(Some(leave_first));

    if control == disk_scan::TREE {
        let _ = harness.key(KeyCode::End);
    } else {
        let _ = harness.key(KeyCode::Enter);
    }

    assert!(harness.app().inner.route == Route::DiskScan);
    let replacement = queued
        .replacement_snapshot
        .borrow()
        .clone()
        .expect("Event replacement snapshot is captured before App::update");
    assert!(
        replacement
            .generation
            .is_some_and(|current| current > first_generation)
    );
    assert_eq!(
        harness.app().inner.disk_scan.generation(),
        replacement.generation
    );
    assert_eq!(
        harness.app().inner.disk_scan.status_for_test(),
        replacement.status
    );
    assert_eq!(
        harness.app().inner.disk_scan.projected_paths(),
        replacement.paths
    );
    assert_eq!(
        harness.app().inner.disk_scan.revealed_count_for_test(),
        replacement.revealed_count
    );
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        replacement.selected.as_deref()
    );
    assert_eq!(context_snapshot(&harness), replacement.context);
    assert!(
        harness.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            termrock::Diagnostic::UndeliveredIntent { owner } if *owner == old_owner
        )),
        "retired presentation input is explicitly settled: {:?}",
        harness.diagnostics()
    );

    // A newly drawn owner can service input for the replacement generation.
    harness.draw();
    let fresh_rescan = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::RESCAN);
    assert!(harness.area_of(fresh_rescan).is_some());
    let _ = harness.click_id(fresh_rescan);
    assert!(
        harness
            .app()
            .inner
            .disk_scan
            .generation()
            .is_some_and(|next| next > replacement.generation.expect("replacement generation"))
    );
}

#[test]
fn old_cancel_rescan_and_tree_inputs_cannot_cross_generation_or_reopened_session() {
    for leave_first in [false, true] {
        for control in [disk_scan::CANCEL, disk_scan::RESCAN, disk_scan::TREE] {
            assert_stale_control_is_ignored_after_replacement(control, leave_first);
        }
    }
}

#[test]
fn stale_tree_projection_input_waits_for_a_fresh_presented_frame() {
    let queued = QueuedDiskScanInputs::default();
    let app = App::for_scenario_without_history(Scenario::ParityHistory, Motion::Paused, 4_000);
    let mut harness = queued_harness(app, 100, 30, queued.clone()).with_auto_draw(false);
    let generation = {
        let app = &mut harness.app_mut().inner;
        app.disk_scan.open(&app.world)
    }
    .expect("modeled scan opens");
    harness.app_mut().inner.route = Route::DiskScan;
    harness.app_mut().inner.world.seek(4_160);
    harness
        .app_mut()
        .inner
        .disk_scan
        .refresh_at_ms_for_test(4_160)
        .expect("first modeled reveal is available");
    harness.draw();

    let old_tree = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::TREE);
    assert!(harness.tab_to(old_tree), "Tree is a real focused owner");
    harness.draw();
    let _ = harness.key(KeyCode::Right);
    harness.draw();
    let old_tree = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::TREE);
    assert_eq!(harness.focus(), Some(old_tree));
    harness.draw();
    let old_paths = harness.app().inner.disk_scan.projected_paths();
    let target_path = old_paths[1].clone();
    let row = PartRef::item(Part::ROW, ItemKey::text(&target_path));
    assert!(harness.area_of_part(old_tree, row).is_some());

    let generation_before = harness.app().inner.disk_scan.generation();
    let status_before = harness.app().inner.disk_scan.status_for_test();
    let revealed_before = harness.app().inner.disk_scan.revealed_count_for_test();
    let selected_before = harness
        .app()
        .inner
        .disk_scan
        .selected_path()
        .map(str::to_owned);
    let context_before = harness.app().inner.disk_scan.context_for_test();
    queued.world_seek_ms.set(Some(4_240));

    let _ = harness.key(KeyCode::End);
    assert!(harness.app().inner.disk_scan.revealed_count_for_test() > revealed_before);
    assert_ne!(
        harness
            .app()
            .inner
            .disk_scan
            .owner_id_for_test(disk_scan::TREE),
        old_tree,
        "the queued refresh creates a distinct row projection"
    );
    assert_eq!(
        harness.app().inner.disk_scan.generation(),
        generation_before
    );
    assert_eq!(
        harness.app().inner.disk_scan.status_for_test(),
        status_before
    );
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        selected_before.as_deref()
    );
    assert_eq!(
        harness.app().inner.disk_scan.context_for_test(),
        context_before
    );
    assert!(
        harness.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            termrock::Diagnostic::UndeliveredIntent { owner } if *owner == old_tree
        )),
        "the stale Tree binding is settled before a new frame: {:?}",
        harness.diagnostics()
    );

    harness.draw();
    let current_tree = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::TREE);
    assert!(harness.area_of_part(current_tree, row).is_some());
    let _ = harness.click_part(current_tree, row);
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        Some(target_path.as_str()),
        "a fresh presentation authorizes the current tree projection"
    );
    assert_eq!(harness.app().inner.disk_scan.generation(), Some(generation));
}

#[test]
fn back_button_response_is_folded_while_returning_to_the_prior_route() {
    let mut harness = harness(100, 30);
    open_scan_from_go_menu(&mut harness);
    let back = harness.app().disk_scan.owner_id_for_test(disk_scan::BACK);
    let area = harness
        .area_of(back)
        .expect("Back has a published button area");
    let x = area.x.saturating_add(area.width / 2);
    let y = area.y.saturating_add(area.height / 2);
    let _ = harness.mouse(MouseKind::Down, x, y);
    let response = harness.mouse(MouseKind::Up, x, y);
    assert!(harness.app().route == Route::Home);
    assert!(
        response.is_changed(),
        "the Back activation response is folded"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn retired_overlay_blocks_disk_scan_command_for_the_transition_update() {
    let mut app = App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 4_000);
    app.keymap = app
        .keymap
        .bind(KeyPhase::Capture, Chord::key(KeyCode::F(9)), DISK_SCAN);
    let queued = QueuedDiskScanInputs::default();
    let mut harness = queued_harness(app, 100, 30, queued.clone());
    assert!(harness.app().inner.route == Route::Disk);

    let _ = harness.key(KeyCode::Char(' '));
    assert!(matches!(
        harness.app().inner.overlay,
        Some(Overlay::DiskSelection(_))
    ));
    assert_eq!(harness.app().inner.disk_scan.generation(), None);
    let return_location = harness.app().inner.disk_scan_return;

    queued.retire_selection.set(true);
    let blocked_response = harness.key(KeyCode::F(9));
    assert!(
        blocked_response.is_consumed(),
        "retired-overlay command is rejected"
    );
    assert!(harness.app().inner.route == Route::Disk);
    assert_eq!(harness.app().inner.disk_scan.generation(), None);
    assert!(
        harness
            .app()
            .inner
            .disk_scan_return
            .map(|location| (location.route, location.focus))
            == return_location.map(|location| (location.route, location.focus))
    );
    for _ in 0..4 {
        if harness.app().inner.retired_overlay.is_none() {
            break;
        }
        let _ = harness.tick();
    }
    assert!(harness.app().inner.retired_overlay.is_none());

    let _ = harness.key(KeyCode::F(9));
    assert!(harness.app().inner.route == Route::DiskScan);
    assert!(harness.app().inner.disk_scan.generation().is_some());
}

#[test]
fn returning_from_files_preview_uses_the_live_files_list_focus() {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let snapshot = crate::sim::fixtures::files_for_cwd(&cwd);
    let mut harness = harness_for_app(app.with_files_snapshot(snapshot), 100, 30);

    let _ = harness.key(KeyCode::F(4));
    let browse = Dialog::new(dialogs::DIALOG).action_id(1);
    assert!(harness.tab_to(browse));
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::Files);

    let readme = crate::sim::FilesSnapshot::join(&cwd, "README.md");
    let readme_row = PartRef::item(Part::ROW, ItemKey::text(&readme));
    let _ = harness.click_part(crate::screens::files::LIST, readme_row);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));
    assert_eq!(harness.focus(), Some(crate::screens::files::PREVIEW));

    open_scan_from_go_menu(&mut harness);
    assert_eq!(harness.app().files.preview_path(), None);
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Files);
    assert_eq!(harness.app().files.preview_path(), None);
    assert_eq!(harness.focus(), Some(crate::screens::files::LIST));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn removed_activity_return_uses_home_query_and_preserves_remaining_activities() {
    let mut app =
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
    let removed_id = app
        .world
        .activities
        .first()
        .expect("activity fixture has a current Activity")
        .id;
    app.activities.show(removed_id);
    app.route = Route::Activity;
    let queued = QueuedDiskScanInputs::default();
    let mut harness = queued_harness(app, 100, 30, queued.clone());
    let activity_owner = activity::output_id(removed_id);
    harness.draw();
    assert!(harness.tab_to(activity_owner));
    assert_eq!(harness.focus(), Some(activity_owner));

    let remaining_activities: Vec<_> = harness
        .app()
        .inner
        .world
        .activities
        .iter()
        .filter(|activity| activity.id != removed_id)
        .cloned()
        .collect();
    open_scan_from_go_menu(&mut harness);
    assert!(harness.app().inner.route == Route::DiskScan);

    let changed_revision = harness.app().inner.world.effect_revision.saturating_add(1);
    let generation = harness.app().inner.disk_scan.generation();
    let selected = harness
        .app()
        .inner
        .disk_scan
        .selected_path()
        .map(str::to_owned);
    queued.remove_activity.set(Some(removed_id));

    let _ = harness.key(KeyCode::Esc);

    assert!(harness.app().inner.route == Route::DiskScan);
    assert!(harness.text().contains("Stale"));
    assert_eq!(harness.top_layer(), termrock::LayerId::PAGE);
    assert!(harness.app().inner.overlay.is_none());
    assert_eq!(harness.app().inner.disk_scan.generation(), generation);
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        selected.as_deref()
    );
    assert_eq!(harness.app().inner.world.effect_revision, changed_revision);
    assert_eq!(
        harness.app().inner.world.activities,
        remaining_activities,
        "the first Escape refreshes the stale Activity context without leaving the scan"
    );
    let back_owner = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::BACK);
    assert_eq!(harness.focus(), Some(back_owner));
    assert!(harness.area_of(back_owner).is_some());

    // The first Escape changed context and focus, so Runtime correctly suppressed
    // the app-level fallback. A second key now targets the newly published Back.
    let _ = harness.key(KeyCode::Esc);

    let route_home = matches!(harness.app().inner.route, Route::Home);
    let current_activities = harness
        .app()
        .inner
        .world
        .activities
        .iter()
        .map(|activity| activity.id)
        .collect::<Vec<_>>();
    let saved_return = harness
        .app()
        .inner
        .disk_scan_return
        .map(|location| (matches!(location.route, Route::Activity), location.focus));
    assert!(
        route_home,
        "route_home={route_home}, focus={:?}, top_layer={:?}, overlay={}, retired_overlay={}, saved_return={saved_return:?}, activities={current_activities:?}, current_id={:?}",
        harness.focus(),
        harness.top_layer(),
        harness.app().inner.overlay.is_some(),
        harness.app().inner.retired_overlay.is_some(),
        harness.app().inner.activities.current_id(),
    );
    assert_eq!(harness.focus(), Some(crate::screens::home::QUERY));
    assert!(harness.area_of(crate::screens::home::QUERY).is_some());
    assert!(harness.area_of(activity_owner).is_none());
    assert_eq!(
        harness.app().inner.world.activities,
        remaining_activities,
        "returning from the stale Activity does not alter other Activity state"
    );
    assert_eq!(harness.app().inner.world.effect_revision, changed_revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn context_drift_keeps_back_available_and_makes_scan_input_inert() {
    let queued = QueuedDiskScanInputs::default();
    let app = App::for_scenario_without_history(Scenario::ParityHistory, Motion::Paused, 4_000);
    let mut harness = queued_harness(app, 100, 30, queued.clone());
    harness.draw();
    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().inner.route == Route::DiskScan);
    let generation = harness.app().inner.disk_scan.generation();
    let selected = harness
        .app()
        .inner
        .disk_scan
        .selected_path()
        .map(str::to_owned);
    let presented_tree = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::TREE);
    let changed_revision = harness.app().inner.world.effect_revision.saturating_add(1);
    queued.effect_revision.set(Some(changed_revision));
    let _ = harness.key(KeyCode::Down);
    assert!(harness.text().contains("Stale"));
    assert_eq!(harness.app().inner.disk_scan.generation(), generation);
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        selected.as_deref()
    );
    assert!(harness.app().inner.route == Route::DiskScan);
    assert_eq!(harness.diagnostics().len(), 1);
    assert!(matches!(
        &harness.diagnostics()[0],
        termrock::Diagnostic::UndeliveredIntent { owner } if *owner == presented_tree
    ));
    let back = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::BACK);
    let _ = harness.click_id(back);
    assert!(harness.app().inner.route == Route::Home);
    assert!(
        harness.diagnostics().is_empty(),
        "Back adds no diagnostic: {:?}",
        harness.diagnostics()
    );
}

#[test]
fn exhausted_projection_rejects_old_tree_input_and_keeps_minimum_size_back_live() {
    let app = App::for_scenario_without_history(Scenario::ParityHistory, Motion::Paused, 4_000);
    let queued = QueuedDiskScanInputs::default();
    let mut harness = queued_harness(app, 72, 20, queued.clone());
    let origin_focus = harness.focus();
    let _ = harness.key(KeyCode::F(10));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().inner.route == Route::DiskScan);
    harness.app_mut().inner.world.seek(9_000);
    harness
        .app_mut()
        .inner
        .disk_scan
        .refresh_at_ms_for_test(9_000)
        .expect("modeled paths reveal");
    harness.draw();

    harness
        .app_mut()
        .inner
        .disk_scan
        .set_projection_for_test(u64::MAX - 1);
    harness.draw();
    let old_tree = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::TREE);
    assert!(
        harness.area_of(old_tree).is_some(),
        "last normal Tree is presented"
    );
    assert!(
        harness.tab_to(old_tree),
        "last normal Tree can receive focus"
    );
    harness.draw();
    assert_eq!(harness.focus(), Some(old_tree));

    let generation = harness.app().inner.disk_scan.generation();
    let paths = harness.app().inner.disk_scan.projected_paths();
    let selected = harness
        .app()
        .inner
        .disk_scan
        .selected_path()
        .map(str::to_owned);
    let status = harness.app().inner.disk_scan.status_for_test();
    harness = harness.with_auto_draw(false);
    queued.exhaust_projection.set(true);

    // Runtime routes Enter to the published MAX-1 Tree before App::update
    // applies the queued test transition to the terminal projection.
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        queued.transitions.get(),
        1,
        "queued exhaustion runs once on Event"
    );
    assert!(queued.succeeded.get(), "the MAX-1 projection was exhausted");
    assert!(harness.app().inner.route == Route::DiskScan);
    assert_eq!(
        harness.app().inner.disk_scan.projection_for_test(),
        u64::MAX
    );
    assert_eq!(harness.app().inner.disk_scan.generation(), generation);
    assert_eq!(harness.app().inner.disk_scan.projected_paths(), paths);
    assert_eq!(
        harness.app().inner.disk_scan.selected_path(),
        selected.as_deref()
    );
    assert_eq!(harness.app().inner.disk_scan.status_for_test(), status);
    assert!(
        harness.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            termrock::Diagnostic::UndeliveredIntent { owner } if *owner == old_tree
        )),
        "MAX-1 input is explicitly rejected: {:?}",
        harness.diagnostics()
    );

    harness.draw();
    let terminal_text = harness.text();
    assert!(
        terminal_text.contains("Presentation identity exhausted"),
        "minimum-size status row exposes the cause: {terminal_text}"
    );
    let terminal_ids = [
        disk_scan::TREE,
        disk_scan::RESCAN,
        disk_scan::CANCEL,
        disk_scan::SORT,
        disk_scan::BACK,
        disk_scan::DETAILS,
    ]
    .map(|base| harness.app().inner.disk_scan.owner_id_for_test(base));
    let back = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::BACK);
    assert!(harness.area_of(back).is_some(), "terminal Back is visible");
    assert!(terminal_text.contains("Back"));
    for (index, owner) in terminal_ids.iter().enumerate() {
        assert!(
            terminal_ids
                .iter()
                .skip(index + 1)
                .all(|next| owner != next)
        );
        assert_ne!(*owner, old_tree, "terminal controls use fresh owner IDs");
    }

    let _ = harness.click_id(back);
    assert!(
        harness.app().inner.route == Route::Home,
        "native Back remains usable"
    );
    assert_eq!(harness.focus(), origin_focus);
    harness.draw();
    let reopened_return_focus = harness.focus();
    let _ = harness.key(KeyCode::F(10));
    harness.draw();
    let _ = harness.key(KeyCode::Right);
    harness.draw();
    let _ = harness.key(KeyCode::Down);
    harness.draw();
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().inner.route == Route::DiskScan);
    harness.draw();
    assert_eq!(
        harness.focus(),
        Some(
            harness
                .app()
                .inner
                .disk_scan
                .owner_id_for_test(disk_scan::TREE)
        )
    );
    let reopened_back = harness
        .app()
        .inner
        .disk_scan
        .owner_id_for_test(disk_scan::BACK);
    let _ = harness.click_id(reopened_back);
    assert!(harness.app().inner.route == Route::Home);
    assert_eq!(harness.focus(), reopened_return_focus);
    let reopened_generation = harness
        .app()
        .inner
        .disk_scan
        .generation()
        .expect("fresh controller generation opens");
    assert!(reopened_generation > generation.expect("previous generation exists"));
    assert!(harness.app().inner.disk_scan.projection_for_test() < u64::MAX);
    let reopened_ids = [
        disk_scan::TREE,
        disk_scan::RESCAN,
        disk_scan::CANCEL,
        disk_scan::SORT,
        disk_scan::BACK,
        disk_scan::DETAILS,
    ]
    .map(|base| harness.app().inner.disk_scan.owner_id_for_test(base));
    assert!(
        terminal_ids
            .iter()
            .all(|old| reopened_ids.iter().all(|new| old != new))
    );
    assert!(
        harness.diagnostics().iter().all(|diagnostic| matches!(
            diagnostic,
            termrock::Diagnostic::UndeliveredIntent { owner } if *owner == old_tree
        )),
        "only the retired MAX-1 Tree was rejected: {:?}",
        harness.diagnostics()
    );
}
