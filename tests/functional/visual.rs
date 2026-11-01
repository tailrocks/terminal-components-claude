//! Strict visual check with latest Tuiscotti (main `d8d5415`, 0.4.0):
//! a live `tui_test` snapshot fed into `tuiscotti::check::FrozenStore`.
//! `check()` returning `Ok` is not a pass — the outcome must be asserted.

use std::path::PathBuf;

use super::support::{App, SHOWCASE, find, text};

#[test]
fn showcase_held_button_matches_frozen_tuiscotti_baseline() {
    let app = App::new(
        "showcase-visual-held",
        SHOWCASE,
        &["--page", "buttons", "--motion", "reduced", "--frame", "40"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Run task");
    let run = find(&initial, "Run task");
    let hit = super::support::Point {
        x: run.x + 2,
        ..run
    };
    app.move_to(hit);
    app.down(hit);
    app.wait_screen("held state before frozen capture", |s| {
        !text(s).contains("Run task ✓")
    });
    let snapshot = app.screen();

    let sample = tuiscotti_latest::sample::Sample::state(
        "showcase/buttons/held",
        snapshot,
        tuiscotti_latest::sample::Observation::empty(),
    )
    .expect("build state sample");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let store = tuiscotti_latest::check::FrozenStore::new(
        &root.join("tests/functional/baselines"),
        &root.join("target/functional/artifacts"),
    )
    .expect("open frozen store");
    let outcome = store
        .check(&sample.to_bytes().expect("serialize sample"))
        .expect("run frozen check");
    assert!(outcome.is_pass(), "visual mismatch: {outcome:?}");
    app.up(hit);
    app.finish();
}
