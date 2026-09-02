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
use crate::ui::text::width;
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
    pub matches: Vec<Range<usize>>,
    pub current: usize,
    pub editing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorEvent {
    Changed,
    CursorMoved,
    /// Esc from editing: back to navigation (document kept).
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
        if f.needle.is_empty() {
            return;
        }
        let case_sensitive = f.needle.chars().any(|c| c.is_uppercase());
        let hay = if case_sensitive {
            self.buffer.text().to_owned()
        } else {
            self.buffer.text().to_lowercase()
        };
        let needle = if case_sensitive {
            f.needle.clone()
        } else {
            f.needle.to_lowercase()
        };
        let mut start = 0;
        while let Some(p) = hay[start..].find(&needle) {
            let s = start + p;
            f.matches.push(s..s + needle.len());
            start = s + needle.len().max(1);
        }
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
                        f.needle.pop();
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
        if !self.editing {
            return self.nav_key(key);
        }
        match edit_key(key, true) {
            EditAction::Cancel => {
                self.commit();
                (Outcome::Changed, Some(EditorEvent::Committed))
            }
            EditAction::Commit => unreachable!("multiline Enter inserts"),
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

    pub fn on_paste(&mut self, text: &str) -> Outcome {
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
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.editing = self.editing && s.focused;
        s.disabled = self.read_only;
        if !s.focused && self.editing {
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
        // gutter: bar(1) marker(1) numbers(num_w) diag(1) space(1)
        let gutter_w = 1 + 1 + num_w + 1 + 1;
        self.gutter_w = gutter_w;
        let has_sb = self.scroll.overflows();
        let text_area = Rect::new(
            area.x + gutter_w,
            area.y,
            area.width.saturating_sub(gutter_w + u16::from(has_sb)),
            body_h,
        );
        self.text_area = text_area;
        let cur = self.buffer.cursor_pos();
        let cur_off = self.buffer.cursor_offset();
        if s.editing {
            self.scroll.ensure_visible(cur.line);
        }
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
            buf.set_string(area.x, y, "▎", gutter_bar);
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
            buf.set_string(area.x + 1, y, marker, marker_style);
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
            buf.set_string(
                area.x + 2,
                y,
                crate::ui::text::fit_right(&(li + 1).to_string(), num_w as usize),
                ns,
            );
            // diagnostic glyph
            if let Some(d) = diags
                .iter()
                .find(|d| d.range.start >= ls && d.range.start <= le)
            {
                let c = if d.severity == Severity::Error {
                    t.error
                } else {
                    t.warning
                };
                buf.set_string(
                    area.x + 2 + num_w,
                    y,
                    "!",
                    fs.fg(c).add_modifier(Modifier::BOLD),
                );
            }
            // text
            let mut x = text_area.x;
            let mut col = 0usize;
            if self.hscroll > 0 && !line.is_empty() {
                buf.set_string(x, y, "…", fs.fg(t.text_muted));
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
                    buf.set_string(
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
                buf.set_string(x, y, g, st);
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
            buf.set_string(
                text_area.x,
                text_area.y,
                crate::ui::text::truncate(&self.placeholder, text_area.width as usize),
                fs.fg(t.text_muted),
            );
        }
        if s.editing {
            let cx = text_area.x + cur.col.saturating_sub(self.hscroll) as u16;
            let cy = area.y + cur.line.saturating_sub(self.scroll.offset) as u16;
            if cy < area.y + body_h {
                ctx.set_cursor(Position::new(cx.min(text_area.right()), cy));
            }
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, body_h);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        // footer row
        let fy = area.y + body_h;
        if let Some(f) = &self.find {
            let label = "find ".to_string();
            buf.set_string(area.x + 1, fy, &label, fs.fg(t.text_muted));
            let nx = area.x + 1 + label.len() as u16;
            let needle = format!("{} ", f.needle);
            let ns = if f.editing {
                fs.add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.accent)
            } else {
                fs
            };
            buf.set_string(nx, fy, &needle, ns);
            if f.editing {
                ctx.set_cursor(Position::new(nx + width(&f.needle) as u16, fy));
            }
            let count = if f.matches.is_empty() {
                if f.needle.is_empty() {
                    String::new()
                } else {
                    "no matches".into()
                }
            } else {
                format!("{}/{}", f.current + 1, f.matches.len())
            };
            buf.set_string(
                nx + width(&needle) as u16 + 1,
                fy,
                &count,
                fs.fg(t.text_muted),
            );
        } else if let Some(d) = diags.iter().min_by_key(|d| d.range.start.abs_diff(cur_off)) {
            let c = if d.severity == Severity::Error {
                t.error
            } else {
                t.warning
            };
            buf.set_string(
                area.x + 1,
                fy,
                crate::ui::text::truncate(&d.message, area.width.saturating_sub(16) as usize),
                fs.fg(c),
            );
        }
        let pos = if s.editing || focused {
            format!("ln {}/{} · col {}", cur.line + 1, line_count, cur.col + 1)
        } else if self.scroll.overflows() {
            scrollbar::position_label(&self.scroll)
        } else {
            String::new()
        };
        if !pos.is_empty() {
            let px = area.right().saturating_sub(width(&pos) as u16 + 1);
            buf.set_string(px, fy, &pos, fs.fg(t.text_faint));
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
