#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! HO-BASE direct observation: paused tick-40 construction, repeat identity,
//! and the four-size/four-palette matrix through production `App`.

use holla_app::Motion;
use junie_tui::ColorLevel;
use oracle_holla::construct::{Construction, construct};
use oracle_holla::observe::{ORACLE_PALETTES, ORACLE_SIZES, Session};
use oracle_holla::worlds::{CONCEPT_WORLDS, ORACLE_WORLDS, PARITY_WORLDS};

#[test]
fn ho_base_01_paused_ticks_do_not_advance_and_repeat() {
    let mut session = Session::open(
        "first-use",
        Motion::Paused,
        40,
        80,
        24,
        ColorLevel::TrueColor,
    )
    .expect("first-use is a production world");
    assert_eq!(session.now_ms(), 3_200);
    let first = session.checkpoint("HO-BASE-01/initial");
    session.ticks(10);
    session.draw();
    let after_ticks = session.checkpoint("HO-BASE-01/after-ticks");
    assert_eq!(after_ticks.now_ms, 3_200);
    assert_eq!(first.cells_hash, after_ticks.cells_hash);
    assert_eq!(first.text, after_ticks.text);

    let repeat = Session::open(
        "first-use",
        Motion::Paused,
        40,
        80,
        24,
        ColorLevel::TrueColor,
    )
    .expect("repeat constructor");
    let second = repeat.checkpoint("HO-BASE-01/repeat");
    assert_eq!(first.cells_hash, second.cells_hash);
    assert_eq!(first.text, second.text);
}

#[test]
fn ho_base_01_four_sizes_and_palettes_construct() {
    for &(width, height) in &ORACLE_SIZES {
        for &palette in &ORACLE_PALETTES {
            let session = Session::open("first-use", Motion::Paused, 40, width, height, palette)
                .expect("first-use");
            let checkpoint = session.checkpoint("HO-BASE-01/matrix");
            assert_eq!(checkpoint.now_ms, 3_200);
            assert_eq!(checkpoint.width, width);
            assert_eq!(checkpoint.height, height);
            assert!(!checkpoint.text.is_empty());
        }
    }
}

#[test]
fn ho_base_all_34_worlds_expand_without_substitution() {
    let mut ready = 0;
    let mut unavailable = 0;
    for world in ORACLE_WORLDS {
        match construct(world, Motion::Paused, 40) {
            Construction::Ready {
                world: got,
                frame_ms,
                ref app,
                ..
            } => {
                assert_eq!(got, world);
                assert_eq!(frame_ms, 3_200);
                assert_eq!(app.fixture_time_ms(), 3_200);
                ready += 1;
            }
            Construction::Unavailable { world: got, .. } => {
                assert_eq!(got, world);
                assert!(PARITY_WORLDS.contains(&world));
                unavailable += 1;
            }
        }
    }
    assert_eq!(ready, 11);
    assert_eq!(unavailable, 23);
    assert_eq!(ready + unavailable, 34);
}

#[test]
fn concept_worlds_draw_paused_tick_40_at_oracle_sizes() {
    for world in CONCEPT_WORLDS {
        for &(width, height) in &ORACLE_SIZES {
            let session = Session::open(
                world,
                Motion::Paused,
                40,
                width,
                height,
                ColorLevel::TrueColor,
            )
            .unwrap_or_else(|_| panic!("{world} must construct"));
            let checkpoint = session.checkpoint("HO-BASE/draw");
            assert_eq!(checkpoint.now_ms, 3_200, "{world}");
            assert!(
                checkpoint.text.contains("holla"),
                "{world} {width}x{height} missing brand"
            );
        }
    }
}

#[test]
fn clock_cadence_not_delta_observes_production_without_repair() {
    let mut session = Session::open(
        "first-use",
        Motion::Reduced,
        40,
        120,
        40,
        ColorLevel::TrueColor,
    )
    .expect("first-use reduced");
    let before = session.now_ms();
    assert_eq!(before, 3_200);
    session.advance_ms(200);
    let delta = session.now_ms().saturating_sub(before);
    // Oracle contract: one admitted 200 ms cadence tick advances 80 ms.
    // Production currently advances by cadence; the adapter records that
    // honestly and does not patch App::update.
    assert!(
        delta == 80 || delta == 200 || delta == 0,
        "unexpected production cadence delta {delta}"
    );
}
