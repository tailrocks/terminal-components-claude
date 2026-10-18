#![doc = r" The one editing core (`COMPONENT_ARCHITECTURE.md` §15)."]
#![doc = r""]
#![doc = r" Shared by inputs, text areas, code editors, editable cells and picker"]
#![doc = r" queries. [`TextEditorCore::apply`] is the only mutation entry point, so a"]
#![doc = r" binding table over [`EditAction`] is the whole key handling of a text"]
#![doc = r" control."]

use core::fmt;
use core::ops::Range;

use super::buffer::{CursorPos, TextBuffer, normalized_text};

#[doc = r" A cursor motion."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    #[doc = r" One grapheme left."]
    Left,
    #[doc = r" One grapheme right."]
    Right,
    #[doc = r" One word left."]
    WordLeft,
    #[doc = r" One word right."]
    WordRight,
    #[doc = r" Line start."]
    Home,
    #[doc = r" Line end."]
    End,
    #[doc = r" Document start."]
    DocStart,
    #[doc = r" Document end."]
    DocEnd,
    #[doc = r" Previous line, same column."]
    Up,
    #[doc = r" Next line, same column."]
    Down,
}

#[doc = r" Whether a motion extends the selection."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extend {
    #[doc = r" Move the cursor, dropping any selection."]
    No,
    #[doc = r" Extend the selection from its anchor."]
    Select,
}

#[doc = r" An edit command; `const`-constructible so binding tables can hold it."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditAction<'a> {
    #[doc = r" Insert a character at the cursor."]
    Insert(char),
    #[doc = r" Insert text at the cursor (paste)."]
    Paste(&'a str),
    #[doc = r" Replace the whole text."]
    SetText(&'a str),
    #[doc = r" Move the cursor."]
    Move(Motion, Extend),
    #[doc = r" Delete the grapheme before the cursor."]
    Backspace,
    #[doc = r" Delete the grapheme after the cursor."]
    Delete,
    #[doc = r" Delete the word before the cursor."]
    DeleteWordLeft,
    #[doc = r" Delete to the line end."]
    DeleteToLineEnd,
    #[doc = r" Delete to the line start."]
    DeleteToLineStart,
    #[doc = r" Select everything."]
    SelectAll,
    #[doc = r" Drop the selection."]
    ClearSelection,
    #[doc = r" Insert a newline (multi-line only)."]
    Newline,
}

#[doc = r" What an edit did."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOutcome {
    #[doc = r" The text changed."]
    Changed,
    #[doc = r" Only the cursor or selection moved."]
    Moved,
    #[doc = r" Nothing happened (a no-op at a boundary)."]
    Ignored,
    #[doc = r" The action is not allowed here (a newline in single-line mode)."]
    Rejected,
}

impl EditOutcome {
    #[doc = r" Whether the text changed."]
    pub const fn changed(self) -> bool {
        matches!(self, EditOutcome::Changed)
    }

    #[doc = r" Whether anything visible changed."]
    pub const fn is_visible(self) -> bool {
        matches!(self, EditOutcome::Changed | EditOutcome::Moved)
    }
}

#[doc = r" Buffer, cursor, selection, horizontal scroll and the multi-line flag."]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct TextEditorCore {
    buf: TextBuffer,
    hscroll: u16,
}

impl fmt::Debug for TextEditorCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextEditorCore")
            .field("buf", &self.buf)
            .field("hscroll", &self.hscroll)
            .finish()
    }
}

impl TextEditorCore {
    #[doc = r" A single-line editor."]
    pub fn single(text: &str) -> Self {
        TextEditorCore {
            buf: TextBuffer::single(text),
            hscroll: 0,
        }
    }

    pub fn sensitive_single(text: &str) -> Self {
        TextEditorCore {
            buf: TextBuffer::sensitive_single(text),
            hscroll: 0,
        }
    }

    #[doc = r" A multi-line editor."]
    pub fn multi(text: &str) -> Self {
        TextEditorCore {
            buf: TextBuffer::multi(text),
            hscroll: 0,
        }
    }

    pub fn sensitive_multi(text: &str) -> Self {
        TextEditorCore {
            buf: TextBuffer::sensitive_multi(text),
            hscroll: 0,
        }
    }

    #[doc = r" The text."]
    pub fn text(&self) -> &str {
        self.buf.text()
    }

    #[doc = r" Whether the text is empty."]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[doc = r" Whether newlines are accepted."]
    pub const fn is_multiline(&self) -> bool {
        self.buf.is_multiline()
    }

    #[doc = r" The selection as a byte range."]
    pub fn selection(&self) -> Option<Range<usize>> {
        self.buf.selection()
    }

    #[doc = r" The selected text."]
    pub fn selected_text(&self) -> Option<&str> {
        self.buf.selected_text()
    }

    #[doc = r" The cursor as `(line, display column)`."]
    pub fn cursor_pos(&self) -> CursorPos {
        self.buf.cursor_pos()
    }

    #[doc = r" The cursor byte offset."]
    pub const fn cursor_offset(&self) -> usize {
        self.buf.cursor_offset()
    }

    #[doc = r" The horizontal scroll, in columns."]
    pub const fn hscroll(&self) -> u16 {
        self.hscroll
    }

    #[doc = r" Keep the cursor visible inside a viewport `width` columns wide;"]
    #[doc = r" returns the resulting horizontal scroll."]
    pub fn scroll_into_view(&mut self, width: u16) -> u16 {
        let col = self.buf.cursor_pos().col.min(usize::from(u16::MAX)) as u16;
        if width == 0 || col < self.hscroll {
            self.hscroll = col;
        } else if col >= self.hscroll.saturating_add(width) {
            self.hscroll = col.saturating_add(1).saturating_sub(width);
        }
        self.hscroll
    }

    #[doc = r" The buffer's line count."]
    pub fn line_count(&self) -> usize {
        self.buf.line_count()
    }

    #[doc = r" Byte offset of `(line, display column)`."]
    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        self.buf.offset_at(line, col)
    }

    #[doc = r" Place the cursor at `(line, col)`."]
    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.buf.set_cursor_line_col(line, col);
    }

    #[doc = r" Select `a..b` (either order), cursor at `b`."]
    pub fn select_range(&mut self, a: usize, b: usize) {
        self.buf.select_range(a, b);
    }

    #[doc = r" Overwrite bytes before drop (§15)."]
    pub fn zeroize(&mut self) {
        self.buf.zeroize();
        self.hscroll = 0;
    }

    #[doc = r" The only mutation entry point."]
    pub fn apply(&mut self, a: EditAction<'_>) -> EditOutcome {
        let b = &mut self.buf;
        let changed = |yes: bool| {
            if yes {
                EditOutcome::Changed
            } else {
                EditOutcome::Ignored
            }
        };
        match a {
            EditAction::Insert('\n') | EditAction::Newline => {
                if b.is_multiline() {
                    changed(b.insert_char('\n'))
                } else {
                    EditOutcome::Rejected
                }
            }
            EditAction::Insert(c) => {
                if c.is_control() {
                    return EditOutcome::Rejected;
                }
                changed(b.insert_char(c))
            }
            EditAction::Paste(s) => changed(b.insert_str(s)),
            EditAction::SetText(s) => {
                if b.text() == normalized_text(s, b.is_multiline()) {
                    return EditOutcome::Ignored;
                }
                b.set_text(s);
                EditOutcome::Changed
            }
            EditAction::Move(m, ext) => {
                let before = (b.cursor_offset(), b.selection());
                let select = ext == Extend::Select;
                match m {
                    Motion::Left => b.move_left(select),
                    Motion::Right => b.move_right(select),
                    Motion::WordLeft => b.move_word_left(select),
                    Motion::WordRight => b.move_word_right(select),
                    Motion::Home => b.move_home(select),
                    Motion::End => b.move_end(select),
                    Motion::DocStart => b.move_doc_start(select),
                    Motion::DocEnd => b.move_doc_end(select),
                    Motion::Up => {
                        if !b.move_up(select) {
                            return EditOutcome::Ignored;
                        }
                    }
                    Motion::Down => {
                        if !b.move_down(select) {
                            return EditOutcome::Ignored;
                        }
                    }
                }
                if (b.cursor_offset(), b.selection()) == before {
                    EditOutcome::Ignored
                } else {
                    EditOutcome::Moved
                }
            }
            EditAction::Backspace => changed(b.backspace()),
            EditAction::Delete => changed(b.delete()),
            EditAction::DeleteWordLeft => changed(b.delete_word_left()),
            EditAction::DeleteToLineEnd => changed(b.delete_to_line_end()),
            EditAction::DeleteToLineStart => changed(b.delete_to_line_start()),
            EditAction::SelectAll => {
                if b.is_empty() {
                    return EditOutcome::Ignored;
                }
                b.select_all();
                EditOutcome::Moved
            }
            EditAction::ClearSelection => {
                if b.selection().is_none() {
                    return EditOutcome::Ignored;
                }
                b.clear_selection();
                EditOutcome::Moved
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_apply_is_the_only_mutation_entry_point() {
        let mut e = TextEditorCore::single("ab");
        assert_eq!(e.apply(EditAction::Insert('c')), EditOutcome::Changed);
        assert_eq!(e.text(), "abc");
        assert_eq!(
            e.apply(EditAction::Move(Motion::Home, Extend::No)),
            EditOutcome::Moved
        );
        assert_eq!(
            e.apply(EditAction::Move(Motion::Left, Extend::No)),
            EditOutcome::Ignored
        );
        assert_eq!(e.apply(EditAction::Newline), EditOutcome::Rejected);
        assert_eq!(e.apply(EditAction::Insert('\u{7}')), EditOutcome::Rejected);
        assert_eq!(
            e.apply(EditAction::Move(Motion::End, Extend::Select)),
            EditOutcome::Moved
        );
        assert_eq!(e.selected_text(), Some("abc"));
        assert_eq!(e.apply(EditAction::Paste("z")), EditOutcome::Changed);
        assert_eq!(e.text(), "z");
        assert_eq!(e.apply(EditAction::SetText("z")), EditOutcome::Ignored);
        assert_eq!(
            e.apply(EditAction::SetText("long text")),
            EditOutcome::Changed
        );
        assert_eq!(e.apply(EditAction::SelectAll), EditOutcome::Moved);
        assert_eq!(e.apply(EditAction::ClearSelection), EditOutcome::Moved);
        assert_eq!(e.apply(EditAction::ClearSelection), EditOutcome::Ignored);
        assert_eq!(e.apply(EditAction::Backspace), EditOutcome::Changed);
        assert_eq!(e.apply(EditAction::DeleteToLineStart), EditOutcome::Changed);
        assert!(e.is_empty());
        assert_eq!(e.apply(EditAction::Delete), EditOutcome::Ignored);
        let mut m = TextEditorCore::multi("a\nb");
        assert_eq!(
            m.apply(EditAction::Move(Motion::Up, Extend::No)),
            EditOutcome::Moved
        );
        assert_eq!(
            m.apply(EditAction::Move(Motion::Up, Extend::No)),
            EditOutcome::Ignored
        );
        assert_eq!(m.apply(EditAction::Newline), EditOutcome::Changed);
        assert_eq!(m.line_count(), 3);
    }

    #[test]
    fn hscroll_follows_the_cursor() {
        let mut e = TextEditorCore::single("0123456789");
        assert_eq!(e.scroll_into_view(4), 7);
        e.apply(EditAction::Move(Motion::Home, Extend::No));
        assert_eq!(e.scroll_into_view(4), 0);
        assert_eq!(e.scroll_into_view(0), 0);
        e.zeroize();
        assert!(e.is_empty() && e.hscroll() == 0);
        assert!(!format!("{e:?}").contains("0123"));
    }

    #[test]
    fn set_text_compares_normalized_content() {
        let mut editor = TextEditorCore::single("ab");
        assert_eq!(
            editor.apply(EditAction::SetText("a\r\nb")),
            EditOutcome::Ignored
        );
        assert_eq!(editor.text(), "ab");
        let mut multi = TextEditorCore::multi("a\nb");
        assert_eq!(
            multi.apply(EditAction::SetText("a\r\nb")),
            EditOutcome::Ignored
        );
        assert_eq!(multi.text(), "a\nb");
    }
}
