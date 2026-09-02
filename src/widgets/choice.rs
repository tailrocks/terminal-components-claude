//! Checkbox, radio group and toggle switch for forms.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone)]
pub struct Checkbox {
    pub id: WidgetId,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub area: Rect,
}

impl Checkbox {
    pub fn new(id: WidgetId, label: &str, checked: bool) -> Self {
        Self {
            id,
            label: label.to_owned(),
            checked,
            disabled: false,
            area: Rect::ZERO,
        }
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        if key.is_char(' ') || key.is(KeyCode::Enter) {
            self.checked = !self.checked;
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_click(&mut self) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.checked = !self.checked;
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        let area = Rect::new(area.x, area.y, area.width, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        s.selected = false;
        if self.disabled {
            s.hovered = false;
        }
        let st = t.row(s, bg);
        crate::ui::ctx::fill(buf, area, st);
        buf.set_string(area.x, area.y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
        let mark = if self.checked { "[✓]" } else { "[ ]" };
        let mark_style = if self.disabled {
            st
        } else if self.checked {
            st.fg(t.accent)
        } else {
            st.fg(t.text_muted)
        };
        buf.set_string(area.x + 1, area.y, mark, mark_style);
        buf.set_string(
            area.x + 5,
            area.y,
            crate::ui::text::truncate(&self.label, area.width.saturating_sub(6) as usize),
            st,
        );
        ctx.control(self.id, area, self.disabled);
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroup {
    pub id: WidgetId,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub cursor: usize,
    pub disabled: bool,
    pub areas: Vec<Rect>,
}

impl RadioGroup {
    pub fn new(id: WidgetId, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            id,
            label: label.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            selected,
            cursor: selected,
            disabled: false,
            areas: vec![],
        }
    }

    pub fn height(&self) -> u16 {
        self.options.len() as u16 + 1
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                self.cursor = self.cursor.saturating_sub(1);
                self.selected = self.cursor;
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if key.plain() => {
                self.cursor = (self.cursor + 1).min(self.options.len().saturating_sub(1));
                self.selected = self.cursor;
                Outcome::Changed
            }
            KeyCode::Char(' ') | KeyCode::Enter if key.plain() => {
                self.selected = self.cursor;
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, index: usize) -> Outcome {
        if self.disabled || index >= self.options.len() {
            return Outcome::Consumed;
        }
        self.cursor = index;
        self.selected = index;
        Outcome::Changed
    }

    /// Ids of the individual options, for hit testing.
    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(focused).bg(bg)
        };
        buf.set_string(area.x + 2, area.y, &self.label, label_style);
        self.areas.clear();
        for (i, opt) in self.options.iter().enumerate() {
            let y = area.y + 1 + i as u16;
            if y >= area.bottom() {
                break;
            }
            let row = Rect::new(area.x, y, area.width, 1);
            self.areas.push(row);
            let mut s = ctx.state(self.option_id(i));
            s.focused = focused && i == self.cursor;
            s.disabled = self.disabled;
            if self.disabled {
                s.hovered = false;
            }
            let st = t.row(s, bg);
            crate::ui::ctx::fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let on = i == self.selected;
            let mark = if on { "(●)" } else { "( )" };
            let ms = if self.disabled {
                st
            } else if on {
                st.fg(t.accent)
            } else {
                st.fg(t.text_muted)
            };
            buf.set_string(row.x + 1, y, mark, ms);
            buf.set_string(row.x + 5, y, opt, st);
            ctx.clickable(self.option_id(i), row);
        }
        let whole = Rect::new(
            area.x,
            area.y,
            area.width,
            (self.options.len() as u16 + 1).min(area.height),
        );
        // the group is a single focus stop; option rows are click targets
        if !ctx.inert && !self.disabled {
            ctx.ring.register(self.id);
        }
        let _ = whole;
    }
}

#[derive(Debug, Clone)]
pub struct Toggle {
    pub id: WidgetId,
    pub label: String,
    pub on: bool,
    pub disabled: bool,
    pub area: Rect,
}

impl Toggle {
    pub fn new(id: WidgetId, label: &str, on: bool) -> Self {
        Self {
            id,
            label: label.to_owned(),
            on,
            disabled: false,
            area: Rect::ZERO,
        }
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        if key.is_char(' ') || key.is(KeyCode::Enter) {
            self.on = !self.on;
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_click(&mut self) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.on = !self.on;
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        let area = Rect::new(area.x, area.y, area.width, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
        }
        let st = t.row(s, bg);
        crate::ui::ctx::fill(buf, area, st);
        buf.set_string(area.x, area.y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
        let (sw, ss) = if self.disabled {
            (if self.on { "──●" } else { "○──" }, st)
        } else if self.on {
            ("──●", st.fg(t.accent))
        } else {
            ("○──", st.fg(t.text_muted))
        };
        buf.set_string(area.x + 1, area.y, sw, ss);
        buf.set_string(area.x + 5, area.y, &self.label, st);
        let state = if self.on { "on" } else { "off" };
        let sx = area.x + 6 + crate::ui::text::width(&self.label) as u16;
        if sx + 3 < area.right() {
            buf.set_string(
                sx,
                area.y,
                state,
                st.fg(if self.disabled {
                    t.disabled
                } else {
                    t.text_muted
                }),
            );
        }
        ctx.control(self.id, area, self.disabled);
    }
}
