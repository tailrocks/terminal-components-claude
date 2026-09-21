//! JA-025: leave branches and fail-once retry.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA025_ID, JA025_SIZES, MOTION_SEED, ja025_leave_branches};

#[test]
fn ja025_covers_leave_status_stay_save_and_retry() {
    let captures = ja025_leave_branches();
    assert_eq!(captures.len(), JA025_SIZES.len());
    for capture in &captures {
        for frame in [
            &capture.general_attempt,
            &capture.dirty,
            &capture.leave_status,
            &capture.stay,
            &capture.preview,
            &capture.saved,
            &capture.failed,
            &capture.retried,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA025_ID), "{}", frame.identity);
        }
        assert!(
            !capture.general_dirty,
            "General keys must not dirty the draft"
        );
        assert_eq!(capture.dirty.route, "editor");
        assert!(
            capture.dirty.text.contains("• 1 change"),
            "{}",
            capture.dirty.text
        );
        assert_eq!(capture.leave_status.route, "editor");
        assert!(
            capture
                .leave_status
                .text
                .contains("Save changes before leaving?"),
            "{}",
            capture.leave_status.text
        );
        assert_eq!(capture.stay.route, "editor");
        assert!(capture.stay_dirty, "Stay must preserve the dirty draft");
        assert!(
            capture.preview.text.contains("Save workspace"),
            "{}",
            capture.preview.text
        );
        assert_eq!(capture.saved.route, "manager");
        assert!(capture.failed_dirty, "failed save must keep edits");
        assert_eq!(capture.failed.route, "editor");
        assert_eq!(capture.retried.route, "manager");
    }
}

#[test]
fn ja025_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja025_leave_branches(), ja025_leave_branches());
}
