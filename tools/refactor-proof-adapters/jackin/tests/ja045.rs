//! JA-045: effective accounts replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA045_ID, JA045_SIZES, MOTION_SEED, ja045_effective_accounts};

#[test]
fn ja045_replay_resolves_both_accounts() {
    let captures = ja045_effective_accounts();
    assert_eq!(captures.len(), JA045_SIZES.len());
    for capture in &captures {
        for frame in [&capture.credentials, &capture.arrived] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA045_ID), "{}", frame.identity);
        }
        assert!(
            capture.found_line,
            "credential line must list both accounts"
        );
        assert_eq!(capture.arrived.route, "capsule");
        assert!(
            capture.instance_accounts >= 2,
            "accounts={}",
            capture.instance_accounts
        );
    }
}

#[test]
fn ja045_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja045_effective_accounts(), ja045_effective_accounts());
}
