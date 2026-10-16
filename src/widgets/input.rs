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
///   like Tab move on, Enter or typing starts editing; a mouse click starts
///   editing at the pointer in one go.
/// - **editing**: the hardware cursor is placed in the field, the field bg
///   drops to canvas, Enter commits, Esc reverts.
/// A text validator: `Some(message)` rejects the value.
#[derive(Clone)]
pub struct TextValidator(pub std::rc::Rc<dyn Fn(&str) -> Option<String>>);

impl std::fmt::Debug for TextValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TextValidator")
    }
}

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
    /// Validation hook; a closure may capture state (an allowed list, a
    /// schema).
    pub validator: Option<TextValidator>,
    /// Hide the "optional" suffix on non-required fields.
    pub plain_label: bool,
    /// Draw every grapheme as `•`. `text()` still returns the raw value,
    /// which is transient edit state only: never log or render it.
    pub masked: bool,
    /// While not editing, show the last N graphemes in clear after the
    /// mask (`••••••••k7Qz`). Editing always masks everything.
    pub reveal_tail: u8,
    /// Wrap a label longer than the field onto several rows instead of
    /// cutting it with `…`. Off by default (form labels are short); a
    /// typed-acknowledgement prompt turns it on, because its label is the
    /// instruction and must be read whole. Measure with [`Self::height_for`].
    pub wrap_label: bool,
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
            masked: false,
            reveal_tail: 0,
            wrap_label: false,
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
    pub fn validator(mut self, v: impl Fn(&str) -> Option<String> + 'static) -> Self {
        self.validator = Some(TextValidator(std::rc::Rc::new(v)));
        self
    }
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }
    pub fn reveal_tail(mut self, n: u8) -> Self {
        self.reveal_tail = n;
        self
    }
    /// See [`Self::wrap_label`].
    pub fn wrap_label(mut self, on: bool) -> Self {
        self.wrap_label = on;
        self
    }

    /// Overwrite and drop the value (owners clear secrets this way).
    pub fn clear(&mut self) {
        self.buffer.set_text("");
        self.snapshot.clear();
        self.error = None;
    }

    /// Graphemes as they will be drawn: raw, or `•` per grapheme when masked
    /// (the tail stays in clear while not editing).
    fn display_graphemes(&self, editing: bool) -> Vec<(usize, String, usize)> {
        let text = self.buffer.text();
        let gs: Vec<(usize, &str)> =
            unicode_segmentation::UnicodeSegmentation::grapheme_indices(text, true).collect();
        let n = gs.len();
        gs.into_iter()
            .enumerate()
            .map(|(i, (bi, g))| {
                let reveal = self.masked
                    && !editing
                    && self.reveal_tail > 0
                    && i + (self.reveal_tail as usize) >= n;
                if self.masked && !reveal {
                    (bi, "•".to_owned(), 1)
                } else {
                    (bi, g.to_owned(), width(g))
                }
            })
            .collect()
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

    /// Recompute validation from the current value. A custom validator owns
    /// validation when supplied; otherwise `required` rejects an empty value.
    pub fn validate(&mut self) -> bool {
        self.error = if let Some(v) = &self.validator {
            (v.0)(self.buffer.text())
        } else if self.required && self.buffer.is_empty() {
            Some("Required".to_owned())
        } else {
            None
        };
        self.error.is_none()
    }

    /// Height of the tallest single-label field: label + field +
    /// (help|error) line. Layouts that reserve a fixed slot use it; a
    /// layout that measures uses [`Self::height`] or [`Self::height_for`].
    pub const HEIGHT: u16 = 3;

    /// Columns the field's focus gutter takes left of the label and the
    /// text. Render the field at `x - GUTTER` to line its label up with
    /// text that starts at `x` (a dialog title, a paragraph).
    pub const GUTTER: u16 = 2;

    /// Rows this field draws: the label, the field, and the help or error
    /// row only when there is something to show in it.
    pub fn height(&self) -> u16 {
        2 + self.message_rows()
    }

    /// [`Self::height`] for a field `width` columns wide, counting every
    /// row of a wrapped label ([`Self::wrap_label`]).
    pub fn height_for(&self, cols: u16) -> u16 {
        self.label_lines(cols).len().max(1) as u16 + 1 + self.message_rows()
    }

    fn message_rows(&self) -> u16 {
        u16::from(self.error.is_some() || !self.help.is_empty())
    }

    /// The label as drawn in a field `width` wide: one line, unless the
    /// label wraps and is longer than the room beside the gutter.
    fn label_lines(&self, cols: u16) -> Vec<String> {
        let room = cols.saturating_sub(Self::GUTTER) as usize;
        if self.wrap_label && room > 0 && width(&self.label) > room {
            crate::ui::text::wrap(&self.label, room)
        } else {
            vec![self.label.clone()]
        }
    }

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
    /// A paste is typing: a focused field that is not editing yet starts
    /// editing, exactly as the first typed character would.
    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        self.begin_edit();
        self.buffer.insert_str(text);
        self.live_validate();
        Outcome::Changed
    }

    fn live_validate(&mut self) {
        if self.error.is_some() {
            self.validate();
        }
    }

    /// Mouse click on the field. Focus is handled by the app; the click
    /// enters editing (one click, never two) and places the cursor at the
    /// pointer.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.begin_edit();
        // Mouse positions use the same display geometry as rendering. A
        // masked CJK/emoji grapheme occupies one cell, regardless of raw width.
        let col = pos.x.saturating_sub(self.text_area.x) as usize + self.scroll;
        let mut width = 0;
        let mut offset = self.buffer.text().len();
        for (bi, _, gw) in self.display_graphemes(true) {
            if width + gw > col {
                offset = bi;
                break;
            }
            width += gw;
        }
        self.buffer.select_range(offset, offset);
        self.buffer.clear_selection();
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
        self.area = Rect::ZERO;
        self.text_area = Rect::ZERO;
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
        if !ctx.interaction.has_focus(self.id) && self.editing {
            // lost focus while editing (e.g. mouse click elsewhere): commit
            self.commit();
            s.editing = false;
        }

        // a wrapped label takes as many rows as it needs, every one whole
        let wrapped = self.label_lines(area.width);
        let label_rows = wrapped.len().max(1) as u16;
        let label_x = area.x + Self::GUTTER.min(area.width);
        let label_style_for = |t: &crate::theme::Theme| {
            if self.disabled {
                t.faint().bg(bg)
            } else {
                t.label(s.focused).bg(bg)
            }
        };
        if label_rows > 1 {
            let st = label_style_for(t);
            for (i, line) in wrapped.iter().enumerate() {
                let y = area.y + i as u16;
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(label_x, y, line, st);
            }
        }
        // label row
        let mut label = self.label.clone();
        let name_w = width(&self.label);
        // the "optional" suffix only appears when the field is wide enough to
        // hold it whole; a clipped suffix reads worse than none
        let show_optional = !self.required
            && !self.label.is_empty()
            && !self.plain_label
            && name_w + 12 <= area.width as usize;
        if self.required {
            label.push_str(" *");
        } else if show_optional {
            label.push_str("  optional");
        }
        let label_style = label_style_for(t);
        if label_rows == 1 {
            buf.set_string(
                label_x,
                area.y,
                crate::ui::text::fit(&label, area.width.saturating_sub(2) as usize),
                label_style,
            );
        }
        if label_rows > 1 {
            // the wrapped label carries no suffix marks
        } else if self.required && !self.disabled && name_w + 4 <= area.width as usize {
            buf.set_string(
                area.x + 2 + name_w as u16 + 1,
                area.y,
                "*",
                t.accent_fg().bg(bg),
            );
        } else if show_optional {
            buf.set_string(
                area.x + 2 + name_w as u16 + 2,
                area.y,
                "optional",
                t.faint().bg(bg),
            );
        }

        // field row
        if area.height < label_rows + 1 {
            return;
        }
        let field = Rect::new(area.x, area.y + label_rows, area.width, 1);
        self.area = field;
        let fs = t.field_style(s);
        crate::ui::ctx::fill(buf, field, fs);
        let gutter = t.gutter(s, fs.bg.unwrap_or(bg), false);
        buf.set_string(field.x, field.y, t.gutter_symbol(s), gutter);
        let trailing = if s.error { 2 } else { 0 };
        let inner = Rect::new(
            field.x + 2.min(field.width),
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
            let glyphs = self.display_graphemes(s.editing);
            let cursor_off = self.buffer.cursor_offset();
            let cursor_col: usize = glyphs
                .iter()
                .filter(|(bi, _, _)| *bi < cursor_off)
                .map(|(_, _, w)| *w)
                .sum();
            let total: usize = glyphs.iter().map(|(_, _, w)| *w).sum();
            let w = inner.width as usize;
            if w > 0 {
                if cursor_col < self.scroll {
                    self.scroll = cursor_col;
                } else if cursor_col >= self.scroll + w {
                    self.scroll = cursor_col + 1 - w;
                }
                if !s.editing {
                    self.scroll = 0;
                }
                // The insertion cursor needs a cell after the final glyph.
                self.scroll = self.scroll.min(total.saturating_add(1).saturating_sub(w));
                let mut boundary = 0;
                for (_, _, gw) in &glyphs {
                    if boundary >= self.scroll {
                        break;
                    }
                    boundary += gw;
                }
                self.scroll = boundary.min(cursor_col);
            }
            let sel = self.buffer.selection();
            let mut col = 0usize;
            let clipped_right = total > self.scroll.saturating_add(w);
            let visible_width = w.saturating_sub(usize::from(clipped_right));
            for (bi, g, gw) in &glyphs {
                let gw = *gw;
                if col + gw <= self.scroll {
                    col += gw;
                    continue;
                }
                let displayed_col = col.saturating_sub(self.scroll);
                if displayed_col + gw > visible_width {
                    break;
                }
                let x = inner.x + displayed_col as u16;
                let mut st = fs;
                if let Some(r) = &sel
                    && r.contains(bi)
                {
                    st = t.selection();
                }
                if s.editing {
                    st = st
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                }
                if displayed_col == 0 && self.scroll > 0 && gw > 0 {
                    buf.set_string(x, inner.y, "…", fs.fg(t.text_muted));
                    col += gw;
                    continue;
                }
                buf.set_string(x, inner.y, g, st);
                col += gw;
            }
            if clipped_right && inner.width > 0 {
                buf.set_string(inner.right() - 1, inner.y, "…", fs.fg(t.text_muted));
            }
            if s.editing && inner.width > 0 {
                let cx = inner.x + (cursor_col - self.scroll) as u16;
                ctx.set_cursor(Position::new(cx.min(inner.right() - 1), inner.y));
            }
        }
        if s.error && field.width >= 2 {
            buf.set_string(
                field.right() - 2,
                field.y,
                "!",
                fs.fg(t.error).add_modifier(Modifier::BOLD),
            );
        }
        ctx.control(self.id, field, self.disabled);

        // help / error row
        if area.height >= label_rows + 2 {
            let msg_y = field.y + 1;
            if let Some(e) = &self.error {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::text::truncate(e, area.width.saturating_sub(2) as usize),
                    t.error_fg().bg(bg),
                );
            } else if !self.help.is_empty() {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::text::truncate(&self.help, area.width.saturating_sub(2) as usize),
                    t.muted().bg(bg),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    fn render(input: &mut TextInput, area: Rect, buf: &mut Buffer) -> Option<Position> {
        let theme = Theme::for_level(ColorLevel::TrueColor);
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

    #[test]
    fn a_validator_closure_can_capture_state() {
        let allowed = vec!["admin".to_owned(), "member".to_owned()];
        let mut i = TextInput::new(WidgetId::of("role"), "Role").validator(move |v| {
            (!allowed.iter().any(|a| a == v)).then(|| format!("{v} is not a role"))
        });
        i = i.value("guest");
        assert!(!i.validate());
        assert_eq!(i.error.as_deref(), Some("guest is not a role"));
        i = i.value("admin");
        assert!(i.validate());
    }

    #[test]
    fn required_error_clears_after_keyboard_and_paste_corrections() {
        let mut input = TextInput::new(WidgetId::of("required"), "Name").required(true);
        assert!(!input.validate());
        input.begin_edit();
        input.on_paste("valid");
        assert!(input.error.is_none());
        input.buffer.select_all();
        input.buffer.backspace();
        assert!(!input.validate());
        input.on_key(&Key {
            code: KeyCode::Char('x'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert!(input.error.is_none());
    }

    #[test]
    fn one_click_enters_editing_at_the_pointer_and_a_disabled_field_ignores_it() {
        let mut input = TextInput::new(WidgetId::of("i"), "Name").value("hello");
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        input.render(area, &mut buf, &mut ctx, theme.canvas);
        assert!(!input.editing);
        let o = input.on_click(Position::new(input.text_area.x + 2, 1));
        assert_eq!(o, Outcome::Changed);
        assert!(input.editing, "the first click edits");
        assert_eq!(
            input.buffer.cursor(),
            2,
            "the caret lands under the pointer"
        );
        // a second click only moves the caret
        input.on_click(Position::new(input.text_area.x + 4, 1));
        assert!(input.editing);
        assert_eq!(input.buffer.cursor(), 4);
        let mut locked = TextInput::new(WidgetId::of("d"), "Locked")
            .value("x")
            .disabled(true);
        locked.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(locked.on_click(Position::new(2, 1)), Outcome::Consumed);
        assert!(!locked.editing);
    }

    #[test]
    fn masked_clicks_follow_display_graphemes() {
        let mut input = TextInput::new(WidgetId::of("masked"), "Secret")
            .value("日👩‍💻e\u{301}a")
            .masked();
        input.begin_edit();
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf);
        for (col, offset) in [(0, 0), (1, 3), (2, 14), (3, 17), (4, 18)] {
            input.on_click(Position::new(input.text_area.x + col, 1));
            assert_eq!(input.buffer.cursor_offset(), offset);
        }
    }

    #[test]
    fn scrolled_masked_clicks_and_cursor_share_geometry() {
        let mut input = TextInput::new(WidgetId::of("masked"), "Secret")
            .value("日本語👩‍💻e\u{301}")
            .masked();
        input.begin_edit();
        let area = Rect::new(0, 0, 6, 3);
        let mut buf = Buffer::empty(area);
        let cursor = render(&mut input, area, &mut buf).unwrap();
        assert!(input.text_area.contains(cursor));
        input.on_click(Position::new(input.text_area.x + 1, 1));
        assert_eq!(input.buffer.cursor_offset(), 20);
        assert_eq!(buf[(input.text_area.x + 1, 1)].symbol(), "•");
    }

    #[test]
    fn narrow_fields_stay_within_their_allocated_rectangle() {
        for width in 0..12 {
            for height in 0..4 {
                for invalid in [false, true] {
                    let mut input = TextInput::new(WidgetId::of("narrow"), "A long label")
                        .value("日本語👩‍💻abcdef")
                        .required(true)
                        .help("Some help");
                    if invalid {
                        input.error = Some("Invalid".into());
                    }
                    input.begin_edit();
                    let area = Rect::new(3, 2, width, height);
                    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
                    let cursor = render(&mut input, area, &mut buf);
                    assert!(cursor.is_none_or(|p| area.contains(p)));
                    for y in 0..8 {
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

    #[test]
    fn wide_graphemes_keep_cursor_and_clicks_aligned_after_scroll() {
        let mut input = TextInput::new(WidgetId::of("wide"), "Name").value("日本語a");
        input.begin_edit();
        let area = Rect::new(0, 0, 8, 3);
        let mut buf = Buffer::empty(area);
        let cursor = render(&mut input, area, &mut buf).unwrap();
        assert!(input.text_area.contains(cursor));
        input.on_click(cursor);
        assert_eq!(input.buffer.cursor_offset(), input.text().len());
    }
}
