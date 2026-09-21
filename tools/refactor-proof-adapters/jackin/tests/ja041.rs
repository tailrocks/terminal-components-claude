//! JA-041: cockpit overlays.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA041_ID, JA041_SIZES, MOTION_SEED, ja041_cockpit_overlays};

#[test]
fn ja041_covers_log_info_and_credentials() {
    let captures = ja041_cockpit_overlays();
    assert_eq!(captures.len(), JA041_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA041_SIZES) {
        for frame in [
            &capture.running,
            &capture.log_scrolled,
            &capture.log_closed,
            &capture.log_clicked,
            &capture.wheel_down,
            &capture.wheel_up,
            &capture.info,
            &capture.info_closed,
            &capture.credentials,
            &capture.closed,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA041_ID), "{}", frame.identity);
        }
        for frame in [
            &capture.running,
            &capture.log_scrolled,
            &capture.log_closed,
            &capture.log_clicked,
            &capture.wheel_down,
            &capture.wheel_up,
            &capture.info,
        ] {
            assert_eq!(frame.route, "cockpit", "{}", frame.identity);
        }
        assert!(
            capture.log_scrolled.text.contains("Docker build"),
            "{}",
            capture.log_scrolled.text
        );
        assert!(
            capture.log_at.is_some(),
            "log click must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        // `i` is inert; `Esc` then leaves the cockpit for the manager.
        assert_eq!(capture.info.digest, capture.running.digest);
        assert_eq!(capture.info_closed.route, "manager");
        // `c` falls through to the global capsule route; `Esc` there returns
        // to the manager while other instances run.
        assert_eq!(capture.credentials.route, "capsule");
        assert_eq!(capture.closed.route, "manager");
    }
}

#[test]
fn ja041_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja041_cockpit_overlays(), ja041_cockpit_overlays());
}
