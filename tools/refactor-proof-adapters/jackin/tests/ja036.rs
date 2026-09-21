//! JA-036: hard-cases refresh replay and variants.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA036_ID, JA036_VARIANT_DOWNS, MOTION_SEED, ja036_hard_cases_refresh,
};

#[test]
fn ja036_refresh_stays_honest_everywhere() {
    let captures = ja036_hard_cases_refresh();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for capture in &captures {
        assert_eq!(capture.variants.len(), JA036_VARIANT_DOWNS.len());
        let mut frames = vec![&capture.help, &capture.refresh, &capture.usage_help];
        frames.extend(capture.variants.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA036_ID), "{}", frame.identity);
        }
        assert_eq!(capture.help.route, "accounts");
        assert!(
            capture.help.text.contains("Credential sources"),
            "{}",
            capture.help.text
        );
        assert!(
            capture.refresh.text.contains("broker unreachable"),
            "{}",
            capture.refresh.text
        );
        assert_eq!(capture.usage_help.route, "usage");
        assert!(
            capture.usage_help.text.contains("Reading meters"),
            "{}",
            capture.usage_help.text
        );
        for frame in &capture.variants {
            assert_eq!(frame.route, "accounts");
            assert!(frame.text.contains("unreachable"), "{}", frame.text);
        }
    }
}

#[test]
fn ja036_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja036_hard_cases_refresh(), ja036_hard_cases_refresh());
}
