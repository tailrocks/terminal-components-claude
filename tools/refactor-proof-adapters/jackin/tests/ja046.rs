//! JA-046: capsule tab switching.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA046_DIGITS, JA046_ID, JA046_TABS, MOTION_SEED, ja046_tab_switch,
};

#[test]
fn ja046_covers_prefix_and_click_tab_switch() {
    let captures = ja046_tab_switch();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA046_DIGITS.len(), 4);
    assert_eq!(JA046_TABS.len(), 3);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.digits.len(), JA046_DIGITS.len());
        assert_eq!(capture.clicks.len(), JA046_TABS.len());
        let mut frames = vec![&capture.initial, &capture.next, &capture.prev];
        frames.extend(capture.digits.iter());
        frames.extend(capture.clicks.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA046_ID), "{}", frame.identity);
            assert_eq!(frame.route, "capsule");
        }
        // Prefix chords are unbound: the active tab never moves.
        assert_eq!(capture.next_tab, 0);
        assert_eq!(capture.prev_tab, 0);
        assert_eq!(capture.digit_tabs, vec![0, 0, 0, 0]);
        for (tab, at) in JA046_TABS.iter().zip(capture.click_at.iter()) {
            assert!(
                at.is_some(),
                "tab {tab} must resolve at {}x{}",
                viewport.width,
                viewport.height
            );
        }
        assert_eq!(capture.click_tabs, vec![0, 1, 2]);
    }
}

#[test]
fn ja046_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja046_tab_switch(), ja046_tab_switch());
}
