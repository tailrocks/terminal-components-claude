//! JA-018: editor save replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA018_ID, JA018_SIZES, MOTION_SEED, ja018_editor_save_replay};

#[test]
fn ja018_covers_dirty_leave_preview_and_save() {
    let captures = ja018_editor_save_replay();
    assert_eq!(captures.len(), JA018_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA018_SIZES) {
        for frame in [
            &capture.editor,
            &capture.after_edits,
            &capture.after_leave_cancel,
            &capture.after_preview,
            &capture.after_confirm,
            &capture.after_saved,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA018_ID), "{}", frame.identity);
        }
        assert_eq!(capture.editor.route, "editor");
        assert_eq!(capture.after_leave_cancel.route, "editor");
        assert!(
            capture.after_saved.route == "manager" || capture.after_saved.route == "editor",
            "{}",
            capture.after_saved.route
        );
    }
}

#[test]
fn ja018_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja018_editor_save_replay(), ja018_editor_save_replay());
}
