//! JA-059: intro ritual.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA059_ID, JA059_SIZES, MOTION_SEED, ja059_intro_ritual};

#[test]
fn ja059_intro_skip_determinism_quit_idle() {
    let captures = ja059_intro_ritual();
    assert_eq!(captures.len(), JA059_SIZES.len());
    for capture in &captures {
        assert!(capture.enter_skips);
        assert!(capture.deterministic);
        assert!(capture.quit_on_q);
        assert!(capture.idle_advances);
        assert!(capture.question_ignored);
        assert_eq!(capture.frames.len(), 5);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA059_ID), "{}", frame.identity);
        }
    }
}
