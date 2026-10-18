//! Checkbox, radio group and toggle switch for forms.
//!
//! Labels and marks stay within the supplied rectangle. At widths of two or
//! three cells, compact marks preserve checked/on state without colour; one
//! cell can show only focus. Empty radio renders clear their old option areas.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;

// Buffer string writes clip to the whole buffer, not the widget rectangle.
// Keep every optional row segment bounded before deriving its coordinates.
fn row_part(buf: &mut Buffer, row: Rect, offset: u16, text: &str, style: Style) {
    if offset < row.width {
        buf.set_stringn(
            row.x + offset,
            row.y,
            text,
            usize::from(row.width - offset),
            style,
        );
    }
}

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
        buf.set_string(
            area.x,
            area.y,
            t.gutter_symbol(s),
            t.gutter(s, st.bg.unwrap_or(bg), false),
        );
        let mark = match (area.width < 4, self.checked) {
            (true, true) => "✓",
            (true, false) => "□",
            (false, true) => "[✓]",
            (false, false) => "[ ]",
        };
        let mark_style = if self.disabled {
            st
        } else if self.checked {
            st.fg(t.accent)
        } else {
            st.fg(t.text_muted)
        };
        row_part(buf, area, 1, mark, mark_style);
        row_part(
            buf,
            area,
            5,
            &crate::ui::text::truncate(&self.label, area.width.saturating_sub(6) as usize),
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
        self.options.len().saturating_add(1).min(u16::MAX as usize) as u16
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
        self.areas.clear();
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
        row_part(
            buf,
            area,
            2,
            &crate::ui::text::truncate(&self.label, area.width.saturating_sub(2) as usize),
            label_style,
        );
        for (i, opt) in self
            .options
            .iter()
            .take(area.height.saturating_sub(1) as usize)
            .enumerate()
        {
            let y = area.y + 1 + i as u16;
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
            buf.set_string(
                row.x,
                y,
                t.gutter_symbol(s),
                t.gutter(s, st.bg.unwrap_or(bg), false),
            );
            let on = i == self.selected;
            let mark = match (row.width < 4, on) {
                (true, true) => "●",
                (true, false) => "○",
                (false, true) => "(●)",
                (false, false) => "( )",
            };
            let ms = if self.disabled {
                st
            } else if on {
                st.fg(t.accent)
            } else {
                st.fg(t.text_muted)
            };
            row_part(buf, row, 1, mark, ms);
            row_part(
                buf,
                row,
                5,
                &crate::ui::text::truncate(opt, row.width.saturating_sub(5) as usize),
                st,
            );
            ctx.clickable(self.option_id(i), row);
        }
        // the group is a single focus stop; option rows are click targets
        if !ctx.inert && !self.disabled && !self.areas.is_empty() {
            ctx.ring.register(self.id);
        }
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
        buf.set_string(
            area.x,
            area.y,
            t.gutter_symbol(s),
            t.gutter(s, st.bg.unwrap_or(bg), false),
        );
        let (sw, ss) = if self.disabled {
            (if self.on { "──●" } else { "○──" }, st)
        } else if self.on {
            ("──●", st.fg(t.accent))
        } else {
            ("○──", st.fg(t.text_muted))
        };
        let sw = if area.width < 4 {
            if self.on { "●" } else { "○" }
        } else {
            sw
        };
        row_part(buf, area, 1, sw, ss);
        row_part(
            buf,
            area,
            5,
            &crate::ui::text::truncate(&self.label, area.width.saturating_sub(5) as usize),
            st,
        );
        let state = if self.on { "on" } else { "off" };
        let state_offset = 6usize.saturating_add(crate::ui::text::width(&self.label));
        if state_offset.saturating_add(3) < usize::from(area.width) {
            row_part(
                buf,
                area,
                state_offset as u16,
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
