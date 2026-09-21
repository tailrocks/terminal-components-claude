//! JA-040: launch stage walk.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA040_ID, JA040_TICK_BOUND, MOTION_SEED, ja040_launch_stages};

#[test]
fn ja040_walks_all_eleven_stages_to_capsule() {
    assert!(JA040_TICK_BOUND >= 400);
    let captures = ja040_launch_stages();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        let mut frames = vec![&capture.initial, &capture.arrived];
        frames.extend(capture.stage_frames.iter());
        frames.extend(capture.handoff.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA040_ID), "{}", frame.identity);
        }
        assert!(capture.completed, "launch must complete within the bound");
        assert_eq!(capture.initial.route, "cockpit");
        // Eleven stage frontiers plus the skipped AgentBinaries marker.
        assert_eq!(capture.stage_frames.len(), 11, "{:?}", capture.stage_trace);
        assert_eq!(capture.stage_trace.len(), 11);
        assert!(
            capture
                .stage_trace
                .iter()
                .any(|entry| entry.contains("Skipped")),
            "{:?}",
            capture.stage_trace
        );
        let handoff = capture.handoff.as_ref().expect("handoff must be observed");
        assert_eq!(handoff.route, "handoff");
        assert_eq!(capture.arrived.route, "capsule");
    }
}

#[test]
fn ja040_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja040_launch_stages(), ja040_launch_stages());
}
