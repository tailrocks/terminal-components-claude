//! JA-023: accounts tab replay and no-match filter.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA023_ID, MOTION_SEED, ja023_accounts_tab_replay};

#[test]
fn ja023_replay_covers_inherited_preferred_and_filter() {
    let captures = ja023_accounts_tab_replay();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.open,
            &capture.disabled,
            &capture.reenabled,
            &capture.experiments,
            &capture.saved,
            &capture.reentered,
            &capture.after_slash,
            &capture.after_type,
            &capture.filter_closed,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA023_ID), "{}", frame.identity);
        }
        assert!(
            capture.open.text.contains("Active accounts"),
            "{}",
            capture.open.text
        );
        assert!(
            capture.disabled.text.contains("off for this Workspace"),
            "{}",
            capture.disabled.text
        );
        assert!(
            capture.experiments_at.is_some(),
            "Experiments row must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert!(
            capture
                .experiments
                .text
                .contains("Codex · Experiments · active for this Workspace"),
            "{}",
            capture.experiments.text
        );
        assert_eq!(capture.saved.route, "manager");
        // No `/` filter binding exists in the pinned source: the key is inert
        // and the typed query falls through to global navigation (`u` → Usage).
        assert_eq!(capture.reentered.route, "editor");
        assert_eq!(capture.after_slash.digest, capture.reentered.digest);
        assert_eq!(capture.after_slash.text, capture.reentered.text);
        assert_eq!(capture.after_type.route, "usage");
        assert_eq!(capture.filter_closed.route, "manager");
    }
}

#[test]
fn ja023_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja023_accounts_tab_replay(), ja023_accounts_tab_replay());
}
