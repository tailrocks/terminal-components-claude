//! Completion popup: an anchored, non-modal suggestion list. The owner keeps
//! keyboard focus and forwards keys; the popup only consumes navigation and
//! accept/dismiss keys.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Placement, place, surface};
use crate::ui::text::{truncate, width};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    /// One-cell kind glyph (T, V, C, K, F, S, A…).
    pub glyph: &'static str,
    pub detail: String,
    pub insert: String,
    /// Byte positions in `label` that matched the typed prefix.
    pub matched: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub id: WidgetId,
    pub items: Vec<CompletionItem>,
    pub cursor: usize,
    pub scroll: ScrollState,
    pub anchor: Rect,
    /// Bytes before the cursor to replace on accept.
    pub replace_len: usize,
    pub max_rows: u16,
    pub area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionEvent {
    Accept(usize),
    Dismiss,
}

impl Completion {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            items: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            anchor: Rect::ZERO,
            replace_len: 0,
            max_rows: 8,
            area: Rect::ZERO,
        }
    }

    pub fn open(&mut self, items: Vec<CompletionItem>, anchor: Rect, replace_len: usize) {
        self.items = items;
        self.cursor = 0;
        self.scroll = ScrollState::new(self.items.len());
        self.anchor = anchor;
        self.replace_len = replace_len;
    }

    pub fn is_open(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn close(&mut self) {
        self.items.clear();
    }

    pub fn current(&self) -> Option<&CompletionItem> {
        self.items.get(self.cursor)
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<CompletionEvent>) {
        if !self.is_open() {
            return (Outcome::Ignored, None);
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('n') if key.plain() || key.ctrl() => {
                if key.code == KeyCode::Char('n') && !key.ctrl() {
                    return (Outcome::Ignored, None);
                }
                self.cursor = (self.cursor + 1).min(self.items.len() - 1);
                self.scroll.ensure_visible(self.cursor);
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Char('p') if key.plain() || key.ctrl() => {
                if key.code == KeyCode::Char('p') && !key.ctrl() {
                    return (Outcome::Ignored, None);
                }
                self.cursor = self.cursor.saturating_sub(1);
                self.scroll.ensure_visible(self.cursor);
                (Outcome::Changed, None)
            }
            KeyCode::PageDown => {
                self.cursor = (self.cursor + self.max_rows as usize).min(self.items.len() - 1);
                self.scroll.ensure_visible(self.cursor);
                (Outcome::Changed, None)
            }
            KeyCode::PageUp => {
                self.cursor = self.cursor.saturating_sub(self.max_rows as usize);
                self.scroll.ensure_visible(self.cursor);
                (Outcome::Changed, None)
            }
            KeyCode::Tab | KeyCode::Enter => {
                (Outcome::Changed, Some(CompletionEvent::Accept(self.cursor)))
            }
            KeyCode::Esc => {
                self.close();
                (Outcome::Changed, Some(CompletionEvent::Dismiss))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> Option<CompletionEvent> {
        let i = self.locate(id)?;
        self.cursor = i;
        Some(CompletionEvent::Accept(i))
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        if !self.is_open() {
            return;
        }
        let t = ctx.theme;
        let label_w = self
            .items
            .iter()
            .map(|i| width(&i.label))
            .max()
            .unwrap_or(4);
        let detail_w = self
            .items
            .iter()
            .map(|i| width(&i.detail))
            .max()
            .unwrap_or(0);
        let w = (label_w + detail_w + 8).clamp(24, 48) as u16;
        let rows = (self.items.len() as u16).min(self.max_rows);
        let h = rows + 2;
        let area = place(screen, self.anchor, w, h, Placement::Below);
        self.area = area;
        eprintln!("DBG completion screen={screen:?} anchor={:?} area={area:?} buf={:?}", self.anchor, buf.area());
        let inner = surface(area, buf, ctx, t);
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(inner.height as usize);
        self.scroll.ensure_visible(self.cursor);
        let has_sb = self.scroll.overflows();
        let bg = t.surface_elevated;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let it = &self.items[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = i == self.cursor;
            let st = t.row(s, bg);
            let row = Rect::new(inner.x, y, inner.width.saturating_sub(u16::from(has_sb)), 1);
            fill(buf, row, st);
            buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(
                row.x + 1,
                y,
                it.glyph,
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.text_muted
                })
                .remove_modifier(Modifier::BOLD),
            );
            // label with matched chars bold
            let mut x = row.x + 3;
            let avail = row.width.saturating_sub(3) as usize;
            let show_detail =
                !it.detail.is_empty() && avail > width(&it.label) + width(&it.detail) + 2;
            let label = truncate(
                &it.label,
                if show_detail {
                    avail - width(&it.detail) - 2
                } else {
                    avail
                },
            );
            for (bi, ch) in label.char_indices() {
                let mut cs = st;
                if it.matched.contains(&bi) {
                    cs = cs.add_modifier(Modifier::BOLD);
                } else if !s.focused {
                    cs = cs.remove_modifier(Modifier::BOLD);
                }
                let g = ch.to_string();
                buf.set_string(x, y, &g, cs);
                x += width(&g) as u16;
            }
            if show_detail {
                let dx = row.right().saturating_sub(width(&it.detail) as u16 + 1);
                buf.set_string(
                    dx,
                    y,
                    &it.detail,
                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                );
            }
            ctx.clickable(rid, row);
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
        }
    }
}
