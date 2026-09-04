//! Editable text storage (`COMPONENT_ARCHITECTURE.md` §15, §18.1).
//!
//! Text is a `String` addressed by byte offset; every cursor movement is
//! grapheme aware and every width is the one width function. `Debug`
//! redacts and `zeroize` overwrites bytes before they are released, so a
//! secret draft never reaches a log or lingers in freed memory.

use core::fmt;
use core::ops::Range;

use super::measure::{grapheme_width, graphemes, is_word_char, width};

/// Cursor as `(line, display column)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPos {
    /// Zero-based line.
    pub line: usize,
    /// Display column within the line, in cells.
    pub col: usize,
}

/// Text-coordinate storage retained for the testing-only perf facade.
///
/// Production callers cannot construct or inspect this type through the
/// default facade. Its public surface is limited to allocation-free position
/// queries; editing storage remains private to the text editor.
#[cfg_attr(
    not(feature = "testing"),
    expect(
        unreachable_pub,
        reason = "the coordinate facade is re-exported only for testing"
    )
)]
#[derive(Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
    multiline: bool,
}

impl fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextBuffer")
            .field("text", &"[redacted]")
            .field("len", &self.text.len())
            .field("cursor", &self.cursor)
            .field("anchor", &self.anchor)
            .field("multiline", &self.multiline)
            .finish()
    }
}

impl Drop for TextBuffer {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl TextBuffer {
    /// A single-line buffer with the cursor at the end.
    pub(crate) fn single(text: impl Into<String>) -> Self {
        let text = text.into();
        TextBuffer {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: false,
        }
    }

    /// A multi-line buffer with the cursor at the end.
    #[cfg_attr(
        not(feature = "testing"),
        expect(
            unreachable_pub,
            reason = "the coordinate facade is re-exported only for testing"
        )
    )]
    pub fn multi(text: impl Into<String>) -> Self {
        let text = text.into();
        TextBuffer {
            cursor: text.len(),
            text,
            anchor: None,
            multiline: true,
        }
    }

    /// The text.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// Whether the text is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Whether newlines are accepted.
    pub(crate) const fn is_multiline(&self) -> bool {
        self.multiline
    }

    /// The cursor byte offset.
    pub(crate) const fn cursor_offset(&self) -> usize {
        self.cursor
    }

    /// Overwrite every byte with zero, then clear (§15 `zeroize`).
    pub(crate) fn zeroize(&mut self) {
        crate::secret::zeroize_string(&mut self.text);
        self.cursor = 0;
        self.anchor = None;
    }

    pub(crate) fn clone_plain(&self) -> Self {
        TextBuffer {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
            multiline: self.multiline,
        }
    }

    /// Replace the text; cursor at the end, no selection.
    pub(crate) fn set_text(&mut self, text: &str) {
        let replacement = String::from(text);
        crate::secret::zeroize_string(&mut self.text);
        self.text = replacement;
        self.cursor = self.text.len();
        self.anchor = None;
    }

    /// Make room for an insertion without allowing `String` to release the
    /// old allocation unwiped. Secret drafts use this same buffer, so the
    /// normal `String::insert*` growth path must be guarded too.
    fn reserve_for_insert(&mut self, additional: usize) {
        let required = self.text.len().saturating_add(additional);
        if required <= self.text.capacity() {
            return;
        }
        let mut replacement = String::with_capacity(required);
        replacement.push_str(&self.text);
        crate::secret::zeroize_string(&mut self.text);
        self.text = replacement;
    }

    /// Select `a..b` (either order), cursor at `b`.
    pub(crate) fn select_range(&mut self, a: usize, b: usize) {
        let len = self.text.len();
        self.anchor = Some(self.snap(a.min(len)));
        self.cursor = self.snap(b.min(len));
    }

    /// The selection, if non-empty.
    pub(crate) fn selection(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some(a.min(self.cursor)..a.max(self.cursor))
    }

    /// The selected text, if any.
    pub(crate) fn selected_text(&self) -> Option<&str> {
        self.selection().and_then(|r| self.text.get(r))
    }

    /// Select everything.
    pub(crate) fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.len();
    }

    /// Drop the selection.
    pub(crate) fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// First and last line touched by the selection (or the cursor line).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "retained for text-core unit coverage")
    )]
    pub(crate) fn selection_lines(&self) -> (usize, usize) {
        if let Some(r) = self.selection() {
            let a = Self::pos_of(&self.text, r.start).line;
            let b = Self::pos_of(&self.text, r.end.saturating_sub(1).max(r.start)).line;
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

    /// Snap a byte offset to the nearest preceding char boundary.
    fn snap(&self, mut at: usize) -> usize {
        at = at.min(self.text.len());
        while at > 0 && !self.text.is_char_boundary(at) {
            at = at.saturating_sub(1);
        }
        at
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

    fn prev_word(&self, from: usize) -> usize {
        let before = self.text.get(..from).unwrap_or("");
        let trimmed = before.trim_end_matches(|c: char| !is_word_char(c));
        if trimmed.is_empty() {
            return 0;
        }
        trimmed
            .char_indices()
            .rev()
            .find(|(_, c)| !is_word_char(*c))
            .map_or(0, |(i, c)| i.saturating_add(c.len_utf8()))
    }

    fn next_word(&self, from: usize) -> usize {
        let after = self.text.get(from..).unwrap_or("");
        let mut in_word = false;
        for (i, c) in after.char_indices() {
            if is_word_char(c) {
                in_word = true;
            } else if in_word {
                return from.saturating_add(i);
            }
        }
        self.text.len()
    }

    /// Move left one grapheme (collapsing a selection to its start).
    pub(crate) fn move_left(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.start;
            return;
        }
        self.begin_move(select);
        self.cursor = self.prev_boundary(self.cursor);
    }

    /// Move right one grapheme (collapsing a selection to its end).
    pub(crate) fn move_right(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.cursor = r.end;
            return;
        }
        self.begin_move(select);
        self.cursor = self.next_boundary(self.cursor);
    }

    /// Move to the previous word start.
    pub(crate) fn move_word_left(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.prev_word(self.cursor);
    }

    /// Move to the next word end.
    pub(crate) fn move_word_right(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.next_word(self.cursor);
    }

    /// Move to the line start.
    pub(crate) fn move_home(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_start(self.cursor);
    }

    /// Move to the line end.
    pub(crate) fn move_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.line_end(self.cursor);
    }

    /// Move to the document start.
    pub(crate) fn move_doc_start(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = 0;
    }

    /// Move to the document end.
    pub(crate) fn move_doc_end(&mut self, select: bool) {
        self.begin_move(select);
        self.cursor = self.text.len();
    }

    /// Move to the same display column on the previous line.
    pub(crate) fn move_up(&mut self, select: bool) -> bool {
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

    /// Move to the same display column on the next line.
    pub(crate) fn move_down(&mut self, select: bool) -> bool {
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

    /// Place the cursor at `(line, col)`, dropping the selection.
    pub(crate) fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.anchor = None;
        self.cursor = self.offset_at(line, col);
    }

    fn delete_selection(&mut self) -> bool {
        if let Some(r) = self.selection() {
            self.replace_range(r.clone(), "");
            self.cursor = r.start;
            self.anchor = None;
            true
        } else {
            self.anchor = None;
            false
        }
    }

    /// Replace a range by rebuilding the allocation before releasing the old
    /// one. `String::replace_range` may leave deleted secret bytes in its
    /// allocation, so every deletion and selection replacement goes through
    /// this path.
    fn replace_range(&mut self, range: Range<usize>, replacement: &str) {
        let retained = range.end.saturating_sub(range.start);
        let capacity = self
            .text
            .len()
            .saturating_sub(retained)
            .saturating_add(replacement.len());
        let mut next = String::with_capacity(capacity);
        next.push_str(&self.text[..range.start]);
        next.push_str(replacement);
        next.push_str(&self.text[range.end..]);
        crate::secret::zeroize_string(&mut self.text);
        self.text = next;
    }

    /// Insert a character (a newline is rejected in single-line mode).
    /// Returns whether the text changed.
    pub(crate) fn insert_char(&mut self, c: char) -> bool {
        if c == '\n' && !self.multiline {
            return false;
        }
        self.delete_selection();
        self.reserve_for_insert(c.len_utf8());
        self.text.insert(self.cursor, c);
        self.cursor = self.cursor.saturating_add(c.len_utf8());
        true
    }

    /// Insert text (newlines are stripped in single-line mode).
    pub(crate) fn insert_str(&mut self, s: &str) -> bool {
        self.delete_selection();
        let before = self.text.len();
        if self.multiline {
            self.reserve_for_insert(s.len());
            self.text.insert_str(self.cursor, s);
        } else {
            let additional = s
                .chars()
                .filter(|c| *c != '\n' && *c != '\r')
                .map(char::len_utf8)
                .sum();
            self.reserve_for_insert(additional);
            let mut at = self.cursor;
            for c in s.chars().filter(|c| *c != '\n' && *c != '\r') {
                self.text.insert(at, c);
                at = at.saturating_add(c.len_utf8());
            }
        }
        let grown = self.text.len().saturating_sub(before);
        self.cursor = self.cursor.saturating_add(grown);
        grown > 0
    }

    /// Delete the grapheme before the cursor (or the selection).
    pub(crate) fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.prev_boundary(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.replace_range(start..self.cursor, "");
        self.cursor = start;
        true
    }

    /// Delete the grapheme after the cursor (or the selection).
    pub(crate) fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = self.next_boundary(self.cursor);
        if end == self.cursor {
            return false;
        }
        self.replace_range(self.cursor..end, "");
        true
    }

    /// Delete to the previous word start (or the selection).
    pub(crate) fn delete_word_left(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.prev_word(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.replace_range(start..self.cursor, "");
        self.cursor = start;
        true
    }

    /// Delete to the line end (or the selection).
    pub(crate) fn delete_to_line_end(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = self.line_end(self.cursor);
        if end == self.cursor {
            return false;
        }
        self.replace_range(self.cursor..end, "");
        true
    }

    /// Delete to the line start (or the selection).
    pub(crate) fn delete_to_line_start(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.line_start(self.cursor);
        if start == self.cursor {
            return false;
        }
        self.replace_range(start..self.cursor, "");
        self.cursor = start;
        true
    }

    /// The line count (one more than the newline count).
    pub(crate) fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    /// The cursor as `(line, display column)`.
    pub(crate) fn cursor_pos(&self) -> CursorPos {
        Self::pos_of(&self.text, self.cursor)
    }

    /// `(line, display column)` of a byte offset in `text`.
    #[cfg_attr(
        not(feature = "testing"),
        expect(
            unreachable_pub,
            reason = "the coordinate facade is re-exported only for testing"
        )
    )]
    pub fn pos_of(text: &str, offset: usize) -> CursorPos {
        let before = text.get(..offset.min(text.len())).unwrap_or("");
        let line = before.matches('\n').count();
        let line_start = before.rfind('\n').map_or(0, |i| i.saturating_add(1));
        let col = usize::from(width(before.get(line_start..).unwrap_or("")));
        CursorPos { line, col }
    }

    /// Byte offset of `(line, display column)`, clamped to the line.
    #[cfg_attr(
        not(feature = "testing"),
        expect(
            unreachable_pub,
            reason = "the coordinate facade is re-exported only for testing"
        )
    )]
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

    /// Display width of the whole text (single-line).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "retained for text-core unit coverage")
    )]
    pub(crate) fn width(&self) -> u16 {
        width(&self.text)
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
        let mut z = TextBuffer::single("e\u{301}!");
        z.move_end(false);
        z.move_left(false);
        z.move_left(false);
        assert_eq!(z.cursor_offset(), 0);
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
        assert!(b.selection().is_none());
    }

    #[test]
    fn word_motion_and_deletion() {
        let mut b = TextBuffer::single("alpha beta  gamma");
        b.move_word_left(false);
        assert_eq!(b.cursor_offset(), 12);
        b.move_word_left(false);
        assert_eq!(b.cursor_offset(), 6);
        b.move_home(false);
        b.move_word_right(false);
        assert_eq!(b.cursor_offset(), 5);
        b.move_end(false);
        b.delete_word_left();
        assert_eq!(b.text(), "alpha beta  ");
    }

    #[test]
    fn word_chars_are_consistent_between_buffer_and_viewport() {
        // one definition: `text::is_word_char`; `_` joins a word, `-` splits
        let mut b = TextBuffer::single("snake_case-kebab");
        b.move_home(false);
        b.move_word_right(false);
        assert_eq!(b.cursor_offset(), "snake_case".len());
        assert!(is_word_char('_') && !is_word_char('-'));
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
        assert_eq!(b.selection_lines(), (0, 0));
    }

    #[test]
    fn single_line_rejects_newline() {
        let mut b = TextBuffer::single("a");
        assert!(!b.insert_char('\n'));
        assert_eq!(b.text(), "a");
        b.insert_str("b\nc");
        assert_eq!(b.text(), "abc");
        let mut m = TextBuffer::multi("a");
        assert!(m.insert_char('\n'));
        assert_eq!(m.line_count(), 2);
        assert!(m.move_up(false));
        assert!(!m.move_up(false));
    }

    #[test]
    fn wide_characters_count_as_two_columns() {
        let b = TextBuffer::single("日本");
        assert_eq!(b.cursor_pos().col, 4);
        assert_eq!(b.offset_at(0, 2), 3);
        assert_eq!(b.offset_at(0, 1), 0);
        assert_eq!(b.width(), 4);
    }

    #[test]
    fn pos_of_and_offset_at_round_trip() {
        let text = "ab\n日本語\ne\u{301}xyz";
        let b = TextBuffer::multi(text);
        for (off, _) in text.char_indices() {
            let p = TextBuffer::pos_of(text, off);
            let back = b.offset_at(p.line, p.col);
            let snapped = TextBuffer::pos_of(text, back);
            assert_eq!(snapped, p, "offset {off}");
        }
        assert_eq!(b.offset_at(99, 0), text.len());
    }

    #[test]
    fn zeroize_overwrites_before_drop() {
        let mut b = TextBuffer::single("hunter2");
        b.select_all();
        b.zeroize();
        assert!(b.is_empty());
        assert_eq!(b.cursor_offset(), 0);
        assert!(b.selection().is_none());
        assert!(!format!("{:?}", TextBuffer::single("hunter2")).contains("hunter2"));
        // Drop runs zeroize: the same path, exercised through `set_text`
        b.set_text("again");
        assert_eq!(b.text(), "again");
    }

    fn forced_capacity(text: &str) -> TextBuffer {
        let mut owned = String::with_capacity(128);
        owned.push_str(text);
        TextBuffer::single(owned)
    }

    fn forced_multiline_capacity(text: &str) -> TextBuffer {
        let mut owned = String::with_capacity(128);
        owned.push_str(text);
        TextBuffer::multi(owned)
    }

    fn assert_rebuilt(
        mut buffer: TextBuffer,
        expected: &str,
        mutate: impl FnOnce(&mut TextBuffer) -> bool,
    ) {
        let old_capacity = buffer.text.capacity();
        assert!(mutate(&mut buffer));
        assert_eq!(buffer.text(), expected);
        assert!(
            buffer.text.capacity() < old_capacity,
            "mutation retained old capacity: {} >= {old_capacity}",
            buffer.text.capacity()
        );
    }

    #[test]
    fn deletion_and_selection_replacement_release_old_capacity() {
        assert_rebuilt(forced_capacity("hunter2"), "hunter", TextBuffer::backspace);
        assert_rebuilt(forced_capacity("hunter2"), "unter2", |b| {
            b.move_doc_start(false);
            b.delete()
        });
        assert_rebuilt(forced_capacity("hunter2"), "", |b| {
            b.select_all();
            b.delete_selection()
        });
        assert_rebuilt(forced_capacity("hunter2"), "x", |b| {
            b.select_all();
            b.insert_str("x")
        });
        assert_rebuilt(forced_capacity("hunter2"), "", TextBuffer::delete_word_left);
        assert_rebuilt(forced_capacity("hunter2"), "x", |b| {
            b.set_text("x");
            true
        });
    }

    #[test]
    fn multiline_deletions_release_old_capacity() {
        assert_rebuilt(
            forced_multiline_capacity("first\nsecret"),
            "\nsecret",
            |b| {
                b.move_doc_start(false);
                b.delete_to_line_end()
            },
        );
        assert_rebuilt(
            forced_multiline_capacity("first\nsecret"),
            "first\ncret",
            |b| {
                b.set_cursor_line_col(1, 2);
                b.delete_to_line_start()
            },
        );
    }

    #[test]
    fn zeroize_releases_capacity_after_deleted_text() {
        let mut b = TextBuffer::single("hunter2");
        b.set_text("x");
        b.zeroize();
        assert_eq!(b.text(), "");
        assert_eq!(b.text.capacity(), 0);
    }

    #[test]
    fn insertion_growth_uses_a_wiped_replacement_allocation() {
        let mut b = TextBuffer::single("a");
        let old_capacity = b.text.capacity();
        b.insert_str(&"x".repeat(old_capacity.saturating_add(1)));
        assert!(b.text.len() > old_capacity);
        assert_eq!(b.text.chars().next(), Some('a'));
    }
}
