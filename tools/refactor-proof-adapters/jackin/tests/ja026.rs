//! JA-026: settings keys, attempted clicks, ring.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA026_ALIASES, JA026_ID, JA026_TABS, MOTION_SEED, ja026_settings_tabs,
};

#[test]
fn ja026_records_single_page_settings_ground_truth() {
    let captures = ja026_settings_tabs();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA026_ALIASES.len(), 7);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.aliases.len(), JA026_ALIASES.len());
        assert_eq!(capture.clicks.len(), JA026_TABS.len());
        let mut frames = vec![&capture.open, &capture.ring];
        frames.extend(capture.aliases.iter());
        frames.extend(capture.clicks.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA026_ID), "{}", frame.identity);
        }
        assert_eq!(capture.open.route, "settings");
        assert_eq!(capture.ring.route, "manager");
        for frame in capture.aliases.iter().chain(capture.clicks.iter()) {
            assert_eq!(frame.route, "settings");
        }
        // Aliases other than `5` are inert: identical digest, cursor, and text.
        for (alias, frame) in capture.aliases.iter().enumerate().take(4) {
            assert_eq!(frame.digest, capture.open.digest, "alias {alias}");
            assert_eq!(frame.text, capture.open.text, "alias {alias}");
        }
        for (alias, frame) in capture.aliases.iter().enumerate().skip(5) {
            assert_eq!(frame.digest, capture.open.digest, "alias {alias}");
            assert_eq!(frame.text, capture.open.text, "alias {alias}");
        }
        // `5` focuses the trust row.
        let trust = &capture.aliases[4];
        let focus = trust.focus.clone().unwrap_or_default().to_lowercase();
        assert!(focus.contains("trust"), "focus={focus}");
        // No tab label is painted. `Trust` resolves only to the trust row, and
        // clicking it toggles the trust draft; the other labels resolve nowhere.
        for (tab, at) in JA026_TABS.iter().zip(capture.click_at.iter()).take(4) {
            assert!(at.is_none(), "tab {tab} unexpectedly painted");
        }
        assert!(
            capture.click_at[4].is_some(),
            "trust row must resolve for the Trust label"
        );
        for frame in capture.clicks.iter().take(4) {
            assert_eq!(frame.digest, capture.open.digest);
        }
        assert_ne!(capture.clicks[4].digest, capture.open.digest);
    }
}

#[test]
fn ja026_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja026_settings_tabs(), ja026_settings_tabs());
}
