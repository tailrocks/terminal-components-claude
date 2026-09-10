//! Code editor: a document editor with a gutter (focus bar, block marker,
//! line numbers, diagnostics), caller-supplied highlighting and block
//! segmentation, horizontal scrolling, selection and an inline find bar.
//! Language knowledge stays outside: the widget only receives spans.
//!
//! Two modes like every text control: focused (navigation) and editing.

use std::ops::Range;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::core::text::TextBuffer;
use crate::theme::SyntaxTone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{find_ranges, width};
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::scrollbar;

pub type Highlighter = fn(&str) -> Vec<(Range<usize>, SyntaxTone)>;
pub type Segmenter = fn(&str) -> Vec<Range<usize>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range<usize>,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindState {
    pub needle: String,
    /// Whole-grapheme byte ranges in the original document, including when
    /// case-insensitive matching expands characters such as `İ`.
    pub matches: Vec<Range<usize>>,
    pub current: usize,
    pub editing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorEvent {
    Changed,
    CursorMoved,
    /// Esc or modified Enter from editing: back to navigation (document kept).
    Committed,
    /// Tab from editing when `tab_leaves` is set.
    Leave {
        backward: bool,
    },
}

#[derive(Debug, Clone)]
pub struct CodeEditor {
    pub id: WidgetId,
    pub buffer: TextBuffer,
    pub editing: bool,
    pub read_only: bool,
    pub scroll: ScrollState,
    pub hscroll: usize,
    pub indent: usize,
    pub highlighter: Option<Highlighter>,
    pub segmenter: Option<Segmenter>,
    pub diagnostics: Vec<Diagnostic>,
    /// Block currently executing (spinner in the marker column).
    pub running: Option<Range<usize>>,
    pub find: Option<FindState>,
    pub placeholder: String,
    /// When true, Tab in editing mode commits and leaves (form-like);
    /// otherwise Tab indents.
    pub tab_leaves: bool,
    pub area: Rect,
    text_area: Rect,
    gutter_w: u16,
    drag_anchor: Option<usize>,
    /// Cached wanted column for vertical motion.
    cached_spans: Vec<(Range<usize>, SyntaxTone)>,
    cached_for: u64,
    /// Manual scrolling survives redraws until editing/cursor/content changes.
    rendered_state: Option<(usize, u64, bool)>,
}

fn hash_text(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ s.len() as u64
}

impl CodeEditor {
    pub fn new(id: WidgetId, text: &str) -> Self {
        let mut buffer = TextBuffer::multi(text);
        buffer.move_doc_start(false);
        Self {
            id,
            buffer,
            editing: false,
            read_only: false,
            scroll: ScrollState::default(),
            hscroll: 0,
            indent: 2,
            highlighter: None,
            segmenter: None,
            diagnostics: vec![],
            running: None,
            find: None,
            placeholder: String::new(),
            tab_leaves: false,
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            gutter_w: 0,
            drag_anchor: None,
            cached_spans: vec![],
            cached_for: 0,
            rendered_state: None,
        }
    }

    pub fn highlighter(mut self, h: Highlighter) -> Self {
        self.highlighter = Some(h);
        self
    }
    pub fn segmenter(mut self, s: Segmenter) -> Self {
        self.segmenter = Some(s);
        self
    }
    pub fn read_only(mut self, ro: bool) -> Self {
        self.read_only = ro;
        self
    }
    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer.set_text(text);
        self.buffer.move_doc_start(false);
        self.diagnostics.clear();
        self.hscroll = 0;
        self.scroll.jump_start();
        self.refind();
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Block (statement) containing the cursor, per the segmenter.
    pub fn current_block(&self) -> Option<Range<usize>> {
        let seg = self.segmenter?;
        let cur = self.buffer.cursor_offset();
        let blocks = seg(self.buffer.text());
        blocks
            .iter()
            .find(|b| cur >= b.start && cur <= b.end)
            .or_else(|| blocks.iter().rev().find(|b| b.end <= cur))
            .cloned()
    }

    /// Selection text and range, or the current block.
    pub fn selection_or_block(&self) -> Option<(String, Range<usize>)> {
        if let Some(r) = self.buffer.selection() {
            return Some((self.buffer.text()[r.clone()].to_owned(), r));
        }
        let b = self.current_block()?;
        Some((self.buffer.text()[b.clone()].to_owned(), b))
    }

    /// Byte ranges of all blocks.
    pub fn blocks(&self) -> Vec<Range<usize>> {
        self.segmenter
            .map(|s| s(self.buffer.text()))
            .unwrap_or_default()
    }

    pub fn jump_to(&mut self, offset: usize) {
        let offset = offset.min(self.buffer.text().len());
        let pos = TextBuffer::pos_of(self.buffer.text(), offset);
        self.buffer.set_cursor_line_col(pos.line, pos.col);
        self.scroll.ensure_visible(pos.line);
    }

    pub fn cursor_offset(&self) -> usize {
        self.buffer.cursor_offset()
    }

    /// Screen cell of the cursor (anchor for a completion popup).
    pub fn cursor_cell(&self) -> Option<Rect> {
        let pos = self.buffer.cursor_pos();
        if pos.line < self.scroll.offset
            || pos.line >= self.scroll.offset + self.scroll.viewport_len
            || pos.col < self.hscroll
            || pos.col >= self.hscroll.saturating_add(self.text_area.width as usize)
        {
            return None;
        }
        let y = self.text_area.y + (pos.line - self.scroll.offset) as u16;
        let x = self.text_area.x + pos.col.saturating_sub(self.hscroll) as u16;
        Some(Rect::new(x, y, 1, 1))
    }

    pub fn begin_edit(&mut self) {
        if !self.read_only {
            self.editing = true;
        }
    }

    /// Mark a block as executing (spinner in the marker column).
    pub fn set_running(&mut self, block: Option<Range<usize>>) {
        self.running = block;
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.buffer.clear_selection();
    }

    // ---- find -------------------------------------------------------

    /// Open a single-line query. Uppercase queries match case exactly;
    /// lowercase queries use Unicode lowercase matching and source graphemes.
    /// Long queries scroll within the footer to keep their insertion point visible.
    pub fn open_find(&mut self) {
        let needle = self
            .find
            .as_ref()
            .map(|f| f.needle.clone())
            .unwrap_or_default();
        self.find = Some(FindState {
            needle,
            matches: vec![],
            current: 0,
            editing: true,
        });
        self.refind();
    }

    fn refind(&mut self) {
        let Some(f) = self.find.as_mut() else {
            return;
        };
        f.matches.clear();
        f.current = 0;
        if f.needle.is_empty() {
            return;
        }
        let case_sensitive = f.needle.chars().any(|c| c.is_uppercase());
        f.matches = find_ranges(self.buffer.text(), &f.needle, case_sensitive);
        let cur = self.buffer.cursor_offset();
        f.current = f.matches.iter().position(|m| m.start >= cur).unwrap_or(0);
    }

    fn goto_match(&mut self, delta: isize) {
        let Some(f) = self.find.as_mut() else {
            return;
        };
        if f.matches.is_empty() {
            return;
        }
        let n = f.matches.len() as isize;
        f.current = ((f.current as isize + delta).rem_euclid(n)) as usize;
        let m = f.matches[f.current].clone();
        self.jump_to(m.start);
    }

    // ---- keys -------------------------------------------------------

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<EditorEvent>) {
        // find bar
        if let Some(f) = self.find.as_mut()
            && f.editing
        {
            {
                match key.code {
                    KeyCode::Esc => {
                        self.find = None;
                        return (Outcome::Changed, None);
                    }
                    KeyCode::Enter => {
                        if key.shift() {
                            self.goto_match(-1);
                        } else if f.needle.is_empty() {
                            f.editing = false;
                        } else {
                            f.editing = false;
                            self.goto_match(0);
                        }
                        return (Outcome::Changed, Some(EditorEvent::CursorMoved));
                    }
                    KeyCode::Backspace => {
                        if let Some((start, _)) = f.needle.grapheme_indices(true).next_back() {
                            f.needle.truncate(start);
                        }
                        self.refind();
                        return (Outcome::Changed, None);
                    }
                    KeyCode::Char(c) if !key.ctrl() && !key.alt() => {
                        f.needle.push(c);
                        self.refind();
                        self.goto_match(0);
                        return (Outcome::Changed, Some(EditorEvent::CursorMoved));
                    }
                    _ => return (Outcome::Consumed, None),
                }
            }
        }
        // Public read_only can change between events. Reconcile the document
        // mode before dispatch so indentation cannot bypass mutation guards.
        if self.read_only && self.editing {
            self.commit();
        }
        if !self.editing {
            return self.nav_key(key);
        }
        match edit_key(key, true) {
            EditAction::Cancel | EditAction::Commit => {
                self.commit();
                (Outcome::Changed, Some(EditorEvent::Committed))
            }
            EditAction::Tab { backward } => {
                if self.tab_leaves {
                    self.commit();
                    return (Outcome::Changed, Some(EditorEvent::Leave { backward }));
                }
                if backward {
                    self.dedent_selection();
                } else if self.buffer.has_selection_lines() {
                    self.indent_selection();
                } else {
                    let spaces = " ".repeat(self.indent);
                    self.buffer.insert_str(&spaces);
                }
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::Changed))
            }
            EditAction::Apply(f) => {
                if self.read_only {
                    return (Outcome::Consumed, None);
                }
                let before = self.buffer.text().len();
                let before_hash = hash_text(self.buffer.text());
                f(&mut self.buffer);
                let changed = self.buffer.text().len() != before
                    || hash_text(self.buffer.text()) != before_hash;
                self.after_change();
                (
                    Outcome::Changed,
                    Some(if changed {
                        EditorEvent::Changed
                    } else {
                        EditorEvent::CursorMoved
                    }),
                )
            }
            EditAction::Insert(c) => {
                if self.read_only {
                    return (Outcome::Consumed, None);
                }
                self.buffer.insert_char(c);
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::Changed))
            }
            EditAction::None => match key.code {
                KeyCode::PageUp => {
                    for _ in 0..self.scroll.viewport_len.max(1) {
                        self.buffer.move_up(false);
                    }
                    self.after_change();
                    (Outcome::Changed, Some(EditorEvent::CursorMoved))
                }
                KeyCode::PageDown => {
                    for _ in 0..self.scroll.viewport_len.max(1) {
                        self.buffer.move_down(false);
                    }
                    self.after_change();
                    (Outcome::Changed, Some(EditorEvent::CursorMoved))
                }
                _ => (Outcome::Ignored, None),
            },
        }
    }

    fn nav_key(&mut self, key: &Key) -> (Outcome, Option<EditorEvent>) {
        match key.code {
            KeyCode::Enter | KeyCode::Char('i') if key.plain() => {
                self.begin_edit();
                (Outcome::Changed, None)
            }
            KeyCode::Char('a') if key.plain() => {
                self.begin_edit();
                self.buffer.move_right(false);
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                self.buffer.move_up(false);
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Down | KeyCode::Char('j') if key.plain() => {
                self.buffer.move_down(false);
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Left | KeyCode::Char('h') if key.plain() => {
                self.hscroll = self.hscroll.saturating_sub(8);
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                self.hscroll += 8;
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
            KeyCode::Home | KeyCode::Char('g') if key.plain() => {
                self.buffer.move_doc_start(false);
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.buffer.move_doc_end(false);
                self.after_change();
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Char('{') => {
                let cur = self.buffer.cursor_offset();
                if let Some(b) = self.blocks().iter().rev().find(|b| b.start < cur) {
                    self.jump_to(b.start);
                }
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Char('}') => {
                let cur = self.buffer.cursor_offset();
                if let Some(b) = self.blocks().iter().find(|b| b.start > cur) {
                    self.jump_to(b.start);
                }
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Char('/') if key.plain() => {
                self.open_find();
                (Outcome::Changed, None)
            }
            KeyCode::Char('n') if key.plain() => {
                self.goto_match(1);
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Char('N') => {
                self.goto_match(-1);
                (Outcome::Changed, Some(EditorEvent::CursorMoved))
            }
            KeyCode::Esc => {
                if self.find.is_some() {
                    self.find = None;
                    (Outcome::Changed, None)
                } else {
                    (Outcome::Ignored, None)
                }
            }
            _ => (Outcome::Ignored, None),
        }
    }

    fn after_change(&mut self) {
        let pos = self.buffer.cursor_pos();
        self.scroll.set_content(self.buffer.line_count());
        self.scroll.ensure_visible(pos.line);
        let w = self.text_area.width.max(8) as usize;
        if pos.col < self.hscroll + 4 {
            self.hscroll = pos.col.saturating_sub(4);
        } else if pos.col + 4 >= self.hscroll + w {
            self.hscroll = pos.col + 5 - w;
        }
        if self.find.is_some() {
            self.refind();
        }
    }

    fn indent_selection(&mut self) {
        let spaces = " ".repeat(self.indent);
        let (a, b) = self.buffer.selection_lines();
        for line in a..=b {
            let off = self.buffer.offset_at(line, 0);
            self.buffer.insert_at(off, &spaces);
        }
    }

    fn dedent_selection(&mut self) {
        let (a, b) = self.buffer.selection_lines();
        for line in a..=b {
            let off = self.buffer.offset_at(line, 0);
            let text = self.buffer.text();
            let n = text[off..]
                .chars()
                .take(self.indent)
                .take_while(|c| *c == ' ')
                .count();
            self.buffer.remove_range(off..off + n);
        }
    }

    // ---- mouse ------------------------------------------------------

    fn offset_at_pos(&self, pos: Position) -> usize {
        let line = (pos.y.saturating_sub(self.text_area.y) as usize + self.scroll.offset)
            .min(self.buffer.line_count().saturating_sub(1));
        let col = pos.x.saturating_sub(self.text_area.x) as usize + self.hscroll;
        self.buffer.offset_at(line, col)
    }

    pub fn on_click(&mut self, pos: Position, was_focused: bool) -> Outcome {
        if self.read_only {
            let off = self.offset_at_pos(pos);
            self.jump_to(off);
            return Outcome::Changed;
        }
        if !self.editing {
            if !was_focused {
                return Outcome::Changed;
            }
            self.begin_edit();
        }
        let off = self.offset_at_pos(pos);
        self.jump_to(off);
        self.drag_anchor = Some(off);
        Outcome::Changed
    }

    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        let Some(anchor) = self.drag_anchor else {
            return Outcome::Ignored;
        };
        let off = self.offset_at_pos(pos);
        self.buffer.select_range(anchor, off);
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32, horizontal: bool) -> Outcome {
        if horizontal {
            self.hscroll = (self.hscroll as isize + delta as isize * 4).max(0) as usize;
        } else {
            self.scroll.scroll_by(delta as isize);
        }
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
    }

    /// Paste into the active find query, or into the editable document.
    /// Find stays available for read-only documents; its single-line query
    /// uses the same paste normalization as other single-line text controls.
    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(find) = self.find.as_mut()
            && find.editing
        {
            let mut query = TextBuffer::single(&find.needle);
            query.insert_str(text);
            find.needle = query.text().to_owned();
            self.refind();
            self.goto_match(0);
            return Outcome::Changed;
        }
        if !self.editing || self.read_only {
            return Outcome::Ignored;
        }
        self.buffer.insert_str(text);
        self.after_change();
        Outcome::Changed
    }

    // ---- render -----------------------------------------------------

    fn spans(&mut self) -> &[(Range<usize>, SyntaxTone)] {
        let h = hash_text(self.buffer.text());
        if h != self.cached_for {
            self.cached_spans = self
                .highlighter
                .map(|f| f(self.buffer.text()))
                .unwrap_or_default();
            self.cached_for = h;
        }
        &self.cached_spans
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        // The parent buffer can be much larger than this editor's pane.
        let put = |buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style| {
            if area.contains(Position::new(x, y)) {
                buf.set_stringn(x, y, text, (area.right() - x) as usize, style);
            }
        };
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.editing = self.editing && s.focused;
        // Read-only documents still accept focus, navigation, selection and find.
        if (!s.focused || self.read_only) && self.editing {
            self.commit();
            s.editing = false;
        }
        let focused = s.focused;
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);

        let fs = if self.read_only {
            Style::new().fg(t.text_primary).bg(bg)
        } else {
            t.field_style(s)
        };
        fill(buf, area, fs);
        let body_h = area.height.saturating_sub(1);
        let rows = body_h as usize;
        let spans: Vec<(Range<usize>, SyntaxTone)> = self.spans().to_vec();
        let text_owned = self.buffer.text().to_owned();
        let lines: Vec<&str> = text_owned.split('\n').collect();
        let line_count = lines.len();
        self.scroll.set_content(line_count);
        self.scroll.set_viewport(rows);
        let num_w = (line_count.to_string().len() as u16).max(2);
        // gutter: bar(1) marker(1) space(1) numbers(num_w) space(1)
        let gutter_w = (1 + 1 + num_w + 1 + 1).min(area.width);
        self.gutter_w = gutter_w;
        let has_sb = self.scroll.overflows();
        let text_area = Rect::new(
            area.x + gutter_w,
            area.y,
            area.width.saturating_sub(gutter_w + u16::from(has_sb)),
            body_h,
        );
        let viewport_changed =
            self.text_area.width != text_area.width || self.text_area.height != text_area.height;
        self.text_area = text_area;
        let cur = self.buffer.cursor_pos();
        let cur_off = self.buffer.cursor_offset();
        let rendered_state = (cur_off, self.cached_for, s.editing);
        if s.editing && (viewport_changed || self.rendered_state != Some(rendered_state)) {
            self.scroll.ensure_visible(cur.line);
        }
        self.rendered_state = Some(rendered_state);
        let block = if self.buffer.selection().is_none() {
            self.current_block()
        } else {
            None
        };
        let running = self.running.clone();
        let sel = self.buffer.selection();
        let find = self.find.clone();
        let diags = self.diagnostics.clone();
        let bracket = self.bracket_pair();

        // line offsets
        let mut line_starts = Vec::with_capacity(line_count);
        let mut acc = 0;
        for l in &lines {
            line_starts.push(acc);
            acc += l.len() + 1;
        }
        let tone_at = |off: usize| -> SyntaxTone {
            spans
                .iter()
                .find(|(r, _)| r.contains(&off))
                .map(|(_, tn)| *tn)
                .unwrap_or(SyntaxTone::Plain)
        };
        let gutter_bar = t.gutter(s, fs.bg.unwrap_or(bg), false);
        for row in 0..rows {
            let li = self.scroll.offset + row;
            let y = area.y + row as u16;
            // the bar marks the line the cursor is on, like a row in a list
            if li == cur.line {
                put(buf, area.x, y, t.gutter_symbol(s), gutter_bar);
            }
            if li >= line_count {
                continue;
            }
            let line = lines[li];
            let ls = line_starts[li];
            let le = ls + line.len();
            // marker column
            let mut marker = " ";
            let mut marker_style = fs;
            if let Some(r) = &running {
                if r.start >= ls && r.start <= le {
                    marker = crate::widgets::progress::spinner_frame(ctx.interaction.tick);
                    marker_style = fs.fg(t.accent);
                }
            } else if let Some(b) = &block {
                let first_content = b.start;
                if first_content >= ls && first_content <= le {
                    marker = "›";
                    marker_style = fs.fg(if focused { t.accent } else { t.text_secondary });
                }
            }
            // a diagnostic owns the marker slot
            let diag_here = diags
                .iter()
                .find(|d| d.range.start >= ls && d.range.start <= le);
            if let Some(d) = diag_here {
                let c = if d.severity == Severity::Error {
                    t.error
                } else {
                    t.warning
                };
                marker = "!";
                marker_style = fs.fg(c).add_modifier(Modifier::BOLD);
            }
            put(buf, area.x + 1, y, marker, marker_style);
            // line number
            let in_block = block.as_ref().is_some_and(|b| le >= b.start && ls <= b.end);
            let ns = if li == cur.line && focused {
                fs.fg(t.text_primary).add_modifier(Modifier::BOLD)
            } else if in_block {
                fs.fg(t.text_secondary)
            } else {
                fs.fg(t.text_muted)
            };
            let ns = if self.read_only {
                fs.fg(t.text_faint)
            } else {
                ns
            };
            put(
                buf,
                area.x + 3,
                y,
                &crate::ui::text::fit_right(&(li + 1).to_string(), num_w as usize),
                ns,
            );
            if text_area.width == 0 {
                continue;
            }
            // text
            let mut x = text_area.x;
            let mut col = 0usize;
            if self.hscroll > 0 && !line.is_empty() {
                put(buf, x, y, "…", fs.fg(t.text_muted));
            }
            let underline_line = s.editing && li == cur.line;
            for (gi, g) in line.grapheme_indices(true) {
                let gw = width(g);
                if col + gw <= self.hscroll {
                    col += gw;
                    continue;
                }
                if col >= self.hscroll && col == self.hscroll && self.hscroll > 0 {
                    // first visible cell is the `…`
                    col += gw;
                    x += gw as u16;
                    continue;
                }
                if x + gw as u16 > text_area.right() {
                    put(
                        buf,
                        text_area.right().saturating_sub(1),
                        y,
                        "…",
                        fs.fg(t.text_muted),
                    );
                    break;
                }
                let off = ls + gi;
                let mut st = fs.patch(t.syntax(tone_at(off)));
                if let Some(r) = &sel
                    && r.contains(&off)
                {
                    st = st.bg(t.popover);
                }
                if let Some(f) = &find
                    && let Some(mi) = f.matches.iter().position(|m| m.contains(&off))
                {
                    st = if mi == f.current {
                        st.bg(t.popover)
                    } else {
                        st.add_modifier(Modifier::UNDERLINED)
                            .underline_color(t.border_strong)
                    };
                }
                if let Some(d) = diags.iter().find(|d| {
                    d.range.contains(&off) || (d.range.is_empty() && d.range.start == off)
                }) {
                    let c = if d.severity == Severity::Error {
                        t.error
                    } else {
                        t.warning
                    };
                    st = st.add_modifier(Modifier::UNDERLINED).underline_color(c);
                }
                if bracket.is_some_and(|(a, b)| off == a || off == b) {
                    st = st
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                        .underline_color(t.border_strong);
                }
                if underline_line && !st.add_modifier.contains(Modifier::UNDERLINED) {
                    st = st
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.border_strong);
                }
                put(buf, x, y, g, st);
                x += gw as u16;
                col += gw;
            }
            if underline_line {
                for xx in x..text_area.right() {
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
        if self.buffer.is_empty() && !self.placeholder.is_empty() && !s.editing {
            put(
                buf,
                text_area.x,
                text_area.y,
                &crate::ui::text::truncate(&self.placeholder, text_area.width as usize),
                fs.fg(t.text_muted),
            );
        }
        if s.editing
            && let Some(cursor) = self.cursor_cell()
        {
            ctx.set_cursor(Position::new(cursor.x, cursor.y));
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, body_h);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        // footer row
        let fy = area.y + body_h;
        if let Some(f) = &self.find {
            let left = area.x + u16::from(area.width > 2);
            let available = (area.right() - left) as usize;
            let label = if available >= 8 { "find " } else { "" };
            put(buf, left, fy, label, fs.fg(t.text_muted));
            let nx = left + label.len() as u16;
            let ns = if f.editing {
                fs.add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.accent)
            } else {
                fs
            };
            let count = if f.matches.is_empty() {
                if f.needle.is_empty() {
                    String::new()
                } else {
                    "no matches".into()
                }
            } else {
                format!("{}/{}", f.current + 1, f.matches.len())
            };
            let count_width = width(&count);
            let count_room = if !count.is_empty() && available >= label.len() + count_width + 4 {
                count_width + 1
            } else {
                0
            };
            let query_room = available.saturating_sub(label.len() + count_room);
            // Keep the query's insertion point visible without splitting a
            // grapheme. Footer position readout yields to active search.
            let mut start = f.needle.len();
            let mut cells = 0;
            for (offset, grapheme) in f.needle.grapheme_indices(true).rev() {
                let next = cells + width(grapheme);
                if next > query_room.saturating_sub(1) {
                    break;
                }
                cells = next;
                start = offset;
            }
            put(buf, nx, fy, &f.needle[start..], ns);
            if f.editing {
                ctx.cursor = None;
                if focused && query_room > 0 {
                    ctx.set_cursor(Position::new(nx + cells as u16, fy));
                }
            }
            if count_room > 0 {
                put(
                    buf,
                    area.right() - count_width as u16,
                    fy,
                    &count,
                    fs.fg(t.text_muted),
                );
            }
            return;
        }
        let pos = if s.editing || focused {
            format!("ln {}/{} · col {}", cur.line + 1, line_count, cur.col + 1)
        } else if self.scroll.overflows() {
            scrollbar::position_label(&self.scroll)
        } else {
            String::new()
        };
        let right_padding = u16::from(area.width > 1);
        let pos = crate::ui::text::truncate(&pos, (area.width - right_padding) as usize);
        let px = area
            .right()
            .saturating_sub(width(&pos) as u16 + right_padding)
            .max(area.x);
        if let Some(d) = diags.iter().min_by_key(|d| d.range.start.abs_diff(cur_off)) {
            let c = if d.severity == Severity::Error {
                t.error
            } else {
                t.warning
            };
            // the diagnostic yields to the position readout on its right
            let room = px.saturating_sub(area.x + 2) as usize;
            put(
                buf,
                area.x + 1,
                fy,
                &crate::ui::text::truncate(&d.message, room),
                fs.fg(c),
            );
        }
        if !pos.is_empty() {
            put(buf, px, fy, &pos, fs.fg(t.text_faint));
        }
    }

    fn bracket_pair(&self) -> Option<(usize, usize)> {
        let text = self.buffer.text().as_bytes();
        let cur = self.buffer.cursor_offset();
        let at = |i: usize| text.get(i).copied();
        let probe = [cur, cur.wrapping_sub(1)];
        for &p in &probe {
            let Some(c) = at(p) else { continue };
            let (open, close, forward) = match c {
                b'(' => (b'(', b')', true),
                b')' => (b'(', b')', false),
                b'[' => (b'[', b']', true),
                b']' => (b'[', b']', false),
                _ => continue,
            };
            let mut depth = 0i32;
            if forward {
                for (i, &b) in text.iter().enumerate().skip(p) {
                    if b == open {
                        depth += 1;
                    } else if b == close {
                        depth -= 1;
                        if depth == 0 {
                            return Some((p, i));
                        }
                    }
                }
            } else {
                for i in (0..=p).rev() {
                    let b = text[i];
                    if b == close {
                        depth += 1;
                    } else if b == open {
                        depth -= 1;
                        if depth == 0 {
                            return Some((i, p));
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn render(editor: &mut CodeEditor, width: u16, height: u16) -> Option<Position> {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(editor.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        editor.render(area, &mut buf, &mut ctx, theme.surface);
        ctx.cursor
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    #[test]
    fn modified_enter_commits_without_changing_document() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "hello");
        editor.begin_edit();
        assert_eq!(
            editor.on_key(&Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::CONTROL
            }),
            (Outcome::Changed, Some(EditorEvent::Committed))
        );
        assert!(!editor.editing);
        assert_eq!(editor.text(), "hello");
    }

    #[test]
    fn find_uses_original_grapheme_ranges_after_case_expansion() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "İé é\u{301}");
        editor.open_find();
        editor.on_key(&key(KeyCode::Char('é')));
        assert_eq!(editor.find.as_ref().unwrap().matches, vec![2..4, 5..9]);
        assert_eq!(editor.cursor_offset(), 2);
    }

    #[test]
    fn paste_and_backspace_edit_find_query_in_each_document_mode() {
        for (editing, read_only) in [(false, false), (true, false), (false, true)] {
            let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "a\u{301}");
            editor.editing = editing;
            editor.read_only = read_only;
            editor.open_find();
            assert_eq!(editor.on_paste("a\u{301}"), Outcome::Changed);
            assert_eq!(editor.text(), "a\u{301}");
            assert_eq!(editor.find.as_ref().unwrap().needle, "a\u{301}");
            editor.on_key(&key(KeyCode::Backspace));
            assert_eq!(editor.find.as_ref().unwrap().needle, "");
        }
    }

    #[test]
    fn replacing_document_refreshes_existing_find_matches() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "abc");
        editor.open_find();
        editor.on_paste("a");
        editor.set_text("xyz");
        assert!(editor.find.as_ref().unwrap().matches.is_empty());
    }

    #[test]
    fn manual_scroll_survives_render_then_typing_reveals_cursor() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"line\n".repeat(30));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.on_wheel(3, false);
        let offset = editor.scroll.offset;
        assert!(offset > 0);
        assert_eq!(render(&mut editor, 40, 8), None);
        assert_eq!(editor.scroll.offset, offset);
        assert_eq!(editor.cursor_cell(), None);
        editor.on_key(&key(KeyCode::Char('x')));
        assert!(render(&mut editor, 40, 8).is_some());
        assert_eq!(editor.scroll.offset, 0);
    }

    #[test]
    fn horizontal_scroll_hides_offscreen_cursor() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"x".repeat(100));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.on_wheel(3, true);
        assert_eq!(render(&mut editor, 40, 8), None);
        assert_eq!(editor.cursor_cell(), None);
    }

    #[test]
    fn focused_read_only_editor_keeps_monochrome_navigation_gutter() {
        let theme = Theme::for_level(ColorLevel::Mono);
        let area = Rect::new(0, 0, 40, 8);
        let mut buf = Buffer::empty(area);
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "hello").read_only(true);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(editor.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        editor.render(area, &mut buf, &mut ctx, theme.surface);
        assert_eq!(buf[(0, 0)].symbol(), "▎");
    }

    #[test]
    fn switching_to_read_only_disables_all_document_edit_actions() {
        for code in [KeyCode::Tab, KeyCode::BackTab, KeyCode::Char('x')] {
            let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "  text");
            editor.begin_edit();
            editor.read_only = true;
            editor.on_key(&key(code));
            assert_eq!(editor.text(), "  text");
            assert!(!editor.editing);
        }
    }

    #[test]
    fn smaller_viewport_reveals_cursor_after_manual_scroll() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"line\n".repeat(30));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.on_wheel(3, false);
        render(&mut editor, 40, 8);
        assert!(editor.scroll.offset > 0);
        assert!(render(&mut editor, 20, 4).is_some());
        assert_eq!(editor.scroll.offset, 0);
    }

    #[test]
    fn narrow_editor_and_long_find_stay_inside_nonzero_area() {
        for width in [1, 3, 7, 16, 40] {
            for find in [false, true] {
                let theme = Theme::for_level(ColorLevel::Mono);
                let outer = Rect::new(0, 0, 60, 10);
                let area = Rect::new(8, 2, width, 5);
                let mut buf = Buffer::empty(outer);
                for cell in &mut buf.content {
                    cell.set_symbol("·");
                }
                let mut editor = CodeEditor::new(
                    WidgetId::of("test.editor"),
                    "界🙂 abcdefghijklmnopqrstuvwxyz",
                );
                editor.begin_edit();
                if find {
                    editor.open_find();
                    editor.on_paste(&"界🙂a\u{301}".repeat(20));
                }
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(
                    &theme,
                    Interaction {
                        focus: Some(editor.id),
                        ..Default::default()
                    },
                    &mut hits,
                    &mut ring,
                );
                editor.render(area, &mut buf, &mut ctx, theme.surface);
                if let Some(cursor) = ctx.cursor {
                    assert!(area.contains(cursor), "cursor {cursor:?}, area {area:?}");
                }
                for y in 0..outer.height {
                    for x in 0..outer.width {
                        if !area.contains(Position::new(x, y)) {
                            assert_eq!(
                                buf[(x, y)].symbol(),
                                "·",
                                "spill at {x},{y}, area {area:?}, find {find}"
                            );
                        }
                    }
                }
            }
        }
    }
}
