//! JA-051: capsule scrollback and copy.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA051_ID, MOTION_SEED, ja051_scrollback_copy};

#[test]
fn ja051_scrolls_selects_and_copies() {
    let captures = ja051_scrollback_copy();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.scrolled_up,
            &capture.active_wheel,
            &capture.inactive_wheel,
            &capture.scrolled_down,
            &capture.dragged,
            &capture.double_clicked,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA051_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule");
        }
        assert!(
            capture.active_at.is_some(),
            "active wheel must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert!(
            capture.inactive_at.is_some(),
            "inactive wheel must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        let needle = capture.needle.as_deref().expect("a copy needle must paint");
        assert_eq!(capture.drag_clipboard, needle);
        assert_eq!(capture.word_clipboard, needle);
    }
}

#[test]
fn ja051_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja051_scrollback_copy(), ja051_scrollback_copy());
}
