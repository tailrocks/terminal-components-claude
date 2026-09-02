use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;

/// Horizontal tab strip. One focus stop; ← → switch immediately.
/// Active tab: white bold with an accent underline row. Focus cursor: bar.
#[derive(Debug, Clone)]
pub struct Tabs {
    pub id: WidgetId,
    pub labels: Vec<String>,
    pub active: usize,
    pub cursor: usize,
    pub areas: Vec<Rect>,
}

impl Tabs {
    pub fn new(id: WidgetId, labels: &[&str]) -> Self {
        Self {
            id,
            labels: labels.iter().map(|s| (*s).to_owned()).collect(),
            active: 0,
            cursor: 0,
            areas: vec![],
        }
    }

    pub fn tab_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.labels.len()).find(|&i| self.tab_id(i) == id)
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if key.plain() => {
                self.cursor = self.cursor.saturating_sub(1);
                self.active = self.cursor;
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                self.cursor = (self.cursor + 1).min(self.labels.len().saturating_sub(1));
                self.active = self.cursor;
                Outcome::Changed
            }
            KeyCode::Char(c) if c.is_ascii_digit() && key.plain() => {
                let i = c as usize - '1' as usize;
                if i < self.labels.len() {
                    self.cursor = i;
                    self.active = i;
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.active = self.cursor;
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, i: usize) -> Outcome {
        if i < self.labels.len() {
            self.cursor = i;
            self.active = i;
        }
        Outcome::Changed
    }

    /// Two rows: labels, then the underline row.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.areas.clear();
        let mut x = area.x;
        let y = area.y;
        // baseline
        if area.height >= 2 {
            for xx in area.left()..area.right() {
                buf.set_string(
                    xx,
                    y + 1,
                    "─",
                    ratatui::style::Style::new().fg(t.border_subtle).bg(bg),
                );
            }
        }
        for (i, label) in self.labels.iter().enumerate() {
            let w = crate::ui::text::width(label) as u16 + 3;
            if x + w > area.right() {
                break;
            }
            let r = Rect::new(x, y, w, 1);
            self.areas.push(r);
            let tid = self.tab_id(i);
            let mut s = ctx.state(tid);
            s.focused = focused && i == self.cursor;
            let active = i == self.active;
            let mut st = ratatui::style::Style::new()
                .bg(bg)
                .fg(if active || s.hovered {
                    t.text_primary
                } else {
                    t.text_secondary
                });
            if s.hovered && !active {
                st = st.bg(t.lift(bg));
            }
            if active || s.focused {
                st = st.add_modifier(Modifier::BOLD);
            }
            crate::ui::ctx::fill(buf, r, st);
            buf.set_string(x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(x + 1, y, label, st);
            if active && area.height >= 2 {
                for xx in x + 1..x + w - 1 {
                    buf.set_string(
                        xx,
                        y + 1,
                        "━",
                        ratatui::style::Style::new().fg(t.accent).bg(bg),
                    );
                }
            }
            ctx.clickable(tid, r);
            x += w + 1;
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}
