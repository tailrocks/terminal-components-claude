//! JA-029: trust save retry replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA029_ID, JA029_SIZES, MOTION_SEED, ja029_trust_save_retry};

#[test]
fn ja029_replay_keeps_edits_across_failed_save() {
    let captures = ja029_trust_save_retry();
    assert_eq!(captures.len(), JA029_SIZES.len());
    for capture in &captures {
        for frame in [
            &capture.toggled,
            &capture.failed,
            &capture.retained,
            &capture.saved,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA029_ID), "{}", frame.identity);
        }
        assert!(
            capture.toggled.text.contains("• 1 change"),
            "{}",
            capture.toggled.text
        );
        assert!(
            capture.failed.text.contains("Settings error"),
            "{}",
            capture.failed.text
        );
        assert_eq!(capture.retained.route, "settings");
        assert!(
            capture.retained.text.contains("• 1 change"),
            "{}",
            capture.retained.text
        );
        assert_eq!(capture.saved.route, "manager");
        assert!(!capture.trust0, "persisted trust row must flip");
    }
}

#[test]
fn ja029_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja029_trust_save_retry(), ja029_trust_save_retry());
}
