//! JA-022: env variants, key validation, folds.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA022_ID, JA022_SIZES, JA022_VARIANT_NAMES, MOTION_SEED, ja022_env_actions};

#[test]
fn ja022_covers_env_variants_validation_and_folds() {
    let captures = ja022_env_actions();
    assert_eq!(captures.len(), JA022_SIZES.len());
    assert_eq!(JA022_VARIANT_NAMES.len(), 5);
    for (capture, viewport) in captures.iter().zip(JA022_SIZES) {
        assert_eq!(capture.variants.len(), JA022_VARIANT_NAMES.len());
        assert_eq!(capture.folds.len(), 3);
        let mut frames = vec![&capture.open, &capture.invalid, &capture.duplicate];
        frames.extend(capture.variants.iter());
        frames.extend(capture.folds.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA022_ID), "{}", frame.identity);
        }
        assert_eq!(capture.open.route, "editor");
        assert!(
            capture
                .invalid
                .text
                .contains("letters, digits and underscores"),
            "invalid key must be rejected:\n{}",
            capture.invalid.text
        );
    }
}

#[test]
fn ja022_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja022_env_actions(), ja022_env_actions());
}
