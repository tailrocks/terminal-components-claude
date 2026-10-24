use super::*;
use crate::sim::{FileEntry, FilesError, FilesSnapshot};
use termrock::{Axis, Dialog, ItemKey, KeyCode, PartRef, Theme};
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

#[test]
fn file_preview_opens_on_activation_and_escape_closes_before_returning_to_caller() {
    let mut harness = app_with_files();
    let caller_focus = harness.focus();
    let cwd = harness.app().world.cwd.clone();
    let revision = harness.app().world.effect_revision;
    let activities = harness.app().world.activities.clone();
    open_files_from_folder_dialog(&mut harness);

    let readme = FilesSnapshot::join(&cwd, "README.md");
    let readme_part = PartRef::item(Part::ROW, ItemKey::text(&readme));
    assert!(harness.area_of_part(files::LIST, readme_part).is_some());
    let _ = harness.click_part(files::LIST, readme_part);
    assert_eq!(harness.app().files.selected_path(), Some(readme.as_str()));
    assert_eq!(harness.app().files.preview_path(), None);
    assert_eq!(harness.focus(), Some(files::LIST));

    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));
    assert_eq!(harness.app().files.selected_path(), Some(readme.as_str()));
    assert_eq!(harness.focus(), Some(files::PREVIEW));
    assert!(harness.text().contains("1 # Simulated workspace"));
    assert!(
        harness
            .text()
            .contains("3 This file is part of the in-memory fixture.")
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Char('h'));
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));
    assert_eq!(harness.app().files.directory(), cwd);
    assert!(!harness.app().files.show_hidden());

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Files);
    assert_eq!(harness.app().files.preview_path(), None);
    assert_eq!(harness.app().files.selected_path(), Some(readme.as_str()));
    assert_eq!(harness.focus(), Some(files::LIST));
    assert_eq!(harness.app().world.cwd, cwd);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert_eq!(harness.app().world.activities, activities);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    // Reopen the same selected file to prove the retiring viewport owner can be
    // recreated cleanly after its actual FocusOut settle.
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.app().files.preview_path(),
        Some(readme.as_str()),
        "after re-enter: focus={:?}, diagnostics={:?}, selected={:?}",
        harness.focus(),
        harness.diagnostics(),
        harness.app().files.selected_path()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(harness.app().files.preview_path(), None);
    assert_eq!(harness.focus(), Some(files::LIST));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().route == Route::Home);
    assert_eq!(harness.focus(), caller_focus);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn changing_route_with_preview_settles_the_retired_viewport_owner() {
    let app = App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let snapshot = crate::sim::fixtures::files_for_cwd(&cwd);
    let mut harness = harness(app.with_files_snapshot(snapshot), 100, 30);
    open_files_from_folder_dialog(&mut harness);

    let readme = FilesSnapshot::join(&cwd, "README.md");
    let readme_part = PartRef::item(Part::ROW, ItemKey::text(&readme));
    let _ = harness.click_part(files::LIST, readme_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.focus(), Some(files::PREVIEW));
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    // The global Activities command changes route without first closing the preview.
    // The App retires the real viewport owner in the following Settle.
    let _ = harness.ctrl('a');
    assert!(harness.app().route == Route::Activity);
    let activity_focus = harness
        .app()
        .activities
        .current_id()
        .map(activity::output_id);
    assert!(activity_focus.is_some(), "the route has a current activity");
    assert_eq!(harness.focus(), activity_focus);
    assert_eq!(harness.app().files.preview_path(), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn preview_focus_loss_before_route_change_allows_files_reentry() {
    let app = App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let snapshot = crate::sim::fixtures::files_for_cwd(&cwd);
    let mut harness = harness(app.with_files_snapshot(snapshot), 100, 30);
    open_files_from_folder_dialog(&mut harness);

    let readme = FilesSnapshot::join(&cwd, "README.md");
    let readme_part = PartRef::item(Part::ROW, ItemKey::text(&readme));
    let _ = harness.click_part(files::LIST, readme_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.focus(), Some(files::PREVIEW));
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));

    // Move focus to the real shared menu owner while the preview remains visible.
    // Its FocusOut is delivered before the route changes, so no later Preview
    // FocusOut will be available to retire the hidden session.
    assert!(harness.tab_to(MENU));
    assert!(!harness.app().menu.is_open());
    assert_eq!(harness.focus(), Some(MENU));
    assert_eq!(harness.app().files.preview_path(), Some(readme.as_str()));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.ctrl('a');
    assert!(harness.app().route == Route::Activity);
    assert_eq!(harness.app().files.preview_path(), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    // Browse back into Files and exercise a different row. If the already-blurred
    // preview session remained hidden in State, it would ignore this List input.
    open_files_from_folder_dialog(&mut harness);
    assert_eq!(harness.app().files.selected_path(), Some(readme.as_str()));
    let resume = FilesSnapshot::join(&cwd, "résumé.txt");
    let resume_part = PartRef::item(Part::ROW, ItemKey::text(&resume));
    assert!(harness.area_of_part(files::LIST, resume_part).is_some());
    let _ = harness.click_part(files::LIST, resume_part);
    assert_eq!(
        harness.app().files.selected_path(),
        Some(resume.as_str()),
        "after résumé row click: focus={:?}, diagnostics={:?}, preview={:?}",
        harness.focus(),
        harness.diagnostics(),
        harness.app().files.preview_path()
    );
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.preview_path(), Some(resume.as_str()));
    assert_eq!(harness.focus(), Some(files::PREVIEW));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn file_symlink_binary_empty_unmodeled_and_special_previews_are_safe() {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let text_path = FilesSnapshot::join(&cwd, "target.txt");
    let link_path = FilesSnapshot::join(&cwd, "link.txt");
    let binary_path = FilesSnapshot::join(&cwd, "binary.dat");
    let empty_path = FilesSnapshot::join(&cwd, "empty.txt");
    let unmodeled_path = FilesSnapshot::join(&cwd, "unmodeled.txt");
    let device_path = FilesSnapshot::join(&cwd, "device");
    let mut snapshot = FilesSnapshot::new();
    snapshot.insert(FileEntry::directory(cwd.clone()));
    snapshot.insert(FileEntry::file_with_bytes(
        text_path.clone(),
        b"resolved text\n".to_vec(),
    ));
    snapshot.insert(FileEntry::symlink(link_path.clone(), text_path.clone()));
    snapshot.insert(FileEntry::file_with_bytes(
        binary_path.clone(),
        b"private\0bytes".to_vec(),
    ));
    snapshot.insert(FileEntry::file_with_bytes(
        empty_path.clone(),
        Vec::<u8>::new(),
    ));
    snapshot.insert(FileEntry::file(unmodeled_path.clone()));
    snapshot.insert(FileEntry::special(device_path.clone()));
    let mut harness = harness(app.with_files_snapshot(snapshot), 100, 30);
    open_files_from_folder_dialog(&mut harness);

    let link_part = PartRef::item(Part::ROW, ItemKey::text(&link_path));
    let _ = harness.click_part(files::LIST, link_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().files.preview_path(), Some(link_path.as_str()));
    assert!(harness.text().contains(&format!("link → {text_path}")));
    assert!(harness.text().contains("1 resolved text"));

    let _ = harness.key(KeyCode::Left);
    assert!(harness.app().route == Route::Files);
    assert_eq!(harness.app().files.directory(), cwd);
    assert_eq!(
        harness.app().files.selected_path(),
        Some(link_path.as_str())
    );
    let binary_part = PartRef::item(Part::ROW, ItemKey::text(&binary_path));
    let _ = harness.click_part(files::LIST, binary_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.app().files.preview_path(),
        Some(binary_path.as_str())
    );
    assert!(harness.text().contains("Binary content is not shown"));
    assert!(!harness.text().contains("private"));

    let _ = harness.key(KeyCode::Left);
    let empty_part = PartRef::item(Part::ROW, ItemKey::text(&empty_path));
    let _ = harness.click_part(files::LIST, empty_part);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.text().contains("Empty file"));
    let _ = harness.key(KeyCode::Left);

    let unmodeled_part = PartRef::item(Part::ROW, ItemKey::text(&unmodeled_path));
    let _ = harness.click_part(files::LIST, unmodeled_part);
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .text()
            .contains("Content was not modeled for this file")
    );
    let _ = harness.key(KeyCode::Left);

    let device_part = PartRef::item(Part::ROW, ItemKey::text(&device_path));
    let _ = harness.click_part(files::LIST, device_part);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.app().files.preview_path(),
        Some(device_path.as_str())
    );
    assert!(harness.text().contains("Special files are not read"));
    assert!(!harness.text().contains("device: special"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn preview_uses_real_viewport_wheels_and_resets_offsets_for_another_file() {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let cwd = app.world.cwd.clone();
    let wide_path = FilesSnapshot::join(&cwd, "a-wide.txt");
    let next_path = FilesSnapshot::join(&cwd, "b-next.txt");
    let mut content = String::new();
    for index in 0..80 {
        if index == 79 {
            content.push_str(&format!(
                "row {index:02} {} HORIZONTAL-END\n",
                "x".repeat(140)
            ));
        } else {
            content.push_str(&format!("row {index:02} {}\n", "x".repeat(140)));
        }
    }
    let mut snapshot = FilesSnapshot::new();
    snapshot.insert(FileEntry::directory(cwd.clone()));
    snapshot.insert(FileEntry::file_with_bytes(
        wide_path.clone(),
        content.into_bytes(),
    ));
    snapshot.insert(FileEntry::file_with_bytes(
        next_path.clone(),
        b"second file starts at the first line\n".to_vec(),
    ));
    let mut harness = harness(app.with_files_snapshot(snapshot), 100, 30);
    open_files_from_folder_dialog(&mut harness);
    let wide_part = PartRef::item(Part::ROW, ItemKey::text(&wide_path));
    let _ = harness.click_part(files::LIST, wide_part);
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let text_area = harness.area_of_part(files::PREVIEW, PartRef::of(Part::TEXT));
    assert!(text_area.is_some(), "preview text has a real viewport part");
    let Some(text_area) = text_area else { return };
    for _ in 0..24 {
        let _ = harness.wheel(Axis::H, 1, text_area.x.saturating_add(1), text_area.y);
    }
    let offsets = harness.app().files.preview_offsets();
    assert!(offsets.is_some(), "open preview owns viewport offsets");
    let Some((_, horizontal)) = offsets else {
        return;
    };
    assert!(horizontal > 0, "horizontal wheel pans the long line");
    let _ = harness.key(KeyCode::End);
    let offsets = harness.app().files.preview_offsets();
    assert!(offsets.is_some(), "open preview retains viewport offsets");
    let Some((vertical, horizontal)) = offsets else {
        return;
    };
    assert!(vertical > 0, "End moves the real viewport to later lines");
    assert!(horizontal > 0, "vertical movement keeps the horizontal pan");
    assert!(harness.text().contains("HORIZONTAL-END"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let _ = harness.resize(120, 40);
    harness.draw();
    let resized_text_area = harness.area_of_part(files::PREVIEW, PartRef::of(Part::TEXT));
    assert!(
        resized_text_area.is_some(),
        "resized preview retains its real text area"
    );
    let Some(resized_text_area) = resized_text_area else {
        return;
    };
    assert!(resized_text_area.width > text_area.width);
    assert!(resized_text_area.height > text_area.height);
    assert!(harness.text().contains("HORIZONTAL-END"));

    let _ = harness.key(KeyCode::Left);
    assert_eq!(harness.focus(), Some(files::LIST));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let next_part = PartRef::item(Part::ROW, ItemKey::text(&next_path));
    let _ = harness.click_part(files::LIST, next_part);
    let _ = harness.key(KeyCode::Right);
    assert_eq!(harness.app().files.preview_path(), Some(next_path.as_str()));
    assert_eq!(harness.app().files.preview_offsets(), Some((0, 0)));
    assert!(
        harness
            .text()
            .contains("1 second file starts at the first line")
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
