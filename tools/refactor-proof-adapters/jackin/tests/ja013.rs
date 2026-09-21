//! JA-013: prelude git URL and Esc rewind.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA013_ID, JA013_SIZES, MOTION_SEED, ja013_prelude_git_and_rewind};

#[test]
fn ja013_covers_git_destination_and_rewind() {
    let captures = ja013_prelude_git_and_rewind();
    assert_eq!(captures.len(), JA013_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA013_SIZES) {
        for frame in [
            &capture.after_n,
            &capture.prelude,
            &capture.after_g,
            &capture.after_git_url,
            &capture.after_source,
            &capture.after_destination,
            &capture.after_continue,
            &capture.after_name,
        ]
        .into_iter()
        .chain(capture.rewind.iter())
        {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.scenario, "returning");
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA013_ID), "{}", frame.identity);
        }
        assert_eq!(capture.prelude.route, "prelude");
        assert!(
            capture.rewind.last().is_none()
                || capture.rewind.last().unwrap().route == "manager"
                || capture.rewind.last().unwrap().route == "prelude",
            "{}",
            capture
                .rewind
                .last()
                .map(|f| f.route.as_str())
                .unwrap_or("-")
        );
    }
}

#[test]
fn ja013_repeat_from_fresh_worlds_matches() {
    assert_eq!(
        ja013_prelude_git_and_rewind(),
        ja013_prelude_git_and_rewind()
    );
}
