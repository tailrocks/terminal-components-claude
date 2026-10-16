//! Completion popup: an anchored, non-modal suggestion list. The owner keeps
//! keyboard focus and forwards keys; the popup only consumes navigation and
//! accept/dismiss keys.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::scroll::ScrollState;
use crate::theme::CursorStyle;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Placement, backdrop_id, place_with, surface_for};
use crate::ui::text::{truncate, width};
use crate::widgets::scrollbar;

/// Columns from the popup's left edge to an item label: the frame, the
/// cursor gutter, the kind glyph and a space.
pub const TEXT_INSET: u16 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    /// One-cell kind glyph (T, V, C, K, F, S, A…).
    pub glyph: std::borrow::Cow<'static, str>,
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
    /// Popup width bounds `(min, max)` in cells. Default `(24, 48)`.
    pub popup_width: (u16, u16),
    /// How the list keeps off the screen edges and lines up with the typed
    /// text. Default: one free cell from every edge, and the item labels
    /// start in the anchor's column ([`TEXT_INSET`]).
    pub fit: crate::ui::popup::Place,
    /// Which side of the anchor the list opens on. Default
    /// [`Placement::Below`] (flipping above when there is no room).
    pub placement: Placement,
    pub area: Rect,
    /// How the list closes without a choice. Default: Esc, or a click
    /// outside that also lands where it was aimed (placing the caret in the
    /// editor underneath).
    pub dismiss: Dismiss,
    /// How the cursor row is marked. Default [`CursorStyle::Fill`]: the
    /// editor that owns the list keeps the only focus bar.
    pub cursor_style: CursorStyle,
    /// Which keys move the list cursor. Default [`Completion::NAV`]: ↑/↓
    /// and paging only, since letters, ←/→ and Home/End belong to the
    /// editor the list completes for. Ctrl+N/Ctrl+P always step.
    pub nav: NavKeys,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionEvent {
    Accept(usize),
    Dismiss,
}

impl Completion {
    /// Default [`Completion::nav`].
    pub const NAV: NavKeys = NavKeys::NO_VIM.home_end(false);

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// Most rows shown before the list scrolls. Default 8.
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows;
        self
    }

    /// Popup width bounds in cells. Default `(24, 48)`.
    pub fn popup_width(mut self, min: u16, max: u16) -> Self {
        self.popup_width = (min, max.max(min));
        self
    }

    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            items: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            anchor: Rect::ZERO,
            replace_len: 0,
            max_rows: 8,
            popup_width: (24, 48),
            fit: crate::ui::popup::Place::DEFAULT.text_inset(TEXT_INSET),
            placement: Placement::Below,
            area: Rect::ZERO,
            dismiss: Dismiss::LIGHT.pass_through(true),
            cursor_style: CursorStyle::default(),
            nav: Self::NAV,
        }
    }

    /// See [`Self::placement`].
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// See [`Self::fit`].
    pub fn fit(mut self, fit: crate::ui::popup::Place) -> Self {
        self.fit = fit;
        self
    }

    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = cursor_style;
        self
    }

    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
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
        id == self.id
            || id == scrollbar::id_for(self.id)
            || (self.is_open() && id == backdrop_id(self.id))
            || self.locate(id).is_some()
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<CompletionEvent>) {
        if !self.is_open() {
            return (Outcome::Ignored, None);
        }
        let last = self.items.len() - 1;
        let page = self.scroll.viewport_len.max(1);
        let up_down = key.plain() || key.ctrl();
        let to = match (self.nav.action(key), key.code) {
            (Some(Nav::Down), _) if up_down => Some((self.cursor + 1).min(last)),
            (Some(Nav::Up), _) if up_down => Some(self.cursor.saturating_sub(1)),
            (Some(Nav::PageDown), _) => Some((self.cursor + page).min(last)),
            (Some(Nav::PageUp), _) => Some(self.cursor.saturating_sub(page)),
            (Some(Nav::Home), _) => Some(0),
            (Some(Nav::End), _) => Some(last),
            // the editor's emacs keys
            (_, KeyCode::Char('n')) if key.ctrl() => Some((self.cursor + 1).min(last)),
            (_, KeyCode::Char('p')) if key.ctrl() => Some(self.cursor.saturating_sub(1)),
            _ => None,
        };
        if let Some(to) = to {
            self.cursor = to;
            self.scroll.ensure_visible(self.cursor);
            return (Outcome::Changed, None);
        }
        match key.code {
            KeyCode::Tab | KeyCode::Enter => {
                (Outcome::Changed, Some(CompletionEvent::Accept(self.cursor)))
            }
            KeyCode::Esc if self.dismiss.escape => {
                self.close();
                (Outcome::Changed, Some(CompletionEvent::Dismiss))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> Option<CompletionEvent> {
        if id == backdrop_id(self.id) {
            if !self.dismiss.outside_click {
                return None;
            }
            self.close();
            return Some(CompletionEvent::Dismiss);
        }
        let i = self.locate(id)?;
        self.cursor = i;
        Some(CompletionEvent::Accept(i))
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        if !self.is_open() {
            return;
        }
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
        let w = (label_w + detail_w + 8)
            .clamp(self.popup_width.0 as usize, self.popup_width.1 as usize) as u16;
        let rows = (self.items.len() as u16).min(self.max_rows);
        let h = rows + 2;
        let area = place_with(screen, self.anchor, w, h, self.placement, self.fit);
        self.area = area;
        let inner = area
            .intersection(*buf.area())
            .inner(ratatui::layout::Margin::new(1, 1));
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(inner.height as usize);
        // drawn as an overlay: above whatever is rendered after the editor
        let me = self.clone();
        ctx.overlay(move |buf, ctx| me.paint(area, buf, ctx));
    }

    fn paint(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let inner = surface_for(area, buf, ctx, t, Some((self.id, self.dismiss)));
        // the key handlers pull the viewport to the cursor; a render never
        // does, so a wheel scroll survives the next frame
        let has_sb = self.scroll.overflows();
        let bg = t.surface_elevated;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let it = &self.items[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = i == self.cursor;
            let cr = t.cursor_row(s, bg, self.cursor_style);
            let st = cr.style;
            let row = Rect::new(inner.x, y, inner.width.saturating_sub(u16::from(has_sb)), 1);
            fill(buf, row, st);
            buf.set_string(row.x, y, cr.gutter, cr.gutter_style);
            buf.set_string(
                row.x + 1,
                y,
                &it.glyph,
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.row_ink(&cr, t.text_muted)
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
                    st.fg(t.row_ink(&cr, t.text_muted))
                        .remove_modifier(Modifier::BOLD),
                );
            }
            ctx.clickable(rid, row);
        }
        if has_sb {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    inner.x,
                    inner.y,
                    (inner.right() - 1).saturating_sub(inner.x),
                    inner.height,
                ),
                &self.scroll,
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn builders_set_rows_and_width_bounds() {
        let c = Completion::new(WidgetId::of("c"))
            .max_rows(4)
            .popup_width(30, 20);
        assert_eq!(c.max_rows, 4);
        assert_eq!(c.popup_width, (30, 30), "max never under min");
    }

    fn popup(rows: u16) -> Completion {
        let mut c = Completion::new(WidgetId::of("c"));
        c.max_rows = rows;
        c.open(
            (0..40)
                .map(|i| CompletionItem {
                    label: format!("item{i:02}"),
                    glyph: "T".into(),
                    detail: String::new(),
                    insert: format!("item{i:02}"),
                    matched: vec![],
                })
                .collect(),
            Rect::new(0, 0, 1, 1),
            0,
        );
        c
    }

    fn draw(c: &mut Completion, h: u16) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, h));
        c.render(Rect::new(0, 0, 60, h), &mut buf, &mut ctx);
        ctx.flush_overlays(&mut buf);
        buf
    }

    #[test]
    fn the_list_opens_on_the_configured_side_of_the_anchor() {
        let mut c = popup(4);
        c.anchor = Rect::new(10, 10, 1, 1);
        draw(&mut c, 24);
        assert_eq!(c.area.y, 11, "below by default: {:?}", c.area);
        let mut c = popup(4).placement(Placement::Above);
        c.anchor = Rect::new(10, 10, 1, 1);
        draw(&mut c, 24);
        assert_eq!(c.area.bottom(), 10, "above on request: {:?}", c.area);
    }

    fn bars(buf: &Buffer) -> usize {
        buf.content().iter().filter(|c| c.symbol() == "▎").count()
    }

    #[test]
    fn the_cursor_row_is_a_fill_so_the_editor_keeps_the_only_focus_bar() {
        let t = Theme::junie();
        let mut c = popup(8);
        let buf = draw(&mut c, 24);
        assert_eq!(bars(&buf), 0, "no second focus bar beside the editor's");
        let row = c.area.y + 1;
        let label = buf[(c.area.x + 4, row)].style();
        assert_eq!(
            label.bg,
            Some(t.highlight),
            "cursor row is the highlight fill"
        );
        assert_ne!(
            label.bg,
            Some(t.accent_bg),
            "green stays reserved for focus"
        );
        // the in-page look stays available
        let mut barred = popup(8).cursor_style(CursorStyle::Bar);
        let buf = draw(&mut barred, 24);
        assert_eq!(bars(&buf), 1);
    }

    #[test]
    fn a_wheel_scroll_survives_the_next_render_and_pages_follow_the_viewport() {
        let mut c = popup(8);
        draw(&mut c, 24);
        assert_eq!(c.on_wheel(3), Outcome::Changed);
        assert_eq!(c.scroll.offset, 3);
        draw(&mut c, 24);
        assert_eq!(
            c.scroll.offset, 3,
            "a render never pulls the view back to the cursor"
        );
        assert_eq!(c.on_wheel(-3), Outcome::Changed);
        assert_eq!(c.on_wheel(-3), Outcome::Consumed);
        // clipped by a short screen: the page is the viewport, not the config
        let mut short = popup(8);
        draw(&mut short, 6);
        let view = short.scroll.viewport_len;
        assert!(view < 8 && view > 0, "{view}");
        let key = Key {
            code: KeyCode::PageDown,
            mods: KeyModifiers::NONE,
        };
        short.on_key(&key);
        assert_eq!(short.cursor, view, "one page is one viewport");
        assert!(short.scroll.visible_range().contains(&short.cursor));
    }
}
