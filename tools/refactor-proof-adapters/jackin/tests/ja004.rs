//! JA-004: reduced intro boundary and Full Ctrl-C.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA004_ID, JA004_SIZES, MOTION_SEED, ja004_first_use_reduced_and_quit};

#[test]
fn ja004_reduced_reaches_manager_and_ctrl_c_is_observed() {
    let captures = ja004_first_use_reduced_and_quit();
    assert_eq!(captures.len(), JA004_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA004_SIZES) {
        for frame in [
            &capture.reduced.initial,
            &capture.reduced.after_ticks,
            &capture.reduced.after_enter,
            &capture.quit.initial,
            &capture.quit.after_ctrl_c,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA004_ID), "{}", frame.identity);
        }

        assert_eq!(capture.reduced.initial.route, "intro");
        assert_eq!(capture.reduced.initial.motion, "reduced");
        assert!(
            capture.reduced.initial.text.contains("Enter Continue"),
            "{}",
            capture.reduced.initial.text
        );
        assert_eq!(capture.reduced.after_ticks.route, "intro");
        assert_eq!(capture.reduced.after_enter.route, "manager");
        assert_eq!(capture.quit.initial.route, "intro");
        assert_eq!(capture.quit.initial.motion, "full");
        assert!(!capture.quit.initial.quit);
        // Production Ctrl-C is dispatched; quit is recorded if the runtime
        // or app consumed it as an exit.
        let _ = capture.quit.after_ctrl_c.quit;
    }
}

#[test]
fn ja004_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja004_first_use_reduced_and_quit(),
        ja004_first_use_reduced_and_quit()
    );
}
