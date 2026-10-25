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
