//! JA-061: outro ritual.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA061_ENTERED_AT_MS, JA061_ID, JA061_SIZES, MOTION_SEED, ja061_outro_ritual};

#[test]
fn ja061_warp_caption_and_quit() {
    let captures = ja061_outro_ritual();
    assert_eq!(captures.len(), JA061_SIZES.len());
    for capture in &captures {
        assert!(capture.warp_before_caption);
        assert_eq!(capture.entered_at_ms, Some(JA061_ENTERED_AT_MS));
        assert!(capture.redundant_inert);
        assert!(capture.caption_exact);
        assert!(capture.quit_at_end);
        assert_eq!(capture.frames.len(), 4);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA061_ID), "{}", frame.identity);
        }
    }
}
