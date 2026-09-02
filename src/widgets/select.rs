//! Dropdown field: closed it looks like a text field with a trailing `▾`;
//! open it shows an anchored popup list. One focus stop.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Placement, place, surface};

#[derive(Debug, Clone)]
pub struct Select {
    pub id: WidgetId,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub cursor: usize,
    pub open: bool,
    pub disabled: bool,
    pub help: String,
    pub area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectEvent {
    Changed(usize),
}

impl Select {
    pub const HEIGHT: u16 = 3;

    pub fn new(id: WidgetId, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            id,
            label: label.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            selected,
            cursor: selected,
            open: false,
            disabled: false,
            help: String::new(),
            area: Rect::ZERO,
        }
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.to_owned();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn value(&self) -> &str {
        self.options
            .get(self.selected)
            .map(String::as_str)
            .unwrap_or("")
    }
    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.options.len()).find(|&i| self.option_id(i) == id)
    }
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || self.locate(id).is_some()
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if self.open {
            return match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.cursor = self.cursor.saturating_sub(1);
                    (Outcome::Changed, None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.cursor = (self.cursor + 1).min(self.options.len().saturating_sub(1));
                    (Outcome::Changed, None)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.open = false;
                    let changed = self.cursor != self.selected;
                    self.selected = self.cursor;
                    (
                        Outcome::Changed,
                        changed.then_some(SelectEvent::Changed(self.selected)),
                    )
                }
                KeyCode::Esc => {
                    self.open = false;
                    self.cursor = self.selected;
                    (Outcome::Changed, None)
                }
                _ => (Outcome::Consumed, None),
            };
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.open = true;
                self.cursor = self.selected;
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Left if key.plain() => {
                let n = self.selected.saturating_sub(1);
                let changed = n != self.selected;
                self.selected = n;
                (Outcome::Changed, changed.then_some(SelectEvent::Changed(n)))
            }
            KeyCode::Down | KeyCode::Right if key.plain() => {
                let n = (self.selected + 1).min(self.options.len().saturating_sub(1));
                let changed = n != self.selected;
                self.selected = n;
                (Outcome::Changed, changed.then_some(SelectEvent::Changed(n)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Consumed, None);
        }
        if id == self.id {
            self.open = !self.open;
            self.cursor = self.selected;
            return (Outcome::Changed, None);
        }
        if let Some(i) = self.locate(id) {
            self.open = false;
            let changed = i != self.selected;
            self.selected = i;
            return (Outcome::Changed, changed.then_some(SelectEvent::Changed(i)));
        }
        (Outcome::Ignored, None)
    }

    /// Close without changing (click outside).
    pub fn dismiss(&mut self) -> Outcome {
        if self.open {
            self.open = false;
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
            self.open = false;
        }
        if !s.focused {
            self.open = false;
        }
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(s.focused).bg(bg)
        };
        buf.set_string(area.x + 2, area.y, &self.label, label_style);
        if area.height < 2 {
            return;
        }
        let field = Rect::new(area.x, area.y + 1, area.width, 1);
        self.area = field;
        let fs = t.field_style(s);
        fill(buf, field, fs);
        buf.set_string(
            field.x,
            field.y,
            "▎",
            t.gutter(s, fs.bg.unwrap_or(bg), false),
        );
        let value = crate::ui::text::truncate(self.value(), field.width.saturating_sub(5) as usize);
        buf.set_string(field.x + 2, field.y, &value, fs);
        buf.set_string(
            field.right().saturating_sub(2),
            field.y,
            if self.open { "▴" } else { "▾" },
            fs.fg(if self.disabled {
                t.disabled
            } else {
                t.text_secondary
            }),
        );
        ctx.control(self.id, field, self.disabled);
        if area.height >= 3 && !self.help.is_empty() {
            buf.set_string(
                area.x + 2,
                area.y + 2,
                crate::ui::text::truncate(&self.help, area.width.saturating_sub(2) as usize),
                t.muted().bg(bg),
            );
        }
        if self.open {
            let screen = *buf.area();
            let h = (self.options.len() as u16 + 2).min(10);
            let w = field.width.clamp(12, 40);
            let pa = place(screen, field, w, h, Placement::Below);
            let inner = surface(pa, buf, ctx, t);
            for (i, opt) in self.options.iter().enumerate() {
                let y = inner.y + i as u16;
                if y >= inner.bottom() {
                    break;
                }
                let rid = self.option_id(i);
                let mut rs = ctx.state(rid);
                rs.focused = i == self.cursor;
                rs.selected = i == self.selected;
                let st = t.row(rs, t.surface_elevated);
                let row = Rect::new(inner.x, y, inner.width, 1);
                fill(buf, row, st);
                buf.set_string(
                    row.x,
                    y,
                    "▎",
                    t.gutter(rs, st.bg.unwrap_or(t.surface_elevated), false),
                );
                if rs.selected {
                    buf.set_string(row.x + 1, y, "›", st.fg(t.accent));
                }
                buf.set_string(
                    row.x + 3,
                    y,
                    crate::ui::text::truncate(opt, row.width.saturating_sub(3) as usize),
                    st,
                );
                ctx.clickable(rid, row);
            }
        }
    }
}
