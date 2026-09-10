//! Editable text model shared by single-line inputs and text areas.
//!
//! Stores text as a `String` and addresses positions by byte offset, while
//! every cursor movement is grapheme aware. Rendering code asks for the
//! logical lines and cursor column; it never edits the string itself.
//! Single-line buffers discard line breaks at every text entry point;
//! multiline buffers normalize CRLF and CR to LF.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn normalized_text(text: &str, multiline: bool) -> std::borrow::Cow<'_, str> {
    if multiline && text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n").into()
    } else if !multiline && text.contains(['\r', '\n']) {
        text.chars()
            .filter(|c| !matches!(c, '\r' | '\n'))
            .collect::<String>()
            .into()
    } else {
        text.into()
    }
}

fn normalize_owned(text: String, multiline: bool) -> String {
    match normalized_text(&text, multiline) {
        std::borrow::Cow::Borrowed(_) => text,
        std::borrow::Cow::Owned(normalized) => normalized,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
    /// Byte offset of the cursor.
    cursor: usize,
    /// Byte offset of the selection anchor when a selection is active.
    anchor: Option<usize>,
    multiline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub line: usize,
    /// Display column (in cells) within the line.
    pub col: usize,
}

impl TextBuffer {
    pub fn single(text: impl Into<String>) -> Self {
        let text = normalize_owned(text.into(), false);
        Self {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: false,
        }
    }

    pub fn multi(text: impl Into<String>) -> Self {
        let text = normalize_owned(text.into(), true);
        Self {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[cfg(test)]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn cursor_offset(&self) -> usize {
        self.cursor
    }

    /// Select `a..b` (either order), cursor at `b`. Offsets are clamped to
    /// the preceding grapheme boundary, including offsets inside UTF-8 bytes.
    pub fn select_range(&mut self, a: usize, b: usize) {
        self.anchor = Some(Self::floor_boundary(&self.text, a));
        self.cursor = Self::floor_boundary(&self.text, b);
    }

    /// First and last line touched by the selection (or the cursor line).
    pub fn selection_lines(&self) -> (usize, usize) {
        match self.selection() {
            Some(r) => {
                let a = Self::pos_of(&self.text, r.start).line;
                // An exclusive endpoint at the next line's start does not
                // select that line. Count newlines without slicing a UTF-8 byte.
                let before = &self.text[..r.end];
                let b = before.matches('\n').count() - usize::from(before.ends_with('\n'));
                (a, b)
            }
            None => {
                let l = self.cursor_pos().line;
                (l, l)
            }
        }
    }

    pub fn has_selection_lines(&self) -> bool {
        self.selection()
            .is_some_and(|r| self.text[r].contains('\n'))
    }

    /// Insert at an arbitrary offset, clamped to the preceding grapheme
    /// boundary, keeping cursor/anchor consistent.
    pub fn insert_at(&mut self, at: usize, s: &str) {
        let at = Self::floor_boundary(&self.text, at);
        let s = normalized_text(s, self.multiline);
        self.text.insert_str(at, &s);
        if self.cursor >= at {
            self.cursor += s.len();
        }
        if let Some(a) = self.anchor.as_mut()
            && *a >= at
        {
            *a += s.len();
        }
        self.normalize_positions();
    }

    /// Remove all graphemes touched by a nonempty byte range. Endpoints are
    /// clamped to the text; reversed or empty ranges do nothing.
    pub fn remove_range(&mut self, r: std::ops::Range<usize>) {
        let r = r.start.min(self.text.len())..r.end.min(self.text.len());
        if r.start >= r.end {
            return;
        }
        let r = Self::floor_boundary(&self.text, r.start)..Self::ceil_boundary(&self.text, r.end);
        let n = r.end - r.start;
        self.text.replace_range(r.clone(), "");
        let adjust = |p: &mut usize| {
            if *p >= r.end {
                *p -= n;
            } else if *p > r.start {
                *p = r.start;
            }
        };
        adjust(&mut self.cursor);
        if let Some(a) = self.anchor.as_mut() {
            adjust(a);
        }
        self.normalize_positions();
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = normalize_owned(text.into(), self.multiline);
        self.cursor = self.text.len();
        self.anchor = None;
    }

    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some(a.min(self.cursor)..a.max(self.cursor))
    }

    #[cfg(test)]
    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    #[cfg(test)]
    pub fn selected_text(&self) -> Option<&str> {
        self.selection().map(|r| &self.text[r])
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

    // --- byte offset helpers -------------------------------------------------

    fn floor_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i <= offset)
            .last()
            .unwrap_or(0)
    }

    fn ceil_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|i| *i >= offset)
            .unwrap_or(text.len())
    }

    // Inserting or removing text can join previously separate clusters (a
    // combining mark, ZWJ sequence or regional-indicator pair). Repair both
    // positions against the new string, not just by adding byte lengths.
    fn normalize_positions(&mut self) {
        self.cursor = Self::ceil_boundary(&self.text, self.cursor);
        self.anchor = self.anchor.map(|a| Self::floor_boundary(&self.text, a));
    }

    fn prev_boundary(&self, from: usize) -> usize {
        self.text[..from]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, from: usize) -> usize {
        self.text[from..]
            .graphemes(true)
            .next()
            .map(|g| from + g.len())
            .unwrap_or(from)
    }

    fn line_start(&self, from: usize) -> usize {
        self.text[..from].rfind('\n').map(|i| i + 1).unwrap_or(0)
    }

    fn line_end(&self, from: usize) -> usize {
        self.text[from..]
            .find('\n')
            .map(|i| from + i)
            .unwrap_or(self.text.len())
    }

    fn prev_word(&self, from: usize) -> usize {
        let mut graphemes = self.text[..from].grapheme_indices(true).rev().peekable();
        while graphemes
            .peek()
            .is_some_and(|(_, g)| !g.chars().any(char::is_alphanumeric))
        {
            graphemes.next();
        }
        let mut start = 0;
        while let Some(&(i, g)) = graphemes.peek() {
            if !g.chars().any(char::is_alphanumeric) {
                break;
            }
            start = i;
            graphemes.next();
        }
        start
    }

    fn next_word(&self, from: usize) -> usize {
        let after = &self.text[from..];
        let mut it = after.grapheme_indices(true);
        let mut i = 0;
        while let Some((idx, g)) = it.next() {
            i = idx + g.len();
            if !g.chars().any(char::is_alphanumeric) {
                continue;
            }
            for (idx2, g2) in it.by_ref() {
                if !g2.chars().any(char::is_alphanumeric) {
                    i = idx2;
                    return from + i;
                }
                i = idx2 + g2.len();
            }
            break;
        }
        from + i
    }

    // --- movement ------------------------------------------------------------

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

    /// Move to the same display column on the previous line.
    pub fn move_up(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line == 0 {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line - 1, col);
        true
    }

    pub fn move_down(&mut self, select: bool) -> bool {
        if !self.multiline {
            return false;
        }
        let CursorPos { line, col } = self.cursor_pos();
        if line + 1 >= self.line_count() {
            return false;
        }
        self.begin_move(select);
        self.cursor = self.offset_at(line + 1, col);
        true
    }

    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.anchor = None;
        self.cursor = self.offset_at(line, col);
    }

    // --- editing -------------------------------------------------------------

    fn delete_selection(&mut self) -> bool {
        match self.selection() {
            Some(r) => {
                self.text.replace_range(r.clone(), "");
                self.cursor = r.start;
                self.anchor = None;
                self.normalize_positions();
                true
            }
            None => {
                self.anchor = None;
                false
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if matches!(c, '\r' | '\n') && !self.multiline {
            return;
        }
        let c = if c == '\r' { '\n' } else { c };
        self.insert_str(c.encode_utf8(&mut [0; 4]));
    }

    pub fn insert_str(&mut self, s: &str) {
        let s = normalized_text(s, self.multiline);
        // Replace atomically: deleting first can join the neighboring
        // graphemes and move the cursor before insertion occurs.
        if let Some(r) = self.selection() {
            self.cursor = r.start + s.len();
            self.text.replace_range(r, &s);
        } else {
            self.text.insert_str(self.cursor, &s);
            self.cursor += s.len();
        }
        self.anchor = None;
        self.normalize_positions();
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_boundary(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.next_boundary(self.cursor);
        self.text.replace_range(self.cursor..end, "");
        self.normalize_positions();
    }

    pub fn delete_word_left(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_word(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    pub fn delete_to_line_end(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.line_end(self.cursor);
        self.text.replace_range(self.cursor..end, "");
        self.normalize_positions();
    }

    pub fn delete_to_line_start(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.line_start(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.normalize_positions();
    }

    // --- geometry ------------------------------------------------------------

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    /// Cursor as (line, display column).
    pub fn cursor_pos(&self) -> CursorPos {
        Self::pos_of(&self.text, self.cursor)
    }

    /// Resolve a byte offset to a display position, clamping to the preceding
    /// grapheme boundary. Arbitrary offsets never split UTF-8 or a cluster.
    pub fn pos_of(text: &str, offset: usize) -> CursorPos {
        let offset = Self::floor_boundary(text, offset);
        let before = &text[..offset];
        let line = before.matches('\n').count();
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = UnicodeWidthStr::width(&text[line_start..offset]);
        CursorPos { line, col }
    }

    /// Byte offset for a (line, display column), clamped to the line.
    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        let mut start = 0;
        for (i, l) in self.text.split('\n').enumerate() {
            if i == line {
                let mut width = 0;
                for (gi, g) in l.grapheme_indices(true) {
                    let w = UnicodeWidthStr::width(g);
                    if width + w > col {
                        return start + gi;
                    }
                    width += w;
                }
                return start + l.len();
            }
            start += l.len() + 1;
        }
        self.text.len()
    }

    /// Display width of the whole (single-line) text.
    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(self.text.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_move_by_grapheme() {
        let mut b = TextBuffer::single("");
        for c in "héllo".chars() {
            b.insert_char(c);
        }
        assert_eq!(b.text(), "héllo");
        b.move_left(false);
        b.move_left(false);
        b.insert_char('X');
        assert_eq!(b.text(), "hélXlo");
        b.move_home(false);
        b.delete();
        assert_eq!(b.text(), "élXlo");
        b.move_end(false);
        b.backspace();
        assert_eq!(b.text(), "élXl");
    }

    #[test]
    fn selection_replaces_on_insert() {
        let mut b = TextBuffer::single("hello world");
        b.move_home(false);
        for _ in 0..5 {
            b.move_right(true);
        }
        assert_eq!(b.selected_text(), Some("hello"));
        b.insert_str("bye");
        assert_eq!(b.text(), "bye world");
        assert!(!b.has_selection());
    }

    #[test]
    fn word_motion_and_deletion() {
        let mut b = TextBuffer::single("alpha beta  gamma");
        b.move_word_left(false);
        assert_eq!(b.cursor(), 12);
        b.move_word_left(false);
        assert_eq!(b.cursor(), 6);
        b.move_home(false);
        b.move_word_right(false);
        assert_eq!(b.cursor(), 5);
        b.move_end(false);
        b.delete_word_left();
        assert_eq!(b.text(), "alpha beta  ");
    }

    #[test]
    fn multiline_vertical_motion_keeps_column() {
        let mut b = TextBuffer::multi("first line\nsecond\nthird line here");
        b.move_doc_start(false);
        b.move_end(false);
        assert_eq!(b.cursor_pos(), CursorPos { line: 0, col: 10 });
        assert!(b.move_down(false));
        assert_eq!(b.cursor_pos(), CursorPos { line: 1, col: 6 });
        assert!(b.move_down(false));
        assert_eq!(b.cursor_pos(), CursorPos { line: 2, col: 6 });
        assert!(!b.move_down(false));
        assert!(b.move_up(false));
        assert!(b.move_up(false));
        assert!(!b.move_up(false));
    }

    #[test]
    fn single_line_rejects_newline() {
        let mut b = TextBuffer::single("a");
        b.insert_char('\n');
        assert_eq!(b.text(), "a");
        b.insert_str("b\nc");
        assert_eq!(b.text(), "abc");
        let mut m = TextBuffer::multi("a");
        m.insert_char('\n');
        assert_eq!(m.line_count(), 2);
    }

    #[test]
    fn wide_characters_count_as_two_columns() {
        let b = TextBuffer::single("日本");
        assert_eq!(b.cursor_pos().col, 4);
        assert_eq!(b.offset_at(0, 2), 3);
        assert_eq!(b.offset_at(0, 1), 0);
    }

    #[test]
    fn selection_lines_handles_unicode_and_exclusive_newline() {
        let mut b = TextBuffer::multi("é\n日本\n👩‍💻");
        b.select_all();
        assert_eq!(b.selection_lines(), (0, 2));
        b.select_range(0, "é\n".len());
        assert_eq!(b.selection_lines(), (0, 0));
        b.select_range("é\n".len(), "é\n日本".len());
        assert_eq!(b.selection_lines(), (1, 1));
    }

    #[test]
    fn public_offsets_clamp_to_graphemes() {
        let mut b = TextBuffer::single("e\u{301}日👩‍💻");
        for a in 0..=b.text().len() + 1 {
            for end in 0..=b.text().len() + 1 {
                b.select_range(a, end);
                b.cursor_pos();
                b.selection_lines();
                if let Some(r) = b.selection() {
                    assert!(b.text().get(r).is_some());
                }
            }
        }
        b.select_range(1, 2);
        assert_eq!(b.cursor(), 0);
        assert_eq!(b.selection(), None);
        b.insert_at(4, "X");
        assert_eq!(b.text(), "e\u{301}X日👩‍💻");
        b.remove_range(5..6);
        assert_eq!(b.text(), "e\u{301}X👩‍💻");
        assert_eq!(TextBuffer::pos_of("é", usize::MAX).col, 1);
        assert_eq!(TextBuffer::pos_of("é", 1).col, 0);
    }

    #[test]
    fn word_motion_and_deletion_preserve_combining_clusters() {
        let mut b = TextBuffer::single("cafe\u{301} 日本");
        b.move_home(false);
        b.move_word_right(true);
        assert_eq!(b.selected_text(), Some("cafe\u{301}"));
        b.clear_selection();
        b.move_word_left(false);
        assert_eq!(b.cursor(), 0);
        b.move_end(false);
        b.delete_word_left();
        b.delete_word_left();
        assert_eq!(b.text(), "");
    }

    #[test]
    fn edits_repair_positions_when_graphemes_join() {
        let mut b = TextBuffer::single("👩💻");
        b.set_cursor_line_col(0, 2);
        b.insert_char('\u{200d}');
        assert_eq!(b.text(), "👩‍💻");
        assert_eq!(b.cursor(), b.text().len());
        b.backspace();
        assert_eq!(b.text(), "");

        let mut b = TextBuffer::single("🇺 🇸");
        b.set_cursor_line_col(0, 1);
        b.remove_range(4..5);
        assert_eq!(b.text(), "🇺🇸");
        assert_eq!(b.cursor(), b.text().len());
        b.move_left(false);
        assert_eq!(b.cursor(), 0);
    }

    #[test]
    fn selection_replacement_does_not_move_past_joining_neighbors() {
        for paste in [false, true] {
            let mut b = TextBuffer::single("🇺 🇸");
            b.select_range(4, 5);
            if paste {
                b.insert_str("X");
            } else {
                b.insert_char('X');
            }
            assert_eq!(b.text(), "🇺X🇸");
            assert_eq!(b.cursor(), 5);
        }
    }

    #[test]
    fn every_text_entry_preserves_line_mode() {
        let mut b = TextBuffer::single("a\r\nb");
        assert_eq!(b.text(), "ab");
        b.set_text("c\nd");
        b.insert_at(1, "\r\nx");
        b.insert_char('\r');
        assert_eq!(b.text(), "cxd");
        let mut b = TextBuffer::multi("a\r\nb\rc");
        assert_eq!(b.text(), "a\nb\nc");
        b.insert_str("\r\nd");
        b.move_doc_start(false);
        b.move_end(false);
        assert_eq!(b.cursor(), 1);
        assert_eq!(b.line_count(), 4);
    }
}
