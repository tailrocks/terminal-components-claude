//! JA-006: returning manager pointer, wheel, and seam drag.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA006_ID, MOTION_SEED, ja006_returning_manager_pointer};

#[test]
fn ja006_covers_pointer_wheel_and_seam_all_sizes() {
    let captures = ja006_returning_manager_pointer();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.expanded,
            &capture.click_workspace,
            &capture.double_click_workspace,
            &capture.click_instance,
            &capture.wheel_tree,
            &capture.wheel_detail,
            &capture.drag_right,
            &capture.drag_left,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion, "full");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA006_ID), "{}", frame.identity);
        }
        assert!(
            capture.expanded.text.contains("7f3a")
                || capture.expanded.text.contains("running")
                || capture.expanded.text.contains("workspace"),
            "{}",
            capture.expanded.text
        );
    }
}

#[test]
fn ja006_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja006_returning_manager_pointer(),
        ja006_returning_manager_pointer()
    );
}
