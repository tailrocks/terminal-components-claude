//! JA-063: help overlays.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA063_ID, JA063_SIZES, MOTION_SEED, ja063_help_overlays};

#[test]
fn ja063_manager_capsule_and_intro_help() {
    let captures = ja063_help_overlays();
    assert_eq!(captures.len(), JA063_SIZES.len());
    for capture in &captures {
        assert!(capture.manager_overlay);
        assert!(capture.manager_restored);
        assert!(capture.capsule_question_typed);
        assert!(capture.capsule_overlay);
        assert!(capture.capsule_restored);
        assert!(capture.palette_shortcuts_inert);
        assert!(capture.intro_ignored);
        assert_eq!(capture.frames.len(), 8);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA063_ID), "{}", frame.identity);
        }
    }
}
