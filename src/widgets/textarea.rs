use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::core::text::TextBuffer;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::input::InputEvent;
use crate::widgets::scrollbar;

/// Multi-line editor. Same two modes as [`TextInput`](super::input::TextInput);
/// in editing mode Enter inserts a newline and Esc *commits* (a document is
/// not cancelled by leaving it).
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
        self.rows + 2
    }

    pub fn begin_edit(&mut self) {
        if !self.disabled {
            self.editing = true;
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
                _ => (Outcome::Ignored, None),
            };
        }
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
        Outcome::Changed
    }

    pub fn on_click(&mut self, pos: Position, was_focused: bool) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        if !self.editing {
            if was_focused {
                self.begin_edit();
            } else {
                return Outcome::Changed;
            }
        }
        let line = pos.y.saturating_sub(self.text_area.y) as usize + self.scroll.offset;
        let col = pos.x.saturating_sub(self.text_area.x) as usize;
        let line = line.min(self.buffer.line_count().saturating_sub(1));
        self.buffer.set_cursor_line_col(line, col);
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
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
        buf.set_string(area.x + 2, area.y, &self.label, label_style);

        // body
        let rows = self.rows.min(area.height.saturating_sub(2));
        if rows == 0 {
            return;
        }
        let body = Rect::new(area.x, area.y + 1, area.width, rows);
        self.area = body;
        ctx.control(self.id, body, self.disabled);
        ctx.scrollable(self.id, body);
        let fs = t.field_style(s);
        crate::ui::ctx::fill(buf, body, fs);
        let gutter = t.gutter(s, fs.bg.unwrap_or(bg), false);
        for y in body.top()..body.bottom() {
            buf.set_string(body.x, y, "▎", gutter);
        }
        let inner = Rect::new(body.x + 2, body.y, body.width.saturating_sub(4), rows);
        self.text_area = inner;

        let text = self.buffer.text();
        let lines: Vec<&str> = text.split('\n').collect();
        self.scroll.set_content(lines.len());
        self.scroll.set_viewport(rows as usize);
        let cur = self.buffer.cursor_pos();
        if s.editing {
            self.scroll.ensure_visible(cur.line);
        }
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
                    let mut x = inner.x;
                    for (gi, g) in
                        unicode_segmentation::UnicodeSegmentation::grapheme_indices(*line, true)
                    {
                        let gw = width(g) as u16;
                        if x + gw > inner.right() {
                            buf.set_string(
                                inner.right().saturating_sub(1),
                                y,
                                "…",
                                fs.fg(t.text_muted),
                            );
                            break;
                        }
                        let mut st = fs;
                        if let Some(r) = &sel
                            && r.contains(&(line_start + gi))
                        {
                            st = t.selection();
                        }
                        buf.set_string(x, y, g, st);
                        x += gw;
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
            if s.editing {
                let cy = inner.y + (cur.line - self.scroll.offset) as u16;
                let cx = inner.x + (cur.col as u16).min(inner.width);
                ctx.set_cursor(Position::new(cx, cy));
            }
        }
        // scrollbar in the last column of the body
        let sb = Rect::new(body.right() - 1, body.y, 1, rows);
        scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, s.focused);
        if self.error.is_some() {
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
