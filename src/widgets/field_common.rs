//! Key bindings shared by every text-editing control.

use std::ops::Range;

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use crate::core::edit::{EditKind, FieldKeys, TextCmd, UndoStack, clipboard};
use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{KeyBinding, lookup};
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
    /// A drag that starts on the second press of a double click extends
    /// the selection by whole words (macOS); off, it extends by
    /// characters from the press.
    pub drag_by_word: bool,
}

impl PointerSelect {
    pub const DEFAULT: Self = Self {
        drag: true,
        word: true,
        line: true,
        drag_by_word: true,
    };
    /// Clicks only place the caret; nothing selects by pointer.
    pub const OFF: Self = Self {
        drag: false,
        word: false,
        line: false,
        drag_by_word: false,
    };

    pub const fn drag(mut self, on: bool) -> Self {
        self.drag = on;
        self
    }
    pub const fn word(mut self, on: bool) -> Self {
        self.word = on;
        self
    }
    pub const fn line(mut self, on: bool) -> Self {
        self.line = on;
        self
    }
    pub const fn drag_by_word(mut self, on: bool) -> Self {
        self.drag_by_word = on;
        self
    }
}

impl Default for PointerSelect {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Pointer gesture bookkeeping shared by the text fields: where the press
/// landed and whether a drag selected from it since.
#[derive(Debug, Clone, Default)]
pub(crate) struct PressState {
    pub at: Option<usize>,
    pub dragged: bool,
    /// The word the second press of a double click selected: a drag from
    /// it extends by words ([`PointerSelect::drag_by_word`]).
    pub word: Option<Range<usize>>,
}

/// A press with its multi-click `count` at byte `offset`: remember it for
/// the drag; the second press of a double click selects the word there
/// and a drag from it extends by words (per `sel`). `true` when the
/// selection changed.
pub(crate) fn press_select(
    buffer: &mut TextBuffer,
    press: &mut PressState,
    sel: PointerSelect,
    offset: usize,
    count: u8,
    whole_word: bool,
) -> bool {
    *press = PressState {
        at: Some(offset),
        dragged: false,
        word: None,
    };
    if count == 2 && sel.word && sel.drag_by_word && !whole_word {
        let w = buffer.word_range_at(offset);
        buffer.select_range(w.start, w.end);
        press.word = Some(w);
        return true;
    }
    false
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
    press.word = None;
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
/// After a double-click press ([`press_select`]) it extends by whole
/// words: the pressed word stays selected and the far end snaps to the
/// word under the pointer.
pub(crate) fn drag_select(buffer: &mut TextBuffer, press: &mut PressState, offset: usize) {
    let anchor = press.at.unwrap_or_else(|| buffer.cursor_offset());
    press.at = Some(anchor);
    press.dragged = true;
    if let Some(w) = press.word.clone() {
        let to = buffer.word_range_at(offset);
        if to.start < w.start {
            buffer.select_range(w.end, to.start);
        } else {
            buffer.select_range(w.start, to.end.max(w.end));
        }
        return;
    }
    buffer.select_range(anchor, offset);
}

/// A text control the pointer selects in: a press, a drag from it and a
/// completed click with its count. `TextInput` and `TextArea` implement
/// it; owners route through [`route_press`], [`route_drag`] and
/// [`route_click`] instead of matching each field by hand.
pub trait PointerField {
    /// The hit id the field registers.
    fn pointer_id(&self) -> WidgetId;
    /// Button down with the click count it starts
    /// ([`crate::core::pointer::PointerState::press_count`]).
    fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome;
    /// Drag with the button held since a press on the field.
    fn on_drag(&mut self, pos: Position) -> Outcome;
    /// A completed click with its multi-click count
    /// ([`crate::core::pointer::PointerState::count_click`]).
    fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome;
}

/// A press on hit `id`: the field it belongs to takes it.
pub fn route_press(
    fields: &mut [&mut dyn PointerField],
    id: WidgetId,
    pos: Position,
    count: u8,
) -> Option<Outcome> {
    fields
        .iter_mut()
        .find(|f| f.pointer_id() == id)
        .map(|f| f.on_press_count(pos, count))
}

/// A drag held since a press on `pressed`: the field it belongs to
/// extends its selection.
pub fn route_drag(
    fields: &mut [&mut dyn PointerField],
    pressed: WidgetId,
    pos: Position,
) -> Option<Outcome> {
    fields
        .iter_mut()
        .find(|f| f.pointer_id() == pressed)
        .map(|f| f.on_drag(pos))
}

/// A completed click on hit `id` with its count.
pub fn route_click(
    fields: &mut [&mut dyn PointerField],
    id: WidgetId,
    pos: Position,
    count: u8,
) -> Option<Outcome> {
    fields
        .iter_mut()
        .find(|f| f.pointer_id() == id)
        .map(|f| f.on_click_count(pos, count.max(1)))
}

#[derive(Clone, Copy)]
pub enum EditAction {
    Commit,
    Cancel,
    Tab {
        backward: bool,
    },
    Apply(fn(&mut TextBuffer)),
    Insert(char),
    /// Step back one edit (Ctrl+Z).
    Undo,
    /// Re-apply an undone edit (Ctrl+Shift+Z; Ctrl+Y under
    /// [`FieldKeys::Desktop`]).
    Redo,
    /// Copy the selection to the field clipboard.
    Copy,
    /// Copy the selection, then delete it.
    Cut,
    /// Insert the field clipboard's text.
    Paste,
    /// Select the whole value ([`TextCmd::SelectAll`]): Cmd+A, plus Ctrl+A
    /// ([`FieldKeys::Desktop`]) or Ctrl+L ([`FieldKeys::Emacs`]).
    SelectAll,
    None,
}

/// Translate a key into an edit action under the client platform's
/// preset ([`FieldKeys::default`]). `multiline` changes Enter to insert a
/// newline.
pub fn edit_key(key: &Key, multiline: bool) -> EditAction {
    edit_key_with(key, multiline, FieldKeys::default())
}

/// Translate a key into an edit action under the `keys` preset and its
/// text commands ([`FieldKeys::text_keymap`]). `multiline` changes Enter
/// to insert a newline.
pub fn edit_key_with(key: &Key, multiline: bool, keys: FieldKeys) -> EditAction {
    edit_key_in(key, multiline, keys, &keys.text_keymap())
}

/// [`edit_key_with`] with the text commands bound by `text_keymap`
/// instead of the preset's (a widget's `text_keymap(..)` override). The
/// one place every text control resolves Select All.
pub fn edit_key_in(
    key: &Key,
    multiline: bool,
    keys: FieldKeys,
    text_keymap: &[(KeyBinding, TextCmd)],
) -> EditAction {
    if let Some(cmd) = lookup(text_keymap, key) {
        return match cmd {
            TextCmd::SelectAll => EditAction::SelectAll,
        };
    }
    let shift = key.shift();
    // the shared keys of both presets
    match key.code {
        KeyCode::Char('z') | KeyCode::Char('Z') if key.ctrl() => {
            return if shift || key.code == KeyCode::Char('Z') {
                EditAction::Redo
            } else {
                EditAction::Undo
            };
        }
        // Ctrl+Backspace often arrives as Ctrl+H (readline: delete left)
        KeyCode::Char('h') if key.ctrl() => return EditAction::Apply(|b| b.backspace()),
        _ => {}
    }
    if keys == FieldKeys::Desktop && key.ctrl() && !key.alt() {
        match key.code {
            // Select All comes from the text keymap; unbound there, Ctrl+A
            // does nothing rather than turn into the Cocoa line start
            KeyCode::Char('a') => return EditAction::None,
            KeyCode::Char('c') => return EditAction::Copy,
            KeyCode::Char('x') => return EditAction::Cut,
            KeyCode::Char('v') => return EditAction::Paste,
            KeyCode::Char('y') => return EditAction::Redo,
            _ => {}
        }
    }
    if keys == FieldKeys::Emacs {
        match key.code {
            KeyCode::Char('y') if key.ctrl() => return EditAction::Paste,
            KeyCode::Char('w') if key.alt() => return EditAction::Copy,
            _ => {}
        }
    }
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
        // Ctrl+W cuts a selection (Emacs kill-region), else the word left
        KeyCode::Char('w') if key.ctrl() => EditAction::Cut,
        KeyCode::Char('b') if key.alt() => EditAction::Apply(|b| b.move_word_left(false)),
        KeyCode::Char('f') if key.alt() => EditAction::Apply(|b| b.move_word_right(false)),
        KeyCode::Char(_) if key.chord() => EditAction::None,
        KeyCode::Char(c) => EditAction::Insert(c),
        _ => EditAction::None,
    }
}

/// What [`apply_edit`] did to the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// The text changed (and the change is on the undo history).
    Edited,
    /// Only the caret or the selection moved (or the selection was
    /// copied).
    Moved,
}

/// Apply the buffer-level actions (typing, edits, undo, redo, copy, cut,
/// paste) to `buf`, recording every text change on `undo`. Returns `None`
/// for the actions the control decides itself (Commit, Cancel, Tab and
/// None). A `read_only` buffer only moves, selects and copies.
pub fn apply_edit(
    buf: &mut TextBuffer,
    undo: &mut UndoStack,
    action: EditAction,
    key_code: KeyCode,
    read_only: bool,
) -> Option<Applied> {
    let before = buf.clone();
    let kind = match (action, key_code) {
        (EditAction::Insert(c), _) if c.is_whitespace() => EditKind::Other,
        (EditAction::Insert(_), _) => EditKind::Typing,
        (EditAction::Apply(_), KeyCode::Backspace | KeyCode::Delete) => EditKind::Deleting,
        _ => EditKind::Other,
    };
    match action {
        EditAction::Commit | EditAction::Cancel | EditAction::Tab { .. } | EditAction::None => {
            return None;
        }
        EditAction::Undo if !read_only => {
            return Some(if undo.undo(buf) {
                Applied::Edited
            } else {
                Applied::Moved
            });
        }
        EditAction::Redo if !read_only => {
            return Some(if undo.redo(buf) {
                Applied::Edited
            } else {
                Applied::Moved
            });
        }
        EditAction::Undo | EditAction::Redo => return Some(Applied::Moved),
        EditAction::SelectAll => {
            buf.select_all();
            undo.seal();
            return Some(Applied::Moved);
        }
        EditAction::Copy => {
            if let Some(t) = buf.selected_text() {
                clipboard::copy(t);
            }
            return Some(Applied::Moved);
        }
        EditAction::Cut => match buf.selected_text().map(str::to_owned) {
            Some(t) => {
                clipboard::copy(&t);
                if !read_only {
                    buf.backspace();
                }
            }
            // Ctrl+W without a selection deletes the word left
            None if key_code == KeyCode::Char('w') && !read_only => buf.delete_word_left(),
            None => {}
        },
        EditAction::Paste => {
            if let Some(t) = clipboard::text()
                && !read_only
            {
                buf.insert_str(&t);
            }
        }
        EditAction::Insert(c) => {
            if !read_only {
                buf.insert_char(c);
            }
        }
        EditAction::Apply(f) => {
            let mut probe = buf.clone();
            f(&mut probe);
            if !read_only || probe.text() == buf.text() {
                *buf = probe;
            }
        }
    }
    if buf.text() != before.text() {
        undo.record(&before, kind);
        Some(Applied::Edited)
    } else {
        undo.seal();
        Some(Applied::Moved)
    }
}

/// Select All in every text control, through the one shared mechanism
/// ([`edit_key_in`] to [`EditAction::SelectAll`] to [`apply_edit`]).
#[cfg(test)]
mod select_all_tests {
    use super::*;
    use crate::core::edit::EditEntry;
    use crate::core::keymap::KeyMap;
    use ratatui::crossterm::event::KeyModifiers as M;

    fn k(c: char, mods: M) -> Key {
        Key {
            code: KeyCode::Char(c),
            mods,
        }
    }

    #[test]
    fn the_shared_table() {
        let is_all = |a| matches!(a, EditAction::SelectAll);
        for keys in [FieldKeys::Emacs, FieldKeys::Desktop] {
            assert!(
                is_all(edit_key_with(&k('a', M::SUPER), false, keys)),
                "Cmd+A"
            );
            assert!(
                matches!(
                    edit_key_with(&k('x', M::SUPER), false, keys),
                    EditAction::None
                ),
                "a Cmd chord never types its letter"
            );
        }
        assert!(is_all(edit_key_with(
            &k('a', M::CONTROL),
            false,
            FieldKeys::Desktop
        )));
        assert!(!is_all(edit_key_with(
            &k('a', M::CONTROL),
            false,
            FieldKeys::Emacs
        )));
        assert!(is_all(edit_key_with(
            &k('l', M::CONTROL),
            false,
            FieldKeys::Emacs
        )));
        // the per-widget key map replaces the preset's bindings
        let map: KeyMap<TextCmd> = vec![(KeyBinding::alt('a'), TextCmd::SelectAll)];
        assert!(is_all(edit_key_in(
            &k('a', M::ALT),
            false,
            FieldKeys::Emacs,
            &map
        )));
        assert!(!is_all(edit_key_in(
            &k('a', M::SUPER),
            false,
            FieldKeys::Emacs,
            &map
        )));
        assert!(!is_all(edit_key_in(
            &k('a', M::CONTROL),
            false,
            FieldKeys::Desktop,
            &map
        )));
    }

    #[test]
    fn text_input_and_its_widget_keymap() {
        use crate::widgets::input::TextInput;
        for keys in [FieldKeys::Emacs, FieldKeys::Desktop] {
            let mut i = TextInput::new(WidgetId::of("i"), "Name")
                .value("hello world")
                .field_keys(keys);
            i.begin_edit();
            i.buffer.move_doc_end(false);
            i.buffer.clear_selection();
            assert_eq!(i.on_key(&k('a', M::SUPER)).0, Outcome::Changed);
            assert_eq!(i.buffer.selected_text(), Some("hello world"), "{keys:?}");
            i.on_key(&k('z', M::NONE));
            assert_eq!(i.text(), "z", "typing replaces the selection");
        }
        // a field waiting for Enter/F2 starts the edit with all selected
        let mut wait = TextInput::new(WidgetId::of("w"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Explicit)
            .field_keys(FieldKeys::Desktop);
        assert_eq!(wait.on_key(&k('a', M::CONTROL)).0, Outcome::Changed);
        assert!(wait.editing);
        assert_eq!(wait.buffer.selected_text(), Some("abc"));
        // the widget key map
        let mut remapped = TextInput::new(WidgetId::of("r"), "Name")
            .value("abc")
            .text_keymap(vec![(KeyBinding::ctrl('g'), TextCmd::SelectAll)]);
        remapped.begin_edit();
        remapped.buffer.clear_selection();
        remapped.on_key(&k('g', M::CONTROL));
        assert_eq!(remapped.buffer.selected_text(), Some("abc"));
    }

    #[test]
    fn text_area_selects_every_line() {
        use crate::widgets::textarea::TextArea;
        let mut a = TextArea::new(WidgetId::of("a"), "Notes", 4)
            .value("one\ntwo\nthree")
            .field_keys(FieldKeys::Desktop);
        a.begin_edit();
        a.on_key(&k('a', M::CONTROL));
        assert_eq!(a.buffer.selected_text(), Some("one\ntwo\nthree"));
        a.commit();
        a.on_key(&k('a', M::SUPER));
        assert!(a.editing, "Select All picks the edit up again");
        assert_eq!(a.buffer.selected_text(), Some("one\ntwo\nthree"));
    }

    #[test]
    fn code_editor_editing_and_read_only() {
        use crate::widgets::code::CodeEditor;
        let mut e =
            CodeEditor::new(WidgetId::of("c"), "SELECT 1;\nSELECT 2;").field_keys(FieldKeys::Emacs);
        e.begin_edit();
        e.on_key(&k('a', M::SUPER));
        assert_eq!(e.buffer.selected_text(), Some("SELECT 1;\nSELECT 2;"));
        e.on_key(&k('x', M::NONE));
        assert_eq!(e.text(), "x");
        let mut ro = CodeEditor::new(WidgetId::of("r"), "fn main() {}").read_only(true);
        ro.on_key(&k('a', M::SUPER));
        assert_eq!(
            ro.buffer.selected_text(),
            Some("fn main() {}"),
            "read-only selects for Copy"
        );
        assert_eq!(ro.text(), "fn main() {}");
    }

    #[test]
    fn grid_and_table_cell_editors() {
        use crate::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
        use crate::widgets::table::{Cell, Column, DataTable};
        use ratatui::layout::Constraint;
        let mut g = DataGrid::new(
            WidgetId::of("g"),
            vec![ColumnSpec::new("name", CellKind::Text)],
        );
        g.set_rows(GridRows {
            rows: vec![vec![CellValue::Text("alpha".into())]],
            total: RowTotal::Exact(1),
            more: false,
        });
        g.begin_edit();
        g.edit.as_mut().unwrap().buffer.clear_selection();
        g.on_key(&k('a', M::SUPER));
        assert_eq!(
            g.edit.as_ref().unwrap().buffer.selected_text(),
            Some("alpha")
        );
        let mut t = DataTable::new(
            WidgetId::of("t"),
            vec![Column::new("Name", Constraint::Length(8)).editable()],
            vec![vec![Cell::new("beta")]],
        )
        .cell_nav(true);
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.clear_selection();
        t.on_key(&k('a', M::SUPER));
        assert_eq!(
            t.edit.as_ref().unwrap().buffer.selected_text(),
            Some("beta")
        );
    }
}
