//! JA-019: mounts tab variants, add dismiss, isolated fixture.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA019_ID, JA019_SIZES, JA019_VARIANT_NAMES, MOTION_SEED, ja019_mount_actions,
};

#[test]
fn ja019_covers_mount_variants_and_isolated_fixture() {
    let captures = ja019_mount_actions();
    assert_eq!(captures.len(), JA019_SIZES.len());
    assert_eq!(JA019_VARIANT_NAMES.len(), 9);
    for (capture, viewport) in captures.iter().zip(JA019_SIZES) {
        assert_eq!(capture.variants.len(), JA019_VARIANT_NAMES.len());
        let mut frames = vec![&capture.open, &capture.add_dismissed, &capture.isolated];
        frames.extend(capture.variants.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA019_ID), "{}", frame.identity);
        }
        assert_eq!(capture.open.route, "editor");
        assert_eq!(capture.isolated.route, "editor");
        assert!(
            capture.isolated_flag,
            "running_isolated fixture must install"
        );
        assert!(
            !capture.isolated_isolation.is_empty(),
            "isolation must be observable"
        );
    }
}

#[test]
fn ja019_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja019_mount_actions(), ja019_mount_actions());
}
