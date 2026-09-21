//! JA-050: capsule typing per focus.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA050_ID, JA050_VARIANT_NAMES, MOTION_SEED, ja050_pane_typing};

#[test]
fn ja050_types_edits_pastes_and_submits_per_focus() {
    let captures = ja050_pane_typing();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA050_VARIANT_NAMES.len(), 3);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.variants.len(), 3);
        assert_eq!(capture.typed.len(), 3);
        assert_eq!(capture.panes.len(), 3);
        let mut frames: Vec<&jackin_adapter::ObservedFrame> = capture.variants.iter().collect();
        frames.extend(capture.typed.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA050_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        // Only the pristine initial pane echoes typed input; after any pane
        // click the typed line is swallowed.
        assert_eq!(capture.echoed, vec![true, false, false]);
        assert_ne!(capture.panes[0], capture.panes[1]);
        assert_eq!(capture.panes[0], capture.panes[2]);
    }
}

#[test]
fn ja050_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja050_pane_typing(), ja050_pane_typing());
}
