use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::data::TaskStatus;
use crate::pages::row_menu::{MENU_HINTS, OPEN_HINT, RowMenu, plain};
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::widgets::menu::{ContextMenu, MenuItem};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::table::{Cell, Column, DataTable, SortDir, TableAction, Tone};
use ratatui::crossterm::event::KeyCode;

const ID: WidgetId = WidgetId::of("tables");
const MENU: WidgetId = WidgetId::of("tables.menu");
const HEADER_MENU: WidgetId = WidgetId::of("tables.header-menu");

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
    sections: crate::pages::layout::Sections,
    /// The task table's cell menu and the `(display row, column)` it is
    /// about.
    menu: RowMenu<(usize, usize)>,
    /// The task table's header menu (sort, fit, reset widths) and the
    /// column it is about.
    header_menu: RowMenu<usize>,
}

/// What the cell menu's items do.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CellAction {
    Select,
    Sort,
    Edit,
}

impl TablesPage {
    /// What cell `(row, col)` of the task table offers: only what the
    /// table's keys and header clicks already do.
    fn cell_items(&self, (row, col): (usize, usize)) -> Vec<(CellAction, MenuItem)> {
        let t = &self.table;
        let src = t.source_row(row);
        let select = MenuItem::new("Select").shortcut("Enter");
        let select = if t.is_disabled(src) {
            select.reason("Unavailable")
        } else if t.selected == Some(src) {
            select.reason("Already selected")
        } else {
            select
        };
        let column = &t.columns[col];
        let name = column.title.to_lowercase();
        let sort = match t.sort {
            Some((c, SortDir::Asc)) if c == col => MenuItem::new(format!("Sort {name} descending")),
            Some((c, SortDir::Desc)) if c == col => MenuItem::new("Clear sort"),
            _ => MenuItem::new(format!("Sort by {name}")),
        };
        // `s` sorts the cursor column; a header click any column
        let sort = if !column.sortable {
            sort.reason("Not sortable")
        } else if col == t.cursor_col {
            sort.shortcut("s")
        } else {
            sort
        };
        let edit = MenuItem::new("Edit cell");
        // F2 edits the cursor cell of a cell-navigating table
        let edit = if column.editable && t.cell_nav && col == t.cursor_col {
            edit.shortcut("F2")
        } else {
            edit.reason("Read-only")
        };
        vec![
            (CellAction::Select, select.separator()),
            (CellAction::Sort, sort.separator()),
            (CellAction::Edit, edit),
        ]
    }

    fn open_menu(&mut self, req: ContextRequest<(usize, usize)>) {
        let src = self.table.source_row(req.target.0);
        let title = self.table.rows[src][1].text.clone();
        let items = self
            .cell_items(req.target)
            .into_iter()
            .map(|(_, m)| m)
            .collect();
        self.menu
            .open(req.menu(MENU, items).title(title), req.target);
    }

    /// Item `i` of the menu about `cell`: the same path the item's key or
    /// header click takes.
    fn choose(&mut self, cell: (usize, usize), i: usize, cx: &mut PageCtx) -> Outcome {
        let Some(&(action, _)) = self.cell_items(cell).get(i) else {
            return Outcome::Consumed;
        };
        match action {
            CellAction::Select => {
                self.table.cursor_row = cell.0;
                let (out, tev) = self.table.on_key(&plain(KeyCode::Enter));
                if let Some(TableAction::Activated(row)) = tev {
                    cx.status(format!("Selected {}", self.table.rows[row][1].text));
                }
                out
            }
            CellAction::Sort => self.table.on_click_header(cell.1),
            CellAction::Edit => self.table.begin_edit(),
        }
    }
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
        Self {
            table,
            empty,
            sections: crate::pages::layout::Sections::new(ID.sub("page")),
            menu: RowMenu::default(),
            header_menu: RowMenu::default(),
        }
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
        // the table keeps at least four rows; on a short terminal the page
        // scrolls to the checks instead of squeezing the table to one row
        let focus = if ctx.interaction.has_focus(self.table.id) {
            Some(0)
        } else if ctx.interaction.has_focus(self.empty.id) {
            Some(1)
        } else {
            None
        };
        let l = self.sections.layout(
            area,
            &[
                StackChild::flex(9, area.height.saturating_sub(9).max(9)),
                StackChild::fixed(8),
            ],
            focus,
            buf,
            ctx,
        );
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
        let panel = Panel::card(Some("Tasks")).focused(ctx.interaction.focused(self.table.id));
        let bg = panel.bg(t);
        match l.slot(0) {
            Some(r) => {
                let inner = panel.render(r, buf, t);
                self.table.render(inner, buf, ctx, bg);
                let pos = scrollbar::position_label(&self.table.scroll);
                let meta = if pos.is_empty() {
                    sort
                } else {
                    format!("{sort} · {pos}")
                };
                panel.draw_meta(r, buf, t, &meta);
            }
            None => ctx.focus_stop(self.table.id),
        }

        let panel = Panel::card(Some("Checks"));
        let bg = panel.bg(t);
        match l.slot(1) {
            Some(r) => {
                let inner = panel.render(r, buf, t);
                self.empty.render(inner, buf, ctx, bg);
            }
            None => ctx.focus_stop(self.empty.id),
        }
        self.menu.render(area, buf, ctx);
        self.header_menu.render(area, buf, ctx);
    }

    fn layer_open(&self) -> bool {
        self.menu.hides_page_focus() || self.header_menu.hides_page_focus()
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some((out, chosen)) = self.header_menu.handle(ev) {
            return match chosen.and_then(|(c, i)| Some((c, self.table.header_menu(c).get(i)?.0))) {
                Some((c, action)) => self.table.on_header_action(c, action).or(out),
                None => out,
            };
        }
        if let Some((out, chosen)) = self.menu.handle(ev) {
            return match chosen {
                Some((cell, i)) => self.choose(cell, i, cx).or(out),
                None => out,
            };
        }
        if let Some(o) = self.sections.handle(ev) {
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                if cx.focus.is(self.table.id) {
                    if !self.table.is_editing()
                        && let Some(req) = self.menu.config.on_key(&self.table, key)
                    {
                        self.open_menu(req);
                        return Outcome::Changed;
                    }
                    let (out, tev) = self.table.on_key(key);
                    if let Some(TableAction::Activated(row)) = tev {
                        cx.status(format!("Selected {}", self.table.rows[row][1].text));
                    }
                    return out;
                }
                if cx.focus.is(self.empty.id) {
                    return self.empty.on_key(key).0;
                }
                Outcome::Ignored
            }
            PageEvent::Secondary { id, pos } if self.table.locate_header_menu(*id).is_some() => {
                let Some(c) = self.table.locate_header_menu(*id) else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(self.table.id);
                let items = self
                    .table
                    .header_menu(c)
                    .into_iter()
                    .map(|(_, m)| m)
                    .collect();
                let title = self.table.columns[c].title.clone();
                let menu = ContextMenu::new(HEADER_MENU, items).at(*pos).title(title);
                self.header_menu.open(menu, c);
                Outcome::Changed
            }
            PageEvent::Press { id, pos } if self.table.locate_seam(*id).is_some() => {
                self.table.on_press(*id, *pos, cx.clicks)
            }
            PageEvent::Secondary { id, pos } => {
                let cfg = self.menu.config;
                let Some(req) = cfg.on_secondary(&mut self.table, *id, *pos) else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(self.table.id);
                self.open_menu(req);
                Outcome::Changed
            }
            PageEvent::Click { id, pos } => {
                if let Some(c) = self.table.locate_header(*id) {
                    cx.focus.focus(self.table.id);
                    return self.table.on_click_header(c);
                }
                if let Some((row, col)) = self.table.locate(*id) {
                    cx.focus.focus(self.table.id);
                    let (out, tev) = self.table.on_click_cell(row, col.unwrap_or(0), *pos);
                    if let Some(TableAction::Activated(r)) = tev {
                        cx.status(format!("Selected {}", self.table.rows[r][1].text));
                    }
                    return out;
                }
                if scrollbar::is_bar_of(*id, self.table.id) {
                    return self.table.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } if self.table.owns(*pressed) => {
                self.table.on_drag(*pressed, *pos)
            }
            PageEvent::Wheel { id, delta } if self.table.owns(*id) => self.table.on_wheel(*delta),
            PageEvent::Wheel { id, delta } if self.empty.owns(*id) => self.empty.on_wheel(*delta),
            PageEvent::WheelH { id, delta } if self.table.owns(*id) => {
                self.table.on_wheel_h(*delta)
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if self.menu.is_open() {
            return MENU_HINTS.to_vec();
        }
        vec![
            ("↑↓", "Move"),
            ("←→", "Columns"),
            ("s", "Sort column"),
            ("Enter", "Select"),
            OPEN_HINT,
        ]
    }
}
