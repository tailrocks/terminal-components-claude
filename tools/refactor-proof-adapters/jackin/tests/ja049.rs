//! JA-049: capsule pane geometry.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA049_FOCUS, JA049_ID, MOTION_SEED, ja049_pane_geometry};

#[test]
fn ja049_covers_focus_resize_zoom_and_drags() {
    let captures = ja049_pane_geometry();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA049_FOCUS.len(), 4);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.focus.len(), 4);
        assert_eq!(capture.focus_panes.len(), 4);
        assert_eq!(capture.click_focus.len(), 3);
        let mut frames = vec![
            &capture.resized,
            &capture.zoomed,
            &capture.unzoomed,
            &capture.dragged_v,
            &capture.dragged_h,
            &capture.focus_left,
        ];
        frames.extend(capture.focus.iter());
        frames.extend(capture.click_focus.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA049_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule");
        }
        // Focus chords leave the leftmost pane selected; clicks move focus.
        assert_eq!(capture.focus_panes, vec![1, 1, 1, 1]);
        for at in &capture.click_at {
            assert!(at.is_some(), "pane click must resolve");
        }
        assert_eq!(capture.click_panes[0], 1);
        assert_eq!(capture.click_panes[1], 3);
        assert_eq!(capture.click_panes[2], 1);
        assert!(
            capture.zoomed.text.contains("zoom"),
            "{}",
            capture.zoomed.text
        );
        assert!(
            capture.seam_v.is_some(),
            "vertical seam must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert!(
            capture.seam_h.is_some(),
            "horizontal seam must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
    }
}

#[test]
fn ja049_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja049_pane_geometry(), ja049_pane_geometry());
}
