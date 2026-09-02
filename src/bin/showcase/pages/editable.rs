use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::table::{Cell, Column, DataTable, TableEvent, Tone};

const ID: WidgetId = WidgetId::of("editable");

fn validate(col: usize, s: &str) -> Option<String> {
    match col {
        1 if s.trim().is_empty() => Some("Task name cannot be empty".into()),
        2 if s.trim().is_empty() || s.contains(' ') => Some("Owner is a single handle".into()),
        4 if s.contains(' ') => Some("Branch names cannot contain spaces".into()),
        5 if s.parse::<u32>().is_err() => Some("Changes must be a whole number".into()),
        _ => None,
    }
}

pub struct EditablePage {
    table: DataTable,
    edits: u32,
}

impl EditablePage {
    pub fn new() -> Self {
        let columns = vec![
            Column::new("ID", Constraint::Length(5)),
            Column::new("Task", Constraint::Min(24)).editable(),
            Column::new("Owner", Constraint::Length(8)).editable(),
            Column::new("Status", Constraint::Length(9)),
            Column::new("Branch", Constraint::Length(22)).editable(),
            Column::new("Changes", Constraint::Length(8))
                .right()
                .editable(),
        ];
        let rows: Vec<Vec<Cell>> = crate::data::tasks()
            .into_iter()
            .take(14)
            .map(|r| {
                vec![
                    Cell::new(format!("#{}", r.id)).tone(Tone::Muted),
                    Cell::new(r.name),
                    Cell::new(r.owner),
                    crate::pages::tables::status_cell(r.status),
                    Cell::new(r.branch).tone(Tone::Muted),
                    Cell::new(r.changes.to_string()),
                ]
            })
            .collect();
        let mut table = DataTable::new(ID.sub("table"), columns, rows)
            .cell_nav(true)
            .numeric(&[0, 5])
            .validator(validate);
        // one pre-existing error to show the row-level error state
        table.rows[2][4].error = Some("Branch names cannot contain spaces".into());
        table.rows[2][4].text = "fix/checkout flake".into();
        table.cursor_col = 1;
        Self { table, edits: 0 }
    }
}

impl Page for EditablePage {
    fn title(&self) -> &'static str {
        "Editable tables"
    }
    fn blurb(&self) -> &'static str {
        "Navigation is reversed cell; editing is a cursor. They never look alike."
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let pos = scrollbar::position_label(&self.table.scroll);
        let err = self.table.edit_error().map(str::to_owned);
        let meta = match &err {
            Some(e) => e.clone(),
            None if pos.is_empty() => format!("{} edits", self.edits),
            None => format!("{} edits · {pos}", self.edits),
        };
        let panel = Panel::card(Some("Tasks"))
            .meta(&meta)
            .focused(ctx.interaction.focused(self.table.id));
        let bg = panel.bg(t);
        let card_h = (self.table.len() as u16 + 4).min(area.height.saturating_sub(4));
        let inner = panel.render(Rect::new(area.x, area.y, area.width, card_h), buf, t);
        if let Some(e) = &err {
            // error message drawn in error colour over the meta slot
            let x = area
                .right()
                .saturating_sub(2 + junie_tui::ui::text::width(e) as u16);
            buf.set_string(x, area.y, e, t.error_fg().bg(bg));
        }
        self.table.render(inner, buf, ctx, bg);
        let y = area.y + card_h + 1;
        let legend = [
            (
                "reversed",
                "cell cursor (navigation)",
                t.on(t.text_primary).fg(t.canvas),
            ),
            ("▁", "editing cursor + accent underline", t.primary()),
            ("!", "validation error", t.error_fg()),
        ];
        for (i, (g, text, st)) in legend.iter().enumerate() {
            let yy = y + i as u16;
            if yy < area.bottom() {
                buf.set_string(area.x, yy, g, *st);
                buf.set_string(area.x + 10, yy, text, t.muted());
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                if !cx.focus.is(self.table.id) {
                    return Outcome::Ignored;
                }
                let (out, tev) = self.table.on_key(key);
                match tev {
                    Some(TableEvent::Committed { .. }) => {
                        self.edits += 1;
                        cx.status("Cell saved");
                    }
                    Some(TableEvent::Cancelled) => cx.status("Edit cancelled"),
                    Some(TableEvent::LeaveForward) => cx.focus_next(),
                    Some(TableEvent::LeaveBackward) => cx.focus_prev(),
                    _ => {}
                }
                out
            }
            PageEvent::Paste(text) => self.table.on_paste(text),
            PageEvent::Click { id, pos } => {
                if let Some(c) = self.table.locate_header(*id) {
                    cx.focus.focus(self.table.id);
                    return self.table.on_click_header(c);
                }
                if let Some((row, col)) = self.table.locate(*id) {
                    cx.focus.focus(self.table.id);
                    let (out, tev) =
                        self.table
                            .on_click_cell(row, col.unwrap_or(self.table.cursor_col), *pos);
                    if let Some(TableEvent::Committed { .. }) = tev {
                        self.edits += 1;
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

    fn editing(&self) -> bool {
        self.table.is_editing()
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if self.table.is_editing() {
            vec![("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            vec![
                ("↑ ↓ ← →", "Cell"),
                ("Enter", "Edit"),
                ("s", "Sort"),
                ("click twice", "Edit"),
            ]
        }
    }
}
