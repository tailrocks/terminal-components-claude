//! JA-008: workspace edit/delete/prewarm/GitHub keys.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA008_ID, JA008_SIZES, JA008_WORLDS, MOTION_SEED, ja008_workspace_actions};

#[test]
fn ja008_covers_workspace_action_variants() {
    let captures = ja008_workspace_actions();
    assert_eq!(captures.len(), JA008_SIZES.len() * JA008_WORLDS.len());
    for capture in &captures {
        assert_eq!(capture.variants.len(), 5);
        for variant in &capture.variants {
            assert!(!variant.frames.is_empty(), "{}", variant.name);
            for frame in &variant.frames {
                assert!(frame.is_complete(), "{}", frame.identity);
                assert_eq!(frame.motion_seed, MOTION_SEED);
                assert!(frame.identity.starts_with(JA008_ID), "{}", frame.identity);
                assert!(
                    frame.scenario == "returning" || frame.scenario == "first-use",
                    "{}",
                    frame.scenario
                );
            }
        }
        let edit = capture
            .variants
            .iter()
            .find(|variant| variant.name == "edit")
            .unwrap();
        if capture.world == "returning" {
            assert_eq!(edit.frames[0].route, "editor");
        }
    }
}

#[test]
fn ja008_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja008_workspace_actions(), ja008_workspace_actions());
}
