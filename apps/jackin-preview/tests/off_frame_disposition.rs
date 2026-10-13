//! Disposition of B45 `c4f47c954f88166ec25ddd78115119529bdae76b` behaviors
//! that are not on the locked visual frame.
//!
//! Ported, behind a key, with no `unsafe` and no `Box::leak`:
//! - Exit confirm names the running count from an owned `String`.
//! - Cockpit Cancel asks, then leaves without attaching an instance.
//! - Escape on a failed launch returns to the Manager and does not invent an
//!   instance. A second Escape still quits.
//!
//! Rejected:
//! - Cockpit debug-info stays closed. `LaunchRunning` frame 0 has no
//!   `otlp://collector.internal` telemetry, and cockpit Escape already
//!   acknowledges a failed launch.
//! - Dirty-editor Cancel/Discard/Save stays the status question. A modal
//!   would take the Right/Enter path `editor_edits_count_once_preview_then_saves_and_returns` uses.
//! - Manager help stays the hint layer. `?` must leave `Current directory`
//!   visible and the last row on Esc/Close.
//! - Choosing a workspace does not fold it. Enter keeps the expanded tree
//!   and opens the agent picker or the capsule.
//! - Mount isolation is set to readonly Clone once. A cycle would move the
//!   assertion in `editor_edits_count_once_preview_then_saves_and_returns`.
//! - Holla keeps `Route::Home` for every paused scenario. `DiskCleanup`'s
//!   source boot page and the grouped Finder default would replace `home.draw`.
//!   The other source pages are a second stack, not an overlay on Home,
//!   Plan, or Activity. Copy stays a simulated clipboard status and does
//!   not emit OSC 52.
//! - Locked-paint rejects stay out: manager meta tone, host-menu header
//!   replacement, prelude source-list replacement, accounts provider tree,
//!   capsule meters and extra menu rows, live keep-awake and git-pull
//!   checkboxes, settings co-author and DCO checkboxes, Showcase diff page,
//!   and `TablePro` safe-mode note wrapping. Visual suites cover those frames.

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

use junie_tui::KeyCode;

use jackin_app::{Motion, Route, Scenario};

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
        "the three-button exit dialog is not the locked editor path: {}",
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
    }
}
