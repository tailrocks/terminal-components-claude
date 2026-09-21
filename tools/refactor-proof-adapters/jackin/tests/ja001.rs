//! JA-001 first leaf: eight worlds, Paused, frame 0, 80×24 truecolor.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    CaptureColor, DirectSession, EPOCH_SECS, HISTORICAL_PAINT_SIZE, JA001_FIRST_LEAF_COLOR,
    JA001_FIRST_LEAF_HEIGHT, JA001_FIRST_LEAF_SIZE, JA001_FIRST_LEAF_WIDTH, JA001_ID,
    JA001_REMAINING_SIZES, JA001_SIZES, MOTION_SEED, Motion, Scenario, Viewport, expected_route,
    ja001_paused_frame0_remaining_colors, ja001_paused_frame0_remaining_sizes_truecolor,
    ja001_paused_frame0_truecolor, ja001_worlds, motion_name, route_name,
};

#[test]
fn ja001_covers_all_eight_worlds_paused_frame0_80x24_truecolor() {
    let worlds = ja001_worlds();
    assert_eq!(worlds.len(), 8);
    assert_eq!(
        worlds.map(Scenario::name),
        [
            "first-use",
            "returning",
            "accounts-mixed",
            "launch-running",
            "launch-failure",
            "capsule-multi",
            "outro-last",
            "hard-cases",
        ]
    );
    assert_ne!(JA001_FIRST_LEAF_SIZE, HISTORICAL_PAINT_SIZE);

    let captures = ja001_paused_frame0_truecolor();
    assert_eq!(captures.len(), 8);

    for (scenario, capture) in worlds.into_iter().zip(captures) {
        let frame = &capture.initial;
        assert_eq!(frame.scenario, scenario.name());
        assert_eq!(frame.motion, "paused");
        assert_eq!(frame.construct_frame, 0);
        assert_eq!(frame.width, JA001_FIRST_LEAF_WIDTH);
        assert_eq!(frame.height, JA001_FIRST_LEAF_HEIGHT);
        assert_eq!(frame.color, JA001_FIRST_LEAF_COLOR);
        assert_eq!(frame.theme, "junie");
        assert_eq!(frame.route, route_name(expected_route(scenario)));
        assert_eq!(frame.motion_seed, MOTION_SEED);
        assert_eq!(frame.epoch_secs, EPOCH_SECS);
        assert_eq!(frame.now_secs, EPOCH_SECS);
        assert!(!frame.clock_running);
        assert!(frame.is_complete(), "{}", frame.identity);
        assert_eq!(
            frame.cells.len(),
            usize::from(JA001_FIRST_LEAF_WIDTH) * usize::from(JA001_FIRST_LEAF_HEIGHT)
        );
        assert!(
            frame.text.chars().any(|c| c != ' ' && c != '\n'),
            "{} empty production draw",
            frame.identity
        );
        assert!(
            frame.identity.starts_with(&format!(
                "{JA001_ID}/{}/{motion}/0/{w}x{h}/{color}/",
                scenario.name(),
                motion = motion_name(Motion::Paused),
                w = JA001_FIRST_LEAF_WIDTH,
                h = JA001_FIRST_LEAF_HEIGHT,
                color = JA001_FIRST_LEAF_COLOR,
            )),
            "{}",
            frame.identity
        );
    }
}

#[test]
fn ja001_paused_t5_leaves_frames_unchanged() {
    for capture in ja001_paused_frame0_truecolor() {
        assert_eq!(
            capture.initial.digest, capture.after_ticks.digest,
            "{}",
            capture.initial.identity
        );
        assert_eq!(capture.initial.route, capture.after_ticks.route);
        assert_eq!(capture.initial.text, capture.after_ticks.text);
        assert_eq!(capture.initial.cursor, capture.after_ticks.cursor);
        assert_eq!(capture.initial.cells, capture.after_ticks.cells);
        assert_eq!(capture.initial.app_frame, capture.after_ticks.app_frame);
        assert_eq!(capture.initial.now_ms, capture.after_ticks.now_ms);
        assert!(!capture.after_ticks.clock_running);
    }
}

#[test]
fn ja001_repeat_from_fresh_worlds_matches() {
    let first = ja001_paused_frame0_truecolor();
    let second = ja001_paused_frame0_truecolor();
    assert_eq!(first, second);
}

#[test]
fn ja001_observes_production_for_scenario_draw_not_historical_paint() {
    assert_eq!(JA001_FIRST_LEAF_SIZE, (80, 24));
    assert_ne!(JA001_FIRST_LEAF_SIZE, HISTORICAL_PAINT_SIZE);

    let mut session = DirectSession::ja001_first_leaf(Scenario::Returning);
    let frame = session.observe("initial");
    assert_eq!(frame.width, 80);
    assert_eq!(frame.height, 24);
    assert_eq!(frame.route, "manager");
    assert!(!frame.clock_running);
    // Production shell hint is painted by App::draw at sizes that do not take
    // the 120×40 historical overpaint branch.
    assert!(
        frame.text.contains("Choose"),
        "production shell missing from 80x24 draw:\n{}",
        frame.text
    );

    session.ticks(5);
    let after = session.observe("after-t5");
    assert_eq!(frame.digest, after.digest);

    let other = DirectSession::paused_frame0(
        Scenario::Returning,
        Viewport::new(80, 24),
        CaptureColor::TrueColor,
    )
    .observe("initial");
    assert_eq!(frame.digest, other.digest);
}

fn assert_ja001_capture(
    scenario: Scenario,
    capture: &jackin_adapter::Ja001Capture,
    viewport: Viewport,
    color: &str,
) {
    let frame = &capture.initial;
    assert_eq!(frame.scenario, scenario.name());
    assert_eq!(frame.motion, "paused");
    assert_eq!(frame.construct_frame, 0);
    assert_eq!(frame.width, viewport.width);
    assert_eq!(frame.height, viewport.height);
    assert_eq!(frame.color, color);
    assert_eq!(frame.theme, "junie");
    assert_eq!(frame.route, route_name(expected_route(scenario)));
    assert_eq!(frame.motion_seed, MOTION_SEED);
    assert_eq!(frame.epoch_secs, EPOCH_SECS);
    assert_eq!(frame.now_secs, EPOCH_SECS);
    assert!(!frame.clock_running);
    assert!(frame.is_complete(), "{}", frame.identity);
    assert_eq!(
        frame.cells.len(),
        usize::from(viewport.width) * usize::from(viewport.height)
    );
    assert!(
        frame.text.chars().any(|c| c != ' ' && c != '\n'),
        "{} empty production draw",
        frame.identity
    );
    assert_eq!(
        capture.initial.digest, capture.after_ticks.digest,
        "{}",
        frame.identity
    );
    assert_eq!(capture.initial.cells, capture.after_ticks.cells);
    assert!(!capture.after_ticks.clock_running);
}

#[test]
fn ja001_remaining_sizes_truecolor_covers_eight_worlds() {
    let captures = ja001_paused_frame0_remaining_sizes_truecolor();
    assert_eq!(
        captures.len(),
        JA001_REMAINING_SIZES.len() * ja001_worlds().len()
    );
    let mut idx = 0;
    for viewport in JA001_REMAINING_SIZES {
        for scenario in ja001_worlds() {
            assert_ja001_capture(scenario, &captures[idx], viewport, "truecolor");
            idx += 1;
        }
    }
}

#[test]
fn ja001_120x40_observes_production_draw_without_blessing_overpaint() {
    let viewport = Viewport::new(120, 40);
    assert!(viewport.is_historical_paint_size());
    let capture =
        DirectSession::paused_frame0(Scenario::Returning, viewport, CaptureColor::TrueColor);
    let frame = capture.observe("initial");
    assert!(frame.at_historical_paint_size());
    assert_eq!(frame.width, 120);
    assert_eq!(frame.height, 40);
    assert_eq!(frame.route, "manager");
    assert!(frame.is_complete());
    assert_eq!(frame.motion_seed, MOTION_SEED);
    // Production draw is observed in memory. There is no blessed expected
    // artifact for the 120×40 overpaint branch.
    let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/baselines");
    assert!(
        !baseline.exists(),
        "adapter must not bless historical overpaint into {baseline:?}"
    );
    let small = DirectSession::ja001_first_leaf(Scenario::Returning).observe("initial");
    assert_ne!(frame.digest, small.digest);
    assert_ne!((frame.width, frame.height), (small.width, small.height));
}

#[test]
fn ja001_remaining_colors_cover_eight_worlds_all_sizes() {
    let captures = ja001_paused_frame0_remaining_colors();
    let expected = JA001_SIZES.len() * CaptureColor::ja001_remaining().len() * ja001_worlds().len();
    assert_eq!(captures.len(), expected);
    let mut idx = 0;
    for viewport in JA001_SIZES {
        for color in CaptureColor::ja001_remaining() {
            for scenario in ja001_worlds() {
                assert_ja001_capture(scenario, &captures[idx], viewport, color.label());
                idx += 1;
            }
        }
    }
}

#[test]
fn ja001_none_and_nocolor_are_distinct_identities() {
    let none = DirectSession::paused_frame0(
        Scenario::Returning,
        Viewport::new(80, 24),
        CaptureColor::None,
    )
    .observe("initial");
    let nocolor = DirectSession::paused_frame0(
        Scenario::Returning,
        Viewport::new(80, 24),
        CaptureColor::NoColor,
    )
    .observe("initial");
    assert_eq!(none.color, "none");
    assert_eq!(nocolor.color, "nocolor");
    assert_ne!(none.identity, nocolor.identity);
    // Direct harness has no separate NO_COLOR env path; cells match mono.
    assert_eq!(none.digest, nocolor.digest);
    assert_eq!(none.cells, nocolor.cells);
}
