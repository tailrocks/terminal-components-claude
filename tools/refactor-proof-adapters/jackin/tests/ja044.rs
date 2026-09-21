//! JA-044: launch cancel variants.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA044_ID, JA044_SIZES, JA044_VARIANT_NAMES, MOTION_SEED, ja044_launch_cancel,
};

#[test]
fn ja044_covers_cancel_confirm_and_detach() {
    let captures = ja044_launch_cancel();
    assert_eq!(captures.len(), JA044_SIZES.len());
    assert_eq!(JA044_VARIANT_NAMES.len(), 4);
    for capture in &captures {
        assert_eq!(capture.variants.len(), JA044_VARIANT_NAMES.len());
        assert_eq!(capture.cancelled.len(), JA044_VARIANT_NAMES.len());
        for frame in &capture.variants {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA044_ID), "{}", frame.identity);
            assert!(!frame.quit, "{}", frame.identity);
        }
        // No variant cancels the run or quits; only the `Esc` departs.
        assert!(capture.cancelled.iter().all(|c| !c));
        assert_eq!(capture.variants[0].route, "cockpit");
        assert_eq!(capture.variants[1].route, "manager");
        assert_eq!(capture.variants[2].route, "cockpit");
        assert_eq!(capture.variants[3].route, "cockpit");
    }
}

#[test]
fn ja044_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja044_launch_cancel(), ja044_launch_cancel());
}
