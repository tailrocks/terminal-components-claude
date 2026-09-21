//! JA-027: settings global attempts and preview cancel.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA027_ID, JA027_SIZES, MOTION_SEED, ja027_settings_global_attempts};

#[test]
fn ja027_records_settings_global_ground_truth() {
    let captures = ja027_settings_global_attempts();
    assert_eq!(captures.len(), JA027_SIZES.len());
    for capture in &captures {
        let mut frames = vec![
            &capture.mount_attempt,
            &capture.env_attempt,
            &capture.preview,
            &capture.cancelled,
        ];
        frames.extend(capture.mount_keys.iter());
        frames.extend(capture.env_keys.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA027_ID), "{}", frame.identity);
        }
        // Tab entries are inert and `Enter` leaves settings for the manager, so
        // the mount keys run from the manager: `s` re-enters settings and `a`
        // falls through to the global Accounts route.
        assert_eq!(capture.mount_attempt.route, "manager");
        assert_eq!(capture.mount_keys[0].route, "settings");
        assert_eq!(capture.mount_keys[1].route, "accounts");
        // Trust toggle → preview → cancel keeps the draft dirty.
        assert_eq!(capture.preview.route, "settings");
        assert!(
            capture.preview.text.contains("Save settings"),
            "{}",
            capture.preview.text
        );
        assert_eq!(capture.cancelled.route, "settings");
        assert!(capture.cancelled_dirty, "cancel must keep the draft");
    }
}

#[test]
fn ja027_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja027_settings_global_attempts(),
        ja027_settings_global_attempts()
    );
}
