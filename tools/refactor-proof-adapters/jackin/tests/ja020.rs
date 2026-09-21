//! JA-020: roles toggle, search, load picker.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA020_ID, JA020_SIZES, MOTION_SEED, ja020_role_actions};

#[test]
fn ja020_covers_role_toggle_search_and_load() {
    let captures = ja020_role_actions();
    assert_eq!(captures.len(), JA020_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA020_SIZES) {
        for frame in [
            &capture.open,
            &capture.toggled,
            &capture.searched,
            &capture.picker,
            &capture.loaded,
            &capture.escaped,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA020_ID), "{}", frame.identity);
        }
        assert_eq!(capture.open.route, "editor");
        assert_eq!(capture.open.scenario, "returning");
        assert!(
            capture.searched.text.contains("architect"),
            "search query must be visible:\n{}",
            capture.searched.text
        );
    }
}

#[test]
fn ja020_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja020_role_actions(), ja020_role_actions());
}
