//! JA-062: cockpit, handoff, failure.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA062_ID, JA062_SIZES, MOTION_SEED, ja062_launch_and_failure};

#[test]
fn ja062_cockpit_handoff_and_failure() {
    let captures = ja062_launch_and_failure();
    assert_eq!(captures.len(), JA062_SIZES.len());
    for capture in &captures {
        assert!(capture.cockpit_first);
        assert!(capture.handoff_caption);
        assert!(capture.capsule_settled);
        assert!(capture.failure_exact);
        assert!(capture.no_retry_surface);
        assert!(capture.failure_keys_inert);
        assert_eq!(capture.frames.len(), 5);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA062_ID), "{}", frame.identity);
        }
    }
}
