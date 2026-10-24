use super::*;
use crate::sim::{FileEntry, FilesError, FilesSnapshot};
use termrock::{Dialog, ItemKey, KeyCode, PartRef, Theme};
use termrock_test_support::Harness;

fn harness(app: App, width: u16, height: u16) -> Harness<App> {
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

fn app_with_files() -> Harness<App> {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let snapshot = crate::sim::fixtures::files_for_cwd(&cwd);
    harness(app.with_files_snapshot(snapshot), 100, 30)
}

fn open_files_from_folder_dialog(harness: &mut Harness<App>) {
    let _ = harness.key(KeyCode::F(4));
    assert!(harness.text().contains("Folder browser"));
    assert!(harness.text().contains("Directory"));
    let browse = Dialog::new(dialogs::DIALOG).action_id(1);
    assert!(
        harness.tab_to(browse),
        "Browse action must be a real dialog owner"
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::Files);
    assert_eq!(harness.focus(), Some(files::LIST));
}

#[test]
fn f4_browse_uses_real_snapshot_list_click_selection_and_keyed_navigation() {
    let mut harness = app_with_files();
    let initial_focus = harness.focus();
    assert!(
        initial_focus.is_some(),
        "the caller route has a focused owner"
    );
    let cwd = harness.app().world.cwd.clone();
    let revision = harness.app().world.effect_revision;
    let activities = harness.app().world.activities.clone();

    open_files_from_folder_dialog(&mut harness);
    assert_eq!(harness.app().files.directory(), cwd);
    assert!(harness.text().contains("archive"));
    assert!(harness.text().contains("drafts"));
    assert!(harness.text().contains("README.md"));

    let visible: Vec<_> = harness
        .app()
        .files
        .visible_paths()
        .map(str::to_owned)
        .collect();
    assert!(visible.iter().any(|path| path.ends_with("/drafts")));
    for path in &visible {
        assert!(
            harness
                .area_of_part(files::LIST, PartRef::item(Part::ROW, ItemKey::text(path)))
                .is_some(),
            "each displayed path must have its keyed row hit: {path}"
        );
    }

    let broken = FilesSnapshot::join(&cwd, "broken-shortcut");
    let broken_part = PartRef::item(Part::ROW, ItemKey::text(&broken));
    assert!(harness.area_of_part(files::LIST, broken_part).is_some());
    let _ = harness.click_part(files::LIST, broken_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.directory(), cwd);
    assert_eq!(
        harness.app().files.activation_error(),
        Some(&FilesError::BrokenLink(broken.clone()))
    );

    let drafts = FilesSnapshot::join(&cwd, "drafts");
    let drafts_part = PartRef::item(Part::ROW, ItemKey::text(&drafts));
    assert!(harness.area_of_part(files::LIST, drafts_part).is_some());
    let _ = harness.click_part(files::LIST, drafts_part);
    assert_eq!(harness.app().files.directory(), cwd);
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));

    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.directory(), drafts);
    assert!(harness.text().contains("meeting-notes.md"));

    let _ = harness.key(KeyCode::Left);
    assert_eq!(harness.app().files.directory(), cwd);
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));
    let _ = harness.key(KeyCode::Char('h'));
    assert!(harness.app().files.show_hidden());
    assert!(harness.text().contains(".notes-index"));
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));
    let _ = harness.key(KeyCode::Char('h'));
    assert!(!harness.app().files.show_hidden());
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));

    let _ = harness.resize(120, 40);
    harness.draw();
    harness.draw();
    assert_eq!(harness.app().files.directory(), cwd);
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));
    assert_eq!(harness.app().world.cwd, cwd);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert_eq!(harness.app().world.activities, activities);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.focus(), initial_focus);
    assert_eq!(harness.app().world.cwd, cwd);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn files_keyboard_navigation_restores_shared_menu_focus() {
    let mut harness = app_with_files();
    assert!(harness.tab_to(MENU));
    assert!(!harness.app().menu.is_open());
    assert_eq!(harness.focus(), Some(MENU));
    let caller_route = harness.app().route;

    let _ = harness.key(KeyCode::F(4));
    assert!(harness.text().contains("Folder browser"));
    let browse = Dialog::new(dialogs::DIALOG).action_id(1);
    assert!(
        harness.tab_to(browse),
        "Browse action must be a real dialog owner"
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::Files);
    assert_eq!(harness.focus(), Some(files::LIST));

    let cwd = harness.app().world.cwd.clone();
    let archive = FilesSnapshot::join(&cwd, "archive");
    let drafts = FilesSnapshot::join(&cwd, "drafts");
    let visible = harness
        .app()
        .files
        .visible_paths()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(visible.contains(&archive));
    assert!(visible.contains(&drafts));

    let _ = harness.key(KeyCode::Home);
    assert_eq!(harness.app().files.selected_path(), Some(archive.as_str()));
    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));
    let _ = harness.key(KeyCode::Right);
    assert!(harness.app().files.directory() == drafts);
    let meeting_notes = FilesSnapshot::join(&drafts, "meeting-notes.md");
    assert_eq!(
        harness.app().files.selected_path(),
        Some(meeting_notes.as_str())
    );
    let _ = harness.key(KeyCode::Left);
    assert!(harness.app().files.directory() == cwd);
    assert_eq!(harness.app().files.selected_path(), Some(drafts.as_str()));

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == caller_route);
    assert_eq!(harness.focus(), Some(MENU));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn folder_dialog_close_and_escape_keep_the_existing_route_and_focus() {
    let mut by_escape = app_with_files();
    let initial_focus = by_escape.focus();
    let _ = by_escape.key(KeyCode::F(4));
    let _ = by_escape.key(KeyCode::Esc);
    assert!(by_escape.app().route == Route::Home);
    assert_eq!(by_escape.focus(), initial_focus);
    assert!(
        by_escape.diagnostics().is_empty(),
        "{:?}",
        by_escape.diagnostics()
    );

    let mut by_close = app_with_files();
    let initial_focus = by_close.focus();
    let _ = by_close.key(KeyCode::F(4));
    let close = Dialog::new(dialogs::DIALOG).action_id(0);
    assert!(by_close.tab_to(close));
    let _ = by_close.key(KeyCode::Enter);
    assert!(by_close.app().route == Route::Home);
    assert_eq!(by_close.focus(), initial_focus);
    assert!(
        by_close.diagnostics().is_empty(),
        "{:?}",
        by_close.diagnostics()
    );
}

#[test]
fn empty_and_unreadable_directories_render_and_return_to_the_caller() {
    let mut empty_snapshot = FilesSnapshot::new();
    empty_snapshot.insert(FileEntry::directory("~/work/probe"));
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
        .with_files_snapshot(empty_snapshot);
    let mut empty = harness(app, 100, 30);
    let home_focus = empty.focus();
    open_files_from_folder_dialog(&mut empty);
    assert!(empty.text().contains("No visible entries"));
    assert_eq!(empty.app().files.listing_error(), None);
    assert_eq!(empty.focus(), Some(files::LIST));
    assert!(empty.diagnostics().is_empty(), "{:?}", empty.diagnostics());
    let _ = empty.key(KeyCode::Esc);
    assert!(empty.app().route == Route::Home);
    assert_eq!(empty.focus(), home_focus);

    let mut denied_snapshot = FilesSnapshot::new();
    denied_snapshot.insert(FileEntry::directory("~/work/probe").unreadable());
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
        .with_files_snapshot(denied_snapshot);
    let mut denied = harness(app, 100, 30);
    let home_focus = denied.focus();
    open_files_from_folder_dialog(&mut denied);
    assert!(denied.text().contains("permission denied"));
    assert_eq!(
        denied.app().files.listing_error(),
        Some(&FilesError::PermissionDenied("~/work/probe".into()))
    );
    assert_eq!(denied.focus(), Some(files::LIST));
    assert!(
        denied.diagnostics().is_empty(),
        "{:?}",
        denied.diagnostics()
    );
    let _ = denied.key(KeyCode::Esc);
    assert!(denied.app().route == Route::Home);
    assert_eq!(denied.focus(), home_focus);
    assert!(
        denied.diagnostics().is_empty(),
        "{:?}",
        denied.diagnostics()
    );
}

#[test]
fn unreadable_and_missing_directories_keep_their_typed_error_state() {
    let mut snapshot = FilesSnapshot::new();
    snapshot.insert(FileEntry::directory("/"));
    snapshot.insert(FileEntry::directory("/work"));
    snapshot.insert(FileEntry::directory("/work/locked").unreadable());
    let locked = files::State::new(snapshot.clone(), "/work/locked");
    assert_eq!(
        locked.listing_error(),
        Some(&FilesError::PermissionDenied("/work/locked".into()))
    );

    let missing = files::State::new(snapshot, "/work/missing");
    assert_eq!(
        missing.listing_error(),
        Some(&FilesError::NotFound("/work/missing".into()))
    );
}
