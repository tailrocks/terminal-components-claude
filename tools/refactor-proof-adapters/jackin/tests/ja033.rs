//! JA-033: masked plain-key replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA001_SIZES, JA033_FIXTURE_KEY, JA033_ID, MOTION_SEED, ja033_plain_key_replay,
};

#[test]
fn ja033_replay_masks_key_and_asks_before_remove() {
    let captures = ja033_plain_key_replay();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.source,
            &capture.typing,
            &capture.tail,
            &capture.saved,
            &capture.remove_ask,
            &capture.remove_cancelled,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA033_ID), "{}", frame.identity);
            assert_eq!(frame.route, "accounts", "{}", frame.identity);
            assert!(
                !frame.text.contains(JA033_FIXTURE_KEY),
                "key leaked at {}",
                frame.identity
            );
            assert!(
                !frame.text.contains("abcdefgh"),
                "key material leaked at {}",
                frame.identity
            );
        }
        // The narrow list clips the `API key` label; the selected index is the
        // size-independent proof.
        assert_eq!(capture.source_index, 2);
        if capture.secret_reachable {
            assert!(
                capture.source.text.contains("API key"),
                "{}",
                capture.source.text
            );
        }
        if !capture.secret_reachable {
            // The reachability probe itself cycles focus, so the stalled
            // checkpoints repeat the post-probe frame rather than `source`.
            for frame in [
                &capture.tail,
                &capture.saved,
                &capture.remove_ask,
                &capture.remove_cancelled,
            ] {
                assert_eq!(frame.digest, capture.typing.digest, "{}", frame.identity);
            }
            continue;
        }
        assert!(capture.tail.text.contains("1234"), "{}", capture.tail.text);
        assert!(
            capture.saved.text.contains("Saved Claude · Spare"),
            "{}",
            capture.saved.text
        );
        assert!(
            capture.remove_ask.text.contains("Remove account Spare?"),
            "{}",
            capture.remove_ask.text
        );
        assert!(
            capture.retained,
            "cancelled removal must retain the account"
        );
        assert!(
            !capture.source_debug.contains("abcdefgh"),
            "fingerprint embeds key material: {}",
            capture.source_debug
        );
        assert!(
            capture.source_debug.contains("1234"),
            "{}",
            capture.source_debug
        );
    }
}

#[test]
fn ja033_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja033_plain_key_replay(), ja033_plain_key_replay());
}
