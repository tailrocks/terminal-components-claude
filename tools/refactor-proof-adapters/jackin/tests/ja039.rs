//! JA-039: usage handoff replay and suffix.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA039_ID, MOTION_SEED, ja039_usage_handoff};

#[test]
fn ja039_replay_hands_off_and_suffix_scrolls() {
    let captures = ja039_usage_handoff();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.usage,
            &capture.limits,
            &capture.handoff,
            &capture.returned,
            &capture.enter_departure,
            &capture.suffix,
            &capture.clicked,
            &capture.list_wheel,
            &capture.detail_wheel,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA039_ID), "{}", frame.identity);
        }
        assert_eq!(capture.usage.route, "usage");
        assert!(
            capture.usage.text.contains("Usage · read-only"),
            "{}",
            capture.usage.text
        );
        assert!(
            capture.limits.text.contains("Limits"),
            "{}",
            capture.limits.text
        );
        assert_eq!(capture.handoff.route, "accounts");
        assert_eq!(capture.returned.route, "manager");
        assert_eq!(capture.enter_departure.route, "manager");
        assert_eq!(capture.suffix.route, "usage");
        assert!(
            capture.click_at.is_some(),
            "account click must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert!(capture.wheel_at.0.is_some(), "list wheel must resolve");
        assert!(capture.wheel_at.1.is_some(), "detail wheel must resolve");
    }
}

#[test]
fn ja039_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja039_usage_handoff(), ja039_usage_handoff());
}
