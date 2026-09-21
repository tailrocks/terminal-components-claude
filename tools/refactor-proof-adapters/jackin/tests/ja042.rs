//! JA-042: launch failure replay and solo variant.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA042_ID, MOTION_SEED, ja042_launch_failure};

#[test]
fn ja042_failure_returns_or_stays_by_instance_count() {
    let captures = ja042_launch_failure();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.failed,
            &capture.acknowledged,
            &capture.solo_failed,
            &capture.solo_acknowledged,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA042_ID), "{}", frame.identity);
        }
        assert!(
            capture.failed.text.contains("Launch failed"),
            "{}",
            capture.failed.text
        );
        assert!(
            capture.failed.text.contains("Network"),
            "{}",
            capture.failed.text
        );
        assert_eq!(capture.acknowledged.route, "manager");
        // Narrow status lines clip the trailing reason.
        if viewport.width < 120 {
            assert!(
                capture.acknowledged.text.contains("Launch failed"),
                "{}",
                capture.acknowledged.text
            );
        } else {
            assert!(
                capture.acknowledged.text.contains("still running"),
                "{}",
                capture.acknowledged.text
            );
        }
        assert_eq!(capture.solo_running, 0);
        assert!(
            capture.solo_failed.text.contains("Network"),
            "{}",
            capture.solo_failed.text
        );
    }
}

#[test]
fn ja042_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja042_launch_failure(), ja042_launch_failure());
}
