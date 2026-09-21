//! JA-012: prelude create then pending editor.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA012_ID, MOTION_SEED, ja012_prelude_pending_editor};

#[test]
fn ja012_covers_prelude_steps_all_sizes() {
    let captures = ja012_prelude_pending_editor();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.after_n,
            &capture.after_end,
            &capture.after_enter,
            &capture.after_space,
            &capture.after_continue_1,
            &capture.after_continue_2,
            &capture.after_continue_3,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion, "reduced");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA012_ID), "{}", frame.identity);
        }
        assert_eq!(capture.after_enter.route, "prelude");
        assert!(
            capture.after_continue_3.route == "editor"
                || capture.after_continue_3.route == "prelude",
            "{}",
            capture.after_continue_3.route
        );
    }
}

#[test]
fn ja012_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja012_prelude_pending_editor(),
        ja012_prelude_pending_editor()
    );
}
