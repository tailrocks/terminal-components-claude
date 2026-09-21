//! JA-058: detach and still-inside.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA058_ID, JA058_SIZES, MOTION_SEED, ja058_detach_and_still_inside};

#[test]
fn ja058_detach_reconnect_outro_and_still_inside() {
    let captures = ja058_detach_and_still_inside();
    assert_eq!(captures.len(), JA058_SIZES.len());
    for capture in &captures {
        assert!(capture.detached);
        assert!(capture.reconnected);
        assert!(capture.outro_caption);
        assert!(capture.quit_after_outro);
        assert!(capture.still_inside);
        assert_eq!(capture.still_inside_running, 1);
        assert_eq!(capture.frames.len(), 5);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA058_ID), "{}", frame.identity);
        }
    }
}
