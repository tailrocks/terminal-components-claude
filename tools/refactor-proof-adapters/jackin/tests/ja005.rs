//! JA-005: returning manager navigation keys.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA005_ID, JA005_PREFIX, JA005_VARIANTS, MOTION_SEED, ja005_returning_manager_keys,
};

#[test]
fn ja005_covers_prefix_and_variant_keys_all_sizes() {
    let captures = ja005_returning_manager_keys();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.prefix.len(), JA005_PREFIX.len());
        assert_eq!(capture.variants.len(), JA005_VARIANTS.len());
        for frame in capture.prefix.iter().chain(capture.variants.iter()) {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion, "full");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA005_ID), "{}", frame.identity);
            assert!(!frame.selected_row.is_empty(), "{}", frame.identity);
        }
        // Production binds End to new-workspace, so that prefix key may leave Manager.
        for frame in capture.prefix.iter().take(7).chain(capture.variants.iter()) {
            assert_eq!(frame.route, "manager", "{}", frame.identity);
        }
        let after_home = &capture.prefix[0];
        assert!(
            after_home.selected_row.starts_with("workspace:")
                || after_home.selected_row == "current-directory",
            "{}",
            after_home.selected_row
        );
        let after_right = &capture.prefix[1];
        assert!(
            after_right.text.contains("7f3a")
                || after_right.text.contains("running")
                || after_right.text.contains("workspace"),
            "{}",
            after_right.text
        );
    }
}

#[test]
fn ja005_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja005_returning_manager_keys(),
        ja005_returning_manager_keys()
    );
}
