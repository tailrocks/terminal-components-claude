//! JA-032: 1Password registration replay.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA032_ID, JA032_SIZES, MOTION_SEED, ja032_one_password_replay};

#[test]
fn ja032_replay_registers_without_rendering_secrets() {
    let captures = ja032_one_password_replay();
    assert_eq!(captures.len(), JA032_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA032_SIZES) {
        for frame in [
            &capture.form,
            &capture.op_accounts,
            &capture.vaults,
            &capture.fields,
            &capture.reference,
            &capture.duplicate,
            &capture.codex_saved,
            &capture.refreshed,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA032_ID), "{}", frame.identity);
            assert_eq!(frame.route, "accounts", "{}", frame.identity);
            assert!(
                !frame.text.contains("valid-ant01") && !frame.text.contains("throttled-thr01"),
                "secret leaked at {}",
                frame.identity
            );
        }
        assert!(
            capture.form.text.contains("New account"),
            "{}",
            capture.form.text
        );
        if !capture.op_reachable {
            // Narrow form: the picker control is never painted, so the flow
            // cannot start; every checkpoint repeats the stalled form.
            for frame in [
                &capture.op_accounts,
                &capture.vaults,
                &capture.fields,
                &capture.reference,
                &capture.duplicate,
                &capture.codex_saved,
                &capture.refreshed,
            ] {
                assert_eq!(frame.digest, capture.form.digest, "{}", frame.identity);
            }
            continue;
        }
        assert!(
            capture.op_accounts.text.contains("chainargos"),
            "{}",
            capture.op_accounts.text
        );
        assert!(
            capture.vaults.text.contains("Engineering"),
            "{}",
            capture.vaults.text
        );
        assert!(
            capture.fields.text.contains("credential"),
            "{}",
            capture.fields.text
        );
        assert!(
            capture
                .reference
                .text
                .contains("Anthropic · Work › credential"),
            "{}",
            capture.reference.text
        );
        assert!(
            capture.duplicate.text.contains("Already registered"),
            "{}",
            capture.duplicate.text
        );
        assert!(capture.duplicate_absent, "duplicate must create no account");
        assert!(
            capture.codex_saved.text.contains("Saved Codex · Team"),
            "{}",
            capture.codex_saved.text
        );
        assert!(
            capture.codex_saved.text.contains("Rate limited"),
            "{}",
            capture.codex_saved.text
        );
        assert!(
            capture
                .refreshed
                .text
                .contains("Refreshed Codex · Team · still rate limited"),
            "{}",
            capture.refreshed.text
        );
    }
    assert!(captures[1].op_reachable, "picker must resolve at 120x40");
}

#[test]
fn ja032_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja032_one_password_replay(), ja032_one_password_replay());
}
