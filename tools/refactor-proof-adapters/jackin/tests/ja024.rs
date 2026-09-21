//! JA-024: hundred-role replay and scroll suffix.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA024_ID, MOTION_SEED, ja024_hundred_roles_replay};

#[test]
fn ja024_replay_stays_readable_and_scrolls() {
    let captures = ja024_hundred_roles_replay();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.roles_end,
            &capture.reentered,
            &capture.scrolled,
            &capture.wheel_down,
            &capture.wheel_up,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA024_ID), "{}", frame.identity);
            assert_eq!(frame.route, "editor");
        }
        assert!(
            capture.roles_end.text.contains("+ Load role…"),
            "{}",
            capture.roles_end.text
        );
        assert!(
            capture.reentered.text.contains("SVC_FLAG"),
            "{}",
            capture.reentered.text
        );
        assert!(
            capture.wheel_at.is_some(),
            "config wheel must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
    }
}

#[test]
fn ja024_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja024_hundred_roles_replay(), ja024_hundred_roles_replay());
}
