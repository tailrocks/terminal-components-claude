//! Showcase application-shell benchmarks (`docs/audit/performance-audit.md`
//! §7.2 A/B). Run in release:
//!
//! ```text
//! cargo test --release --bin showcase perf_tests -- --test-threads=1 --nocapture
//! ```

#[path = "../../../tests/perf_common.rs"]
mod perf_common;

use std::hint::black_box;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

use crate::app::{App, PageId};
use perf_common::{Counting, bench, iters, lock, measure_once, report};

#[global_allocator]
static GLOBAL: Counting = Counting;

struct H {
    app: App,
    term: Terminal<TestBackend>,
}

impl H {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut hh = Self { app, term };
        hh.draw();
        hh
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key(&mut self, code: KeyCode) {
        self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
    }

    fn regions(&self) -> (usize, usize) {
        (self.app.hits.len(), self.app.ring.reachable().len())
    }
}

fn key(code: KeyCode) -> Input {
    Input::Key(Key {
        code,
        mods: KeyModifiers::NONE,
    })
}

fn frame_bench(name: &str, h: &mut H, frames: usize) {
    let s = bench(1, iters(frames), &mut || h.draw());
    let (hits, ring) = h.regions();
    report(name, &s.with_regions(hits, ring));
}

// ------------------------------------------------------------ A. frames

#[test]
fn frame_showcase_lists_120x40() {
    let _g = lock();
    let mut h = H::new(120, 40, PageId::Lists);
    frame_bench("frame_showcase_lists_120x40", &mut h, 200);
}

#[test]
fn frame_showcase_lists_80x24() {
    let _g = lock();
    let mut h = H::new(80, 24, PageId::Lists);
    frame_bench("frame_showcase_lists_80x24", &mut h, 200);
}

/// Lists page with the help dialog open: the shadowed page is still fully
/// rendered and registered below the barrier (§3.1).
#[test]
fn frame_showcase_dialog_open() {
    let _g = lock();
    let mut h = H::new(120, 40, PageId::Lists);
    h.key(KeyCode::Char('?'));
    assert!(h.app.dialog.is_some(), "help dialog opens with ?");
    frame_bench("frame_showcase_dialog_open", &mut h, 200);
}

/// Identical frames must allocate identically (no lazily growing caches).
#[test]
fn render_twice_allocates_the_same() {
    let _g = lock();
    let mut h = H::new(120, 40, PageId::Lists);
    h.draw();
    let a = measure_once(&mut || h.draw());
    let b = measure_once(&mut || h.draw());
    println!(
        "PERF showcase_render_twice ns={} allocs={} bytes={}",
        b.ns, b.allocs, b.bytes
    );
    assert_eq!(a.allocs, b.allocs);
    assert_eq!(a.bytes, b.bytes);
}

// ------------------------------------------------------------ B. events

/// 1 000 `Down` keys into the focused language list after one render;
/// includes `describe_key` on every keystroke (`app.rs`).
#[test]
fn key_showcase_down_lists() {
    let _g = lock();
    let mut h = H::new(120, 40, PageId::Lists);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.focus.current(),
        Some(WidgetId::of("lists").sub("single")),
        "Tab focuses the first list"
    );
    let s = bench(10, iters(1000), &mut || {
        black_box(h.app.handle(key(KeyCode::Down)));
    });
    report("key_showcase_down_lists", &s);
}

/// A raster of pointer positions (every other column, every row: 2 400
/// probes) over the real Lists-page registry.
#[test]
fn mouse_move_showcase_frame() {
    let _g = lock();
    let h = H::new(120, 40, PageId::Lists);
    let s = bench(1, iters(10), &mut || {
        for y in 0..40u16 {
            for x in (0..120u16).step_by(2) {
                black_box(h.app.hits.hit(Position::new(x, y)));
            }
        }
    });
    let (hits, ring) = h.regions();
    println!(
        "PERF-NOTE mouse_move_showcase_frame ns_per_hit={}",
        s.ns / 2400
    );
    report("mouse_move_showcase_frame", &s.with_regions(hits, ring));
    assert_eq!(s.allocs, 0);
}

/// 1 000 wheel events over the language list, alternating direction.
#[test]
fn wheel_showcase_lists() {
    let _g = lock();
    let mut h = H::new(120, 40, PageId::Lists);
    let area = h
        .app
        .hits
        .area_of(WidgetId::of("lists").sub("single"))
        .expect("language list registered");
    let pos = Position::new(area.x + area.width / 2, area.y + area.height / 2);
    let mut down = true;
    let s = bench(10, iters(1000), &mut || {
        down = !down;
        let kind = if down {
            MouseKind::WheelDown
        } else {
            MouseKind::WheelUp
        };
        black_box(h.app.handle(Input::Mouse(Mouse { kind, pos })));
    });
    report("wheel_showcase_lists", &s);
    assert_eq!(s.allocs, 0, "wheel routing must not allocate");
}
