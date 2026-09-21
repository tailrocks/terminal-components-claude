//! JA-047: capsule prefix keys.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA047_ID, JA047_SIZES, JA047_TIMEOUT_TICKS, MOTION_SEED, ja047_prefix_keys};

fn row0(frame: &jackin_adapter::ObservedFrame) -> String {
    frame.text.lines().next().unwrap_or("").to_owned()
}

#[test]
fn ja047_covers_prefix_capture_and_followups() {
    assert_eq!(JA047_TIMEOUT_TICKS, 120);
    let captures = ja047_prefix_keys();
    assert_eq!(captures.len(), JA047_SIZES.len());
    for capture in &captures {
        for frame in [
            &capture.prefix,
            &capture.persisted,
            &capture.cancelled,
            &capture.literal,
            &capture.cleared,
            &capture.unknown,
            &capture.redrawn,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA047_ID), "{}", frame.identity);
        }
        for frame in [
            &capture.prefix,
            &capture.persisted,
            &capture.literal,
            &capture.cleared,
            &capture.unknown,
            &capture.redrawn,
        ] {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        assert!(
            row0(&capture.prefix).contains("prefix"),
            "{}",
            row0(&capture.prefix)
        );
        assert!(
            row0(&capture.persisted).contains("prefix"),
            "no timeout within T(120)"
        );
        // `Esc` clears the prefix and, with other instances running, departs
        // for the manager rather than staying in the capsule.
        assert_eq!(capture.cancelled.route, "manager");
        assert!(!row0(&capture.cancelled).contains("prefix"));
    }
}

#[test]
fn ja047_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja047_prefix_keys(), ja047_prefix_keys());
}
