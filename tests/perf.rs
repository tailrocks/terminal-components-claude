//! Library-level performance benchmarks (`docs/audit/performance-audit.md`
//! §7.2 B–F). Application-shell benchmarks (A and the app-level rows of B)
//! live in `src/bin/*/perf_tests.rs`, which include the same harness.
//!
//! Run in release, single-threaded, with output visible:
//!
//! ```text
//! cargo test --release --test perf -- --test-threads=1 --nocapture
//! ```
//!
//! `--test-threads=1` is only needed for stable wall times: allocation
//! counts are protected by `perf_common::lock()`. See `tests/perf_common.rs`
//! for the environment knobs (`PERF_BLESS`, `PERF_STRICT`, `PERF_TARGET`,
//! `PERF_ITERS`, `PERF_FULL`).

mod perf_common;

use std::hint::black_box;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use junie_tui::core::event::Key;
use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::core::text::TextBuffer;
use junie_tui::theme::{ColorLevel, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx, VisualState};
use junie_tui::ui::text;
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::tree::{TreeNode, TreeView};
use junie_tui::widgets::viewport::{Line, Span, TextViewport};

use perf_common::{Counting, bench, big, check_ratio, env_flag, iters, lock, measure_once, report};

#[global_allocator]
static GLOBAL: Counting = Counting;

// ------------------------------------------------------------ helpers

/// A buffer plus per-frame registries, rebuilt exactly as the apps do.
struct Canvas {
    theme: Theme,
    buf: Buffer,
    hits: HitRegistry,
    ring: FocusRing,
}

impl Canvas {
    fn new(w: u16, h: u16) -> Self {
        Self {
            theme: Theme::junie(),
            buf: Buffer::empty(Rect::new(0, 0, w, h)),
            hits: HitRegistry::default(),
            ring: FocusRing::default(),
        }
    }

    fn frame(&mut self, f: impl FnOnce(Rect, &mut Buffer, &mut RenderCtx)) {
        self.buf.reset();
        self.hits = HitRegistry::default();
        self.ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &self.theme,
            Interaction::default(),
            &mut self.hits,
            &mut self.ring,
        );
        let area = *self.buf.area();
        f(area, &mut self.buf, &mut ctx);
    }

    fn regions(&self) -> (usize, usize) {
        (self.hits.len(), self.ring.reachable().len())
    }
}

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn list_of(n: usize) -> ListBox {
    let items = (0..n)
        .map(|i| ListItem::new(&format!("Row {i:06} — item label")))
        .collect();
    ListBox::new(WidgetId::of("perf.list"), items, SelectMode::Single)
}

/// `dirs` folders × 999 leaves (+ the folders themselves).
fn tree_of(dirs: usize) -> TreeView {
    let nodes = (0..dirs)
        .map(|d| {
            let leaves = (0..999)
                .map(|l| TreeNode::leaf_meta(&format!("file_{d:03}_{l:04}.rs"), "modified"))
                .collect();
            TreeNode::dir(&format!("folder_{d:03}"), leaves)
        })
        .collect();
    TreeView::new(WidgetId::of("perf.tree"), nodes)
}

const GRID_KINDS: [CellKind; 12] = [
    CellKind::Id,
    CellKind::Text,
    CellKind::Number,
    CellKind::Bool,
    CellKind::Timestamp,
    CellKind::Json,
    CellKind::Enum,
    CellKind::Text,
    CellKind::Number,
    CellKind::Text,
    CellKind::Number,
    CellKind::Text,
];

fn grid_row(i: usize, cols: usize) -> Vec<CellValue> {
    (0..cols)
        .map(|c| match GRID_KINDS[c % GRID_KINDS.len()] {
            CellKind::Id => CellValue::Text(format!("{i:08x}-0000-4000-8000-{c:012x}")),
            CellKind::Text => CellValue::Text(format!("value {} of row {i}", (i * 7 + c) % 97)),
            CellKind::Number => CellValue::Num(((i * 31 + c) % 10_000) as f64 / 100.0),
            CellKind::Bool => CellValue::Bool(i.is_multiple_of(3)),
            CellKind::Timestamp => {
                CellValue::Text(format!("2026-09-{:02} 12:{:02}:00", i % 28 + 1, i % 60))
            }
            CellKind::Json => CellValue::Json(format!("{{\"k\":{i}}}")),
            CellKind::Enum => CellValue::Text(["pending", "paid", "shipped"][i % 3].to_owned()),
        })
        .collect()
}

fn grid_of(rows: usize, cols: usize) -> DataGrid {
    let columns = (0..cols)
        .map(|c| ColumnSpec::new(&format!("col_{c}"), GRID_KINDS[c % GRID_KINDS.len()]))
        .collect();
    let mut g = DataGrid::new(WidgetId::of("perf.grid"), columns);
    g.set_rows(GridRows {
        rows: (0..rows).map(|i| grid_row(i, cols)).collect(),
        total: RowTotal::Exact(rows),
        more: false,
    });
    g
}

fn term_line(i: usize) -> Line {
    vec![Span::plain(format!(
        "[{i:06}] lorem ipsum dolor sit amet, consectetur adipiscing elit {i}"
    ))]
}

fn viewport_of(n: usize) -> TextViewport {
    TextViewport::with_lines(
        WidgetId::of("perf.viewport"),
        (0..n).map(term_line).collect(),
    )
}

fn bg() -> ratatui::style::Color {
    Theme::junie().canvas
}

// ------------------------------------------------------------ A. frames (control)

/// Harness overhead: a `Terminal::draw` whose render does nothing. Subtract
/// this from the app-shell `frame_*` numbers to isolate application cost.
#[test]
fn frame_testbackend_empty_120x40() {
    let _g = lock();
    let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let s = bench(1, iters(200), &mut || {
        term.draw(|_f| {}).unwrap();
    });
    report("frame_testbackend_empty_120x40", &s);
}

// ------------------------------------------------------------ B. events

#[test]
fn mouse_move_over_1000_regions() {
    let _g = lock();
    let mut plain = HitRegistry::default();
    let mut barred = HitRegistry::default();
    for y in 0..10u16 {
        for x in 0..100u16 {
            let id = WidgetId::of("perf.region").child((y * 100 + x) as usize);
            plain.register(id, Rect::new(x, y, 1, 1));
            barred.register(id, Rect::new(x, y, 1, 1));
        }
    }
    barred.push_barrier();
    for i in 0..10u16 {
        barred.register(
            WidgetId::of("perf.modal").child(i as usize),
            Rect::new(i, 12, 1, 1),
        );
    }
    assert_eq!(plain.len(), 1000);
    // 10 000 probes per iteration: hits, misses (x ≥ 100 or y ≥ 10) and the
    // barrier case (everything below the barrier is unreachable)
    let s = bench(1, iters(10), &mut || {
        for i in 0..5000u32 {
            let pos = Position::new((i % 125) as u16, ((i / 125) % 20) as u16);
            black_box(plain.hit(pos));
            black_box(barred.hit(pos));
        }
    });
    report("mouse_move_over_1000_regions", &s);
    assert_eq!(s.allocs, 0, "hit-testing must not allocate");
}

#[test]
fn focus_tab_traversal_ring_200() {
    let _g = lock();
    let mut ring = FocusRing::default();
    for i in 0..200 {
        ring.register(WidgetId::of("perf.stop").child(i));
    }
    let mut cur = None;
    let s = bench(1, iters(10), &mut || {
        for _ in 0..10_000 {
            cur = ring.next(cur);
        }
        black_box(cur);
    });
    report("focus_tab_traversal_ring_200", &s);
    assert_eq!(s.allocs, 0, "focus traversal must not allocate");
}

#[test]
fn key_tree_toggle_10k() {
    let _g = lock();
    let mut tv = tree_of(big(10));
    tv.expand_all();
    let n = tv.rows().len();
    // one collapse + one expand of the first folder per iteration
    let s = bench(1, iters(100), &mut || {
        tv.toggle(0);
        tv.toggle(0);
    });
    println!("PERF-NOTE key_tree_toggle_10k rows={n} (collapse+expand pair)");
    report("key_tree_toggle_10k", &s);
}

// ------------------------------------------------------------ C. style

const STATES: [VisualState; 6] = [
    VisualState {
        focused: false,
        hovered: false,
        pressed: false,
        selected: false,
        disabled: false,
        error: false,
        editing: false,
        busy: false,
    },
    VisualState {
        focused: true,
        hovered: false,
        pressed: false,
        selected: false,
        disabled: false,
        error: false,
        editing: false,
        busy: false,
    },
    VisualState {
        focused: false,
        hovered: true,
        pressed: false,
        selected: false,
        disabled: false,
        error: false,
        editing: false,
        busy: false,
    },
    VisualState {
        focused: true,
        hovered: false,
        pressed: false,
        selected: true,
        disabled: false,
        error: false,
        editing: false,
        busy: false,
    },
    VisualState {
        focused: false,
        hovered: true,
        pressed: true,
        selected: true,
        disabled: false,
        error: false,
        editing: false,
        busy: false,
    },
    VisualState {
        focused: false,
        hovered: false,
        pressed: false,
        selected: false,
        disabled: true,
        error: false,
        editing: false,
        busy: false,
    },
];

/// 10 000 resolutions in the mix a list frame uses: row, gutter, marker fg,
/// meta fg (four parts × 2 500 elements).
fn resolve_10k(t: &Theme, overlays: &[Style]) -> u64 {
    let mut acc = 0u64;
    for i in 0..2500usize {
        let s = STATES[i % STATES.len()];
        let mut row = t.row(s, t.canvas);
        let mut gutter = t.gutter(s, row.bg.unwrap_or(t.canvas), false);
        let mut marker = row.fg(t.accent);
        let mut meta = row.fg(t.text_muted);
        for o in overlays {
            row = row.patch(*o);
            gutter = gutter.patch(*o);
            marker = marker.patch(*o);
            meta = meta.patch(*o);
        }
        acc = acc.wrapping_add(fingerprint(row) ^ fingerprint(gutter));
        acc = acc.wrapping_add(fingerprint(marker) ^ fingerprint(meta));
    }
    acc
}

fn fingerprint(s: Style) -> u64 {
    let f = s.fg.map(format_color).unwrap_or(0);
    let b = s.bg.map(format_color).unwrap_or(0);
    f ^ (b << 8) ^ ((s.add_modifier.bits() as u64) << 16)
}

fn format_color(c: ratatui::style::Color) -> u64 {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => ((r as u64) << 16) | ((g as u64) << 8) | b as u64,
        ratatui::style::Color::Indexed(i) => i as u64,
        _ => 1,
    }
}

#[test]
fn style_resolve_10k_parts() {
    let _g = lock();
    let t = Theme::junie();
    let s = bench(1, iters(10), &mut || {
        black_box(resolve_10k(&t, &[]));
    });
    report("style_resolve_10k_parts", &s);
    assert_eq!(s.allocs, 0, "style resolution must not allocate (R2)");
}

/// Today's tree has no scoped overlay stack; this measures the same mix plus
/// two `Style::patch` layers per part, the minimum an overlay must cost.
#[test]
fn style_resolve_10k_parts_with_two_overlays() {
    let _g = lock();
    let t = Theme::junie();
    let overlays = [
        Style::new().fg(t.warning),
        Style::new().add_modifier(Modifier::ITALIC),
    ];
    let base = bench(1, iters(10), &mut || {
        black_box(resolve_10k(&t, &[]));
    });
    let s = bench(1, iters(10), &mut || {
        black_box(resolve_10k(&t, &overlays));
    });
    report("style_resolve_10k_parts_with_two_overlays", &s);
    assert_eq!(s.allocs, 0, "overlay resolution must not allocate (R3)");
    check_ratio(
        "style_two_overlays_vs_none",
        s.ns,
        base.ns,
        2.0,
        env_flag("PERF_TARGET"),
    );
}

/// The modal dim walk of `Dialog::render` over a 120×39 area: 4 680
/// `Theme::backdrop` calls per iteration.
#[test]
fn style_backdrop_full_screen_120x40() {
    let _g = lock();
    let t = Theme::junie();
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    buf.set_style(*buf.area(), t.base());
    let dim = Rect::new(0, 0, 120, 39);
    let s = bench(1, iters(100), &mut || {
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                let st = t.backdrop(c.style());
                c.set_style(st);
                c.modifier = Modifier::empty();
            }
        }
    });
    report("style_backdrop_full_screen_120x40", &s);
    assert_eq!(s.allocs, 0, "backdrop must not allocate");
}

#[test]
fn style_downgrade_theme_all_levels() {
    let _g = lock();
    const LEVELS: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    let s = bench(1, iters(1000), &mut || {
        for l in LEVELS {
            black_box(Theme::for_level(l));
        }
    });
    report("style_downgrade_theme_all_levels", &s);
}

// ------------------------------------------------------------ D. large data

#[test]
fn list_100k_rows_construct() {
    let _g = lock();
    let n = big(100_000);
    let s = bench(0, iters(3), &mut || {
        black_box(list_of(n));
    });
    report("list_100k_rows_construct", &s);
}

fn list_render_frame(c: &mut Canvas, l: &mut ListBox) {
    c.frame(|area, buf, ctx| l.render(area, buf, ctx, bg()));
}

#[test]
fn list_100k_rows_render() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut big_list = list_of(big(100_000));
    let mut small = list_of(1_000);
    let s = bench(1, iters(100), &mut || {
        list_render_frame(&mut c, &mut big_list)
    });
    let (h, r) = c.regions();
    report("list_100k_rows_render", &s.with_regions(h, r));
    let ctl = bench(1, iters(100), &mut || list_render_frame(&mut c, &mut small));
    assert!(
        s.allocs < 500,
        "list frame allocates {} times (threshold 500)",
        s.allocs
    );
    check_ratio(
        "list_100k_vs_1k_render",
        s.ns,
        ctl.ns,
        1.5,
        env_flag("PERF_TARGET"),
    );
}

#[test]
fn list_1k_rows_render() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut l = list_of(1_000);
    let s = bench(1, iters(100), &mut || list_render_frame(&mut c, &mut l));
    let (h, r) = c.regions();
    report("list_1k_rows_render", &s.with_regions(h, r));
}

#[test]
fn tree_100k_nodes_flatten() {
    let _g = lock();
    let mut tv = tree_of(big(100));
    tv.expand_all();
    let n = tv.rows().len();
    // collapse the first folder (cheap) then expand it again: the expand is
    // the full-tree `flatten` the audit measures
    let s = bench(1, iters(3), &mut || {
        tv.toggle(0);
        tv.toggle(0);
    });
    println!("PERF-NOTE tree_100k_nodes_flatten rows={n} (collapse+expand pair)");
    report("tree_100k_nodes_flatten", &s);
}

#[test]
fn tree_100k_nodes_render() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut tv = tree_of(big(100));
    tv.expand_all();
    let s = bench(1, iters(100), &mut || {
        c.frame(|area, buf, ctx| tv.render(area, buf, ctx, bg()));
    });
    let (h, r) = c.regions();
    report("tree_100k_nodes_render", &s.with_regions(h, r));
}

#[test]
fn grid_500x12_render() {
    let _g = lock();
    let mut c = Canvas::new(120, 30);
    let mut g = grid_of(500, 12);
    let s = bench(1, iters(200), &mut || {
        c.frame(|area, buf, ctx| g.render(area, buf, ctx, bg()));
    });
    let (h, r) = c.regions();
    report("grid_500x12_render", &s.with_regions(h, r));
}

/// Isolated `DataGrid::set_rows` on already-converted rows (the app-level
/// three-copy load is `grid_500x12_load` in the tablepro perf tests).
#[test]
fn grid_500x12_set_rows() {
    let _g = lock();
    let mut g = grid_of(500, 12);
    let rows: Vec<Vec<CellValue>> = (0..500).map(|i| grid_row(i, 12)).collect();
    let s = bench(1, iters(20), &mut || {
        g.set_rows(GridRows {
            rows: rows.clone(),
            total: RowTotal::Exact(500),
            more: false,
        });
    });
    report("grid_500x12_set_rows", &s);
}

#[test]
fn grid_100k_local_sort() {
    let _g = lock();
    let n = big(100_000);
    let mut g = grid_of(n, 4);
    g.local_sort = true;
    // cursor column 0 is an Id column (text compare); one ascending sort
    // then a reset per iteration
    let s = bench(0, iters(2), &mut || {
        g.on_key(&key(KeyCode::Char('s')));
        g.on_key(&key(KeyCode::Char('S')));
    });
    println!("PERF-NOTE grid_100k_local_sort rows={n}");
    report("grid_100k_local_sort", &s);
}

#[test]
fn viewport_100k_lines_push() {
    let _g = lock();
    let n = big(100_000);
    let mut c = Canvas::new(80, 40);
    let mut vp = viewport_of(n);
    c.frame(|area, buf, ctx| vp.render(area, buf, ctx, bg()));
    let base_allocs = {
        let mut vp_small = viewport_of(n / 10);
        c.frame(|area, buf, ctx| vp_small.render(area, buf, ctx, bg()));
        let mut k = 0;
        let s = measure_once(&mut || {
            for _ in 0..1000 {
                vp_small.push(term_line(k));
                k += 1;
            }
            c.frame(|area, buf, ctx| vp_small.render(area, buf, ctx, bg()));
        });
        s.allocs
    };
    let mut k = n;
    // push 1 000 lines, then one frame so the layout runs (that is where
    // `ensure_layout` re-lays out the whole buffer)
    let s = bench(0, iters(1), &mut || {
        for _ in 0..1000 {
            vp.push(term_line(k));
            k += 1;
        }
        c.frame(|area, buf, ctx| vp.render(area, buf, ctx, bg()));
    });
    println!(
        "PERF-NOTE viewport_100k_lines_push lines={n} allocs_at_{}_lines={base_allocs}",
        n / 10
    );
    report("viewport_100k_lines_push", &s);
    if env_flag("PERF_TARGET") {
        // allocations must be independent of `lines.len()`: same ±10 %
        let hi = base_allocs + base_allocs / 10;
        assert!(
            s.allocs <= hi,
            "push cost scales with buffer size: {} vs {base_allocs}",
            s.allocs
        );
    }
}

/// Steady-state frames with no pushes. Today every frame of an overflowing
/// viewport re-lays out the whole buffer twice (`render` calls
/// `ensure_layout(area.width)` and then `ensure_layout(area.width - 1)` for
/// the scrollbar, so `layout_width` never matches on the next frame), which
/// is why the iteration count is small.
#[test]
fn viewport_100k_lines_render() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut vp = viewport_of(big(100_000));
    let s = bench(1, iters(3), &mut || {
        c.frame(|area, buf, ctx| vp.render(area, buf, ctx, bg()));
    });
    let (h, r) = c.regions();
    report("viewport_100k_lines_render", &s.with_regions(h, r));
}

/// Isolates `pane.term.clone()` (`capsule.rs:1567`): four laid-out 2 000-line
/// viewports cloned once each. Delete this test when the clone is gone.
#[test]
fn capsule_pane_clone_4x2000() {
    let _g = lock();
    let mut c = Canvas::new(120, 40);
    let mut panes: Vec<TextViewport> = (0..4).map(|_| viewport_of(2000)).collect();
    for p in panes.iter_mut() {
        c.frame(|_area, buf, ctx| p.render(Rect::new(0, 0, 58, 18), buf, ctx, bg()));
    }
    let s = bench(1, iters(3), &mut || {
        for p in &panes {
            black_box(p.clone());
        }
    });
    report("capsule_pane_clone_4x2000", &s);
}

// ------------------------------------------------------------ E. unicode

#[test]
fn width_10k_grapheme_line() {
    let _g = lock();
    let line = perf_common::unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(text::width(&line));
    });
    report("width_10k_grapheme_line", &s);
    assert_eq!(s.allocs, 0);
}

#[test]
fn truncate_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = perf_common::unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(text::truncate(&line, 80));
    });
    report("truncate_10k_grapheme_line_to_80", &s);
}

#[test]
fn fit_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = perf_common::unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(text::fit(&line, 80));
    });
    report("fit_10k_grapheme_line_to_80", &s);
    if env_flag("PERF_TARGET") {
        assert_eq!(s.allocs, 0, "R5: the row painter must not allocate");
    }
}

#[test]
fn truncate_middle_10k_to_40() {
    let _g = lock();
    let line = perf_common::unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(text::truncate_middle(&line, 40));
    });
    report("truncate_middle_10k_to_40", &s);
}

#[test]
fn wrap_10k_graphemes_to_80() {
    let _g = lock();
    // words so the wrapper has boundaries to break on
    let raw = perf_common::unicode_line(10_000);
    let mut line = String::with_capacity(raw.len() + raw.len() / 8);
    for (i, g) in
        unicode_segmentation::UnicodeSegmentation::graphemes(raw.as_str(), true).enumerate()
    {
        if i > 0 && i.is_multiple_of(9) {
            line.push(' ');
        }
        line.push_str(g);
    }
    let s = bench(3, iters(200), &mut || {
        black_box(text::wrap(&line, 80));
    });
    report("wrap_10k_graphemes_to_80", &s);
}

#[test]
fn textbuffer_pos_of_10k_line() {
    let _g = lock();
    let mut doc = String::new();
    for i in 0..20 {
        doc.push_str(&format!("line {i}\n"));
    }
    doc.push_str(&perf_common::unicode_line(10_000));
    let off = doc.len();
    let s = bench(10, iters(1000), &mut || {
        black_box(TextBuffer::pos_of(&doc, off));
    });
    report("textbuffer_pos_of_10k_line", &s);
    assert_eq!(s.allocs, 0);
}

#[test]
fn textbuffer_offset_at_10k_line() {
    let _g = lock();
    let mut doc = String::new();
    for i in 0..20 {
        doc.push_str(&format!("line {i}\n"));
    }
    doc.push_str(&perf_common::unicode_line(10_000));
    let tb = TextBuffer::multi(doc);
    let s = bench(10, iters(1000), &mut || {
        black_box(tb.offset_at(20, 12_000));
    });
    report("textbuffer_offset_at_10k_line", &s);
    assert_eq!(s.allocs, 0);
}

/// `TextViewport::ensure_layout` for one 10 000-grapheme line, forced by
/// alternating the layout width (80 ↔ 81 cells).
#[test]
fn viewport_layout_10k_grapheme_line() {
    let _g = lock();
    let mut c = Canvas::new(100, 3);
    let mut vp = TextViewport::with_lines(
        WidgetId::of("perf.viewport.one"),
        vec![vec![Span::plain(perf_common::unicode_line(10_000))]],
    );
    let mut flip = false;
    let s = bench(2, iters(20), &mut || {
        flip = !flip;
        let w = if flip { 82 } else { 83 };
        c.frame(|_area, buf, ctx| vp.render(Rect::new(0, 0, w, 3), buf, ctx, bg()));
    });
    report("viewport_layout_10k_grapheme_line", &s);
    if env_flag("PERF_TARGET") {
        assert_eq!(s.allocs, 0, "layout must not allocate per grapheme");
    }
}

// ------------------------------------------------------------ F. invariants

#[test]
fn render_twice_allocates_the_same() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut l = list_of(1_000);
    list_render_frame(&mut c, &mut l);
    let a = measure_once(&mut || list_render_frame(&mut c, &mut l));
    let b = measure_once(&mut || list_render_frame(&mut c, &mut l));
    println!(
        "PERF render_twice_allocates_the_same ns={} allocs={} bytes={}",
        b.ns, b.allocs, b.bytes
    );
    assert_eq!(
        a.allocs, b.allocs,
        "identical frames must allocate identically"
    );
    assert_eq!(a.bytes, b.bytes);
}

/// §25.6: no path may copy a whole data set per frame. Expected to fail on
/// the pre-refactor tree for the viewport; the assertion is gated behind
/// `PERF_TARGET=1` and the measurement is always printed.
#[test]
fn no_full_collection_clone_per_frame() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut l = list_of(big(100_000));
    let ls = bench(1, iters(20), &mut || list_render_frame(&mut c, &mut l));
    let mut vp = viewport_of(big(100_000));
    let vs = bench(1, iters(2), &mut || {
        c.frame(|area, buf, ctx| vp.render(area, buf, ctx, bg()));
    });
    println!(
        "PERF no_full_collection_clone_per_frame_list ns={} allocs={} bytes={}",
        ls.ns, ls.allocs, ls.bytes
    );
    println!(
        "PERF no_full_collection_clone_per_frame_viewport ns={} allocs={} bytes={}",
        vs.ns, vs.allocs, vs.bytes
    );
    if env_flag("PERF_TARGET") {
        assert!(ls.bytes < 64 * 1024, "list frame copies {} bytes", ls.bytes);
        assert!(
            vs.bytes < 64 * 1024,
            "viewport frame copies {} bytes",
            vs.bytes
        );
    }
}

/// One click into a 100 000-row list versus a 100-row list: same cost.
/// Hard assertions gated behind `PERF_TARGET=1`; numbers always printed.
#[test]
fn event_dispatch_is_not_o_n() {
    let _g = lock();
    let mut c = Canvas::new(80, 40);
    let mut click = |l: &mut ListBox, name: &str| {
        list_render_frame(&mut c, l);
        let pos = Position::new(10, 5);
        let s = bench(1, iters(1000), &mut || {
            let id = c.hits.hit(pos).expect("row under the pointer");
            let row = l.locate(id).expect("row belongs to the list");
            black_box(l.on_click(row));
        });
        println!(
            "PERF event_dispatch_is_not_o_n_{name} ns={} allocs={} bytes={}",
            s.ns, s.allocs, s.bytes
        );
        s
    };
    let big_l = click(&mut list_of(big(100_000)), "100k");
    let small = click(&mut list_of(100), "100");
    let target = env_flag("PERF_TARGET");
    if target {
        assert_eq!(big_l.allocs, 0, "a click must not allocate");
    }
    check_ratio(
        "event_dispatch_100k_vs_100",
        big_l.ns,
        small.ns,
        3.0,
        target,
    );
}

// `hit_registry_size_is_bounded` is enforced inside `perf_common::report`:
// every benchmark that records `hits=` is checked against its baseline
// ±10 % in release builds. `debug_and_release_alloc_counts_match` lives in
// `src/bin/tablepro/perf_tests.rs` next to the grid frame it compares.
