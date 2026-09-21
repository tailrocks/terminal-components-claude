//! JA-031: accounts pointer paths.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA031_ID, MOTION_SEED, ja031_accounts_pointer};

#[test]
fn ja031_covers_row_click_hover_release_drag_and_wheel() {
    let captures = ja031_accounts_pointer();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.initial,
            &capture.clicked,
            &capture.pressed,
            &capture.released,
            &capture.outside,
            &capture.dragged,
            &capture.wheel_down,
            &capture.wheel_up,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA031_ID), "{}", frame.identity);
            assert_eq!(frame.route, "accounts");
        }
        assert!(
            capture.work_at.is_some(),
            "Work row must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert_ne!(capture.clicked_cursor, capture.initial_cursor);
        assert!(
            capture.hover.is_some(),
            "row hover must resolve an owner id"
        );
        // Pointer down selects the row immediately; releasing outside leaves
        // that selection in place without further activation.
        assert_eq!(
            capture.outside_cursor, capture.clicked_cursor,
            "before={}",
            capture.outside_before
        );
        assert_ne!(capture.outside_cursor, capture.outside_before);
    }
}

#[test]
fn ja031_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja031_accounts_pointer(), ja031_accounts_pointer());
}
