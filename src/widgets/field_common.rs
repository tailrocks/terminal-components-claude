//! Key bindings shared by every text-editing control.

use ratatui::crossterm::event::KeyCode;

use crate::core::event::Key;
use crate::core::text::TextBuffer;

/// Pointer text selection in a text field: what a drag, a double click
/// and a triple click select. Shared by `TextInput` and `TextArea`.
/// [`PointerSelect::DEFAULT`] follows macOS (all on); the click counts come
/// from [`crate::core::pointer::PointerState::count_click`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerSelect {
    /// A drag from the press selects the text it crosses.
    pub drag: bool,
    /// A double click selects the word under the pointer.
    pub word: bool,
    /// A triple click selects the line (the whole value of a single-line
    /// field).
    pub line: bool,
}

impl PointerSelect {
    pub const DEFAULT: Self = Self {
        drag: true,
        word: true,
        line: true,
    };
    /// Clicks only place the caret; nothing selects by pointer.
    pub const OFF: Self = Self {
        drag: false,
        word: false,
        line: false,
    };
}

impl Default for PointerSelect {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Pointer gesture bookkeeping shared by the text fields: where the press
/// landed and whether a drag selected from it since.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PressState {
    pub at: Option<usize>,
    pub dragged: bool,
}

/// Apply a click with `count` (1 single, 2 double, 3+ triple) at byte
/// `offset` to `buffer` per `sel`. A release that ends a drag keeps the
/// dragged selection.
pub(crate) fn click_select(
    buffer: &mut TextBuffer,
    press: &mut PressState,
    sel: PointerSelect,
    offset: usize,
    count: u8,
    whole_word: bool,
) {
    let dragged = std::mem::take(&mut press.dragged);
    press.at = None;
    if dragged {
        return;
    }
    if count >= 3 && sel.line {
        buffer.select_line_at(offset);
    } else if count == 2 && sel.word {
        if whole_word {
            buffer.select_all();
        } else {
            buffer.select_word_at(offset);
        }
    } else {
        buffer.select_range(offset, offset);
        buffer.clear_selection();
    }
}

/// A drag to byte `offset`: extend the selection from the press.
pub(crate) fn drag_select(buffer: &mut TextBuffer, press: &mut PressState, offset: usize) {
    let anchor = press.at.unwrap_or_else(|| buffer.cursor_offset());
    press.at = Some(anchor);
    press.dragged = true;
    buffer.select_range(anchor, offset);
}

pub enum EditAction {
    Commit,
    Cancel,
    Tab { backward: bool },
    Apply(fn(&mut TextBuffer)),
    Insert(char),
    None,
}

/// Translate a key into an edit action. `multiline` changes Enter to insert a
/// newline and makes Esc commit instead of cancel (a text area is a document,
/// not a value).
pub fn edit_key(key: &Key, multiline: bool) -> EditAction {
    let shift = key.shift();
    match key.code {
        KeyCode::Esc => EditAction::Cancel,
        KeyCode::Enter if multiline && key.plain() => EditAction::Apply(|b| b.insert_char('\n')),
        KeyCode::Enter => EditAction::Commit,
        KeyCode::Tab => EditAction::Tab { backward: false },
        KeyCode::BackTab => EditAction::Tab { backward: true },
        KeyCode::Left if key.ctrl() || key.alt() => {
            if shift {
                EditAction::Apply(|b| b.move_word_left(true))
            } else {
                EditAction::Apply(|b| b.move_word_left(false))
            }
        }
        KeyCode::Right if key.ctrl() || key.alt() => {
            if shift {
                EditAction::Apply(|b| b.move_word_right(true))
            } else {
                EditAction::Apply(|b| b.move_word_right(false))
            }
        }
        KeyCode::Left => {
            if shift {
                EditAction::Apply(|b| b.move_left(true))
            } else {
                EditAction::Apply(|b| b.move_left(false))
            }
        }
        KeyCode::Right => {
            if shift {
                EditAction::Apply(|b| b.move_right(true))
            } else {
                EditAction::Apply(|b| b.move_right(false))
            }
        }
        KeyCode::Up if multiline => {
            if shift {
                EditAction::Apply(|b| {
                    b.move_up(true);
                })
            } else {
                EditAction::Apply(|b| {
                    b.move_up(false);
                })
            }
        }
        KeyCode::Down if multiline => {
            if shift {
                EditAction::Apply(|b| {
                    b.move_down(true);
                })
            } else {
                EditAction::Apply(|b| {
                    b.move_down(false);
                })
            }
        }
        KeyCode::Home if key.ctrl() => EditAction::Apply(|b| b.move_doc_start(false)),
        KeyCode::End if key.ctrl() => EditAction::Apply(|b| b.move_doc_end(false)),
        KeyCode::Home => {
            if shift {
                EditAction::Apply(|b| b.move_home(true))
            } else {
                EditAction::Apply(|b| b.move_home(false))
            }
        }
        KeyCode::End => {
            if shift {
                EditAction::Apply(|b| b.move_end(true))
            } else {
                EditAction::Apply(|b| b.move_end(false))
            }
        }
        KeyCode::Backspace if key.ctrl() || key.alt() => {
            EditAction::Apply(|b| b.delete_word_left())
        }
        KeyCode::Backspace => EditAction::Apply(|b| b.backspace()),
        KeyCode::Delete => EditAction::Apply(|b| b.delete()),
        KeyCode::Char('a') if key.ctrl() => EditAction::Apply(|b| b.move_home(false)),
        KeyCode::Char('e') if key.ctrl() => EditAction::Apply(|b| b.move_end(false)),
        KeyCode::Char('u') if key.ctrl() => EditAction::Apply(|b| b.delete_to_line_start()),
        KeyCode::Char('k') if key.ctrl() => EditAction::Apply(|b| b.delete_to_line_end()),
        KeyCode::Char('w') if key.ctrl() => EditAction::Apply(|b| b.delete_word_left()),
        KeyCode::Char('l') if key.ctrl() => EditAction::Apply(|b| b.select_all()),
        KeyCode::Char('b') if key.alt() => EditAction::Apply(|b| b.move_word_left(false)),
        KeyCode::Char('f') if key.alt() => EditAction::Apply(|b| b.move_word_right(false)),
        KeyCode::Char(_) if key.ctrl() || key.alt() => EditAction::None,
        KeyCode::Char(c) => EditAction::Insert(c),
        _ => EditAction::None,
    }
}
