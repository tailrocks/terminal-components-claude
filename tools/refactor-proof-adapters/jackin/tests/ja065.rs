//! JA-065: hover and focus.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA065_ID, JA065_SIZES, JA065_TREE_HOVER, MOTION_SEED, ja065_hover_and_focus};

#[test]
fn ja065_hover_focus_and_click() {
    let captures = ja065_hover_and_focus();
    assert_eq!(captures.len(), JA065_SIZES.len());
    for capture in &captures {
        assert!(capture.hover_starts_none);
        assert_eq!(
            capture.hover_tree.as_deref(),
            Some(JA065_TREE_HOVER),
            "{:?}",
            capture.hover_tree
        );
        assert_eq!(capture.hover_corner, None);
        assert!(capture.keyboard_clears_hover);
        assert!(capture.selection_moves);
        assert!(capture.click_focuses);
        assert_eq!(capture.frames.len(), 6);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA065_ID), "{}", frame.identity);
        }
    }
}
