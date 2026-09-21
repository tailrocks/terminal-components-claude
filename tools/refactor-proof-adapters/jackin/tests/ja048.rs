//! JA-048: capsule topology growth.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA048_ID, JA048_VARIANTS, MOTION_SEED, ja048_topology_growth};

#[test]
fn ja048_creates_cancels_and_spawns_shell() {
    let captures = ja048_topology_growth();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA048_VARIANTS.len(), 3);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.pickers.len(), 3);
        assert_eq!(capture.created.len(), 3);
        assert_eq!(capture.after.len(), 3);
        assert_eq!(capture.cancelled.len(), 3);
        let mut frames = vec![&capture.shell.0, &capture.shell.1];
        frames.extend(capture.pickers.iter());
        frames.extend(capture.created.iter());
        frames.extend(capture.cancelled.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA048_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule");
        }
        assert_eq!(capture.before, (3, 5));
        // New tab adds a tab; splits add panes; shell spawn grows a tab.
        assert_eq!(capture.after[0], (4, 6));
        assert_eq!(capture.after[1], (3, 6));
        assert_eq!(capture.after[2], (3, 6));
        assert_eq!(capture.shell_after, (4, 6));
    }
}

#[test]
fn ja048_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja048_topology_growth(), ja048_topology_growth());
}
