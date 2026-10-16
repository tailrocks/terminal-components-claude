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

/// What an editing text control answers to a key it has no use for
/// ([`EditAction::None`]). A modified chord (Ctrl, Alt or Cmd) the field
/// does not bind passes on (`Ignored`), so the container's and the app's
/// shortcuts (Ctrl+Q quit, Ctrl+N new, Ctrl+S save) work while a field is
/// editing, as in a desktop text field, which handles only the keys it
/// uses. Bound chords never get here: the preset's readline keys and the
/// text keymap ([`FieldKeys::text_keymap`], the Cmd chords included)
/// resolve first. Any other unused key (an F-key, ↑↓ in a single-line
/// field) stays with the edit (`Consumed`).
pub fn unbound_key(key: &Key) -> Outcome {
    if key.chord() {
        Outcome::Ignored
    } else {
        Outcome::Consumed
    }
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
        return text_cmd_action(cmd, multiline);
    }
    let shift = key.shift();
    // Ctrl+Backspace often arrives as Ctrl+H (readline: delete left)
    if key.code == KeyCode::Char('h') && key.ctrl() {
        return EditAction::Apply(|b| b.backspace());
    }
    if keys == FieldKeys::Desktop && key.ctrl() && !key.alt() {
        // Select All comes from the text keymap; unbound there, Ctrl+A
        // does nothing rather than turn into the Cocoa line start
        if key.code == KeyCode::Char('a') {
            return EditAction::None;
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
        KeyCode::Char('b') if key.alt() => EditAction::Apply(|b| b.move_word_left(false)),
        KeyCode::Char('f') if key.alt() => EditAction::Apply(|b| b.move_word_right(false)),
        KeyCode::Char(_) if key.chord() => EditAction::None,
        KeyCode::Char(c) => EditAction::Insert(c),
        _ => EditAction::None,
    }
}

/// The edit action a [`TextCmd`] stands for. `multiline` decides whether
/// the text-start and text-end commands apply: a single-line control
/// leaves Cmd+↑/↓ to its container.
pub fn text_cmd_action(cmd: TextCmd, multiline: bool) -> EditAction {
    match cmd {
        TextCmd::SelectAll => EditAction::SelectAll,
        TextCmd::Copy => EditAction::Copy,
        TextCmd::Cut => EditAction::Cut,
        TextCmd::Paste => EditAction::Paste,
        TextCmd::Undo => EditAction::Undo,
        TextCmd::Redo => EditAction::Redo,
        TextCmd::LineStart => EditAction::Apply(|b| b.move_home(false)),
        TextCmd::LineEnd => EditAction::Apply(|b| b.move_end(false)),
        TextCmd::SelectToLineStart => EditAction::Apply(|b| b.move_home(true)),
        TextCmd::SelectToLineEnd => EditAction::Apply(|b| b.move_end(true)),
        TextCmd::DocStart
        | TextCmd::DocEnd
        | TextCmd::SelectToDocStart
        | TextCmd::SelectToDocEnd
            if !multiline =>
        {
            EditAction::None
        }
        TextCmd::DocStart => EditAction::Apply(|b| b.move_doc_start(false)),
        TextCmd::DocEnd => EditAction::Apply(|b| b.move_doc_end(false)),
        TextCmd::SelectToDocStart => EditAction::Apply(|b| b.move_doc_start(true)),
        TextCmd::SelectToDocEnd => EditAction::Apply(|b| b.move_doc_end(true)),
    }
}

/// A Paste key on a control that is not editing yet: the field
/// clipboard's text goes through the control's bracketed-paste handler
/// (`on_paste`), which picks the edit up exactly as a terminal paste does.
/// `Ignored` when the clipboard is empty, so the key still reaches the
/// page.
pub fn paste_clipboard(on_paste: impl FnOnce(&str) -> Outcome) -> Outcome {
    match clipboard::text() {
        Some(t) => on_paste(&t),
        None => Outcome::Ignored,
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
                    edit_key_with(&k('q', M::SUPER), false, keys),
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

/// The macOS standard editing chords (Cmd, delivered through the kitty
/// keyboard protocol) in every text control, under both presets, through
/// the one shared mechanism ([`FieldKeys::text_keymap`] to
/// [`edit_key_in`] to [`apply_edit`]).
#[cfg(test)]
mod cmd_edit_tests {
    use super::*;
    use crate::core::edit::CMD_TEXT_KEYS;
    use crate::core::keymap::hint_key;
    use ratatui::crossterm::event::KeyModifiers as M;

    const PRESETS: [FieldKeys; 2] = [FieldKeys::Emacs, FieldKeys::Desktop];

    fn k(code: KeyCode, mods: M) -> Key {
        Key { code, mods }
    }
    fn cmd(c: char) -> Key {
        k(KeyCode::Char(c), M::SUPER)
    }
    fn cmd_key(code: KeyCode) -> Key {
        k(code, M::SUPER)
    }
    fn cmd_shift(code: KeyCode) -> Key {
        k(code, M::SUPER | M::SHIFT)
    }

    /// The system clipboard (the hook) sees every copy and cut.
    fn watch_clipboard() -> std::rc::Rc<std::cell::RefCell<Vec<String>>> {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let s = seen.clone();
        clipboard::set_hook(move |t| s.borrow_mut().push(t.to_owned()));
        seen
    }

    #[test]
    fn the_shared_table_answers_cmd_under_both_presets() {
        for keys in PRESETS {
            let act = |key: &Key, multiline| edit_key_with(key, multiline, keys);
            assert!(
                matches!(act(&cmd('c'), false), EditAction::Copy),
                "{keys:?}"
            );
            assert!(matches!(act(&cmd('x'), false), EditAction::Cut));
            assert!(matches!(act(&cmd('v'), false), EditAction::Paste));
            assert!(matches!(act(&cmd('z'), false), EditAction::Undo));
            // Cmd+Shift+Z arrives as `z` or `Z` with Shift
            for c in ['z', 'Z'] {
                let redo = k(KeyCode::Char(c), M::SUPER | M::SHIFT);
                assert!(matches!(act(&redo, false), EditAction::Redo), "{c}");
            }
            assert!(matches!(
                act(&k(KeyCode::Char('Z'), M::CONTROL | M::SHIFT), false),
                EditAction::Redo
            ));
            assert!(matches!(
                act(&k(KeyCode::Char('z'), M::CONTROL), false),
                EditAction::Undo
            ));
            for code in [KeyCode::Left, KeyCode::Right] {
                assert!(matches!(act(&cmd_key(code), false), EditAction::Apply(_)));
                assert!(matches!(act(&cmd_shift(code), false), EditAction::Apply(_)));
            }
            for code in [KeyCode::Up, KeyCode::Down] {
                assert!(matches!(act(&cmd_key(code), true), EditAction::Apply(_)));
                assert!(
                    matches!(act(&cmd_key(code), false), EditAction::None),
                    "a single-line field leaves Cmd+↑/↓ to its container"
                );
            }
            assert!(matches!(
                act(&k(KeyCode::Backspace, M::ALT), false),
                EditAction::Apply(_)
            ));
        }
        // the presets' own keys are unchanged
        let d = |key: &Key| edit_key_with(key, false, FieldKeys::Desktop);
        let e = |key: &Key| edit_key_with(key, false, FieldKeys::Emacs);
        let ctrl = |c| k(KeyCode::Char(c), M::CONTROL);
        assert!(matches!(d(&ctrl('c')), EditAction::Copy));
        assert!(matches!(d(&ctrl('x')), EditAction::Cut));
        assert!(matches!(d(&ctrl('v')), EditAction::Paste));
        assert!(matches!(d(&ctrl('y')), EditAction::Redo));
        assert!(matches!(e(&ctrl('y')), EditAction::Paste));
        assert!(matches!(e(&ctrl('w')), EditAction::Cut));
        assert!(matches!(d(&ctrl('w')), EditAction::Cut));
        assert!(matches!(
            e(&k(KeyCode::Char('w'), M::ALT)),
            EditAction::Copy
        ));
        assert!(matches!(e(&ctrl('a')), EditAction::Apply(_)), "line start");
        assert!(matches!(e(&ctrl('c')), EditAction::None));
    }

    #[test]
    fn caret_chords_move_and_select_by_line_and_text() {
        let mut b = TextBuffer::multi("one two\nthree four");
        let mut u = crate::core::edit::UndoStack::new();
        let mut run = |b: &mut TextBuffer, key: Key| {
            let a = edit_key_with(&key, true, FieldKeys::Emacs);
            apply_edit(b, &mut u, a, key.code, false);
        };
        b.move_doc_end(false);
        run(&mut b, cmd_key(KeyCode::Left));
        assert_eq!(b.cursor_offset(), "one two\n".len(), "Cmd+← line start");
        run(&mut b, cmd_key(KeyCode::Right));
        assert_eq!(b.cursor_offset(), b.text().len(), "Cmd+→ line end");
        run(&mut b, cmd_key(KeyCode::Up));
        assert_eq!(b.cursor_offset(), 0, "Cmd+↑ text start");
        run(&mut b, cmd_shift(KeyCode::Right));
        assert_eq!(b.selected_text(), Some("one two"));
        run(&mut b, cmd_shift(KeyCode::Down));
        assert_eq!(b.selected_text(), Some("one two\nthree four"));
        run(&mut b, cmd_key(KeyCode::Down));
        assert_eq!(b.cursor_offset(), b.text().len(), "Cmd+↓ text end");
        run(&mut b, k(KeyCode::Left, M::ALT));
        assert_eq!(b.cursor_offset(), "one two\nthree ".len(), "Alt+← word");
        run(&mut b, k(KeyCode::Backspace, M::ALT));
        assert_eq!(b.text(), "one two\nfour", "Alt+Backspace deletes the word");
    }

    #[test]
    fn hints_name_cmd_only_once_delivered_else_the_preset_key() {
        let desktop = FieldKeys::Desktop.text_keymap();
        let emacs = FieldKeys::Emacs.text_keymap();
        let fallback = [
            (TextCmd::Copy, "Ctrl+C", "Alt+W"),
            (TextCmd::Cut, "Ctrl+X", "Ctrl+W"),
            (TextCmd::Paste, "Ctrl+V", "Ctrl+Y"),
            (TextCmd::Undo, "Ctrl+Z", "Ctrl+Z"),
            (TextCmd::Redo, "Ctrl+Y", "Ctrl+Shift+Z"),
        ];
        for (cmd, d, e) in fallback {
            assert_eq!(hint_key(&desktop, cmd), Some(d), "{cmd:?}");
            assert_eq!(hint_key(&emacs, cmd), Some(e), "{cmd:?}");
        }
        assert_eq!(hint_key(&emacs, TextCmd::LineStart), None, "Cmd only");
        crate::core::keyboard::set_super_delivered(true);
        for (b, cmd) in CMD_TEXT_KEYS {
            if crate::core::keymap::binding_for(&CMD_TEXT_KEYS, cmd) != Some(b) {
                continue;
            }
            let label = b.label();
            assert_eq!(hint_key(&desktop, cmd), Some(label.as_str()), "{cmd:?}");
            assert_eq!(hint_key(&emacs, cmd), Some(label.as_str()), "{cmd:?}");
        }
        use crate::core::event::ClientPlatform::MacOs;
        let label = |cmd| {
            crate::core::keymap::deliverable_binding_for(&emacs, cmd)
                .unwrap()
                .label_on(MacOs)
        };
        assert_eq!(label(TextCmd::Copy), "⌘C");
        assert_eq!(label(TextCmd::Redo), "⇧⌘Z");
        assert_eq!(label(TextCmd::LineStart), "⌘←");
        assert_eq!(label(TextCmd::SelectToDocEnd), "⇧⌘↓");
    }

    #[test]
    fn text_input_round_trips_cut_and_paste_and_undoes_it() {
        use crate::widgets::input::TextInput;
        for keys in PRESETS {
            let seen = watch_clipboard();
            let mut i = TextInput::new(WidgetId::of("i"), "Name")
                .value("hello world")
                .field_keys(keys);
            i.begin_edit();
            i.buffer.move_doc_end(false);
            i.on_key(&cmd_key(KeyCode::Left));
            assert_eq!(i.buffer.cursor_offset(), 0, "{keys:?}");
            i.on_key(&k(KeyCode::Right, M::ALT));
            i.on_key(&cmd_shift(KeyCode::Right));
            assert_eq!(i.buffer.selected_text(), Some(" world"));
            i.on_key(&cmd('c'));
            assert_eq!(seen.borrow().last().map(String::as_str), Some(" world"));
            i.on_key(&cmd('x'));
            assert_eq!(i.text(), "hello");
            assert_eq!(seen.borrow().len(), 2, "copy and cut both reach the hook");
            i.on_key(&cmd_key(KeyCode::Left));
            i.on_key(&cmd('v'));
            assert_eq!(i.text(), " worldhello");
            i.on_key(&cmd('z'));
            assert_eq!(i.text(), "hello");
            i.on_key(&cmd_shift(KeyCode::Char('z')));
            assert_eq!(i.text(), " worldhello", "Cmd+Shift+Z redoes");
            clipboard::clear_hook();
        }
    }

    #[test]
    fn text_input_paste_picks_the_edit_up() {
        use crate::core::edit::EditEntry;
        use crate::widgets::input::{InputEvent, TextInput};
        clipboard::copy("abc");
        let mut i = TextInput::new(WidgetId::of("w"), "Name")
            .value("")
            .edit_entry(EditEntry::Explicit);
        assert!(!i.editing);
        assert_eq!(
            i.on_key(&cmd('v')),
            (Outcome::Changed, Some(InputEvent::Changed))
        );
        assert!(i.editing);
        assert_eq!(i.text(), "abc");
    }

    #[test]
    fn text_area_lines_text_and_paste_after_done() {
        use crate::widgets::textarea::TextArea;
        for keys in PRESETS {
            let mut a = TextArea::new(WidgetId::of("a"), "Notes", 4)
                .value("one\ntwo\nthree")
                .edit_entry(crate::core::edit::EditEntry::TypeToEdit)
                .field_keys(keys);
            a.begin_edit();
            a.on_key(&cmd_key(KeyCode::Down));
            assert_eq!(a.buffer.cursor_offset(), a.buffer.text().len(), "{keys:?}");
            a.on_key(&cmd_key(KeyCode::Up));
            assert_eq!(a.buffer.cursor_offset(), 0);
            a.on_key(&cmd_key(KeyCode::Right));
            assert_eq!(a.buffer.cursor_offset(), 3);
            a.on_key(&cmd_shift(KeyCode::Down));
            assert_eq!(a.buffer.selected_text(), Some("\ntwo\nthree"));
            a.on_key(&cmd('x'));
            assert_eq!(a.buffer.text(), "one");
            a.on_key(&k(KeyCode::Esc, M::NONE));
            assert!(!a.editing);
            a.on_key(&cmd('v'));
            assert!(a.editing, "Cmd+V picks the edit up");
            assert_eq!(a.buffer.text(), "one\ntwo\nthree");
            a.on_key(&cmd('z'));
            assert_eq!(a.buffer.text(), "one");
        }
    }

    #[test]
    fn code_editor_editing_browsing_and_read_only() {
        use crate::widgets::code::CodeEditor;
        let seen = watch_clipboard();
        let mut e = CodeEditor::new(WidgetId::of("c"), "SELECT 1;\nSELECT 2;")
            .edit_entry(crate::core::edit::EditEntry::Explicit);
        e.begin_edit();
        e.on_key(&cmd_key(KeyCode::Down));
        e.on_key(&cmd_shift(KeyCode::Left));
        assert_eq!(e.buffer.selected_text(), Some("SELECT 2;"));
        e.on_key(&cmd('x'));
        assert_eq!(e.text(), "SELECT 1;\n");
        e.on_key(&cmd('z'));
        assert_eq!(e.text(), "SELECT 1;\nSELECT 2;");
        // browsing (after Esc): Cmd+V picks the edit up and pastes
        e.on_key(&k(KeyCode::Esc, M::NONE));
        assert!(!e.editing);
        e.on_key(&cmd_key(KeyCode::Up));
        assert_eq!(e.buffer.cursor_offset(), 0, "Cmd+↑ while browsing");
        e.on_key(&cmd('v'));
        assert!(e.editing);
        assert_eq!(e.text(), "SELECT 2;SELECT 1;\nSELECT 2;");
        // a read-only document selects and copies, never changes
        let mut ro = CodeEditor::new(WidgetId::of("r"), "fn main() {}\nfn x() {}").read_only(true);
        ro.on_key(&cmd_key(KeyCode::Down));
        ro.on_key(&cmd_shift(KeyCode::Left));
        assert_eq!(ro.buffer.selected_text(), Some("fn x() {}"));
        ro.on_key(&cmd('c'));
        assert_eq!(seen.borrow().last().map(String::as_str), Some("fn x() {}"));
        for key in [cmd('x'), cmd('v'), cmd('z')] {
            assert_eq!(ro.on_key(&key).0, Outcome::Consumed);
        }
        assert_eq!(ro.text(), "fn main() {}\nfn x() {}");
        clipboard::clear_hook();
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
        g.on_key(&cmd('a'));
        g.on_key(&cmd('x'));
        assert_eq!(g.edit.as_ref().unwrap().buffer.text(), "");
        g.on_key(&cmd('v'));
        g.on_key(&cmd_key(KeyCode::Left));
        g.on_key(&cmd('v'));
        assert_eq!(g.edit.as_ref().unwrap().buffer.text(), "alphaalpha");
        let mut t = DataTable::new(
            WidgetId::of("t"),
            vec![Column::new("Name", Constraint::Length(8)).editable()],
            vec![vec![Cell::new("beta")]],
        )
        .cell_nav(true);
        t.begin_edit();
        t.on_key(&cmd('a'));
        t.on_key(&cmd('c'));
        assert_eq!(clipboard::text().as_deref(), Some("beta"));
        t.on_key(&cmd_key(KeyCode::Right));
        t.on_key(&cmd('v'));
        assert_eq!(t.edit.as_ref().unwrap().buffer.text(), "betabeta");
    }
}
