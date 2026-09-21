//! JA-057: takeover substrate.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA057_ID, JA057_SIZES, MOTION_SEED, ja057_takeover_substrate};

#[test]
fn ja057_tab_focus_substrate_and_launch_leg() {
    let captures = ja057_takeover_substrate();
    assert_eq!(captures.len(), JA057_SIZES.len());
    for capture in &captures {
        assert!(capture.tab_row.contains("1 Mix"), "{}", capture.tab_row);
        assert!(capture.tab_row.contains("2 Shell"), "{}", capture.tab_row);
        // The 80-column tabbar clips the third tab; the wide tabbar shows it.
        if capture
            .frames
            .first()
            .is_some_and(|frame| frame.width >= 120)
        {
            assert!(capture.tab_row.contains("3 docs"), "{}", capture.tab_row);
        }
        assert!(capture.same_tab_followed);
        assert!(capture.other_tab_preserved);
        assert_eq!(capture.before_detach, capture.after_detach);
        assert!(!capture.before_detach.is_empty());
        assert_eq!(capture.launch_route, "capsule");
        assert!(capture.launch_transcript);
        assert_eq!(capture.frames.len(), 7);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA057_ID), "{}", frame.identity);
        }
    }
}
