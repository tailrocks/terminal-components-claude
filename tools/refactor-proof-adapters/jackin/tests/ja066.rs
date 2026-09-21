//! JA-066: too-small state.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA066_ID, JA066_SIZES, MOTION_SEED, ja066_too_small};

#[test]
fn ja066_notice_blocked_keys_quit_and_recover() {
    let captures = ja066_too_small();
    assert_eq!(captures.len(), JA066_SIZES.len());
    for capture in &captures {
        assert!(capture.notice_60x18);
        assert!(capture.notice_71x19);
        assert!(capture.keys_blocked);
        assert!(capture.quit_on_q);
        assert!(capture.ctrl_c_inert);
        assert!(capture.exact_72x20);
        assert!(capture.restored_80x24);
        assert_eq!(capture.frames.len(), 7);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA066_ID), "{}", frame.identity);
        }
    }
}
