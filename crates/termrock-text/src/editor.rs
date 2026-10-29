use core::fmt;
use core::ops::Range;

use super::buffer::{CursorPos, TextBuffer, normalized_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    Home,
    End,
    DocStart,
    DocEnd,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extend {
    No,
    Select,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditAction<'a> {
    Insert(char),
    Paste(&'a str),
    SetText(&'a str),
    Move(Motion, Extend),
    Backspace,
    Delete,
    DeleteWordLeft,
    DeleteToLineEnd,
    DeleteToLineStart,
    SelectAll,
    ClearSelection,
    Newline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOutcome {
    Changed,
    Moved,
    Ignored,
    Rejected,
}

impl EditOutcome {
    pub const fn changed(self) -> bool {
        matches!(self, EditOutcome::Changed)
    }

    pub const fn is_visible(self) -> bool {
        matches!(self, EditOutcome::Changed | EditOutcome::Moved)
    }
}

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

    pub fn text(&self) -> &str {
        self.buf.text()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub const fn is_multiline(&self) -> bool {
        self.buf.is_multiline()
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.buf.selection()
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.buf.selected_text()
    }

    pub fn cursor_pos(&self) -> CursorPos {
        self.buf.cursor_pos()
    }

    pub const fn cursor_offset(&self) -> usize {
        self.buf.cursor_offset()
    }

    pub const fn hscroll(&self) -> u16 {
        self.hscroll
    }

    pub fn scroll_into_view(&mut self, width: u16) -> u16 {
        let col = self.buf.cursor_pos().col.min(usize::from(u16::MAX)) as u16;
        if width == 0 || col < self.hscroll {
            self.hscroll = col;
        } else if col >= self.hscroll.saturating_add(width) {
            self.hscroll = col.saturating_add(1).saturating_sub(width);
        }
        self.hscroll
    }

    /// Keep the cursor column from scrolling past the last visible glyph.
    pub fn clamp_hscroll(&mut self, max: u16) {
        if self.hscroll > max {
            self.hscroll = max;
        }
    }

    pub fn line_count(&self) -> usize {
        self.buf.line_count()
    }

    pub fn offset_at(&self, line: usize, col: usize) -> usize {
        self.buf.offset_at(line, col)
    }

    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.buf.set_cursor_line_col(line, col);
    }

    pub fn select_range(&mut self, a: usize, b: usize) {
        self.buf.select_range(a, b);
    }

    pub fn select_word_at(&mut self, offset: usize) -> EditOutcome {
        let before = (self.selection(), self.cursor_offset());
        self.buf.select_word_at(offset);
        if (self.selection(), self.cursor_offset()) == before {
            EditOutcome::Ignored
        } else {
            EditOutcome::Moved
        }
    }

    pub fn select_line_at(&mut self, offset: usize) -> EditOutcome {
        let before = (self.selection(), self.cursor_offset());
        self.buf.select_line_at(offset);
        if (self.selection(), self.cursor_offset()) == before {
            EditOutcome::Ignored
        } else {
            EditOutcome::Moved
        }
    }

    pub fn zeroize(&mut self) {
        self.buf.zeroize();
        self.hscroll = 0;
    }

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

    #[test]
    fn word_and_line_selection_use_grapheme_boundaries() {
        let mut editor = TextEditorCore::multi("éclair cafe\u{301} !\nnext");
        assert_eq!(editor.select_word_at(2), EditOutcome::Moved);
        assert_eq!(editor.selected_text(), Some("éclair"));
        assert_eq!(
            editor.select_word_at("éclair ".len() + 2),
            EditOutcome::Moved
        );
        assert_eq!(editor.selected_text(), Some("cafe\u{301}"));
        assert_eq!(
            editor.select_word_at("éclair cafe\u{301} ".len()),
            EditOutcome::Moved
        );
        assert_eq!(editor.selected_text(), Some("!"));
        assert_eq!(
            editor.select_line_at("éclair cafe\u{301} !\nnext".len()),
            EditOutcome::Moved
        );
        assert_eq!(editor.selected_text(), Some("next"));
    }

    #[test]
    fn sensitive_word_selection_stays_redacted() {
        let mut editor = TextEditorCore::sensitive_single("hunter2");
        assert_eq!(editor.select_word_at(2), EditOutcome::Moved);
        assert_eq!(editor.selected_text(), Some("hunter2"));
        assert!(!format!("{editor:?}").contains("hunter2"));
    }
}
