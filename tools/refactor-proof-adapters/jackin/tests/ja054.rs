//! JA-054: palette replay and extensions.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA054_ID, JA054_SIZES, MOTION_SEED, ja054_palette};

#[test]
fn ja054_replay_scrolls_and_keeps_selection() {
    let captures = ja054_palette();
    assert_eq!(captures.len(), JA054_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA054_SIZES) {
        for frame in [
            &capture.palette,
            &capture.wheel_down,
            &capture.wheel_up,
            &capture.selected,
            &capture.no_match,
            &capture.rename_query,
            &capture.rename_prompt,
            &capture.renamed,
            &capture.closed,
            &capture.space_chord,
            &capture.colon_chord,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA054_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        assert!(
            capture.palette.text.contains("Command palette"),
            "{}",
            capture.palette.text
        );
        assert!(
            capture.new_tab_at.is_some(),
            "New tab must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert_ne!(
            capture.wheel_down.digest, capture.palette.digest,
            "wheel must move rows"
        );
        assert_eq!(
            capture.wheel_up.digest, capture.palette.digest,
            "wheel up must restore rows"
        );
        assert!(
            capture.selected.text.contains("New tab"),
            "{}",
            capture.selected.text
        );
    }
}

#[test]
fn ja054_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja054_palette(), ja054_palette());
}
