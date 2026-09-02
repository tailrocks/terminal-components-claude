//! Chip bar: a row of removable, toggleable chips (filters, tags). One
//! focus stop; ← → move the cursor, Enter activates, Delete removes.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::ButtonKind;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{truncate, width};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub label: String,
    pub enabled: bool,
    pub removable: bool,
    pub error: bool,
}

impl Chip {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            enabled: true,
            removable: true,
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipBar {
    pub id: WidgetId,
    pub chips: Vec<Chip>,
    pub cursor: usize,
    pub add_label: Option<String>,
    /// Leading label such as `match all ▾` (clickable, emits `Toggle`).
    pub lead: Option<String>,
    pub area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipEvent {
    Activate(usize),
    Toggle(usize),
    Remove(usize),
    Add,
    Lead,
    ClearAll,
}

impl ChipBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            chips: vec![],
            cursor: 0,
            add_label: Some("+ Add filter".into()),
            lead: None,
            area: Rect::ZERO,
        }
    }

    pub fn chip_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.child(i).sub("x")
    }
    pub fn add_id(&self) -> WidgetId {
        self.id.sub("add")
    }
    pub fn lead_id(&self) -> WidgetId {
        self.id.sub("lead")
    }

    /// Cursor positions: 0..n chips, n = add button.
    fn stops(&self) -> usize {
        self.chips.len() + usize::from(self.add_label.is_some())
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ChipEvent>) {
        let n = self.stops();
        if n == 0 {
            return (Outcome::Ignored, None);
        }
        self.cursor = self.cursor.min(n - 1);
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if key.plain() => {
                self.cursor = self.cursor.saturating_sub(1);
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                self.cursor = (self.cursor + 1).min(n - 1);
                (Outcome::Changed, None)
            }
            KeyCode::Enter => {
                if self.cursor < self.chips.len() {
                    (Outcome::Changed, Some(ChipEvent::Activate(self.cursor)))
                } else {
                    (Outcome::Changed, Some(ChipEvent::Add))
                }
            }
            KeyCode::Char(' ') if self.cursor < self.chips.len() => {
                (Outcome::Changed, Some(ChipEvent::Toggle(self.cursor)))
            }
            KeyCode::Delete | KeyCode::Backspace | KeyCode::Char('x')
                if self.cursor < self.chips.len() && self.chips[self.cursor].removable =>
            {
                (Outcome::Changed, Some(ChipEvent::Remove(self.cursor)))
            }
            KeyCode::Char('+') => (Outcome::Changed, Some(ChipEvent::Add)),
            KeyCode::Char('X') => (Outcome::Changed, Some(ChipEvent::ClearAll)),
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ChipEvent>) {
        if id == self.add_id() {
            self.cursor = self.chips.len();
            return (Outcome::Changed, Some(ChipEvent::Add));
        }
        if id == self.lead_id() {
            return (Outcome::Changed, Some(ChipEvent::Lead));
        }
        for i in 0..self.chips.len() {
            if self.close_id(i) == id {
                return (Outcome::Changed, Some(ChipEvent::Remove(i)));
            }
            if self.chip_id(i) == id {
                self.cursor = i;
                return (Outcome::Changed, Some(ChipEvent::Activate(i)));
            }
        }
        (Outcome::Ignored, None)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == self.add_id()
            || id == self.lead_id()
            || (0..self.chips.len()).any(|i| self.chip_id(i) == id || self.close_id(i) == id)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let mut x = area.x;
        let y = area.y;
        if let Some(lead) = &self.lead {
            let lid = self.lead_id();
            let hovered = ctx.interaction.hovered(lid);
            let st = if hovered {
                t.primary().bg(t.lift(bg))
            } else {
                t.muted().bg(bg)
            };
            let text = format!(" {lead} ");
            buf.set_string(x, y, &text, st);
            ctx.clickable(lid, Rect::new(x, y, width(&text) as u16, 1));
            x += width(&text) as u16 + 1;
        }
        for (i, chip) in self.chips.iter().enumerate() {
            let cid = self.chip_id(i);
            let mut s = ctx.state(cid);
            s.focused = focused && i == self.cursor;
            if ctx.interaction.hovered(self.close_id(i)) {
                s.hovered = true;
            }
            let kind = if chip.enabled {
                ButtonKind::Toggle
            } else {
                ButtonKind::Secondary
            };
            let mut st = t.button(kind, s, bg);
            if !chip.enabled {
                st = st.fg(t.text_faint);
            }
            if chip.error {
                st = st.fg(t.error);
            }
            let label_w = width(&chip.label) as u16;
            let w = 1 + label_w + 1 + if chip.removable { 2 } else { 0 } + 1;
            if x + w > area.right() {
                buf.set_string(x, y, "…", t.muted().bg(bg));
                return;
            }
            let r = Rect::new(x, y, w, 1);
            fill(buf, r, st);
            buf.set_string(x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(x + 1, y, truncate(&chip.label, label_w as usize), st);
            if chip.removable {
                let xid = self.close_id(i);
                let xh = ctx.interaction.hovered(xid);
                let xs = if xh {
                    st.fg(t.text_primary).add_modifier(Modifier::BOLD)
                } else {
                    st.fg(t.text_muted)
                };
                buf.set_string(x + 2 + label_w, y, "×", xs);
                ctx.clickable(cid, r);
                ctx.clickable(xid, Rect::new(x + 2 + label_w, y, 1, 1));
            } else {
                ctx.clickable(cid, r);
            }
            x += w + 1;
        }
        if let Some(add) = &self.add_label {
            let aid = self.add_id();
            let mut s = ctx.state(aid);
            s.focused = focused && self.cursor == self.chips.len();
            let st = t.button(ButtonKind::Subtle, s, bg);
            let w = width(add) as u16 + 2;
            if x + w <= area.right() {
                let r = Rect::new(x, y, w, 1);
                fill(buf, r, st);
                buf.set_string(x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                buf.set_string(x + 1, y, add, st);
                ctx.clickable(aid, r);
            }
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}
