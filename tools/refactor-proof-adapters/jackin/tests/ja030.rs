//! JA-030: accounts navigation and filter attempts.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA030_ID, JA030_PREFIX, JA030_PREFIX_NO_ESC, MOTION_SEED,
    ja030_accounts_navigation,
};

#[test]
fn ja030_covers_navigation_and_filter_ground_truth() {
    let captures = ja030_accounts_navigation();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.prefix.len(), JA030_PREFIX.len());
        assert_eq!(capture.navigation.len(), JA030_PREFIX_NO_ESC.len());
        let mut frames = vec![
            &capture.filter_base,
            &capture.after_slash,
            &capture.after_type,
            &capture.after_esc,
            &capture.no_match_base,
            &capture.no_match,
            &capture.closed,
        ];
        frames.extend(capture.prefix.iter());
        frames.extend(capture.navigation.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA030_ID), "{}", frame.identity);
        }
        // Verbatim prefix: accounts until `Esc` departs to the manager.
        for frame in capture.prefix.iter().take(4) {
            assert_eq!(frame.route, "accounts");
        }
        assert_eq!(capture.prefix[4].route, "manager");
        assert!(
            capture.prefix[0].text.contains("Overview"),
            "{}",
            capture.prefix[0].text
        );
        // Esc-free navigation stays in accounts throughout.
        for frame in &capture.navigation {
            assert_eq!(frame.route, "accounts");
        }
        // `/` is inert: identical digest to the base frame. Typing `Work`
        // stays in accounts (`r` refreshes); the inert no-match query changes
        // nothing, proving no filter field consumes text.
        assert_eq!(capture.after_slash.digest, capture.filter_base.digest);
        assert_eq!(capture.after_slash.text, capture.filter_base.text);
        assert_eq!(capture.after_type.route, "accounts");
        assert!(
            capture.after_type.text.contains("Work"),
            "{}",
            capture.after_type.text
        );
        assert_eq!(capture.after_esc.route, "manager");
        assert_eq!(capture.no_match.route, "accounts");
        assert_eq!(capture.no_match.digest, capture.no_match_base.digest);
        assert_eq!(capture.no_match.text, capture.no_match_base.text);
        assert_eq!(capture.closed.route, "manager");
    }
}

#[test]
fn ja030_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja030_accounts_navigation(), ja030_accounts_navigation());
}
