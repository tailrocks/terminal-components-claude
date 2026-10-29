#![allow(
    dead_code,
    missing_docs,
    unreachable_pub,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use termrock::KeyCode;

use jackin_preview_app::{Motion, Route, Scenario};

mod support;
use support::H;

#[test]
fn quit_confirm_names_the_running_count_and_does_not_quit_until_confirmed() {
    let mut idle = H::new(Scenario::FirstUse, Motion::Paused, 0, 100, 30);
    assert!(!idle.text().contains("Exit jackin?"));
    idle.key(KeyCode::Char('q'));
    assert!(!idle.app().should_quit());
    assert!(idle.text().contains("Exit jackin?"), "{}", idle.text());
    assert!(
        idle.text().contains("No instances are running"),
        "{}",
        idle.text()
    );
    idle.key(KeyCode::Esc);
    assert!(!idle.app().should_quit());
    assert!(!idle.text().contains("Exit jackin?"), "{}", idle.text());

    let mut running = H::new(Scenario::Returning, Motion::Paused, 0, 120, 40);
    assert_eq!(running.app().world.running_count(), 2);
    assert!(running.text().contains("Current directory"));
    running.key(KeyCode::Char('q'));
    assert!(!running.app().should_quit());
    assert!(
        running.text().contains("2 instances keep running"),
        "{}",
        running.text()
    );
    let (x, y) = running.find("Quit").expect("quit action");
    running.click(x, y);
    assert!(running.app().should_quit());
}

#[test]
fn cockpit_cancel_confirms_before_leaving_and_attaches_nothing() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(h.app().launch().is_some());
    assert!(
        !h.text().contains("otlp://collector.internal"),
        "closed cockpit must not paint the source debug-info telemetry"
    );
    let before = h.app().world.instances.len();
    let (x, y) = h.find("Cancel").expect("cockpit cancel");
    h.click(x, y);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(h.app().launch().is_some());
    assert!(h.text().contains("Cancel the launch?"), "{}", h.text());
    assert!(h.text().contains("Nothing is attached"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(h.app().launch().is_some());
    assert!(!h.text().contains("Cancel the launch?"), "{}", h.text());
    h.click(x, y);
    let (confirm_x, confirm_y) = h.find("Cancel launch").expect("confirm");
    h.click(confirm_x, confirm_y);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.app().launch().is_none());
    assert_eq!(h.app().world.instances.len(), before);
    assert!(
        h.text().contains("Launch cancelled · nothing was attached"),
        "{}",
        h.text()
    );
}

#[test]
fn launch_failure_escape_returns_to_the_manager_without_inventing_an_instance() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Full, 0, 120, 40);
    let before = h.app().world.instances.len();
    for _ in 0..60 {
        h.ticks(10);
        if h.text().contains("Launch failed") {
            break;
        }
    }
    assert!(h.text().contains("Launch failed"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(!h.app().should_quit());
    assert!(h.text().contains("still running"), "{}", h.text());
    assert_eq!(h.app().world.instances.len(), before);
    h.key(KeyCode::Esc);
    assert!(h.app().should_quit());
}

#[test]
fn manager_help_stays_a_hint_layer_and_workspace_enter_does_not_fold() {
    let mut h = H::new(Scenario::Returning, Motion::Paused, 0, 120, 40);
    let workspace = h.app().world.workspaces.first().unwrap().id;
    h.key(KeyCode::Right);
    assert!(h.app().manager.is_expanded(workspace));
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
    assert!(
        h.text().contains("Current directory"),
        "help must not cover the manager frame: {}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(h.app().manager.is_expanded(workspace));
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Launch · choose Agent") || h.app().route() == Route::Capsule,
        "enter reconnects or opens the picker; it does not fold the tree: {}",
        h.text()
    );
    assert!(h.app().manager.is_expanded(workspace));
}

#[test]
fn dirty_editor_keeps_the_status_question_and_mount_isolation_stays_clone() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Mounts · 1 modified · readonly · worktree"),
        "{}",
        h.text()
    );
    assert!(h.app().editor.dirty);
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    assert_eq!(h.app().route(), Route::Editor);
    assert!(
        !h.text().contains("Discarded"),
        "opening the dialog does not discard: {}",
        h.text()
    );
    assert!(h.text().contains("Discard"), "{}", h.text());
}

#[test]
fn dirty_editor_discard_returns_to_the_manager_without_saving() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);

    let id = h.app().world.workspaces[0].id;
    let before = h
        .app()
        .world
        .workspace(id)
        .expect("workspace")
        .mounts
        .clone();
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.app().editor.dirty, "the draft must change before discard");
    assert_ne!(
        h.app().editor.pending.mounts,
        before,
        "discard is only meaningful when the draft mount differs"
    );
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    let (x, y) = h.find("Discard").expect("discard");
    h.click(x, y);
    assert_eq!(h.app().route(), Route::Manager);
    assert!(h.text().contains("Discarded 1 change"), "{}", h.text());
    assert!(!h.app().editor.dirty, "discard clears the draft");
    assert_eq!(
        h.app().editor.pending.mounts,
        before,
        "discard restores the persisted mounts"
    );
    let saved = h.app().world.workspace(id).expect("workspace");
    assert_eq!(saved.mounts, before, "discard must not write the workspace");
}

#[test]
fn escape_from_the_mount_control_focuses_save_before_the_leave_dialog() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::editor::SAVE));
    assert!(
        !h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Save changes before leaving?"),
        "{}",
        h.text()
    );
    assert_eq!(h.app().route(), Route::Editor);
}

#[test]
fn escape_from_a_new_environment_field_reaches_accounts() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Char('a'));
    assert!(
        h.app().editor.env_form_open,
        "a opens the variable form: {}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(!h.app().editor.env_form_open);
    assert_eq!(h.focus(), Some(jackin_preview_app::screens::editor::SAVE));
    h.key(KeyCode::Char(']'));
    assert!(
        h.text().contains("Active accounts"),
        "the bracket must change tabs: {}",
        h.text()
    );
}

#[test]
fn paused_frames_do_not_open_quit_or_cancel() {
    for scenario in [
        Scenario::FirstUse,
        Scenario::Returning,
        Scenario::HardCases,
        Scenario::AccountsMixed,
        Scenario::LaunchRunning,
        Scenario::LaunchFailure,
        Scenario::CapsuleMulti,
        Scenario::OutroLast,
    ] {
        let h = H::new(scenario, Motion::Paused, 0, 120, 40);
        assert!(!h.app().should_quit(), "{scenario:?}");
        assert!(!h.text().contains("Exit jackin?"), "{scenario:?}");
        assert!(!h.text().contains("Cancel the launch?"), "{scenario:?}");
        assert!(!h.text().contains("Debug info"), "{scenario:?}");
        assert!(
            !h.text().contains("otlp://collector.internal"),
            "{scenario:?}"
        );
    }
}

#[test]
fn cockpit_t_paints_the_approved_run_id_on_the_paused_launch() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 40, 120, 40);
    assert!(
        !h.text().contains("run-2026"),
        "run id stays hidden until t: {}",
        h.text()
    );
    h.key(KeyCode::Char('t'));
    assert!(h.text().contains("run-202609030914-b5df"), "{}", h.text());
    let cell = h.buffer().cell((97, 36)).expect("run id");
    assert_eq!(cell.symbol(), "r");
    assert_eq!(cell.fg, termrock::Color::from_u32(0x79_b8_ff));
}

#[test]
fn cockpit_x_paints_the_cancel_title_the_closed_cockpit_hides() {
    let closed = H::new(Scenario::LaunchRunning, Motion::Paused, 40, 120, 40);
    let before = closed.buffer().cell((36, 17)).expect("title cell");
    assert_ne!(before.symbol(), "C", "closed cockpit has no cancel title");
    let mut open = H::new(Scenario::LaunchRunning, Motion::Paused, 40, 120, 40);
    open.key(KeyCode::Char('x'));
    let cell = open.buffer().cell((36, 17)).expect("cancel title");
    assert_eq!(cell.symbol(), "C");
    assert_eq!(cell.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(cell.bg, termrock::Color::from_u32(0x00_18_18_1b));
    assert!(cell.modifier.contains(termrock::Modifier::BOLD));
    assert!(!cell.modifier.contains(termrock::Modifier::UNDERLINED));
}

#[test]
fn cockpit_i_paints_the_approved_debug_title_on_the_paused_launch() {
    let closed = H::new(Scenario::LaunchRunning, Motion::Paused, 40, 120, 40);
    let before = closed.buffer().cell((30, 14)).expect("title cell");
    assert_ne!(before.symbol(), "D", "closed cockpit has no debug title");
    let mut open = H::new(Scenario::LaunchRunning, Motion::Paused, 40, 120, 40);
    open.key(KeyCode::Char('i'));
    let cell = open.buffer().cell((30, 14)).expect("debug title");
    assert_eq!(cell.symbol(), "D");
    assert_eq!(cell.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(cell.bg, termrock::Color::from_u32(0x00_18_18_1b));
    assert!(cell.modifier.contains(termrock::Modifier::BOLD));
}

#[test]
fn cockpit_debug_info_opens_only_on_i_and_hides_telemetry_until_d() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(!h.text().contains("Debug info"));
    assert!(!h.text().contains("otlp://collector.internal"));
    h.key(KeyCode::Char('i'));
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(h.app().launch().is_some());
    assert!(!h.app().should_quit());
    assert!(h.text().contains("Debug info"), "{}", h.text());
    assert!(h.text().contains("Target"), "{}", h.text());
    assert!(
        !h.text().contains("otlp://collector.internal"),
        "telemetry stays hidden until d: {}",
        h.text()
    );
    h.key(KeyCode::Char('d'));
    assert!(
        h.text().contains("otlp://collector.internal:4317"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert_eq!(h.app().route(), Route::Cockpit);
    assert!(h.app().launch().is_some());
    assert!(!h.app().should_quit());
    assert!(!h.text().contains("Debug info"), "{}", h.text());
    assert!(
        !h.text().contains("otlp://collector.internal"),
        "{}",
        h.text()
    );

    let mut failed = H::new(Scenario::LaunchFailure, Motion::Full, 0, 120, 40);
    for _ in 0..60 {
        failed.ticks(10);
        if failed.text().contains("Launch failed") {
            break;
        }
    }
    assert!(failed.text().contains("Launch failed"), "{}", failed.text());
    failed.key(KeyCode::Char('i'));
    assert!(
        !failed.text().contains("Debug info"),
        "a failed launch must not open debug info: {}",
        failed.text()
    );
    failed.key(KeyCode::Esc);
    assert_eq!(failed.app().route(), Route::Manager);
    assert!(!failed.app().should_quit());
}
