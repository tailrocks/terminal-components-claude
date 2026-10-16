use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::edit::{EditEntry, EditKind, FieldKeys, FocusSelect, UndoStack};
use crate::core::event::{Key, Outcome};
use crate::core::focus::{BlurAction, request_focus_next};
use crate::core::id::WidgetId;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::scroll::{HScroll, ScrollState};
use crate::core::text::{CursorPos, TextBuffer};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::widgets::field_common::{
    Applied, EditAction, PointerField, PointerSelect, PressState, apply_edit, click_select,
    drag_select, edit_key_with, press_select,
};
use crate::widgets::input::InputEvent;
use crate::widgets::scrollbar;

/// Multi-line editor. Focus starts editing per its [`EditEntry`] like
/// [`TextInput`](super::input::TextInput) (default
/// [`EditEntry::Immediate`]: the caret shows the moment Tab lands here).
/// While editing Enter inserts a newline, Tab ends the edit and moves focus
/// on, and Esc ends the edit keeping the text ("Done": a document is not
/// cancelled by leaving it; [`Self::esc_reverts`] opts into reverting).
/// After Esc the area is in navigation: arrows scroll, Tab moves on,
/// Enter, F2 or typing edit again.
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
    /// What an edit in progress does when the area loses focus
    /// ([`Self::on_blur`]). Default [`BlurAction::Commit`].
    pub blur_action: BlurAction,
    /// Text before editing began, for [`BlurAction::Cancel`].
    snapshot: String,
    /// What a drag, double click and triple click select.
    /// Default [`PointerSelect::DEFAULT`].
    pub pointer_select: PointerSelect,
    press: PressState,
    /// Which keys scroll the area while it is not being edited (default
    /// [`NavKeys::DEFAULT`]). Editing keys are the text buffer's own.
    pub nav: NavKeys,
    /// How focus starts an edit. `None` (default) follows the theme's
    /// [`crate::theme::Theme::edit_entry`] ([`EditEntry::Immediate`]).
    pub edit_entry: Option<EditEntry>,
    entry: EditEntry,
    /// Where the caret lands when the keyboard starts an immediate edit.
    /// Default [`FocusSelect::CaretEnd`]: a document is continued, not
    /// replaced.
    pub focus_select: FocusSelect,
    /// Esc reverts the text to what it was when the edit began instead of
    /// keeping it. Default `false` (Esc is "Done"; Ctrl+Z undoes).
    pub esc_reverts: bool,
    /// The control-key preset. `None` (default) follows the theme's
    /// [`crate::theme::Theme::field_keys`].
    pub field_keys: Option<FieldKeys>,
    keys: FieldKeys,
    /// The text commands' keys (Select All). `None` (default) uses the
    /// preset's ([`FieldKeys::text_keymap`]: Cmd+A, plus Ctrl+A or Ctrl+L).
    pub text_keymap: Option<crate::core::keymap::KeyMap<crate::core::edit::TextCmd>>,
    undo: UndoStack,
    focus_seen: bool,
    /// Horizontal scrolling of long lines. `None` (default) follows
    /// [`RenderCtx::hscroll`]; the wheel moves [`HScroll::step`] cells
    /// per notch ([`Self::on_wheel_h`]).
    pub horizontal: Option<HScroll>,
    h_cfg: HScroll,
    /// Widest line in the last frame, in cells.
    content_cols: usize,
    /// The area was editing in the last frame: an edit that ends returns
    /// the view to the first column.
    was_editing: bool,
    /// A click on the label focuses the area (the HTML `<label>`
    /// convention, macOS form labels). Default `true`.
    pub label_hit: bool,
    /// The edit in progress was started by focus, so it may end quietly
    /// with the focus.
    focus_edit: bool,
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
            blur_action: BlurAction::Commit,
            snapshot: String::new(),
            pointer_select: PointerSelect::DEFAULT,
            press: PressState::default(),
            nav: NavKeys::DEFAULT,
            edit_entry: None,
            entry: crate::theme::Theme::DEFAULT_EDIT_ENTRY,
            focus_select: FocusSelect::CaretEnd,
            esc_reverts: false,
            field_keys: None,
            keys: FieldKeys::default(),
            text_keymap: None,
            undo: UndoStack::new(),
            focus_seen: false,
            horizontal: None,
            h_cfg: HScroll::default(),
            content_cols: 0,
            was_editing: false,
            label_hit: true,
            focus_edit: false,
        }
    }

    /// See [`Self::edit_entry`].
    pub fn edit_entry(mut self, entry: EditEntry) -> Self {
        self.edit_entry = Some(entry);
        self.entry = entry;
        self
    }
    /// See [`Self::focus_select`].
    pub fn focus_select(mut self, select: FocusSelect) -> Self {
        self.focus_select = select;
        self
    }
    /// See [`Self::esc_reverts`].
    pub fn esc_reverts(mut self, on: bool) -> Self {
        self.esc_reverts = on;
        self
    }
    /// See [`Self::text_keymap`].
    pub fn text_keymap(
        mut self,
        keymap: crate::core::keymap::KeyMap<crate::core::edit::TextCmd>,
    ) -> Self {
        self.text_keymap = Some(keymap);
        self
    }
    /// The edit action `key` means under the resolved preset and text keys.
    fn edit_action(&self, key: &Key) -> EditAction {
        match &self.text_keymap {
            Some(map) => crate::widgets::field_common::edit_key_in(key, true, self.keys, map),
            None => edit_key_with(key, true, self.keys),
        }
    }
    /// See [`Self::field_keys`].
    pub fn field_keys(mut self, keys: FieldKeys) -> Self {
        self.field_keys = Some(keys);
        self.keys = keys;
        self
    }
    /// The entry policy in effect.
    pub fn entry(&self) -> EditEntry {
        self.edit_entry.unwrap_or(self.entry)
    }
    /// The focused area takes printable keys before page shortcuts (see
    /// [`TextInput::wants_text_input`](super::input::TextInput::wants_text_input)).
    pub fn wants_text_input(&self) -> bool {
        !self.disabled && (self.editing || self.entry() != EditEntry::Explicit)
    }
    /// The text differs from the one the edit began with.
    pub fn is_dirty(&self) -> bool {
        self.editing && self.buffer.text() != self.snapshot
    }
    /// The footer hints for this area while it holds focus:
    /// [`crate::widgets::keyhint::EDIT_TEXT`] while editing (Esc
    /// "Cancel" under [`Self::esc_reverts`]), `Enter Edit · ↑↓ Scroll`
    /// after Esc ended the edit.
    pub fn hints(&self) -> Vec<crate::widgets::keyhint::Hint> {
        use crate::widgets::keyhint::{EDIT_TEXT, hint};
        if self.disabled {
            Vec::new()
        } else if self.editing && self.esc_reverts {
            vec![hint("Tab", "Next field"), hint("Esc", "Cancel")]
        } else if self.editing {
            EDIT_TEXT.to_vec()
        } else {
            vec![hint("Enter", "Edit"), hint("↑↓", "Scroll")]
        }
    }

    /// See [`Self::pointer_select`].
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn pointer_select(mut self, sel: PointerSelect) -> Self {
        self.pointer_select = sel;
        self
    }

    /// See [`Self::blur_action`].
    pub fn blur_action(mut self, action: BlurAction) -> Self {
        self.blur_action = action;
        self
    }

    /// Focus left the area (see [`crate::core::focus::Focus::take_changes`]).
    /// An edit in progress ends per [`Self::blur_action`] and reports
    /// `Committed` (or `Cancelled`, which restores the text from before
    /// editing) so the owner's hooks run; `None` when not editing.
    pub fn on_blur(&mut self) -> Option<InputEvent> {
        if !self.editing {
            return None;
        }
        if self.entry() == EditEntry::Immediate && !self.is_dirty() {
            self.commit();
            return None;
        }
        match self.blur_action {
            BlurAction::Commit => {
                self.commit();
                Some(InputEvent::Committed)
            }
            BlurAction::Cancel => {
                self.editing = false;
                let snap = std::mem::take(&mut self.snapshot);
                self.buffer.set_text(snap);
                Some(InputEvent::Cancelled)
            }
            BlurAction::Keep => None,
        }
    }

    pub fn value(mut self, v: &str) -> Self {
        self.buffer.set_text(v);
        self.buffer.move_doc_start(false);
        self.undo.clear();
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
            if !self.editing {
                self.snapshot = self.buffer.text().to_owned();
                self.focus_edit = false;
            }
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
        // Ctrl+C copies only a selection; with nothing selected it is not
        // the field's, so it reaches the app's quit guard
        if key.ctrl_char('c') && self.buffer.selected_text().is_none() {
            return (Outcome::Ignored, None);
        }
        let entry = self.entry();
        if !self.editing {
            if entry == EditEntry::Immediate && !self.focus_seen {
                // focus arrived and this key beat the first render
                self.begin_edit();
                self.focus_select.apply(&mut self.buffer);
            } else if matches!(key.code, KeyCode::Enter | KeyCode::F(2)) && key.plain() {
                self.begin_edit();
                return (Outcome::Changed, None);
            } else if matches!(self.edit_action(key), EditAction::SelectAll) {
                // Select All picks the edit up with the whole text selected
                self.begin_edit();
                self.buffer.select_all();
                return (Outcome::Changed, None);
            } else if entry != EditEntry::Explicit
                && let KeyCode::Char(c) = key.code
                && !key.chord()
            {
                // typing after Esc ("Done") picks the edit up again
                self.begin_edit();
                self.undo.record(&self.buffer, EditKind::Typing);
                self.buffer.insert_char(c);
                return (Outcome::Changed, Some(InputEvent::Changed));
            } else {
                let _moved = match self.nav.action(key) {
                    Some(Nav::Up) => self.scroll.scroll_by(-1),
                    Some(Nav::Down) => self.scroll.scroll_by(1),
                    Some(Nav::PageUp) => self.scroll.page_up(),
                    Some(Nav::PageDown) => self.scroll.page_down(),
                    Some(Nav::Home) => self.scroll.jump_start(),
                    Some(Nav::End) => self.scroll.jump_end(),
                    Some(Nav::Left | Nav::Right) | None => return (Outcome::Ignored, None),
                };
                return (Outcome::Changed, None);
            }
        }
        self.follow_cursor = true;
        match self.edit_action(key) {
            EditAction::Cancel if entry != EditEntry::Explicit && self.esc_reverts => {
                let dirty = self.is_dirty();
                self.revert();
                (
                    Outcome::Changed,
                    Some(if dirty {
                        InputEvent::Cancelled
                    } else {
                        InputEvent::Committed
                    }),
                )
            }
            EditAction::Commit | EditAction::Cancel => {
                self.commit();
                (Outcome::Changed, Some(InputEvent::Committed))
            }
            EditAction::Tab { backward } => {
                self.commit();
                request_focus_next(backward);
                (
                    Outcome::Changed,
                    Some(InputEvent::CommittedTab { backward }),
                )
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
            action => match apply_edit(&mut self.buffer, &mut self.undo, action, key.code, false) {
                Some(Applied::Edited) => (Outcome::Changed, Some(InputEvent::Changed)),
                _ => (Outcome::Changed, None),
            },
        }
    }

    /// End the edit restoring the text it began with (Esc under
    /// [`Self::esc_reverts`]); the revert is itself undoable.
    fn revert(&mut self) {
        if self.buffer.text() != self.snapshot {
            self.undo.record(&self.buffer, EditKind::Other);
        }
        self.editing = false;
        let snap = std::mem::take(&mut self.snapshot);
        self.buffer.set_text(snap);
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.disabled || (!self.editing && self.entry() == EditEntry::Explicit) {
            return Outcome::Ignored;
        }
        self.begin_edit();
        self.undo.record(&self.buffer, EditKind::Other);
        self.buffer.insert_str(text);
        self.follow_cursor = true;
        Outcome::Changed
    }

    /// `pos` is on the label row above the body ([`Self::label_hit`]).
    fn on_label(&self, pos: Position) -> bool {
        self.label_hit && pos.y < self.area.y
    }

    /// Byte offset under `pos`, clamped to the text.
    fn offset_at(&self, pos: Position) -> usize {
        // a pointer above or left of the text names the line or column
        // before the first visible one, so a drag past the edge scrolls
        let line = (self.scroll.offset as isize + (pos.y as isize - self.text_area.y as isize))
            .max(0) as usize;
        let col =
            (self.hscroll as isize + (pos.x as isize - self.text_area.x as isize)).max(0) as usize;
        let line = line.min(self.buffer.line_count().saturating_sub(1));
        self.buffer.offset_at(line, col)
    }

    /// A click enters editing (one click, never two) and places the cursor
    /// at the pointer. Same as [`Self::on_click_count`] with a count of 1.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        self.on_click_count(pos, 1)
    }

    /// A completed click with its multi-click `count`: 1 places the caret,
    /// 2 selects the word, 3 the line, per [`Self::pointer_select`]. The
    /// release that ends a drag keeps the dragged selection.
    pub fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        if self.on_label(pos) {
            // a label click edits without moving the caret
            if !self.editing {
                self.begin_edit();
                self.focus_select.apply(&mut self.buffer);
            }
            self.follow_cursor = true;
            return Outcome::Changed;
        }
        self.begin_edit();
        let offset = self.offset_at(pos);
        click_select(
            &mut self.buffer,
            &mut self.press,
            self.pointer_select,
            offset,
            count,
            false,
        );
        self.follow_cursor = true;
        Outcome::Changed
    }

    /// Button down on the area: remembers where, so a drag selects from
    /// there. The press alone changes nothing; the completed click (or the
    /// drag) starts editing.
    pub fn on_press(&mut self, pos: Position) -> Outcome {
        self.on_press_count(pos, 1)
    }

    /// A press with the click `count` it starts: the second press of a
    /// double click selects the word, and a drag from it extends by words
    /// ([`PointerSelect::drag_by_word`]).
    pub fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome {
        if self.disabled || self.on_label(pos) {
            return Outcome::Consumed;
        }
        // the pointer owns this focus: the click places the caret
        self.focus_seen = true;
        self.focus_edit = false;
        let offset = self.offset_at(pos);
        if press_select(
            &mut self.buffer,
            &mut self.press,
            self.pointer_select,
            offset,
            count,
            false,
        ) {
            self.begin_edit();
            self.follow_cursor = true;
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    /// Drag with the button held since a press on the area: starts
    /// editing and selects from the press to the pointer
    /// ([`PointerSelect::drag`]). Past an edge the selection runs on and
    /// the area scrolls to follow it.
    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        if self.disabled || !self.pointer_select.drag {
            return Outcome::Ignored;
        }
        let offset = self.offset_at(pos);
        self.begin_edit();
        drag_select(&mut self.buffer, &mut self.press, offset);
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

    /// Horizontal wheel (or Shift+wheel): long lines scroll sideways by
    /// [`HScroll::step`] cells per notch, under the [`Self::horizontal`]
    /// policy (default [`RenderCtx::hscroll`]). Nothing wider than the
    /// area, or horizontal scrolling off: ignored, so the notch can go to
    /// the container.
    pub fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        let width = self.text_area.width as usize;
        if !self.h_cfg.enabled || delta == 0 || self.content_cols <= width {
            return Outcome::Ignored;
        }
        let max = self.content_cols - width;
        let step = self.h_cfg.step.max(1) * delta.unsigned_abs() as usize;
        let to = if delta < 0 {
            self.hscroll.saturating_sub(step)
        } else {
            (self.hscroll + step).min(max)
        };
        if to == self.hscroll {
            return Outcome::Consumed;
        }
        self.hscroll = to;
        Outcome::Changed
    }

    /// See [`Self::horizontal`].
    pub fn horizontal(mut self, h: HScroll) -> Self {
        self.horizontal = Some(h);
        self
    }

    /// See [`Self::label_hit`].
    pub fn label_hit(mut self, on: bool) -> Self {
        self.label_hit = on;
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.entry = self.edit_entry.unwrap_or(ctx.theme.edit_entry);
        self.keys = self.field_keys.or(ctx.theme.field_keys).unwrap_or_default();
        self.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        // focus newly gained starts an immediate edit: the caret shows the
        // moment Tab lands here
        let has_focus = ctx.interaction.has_focus(self.id) && !ctx.inert;
        if has_focus && !self.focus_seen && self.entry == EditEntry::Immediate && !self.editing {
            self.begin_edit();
            self.focus_select.apply(&mut self.buffer);
            self.focus_edit = true;
        } else if !ctx.interaction.has_focus(self.id)
            && self.entry == EditEntry::Immediate
            && self.editing
            && self.focus_edit
            && !self.is_dirty()
        {
            // a clean edit focus started ends with the focus even when the
            // owner never forwarded the blur
            self.commit();
        }
        self.focus_seen = has_focus;
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
        // An edit that lost focus is ended by `on_blur` on the event path,
        // never here (`s.editing` needs focus); render only starts the
        // edit a newly gained focus asks for (above).

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
        if self.label_hit && !self.label.is_empty() {
            // the label row is the area's own target: a click there focuses
            // and edits it like a click in the body
            let lw = width(&self.label).min(area.width.saturating_sub(2) as usize) as u16;
            ctx.clickable(self.id, Rect::new(area.x, area.y, lw + 2, 1));
        }
        ctx.control(self.id, body, self.disabled);
        if !self.disabled {
            ctx.pointer_shape(self.id, crate::core::shape::PointerShape::Text);
        }
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
        if !self.disabled && self.pointer_select.drag {
            ctx.autoscroll(inner);
        }

        let text = self.buffer.text();
        let lines: Vec<&str> = text.split('\n').collect();
        self.content_cols = lines.iter().map(|l| width(l)).max().unwrap_or(0);
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
        } else if !s.editing && self.was_editing {
            // the edit ended: back to the first column; a wheel scroll
            // outside an edit stays where the user left it
            self.hscroll = 0;
        }
        self.was_editing = s.editing;
        if !s.editing {
            let max = self.content_cols.saturating_sub(inner.width as usize);
            self.hscroll = self.hscroll.min(max);
        }
        self.last_cursor = Some(cur);
        self.follow_cursor = false;
        if text.is_empty() {
            // the placeholder stays until the first character, with the
            // caret on it while editing (macOS)
            let p = crate::ui::text::truncate(&self.placeholder, inner.width as usize);
            buf.set_string(inner.x, inner.y, &p, t.placeholder(s));
            if s.editing && inner.width > 0 {
                ctx.set_cursor(Position::new(inner.x, inner.y));
            }
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
                                buf.set_string(inner.x, y, t.glyphs.ellipsis, fs.fg(t.text_muted));
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
                        let st = fs.fg(t.text_muted);
                        buf.set_string(inner.right() - 1, y, t.glyphs.ellipsis, st);
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

impl crate::core::focus::BlurTarget for TextArea {
    fn blur_id(&self) -> WidgetId {
        self.id
    }
    fn blur(&mut self) -> Outcome {
        match self.on_blur() {
            Some(_) => Outcome::Changed,
            None => Outcome::Ignored,
        }
    }
}

impl PointerField for TextArea {
    fn pointer_id(&self) -> WidgetId {
        self.id
    }
    fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome {
        TextArea::on_press_count(self, pos, count)
    }
    fn on_drag(&mut self, pos: Position) -> Outcome {
        TextArea::on_drag(self, pos)
    }
    fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome {
        TextArea::on_click_count(self, pos, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_above_the_area_scrolls_it_and_selects_on() {
        let text = (0..20)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut a = TextArea::new(WidgetId::of("v"), "Notes", 4).value(&text);
        let area = Rect::new(0, 0, 30, 6);
        let mut buf = Buffer::empty(area);
        render(&mut a, area, &mut buf, ColorLevel::TrueColor);
        a.scroll.offset = 8;
        render(&mut a, area, &mut buf, ColorLevel::TrueColor);
        let ta = a.text_area;
        a.on_press(Position::new(ta.x + 2, ta.y + 1));
        // two rows above the text: the selection runs into hidden lines
        a.on_drag(Position::new(ta.x + 2, ta.y.saturating_sub(2)));
        render(&mut a, area, &mut buf, ColorLevel::TrueColor);
        assert!(
            a.scroll.offset < 8,
            "scrolled up to follow: {}",
            a.scroll.offset
        );
        assert!(
            a.buffer
                .selected_text()
                .is_some_and(|s| s.starts_with("ne 7\nline 8")),
            "{:?}",
            a.buffer.selected_text()
        );
    }

    #[test]
    fn drag_double_and_triple_click_select_text() {
        let mut a = TextArea::new(WidgetId::of("s"), "Notes", 4)
            .value("one two\nthree four")
            .edit_entry(crate::core::edit::EditEntry::Explicit);
        let area = Rect::new(0, 0, 30, 6);
        let mut buf = Buffer::empty(area);
        render(&mut a, area, &mut buf, ColorLevel::TrueColor);
        let ta = a.text_area;
        let at = |col: u16, row: u16| Position::new(ta.x + col, ta.y + row);
        let (p, d) = (at(4, 0), at(5, 1));
        a.on_press(p);
        assert!(!a.editing, "the press alone changes nothing");
        assert_eq!(a.on_drag(d), Outcome::Changed);
        assert!(a.editing, "the drag edits");
        assert_eq!(a.buffer.selected_text(), Some("two\nthree"));
        a.on_click_count(d, 1);
        assert_eq!(
            a.buffer.selected_text(),
            Some("two\nthree"),
            "release keeps it"
        );
        let (w, l) = (at(7, 1), at(1, 1));
        a.on_click_count(w, 2);
        assert_eq!(a.buffer.selected_text(), Some("four"));
        a.on_click_count(l, 3);
        assert_eq!(a.buffer.selected_text(), Some("three four"), "one line");
        let mut off = TextArea::new(WidgetId::of("o"), "Notes", 4)
            .value("one two")
            .pointer_select(crate::widgets::field_common::PointerSelect::OFF);
        render(&mut off, area, &mut buf, ColorLevel::TrueColor);
        let p = Position::new(off.text_area.x + 5, off.text_area.y);
        off.on_click_count(p, 2);
        assert_eq!(off.buffer.selected_text(), None);
    }

    #[test]
    fn rendering_without_focus_never_ends_the_edit() {
        use crate::core::{focus::FocusRing, hit::HitRegistry};
        use crate::theme::Theme;
        use crate::ui::ctx::Interaction;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut a = TextArea::new(WidgetId::of("r"), "Notes", 3).value("v");
        a.begin_edit();
        // dirty: a clean immediate edit ends quietly with the focus
        a.buffer.insert_str("w");
        let area = Rect::new(0, 0, 30, 5);
        let mut buf = Buffer::empty(area);
        a.render(area, &mut buf, &mut ctx, theme.canvas);
        assert!(a.editing, "render is not a blur: on_blur ends the edit");
        assert_eq!(ctx.cursor, None);
        assert_eq!(a.on_blur(), Some(InputEvent::Committed));
    }

    #[test]
    fn blur_ends_the_edit_per_blur_action_and_reports_it() {
        let edited = |action| {
            let mut a = TextArea::new(WidgetId::of("b"), "Notes", 3)
                .value("old")
                .blur_action(action);
            assert_eq!(
                a.on_blur(),
                None,
                "an area that is not editing ignores blur"
            );
            a.begin_edit();
            a.buffer.move_doc_end(false);
            a.buffer.insert_str("\nnew");
            a
        };
        let mut commit = edited(BlurAction::default());
        assert_eq!(commit.on_blur(), Some(InputEvent::Committed));
        assert!(!commit.editing);
        assert_eq!(commit.buffer.text(), "old\nnew");
        let mut cancel = edited(BlurAction::Cancel);
        assert_eq!(cancel.on_blur(), Some(InputEvent::Cancelled));
        assert!(!cancel.editing);
        assert_eq!(cancel.buffer.text(), "old");
        let mut keep = edited(BlurAction::Keep);
        assert_eq!(keep.on_blur(), None);
        assert!(keep.editing);
    }

    #[test]
    fn home_and_end_scroll_a_read_only_view_and_boundary_wheels_are_consumed() {
        let text = (1..=30)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut ta = TextArea::new(WidgetId::of("t"), "Notes", 4)
            .value(&text)
            .edit_entry(crate::core::edit::EditEntry::Explicit);
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

    /// Keyboard audit #1, #15 and #16: focus starts the edit with the caret
    /// at the end; Enter is a newline; Tab applies and asks the ring to
    /// move on; Esc is "Done" (keeps the text) and a typed letter picks the
    /// edit up again; `esc_reverts` restores the text instead.
    #[test]
    fn immediate_entry_esc_is_done_and_tab_moves_on() {
        let area = Rect::new(0, 0, 30, 6);
        let mut buf = Buffer::empty(area);
        let mut a = TextArea::new(WidgetId::of("a"), "Notes", 3).value("one");
        assert!(render(&mut a, area, &mut buf, ColorLevel::TrueColor).is_some());
        assert!(a.editing, "focus alone starts the edit");
        a.on_key(&key(KeyCode::Enter));
        a.on_key(&key(KeyCode::Char('x')));
        assert_eq!(
            a.buffer.text(),
            "one\nx",
            "caret at the end, Enter a newline"
        );
        assert_eq!(a.on_key(&key(KeyCode::Esc)).1, Some(InputEvent::Committed));
        assert!(!a.editing);
        assert_eq!(a.buffer.text(), "one\nx", "Esc is Done: the text stays");
        a.on_key(&key(KeyCode::Char('y')));
        assert!(a.editing, "typing picks the edit up again");
        assert_eq!(
            a.on_key(&key(KeyCode::Tab)).1,
            Some(InputEvent::CommittedTab { backward: false })
        );
        let mut r = TextArea::new(WidgetId::of("r"), "Notes", 3)
            .value("keep")
            .esc_reverts(true);
        render(&mut r, area, &mut buf, ColorLevel::TrueColor);
        r.on_key(&key(KeyCode::Char('x')));
        assert_eq!(r.on_key(&key(KeyCode::Esc)).1, Some(InputEvent::Cancelled));
        assert_eq!(r.buffer.text(), "keep");
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

    #[test]
    fn the_text_is_the_edge_autoscroll_body_unless_disabled() {
        let theme = Theme::junie();
        for disabled in [false, true] {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut w = TextArea::new(WidgetId::of("v"), "Notes", 4).value("a\nb");
            w.disabled = disabled;
            let area = Rect::new(0, 0, 20, 6);
            let mut buf = Buffer::empty(area);
            w.render(area, &mut buf, &mut ctx, theme.surface);
            let ta = w.text_area;
            let found = hits.autoscroll_at(Position::new(ta.x, ta.y));
            assert_eq!(found, (!disabled).then_some(ta), "disabled {disabled}");
        }
    }

    fn draw_hits(input: &mut TextArea, area: Rect, focused: bool) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(area);
        {
            let mut ctx = RenderCtx::new(
                &theme,
                Interaction {
                    focus: focused.then_some(input.id),
                    ..Default::default()
                },
                &mut hits,
                &mut ring,
            );
            input.render(area, &mut buf, &mut ctx, theme.surface);
        }
        (buf, hits)
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (buf.area.x..buf.area.right())
            .map(|x| buf[(x, y)].symbol())
            .collect()
    }

    #[test]
    fn the_horizontal_wheel_scrolls_long_lines_outside_an_edit() {
        let mut input = TextArea::new(WidgetId::of("h"), "Notes", 2)
            .value("abcdefghijklmnopqrstuvwxyz0123456789");
        let area = Rect::new(0, 0, 12, 4);
        let (buf, _) = draw_hits(&mut input, area, false);
        assert!(row(&buf, 1).contains("abc"));
        assert_eq!(input.on_wheel_h(1), Outcome::Changed);
        let (buf, _) = draw_hits(&mut input, area, false);
        assert!(
            !row(&buf, 1).contains("abc") && row(&buf, 1).contains("ef"),
            "one notch moves HScroll::step cells: {:?}",
            row(&buf, 1)
        );
        // the far end stops the wheel
        for _ in 0..20 {
            input.on_wheel_h(1);
        }
        assert_eq!(input.on_wheel_h(1), Outcome::Consumed);
        let (buf, _) = draw_hits(&mut input, area, false);
        assert!(row(&buf, 1).contains('9'));
        // short text: the notch is left to the container
        let mut short = TextArea::new(WidgetId::of("s"), "Notes", 2).value("ab");
        draw_hits(&mut short, area, false);
        assert_eq!(short.on_wheel_h(1), Outcome::Ignored);
        // switched off by policy
        let mut off = TextArea::new(WidgetId::of("o"), "Notes", 2)
            .value("abcdefghijklmnopqrstuvwxyz")
            .horizontal(HScroll::off());
        draw_hits(&mut off, area, false);
        assert_eq!(off.on_wheel_h(1), Outcome::Ignored);
    }

    #[test]
    fn clicking_the_label_focuses_and_edits_the_area() {
        let mut input = TextArea::new(WidgetId::of("l"), "Notes", 2).value("one\ntwo");
        let area = Rect::new(0, 0, 20, 4);
        let (_, hits) = draw_hits(&mut input, area, false);
        assert_eq!(
            hits.hit(Position::new(3, 0)),
            Some(input.id),
            "the label is a target"
        );
        assert_eq!(
            hits.hit(Position::new(15, 0)),
            None,
            "blank space after it is not"
        );
        input.on_press(Position::new(3, 0));
        input.on_click(Position::new(3, 0));
        assert!(input.editing);
        assert_eq!(
            input.buffer.cursor_offset(),
            input.buffer.text().len(),
            "the caret goes where focus puts it, not under the label"
        );
        let mut off = TextArea::new(WidgetId::of("n"), "Notes", 2).label_hit(false);
        let (_, hits) = draw_hits(&mut off, area, false);
        assert_eq!(hits.hit(Position::new(3, 0)), None);
    }
}
