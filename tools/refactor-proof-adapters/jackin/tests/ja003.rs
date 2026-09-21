//! JA-003: Full intro skip guard and returning join.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA003_ID, JA003_SIZES, MOTION_SEED, ja003_first_use_full};

#[test]
fn ja003_covers_sizes_and_replay() {
    let captures = ja003_first_use_full();
    assert_eq!(captures.len(), JA003_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA003_SIZES) {
        for frame in [
            &capture.guard.stale_enter,
            &capture.guard.after_ticks,
            &capture.guard.first_skip,
            &capture.guard.second_skip,
            &capture.replay.after_ticks_45,
            &capture.replay.after_ticks_3,
            &capture.replay.first_enter,
            &capture.replay.second_enter,
            &capture.replay.returning,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA003_ID), "{}", frame.identity);
            assert_eq!(frame.color, "truecolor");
        }

        assert_eq!(capture.guard.stale_enter.scenario, "first-use");
        assert_eq!(capture.guard.stale_enter.motion, "full");
        assert_eq!(capture.guard.stale_enter.route, "intro");
        assert_eq!(capture.replay.after_ticks_45.route, "intro");
        assert!(
            capture.replay.after_ticks_45.text.contains("jackin❯"),
            "{}",
            capture.replay.after_ticks_45.text
        );
        assert!(
            capture
                .replay
                .after_ticks_45
                .text
                .contains("Stand up, operator…")
                || capture
                    .replay
                    .after_ticks_45
                    .text
                    .contains("Host stays outside…")
                || capture
                    .replay
                    .after_ticks_45
                    .text
                    .contains("Follow the green.")
                || capture
                    .replay
                    .after_ticks_45
                    .text
                    .contains("Knock, knock, operator"),
            "{}",
            capture.replay.after_ticks_45.text
        );
        assert_eq!(capture.replay.second_enter.route, "manager");
        assert!(
            capture
                .replay
                .second_enter
                .text
                .contains("Current directory")
                || capture.replay.second_enter.text.contains("workspace")
                || capture.replay.second_enter.text.contains("Choose"),
            "{}",
            capture.replay.second_enter.text
        );
        assert_eq!(capture.replay.returning.scenario, "returning");
        assert_eq!(capture.replay.returning.route, "manager");
        assert!(
            capture.replay.returning.text.contains("2 running")
                || capture.replay.returning.text.contains("running"),
            "{}",
            capture.replay.returning.text
        );
        assert_ne!(capture.replay.returning.route, "intro");
    }
}

#[test]
fn ja003_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja003_first_use_full(), ja003_first_use_full());
}
