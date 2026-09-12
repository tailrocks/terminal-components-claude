//! Visual-baseline suite: every capturable surface of the four binaries,
//! driven as real processes in PTYs via the tuisnap library and gated
//! cell-exact + pixel-exact against the approved frames in `shots/tuisnap/`.
//!
//! This suite replaces the retired `tools/tuisnap_baseline.sh`; the capture
//! matrix and its rationale live in `docs/baseline/tuisnap-coverage.md`. The
//! 367 ported captures keep the bash runner's names, argv, boot needles,
//! send steps and per-capture timeouts verbatim (via [`tuisnap::pty::run_once`],
//! the same runner the `tuisnap run` CLI used); the `pointer` module adds the
//! mouse/resize group the CLI could not express (hover, drag-select, wheel
//! scroll-fade, resize sequences).
//!
//! Every capture test is `#[ignore]`d: default `cargo test` compiles the
//! suite but runs no PTY captures. Run the baseline explicitly:
//!
//! ```sh
//! cargo test --test visual_baseline -- --ignored            # whole matrix
//! cargo test --test visual_baseline holla_ -- --ignored     # one app
//! cargo test --test visual_baseline report -- --ignored     # rebuild report.html
//! ```
//!
//! Gate policy (fail-closed, unchanged from the CLI): `matched` passes,
//! `missing-approval` passes but is logged as pending (expected for new
//! names on a first run — `tuisnap accept --store shots/tuisnap --all` is
//! the only bless), drift after approval or a capture error fails the test.

#![cfg(any(target_os = "macos", target_os = "linux"))]

mod holla;
mod jackin;
mod pointer;
mod showcase;
mod support;
mod tablepro;

use tuisnap::{Profile, VENDORED_FACES};

/// Rebuild `shots/tuisnap/report.html` from the store (the library form of
/// `tuisnap report --store shots/tuisnap`): re-verifies every actual frame
/// against its approval. Run after a generation run + accept; 0 failed is
/// the green gate.
#[test]
#[ignore = "rebuilds shots/tuisnap/report.html; run after accept"]
fn report() {
    let store = support::store();
    let mut renderer = Profile::default_profile()
        .renderer(&VENDORED_FACES)
        .expect("vendored faces parse");
    let report = store
        .report_with(&mut renderer, 1.0, "tuisnap visual report")
        .expect("report generation");
    eprintln!(
        "report: {} ({} captures, {} failed)",
        report.path.display(),
        report.outcomes.len(),
        report.failed()
    );
    assert_eq!(report.failed(), 0, "unmatched gates — review report.html");
}
