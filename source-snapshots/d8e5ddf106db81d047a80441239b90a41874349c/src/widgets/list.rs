use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

#[derive(Debug, Clone)]
pub struct ListItem {
    pub label: String,
    pub meta: Option<String>,
    pub disabled: bool,
}

impl ListItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            meta: None,
            disabled: false,
        }
    }
    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Single,
    Multi,
}

/// Scrollable list. The list is one focus stop; `cursor` is the keyboard row.
/// Single mode: `›` marks the chosen item. Multi mode: `✓` marks each.
#[derive(Debug, Clone)]
pub struct ListBox {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    pub id: WidgetId,
    pub items: Vec<ListItem>,
    pub cursor: usize,
    pub mode: SelectMode,
    pub chosen: Option<usize>,
    pub checked: Vec<bool>,
    pub scroll: ScrollState,
    pub area: Rect,
    pub empty_text: String,
    /// Range-select anchor for Shift+arrows in multi mode.
    anchor: Option<usize>,
}

impl ListBox {
    pub fn new(id: WidgetId, items: Vec<ListItem>, mode: SelectMode) -> Self {
        let n = items.len();
        Self {
            id,
            items,
            cursor: 0,
            mode,
            chosen: None,
            checked: vec![false; n],
            scroll: ScrollState::new(n),
            area: Rect::ZERO,
            track: Rect::ZERO,
            empty_text: "Nothing here yet".to_owned(),
            anchor: None,
        }
    }

    pub fn empty_text(mut self, s: &str) -> Self {
        self.empty_text = s.to_owned();
        self
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn checked_count(&self) -> usize {
        self.checked.iter().filter(|c| **c).count()
    }

    fn move_cursor(&mut self, to: usize, extend: bool) {
        let to = to.min(self.items.len().saturating_sub(1));
        if self.mode == SelectMode::Multi && extend {
            let anchor = *self.anchor.get_or_insert(self.cursor);
            let (a, b) = (anchor.min(to), anchor.max(to));
            for i in a..=b {
                if !self.items[i].disabled {
                    self.checked[i] = true;
                }
            }
        } else {
            self.anchor = None;
        }
        self.cursor = to;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn activate(&mut self, i: usize) -> Outcome {
        if i >= self.items.len() || self.items[i].disabled {
            return Outcome::Consumed;
        }
        match self.mode {
            SelectMode::Single => self.chosen = Some(i),
            SelectMode::Multi => self.checked[i] = !self.checked[i],
        }
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.items.is_empty() {
            return Outcome::Ignored;
        }
        let shift = key.shift();
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                self.move_cursor(self.cursor.saturating_sub(1), shift);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                self.move_cursor(self.cursor + 1, shift);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.move_cursor(
                    self.cursor.saturating_sub(self.scroll.viewport_len.max(1)),
                    shift,
                );
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.move_cursor(self.cursor + self.scroll.viewport_len.max(1), shift);
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.move_cursor(0, shift);
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.move_cursor(usize::MAX, shift);
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.cursor),
            KeyCode::Char('a') if self.mode == SelectMode::Multi => {
                let all = self
                    .items
                    .iter()
                    .zip(&self.checked)
                    .filter(|(it, _)| !it.disabled)
                    .all(|(_, c)| *c);
                for (it, c) in self.items.iter().zip(self.checked.iter_mut()) {
                    if !it.disabled {
                        *c = !all;
                    }
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, row: usize) -> Outcome {
        if row >= self.items.len() {
            return Outcome::Consumed;
        }
        self.cursor = row;
        self.anchor = None;
        self.scroll.ensure_visible(row);
        self.activate(row)
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Which visible row a widget id refers to.
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }

    /// True if `id` is the list or any of its parts.
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    fn track(&self) -> Rect {
        if self.track.is_empty() {
            Rect::new(
                self.area.right().saturating_sub(1),
                self.area.y,
                1,
                self.area.height,
            )
        } else {
            self.track
        }
    }

    /// The pointer went down on the scrollbar (or a completed click): a
    /// press on the thumb grabs it, a press on the track jumps to it.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        if scrollbar::press(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        if scrollbar::drag(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(area.height as usize);
        // the container registers first so rows drawn later win hit-testing
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        if self.items.is_empty() {
            let msg = crate::ui::text::truncate(&self.empty_text, area.width as usize);
            let y = area.y + area.height / 2;
            let x = area.x
                + (area
                    .width
                    .saturating_sub(crate::ui::text::width(&msg) as u16))
                    / 2;
            buf.set_string(x, y, &msg, t.muted().bg(bg));
            return;
        }
        let has_sb = self.scroll.overflows();
        let text_w = area.width.saturating_sub(if has_sb { 2 } else { 1 }) as usize;
        for (i, li) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let item = &self.items[li];
            let row = Rect::new(
                area.x,
                y,
                area.width.saturating_sub(if has_sb { 1 } else { 0 }),
                1,
            );
            let rid = self.row_id(li);
            let mut s = ctx.state(rid);
            s.focused = focused && li == self.cursor;
            s.disabled = item.disabled;
            s.selected = match self.mode {
                SelectMode::Single => self.chosen == Some(li),
                SelectMode::Multi => self.checked[li],
            };
            if item.disabled {
                s.hovered = false;
            }
            let st = t.row(s, bg);
            fill(buf, row, st);
            buf.set_string(
                row.x,
                y,
                t.gutter_symbol(s),
                t.gutter(s, st.bg.unwrap_or(bg), false),
            );
            let marker = match (self.mode, s.selected) {
                (SelectMode::Single, true) => "›",
                (SelectMode::Multi, true) => "✓",
                _ => " ",
            };
            let ms = if s.selected && !item.disabled {
                st.fg(if focused || s.hovered {
                    t.accent
                } else {
                    t.text_secondary
                })
            } else {
                st
            };
            buf.set_string(row.x + 1, y, marker, ms);
            let label_w = text_w.saturating_sub(2);
            let meta_w = item
                .meta
                .as_ref()
                .map(|m| crate::ui::text::width(m))
                .unwrap_or(0);
            // hide metadata rather than starve the label
            let meta_w = if label_w.saturating_sub(meta_w + 2) < 12 {
                0
            } else {
                meta_w
            };
            let lw = label_w.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 });
            buf.set_string(row.x + 3, y, crate::ui::text::fit(&item.label, lw), st);
            if let Some(m) = &item.meta
                && meta_w > 0
                && meta_w + 4 < label_w
            {
                let mx = row.right().saturating_sub(meta_w as u16 + 1);
                let ms = if item.disabled {
                    st
                } else {
                    st.fg(t.text_muted)
                };
                buf.set_string(mx, y, m, ms);
            }
            ctx.clickable(rid, row);
        }
        self.track = Rect::ZERO;
        if has_sb {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    area.height,
                ),
                &self.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn render(l: &mut ListBox) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
        l.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
        (0..6)
            .map(|y| {
                (0..40)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn scroll_edges_fade_toward_hidden_content_and_never_the_cursor_row() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let theme = Theme::junie();
        let render = |l: &mut ListBox| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
            l.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
            buf
        };
        let grey = |buf: &Buffer, y: u16| match buf[(4, y)].fg {
            Color::Rgb(r, _, _) => r,
            other => panic!("{other:?}"),
        };
        // at the top: the last row hints at more below, the first row is whole
        let b = render(&mut l);
        assert_eq!(b[(4, 0)].fg, theme.text_primary, "cursor row untouched");
        assert!(
            grey(&b, 5) < grey(&b, 4),
            "bottom row fades: {}",
            grey(&b, 5)
        );
        // scrolled: the top row fades too, the cursor row never does
        l.on_wheel(10);
        let b = render(&mut l);
        assert!(grey(&b, 0) < grey(&b, 1), "top row fades");
        assert!(grey(&b, 5) < grey(&b, 4));
        // at the end: only the top row hints
        l.scroll.jump_end();
        let b = render(&mut l);
        assert!(grey(&b, 0) < grey(&b, 1));
        assert_eq!(grey(&b, 5), grey(&b, 4), "nothing below, no fade");
    }

    #[test]
    fn boundary_wheels_are_consumed_and_the_thumb_drags_with_its_grab() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        assert_eq!(l.on_wheel(-1), Outcome::Consumed, "nothing above");
        render(&mut l);
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        assert_eq!(l.on_wheel(-1), Outcome::Changed);
        assert_eq!(l.on_wheel(-1), Outcome::Consumed);
        // the track is the drawn scrollbar: six rows, thumb one row at the top
        let x = 39;
        assert_eq!(
            l.on_scrollbar(Position::new(x, 0)),
            Outcome::Consumed,
            "a press on the thumb"
        );
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 1)), Outcome::Changed);
        let after_one = l.scroll.offset;
        assert!(after_one > 0 && after_one < 10, "{after_one}");
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 5)), Outcome::Changed);
        assert_eq!(
            l.scroll.offset,
            l.scroll.max_offset(),
            "the end of the track is the end"
        );
        assert_eq!(
            l.on_scrollbar_drag(Position::new(x, 40)),
            Outcome::Consumed,
            "past the track clamps"
        );
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 0)), Outcome::Changed);
        assert_eq!(l.scroll.offset, 0);
    }

    #[test]
    fn wheel_moves_the_viewport_and_keeps_the_cursor() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let a = render(&mut l);
        assert!(a.contains("Row 00"));
        l.on_wheel(4);
        let b = render(&mut l);
        assert!(!b.contains("Row 00"));
        assert!(b.contains("Row 04"));
        assert_eq!(l.scroll.offset, 4);
        assert_eq!(render(&mut l), b, "render does not reset a wheel scroll");
        assert_eq!(l.cursor, 0);
        let key = Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        l.on_key(&key);
        render(&mut l);
        assert!(l.scroll.visible_range().contains(&l.cursor));
        l.on_wheel(-40);
        assert_eq!(l.scroll.offset, 0);
    }
}
