use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::data::TaskStatus;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::table::{Cell, Column, DataTable, TableEvent, Tone};

const ID: WidgetId = WidgetId::of("tables");

pub fn task_columns() -> Vec<Column> {
    vec![
        Column::new("ID", Constraint::Length(5)),
        Column::new("Task", Constraint::Min(24)),
        Column::new("Owner", Constraint::Length(7)),
        Column::new("Status", Constraint::Length(9)),
        Column::new("Branch", Constraint::Length(20)),
        Column::new("Changes", Constraint::Length(9)).right(),
        Column::new("Duration", Constraint::Length(9)).right(),
    ]
}

pub fn status_cell(s: TaskStatus) -> Cell {
    // status is text + tone; green is not spent on rows
    let (tone, text) = match s {
        TaskStatus::Running => (Tone::Normal, "▸ Running"),
        TaskStatus::Failed => (Tone::Error, "Failed"),
        TaskStatus::Paused => (Tone::Warning, "Paused"),
        TaskStatus::Queued => (Tone::Muted, "Queued"),
        TaskStatus::Done => (Tone::Secondary, "Done"),
    };
    Cell::new(text).tone(tone)
}

pub fn task_rows() -> Vec<Vec<Cell>> {
    crate::data::tasks()
        .into_iter()
        .map(|r| {
            let dur = if r.duration_s == 0 {
                "0s".to_owned()
            } else if r.duration_s >= 60 {
                format!("{}m {:02}s", r.duration_s / 60, r.duration_s % 60)
            } else {
                format!("{}s", r.duration_s)
            };
            vec![
                Cell::new(format!("#{}", r.id)).tone(Tone::Muted),
                Cell::new(r.name),
                Cell::new(r.owner),
                status_cell(r.status),
                Cell::new(r.branch).tone(Tone::Muted),
                Cell::new(r.changes.to_string()).tone(if r.changes == 0 {
                    Tone::Muted
                } else {
                    Tone::Normal
                }),
                Cell::new(dur),
            ]
        })
        .collect()
}

pub struct TablesPage {
    table: DataTable,
    empty: DataTable,
}

impl TablesPage {
    pub fn new() -> Self {
        let table =
            DataTable::new(ID.sub("tasks"), task_columns(), task_rows()).numeric(&[0, 5, 6]);
        let empty = DataTable::new(
            ID.sub("empty"),
            vec![
                Column::new("Check", Constraint::Min(12)),
                Column::new("Result", Constraint::Length(8)),
            ],
            vec![],
        )
        .empty_text("No checks have run yet");
        Self { table, empty }
    }
}

impl Page for TablesPage {
    fn title(&self) -> &'static str {
        "Tables"
    }
    fn blurb(&self) -> &'static str {
        "Sort by header, hover rows, select with Enter, overflow scrolls"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[area.height.saturating_sub(9), 1, 0]);
        let pos = scrollbar::position_label(&self.table.scroll);
        let sort = match self.table.sort {
            Some((c, d)) => format!(
                "sorted by {} {}",
                self.table.columns[c].title.to_lowercase(),
                if d == junie_tui::widgets::table::SortDir::Asc {
                    "▴"
                } else {
                    "▾"
                }
            ),
            None => "unsorted".to_owned(),
        };
        let meta = if pos.is_empty() {
            sort
        } else {
            format!("{sort} · {pos}")
        };
        let panel = Panel::card(Some("Tasks"))
            .meta(&meta)
            .focused(ctx.interaction.focused(self.table.id));
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        self.table.render(inner, buf, ctx, bg);

        let panel = Panel::card(Some("Checks"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[2], buf, t);
        self.empty.render(inner, buf, ctx, bg);
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.table.id) {
                    let (out, tev) = self.table.on_key(key);
                    if let Some(TableEvent::Activated(row)) = tev {
                        cx.status(format!("Selected {}", self.table.rows[row][1].text));
                    }
                    return out;
                }
                if cx.focus.is(self.empty.id) {
                    return self.empty.on_key(key).0;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, pos } => {
                if let Some(c) = self.table.locate_header(*id) {
                    cx.focus.focus(self.table.id);
                    return self.table.on_click_header(c);
                }
                if let Some((row, col)) = self.table.locate(*id) {
                    cx.focus.focus(self.table.id);
                    let (out, tev) = self.table.on_click_cell(row, col.unwrap_or(0), *pos);
                    if let Some(TableEvent::Activated(r)) = tev {
                        cx.status(format!("Selected {}", self.table.rows[r][1].text));
                    }
                    return out;
                }
                if *id == scrollbar::id_for(self.table.id) {
                    return self.table.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } if *pressed == scrollbar::id_for(self.table.id) => {
                self.table.on_scrollbar(*pos)
            }
            PageEvent::Wheel { id, delta } if self.table.owns(*id) => self.table.on_wheel(*delta),
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![
            ("↑ ↓", "Move"),
            ("← →", "Columns"),
            ("s", "Sort column"),
            ("Enter", "Select"),
        ]
    }
}
