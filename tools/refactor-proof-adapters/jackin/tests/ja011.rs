//! JA-011: launch picker refuse/open/Esc.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA011_ID, JA011_SIZES, MOTION_SEED, ja011_launch_picker};

#[test]
fn ja011_covers_first_use_refuse_and_returning_picker() {
    let captures = ja011_launch_picker();
    assert_eq!(captures.len(), JA011_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA011_SIZES) {
        for frame in [
            &capture.first_use_refused,
            &capture.returning_open,
            &capture.returning_esc,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA011_ID), "{}", frame.identity);
        }
        assert_eq!(capture.first_use_refused.scenario, "first-use");
        assert_eq!(capture.first_use_refused.route, "manager");
        assert_eq!(capture.returning_open.scenario, "returning");
        assert_eq!(capture.returning_esc.scenario, "returning");
    }
}

#[test]
fn ja011_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja011_launch_picker(), ja011_launch_picker());
}
