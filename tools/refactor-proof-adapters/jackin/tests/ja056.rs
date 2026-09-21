//! JA-056: close attempts.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA056_ID, JA056_SIZES, JA056_VARIANTS, MOTION_SEED, ja056_close_attempts};

#[test]
fn ja056_close_keys_leave_topology_unchanged() {
    let captures = ja056_close_attempts();
    assert_eq!(captures.len(), JA056_SIZES.len());
    assert_eq!(JA056_VARIANTS.len(), 2);
    for capture in &captures {
        assert_eq!(capture.variants.len(), 2);
        assert_eq!(capture.after.len(), 2);
        assert_eq!(capture.solo_variants.len(), 2);
        assert_eq!(capture.solo_after.len(), 2);
        let mut frames = Vec::new();
        frames.extend(capture.variants.iter());
        frames.extend(capture.solo_variants.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA056_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule");
        }
        for topo in &capture.after {
            assert_eq!(*topo, capture.before);
        }
        for topo in &capture.solo_after {
            assert_eq!(*topo, capture.solo_before);
        }
    }
}

#[test]
fn ja056_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja056_close_attempts(), ja056_close_attempts());
}
