//! JA-037: 1Password errors and picker walk.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA037_ID, JA037_SIZES, MOTION_SEED, ja037_error_taxonomy, ja037_op_errors};

#[test]
fn ja037_taxonomy_covers_all_three_error_variants() {
    let (errors, unlocked) = ja037_error_taxonomy();
    assert_eq!(errors.len(), 3);
    assert_eq!(errors[0].name, "locked");
    assert!(
        errors[0].message.contains("locked"),
        "{}",
        errors[0].message
    );
    assert!(errors[0].retryable);
    assert_eq!(errors[1].name, "authorization");
    assert!(
        errors[1].message.contains("authorization required"),
        "{}",
        errors[1].message
    );
    assert!(!errors[1].retryable);
    assert_eq!(errors[2].name, "denied");
    assert!(
        errors[2].message.contains("access denied"),
        "{}",
        errors[2].message
    );
    assert!(!errors[2].retryable);
    assert_eq!(unlocked, 3);
}

#[test]
fn ja037_picker_walk_unwinds_and_cancels() {
    let captures = ja037_op_errors();
    assert_eq!(captures.len(), JA037_SIZES.len());
    for capture in &captures {
        assert_eq!(capture.errors.len(), 3);
        assert_eq!(capture.unwind.len(), 3);
        let mut frames = vec![&capture.picker, &capture.cancelled];
        frames.extend(capture.unwind.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA037_ID), "{}", frame.identity);
        }
        if !capture.op_reachable {
            // Narrow form: no picker control, so the walk stalls on the form.
            assert!(!capture.picker_open);
            assert!(capture.picker_closed);
            for frame in &capture.unwind {
                assert_eq!(frame.digest, capture.picker.digest, "{}", frame.identity);
            }
            continue;
        }
        assert_eq!(
            capture.picker.route, "accounts",
            "{}",
            capture.picker.identity
        );
        // The final `Esc` closes the picker and the form, landing on the manager.
        assert_eq!(
            capture.cancelled.route, "manager",
            "{}",
            capture.cancelled.identity
        );
        assert!(capture.picker_open, "op picker must open");
        assert!(capture.picker_closed, "cancel must close the picker");
    }
    assert!(captures[1].op_reachable, "picker must resolve at 120x40");
}

#[test]
fn ja037_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja037_op_errors(), ja037_op_errors());
    assert_eq!(ja037_error_taxonomy(), ja037_error_taxonomy());
}
