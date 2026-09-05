//! Pre-refactor cell-exact visual digests for jackin-preview (WP-0).
//!
//! Every surface is a fixture scenario at a fixed frame under `Paused` or
//! `Reduced` motion, driven by a deterministic key sequence through the real
//! `App` on a `TestBackend`. The whole buffer (symbol, fg, bg, modifier of
//! every cell) is folded into an FNV-1a digest, one line per surface, and
//! compared against this package's `tests/baselines/jackin.txt`. Regenerate only with
//! `UPDATE_BASELINE=1` after inspecting the change.

#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::cast_lossless,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use junie_tui::KeyCode;

use jackin_app::Route;
mod support;
use jackin_app::{Motion, Scenario};
use support::H;

/// FNV-1a over every cell of the current frame. No rect is excluded.
fn digest(h: &H) -> u64 {
    let buf = h.buffer();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for cell in buf.content.iter() {
        let s = format!(
            "{}|{:?}|{:?}|{:?};",
            cell.symbol(),
            cell.fg,
            cell.bg,
            cell.modifier
        );
        for b in s.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

/// Reduced-motion ticks after which the `LaunchFailure` cockpit shows its
/// failure state (the failure is clock-driven, so a paused seek cannot reach
/// it; the virtual clock advances only with ticks, so the count is exact).
const FAILURE_TICKS: usize = 231;
/// Mid-pipeline frame of the `LaunchRunning` cockpit.
const RUNNING_FRAME: u64 = 20;
/// Outro frame inside the caption phase.
const OUTRO_FRAME: u64 = 150;

fn paused(sc: Scenario, frame: u64, w: u16, h: u16) -> H {
    H::new(sc, Motion::Paused, frame, w, h)
}

fn manager(w: u16, h: u16) -> H {
    let hh = paused(Scenario::Returning, 0, w, h);
    assert_eq!(hh.app().route(), Route::Manager);
    hh
}

fn capsule(w: u16, h: u16) -> H {
    let hh = paused(Scenario::CapsuleMulti, 0, w, h);
    assert_eq!(hh.app().route(), Route::Capsule);
    hh
}

fn editor_tab(n: usize) -> H {
    let mut h = manager(120, 40);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    assert_eq!(h.app().route(), Route::Editor);
    for _ in 0..n {
        h.key(KeyCode::Char(']'));
    }
    h
}

type Builder = fn() -> H;

const SURFACES: &[(&str, u16, u16, Builder)] = &[
    // start route of every scenario
    ("start-first-use", 120, 40, || {
        let h = paused(Scenario::FirstUse, 0, 120, 40);
        assert_eq!(h.app().route(), Route::Intro);
        h
    }),
    ("start-returning", 120, 40, || manager(120, 40)),
    ("start-accounts-mixed", 120, 40, || {
        let h = paused(Scenario::AccountsMixed, 0, 120, 40);
        assert_eq!(h.app().route(), Route::Accounts);
        h
    }),
    ("start-launch-running", 120, 40, || {
        let h = paused(Scenario::LaunchRunning, 0, 120, 40);
        assert_eq!(h.app().route(), Route::Cockpit);
        h
    }),
    ("start-launch-failure", 120, 40, || {
        let h = paused(Scenario::LaunchFailure, 0, 120, 40);
        assert_eq!(h.app().route(), Route::Cockpit);
        h
    }),
    ("start-capsule-multi", 120, 40, || capsule(120, 40)),
    ("start-outro-last", 120, 40, || {
        let h = paused(Scenario::OutroLast, 0, 120, 40);
        assert_eq!(h.app().route(), Route::Capsule);
        h
    }),
    ("start-hard-cases", 120, 40, || {
        paused(Scenario::HardCases, 0, 120, 40)
    }),
    // intro mid-phrase
    ("intro-phrase", 120, 40, || {
        let h = paused(Scenario::FirstUse, 45, 120, 40);
        assert!(h.text().contains("Stand up, operator…"), "{}", h.text());
        h
    }),
    // manager
    ("manager-expanded-detail", 120, 40, || {
        let mut h = manager(120, 40);
        h.key(KeyCode::Down);
        h.key(KeyCode::Right);
        h.key(KeyCode::Down);
        h.key(KeyCode::Tab);
        assert!(h.text().contains("Live topology"), "{}", h.text());
        h
    }),
    // prelude
    ("prelude-step-1", 120, 40, || {
        let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
        h.key(KeyCode::End);
        h.key(KeyCode::Enter);
        assert_eq!(h.app().route(), Route::Prelude);
        assert!(h.text().contains("step 1 of 5"), "{}", h.text());
        h
    }),
    // editor tabs
    ("editor-general", 120, 40, || editor_tab(0)),
    ("editor-mounts", 120, 40, || editor_tab(1)),
    ("editor-roles", 120, 40, || editor_tab(2)),
    ("editor-environments", 120, 40, || editor_tab(3)),
    ("editor-accounts", 120, 40, || editor_tab(4)),
    // settings
    ("settings", 120, 40, || {
        let mut h = manager(120, 40);
        h.key(KeyCode::Char('s'));
        assert_eq!(h.app().route(), Route::Settings);
        h
    }),
    // accounts
    ("accounts-form", 120, 40, || {
        let mut h = paused(Scenario::AccountsMixed, 0, 120, 40);
        h.key(KeyCode::Char('a'));
        assert!(h.text().contains("New account"), "{}", h.text());
        h
    }),
    ("accounts-1password-step-1", 120, 40, || {
        let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
        h.key(KeyCode::Char('a'));
        h.key(KeyCode::Enter);
        h.type_str("Team");
        for _ in 0..4 {
            h.key(KeyCode::Tab);
        }
        h.key(KeyCode::Enter);
        h.ticks(4);
        assert!(h.text().contains("chainargos"), "{}", h.text());
        h
    }),
    // usage
    ("usage", 120, 40, || {
        let mut h = manager(120, 40);
        h.key(KeyCode::Char('u'));
        assert_eq!(h.app().route(), Route::Usage);
        h
    }),
    // cockpit
    ("cockpit-running", 120, 40, || {
        let h = paused(Scenario::LaunchRunning, RUNNING_FRAME, 120, 40);
        assert_eq!(h.app().route(), Route::Cockpit);
        h
    }),
    ("cockpit-failure", 120, 40, || {
        let mut h = H::new(Scenario::LaunchFailure, Motion::Reduced, 0, 120, 40);
        h.ticks(FAILURE_TICKS);
        assert!(h.text().contains("Launch failed"), "{}", h.text());
        h
    }),
    // capsule
    ("capsule-app-menu", 120, 40, || {
        let mut h = capsule(120, 40);
        h.key(KeyCode::F(10));
        assert!(h.text().contains("New tab"), "{}", h.text());
        h
    }),
    ("capsule-tab-context-menu", 120, 40, || {
        let mut h = capsule(120, 40);
        h.ctrl('b');
        h.key(KeyCode::Char('m'));
        assert!(h.text().contains("Change title…"), "{}", h.text());
        h
    }),
    ("capsule-command-palette", 120, 40, || {
        let mut h = capsule(120, 40);
        h.ctrl('\\');
        assert!(h.text().contains("Command palette"), "{}", h.text());
        h
    }),
    ("capsule-inspect-changes", 120, 40, || {
        let mut h = capsule(120, 40);
        h.key(KeyCode::F(10));
        h.key(KeyCode::Right);
        h.key(KeyCode::Right);
        h.key(KeyCode::End);
        h.key(KeyCode::Enter);
        assert!(h.text().contains("Inspect changes ·"), "{}", h.text());
        h
    }),
    ("capsule-choice-dialog", 120, 40, || {
        let mut h = capsule(120, 40);
        h.ctrl('q');
        assert!(h.text().contains("Unsaved work"), "{}", h.text());
        h
    }),
    ("capsule-help", 120, 40, || {
        let mut h = capsule(120, 40);
        h.key(KeyCode::F(10));
        h.key(KeyCode::Left);
        h.key(KeyCode::Enter);
        assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
        h
    }),
    // outro
    ("outro-caption", 120, 40, || {
        let h = paused(Scenario::OutroLast, OUTRO_FRAME, 120, 40);
        assert_eq!(h.app().route(), Route::Outro);
        assert!(
            h.text().contains("You were in the Construct"),
            "{}",
            h.text()
        );
        h
    }),
    // too small
    ("too-small", 60, 18, || {
        let h = paused(Scenario::Returning, 0, 60, 18);
        assert!(h.text().contains("Terminal too small"), "{}", h.text());
        h
    }),
    // responsive
    ("manager", 80, 24, || manager(80, 24)),
    ("manager", 100, 30, || manager(100, 30)),
    ("manager", 160, 50, || manager(160, 50)),
    ("capsule", 80, 24, || capsule(80, 24)),
    ("capsule", 100, 30, || capsule(100, 30)),
    ("capsule", 160, 50, || capsule(160, 50)),
];

#[test]
fn jackin_visual_baseline() {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (label, w, hgt, build) in SURFACES {
        let a = digest(&build());
        let b = digest(&build());
        assert_eq!(
            a, b,
            "{w}x{hgt} {label}: two builds of the same frame differ"
        );
        writeln!(out, "{w}x{hgt} {label} {a:016x}").unwrap();
    }
    // The frozen 37-surface before-image travels with this package. This test
    // compares against it; only the explicit coordinator workflow may update
    // that file.
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/baselines/jackin.txt");
    if std::env::var_os("UPDATE_BASELINE").is_some() {
        std::fs::write(path, &out).unwrap();
        return;
    }
    let expected =
        std::fs::read_to_string(path).expect("baseline file; run with UPDATE_BASELINE=1");
    assert_eq!(
        out, expected,
        "jackin-preview rendering changed; inspect before updating the baseline"
    );
}
