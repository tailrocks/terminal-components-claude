//! Editable text model shared by single-line inputs and text areas.
//!
//! Stores text as a `String` and addresses positions by byte offset, while
//! every cursor movement is grapheme aware. Rendering code asks for the
//! logical lines and cursor column; it never edits the string itself.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
        let text = text.into();
        Self {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: false,
        }
    }

    pub fn multi(text: impl Into<String>) -> Self {
        let text = text.into();
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

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
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
        let before = &self.text[..from];
        let trimmed = before.trim_end_matches(|c: char| !c.is_alphanumeric());
        let word_start = trimmed
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_alphanumeric())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        if trimmed.is_empty() { 0 } else { word_start }
    }

    fn next_word(&self, from: usize) -> usize {
        let after = &self.text[from..];
        let mut it = after.char_indices().peekable();
        let mut i = 0;
        while let Some((idx, c)) = it.next() {
            i = idx + c.len_utf8();
            if !c.is_alphanumeric() {
                continue;
            }
            for (idx2, c2) in it.by_ref() {
                if !c2.is_alphanumeric() {
                    i = idx2;
                    return from + i;
                }
                i = idx2 + c2.len_utf8();
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
                true
            }
            None => {
                self.anchor = None;
                false
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' && !self.multiline {
            return;
        }
        self.delete_selection();
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        let s: String = if self.multiline {
            s.to_owned()
        } else {
            s.chars().filter(|c| *c != '\n' && *c != '\r').collect()
        };
        self.text.insert_str(self.cursor, &s);
        self.cursor += s.len();
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_boundary(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.next_boundary(self.cursor);
        self.text.replace_range(self.cursor..end, "");
    }

    pub fn delete_word_left(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.prev_word(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
    }

    pub fn delete_to_line_end(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.line_end(self.cursor);
        self.text.replace_range(self.cursor..end, "");
    }

    pub fn delete_to_line_start(&mut self) {
        if self.delete_selection() {
            return;
        }
        let start = self.line_start(self.cursor);
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
    }

    // --- geometry ------------------------------------------------------------

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    /// Cursor as (line, display column).
    pub fn cursor_pos(&self) -> CursorPos {
        Self::pos_of(&self.text, self.cursor)
    }

    pub fn pos_of(text: &str, offset: usize) -> CursorPos {
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
}
