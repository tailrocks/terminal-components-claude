use core::fmt;
use core::ops::Range;
use std::borrow::Cow;

use super::measure::{grapheme_width, graphemes, is_word_grapheme, width};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

#[derive(Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
    multiline: bool,
    sensitive: bool,
}

impl Clone for TextBuffer {
    fn clone(&self) -> Self {
        if self.sensitive {
            return TextBuffer {
                text: String::new(),
                cursor: 0,
                anchor: None,
                multiline: self.multiline,
                sensitive: true,
            };
        }
        TextBuffer {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
            multiline: self.multiline,
            sensitive: false,
        }
    }
}

impl fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextBuffer")
            .field("text", &"[redacted]")
            .field("len", &self.text.len())
            .field("cursor", &self.cursor)
            .field("anchor", &self.anchor)
            .field("multiline", &self.multiline)
            .field("sensitive", &self.sensitive)
            .finish()
    }
}

impl Drop for TextBuffer {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub(super) fn normalized_text(text: &str, multiline: bool) -> Cow<'_, str> {
    if multiline && text.contains('\r') {
        let mut normalized = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\r' {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                normalized.push('\n');
            } else {
                normalized.push(ch);
            }
        }
        normalized.into()
    } else if !multiline && text.contains(['\r', '\n']) {
        text.chars()
            .filter(|c| !matches!(c, '\r' | '\n'))
            .collect::<String>()
            .into()
    } else {
        text.into()
    }
}

fn normalize_owned(text: String, multiline: bool, sensitive: bool) -> String {
    match normalized_text(&text, multiline) {
        Cow::Borrowed(_) => text,
        Cow::Owned(normalized) => {
            if sensitive {
                wipe_string(text);
            }
            normalized
        }
    }
}

impl TextBuffer {
    fn floor_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        graphemes(text)
            .map(|(i, _)| i)
            .take_while(|i| *i <= offset)
            .last()
            .unwrap_or(0)
    }

    fn ceil_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        graphemes(text)
            .map(|(i, _)| i)
            .find(|i| *i >= offset)
            .unwrap_or(text.len())
    }

    fn normalize_positions(&mut self) {
        self.cursor = Self::ceil_boundary(&self.text, self.cursor);
        self.anchor = self.anchor.map(|a| Self::floor_boundary(&self.text, a));
    }

    fn from_text(text: String, multiline: bool, sensitive: bool) -> Self {
        let text = normalize_owned(text, multiline, sensitive);
        TextBuffer {
            cursor: text.len(),
            text,
            anchor: None,
            multiline,
            sensitive,
        }
    }

    pub fn single(text: impl Into<String>) -> Self {
        Self::from_text(text.into(), false, false)
    }

    pub(crate) fn sensitive_single(text: &str) -> Self {
        Self::from_text(text.to_owned(), false, true)
    }

    pub fn multi(text: impl Into<String>) -> Self {
        Self::from_text(text.into(), true, false)
    }

    pub(crate) fn sensitive_multi(text: &str) -> Self {
        Self::from_text(text.to_owned(), true, true)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub const fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub const fn cursor_offset(&self) -> usize {
        self.cursor
    }

    pub fn zeroize(&mut self) {
        wipe_string(core::mem::take(&mut self.text));
        self.text = String::new();
        self.cursor = 0;
        self.anchor = None;
    }

    pub fn set_text(&mut self, text: &str) {
        if self.sensitive {
            self.replace_text(normalize_owned(text.to_owned(), self.multiline, true));
            self.anchor = None;
        } else {
            self.zeroize();
            self.text.push_str(&normalized_text(text, self.multiline));
        }
        self.cursor = self.text.len();
    }

    fn replace_text(&mut self, next: String) {
        let old = core::mem::replace(&mut self.text, next);
        if self.sensitive {
            wipe_string(old);
        }
    }

    pub fn select_range(&mut self, a: usize, b: usize) {
        let len = self.text.len();
        self.anchor = Some(self.snap(a.min(len)));
        self.cursor = self.snap(b.min(len));
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some(a.min(self.cursor)..a.max(self.cursor))
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selection().and_then(|r| self.text.get(r))
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    pub fn selection_lines(&self) -> (usize, usize) {
        if let Some(r) = self.selection() {
            let a = Self::pos_of(&self.text, r.start).line;
            let before = &self.text[..r.end.min(self.text.len())];
            let b = before
                .matches('\n')
                .count()
                .saturating_sub(usize::from(before.ends_with('\n')));
            (a, b)
        } else {
            let l = self.cursor_pos().line;
            (l, l)
        }
    }

    fn begin_move(&mut self, select: bool) {
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
    }

    fn snap(&self, at: usize) -> usize {
        Self::floor_boundary(&self.text, at.min(self.text.len()))
    }

    fn prev_boundary(&self, from: usize) -> usize {
        self.text
            .get(..from)
            .and_then(|s| graphemes(s).last())
            .map_or(0, |(i, _)| i)
    }

    fn next_boundary(&self, from: usize) -> usize {
        self.text
            .get(from..)
            .and_then(|s| graphemes(s).next())
            .map_or(from, |(_, g)| from.saturating_add(g.len()))
    }

    fn line_start(&self, from: usize) -> usize {
        self.text
            .get(..from)
            .and_then(|s| s.rfind('\n'))
            .map_or(0, |i| i.saturating_add(1))
    }

    fn line_end(&self, from: usize) -> usize {
        self.text
            .get(from..)
            .and_then(|s| s.find('\n'))
            .map_or(self.text.len(), |i| from.saturating_add(i))
    }

    pub fn word_range_at(&self, offset: usize) -> Range<usize> {
        let offset = Self::floor_boundary(&self.text, offset);
        let (start, end) = (self.line_start(offset), self.line_end(offset));
        let line = &self.text[start..end];
        let clusters: Vec<(usize, &str)> = graphemes(line)
            .map(|(index, grapheme)| (start.saturating_add(index), grapheme))
            .collect();
        let Some(at) = clusters
            .iter()
            .position(|(index, grapheme)| offset < index.saturating_add(grapheme.len()))
            .or_else(|| clusters.len().checked_sub(1))
        else {
            return offset..offset;
        };

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Class {
            Word,
            Space,
            Other,
        }

        let class = |grapheme: &str| {
            if grapheme.chars().any(|ch| ch.is_alphanumeric() || ch == '_') {
                Class::Word
            } else if grapheme.chars().all(char::is_whitespace) {
                Class::Space
            } else {
                Class::Other
            }
        };
        let selected_class = class(clusters[at].1);
        if selected_class == Class::Other {
            let (index, grapheme) = clusters[at];
            return index..index.saturating_add(grapheme.len());
        }

        let mut first = at;
        while first > 0 && class(clusters[first - 1].1) == selected_class {
            first -= 1;
        }
        let mut last = at;
        while last + 1 < clusters.len() && class(clusters[last + 1].1) == selected_class {
            last += 1;
        }
        let start = clusters[first].0;
        let (last_start, last_grapheme) = clusters[last];
        start..last_start.saturating_add(last_grapheme.len())
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let range = self.word_range_at(offset);
        self.select_range(range.start, range.end);
    }

    pub fn select_line_at(&mut self, offset: usize) {
        let offset = Self::floor_boundary(&self.text, offset);
        let start = self.line_start(offset);
        let end = self.line_end(offset);
        self.select_range(start, end);
    }

    fn prev_word(&self, from: usize) -> usize {
        let end = Self::floor_boundary(&self.text, from);
        let mut clusters = graphemes(&self.text[..end]).rev().peekable();
        while clusters.peek().is_some_and(|(_, g)| !is_word_grapheme(g)) {
            clusters.next();
        }
        let mut start = 0;
        while let Some(&(i, g)) = clusters.peek() {
            if !is_word_grapheme(g) {
                break;
            }
            start = i;
            clusters.next();
        }
        start
    }

    fn next_word(&self, from: usize) -> usize {
        let start = Self::floor_boundary(&self.text, from);
        let suffix = &self.text[start..];
        let mut clusters = graphemes(suffix).peekable();
        while clusters.peek().is_some_and(|(_, g)| !is_word_grapheme(g)) {
            clusters.next();
        }
        let mut end = 0;
        for (i, g) in clusters {
            if !is_word_grapheme(g) {
                return start.saturating_add(i);
            }
            end = i.saturating_add(g.len());
        }
        start.saturating_add(end)
    }

    pub fn move_left(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.start;
            return;
        }
        self.begin_move(select);
        self.cursor = self.prev_boundary(self.cursor);
    }

    pub fn move_right(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.end;
            return;
        }
        self.begin_move(select);
        self.cursor = self.next_boundary(self.cursor);
    }

    pub fn move_word_left(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.prev_word(self.cursor);
    }

    pub fn move_word_right(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.next_word(self.cursor);
    }

    pub fn move_home(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_start(self.cursor);
    }

    pub fn move_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_end(self.cursor);
    }

    pub fn move_doc_start(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = 0;
    }

    pub fn move_doc_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.text.len();
    }

    pub fn move_up(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line == 0 {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line.saturating_sub(1), col);
        true
    }

    pub fn move_down(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line.saturating_add(1) >= self.line_count() {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line.saturating_add(1), col);
        true
    }

    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.anchor = None;
        self.cursor = self.offset_at(line, col);
    }

    fn delete_selection(&mut self) -> bool {
        if let Some(r) = self.selection() {
            self.remove_range_raw(r.clone());
            self.cursor = r.start;
            self.anchor = None;
            self.normalize_positions();
            true
        } else {
            self.anchor = None;
            false
        }
    }

    pub fn insert_char(&mut self, c: char) -> bool {
        if matches!(c, '\r' | '\n') && !self.multiline {
            return false;
        }
        let c = if c == '\r' { '\n' } else { c };
        if self.sensitive {
            let range = self.selection().unwrap_or(self.cursor..self.cursor);
            let mut next = String::with_capacity(
                self.text
                    .len()
                    .saturating_sub(range.len())
                    .saturating_add(c.len_utf8()),
            );
            next.push_str(&self.text[..range.start]);
            next.push(c);
            next.push_str(&self.text[range.end..]);
            self.replace_text(next);
            self.cursor = range.start.saturating_add(c.len_utf8());
            self.anchor = None;
            self.normalize_positions();
            return true;
        }
        let range = self.selection().unwrap_or(self.cursor..self.cursor);
        let mut encoded = [0; 4];
        let inserted = c.encode_utf8(&mut encoded);
        self.text.replace_range(range.clone(), inserted);
        self.cursor = range.start.saturating_add(inserted.len());
        self.anchor = None;
        self.normalize_positions();
        true
    }

    pub fn insert_str(&mut self, s: &str) -> bool {
        let ins = normalized_text(s, self.multiline);
        let range = self.selection().unwrap_or(self.cursor..self.cursor);
        let changed = !range.is_empty() || !ins.is_empty();
        if self.sensitive {
            let before = self.text.len().saturating_sub(range.len());
            let mut next = String::with_capacity(before.saturating_add(ins.len()));
            next.push_str(&self.text[..range.start]);
            next.push_str(&ins);
            next.push_str(&self.text[range.end..]);
            let inserted_len = ins.len();
            self.replace_text(next);
            self.cursor = range.start.saturating_add(inserted_len);
            self.anchor = None;
            self.normalize_positions();
            if let Cow::Owned(normalized) = ins {
                wipe_string(normalized);
            }
            return changed;
        }
        self.text.replace_range(range.clone(), &ins);
        self.cursor = range.start.saturating_add(ins.len());
        self.anchor = None;
        self.normalize_positions();
        changed
    }

    pub fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.prev_boundary(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.remove_range_raw(start..self.cursor);
        self.cursor = start;
        self.normalize_positions();
        true
    }

    pub fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = self.next_boundary(self.cursor);
        if end == self.cursor {
            return false;
        }
        self.remove_range_raw(self.cursor..end);
        self.normalize_positions();
        true
    }

    pub fn delete_word_left(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.prev_word(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.remove_range_raw(start..self.cursor);
        self.cursor = start;
        self.normalize_positions();
        true
    }

    pub fn delete_to_line_end(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = self.line_end(self.cursor);
        if end == self.cursor {
            return false;
        }
        self.remove_range_raw(self.cursor..end);
        self.normalize_positions();
        true
    }

    pub fn delete_to_line_start(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.line_start(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.remove_range_raw(start..self.cursor);
        self.cursor = start;
        self.normalize_positions();
        true
    }

    pub fn insert_at(&mut self, at: usize, text: &str) {
        let at = Self::floor_boundary(&self.text, at.min(self.text.len()));
        let inserted = normalized_text(text, self.multiline);
        let inserted_len = inserted.len();
        if inserted_len == 0 {
            if self.sensitive
                && let Cow::Owned(normalized) = inserted
            {
                wipe_string(normalized);
            }
            return;
        }

        if self.sensitive {
            let mut next = String::with_capacity(self.text.len().saturating_add(inserted_len));
            next.push_str(&self.text[..at]);
            next.push_str(&inserted);
            next.push_str(&self.text[at..]);
            self.replace_text(next);
            if let Cow::Owned(normalized) = inserted {
                wipe_string(normalized);
            }
        } else {
            self.text.insert_str(at, &inserted);
        }

        if self.cursor >= at {
            self.cursor = self.cursor.saturating_add(inserted_len);
        }
        if let Some(anchor) = self.anchor {
            self.anchor = Some(if anchor >= at {
                anchor.saturating_add(inserted_len)
            } else {
                anchor
            });
        }
        self.normalize_positions();
    }

    pub fn remove_range(&mut self, range: Range<usize>) {
        let len = self.text.len();
        let start = range.start.min(len);
        let end = range.end.min(len);
        if start >= end {
            return;
        }

        let start = Self::floor_boundary(&self.text, start);
        let end = Self::ceil_boundary(&self.text, end);
        let removed_len = end.saturating_sub(start);
        if removed_len == 0 {
            return;
        }

        self.remove_range_raw(start..end);
        if self.cursor >= end {
            self.cursor = self.cursor.saturating_sub(removed_len);
        } else if self.cursor > start {
            self.cursor = start;
        }
        if let Some(anchor) = self.anchor {
            self.anchor = Some(if anchor >= end {
                anchor.saturating_sub(removed_len)
            } else if anchor > start {
                start
            } else {
                anchor
            });
        }
        self.normalize_positions();
    }

    fn remove_range_raw(&mut self, range: Range<usize>) {
        if self.sensitive {
            self.replace_text(without_range(&self.text, range));
        } else {
            self.text.replace_range(range, "");
        }
    }

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    pub fn cursor_pos(&self) -> CursorPos {
        Self::pos_of(&self.text, self.cursor)
    }

    pub fn pos_of(text: &str, offset: usize) -> CursorPos {
        let mut end = offset.min(text.len());
        while !text.is_char_boundary(end) {
            end = end.saturating_sub(1);
        }
        let before = &text[..end];
        let line = before.matches('\n').count();
        let line_start = before.rfind('\n').map_or(0, |i| i.saturating_add(1));
        let col = usize::from(width(&before[line_start..]));
        CursorPos { line, col }
    }

    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        let mut start = 0usize;
        for (i, l) in self.text.split('\n').enumerate() {
            if i == line {
                let mut w = 0usize;
                for (gi, g) in graphemes(l) {
                    let gw = usize::from(grapheme_width(g));
                    if w.saturating_add(gw) > col {
                        return start.saturating_add(gi);
                    }
                    w = w.saturating_add(gw);
                }
                return start.saturating_add(l.len());
            }
            start = start.saturating_add(l.len()).saturating_add(1);
        }
        self.text.len()
    }

    pub fn width(&self) -> u16 {
        width(&self.text)
    }
}

fn without_range(text: &str, range: Range<usize>) -> String {
    let mut next = String::with_capacity(text.len().saturating_sub(range.len()));
    next.push_str(&text[..range.start]);
    next.push_str(&text[range.end..]);
    next
}

fn wipe_string(value: String) {
    crate::secret::wipe_string(value);
}
