//! JA-021: env masking replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA021_FIXTURE_SECRET, JA021_ID, MOTION_SEED, ja021_env_masked_replay,
};

#[test]
fn ja021_replay_masks_plain_values_and_commits_on_save() {
    let captures = ja021_env_masked_replay();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.masked,
            &capture.after_reveal,
            &capture.typing,
            &capture.staged,
            &capture.saved,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA021_ID), "{}", frame.identity);
            assert!(
                !frame.text.contains(JA021_FIXTURE_SECRET),
                "secret leaked at {}:\n{}",
                frame.identity,
                frame.text
            );
            assert!(
                !frame.text.contains("pw-fixture-only"),
                "plain value leaked at {}",
                frame.identity
            );
        }
        assert!(
            capture.masked.text.contains("DATABASE_URL"),
            "{}",
            capture.masked.text
        );
        assert!(
            capture
                .after_reveal
                .text
                .contains("plain values stay masked"),
            "{}",
            capture.after_reveal.text
        );
        assert!(
            capture.staged.text.contains("NEW_SECRET"),
            "{}",
            capture.staged.text
        );
        assert!(
            capture.staged.text.contains("************1234"),
            "{}",
            capture.staged.text
        );
        assert!(
            capture.staged.text.contains("• 1 change"),
            "{}",
            capture.staged.text
        );
        assert_eq!(capture.saved.route, "manager");
        assert!(capture.committed, "saved draft must commit NEW_SECRET");
    }
}

#[test]
fn ja021_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja021_env_masked_replay(), ja021_env_masked_replay());
}
