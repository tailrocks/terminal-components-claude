//! JA-017: editor name editing keys.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA017_ID, JA017_SIZES, MOTION_SEED, ja017_editor_name_edit};

#[test]
fn ja017_covers_name_edit_keys() {
    let captures = ja017_editor_name_edit();
    assert_eq!(captures.len(), JA017_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA017_SIZES) {
        for frame in [
            &capture.editor,
            &capture.after_enter,
            &capture.after_type,
            &capture.after_edit_keys,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA017_ID), "{}", frame.identity);
        }
        assert_eq!(capture.editor.route, "editor");
    }
}

#[test]
fn ja017_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja017_editor_name_edit(), ja017_editor_name_edit());
}
