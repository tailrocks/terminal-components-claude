//! JA-043: launch blocks.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA043_ID, JA043_SIZES, MOTION_SEED, ja043_blocked_run, ja043_launch_blocks, ja043_locked_run,
};

#[test]
fn ja043_locked_run_holds_retries_and_completes() {
    let (trace, done, held) = ja043_locked_run();
    assert!(held, "credentials run must hold before the retry");
    assert!(done, "credentials run must complete after the retry");
    assert!(
        trace.iter().any(|e| e.contains("CredentialError")),
        "{trace:?}"
    );
    assert!(
        trace.iter().any(|e| e.contains("CredentialsResolved")),
        "{trace:?}"
    );
    assert!(
        matches!(trace.last(), Some(e) if e.contains("Ready")),
        "{trace:?}"
    );
}

#[test]
fn ja043_blocked_run_is_terminal_then_cancelled() {
    let (trace, blocked_at, terminal, cancelled) = ja043_blocked_run();
    assert_eq!(blocked_at.as_deref(), Some("Sidecar"));
    assert!(terminal, "blocked run must ignore further ticks");
    assert!(cancelled, "cancel must dismiss the blocked run");
    assert!(trace.iter().any(|e| e.contains("Blocked")), "{trace:?}");
}

#[test]
fn ja043_covers_blocks_and_cockpit_context() {
    let captures = ja043_launch_blocks();
    assert_eq!(captures.len(), JA043_SIZES.len());
    for capture in &captures {
        assert!(capture.locked_held);
        assert!(capture.locked_done);
        assert_eq!(capture.blocked_at.as_deref(), Some("Sidecar"));
        assert!(capture.blocked_terminal);
        assert!(capture.blocked_cancelled);
        assert!(
            capture.cockpit.is_complete(),
            "{}",
            capture.cockpit.identity
        );
        assert_eq!(capture.cockpit.motion_seed, MOTION_SEED);
        assert!(
            capture.cockpit.identity.starts_with(JA043_ID),
            "{}",
            capture.cockpit.identity
        );
        assert_eq!(capture.cockpit.route, "cockpit");
    }
}

#[test]
fn ja043_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja043_launch_blocks(), ja043_launch_blocks());
    assert_eq!(ja043_locked_run(), ja043_locked_run());
    assert_eq!(ja043_blocked_run(), ja043_blocked_run());
}
