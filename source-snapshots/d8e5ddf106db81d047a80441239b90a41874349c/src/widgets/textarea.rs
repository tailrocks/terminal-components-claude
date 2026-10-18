use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::core::text::{CursorPos, TextBuffer};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::input::InputEvent;
use crate::widgets::scrollbar;

/// Multi-line editor. Same two modes as [`TextInput`](super::input::TextInput);
/// in editing mode Enter inserts a newline and Esc *commits* (a document is
/// not cancelled by leaving it).
/// Long lines scroll horizontally with the insertion cursor. Explicit vertical
/// scrolling stays in place until editing or cursor movement requests follow.
#[derive(Debug, Clone)]
pub struct TextArea {
    pub id: WidgetId,
    pub label: String,
    pub placeholder: String,
    pub buffer: TextBuffer,
    pub disabled: bool,
    pub error: Option<String>,
    pub help: String,
    pub editing: bool,
    pub scroll: ScrollState,
    pub area: Rect,
    text_area: Rect,
    hscroll: usize,
    last_cursor: Option<CursorPos>,
    follow_cursor: bool,
    /// Fixed height of the text region (rows).
    pub rows: u16,
}

impl TextArea {
    pub fn new(id: WidgetId, label: &str, rows: u16) -> Self {
        Self {
            id,
            label: label.to_owned(),
            placeholder: String::new(),
            buffer: TextBuffer::multi(""),
            disabled: false,
            error: None,
            help: String::new(),
            editing: false,
            scroll: ScrollState::default(),
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            hscroll: 0,
            last_cursor: None,
            follow_cursor: false,
            rows,
        }
    }

    pub fn value(mut self, v: &str) -> Self {
        self.buffer.set_text(v);
        self.buffer.move_doc_start(false);
        self
    }
    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn error(mut self, e: Option<&str>) -> Self {
        self.error = e.map(str::to_owned);
        self
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.to_owned();
        self
    }

    pub fn height(&self) -> u16 {
        self.rows.saturating_add(2)
    }

    pub fn begin_edit(&mut self) {
        if !self.disabled {
            self.editing = true;
            self.follow_cursor = true;
        }
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.buffer.clear_selection();
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<InputEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if !self.editing {
            return match key.code {
                KeyCode::Enter | KeyCode::F(2) if key.plain() => {
                    self.begin_edit();
                    (Outcome::Changed, None)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.scroll.scroll_by(-1);
                    (Outcome::Changed, None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.scroll.scroll_by(1);
                    (Outcome::Changed, None)
                }
                KeyCode::PageUp => {
                    self.scroll.page_up();
                    (Outcome::Changed, None)
                }
                KeyCode::PageDown => {
                    self.scroll.page_down();
                    (Outcome::Changed, None)
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    self.scroll.jump_start();
                    (Outcome::Changed, None)
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.scroll.jump_end();
                    (Outcome::Changed, None)
                }
                _ => (Outcome::Ignored, None),
            };
        }
        self.follow_cursor = true;
        match edit_key(key, true) {
            EditAction::Commit | EditAction::Cancel => {
                self.commit();
                (Outcome::Changed, Some(InputEvent::Committed))
            }
            EditAction::Tab { backward } => {
                self.commit();
                (
                    Outcome::Changed,
                    Some(InputEvent::CommittedTab { backward }),
                )
            }
            EditAction::Apply(f) => {
                f(&mut self.buffer);
                (Outcome::Changed, Some(InputEvent::Changed))
            }
            EditAction::Insert(c) => {
                self.buffer.insert_char(c);
                (Outcome::Changed, Some(InputEvent::Changed))
            }
            EditAction::None => match key.code {
                KeyCode::PageUp => {
                    for _ in 0..self.rows {
                        self.buffer.move_up(false);
                    }
                    (Outcome::Changed, None)
                }
                KeyCode::PageDown => {
                    for _ in 0..self.rows {
                        self.buffer.move_down(false);
                    }
                    (Outcome::Changed, None)
                }
                _ => (Outcome::Consumed, None),
            },
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if !self.editing || self.disabled {
            return Outcome::Ignored;
        }
        self.buffer.insert_str(text);
        self.follow_cursor = true;
        Outcome::Changed
    }

    /// A click enters editing (one click, never two) and places the cursor
    /// at the pointer.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.begin_edit();
        let line = pos.y.saturating_sub(self.text_area.y) as usize + self.scroll.offset;
        let col = pos.x.saturating_sub(self.text_area.x) as usize + self.hscroll;
        let line = line.min(self.buffer.line_count().saturating_sub(1));
        self.buffer.set_cursor_line_col(line, col);
        self.follow_cursor = true;
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            self.area = Rect::ZERO;
            self.text_area = Rect::ZERO;
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        s.editing = self.editing && s.focused;
        s.error = self.error.is_some();
        if self.disabled {
            s.hovered = false;
        }
        if !s.focused && self.editing {
            self.commit();
            s.editing = false;
        }

        // label
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(s.focused).bg(bg)
        };
        buf.set_string(
            area.x + 2.min(area.width),
            area.y,
            crate::ui::text::truncate(&self.label, area.width.saturating_sub(2) as usize),
            label_style,
        );

        // body
        let rows = self.rows.min(area.height.saturating_sub(2));
        if rows == 0 {
            self.area = Rect::ZERO;
            self.text_area = Rect::ZERO;
            return;
        }
        let body = Rect::new(area.x, area.y + 1, area.width, rows);
        let resized = body.width != self.area.width || body.height != self.area.height;
        self.area = body;
        ctx.control(self.id, body, self.disabled);
        ctx.scrollable(self.id, body);
        let fs = t.field_style(s);
        crate::ui::ctx::fill(buf, body, fs);
        let gutter = t.gutter(s, fs.bg.unwrap_or(bg), false);
        for y in body.top()..body.bottom() {
            buf.set_string(body.x, y, t.gutter_symbol(s), gutter);
        }
        let inner = Rect::new(
            body.x + 2.min(body.width),
            body.y,
            body.width.saturating_sub(4),
            rows,
        );
        self.text_area = inner;

        let text = self.buffer.text();
        let lines: Vec<&str> = text.split('\n').collect();
        self.scroll.set_content(lines.len());
        self.scroll.set_viewport(rows as usize);
        let cur = self.buffer.cursor_pos();
        if s.editing && (self.follow_cursor || self.last_cursor != Some(cur) || resized) {
            self.scroll.ensure_visible(cur.line);
            let width = inner.width as usize;
            if cur.col < self.hscroll {
                self.hscroll = cur.col;
            } else if width > 0 && cur.col >= self.hscroll.saturating_add(width) {
                self.hscroll = cur.col + 1 - width;
            }
        } else if !s.editing {
            self.hscroll = 0;
        }
        self.last_cursor = Some(cur);
        self.follow_cursor = false;
        if text.is_empty() && !s.editing {
            let p = crate::ui::text::truncate(&self.placeholder, inner.width as usize);
            buf.set_string(inner.x, inner.y, &p, t.placeholder(s));
        } else {
            let sel = self.buffer.selection();
            let mut line_start = 0usize;
            for (li, line) in lines.iter().enumerate() {
                let visible = li >= self.scroll.offset && li < self.scroll.offset + rows as usize;
                if visible {
                    let y = inner.y + (li - self.scroll.offset) as u16;
                    let mut col = 0usize;
                    let clipped_right = width(line) > self.hscroll + inner.width as usize;
                    let available =
                        inner.width as usize - usize::from(clipped_right && inner.width > 0);
                    for (gi, g) in
                        unicode_segmentation::UnicodeSegmentation::grapheme_indices(*line, true)
                    {
                        let gw = width(g);
                        let start = col;
                        col += gw;
                        if col <= self.hscroll {
                            continue;
                        }
                        if start < self.hscroll {
                            if inner.width > 0 {
                                buf.set_string(inner.x, y, "…", fs.fg(t.text_muted));
                            }
                            continue;
                        }
                        let visible_col = start - self.hscroll;
                        if visible_col + gw > available {
                            break;
                        }
                        let mut st = fs;
                        if let Some(r) = &sel
                            && r.contains(&(line_start + gi))
                        {
                            st = t.selection();
                        }
                        buf.set_string(inner.x + visible_col as u16, y, g, st);
                    }
                    if clipped_right && inner.width > 0 {
                        buf.set_string(inner.right() - 1, y, "…", fs.fg(t.text_muted));
                    }
                    if s.editing && li == cur.line {
                        // accent underline on the cursor line marks where input goes
                        for xx in inner.x..inner.right() {
                            if let Some(c) = buf.cell_mut(Position::new(xx, y)) {
                                c.set_style(
                                    c.style()
                                        .add_modifier(Modifier::UNDERLINED)
                                        .underline_color(t.border_strong),
                                );
                            }
                        }
                    }
                }
                line_start += line.len() + 1;
            }
            if s.editing
                && self.scroll.visible_range().contains(&cur.line)
                && cur.col >= self.hscroll
                && cur.col - self.hscroll < inner.width as usize
            {
                let cy = inner.y + (cur.line - self.scroll.offset) as u16;
                let cx = inner.x + (cur.col - self.hscroll) as u16;
                ctx.set_cursor(Position::new(cx, cy));
            }
        }
        // scrollbar in the last column of the body
        crate::ui::fade::scroll_edges(
            buf,
            ctx,
            Rect::new(
                body.x,
                body.y,
                (body.right() - 1).saturating_sub(body.x),
                rows,
            ),
            &self.scroll,
        );
        let sb = Rect::new(body.right() - 1, body.y, 1, rows);
        scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, s.focused);
        if self.error.is_some() && body.width >= 2 {
            buf.set_string(
                body.right() - 2,
                body.y,
                "!",
                fs.fg(t.error).add_modifier(Modifier::BOLD),
            );
        }

        // footer row: help / error left, position right
        let fy = body.bottom();
        if fy < area.bottom() {
            let pos = if s.editing {
                format!("ln {}/{}", cur.line + 1, lines.len())
            } else if self.scroll.overflows() {
                scrollbar::position_label(&self.scroll)
            } else {
                String::new()
            };
            let pos = crate::ui::text::truncate(&pos, area.width.saturating_sub(3) as usize);
            let pos_w = if pos.is_empty() {
                0
            } else {
                crate::ui::text::width(&pos) as u16 + 3
            };
            let msg_w = area.width.saturating_sub(2 + pos_w) as usize;
            if let Some(e) = &self.error {
                buf.set_string(
                    area.x + 2,
                    fy,
                    crate::ui::text::truncate(e, msg_w),
                    t.error_fg().bg(bg),
                );
            } else if !self.help.is_empty() {
                buf.set_string(
                    area.x + 2,
                    fy,
                    crate::ui::text::truncate(&self.help, msg_w),
                    t.muted().bg(bg),
                );
            }
            if !pos.is_empty() {
                let px = area
                    .right()
                    .saturating_sub(crate::ui::text::width(&pos) as u16 + 1);
                buf.set_string(px, fy, &pos, t.faint().bg(bg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_and_end_scroll_a_read_only_view_and_boundary_wheels_are_consumed() {
        let text = (1..=30)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut ta = TextArea::new(WidgetId::of("t"), "Notes", 4).value(&text);
        let theme = crate::theme::Theme::junie();
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 7));
        ta.render(Rect::new(0, 0, 40, 7), &mut buf, &mut ctx, theme.canvas);
        assert!(ta.scroll.overflows());
        assert_eq!(ta.on_wheel(-1), Outcome::Consumed);
        let end = Key {
            code: KeyCode::End,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(ta.on_key(&end).0, Outcome::Changed);
        assert_eq!(ta.scroll.offset, ta.scroll.max_offset());
        let home = Key {
            code: KeyCode::Home,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(ta.on_key(&home).0, Outcome::Changed);
        assert_eq!(ta.scroll.offset, 0);
    }
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    fn render(
        input: &mut TextArea,
        area: Rect,
        buf: &mut Buffer,
        level: ColorLevel,
    ) -> Option<Position> {
        let theme = Theme::for_level(level);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(input.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        input.render(area, buf, &mut ctx, theme.surface);
        ctx.cursor
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn long_unicode_lines_follow_cursor_and_click_the_visible_position() {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let mut input =
                TextArea::new(WidgetId::of("long"), "Notes", 3).value("日本語👩‍💻cafe\u{301}");
            input.begin_edit();
            input.on_key(&key(KeyCode::End));
            let area = Rect::new(0, 0, 10, 5);
            let mut buf = Buffer::empty(area);
            let cursor = render(&mut input, area, &mut buf, level).unwrap();
            assert!(input.text_area.contains(cursor));
            assert!(input.hscroll > 0);
            input.on_click(cursor);
            assert_eq!(input.buffer.cursor_offset(), input.buffer.text().len());
            input.on_paste("🙂");
            let cursor = render(&mut input, area, &mut buf, level).unwrap();
            assert!(input.text_area.contains(cursor));
            assert_eq!(buf[(cursor.x - 2, cursor.y)].symbol(), "🙂");
            input.on_key(&key(KeyCode::Home));
            let cursor = render(&mut input, area, &mut buf, level).unwrap();
            assert_eq!(cursor.x, input.text_area.x);
            assert_eq!(input.hscroll, 0);
        }
    }

    #[test]
    fn manual_scroll_stays_until_cursor_movement_or_editing() {
        let text = (0..20).map(|i| format!("line {i}\n")).collect::<String>();
        let mut input = TextArea::new(WidgetId::of("scroll"), "Notes", 3).value(&text);
        input.begin_edit();
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf, ColorLevel::TrueColor);
        input.on_wheel(8);
        assert!(render(&mut input, area, &mut buf, ColorLevel::TrueColor).is_none());
        assert_eq!(input.scroll.offset, 8);
        input.on_key(&key(KeyCode::Right));
        assert!(render(&mut input, area, &mut buf, ColorLevel::TrueColor).is_some());
        assert_eq!(input.scroll.offset, 0);
        input.scroll.scroll_to(12);
        render(&mut input, area, &mut buf, ColorLevel::TrueColor);
        assert_eq!(input.scroll.offset, 12);
        input.on_paste("x");
        assert!(render(&mut input, area, &mut buf, ColorLevel::TrueColor).is_some());
        assert_eq!(input.scroll.offset, 0);
    }

    #[test]
    fn resizing_keeps_long_line_cursor_visible() {
        let mut input =
            TextArea::new(WidgetId::of("resize"), "Notes", 3).value("日本語👩‍💻a very long line");
        input.begin_edit();
        input.on_key(&key(KeyCode::End));
        for width in [40, 10, 7, 30] {
            let area = Rect::new(0, 0, width, 5);
            let mut buf = Buffer::empty(area);
            let cursor = render(&mut input, area, &mut buf, ColorLevel::Mono).unwrap();
            assert!(input.text_area.contains(cursor));
        }
    }

    #[test]
    fn narrow_textareas_do_not_write_outside_their_allocation() {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            for width in 0..12 {
                for height in 0..6 {
                    let mut input = TextArea::new(WidgetId::of("narrow"), "Long label", 3)
                        .value("日本語👩‍💻abcdef\nnext")
                        .help("Some help")
                        .error(Some("Invalid"));
                    input.begin_edit();
                    input.buffer.move_doc_end(false);
                    let area = Rect::new(3, 2, width, height);
                    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 10));
                    let cursor = render(&mut input, area, &mut buf, level);
                    assert!(cursor.is_none_or(|p| area.contains(p)));
                    for y in 0..10 {
                        for x in 0..20 {
                            if !area.contains(Position::new(x, y)) {
                                assert_eq!(buf[(x, y)].symbol(), " ", "{area:?}: ({x}, {y})");
                            }
                        }
                    }
                }
            }
        }
    }
}
