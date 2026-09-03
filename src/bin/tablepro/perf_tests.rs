//! TablePro application-shell benchmarks (`docs/audit/performance-audit.md`
//! §7.2 A/B/D/F). Run in release:
//!
//! ```text
//! cargo test --release --bin tablepro perf_tests -- --test-threads=1 --nocapture
//! ```
//!
//! Fixture: the `Production` connection with `public.orders` opened as a
//! table tab. `orders` has 14 columns and the loader caps at
//! `sql::ROW_CAP` = 500 rows, so the "500×12" benchmarks of the audit run
//! on a 500×14 grid here.

#[path = "../../../tests/perf_common.rs"]
mod perf_common;

use std::hint::black_box;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

use crate::app::App;
use crate::tabs::TableTab;
use crate::workbench::WorkTab;
use perf_common::{Counting, bench, env_flag, iters, lock, report};

#[global_allocator]
static GLOBAL: Counting = Counting;

struct H {
    app: App,
    term: Terminal<TestBackend>,
}

impl H {
    /// Connected workbench with `public.orders` open and the grid focused.
    fn orders(w: u16, h: u16) -> Self {
        let mut app = App::new(Theme::junie());
        let i = app
            .connections
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        app.connect(i);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut hh = Self { app, term };
        hh.draw();
        // cursor starts on the schema row; `orders` is the 4th table
        for _ in 0..5 {
            hh.key(KeyCode::Down);
        }
        hh.key(KeyCode::Enter);
        let t = hh.table();
        assert_eq!(t.name, "orders");
        assert_eq!(t.grid.len(), 500, "ROW_CAP rows loaded");
        assert_eq!(t.grid.columns.len(), 14);
        hh
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key(&mut self, code: KeyCode) {
        self.app.handle(key(code));
        self.draw();
    }

    fn table(&self) -> &TableTab {
        match self.app.workbench.as_ref().unwrap().active_tab() {
            Some(WorkTab::Table(t)) => t,
            _ => panic!("active tab is not a table"),
        }
    }

    fn table_mut(&mut self) -> &mut TableTab {
        match self.app.workbench.as_mut().unwrap().active_tab_mut() {
            Some(WorkTab::Table(t)) => t,
            _ => panic!("active tab is not a table"),
        }
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

const FRAME: &str = "frame_tablepro_grid_500x12_120x40";

// ------------------------------------------------------------ A. frames

#[test]
fn frame_tablepro_grid_500x12_120x40() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    let s = bench(1, iters(200), &mut || h.draw());
    let (hits, ring) = h.regions();
    report(FRAME, &s.with_regions(hits, ring));
}

// ------------------------------------------------------------ B. events

/// 1 000 arrow keys cycling Down/Right/Up/Left on the 500-row grid.
#[test]
fn key_tablepro_grid_cursor() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    const ARROWS: [KeyCode; 4] = [KeyCode::Down, KeyCode::Right, KeyCode::Up, KeyCode::Left];
    let mut i = 0usize;
    let s = bench(10, iters(1000), &mut || {
        black_box(h.app.handle(key(ARROWS[i % 4])));
        i += 1;
    });
    let c = h.table().grid.cursor;
    println!("PERF-NOTE key_tablepro_grid_cursor cursor={c:?}");
    report("key_tablepro_grid_cursor", &s);
}

/// 20 `s` presses with `local_sort = true`: asc → desc → none cycles on the
/// cursor column, isolating `cmp_cells`' allocations per comparison.
#[test]
fn key_tablepro_grid_sort_local() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    h.table_mut().grid.local_sort = true;
    // a fixed 20 presses in every build: the asc/desc/none mix per press
    // depends on the count, so it must not scale with `iters`
    let s = bench(0, 20, &mut || {
        black_box(h.app.handle(key(KeyCode::Char('s'))));
    });
    report("key_tablepro_grid_sort_local", &s);
}

/// 1 000 clicks (Down + Up) on a body cell, routed through the workbench,
/// `DataGrid::owns` and `DataGrid::locate`.
#[test]
fn mouse_click_grid_cell() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    let cell = h.table().grid.cell_id(2, 1);
    let area = h.app.hits.area_of(cell).expect("cell (2,1) registered");
    let pos = Position::new(area.x + area.width / 2, area.y);
    let click = |h: &mut H| {
        h.app.handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos,
        }));
        black_box(h.app.handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos,
        })));
    };
    let s = bench(10, iters(1000), &mut || click(&mut h));
    assert_eq!(h.table().grid.cursor, (2, 1), "click lands on the cell");
    report("mouse_click_grid_cell", &s);
}

/// 1 000 wheel events over the grid, alternating direction.
#[test]
fn wheel_tablepro_grid() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    let area = h.table().grid.area;
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
    report("wheel_tablepro_grid", &s);
    assert_eq!(s.allocs, 0, "wheel routing must not allocate");
}

// ------------------------------------------------------------ D. large data

/// One `TableTab::load`: the three-stage copy (`sql::run_select` →
/// `to_cell` → `DataGrid::set_rows`).
#[test]
fn grid_500x12_load() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    let cat = h.app.workbench.as_ref().unwrap().catalog.clone();
    let s = bench(1, iters(5), &mut || {
        h.table_mut().load(&cat);
    });
    assert_eq!(h.table().grid.len(), 500);
    report("grid_500x12_load", &s);
    if env_flag("PERF_TARGET") {
        assert!(s.allocs < 8_000, "load allocates {} times", s.allocs);
    }
}

// ------------------------------------------------------------ F. invariants

/// R4 guard: the grid frame must allocate the same number of times in debug
/// and release. The baseline is recorded in release, so a debug run of this
/// test compares its own count against that number:
///
/// ```text
/// cargo test --release --bin tablepro debug_and_release_alloc_counts_match -- --nocapture
/// cargo test           --bin tablepro debug_and_release_alloc_counts_match -- --nocapture
/// ```
///
/// Both print `PERF-BUILD ... allocs=<n> baseline=<n>`; the debug line must
/// show equal numbers. The assertion is enforced under `PERF_TARGET=1`.
#[test]
fn debug_and_release_alloc_counts_match() {
    let _g = lock();
    let mut h = H::orders(120, 40);
    let s = bench(1, iters(20), &mut || h.draw());
    let baseline = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/perf_baseline.txt"
    ))
    .ok()
    .and_then(|text| {
        text.lines()
            .find(|l| l.starts_with(&format!("{FRAME} ")))
            .and_then(|l| l.split_whitespace().nth(2)?.parse::<usize>().ok())
    });
    println!(
        "PERF-BUILD {FRAME} debug_assertions={} allocs={} baseline={}",
        cfg!(debug_assertions),
        s.allocs,
        baseline.map_or("none".to_owned(), |b| b.to_string())
    );
    if let Some(b) = baseline {
        let matches = s.allocs == b;
        println!(
            "PERF-BUILD-{} {FRAME}",
            if matches { "MATCH" } else { "MISMATCH" }
        );
        if env_flag("PERF_TARGET") {
            assert!(matches, "debug allocs {} != release baseline {b}", s.allocs);
        }
    }
}
