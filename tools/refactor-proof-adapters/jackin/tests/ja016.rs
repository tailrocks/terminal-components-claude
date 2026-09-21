//! JA-016: editor tabs and focus ring.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA016_ID, MOTION_SEED, ja016_editor_tabs};

#[test]
fn ja016_covers_tab_aliases_clicks_and_ring() {
    let captures = ja016_editor_tabs();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.editor,
            &capture.after_aliases,
            &capture.after_clicks,
            &capture.after_ring,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA016_ID), "{}", frame.identity);
        }
        assert_eq!(capture.editor.route, "editor");
    }
}

#[test]
fn ja016_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja016_editor_tabs(), ja016_editor_tabs());
}
