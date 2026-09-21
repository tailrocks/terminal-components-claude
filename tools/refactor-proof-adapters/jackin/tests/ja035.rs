//! JA-035: account action variants.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA035_ID, JA035_SIZES, JA035_VARIANT_NAMES, MOTION_SEED, ja035_account_actions,
};

#[test]
fn ja035_covers_account_action_variants() {
    let captures = ja035_account_actions();
    assert_eq!(captures.len(), JA035_SIZES.len());
    assert_eq!(JA035_VARIANT_NAMES.len(), 9);
    for capture in &captures {
        assert_eq!(capture.variants.len(), JA035_VARIANT_NAMES.len());
        for frame in &capture.variants {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA035_ID), "{}", frame.identity);
        }
        // `m` is the global manager key: no mask control exists in the pinned
        // accounts screen, so that variant lands on the manager.
        for (index, frame) in capture.variants.iter().enumerate() {
            if index == 5 {
                assert_eq!(frame.route, "manager", "{}", frame.identity);
            } else {
                assert_eq!(frame.route, "accounts", "{}", frame.identity);
            }
        }
        assert!(
            capture.cancel_retained,
            "cancelled removal must retain Work"
        );
        assert!(
            capture.confirm_retained,
            "no key consumes the removal confirmation in pinned source"
        );
    }
}

#[test]
fn ja035_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja035_account_actions(), ja035_account_actions());
}
