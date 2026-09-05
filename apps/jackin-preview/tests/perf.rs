//! Jackin application-shell benchmarks (`docs/audit/performance-audit.md`
//! §7.2 A/B). Run in release:
//!
//! ```text
//! cargo test --release --bin jackin-preview perf_tests -- --test-threads=1 --nocapture
//! ```
//!
//! Fixtures: the `hard-cases` scenario for the manager (every Workspace
//! expanded so all instance rows are visible) and `capsule-multi` for the
//! Capsule, with the active tab split to four panes and every pane's
//! scrollback filled to `SCROLLBACK` (2 000) lines.

#![allow(
    unsafe_code,
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::print_stdout,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::undocumented_unsafe_blocks
)]

use std::hint::black_box;

use jackin_app::Route;
use jackin_app::sim::pty::{Line, SCROLLBACK, Span, SplitDir, Tone};
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;
use junie_tui_testing::perf::{Counting, Stats, bench, env_flag, iters, lock, report_to};

mod support;
use support::H;

#[global_allocator]
static GLOBAL: Counting = Counting;

const PERF_BASELINE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt");

fn report(name: &str, stats: &Stats) {
    report_to(PERF_BASELINE, name, stats);
}

impl H {
    /// `hard-cases` manager with every Workspace expanded.
    fn manager(w: u16, h: u16) -> Self {
        let mut hh = H::new(Scenario::HardCases, Motion::Reduced, 0, w, h);
        assert_eq!(hh.app().route(), Route::Manager);
        let ids: Vec<_> = hh.app().world.workspaces.iter().map(|ws| ws.id).collect();
        for id in ids {
            hh.app_mut().manager.toggle(id);
        }
        hh.draw();
        hh
    }

    /// `capsule-multi` Capsule with four panes of 2 000 scrollback lines.
    fn capsule(w: u16, h: u16) -> Self {
        let mut hh = H::new(Scenario::CapsuleMulti, Motion::Full, 0, w, h);
        assert_eq!(hh.app().route(), Route::Capsule);
        let instance = hh
            .app()
            .world
            .running()
            .first()
            .map(|instance| instance.id.clone())
            .expect("running instance");
        let now = hh.app().world.now_ms();
        let d = hh
            .app_mut()
            .world
            .daemons
            .get_mut(&instance)
            .expect("daemon");
        if d.active_tab().map(|t| t.leaves().len()) == Some(3) {
            d.split(SplitDir::Vertical, false, None, None, now, false)
                .expect("split the focused pane");
        }
        let leaves = d.active_tab().unwrap().leaves();
        assert_eq!(leaves.len(), 4, "four panes on the active tab");
        for id in leaves {
            let pane = d.pane_mut(id).unwrap();
            let mut i = pane.term.lines.len();
            while pane.term.lines.len() < SCROLLBACK {
                let line: Line = vec![
                    Span::new(format!("[{i:05}] "), Tone::Secondary),
                    Span::new(
                        format!("lorem ipsum dolor sit amet, consectetur adipiscing elit {i}"),
                        Tone::Normal,
                    ),
                ];
                pane.term.push(line);
                i += 1;
            }
            assert_eq!(pane.term.lines.len(), SCROLLBACK);
        }
        hh.draw();
        hh
    }

    fn regions(&self) -> (usize, usize) {
        (
            self.harness.runtime().region_count(),
            self.harness.ring().reachable().count(),
        )
    }
}

/// The application baseline carries the three live Slice 7 measurements.
/// The old per-frame viewport clone is intentionally absent: its deletion is
/// an acceptance condition, not another benchmark row.
#[test]
fn jackin_perf_baseline_has_required_rows_and_no_deleted_clone() {
    const ROWS: [&str; 3] = [
        "frame_jackin_capsule_4panes_120x40",
        "frame_jackin_manager_100rows_120x40",
        "key_jackin_manager_move",
    ];
    let names: Vec<_> = include_str!("perf_baseline.txt")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| !name.starts_with('#'))
        .collect();
    assert_eq!(names, ROWS);
    assert!(!names.contains(&"capsule_pane_clone_4x2000"));
}

// ------------------------------------------------------------ A. frames

/// Manager render with every Workspace expanded: exercises the cached row
/// projection, detail projection and action controls per frame.
#[test]
fn frame_jackin_manager_100rows_120x40() {
    let _g = lock();
    let mut h = H::manager(120, 40);
    println!(
        "PERF-NOTE frame_jackin_manager_100rows_120x40 workspaces={} instances={}",
        h.app().world.workspaces.len(),
        h.app().world.instances.len()
    );
    let s = bench(1, iters(200), &mut || h.draw());
    let (hits, ring) = h.regions();
    report(
        "frame_jackin_manager_100rows_120x40",
        &s.with_regions(hits, ring),
    );
    if env_flag("PERF_STRICT") {
        assert!(s.allocs < 60, "manager frame allocates {} times", s.allocs);
    }
}

/// Four panes × 2 000 scrollback lines: renders each daemon viewport in place
/// without cloning its scrollback (`capsule.rs`).
#[test]
fn frame_jackin_capsule_4panes_120x40() {
    let _g = lock();
    let mut h = H::capsule(120, 40);
    let s = bench(1, iters(10), &mut || h.draw());
    let (hits, ring) = h.regions();
    report(
        "frame_jackin_capsule_4panes_120x40",
        &s.with_regions(hits, ring),
    );
    if env_flag("PERF_STRICT") {
        assert!(s.allocs < 200, "capsule frame allocates {} times", s.allocs);
    }
}

// ------------------------------------------------------------ B. events

/// 1 000 arrow keys (Down/Up alternating) in the manager tree; the stable
/// projection must not rebuild or allocate per key.
#[test]
fn key_jackin_manager_move() {
    let _g = lock();
    let mut h = H::manager(120, 40);
    let mut down = true;
    let s = bench(10, iters(1000), &mut || {
        let code = if down { KeyCode::Down } else { KeyCode::Up };
        down = !down;
        black_box(h.key(code));
    });
    report("key_jackin_manager_move", &s);
    if env_flag("PERF_STRICT") {
        assert_eq!(s.allocs, 0, "manager key path allocates {} times", s.allocs);
    }
}
