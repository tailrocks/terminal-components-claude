//! Horizontal tab strip. One focus stop; ← → move the cursor and switch.
//! Active tab: white bold with an accent underline. Focus cursor: bar.
//! Tabs carry state glyphs (dirty, busy, error) and a close affordance; the
//! strip scrolls with `‹ ›` indicators instead of shrinking labels.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabItem {
    pub label: String,
    pub dirty: bool,
    pub busy: bool,
    pub error: bool,
    pub closable: bool,
    /// Muted prefix such as a type glyph or a schema.
    pub prefix: Option<String>,
}

impl TabItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            ..Default::default()
        }
    }
    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
    pub fn prefix(mut self, p: &str) -> Self {
        self.prefix = Some(p.to_owned());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Tabs {
    pub id: WidgetId,
    pub items: Vec<TabItem>,
    pub active: usize,
    pub cursor: usize,
    /// First visible tab (strip scroll).
    pub first: usize,
    pub areas: Vec<Rect>,
    /// Show a trailing `+` that emits `TabEvent::New`.
    pub allow_new: bool,
    /// Secondary level: the active rule is white, so one screen keeps one
    /// accent underline (the document tabs).
    pub quiet: bool,
    /// Number of tabs that fit in the last render.
    fit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabEvent {
    Activated(usize),
    Close(usize),
    New,
}

impl Tabs {
    pub fn new(id: WidgetId, labels: &[&str]) -> Self {
        Self {
            id,
            items: labels.iter().map(|l| TabItem::new(l)).collect(),
            active: 0,
            cursor: 0,
            first: 0,
            areas: vec![],
            allow_new: false,
            quiet: false,
            fit: 0,
        }
    }

    pub fn with_items(id: WidgetId, items: Vec<TabItem>) -> Self {
        Self {
            id,
            items,
            active: 0,
            cursor: 0,
            first: 0,
            areas: vec![],
            allow_new: false,
            quiet: false,
            fit: 0,
        }
    }

    pub fn tab_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.child(i).sub("close")
    }
    pub fn new_id(&self) -> WidgetId {
        self.id.sub("new")
    }
    pub fn left_id(&self) -> WidgetId {
        self.id.sub("left")
    }
    pub fn right_id(&self) -> WidgetId {
        self.id.sub("right")
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.items.len()).find(|&i| self.tab_id(i) == id)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == self.new_id()
            || id == self.left_id()
            || id == self.right_id()
            || self.locate(id).is_some()
            || (0..self.items.len()).any(|i| self.close_id(i) == id)
    }

    pub fn set_active(&mut self, i: usize) {
        if self.items.is_empty() {
            self.active = 0;
            self.cursor = 0;
            return;
        }
        self.active = i.min(self.items.len() - 1);
        self.cursor = self.active;
        self.ensure_visible(self.active);
    }

    pub fn remove(&mut self, i: usize) {
        if i < self.items.len() {
            self.items.remove(i);
            if self.active >= self.items.len() {
                self.active = self.items.len().saturating_sub(1);
            } else if self.active > i {
                self.active -= 1;
            }
            self.cursor = self.active;
            self.first = self.first.min(self.items.len().saturating_sub(1));
        }
    }

    fn ensure_visible(&mut self, i: usize) {
        if i < self.first {
            self.first = i;
        } else if self.fit > 0 && i >= self.first + self.fit {
            self.first = i + 1 - self.fit;
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TabEvent>) {
        if self.items.is_empty() {
            return match key.code {
                KeyCode::Enter | KeyCode::Char('n') if self.allow_new => {
                    (Outcome::Changed, Some(TabEvent::New))
                }
                _ => (Outcome::Ignored, None),
            };
        }
        match key.code {
            KeyCode::Left | KeyCode::Char('h') if key.plain() => {
                self.set_active(self.cursor.saturating_sub(1));
                (Outcome::Changed, Some(TabEvent::Activated(self.active)))
            }
            KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                self.set_active((self.cursor + 1).min(self.items.len() - 1));
                (Outcome::Changed, Some(TabEvent::Activated(self.active)))
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' && key.plain() => {
                let i = c as usize - '1' as usize;
                if i < self.items.len() {
                    self.set_active(i);
                    (Outcome::Changed, Some(TabEvent::Activated(i)))
                } else {
                    (Outcome::Ignored, None)
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.set_active(self.cursor);
                (Outcome::Changed, Some(TabEvent::Activated(self.active)))
            }
            KeyCode::Char('x') | KeyCode::Delete
                if key.plain() && self.items[self.cursor].closable =>
            {
                (Outcome::Changed, Some(TabEvent::Close(self.cursor)))
            }
            KeyCode::Char('n') if key.plain() && self.allow_new => {
                (Outcome::Changed, Some(TabEvent::New))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<TabEvent>) {
        if id == self.new_id() {
            return (Outcome::Changed, Some(TabEvent::New));
        }
        if id == self.left_id() {
            self.first = self.first.saturating_sub(1);
            return (Outcome::Changed, None);
        }
        if id == self.right_id() {
            self.first = (self.first + 1).min(self.items.len().saturating_sub(1));
            return (Outcome::Changed, None);
        }
        for i in 0..self.items.len() {
            if self.close_id(i) == id {
                return (Outcome::Changed, Some(TabEvent::Close(i)));
            }
        }
        if let Some(i) = self.locate(id) {
            self.set_active(i);
            return (Outcome::Changed, Some(TabEvent::Activated(i)));
        }
        (Outcome::Ignored, None)
    }

    fn tab_width(&self, it: &TabItem) -> u16 {
        let mut w = 1 + crate::ui::text::width(&it.label) as u16 + 2;
        if let Some(p) = &it.prefix {
            w += crate::ui::text::width(p) as u16 + 1;
        }
        if it.dirty || it.busy || it.error {
            w += 2;
        }
        if it.closable {
            w += 2;
        }
        w
    }

    /// Two rows: labels, then the underline row.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.areas = vec![Rect::ZERO; self.items.len()];
        let y = area.y;
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
        // reserve space for the overflow indicators and the new-tab button
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let mut x = area.x;
        // figure out how many fit from `first`
        let widths: Vec<u16> = self.items.iter().map(|it| self.tab_width(it)).collect();
        let total: u16 = widths.iter().map(|w| w + 1).sum();
        let overflow = total + new_w > area.width;
        let left_w: u16 = if overflow { 3 } else { 0 };
        let right_w: u16 = if overflow { 3 } else { 0 };
        let avail = area.width.saturating_sub(left_w + right_w + new_w);
        // make sure the active tab is within the window
        let mut fit;
        // shrink `first` if active is before it
        if self.active < self.first {
            self.first = self.active;
        }
        loop {
            fit = 0;
            let mut used = 0u16;
            for w in widths.iter().skip(self.first) {
                if used + w + 1 > avail {
                    break;
                }
                used += w + 1;
                fit += 1;
            }
            if fit == 0 && !self.items.is_empty() {
                fit = 1;
            }
            if self.active >= self.first + fit && self.first + 1 < self.items.len() {
                self.first += 1;
                continue;
            }
            break;
        }
        self.fit = fit;
        if overflow {
            let more_left = self.first > 0;
            let st = if more_left {
                t.secondary().bg(bg)
            } else {
                t.faint().bg(bg)
            };
            let hovered = ctx.interaction.hovered(self.left_id());
            buf.set_string(
                x,
                y,
                if more_left { " ‹ " } else { "   " },
                if hovered && more_left {
                    st.bg(t.lift(bg))
                } else {
                    st
                },
            );
            if more_left {
                ctx.clickable(self.left_id(), Rect::new(x, y, 3, 1));
            }
            x += 3;
        }
        // index is needed for ids and the parallel `widths` vector
        #[allow(clippy::needless_range_loop)]
        for i in self.first..(self.first + fit).min(self.items.len()) {
            let it = &self.items[i];
            let w = widths[i];
            if x + w > area.right() {
                break;
            }
            let r = Rect::new(x, y, w, 1);
            self.areas[i] = r;
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
            let mut cx = x + 1;
            if let Some(p) = &it.prefix {
                buf.set_string(
                    cx,
                    y,
                    p,
                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                );
                cx += crate::ui::text::width(p) as u16 + 1;
            }
            buf.set_string(cx, y, &it.label, st);
            cx += crate::ui::text::width(&it.label) as u16;
            if it.busy {
                buf.set_string(
                    cx + 1,
                    y,
                    crate::widgets::progress::spinner_frame(ctx.interaction.tick),
                    st.fg(t.accent),
                );
                cx += 2;
            } else if it.error {
                buf.set_string(cx + 1, y, "!", st.fg(t.error));
                cx += 2;
            } else if it.dirty {
                buf.set_string(cx + 1, y, "•", st.fg(t.warning));
                cx += 2;
            }
            if it.closable {
                let cid = self.close_id(i);
                let ch = ctx.interaction.hovered(cid);
                let cs = if ch {
                    st.fg(t.text_primary).bg(t.lift(st.bg.unwrap_or(bg)))
                } else {
                    st.fg(t.text_faint)
                };
                buf.set_string(cx + 1, y, "×", cs);
                ctx.clickable(cid, Rect::new(cx + 1, y, 1, 1));
            }
            if active && area.height >= 2 {
                let rule = if self.quiet {
                    t.border_strong
                } else {
                    t.accent
                };
                for xx in x + 1..x + w - 1 {
                    buf.set_string(xx, y + 1, "━", ratatui::style::Style::new().fg(rule).bg(bg));
                }
            }
            ctx.clickable(tid, r);
            // close button on top
            if it.closable {
                ctx.clickable(self.close_id(i), Rect::new(cx + 1, y, 1, 1));
            }
            x += w + 1;
        }
        if overflow {
            let more_right = self.first + fit < self.items.len();
            let rx = area.right().saturating_sub(right_w + new_w);
            let st = if more_right {
                t.secondary().bg(bg)
            } else {
                t.faint().bg(bg)
            };
            let hovered = ctx.interaction.hovered(self.right_id());
            buf.set_string(
                rx,
                y,
                if more_right { " › " } else { "   " },
                if hovered && more_right {
                    st.bg(t.lift(bg))
                } else {
                    st
                },
            );
            if more_right {
                ctx.clickable(self.right_id(), Rect::new(rx, y, 3, 1));
            }
        }
        if self.allow_new {
            let nx = area.right().saturating_sub(new_w);
            let nid = self.new_id();
            let hovered = ctx.interaction.hovered(nid);
            let st = if hovered {
                t.primary().bg(t.lift(bg))
            } else {
                t.muted().bg(bg)
            };
            buf.set_string(nx, y, " + ", st);
            ctx.clickable(nid, Rect::new(nx, y, 3, 1));
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
    }
}
