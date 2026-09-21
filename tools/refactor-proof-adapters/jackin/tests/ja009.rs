//! JA-009: instance session/inspect/stop/purge keys.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA009_ID, JA009_SIZES, MOTION_SEED, ja009_instance_actions};

#[test]
fn ja009_covers_instance_action_variants() {
    let captures = ja009_instance_actions();
    assert_eq!(captures.len(), JA009_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA009_SIZES) {
        assert_eq!(capture.variants.len(), 9);
        for variant in &capture.variants {
            assert!(!variant.frames.is_empty(), "{}", variant.name);
            for frame in &variant.frames {
                assert!(frame.is_complete(), "{}", frame.identity);
                assert_eq!(frame.width, viewport.width);
                assert_eq!(frame.height, viewport.height);
                assert_eq!(frame.scenario, "returning");
                assert_eq!(frame.motion_seed, MOTION_SEED);
                assert!(frame.identity.starts_with(JA009_ID), "{}", frame.identity);
            }
        }
    }
}

#[test]
fn ja009_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja009_instance_actions(), ja009_instance_actions());
}
