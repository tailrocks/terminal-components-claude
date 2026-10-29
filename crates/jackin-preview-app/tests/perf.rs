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

use jackin_preview_app::Route;
use jackin_preview_app::sim::pty::{Line, SCROLLBACK, Span, SplitDir, Tone};
use jackin_preview_app::{Motion, Scenario};
use termrock::KeyCode;
use termrock_test_support::perf::{Counting, Stats, bench, env_flag, iters, lock, report_to};

mod support;
use support::H;

#[global_allocator]
static GLOBAL: Counting = Counting;

const PERF_BASELINE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt");

fn report(name: &str, stats: &Stats) {
    report_to(PERF_BASELINE, name, stats);
}

impl H {
    fn manager(w: u16, h: u16) -> Self {
        let mut hh = H::new(Scenario::HardCases, Motion::Reduced, 0, w, h);
        assert_eq!(hh.app().route(), Route::Manager);
        let ids: Vec<_> = hh.app().world.workspaces.iter().map(|ws| ws.id).collect();
        for id in ids {
            hh.app_mut().manager.toggle(id);
        }
        hh.ticks(1);
        hh.draw();
        hh
    }

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
        while d.active_tab().map_or(0, |t| t.leaves().len()) < 4 {
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
}

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
    report("frame_jackin_manager_100rows_120x40", &s);
    if env_flag("PERF_STRICT") {
        assert!(s.allocs < 60, "manager frame allocates {} times", s.allocs);
    }
}

#[test]
fn frame_jackin_capsule_4panes_120x40() {
    let _g = lock();
    let mut h = H::capsule(120, 40);
    let s = bench(1, iters(10), &mut || h.draw());
    report("frame_jackin_capsule_4panes_120x40", &s);
    if env_flag("PERF_STRICT") {
        assert!(s.allocs < 200, "capsule frame allocates {} times", s.allocs);
    }
}

#[test]
fn key_jackin_manager_move() {
    let _g = lock();
    let mut h = H::manager(120, 40);
    h.harness = h.harness.with_auto_draw(false);
    let mut down = true;
    let s = bench(10, iters(1000), &mut || {
        let code = if down { KeyCode::Down } else { KeyCode::Up };
        down = !down;
        h.key(code);
    });
    report("key_jackin_manager_move", &s);
    if env_flag("PERF_STRICT") {
        assert_eq!(s.allocs, 0, "manager key path allocates {} times", s.allocs);
    }
}
