//! JA-010: hard-cases refresh and tree scroll.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA010_ID, MOTION_SEED, ja010_hard_cases_refresh_scroll};

#[test]
fn ja010_covers_refresh_and_scroll_all_sizes() {
    let captures = ja010_hard_cases_refresh_scroll();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.initial,
            &capture.after_f5,
            &capture.after_ticks,
            &capture.after_end,
            &capture.after_page_up,
            &capture.after_home,
            &capture.wheel_down,
            &capture.wheel_up,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "hard-cases");
            assert_eq!(frame.motion, "reduced");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA010_ID), "{}", frame.identity);
        }
        assert!(
            capture.initial.text.contains("running")
                || capture.initial.text.contains("workspace")
                || capture.initial.text.contains("jackin"),
            "{}",
            capture.initial.text
        );
    }
}

#[test]
fn ja010_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja010_hard_cases_refresh_scroll(),
        ja010_hard_cases_refresh_scroll()
    );
}
