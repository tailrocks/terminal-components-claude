use super::*;
use crate::domain::effect::Mutation;
use termrock::{Dialog, ItemKey, KeyCode, MouseKind, PartRef, Theme};
use termrock_test_support::Harness;

const CARGO_TARGET: &str = "~/work/forge/target";
const NODE_MODULES: &str = "~/work/monorepo/apps/frontend/node_modules";
const GRADLE: &str = "~/work/shop/.gradle";

fn disk_harness(width: u16, height: u16) -> Harness<App> {
    let app = App::for_scenario_without_history(Scenario::DiskCleanup, Motion::Paused, 80);
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

fn open_selection(harness: &mut Harness<App>) {
    assert!(harness.app().route == Route::Disk);
    let _ = harness.key(KeyCode::Char(' '));
    assert!(
        matches!(harness.app().overlay, Some(Overlay::DiskSelection(_))),
        "{}",
        harness.text()
    );
    assert!(harness.text().contains("Select disk candidates"));
    assert!(harness.text().contains("0 selected"));
}

fn click_candidate(harness: &mut Harness<App>, path: &str) {
    let row = PartRef::item(termrock::Part::ROW, ItemKey::text(path));
    assert!(
        harness.area_of_part(disk::SELECTION_LIST, row).is_some(),
        "candidate row is visible: {path}\n{}",
        harness.text()
    );
    let _ = harness.click_part(disk::SELECTION_LIST, row);
}

fn selected_paths(harness: &Harness<App>) -> Option<Vec<String>> {
    match harness.app().overlay.as_ref() {
        Some(Overlay::DiskSelection(selection)) => Some(selection.selected_paths_for_test()),
        _ => None,
    }
}

fn choose_review(harness: &mut Harness<App>) {
    let review = Dialog::new(disk::SELECTION_DIALOG).action_id(1);
    assert!(
        harness.tab_to(review),
        "Review is a real enabled Dialog action"
    );
    let _ = harness.key(KeyCode::Enter);
}

#[test]
fn disk_space_opens_an_empty_native_chooser_and_navigation_does_not_select() {
    let mut harness = disk_harness(100, 30);
    assert!(harness.text().contains("Space"));
    let before = harness.app().world.disk.clone();
    let revision = harness.app().world.effect_revision;

    open_selection(&mut harness);
    assert_eq!(harness.focus(), Some(disk::SELECTION_LIST));
    assert!(harness.text().contains("activity unknown"));
    assert!(harness.text().contains("active today"));

    let _ = harness.key(KeyCode::Down);
    assert_eq!(selected_paths(&harness), Some(Vec::new()));
    assert!(harness.text().contains("0 selected"));
    assert_eq!(harness.app().world.disk, before);
    assert_eq!(harness.app().world.effect_revision, revision);

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Disk);
    assert!(harness.app().overlay.is_none());
    assert_eq!(harness.app().world.disk, before);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.tick();
    open_selection(&mut harness);
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().overlay.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn click_and_space_select_exact_rows_while_active_today_stays_disabled() {
    let mut harness = disk_harness(100, 30);
    open_selection(&mut harness);

    click_candidate(&mut harness, NODE_MODULES);
    assert_eq!(selected_paths(&harness), Some(Vec::new()));
    assert!(harness.text().contains("active today"));

    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(selected_paths(&harness), Some(Vec::new()));
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(selected_paths(&harness), Some(vec![GRADLE.to_owned()]));
    assert!(harness.text().contains("activity unknown"));

    click_candidate(&mut harness, CARGO_TARGET);
    assert_eq!(
        selected_paths(&harness),
        Some(vec![CARGO_TARGET.to_owned(), GRADLE.to_owned()])
    );
    let next_click_elapsed = core::time::Duration::from_millis(
        harness
            .runtime()
            .theme()
            .design
            .motion
            .double_click_ms
            .saturating_add(1),
    );
    let _ = harness.advance(next_click_elapsed);
    click_candidate(&mut harness, CARGO_TARGET);
    assert_eq!(selected_paths(&harness), Some(vec![GRADLE.to_owned()]));
    assert!(harness.text().contains("1 selected"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn pointer_down_only_moves_cursor_and_release_click_toggles_the_exact_candidate() {
    let mut harness = disk_harness(100, 30);
    open_selection(&mut harness);

    let path = GRADLE;
    let row = PartRef::item(termrock::Part::ROW, ItemKey::text(path));
    let area = harness
        .area_of_part(disk::SELECTION_LIST, row)
        .expect("the selectable candidate row has published native geometry");
    let x = area.x.saturating_add(area.width / 2);
    let y = area.y;

    let _ = harness.mouse(MouseKind::Down, x, y);
    assert_eq!(selected_paths(&harness), Some(Vec::new()));
    assert!(harness.text().contains("0 selected"));

    let _ = harness.mouse(MouseKind::Up, x, y);
    assert_eq!(selected_paths(&harness), Some(vec![path.to_owned()]));
    assert!(harness.text().contains("1 selected"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn exact_unknown_candidate_uses_existing_phrase_gate_and_only_changes_its_target() {
    let mut harness = disk_harness(120, 40);
    let before = harness.app().world.disk.clone();
    let revision = harness.app().world.effect_revision;
    let selected = before.as_ref().and_then(|disk| {
        disk.candidates
            .iter()
            .find(|candidate| candidate.path == GRADLE)
            .cloned()
    });
    assert!(
        selected.is_some(),
        "DiskCleanup fixture has its Gradle candidate"
    );

    open_selection(&mut harness);
    click_candidate(&mut harness, GRADLE);
    assert_eq!(selected_paths(&harness), Some(vec![GRADLE.to_owned()]));
    choose_review(&mut harness);
    assert!(harness.app().route == Route::Plan, "{}", harness.text());
    assert!(harness.text().contains("1 selected candidates"));

    let plan = harness.app().plan.as_ref();
    assert!(plan.is_some());
    if let (Some(plan), Some(selected), Some(original)) = (plan, selected.as_ref(), before.as_ref())
    {
        let steps = plan.plan().steps();
        assert_eq!(steps.len(), 3, "preflight + selected removal + verify");
        assert_eq!(steps[1].id, format!("rm:{GRADLE}"));
        assert_eq!(
            steps[1].success_effects,
            vec![Mutation::RemoveDisk(selected.clone())]
        );
        assert!(steps.iter().all(|step| {
            step.id == "preflight" || step.id == "verify" || step.id == format!("rm:{GRADLE}")
        }));
        assert_eq!(plan.plan().phrase(), "DELETE GENERATED ARTIFACTS ON devbox");
        assert_eq!(harness.app().world.disk.as_ref(), Some(original));
        assert_eq!(harness.app().world.effect_revision, revision);
    }

    let phrase = harness
        .app()
        .plan
        .as_ref()
        .map(|plan| plan.plan().phrase().to_owned());
    assert!(phrase.is_some());
    if let Some(phrase) = phrase {
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str(&phrase);
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.key(KeyCode::Right);
        let _ = harness.key(KeyCode::Enter);
    }
    assert_eq!(
        harness.app().world.effect_revision,
        revision.saturating_add(1)
    );
    let resulting = harness.app().world.disk.as_ref();
    assert!(resulting.is_some());
    if let (Some(resulting), Some(selected), Some(original)) =
        (resulting, selected.as_ref(), before.as_ref())
    {
        let expected_candidates: Vec<_> = original
            .candidates
            .iter()
            .filter(|candidate| candidate.path != GRADLE)
            .cloned()
            .collect();
        assert_eq!(
            resulting.used_bytes,
            original.used_bytes - selected.size_bytes
        );
        assert_eq!(resulting.candidates, expected_candidates);
    }
    assert!(harness.text().contains("Finished:"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn wrong_disk_phrase_leaves_the_exact_selected_inventory_unchanged() {
    let mut harness = disk_harness(120, 40);
    let before = harness.app().world.disk.clone();
    open_selection(&mut harness);
    click_candidate(&mut harness, GRADLE);
    choose_review(&mut harness);
    assert!(harness.app().route == Route::Plan, "{}", harness.text());

    let _ = harness.key(KeyCode::Enter);
    let _ = harness.type_str("DELETE GENERATED ARTIFACTS ON another-host");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().world.effect_revision, 0);
    assert_eq!(harness.app().world.disk, before);
    assert!(!harness.text().contains("Finished:"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn stale_opening_host_cwd_revision_or_full_inventory_cannot_enter_plan_review() {
    for changed in 0..4 {
        let mut harness = disk_harness(100, 30);
        open_selection(&mut harness);
        click_candidate(&mut harness, CARGO_TARGET);
        assert_eq!(
            selected_paths(&harness),
            Some(vec![CARGO_TARGET.to_owned()])
        );
        let review = Dialog::new(disk::SELECTION_DIALOG).action_id(1);
        assert!(harness.tab_to(review));
        match changed {
            0 => harness.app_mut().world.host.name.push_str("-changed"),
            1 => harness.app_mut().world.cwd.push_str("/changed"),
            2 => harness.app_mut().world.effect_revision = 1,
            _ => {
                if let Some(disk) = harness.app_mut().world.disk.as_mut()
                    && let Some(candidate) = disk
                        .candidates
                        .iter_mut()
                        .find(|candidate| candidate.path == GRADLE)
                {
                    candidate.size_bytes = candidate.size_bytes.saturating_add(1);
                }
            }
        }
        let revision = harness.app().world.effect_revision;
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().route == Route::Disk, "{}", harness.text());
        assert!(harness.app().plan.is_none());
        assert!(harness.app().overlay.is_none());
        assert_eq!(harness.app().world.effect_revision, revision);
        assert!(
            harness
                .app()
                .status
                .as_deref()
                .is_some_and(|status| status.contains("Disk inventory changed")),
            "{}",
            harness.text()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn chooser_keeps_its_native_layer_across_too_small_and_resize_frames() {
    let mut harness = disk_harness(80, 24);
    open_selection(&mut harness);
    let _ = harness.resize(60, 15);
    harness.draw();
    assert!(harness.text().contains("too small"));
    assert!(matches!(
        harness.app().overlay,
        Some(Overlay::DiskSelection(_))
    ));

    let _ = harness.resize(100, 30);
    harness.draw();
    assert!(harness.text().contains("Select disk candidates"));
    assert!(matches!(
        harness.app().overlay,
        Some(Overlay::DiskSelection(_))
    ));
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().overlay.is_none());
    assert!(harness.app().route == Route::Disk);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn chooser_publishes_geometry_labels_and_cancel_at_each_color_level() {
    let colors = [
        termrock::ColorLevel::TrueColor,
        termrock::ColorLevel::Ansi256,
        termrock::ColorLevel::Ansi16,
        termrock::ColorLevel::Mono,
    ];
    for (width, height) in [(80, 24), (120, 40)] {
        for color in colors {
            let mut harness = disk_harness(width, height).with_color(color);
            assert!(harness.app().route == Route::Disk);
            let _ = harness.key(KeyCode::Char(' '));
            assert!(matches!(
                harness.app().overlay,
                Some(Overlay::DiskSelection(_))
            ));
            let dialog = harness
                .area_of(disk::SELECTION_DIALOG)
                .expect("the dialog root has published geometry");
            let list = harness
                .area_of(disk::SELECTION_LIST)
                .expect("the native list has published geometry");
            let row = harness
                .area_of_part(
                    disk::SELECTION_LIST,
                    PartRef::item(termrock::Part::ROW, ItemKey::text(CARGO_TARGET)),
                )
                .expect("the candidate row has published item geometry");
            assert!(dialog.width > 0 && dialog.height > 0);
            assert!(list.width > 0 && list.height > 0);
            assert!(row.width > 0 && row.height > 0);
            assert!(harness.text().contains("activity unknown"));
            assert!(harness.text().contains("active today"));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );

            let _ = harness.key(KeyCode::Esc);
            assert!(harness.app().overlay.is_none());
            assert!(harness.app().route == Route::Disk);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
}
