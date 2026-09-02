//! Data grid: typed cells, a pending-change queue with preview / discard /
//! save, fetch-more paging, local sort, and row-level errors from a commit.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::grid::{
    CellKind, CellValue, ColumnSpec, DataGrid, GridEvent, GridRows, RowTotal,
};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::Prop;

const ID: WidgetId = WidgetId::of("grid");
const PREVIEW: WidgetId = ID.sub("preview");
const PAGE: usize = 40;
const ALL: usize = 96;

const NAMES: &[&str] = &[
    "Northwind Traders",
    "Blue Yonder Airlines",
    "Contoso Pharmaceuticals",
    "Fabrikam Robotics",
    "Litware Analytics",
    "Tailspin Toys",
    "Wide World Importers",
    "Adventure Works",
    "Proseware Studio",
    "Woodgrove Bank",
    "Alpine Ski House",
    "Coho Winery",
    "Lucerne Publishing",
    "Margie's Travel",
    "Trey Research",
    "Humongous Insurance",
];
const PLANS: &[&str] = &["free", "pro", "team", "enterprise"];
const OWNERS: &[&str] = &["mira", "jonas", "ana", "kai"];

fn row(i: usize) -> Vec<CellValue> {
    let plan = PLANS[(i * 7 + 3) % PLANS.len()];
    let seats = [1, 3, 5, 12, 25, 40, 80, 150][(i * 5 + 1) % 8];
    let mrr = match plan {
        "free" => 0.0,
        "pro" => 29.0 * seats as f64,
        "team" => 24.0 * seats as f64,
        _ => 19.0 * seats as f64,
    };
    let suffix = if i >= NAMES.len() {
        format!(" {}", i / NAMES.len() + 1)
    } else {
        String::new()
    };
    let renewed = if plan == "free" {
        CellValue::Null
    } else {
        CellValue::Text(format!("2026-{:02}-{:02}", 1 + i % 12, 1 + (i * 3) % 28))
    };
    let notes = if i.is_multiple_of(4) {
        CellValue::Json(format!(
            "{{\"owner\":\"{}\",\"seats\":{seats}}}",
            OWNERS[i % OWNERS.len()]
        ))
    } else {
        CellValue::Null
    };
    vec![
        CellValue::Int(1001 + i as i64),
        CellValue::Text(format!("{}{suffix}", NAMES[i % NAMES.len()])),
        CellValue::Text(plan.to_owned()),
        CellValue::Int(seats),
        CellValue::Num(mrr),
        CellValue::Bool(i % 5 != 3),
        renewed,
        notes,
    ]
}

fn page(from: usize) -> GridRows {
    let to = (from + PAGE).min(ALL);
    GridRows {
        rows: (from..to).map(row).collect(),
        total: if to >= ALL {
            RowTotal::Exact(ALL)
        } else {
            RowTotal::Estimated(4_812)
        },
        more: to < ALL,
    }
}

fn literal(v: &CellValue) -> String {
    match v {
        CellValue::Null => "NULL".into(),
        CellValue::Default => "DEFAULT".into(),
        CellValue::Text(s) | CellValue::Json(s) => format!("'{}'", s.replace('\'', "''")),
        other => other.text(),
    }
}

pub struct GridPage {
    grid: DataGrid,
    commit_ticks: u8,
    saved: u32,
}

impl GridPage {
    pub fn new() -> Self {
        let columns = vec![
            ColumnSpec::new("id", CellKind::Id)
                .primary()
                .read_only()
                .nullable(false)
                .type_label("integer"),
            ColumnSpec::new("customer", CellKind::Text)
                .nullable(false)
                .type_label("text"),
            ColumnSpec::new("plan", CellKind::Enum)
                .enum_values(PLANS)
                .nullable(false),
            ColumnSpec::new("seats", CellKind::Number)
                .nullable(false)
                .type_label("integer"),
            ColumnSpec::new("mrr", CellKind::Number)
                .read_only()
                .type_label("numeric(10,2)"),
            ColumnSpec::new("active", CellKind::Bool).nullable(false),
            ColumnSpec::new("renewed_at", CellKind::Timestamp).type_label("date"),
            ColumnSpec::new("notes", CellKind::Json),
        ];
        let mut grid = DataGrid::new(ID.sub("grid"), columns).editable(true);
        grid.local_sort = true;
        grid.set_rows(page(0));
        Self {
            grid,
            commit_ticks: 0,
            saved: 0,
        }
    }

    fn statements(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut cells: Vec<_> = self.grid.pending.cells.iter().collect();
        cells.sort_by_key(|((r, c), _)| (*r, *c));
        for ((r, c), v) in cells {
            if self.grid.pending.inserted.contains(r) {
                continue;
            }
            out.push(format!(
                "UPDATE customers SET {} = {} WHERE id = {};",
                self.grid.columns[*c].name,
                literal(v),
                self.grid.value(*r, 0).text()
            ));
        }
        for r in &self.grid.pending.inserted {
            let cols: Vec<&str> = self
                .grid
                .columns
                .iter()
                .enumerate()
                .filter(|(c, _)| self.grid.pending.cells.contains_key(&(*r, *c)))
                .map(|(_, col)| col.name.as_str())
                .collect();
            if cols.is_empty() {
                out.push("INSERT INTO customers DEFAULT VALUES;".into());
            } else {
                let vals: Vec<String> = self
                    .grid
                    .columns
                    .iter()
                    .enumerate()
                    .filter_map(|(c, _)| self.grid.pending.cells.get(&(*r, c)))
                    .map(literal)
                    .collect();
                out.push(format!(
                    "INSERT INTO customers ({}) VALUES ({});",
                    cols.join(", "),
                    vals.join(", ")
                ));
            }
        }
        for r in &self.grid.pending.deleted {
            out.push(format!(
                "DELETE FROM customers WHERE id = {};",
                self.grid.value(*r, 0).text()
            ));
        }
        out
    }

    fn on_event(&mut self, ev: Option<GridEvent>, cx: &mut PageCtx) {
        match ev {
            Some(GridEvent::CellChanged { .. }) => {
                cx.status(format!("{} pending", self.grid.pending.total()))
            }
            Some(GridEvent::RowInserted(_)) => cx.status("Row inserted · fill it in, then Save"),
            Some(GridEvent::RowDeleted(_)) => cx.status("Row queued for deletion · u undoes"),
            Some(GridEvent::FetchMore) => {
                let from = self.grid.len();
                self.grid.append_rows(page(from));
                cx.status(format!("Fetched rows {}–{}", from + 1, self.grid.len()));
            }
            Some(GridEvent::Refresh) => {
                self.grid.set_rows(page(0));
                cx.status("Reloaded from the source");
            }
            Some(GridEvent::CommitRequested) => {
                if self.grid.pending.is_empty() {
                    cx.status("Nothing to save");
                } else {
                    self.grid.set_loading(true);
                    self.commit_ticks = 4;
                    cx.status("Saving…");
                }
            }
            Some(GridEvent::DiscardRequested) => {
                self.grid.discard();
                cx.status("Changes discarded");
            }
            Some(GridEvent::PreviewSql) => {
                let code = self.statements();
                let (changed, inserted, deleted) = self.grid.pending.counts();
                cx.open(Dialog::facts(
                    PREVIEW,
                    "Pending changes",
                    vec![
                        Prop::new("Statements", code.len().to_string()),
                        Prop::new(
                            "Rows",
                            format!("{changed} changed · {inserted} inserted · {deleted} deleted"),
                        ),
                        Prop::new("Target", "customers"),
                    ],
                    code,
                    None,
                    Button::primary(PREVIEW.sub("ok"), "Close"),
                ));
            }
            Some(GridEvent::Copy(s)) => cx.status(format!("Copied {} chars", s.len())),
            Some(GridEvent::FollowReference { row, .. }) => {
                cx.status(format!("Would follow the reference on row {}", row + 1))
            }
            Some(GridEvent::OpenViewer { row, col }) => cx.status(format!(
                "Would open the viewer for {} on row {}",
                self.grid.columns[col].name,
                row + 1
            )),
            Some(GridEvent::FilterOnCell { col, value }) => cx.status(format!(
                "Would filter {} = {}",
                self.grid.columns[col].name,
                value.text()
            )),
            Some(GridEvent::OpenFilters) => cx.status("The filter editor belongs to the app"),
            Some(GridEvent::ClearFilters) => cx.status("No filters to clear"),
            Some(GridEvent::Activated(r)) => cx.status(format!("Row {} activated", r + 1)),
            Some(GridEvent::LeaveForward) => cx.focus_next(),
            Some(GridEvent::LeaveBackward) => cx.focus_prev(),
            Some(GridEvent::SortRequested(_)) | None => {}
        }
    }

    fn finish_commit(&mut self, cx: &mut PageCtx) {
        self.grid.set_loading(false);
        // the "server" rejects seat counts above the plan limit
        let bad = self
            .grid
            .pending
            .cells
            .iter()
            .find(|((_, c), v)| *c == 3 && matches!(v, CellValue::Int(n) if *n > 500))
            .map(|((r, _), _)| *r);
        match bad {
            Some(r) => {
                self.grid
                    .apply_commit_result(Err((r, "seats above the plan limit (500)".into())));
                cx.status("Save failed · the row is marked");
            }
            None => {
                let n = self.grid.pending.total();
                self.grid.apply_commit_result(Ok(()));
                self.saved += n as u32;
                cx.status(format!("Saved {n} changes"));
            }
        }
    }
}

impl Page for GridPage {
    fn title(&self) -> &'static str {
        "Data grid"
    }
    fn blurb(&self) -> &'static str {
        "Typed cells, a pending-change queue, paging and local sort"
    }
    fn editing(&self) -> bool {
        self.grid.is_editing()
    }
    fn animating(&self) -> bool {
        self.commit_ticks > 0
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.grid.id)
            || self
                .grid
                .bar_ids()
                .iter()
                .any(|b| ctx.interaction.focused(*b));
        let meta = self.grid.position_label();
        let panel = Panel::card(Some("customers")).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        let h = area.height.min(30);
        let inner = panel.render(Rect::new(area.x, area.y, area.width, h), buf, t);
        self.grid.render(inner, buf, ctx, bg);
        let y = area.y + h + 1;
        if y < area.bottom() {
            let line = format!(
                "p previews SQL · Ctrl+S saves · seats over 500 are rejected on save · saved so far: {}",
                self.saved
            );
            buf.set_string(
                area.x + 2,
                y,
                junie_tui::ui::text::truncate(&line, area.width.saturating_sub(2) as usize),
                t.muted().bg(t.canvas),
            );
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if self.commit_ticks == 0 {
                    return Outcome::Ignored;
                }
                self.commit_ticks -= 1;
                if self.commit_ticks == 0 {
                    self.finish_commit(cx);
                }
                Outcome::Changed
            }
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.grid.id {
                    let (o, ev) = self.grid.on_key(key);
                    self.on_event(ev, cx);
                    return o;
                }
                if self.grid.bar_ids().contains(&f) {
                    let (o, ev) = self.grid.on_bar_key(f, key);
                    self.on_event(ev, cx);
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Paste(text) => self.grid.on_paste(text),
            PageEvent::Click { id, pos } => {
                if !self.grid.owns(*id) {
                    return Outcome::Ignored;
                }
                if self.grid.bar_ids().contains(id) {
                    cx.focus.focus(*id);
                } else {
                    cx.focus.focus(self.grid.id);
                }
                let (o, ev) = self.grid.on_click(*id, *pos);
                self.on_event(ev, cx);
                o
            }
            PageEvent::Drag { pressed, pos } if self.grid.owns(*pressed) => {
                self.grid.on_drag(*pressed, *pos)
            }
            PageEvent::Wheel { id, delta } if self.grid.owns(*id) => {
                self.grid.on_wheel(*delta, false)
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.grid.is_editing() {
            return vec![("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")];
        }
        if focus.is_some_and(|f| self.grid.bar_ids().contains(&f)) {
            return vec![("Enter", "Activate"), ("Tab", "Next")];
        }
        vec![
            ("↑↓←→", "Cell"),
            ("Enter", "Edit"),
            ("s", "Sort"),
            ("Space", "Select row"),
            ("+ -", "Insert / delete"),
            ("u", "Undo"),
        ]
    }
}
