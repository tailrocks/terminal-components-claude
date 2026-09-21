//! JA-064: overlay dismissal.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA064_ID, JA064_PALETTE_FOCUS, JA064_SIZES, MOTION_SEED, ja064_overlay_dismissal,
};

#[test]
fn ja064_palette_menu_and_bare_esc() {
    let captures = ja064_overlay_dismissal();
    assert_eq!(captures.len(), JA064_SIZES.len());
    for capture in &captures {
        assert!(
            capture
                .palette_focus
                .as_deref()
                .is_some_and(|id| id.contains(JA064_PALETTE_FOCUS)),
            "{:?}",
            capture.palette_focus
        );
        assert!(capture.pane_blocked);
        assert!(capture.palette_esc_inert);
        assert_eq!(capture.palette_enter_route, "usage");
        assert!(capture.palette_enter_closed);
        assert!(capture.menu_esc);
        assert!(capture.menu_survives_palette_esc);
        assert!(capture.bare_esc_noop);
        assert_eq!(capture.frames.len(), 8);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA064_ID), "{}", frame.identity);
        }
    }
}
