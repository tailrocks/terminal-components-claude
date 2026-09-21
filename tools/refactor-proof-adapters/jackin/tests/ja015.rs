//! JA-015: prelude browser navigation.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA015_ID, MOTION_SEED, ja015_prelude_browser};

#[test]
fn ja015_covers_browser_path_paste_and_wheel() {
    let captures = ja015_prelude_browser();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.prelude,
            &capture.after_browser_keys,
            &capture.after_missing,
            &capture.after_commit,
            &capture.after_paste,
            &capture.after_wheel,
            &capture.after_esc,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA015_ID), "{}", frame.identity);
        }
        assert_eq!(capture.prelude.route, "prelude");
    }
}

#[test]
fn ja015_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja015_prelude_browser(), ja015_prelude_browser());
}
