//! JA-007: returning manager row activation.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA007_ID, JA007_LABELS, JA007_SIZES, MOTION_SEED, ja007_returning_row_activation,
};

#[test]
fn ja007_covers_activation_variants() {
    let captures = ja007_returning_row_activation();
    assert_eq!(captures.len(), JA007_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA007_SIZES) {
        assert_eq!(capture.variants.len(), JA007_LABELS.len());
        for (variant, label) in capture.variants.iter().zip(JA007_LABELS) {
            assert_eq!(variant.label, label);
            for frame in [&variant.selected, &variant.activated] {
                assert!(frame.is_complete(), "{}", frame.identity);
                assert_eq!(frame.width, viewport.width);
                assert_eq!(frame.height, viewport.height);
                assert_eq!(frame.scenario, "returning");
                assert_eq!(frame.motion, "full");
                assert_eq!(frame.motion_seed, MOTION_SEED);
                assert!(frame.identity.starts_with(JA007_ID), "{}", frame.identity);
            }
            assert!(
                !variant.activated.selected_row.is_empty(),
                "{label} lost selection"
            );
        }
    }
}

#[test]
fn ja007_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja007_returning_row_activation(),
        ja007_returning_row_activation()
    );
}
