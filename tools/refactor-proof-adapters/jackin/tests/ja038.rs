//! JA-038: op item picker interactions.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA038_ID, JA038_SIZES, MOTION_SEED, ja038_op_item_picker};

#[test]
fn ja038_covers_picker_query_scroll_wheel_and_resize() {
    let captures = ja038_op_item_picker();
    assert_eq!(captures.len(), JA038_SIZES.len());
    for capture in &captures {
        for frame in [
            &capture.queried,
            &capture.scrolled,
            &capture.no_match,
            &capture.stepped_back,
            &capture.wheeled,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA038_ID), "{}", frame.identity);
            assert_eq!(frame.route, "accounts", "{}", frame.identity);
        }
        for frame in [&capture.small, &capture.large] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
        }
        assert_eq!(capture.small.width, 80);
        assert_eq!(capture.small.height, 24);
        assert_eq!(capture.large.width, 120);
        assert_eq!(capture.large.height, 40);
        if !capture.op_reachable {
            assert!(!capture.picker_open);
            assert!(capture.wheel_at.is_none());
            continue;
        }
        assert!(capture.picker_open, "Esc must unwind one step, not close");
        assert!(capture.wheel_at.is_some(), "picker wheel must resolve");
    }
    assert!(captures[1].op_reachable, "picker must resolve at 120x40");
}

#[test]
fn ja038_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja038_op_item_picker(), ja038_op_item_picker());
}
