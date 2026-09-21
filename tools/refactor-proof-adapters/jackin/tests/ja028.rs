//! JA-028: settings trust walk.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA028_ID, JA028_SIZES, MOTION_SEED, ja028_settings_trust_walk};

#[test]
fn ja028_covers_trust_toggle_preview_and_cancel() {
    let captures = ja028_settings_trust_walk();
    assert_eq!(captures.len(), JA028_SIZES.len());
    for capture in &captures {
        for frame in [
            &capture.agents_attempt,
            &capture.agents_keys,
            &capture.trust_attempt,
            &capture.toggled,
            &capture.trust_walk,
            &capture.preview,
            &capture.cancelled,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA028_ID), "{}", frame.identity);
        }
        // The verbatim `4,Enter` walk departs settings immediately: there is no
        // Agents tab and `Enter` with default focus returns to the manager.
        // With `5` focusing the trust row, `Enter` activates the row instead.
        assert_eq!(capture.agents_attempt.route, "manager");
        assert_eq!(capture.trust_attempt.route, "settings");
        // The production trust path toggles, previews, and cancels in place.
        assert_eq!(capture.toggled.route, "settings");
        assert!(capture.toggled_dirty, "Space must dirty the trust draft");
        assert!(
            capture.toggled.text.contains("• 1 change"),
            "{}",
            capture.toggled.text
        );
        assert_eq!(capture.trust_walk.route, "settings");
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
fn ja028_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja028_settings_trust_walk(), ja028_settings_trust_walk());
}
