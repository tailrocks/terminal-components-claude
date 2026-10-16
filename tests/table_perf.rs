//! Scroll performance of the table-like widgets (DataGrid, DataTable,
//! ListBox, TreeView) rendered alone into a buffer, at small and large row
//! counts (see docs/audits/design-review-2026-10/performance.md).
//!
//! `measure_widgets` is an ignored measurement that prints timings; run it
//! in the release profile:
//!
//! `cargo nextest run --cargo-profile release --test table_perf --run-ignored only --no-capture`

use std::time::{Duration, Instant};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::tree::{TreeNode, TreeView};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Rect};

const COLS: usize = 11;

/// Rows whose columns 8 and 9 carry `blob` bytes of JSON and text: cut
/// cells whose full value is far wider than any viewport.
fn grid_rows_with(n: usize, blob: usize) -> Vec<Vec<CellValue>> {
    let json = format!("{{\"k\": [{}]}}", "1, ".repeat(blob / 3));
    let text = "lorem ipsum ".repeat(blob / 12);
    (0..n)
        .map(|r| {
            (0..COLS)
                .map(|c| match c {
                    0 => CellValue::Int(r as i64),
                    3 => CellValue::Num(r as f64 * 1.5),
                    5 => CellValue::Bool(r % 2 == 0),
                    7 if r % 7 == 0 => CellValue::Null,
                    8 if blob > 0 => CellValue::Json(json.clone()),
                    9 if blob > 0 => CellValue::Text(text.clone()),
                    _ => CellValue::Text(format!("row {r} column {c} · some text")),
                })
                .collect()
        })
        .collect()
}

fn grid(n: usize) -> DataGrid {
    grid_with(n, 0)
}

fn grid_with(n: usize, blob: usize) -> DataGrid {
    let cols = (0..COLS)
        .map(|c| {
            let kind = match c {
                0 => CellKind::Id,
                3 => CellKind::Number,
                5 => CellKind::Bool,
                _ => CellKind::Text,
            };
            ColumnSpec::new(&format!("column_{c}"), kind)
        })
        .collect();
    let mut g = DataGrid::new(WidgetId::of("perf-grid"), cols);
    g.set_rows(GridRows {
        rows: grid_rows_with(n, blob),
        total: RowTotal::Exact(n),
        more: false,
    });
    g
}

fn table(n: usize) -> DataTable {
    table_with(n, 0)
}

fn table_with(n: usize, blob: usize) -> DataTable {
    let text = "lorem ipsum ".repeat(blob / 12);
    let cols = (0..COLS)
        .map(|c| Column::new(&format!("column_{c}"), Constraint::Min(12)))
        .collect();
    let rows = (0..n)
        .map(|r| {
            (0..COLS)
                .map(|c| {
                    if blob > 0 && c >= 8 {
                        Cell::new(text.clone())
                    } else {
                        Cell::new(format!("row {r} column {c}"))
                    }
                })
                .collect()
        })
        .collect();
    DataTable::new(WidgetId::of("perf-table"), cols, rows)
}

fn list(n: usize) -> ListBox {
    let items = (0..n)
        .map(|i| ListItem::new(&format!("item {i} with a label")).meta("meta"))
        .collect();
    ListBox::new(WidgetId::of("perf-list"), items, SelectMode::Single)
}

fn tree(n: usize) -> TreeView {
    // one expanded folder per 100 leaves
    let dirs = (0..n.div_ceil(100))
        .map(|d| {
            TreeNode::dir(
                &format!("folder {d}"),
                (0..100)
                    .map(|l| TreeNode::leaf(&format!("leaf {d}.{l}")))
                    .collect(),
            )
        })
        .collect();
    TreeView::new(WidgetId::of("perf-tree"), dirs)
}

trait Surface {
    fn id(&self) -> WidgetId;
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);
    fn key(&mut self, key: &Key) -> Outcome;
    fn wheel(&mut self, delta: i32) -> Outcome;
}

impl Surface for DataGrid {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let bg = ctx.theme.surface;
        self.render(area, buf, ctx, bg);
    }
    fn key(&mut self, key: &Key) -> Outcome {
        self.on_key(key).0
    }
    fn wheel(&mut self, delta: i32) -> Outcome {
        self.on_wheel(delta, false)
    }
}

impl Surface for DataTable {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let bg = ctx.theme.surface;
        self.render(area, buf, ctx, bg);
    }
    fn key(&mut self, key: &Key) -> Outcome {
        self.on_key(key).0
    }
    fn wheel(&mut self, delta: i32) -> Outcome {
        self.on_wheel(delta)
    }
}

impl Surface for ListBox {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let bg = ctx.theme.surface;
        self.render(area, buf, ctx, bg);
    }
    fn key(&mut self, key: &Key) -> Outcome {
        self.on_key(key)
    }
    fn wheel(&mut self, delta: i32) -> Outcome {
        self.on_wheel(delta)
    }
}

impl Surface for TreeView {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let bg = ctx.theme.surface;
        self.render(area, buf, ctx, bg);
    }
    fn key(&mut self, key: &Key) -> Outcome {
        self.on_key(key).0
    }
    fn wheel(&mut self, delta: i32) -> Outcome {
        self.on_wheel(delta)
    }
}

/// One frame: a fresh hit registry and focus ring, as the app does.
fn frame(s: &mut impl Surface, area: Rect, theme: &Theme) -> HitRegistry {
    let mut buf = Buffer::empty(area);
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let interaction = Interaction {
        focus: Some(s.id()),
        ..Default::default()
    };
    let mut ctx = RenderCtx::new(theme, interaction, &mut hits, &mut ring);
    s.draw(area, &mut buf, &mut ctx);
    drop(ctx);
    hits
}

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn time(f: impl FnOnce()) -> Duration {
    let t = Instant::now();
    f();
    t.elapsed()
}

fn measure(label: &str, n: usize, build: Duration, s: &mut impl Surface) {
    let theme = Theme::junie();
    for (w, h) in [(120u16, 40u16), (200, 55)] {
        let area = Rect::new(0, 0, w, h);
        frame(s, area, &theme);
        let frames = 60;
        let mut draw = Duration::ZERO;
        let mut handle = Duration::ZERO;
        for i in 0..frames {
            handle += time(|| {
                match i % 3 {
                    0 => s.wheel(3),
                    1 => s.key(&key(KeyCode::PageDown)),
                    _ => s.key(&key(KeyCode::Down)),
                };
            });
            draw += time(|| {
                frame(s, area, &theme);
            });
        }
        let end = time(|| {
            s.key(&key(KeyCode::End));
            frame(s, area, &theme);
        });
        s.key(&key(KeyCode::Home));
        println!(
            "{label:<10} {n:>7} rows {w}x{h:<3} build {:>9.2} ms  handle {:>6.2} µs  frame {:>8.1} µs  End+frame {:>8.1} µs",
            build.as_secs_f64() * 1e3,
            handle.as_secs_f64() * 1e6 / frames as f64,
            draw.as_secs_f64() * 1e6 / frames as f64,
            end.as_secs_f64() * 1e6,
        );
    }
}

#[test]
#[ignore = "measurement: prints timings, asserts nothing"]
fn measure_widgets() {
    for n in [500, 10_000, 100_000] {
        let mut g = None;
        let b = time(|| g = Some(grid(n)));
        measure("DataGrid", n, b, g.as_mut().unwrap());
        let mut t = None;
        let b = time(|| t = Some(table(n)));
        measure("DataTable", n, b, t.as_mut().unwrap());
        let mut l = None;
        let b = time(|| l = Some(list(n)));
        measure("ListBox", n, b, l.as_mut().unwrap());
        let mut tr = None;
        let b = time(|| tr = Some(tree(n)));
        measure("TreeView", n, b, tr.as_mut().unwrap());
    }
    // cut cells holding long values (JSON documents, long text)
    for blob in [4_096, 65_536] {
        let mut g = None;
        let b = time(|| g = Some(grid_with(2_000, blob)));
        measure(
            &format!("Grid {}K", blob / 1024),
            2_000,
            b,
            g.as_mut().unwrap(),
        );
        let mut t = None;
        let b = time(|| t = Some(table_with(2_000, blob)));
        measure(
            &format!("Table {}K", blob / 1024),
            2_000,
            b,
            t.as_mut().unwrap(),
        );
    }
}
