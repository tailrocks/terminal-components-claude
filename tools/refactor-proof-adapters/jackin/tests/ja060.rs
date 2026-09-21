//! JA-060: motion policies.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA060_ID, JA060_PINNED_FRAME, JA060_SIZES, MOTION_SEED, ja060_motion_policies,
};

#[test]
fn ja060_resolve_routes_pause_and_pin() {
    let captures = ja060_motion_policies();
    assert_eq!(captures.len(), JA060_SIZES.len());
    for capture in &captures {
        assert!(capture.resolve_matrix);
        assert!(capture.routes_match);
        assert!(capture.frames_differ);
        assert!(capture.paused_frozen);
        assert_eq!(capture.pinned_frame, JA060_PINNED_FRAME);
        assert!(capture.pin45_static);
        assert!(capture.pin_phrase_differs);
        assert_eq!(capture.frames.len(), 7);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA060_ID), "{}", frame.identity);
        }
    }
}
