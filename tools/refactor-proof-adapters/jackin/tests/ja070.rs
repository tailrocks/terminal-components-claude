//! JA-070: arbiter.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA070_ID, JA070_SIZES, MOTION_SEED, ja070_arbiter};

#[test]
fn ja070_decisions_and_entry_exit_outcomes() {
    let captures = ja070_arbiter();
    assert_eq!(captures.len(), JA070_SIZES.len());
    for capture in &captures {
        assert_eq!(
            capture.decisions,
            [
                ("entry-play-intro".to_owned(), "PlayIntro".to_owned()),
                ("entry-play-repeat".to_owned(), "PlayIntro".to_owned()),
                (
                    "entry-join-active".to_owned(),
                    "JoinActive { running: 2 }".to_owned()
                ),
                ("entry-duplicate".to_owned(), "Duplicate".to_owned()),
                ("entry-after-release".to_owned(), "PlayIntro".to_owned()),
                (
                    "exit-still-inside".to_owned(),
                    "StillInside { remaining: 1 }".to_owned()
                ),
                (
                    "exit-outro".to_owned(),
                    "Outro { elapsed_secs: Some(8041) }".to_owned()
                ),
                ("exit-already-ended".to_owned(), "AlreadyEnded".to_owned()),
            ]
        );
        assert_eq!(capture.completed_at_ms, Some(1_234));
        assert_eq!(
            capture.entry_routes,
            ("intro".to_owned(), "manager".to_owned())
        );
        assert!(capture.exit_token_once);
        assert!(capture.exit_still_inside);
        assert_eq!(capture.frames.len(), 5);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA070_ID), "{}", frame.identity);
        }
    }
}
