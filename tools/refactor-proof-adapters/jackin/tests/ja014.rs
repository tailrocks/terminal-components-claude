//! JA-014: prelude duplicate/empty name validation.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA014_ID, JA014_SIZES, MOTION_SEED, ja014_prelude_name_validation};

#[test]
fn ja014_covers_duplicate_and_empty_name() {
    let captures = ja014_prelude_name_validation();
    assert_eq!(captures.len(), JA014_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA014_SIZES) {
        for frame in [
            &capture.duplicate.name_step,
            &capture.duplicate.refused,
            &capture.duplicate.cancelled,
            &capture.empty.cleared,
            &capture.empty.submitted,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA014_ID), "{}", frame.identity);
        }
        assert_eq!(capture.duplicate.cancelled.route, "manager");
        assert!(
            capture.duplicate.refused.text.contains("already exists")
                || capture.duplicate.refused.route == "prelude",
            "{}",
            capture.duplicate.refused.text
        );
    }
}

#[test]
fn ja014_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja014_prelude_name_validation(),
        ja014_prelude_name_validation()
    );
}
