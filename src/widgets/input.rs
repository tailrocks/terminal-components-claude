use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::text::TextBuffer;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::widgets::field_common::{EditAction, edit_key};

/// Single-line text input with two modes:
/// - **navigation** (focused, not editing): the gutter bar shows focus, keys
///   like Tab move on, Enter or typing starts editing.
/// - **editing**: the hardware cursor is placed in the field, the field bg
///   drops to canvas, Enter commits, Esc reverts.
#[derive(Debug, Clone)]
pub struct TextInput {
    pub id: WidgetId,
    pub label: String,
    pub placeholder: String,
    pub buffer: TextBuffer,
    pub disabled: bool,
    pub required: bool,
    pub help: String,
    pub error: Option<String>,
    pub editing: bool,
    /// Value before editing began, for Esc.
    snapshot: String,
    /// Horizontal scroll (display columns).
    scroll: usize,
    pub area: Rect,
    /// Area of the text run (inside the field), for click-to-cursor.
    text_area: Rect,
    pub validator: Option<fn(&str) -> Option<String>>,
    /// Hide the "optional" suffix on non-required fields.
    pub plain_label: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Committed,
    Cancelled,
    /// Committed via Tab: caller should also move focus forward/backward.
    CommittedTab {
        backward: bool,
    },
    Changed,
}

impl TextInput {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            placeholder: String::new(),
            buffer: TextBuffer::single(""),
            disabled: false,
            required: false,
            help: String::new(),
            error: None,
            editing: false,
            snapshot: String::new(),
            scroll: 0,
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            validator: None,
            plain_label: false,
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }
    pub fn value(mut self, v: &str) -> Self {
        self.buffer.set_text(v);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.to_owned();
        self
    }
    pub fn plain_label(mut self) -> Self {
        self.plain_label = true;
        self
    }
    pub fn validator(mut self, v: fn(&str) -> Option<String>) -> Self {
        self.validator = Some(v);
        self
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn begin_edit(&mut self) {
        if self.disabled || self.editing {
            return;
        }
        self.editing = true;
        self.snapshot = self.buffer.text().to_owned();
        self.buffer.clear_selection();
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.buffer.clear_selection();
        self.validate();
    }

    pub fn cancel(&mut self) {
        self.editing = false;
        let snap = self.snapshot.clone();
        self.buffer.set_text(snap);
        self.validate();
    }

    pub fn validate(&mut self) -> bool {
        if let Some(v) = self.validator {
            self.error = v(self.buffer.text());
        } else if self.required && self.buffer.is_empty() {
            self.error = Some("Required".to_owned());
        }
        self.error.is_none()
    }

    /// Height needed: label + field + (help|error) line.
    pub const HEIGHT: u16 = 3;

    /// Handle a key while focused. The caller decides what to do with the
    /// returned event (e.g. move focus on `CommittedTab`).
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<InputEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if !self.editing {
            if key.is(KeyCode::Enter) || key.is(KeyCode::F(2)) {
                self.begin_edit();
                return (Outcome::Changed, None);
            }
            return (Outcome::Ignored, None);
        }
        match edit_key(key, false) {
            EditAction::Commit => {
                self.commit();
                (Outcome::Changed, Some(InputEvent::Committed))
            }
            EditAction::Cancel => {
                self.cancel();
                (Outcome::Changed, Some(InputEvent::Cancelled))
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
                self.live_validate();
                (Outcome::Changed, Some(InputEvent::Changed))
            }
            EditAction::Insert(c) => {
                self.buffer.insert_char(c);
                self.live_validate();
                (Outcome::Changed, Some(InputEvent::Changed))
            }
            EditAction::None => (Outcome::Consumed, None),
        }
    }

    /// Insert pasted text (only while editing).
    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if !self.editing || self.disabled {
            return Outcome::Ignored;
        }
        self.buffer.insert_str(text);
        self.live_validate();
        Outcome::Changed
    }

    fn live_validate(&mut self) {
        if self.error.is_some() {
            self.validate();
        }
    }

    /// Mouse click on the field. Focus is handled by the app; this places the
    /// cursor and enters editing if the field was already focused.
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
        let col = pos.x.saturating_sub(self.text_area.x) as usize + self.scroll;
        self.buffer.set_cursor_line_col(0, col);
        Outcome::Changed
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
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
            // lost focus while editing (e.g. mouse click elsewhere): commit
            self.commit();
            s.editing = false;
        }

        // label row
        let mut label = self.label.clone();
        if self.required {
            label.push_str(" *");
        } else if !self.label.is_empty() && !self.plain_label {
            label.push_str("  optional");
        }
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(s.focused).bg(bg)
        };
        let name_w = width(&self.label);
        buf.set_string(
            area.x + 2,
            area.y,
            crate::ui::text::fit(&label, area.width.saturating_sub(2) as usize),
            label_style,
        );
        if self.required && !self.disabled {
            buf.set_string(
                area.x + 2 + name_w as u16 + 1,
                area.y,
                "*",
                t.accent_fg().bg(bg),
            );
        } else if !self.required && !self.plain_label && !self.label.is_empty() {
            buf.set_string(
                area.x + 2 + name_w as u16 + 2,
                area.y,
                "optional",
                t.faint().bg(bg),
            );
        }

        // field row
        if area.height < 2 {
            return;
        }
        let field = Rect::new(area.x, area.y + 1, area.width, 1);
        self.area = field;
        let fs = t.field_style(s);
        crate::ui::ctx::fill(buf, field, fs);
        let gutter = t.gutter(s, fs.bg.unwrap_or(bg), false);
        buf.set_string(field.x, field.y, "▎", gutter);
        let trailing = if s.error { 2 } else { 0 };
        let inner = Rect::new(
            field.x + 2,
            field.y,
            field.width.saturating_sub(3 + trailing),
            1,
        );
        self.text_area = inner;
        let text = self.buffer.text();
        if text.is_empty() && !s.editing {
            let p = crate::ui::text::truncate(&self.placeholder, inner.width as usize);
            buf.set_string(inner.x, inner.y, &p, t.placeholder(s));
        } else {
            // horizontal scroll so the cursor stays visible
            let cursor_col = self.buffer.cursor_pos().col;
            let w = inner.width as usize;
            if w > 0 {
                if cursor_col < self.scroll {
                    self.scroll = cursor_col;
                } else if cursor_col >= self.scroll + w {
                    self.scroll = cursor_col + 1 - w;
                }
                let total = self.buffer.width();
                if !s.editing {
                    self.scroll = 0;
                }
                self.scroll = self.scroll.min(total.saturating_sub(w));
            }
            let sel = self.buffer.selection();
            let mut col = 0usize;
            let mut x = inner.x;
            let mut shown_left = false;
            for (bi, g) in unicode_segmentation::UnicodeSegmentation::grapheme_indices(text, true) {
                let gw = width(g);
                if col + gw <= self.scroll {
                    col += gw;
                    continue;
                }
                if x + gw as u16 > inner.right() {
                    break;
                }
                let mut st = fs;
                if let Some(r) = &sel
                    && r.contains(&bi)
                {
                    st = t.selection();
                }
                if s.editing {
                    st = st
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                }
                if !shown_left && self.scroll > 0 {
                    buf.set_string(x, inner.y, "…", fs.fg(t.text_muted));
                    shown_left = true;
                    x += 1;
                    col += gw;
                    continue;
                }
                buf.set_string(x, inner.y, g, st);
                x += gw as u16;
                col += gw;
            }
            if col < self.buffer.width() && inner.width > 0 {
                buf.set_string(inner.right() - 1, inner.y, "…", fs.fg(t.text_muted));
            }
            if s.editing {
                let cx = inner.x + (cursor_col - self.scroll) as u16;
                ctx.set_cursor(Position::new(cx.min(inner.right()), inner.y));
            }
        }
        if s.error {
            buf.set_string(
                field.right() - 2,
                field.y,
                "!",
                fs.fg(t.error).add_modifier(Modifier::BOLD),
            );
        }
        ctx.control(self.id, field, self.disabled);

        // help / error row
        if area.height >= 3 {
            let msg_y = area.y + 2;
            if let Some(e) = &self.error {
                buf.set_string(
                    area.x + 2,
                    msg_y,
                    crate::ui::text::truncate(e, area.width as usize - 2),
                    t.error_fg().bg(bg),
                );
            } else if !self.help.is_empty() {
                buf.set_string(
                    area.x + 2,
                    msg_y,
                    crate::ui::text::truncate(&self.help, area.width as usize - 2),
                    t.muted().bg(bg),
                );
            }
        }
    }
}
