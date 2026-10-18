//! The one edit key table shared by every text control (R1).
//!
//! [`TextCmd`] is the binding flavour: what a key gesture *means* in an edit
//! control, including lifecycle commands ([`Cancel`](TextCmd::Cancel),
//! [`Commit`](TextCmd::Commit)) and viewport paging ([`PageUp`](TextCmd::PageUp),
//! [`PageDown`](TextCmd::PageDown)) that no text buffer understands.
//! [`EditAction`] is the mutation vocabulary: what the text core *does* —
//! insert, delete, move, select. [`edit_action_of`] translates the former into
//! the latter (R7); lifecycle and paging commands never reach it on a live
//! path because each control's `edit_command_*` handles them first, so they
//! map to the defensive [`EditAction::ClearSelection`] fallback.
//!
//! Both flavours are assembled from one shared base — every arm with an
//! identical chord, command, label and visibility in both controls is defined
//! exactly once below — plus delta arms for what differs:
//!
//! * `Esc`/`Enter`: single-line cancels on `Esc` and commits on `Enter`; a
//!   document is not cancelled by leaving it, so multi-line commits on `Esc`
//!   (`Done`) and inserts a newline on `Enter`.
//! * `↑`/`↓` (plain and `Shift`-extended) and `PgUp`/`PgDn`: multi-line only.
//! * the `Home`/`End` family labels (`Start` vs `Line start`, `Document start`
//!   vs `Start`, …): the label feeds `ActionKey::custom`, so a different label
//!   is a different binding identity and must stay per-flavour.
//!
//! The assembled tables preserve the legacy arm order exactly (the legacy
//! `field_common::edit_key` table selected its arms on a `multiline` flag).

use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::input::TextCmd;
use crate::keymap::Binding;
use crate::text::{EditAction, Extend, Motion};

const fn b(chord: Chord, cmd: TextCmd, label: &'static str, visible: bool) -> Binding<TextCmd> {
    Binding {
        action: ActionKey::custom(label),
        chord: Some(chord),
        cmd,
        label,
        priority: 50,
        visible,
    }
}

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
const CTRL_SHIFT: KeyModifiers = CTRL.union(SHIFT);

// Shared base: identical chord, command, label and visibility in both flavours.
const LEFT: Binding<TextCmd> = b(
    Chord::key(KeyCode::Left),
    TextCmd::Move(Motion::Left, Extend::No),
    "Left",
    false,
);
const RIGHT: Binding<TextCmd> = b(
    Chord::key(KeyCode::Right),
    TextCmd::Move(Motion::Right, Extend::No),
    "Right",
    false,
);
const SELECT_LEFT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Left, SHIFT),
    TextCmd::Move(Motion::Left, Extend::Select),
    "Select left",
    false,
);
const SELECT_RIGHT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Right, SHIFT),
    TextCmd::Move(Motion::Right, Extend::Select),
    "Select right",
    false,
);
const WORD_LEFT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Left, CTRL),
    TextCmd::Move(Motion::WordLeft, Extend::No),
    "Word left",
    false,
);
const WORD_RIGHT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Right, CTRL),
    TextCmd::Move(Motion::WordRight, Extend::No),
    "Word right",
    false,
);
const WORD_LEFT_ALT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Left, ALT),
    TextCmd::Move(Motion::WordLeft, Extend::No),
    "Word left (Alt+Left)",
    false,
);
const WORD_RIGHT_ALT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Right, ALT),
    TextCmd::Move(Motion::WordRight, Extend::No),
    "Word right (Alt+Right)",
    false,
);
const BACKSPACE: Binding<TextCmd> = b(
    Chord::key(KeyCode::Backspace),
    TextCmd::Backspace,
    "Backspace",
    false,
);
const DELETE_WORD: Binding<TextCmd> = b(
    Chord::with(KeyCode::Backspace, CTRL),
    TextCmd::DeleteWordLeft,
    "Delete word",
    false,
);
const DELETE_WORD_ALT: Binding<TextCmd> = b(
    Chord::with(KeyCode::Backspace, ALT),
    TextCmd::DeleteWordLeft,
    "Delete word (Alt+Backspace)",
    false,
);
const DELETE: Binding<TextCmd> = b(
    Chord::key(KeyCode::Delete),
    TextCmd::Delete,
    "Delete",
    false,
);
const DELETE_TO_START: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('u'), CTRL),
    TextCmd::DeleteToLineStart,
    "Delete to start",
    false,
);
const DELETE_TO_END: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('k'), CTRL),
    TextCmd::DeleteToLineEnd,
    "Delete to end",
    false,
);
const DELETE_WORD_CTRL_W: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('w'), CTRL),
    TextCmd::DeleteWordLeft,
    "Delete word (Ctrl+W)",
    false,
);
const SELECT_ALL: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('l'), CTRL),
    TextCmd::SelectAll,
    "Select all",
    false,
);
const WORD_LEFT_ALT_B: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('b'), ALT),
    TextCmd::Move(Motion::WordLeft, Extend::No),
    "Word left (Alt+B)",
    false,
);
const WORD_RIGHT_ALT_F: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('f'), ALT),
    TextCmd::Move(Motion::WordRight, Extend::No),
    "Word right (Alt+F)",
    false,
);

// Delta: Esc/Enter differ per flavour.
const ESC_CANCEL: Binding<TextCmd> = b(Chord::key(KeyCode::Esc), TextCmd::Cancel, "Cancel", true);
const ENTER_COMMIT: Binding<TextCmd> =
    b(Chord::key(KeyCode::Enter), TextCmd::Commit, "Commit", true);
const ESC_DONE: Binding<TextCmd> = b(Chord::key(KeyCode::Esc), TextCmd::Commit, "Done", true);
const ENTER_NEWLINE: Binding<TextCmd> = b(
    Chord::key(KeyCode::Enter),
    TextCmd::Newline,
    "New line",
    true,
);

// Delta: vertical motion and paging exist only in the multi-line flavour.
const UP: Binding<TextCmd> = b(
    Chord::key(KeyCode::Up),
    TextCmd::Move(Motion::Up, Extend::No),
    "Up",
    false,
);
const DOWN: Binding<TextCmd> = b(
    Chord::key(KeyCode::Down),
    TextCmd::Move(Motion::Down, Extend::No),
    "Down",
    false,
);
const SELECT_UP: Binding<TextCmd> = b(
    Chord::with(KeyCode::Up, SHIFT),
    TextCmd::Move(Motion::Up, Extend::Select),
    "Select up",
    false,
);
const SELECT_DOWN: Binding<TextCmd> = b(
    Chord::with(KeyCode::Down, SHIFT),
    TextCmd::Move(Motion::Down, Extend::Select),
    "Select down",
    false,
);
const PAGE_UP: Binding<TextCmd> = b(
    Chord::key(KeyCode::PageUp),
    TextCmd::PageUp,
    "Page up",
    true,
);
const PAGE_DOWN: Binding<TextCmd> = b(
    Chord::key(KeyCode::PageDown),
    TextCmd::PageDown,
    "Page down",
    true,
);

// Delta: same chord and command, different label (hence different ActionKey).
const HOME_SINGLE: Binding<TextCmd> = b(
    Chord::key(KeyCode::Home),
    TextCmd::Move(Motion::Home, Extend::No),
    "Start",
    false,
);
const HOME_MULTI: Binding<TextCmd> = b(
    Chord::key(KeyCode::Home),
    TextCmd::Move(Motion::Home, Extend::No),
    "Line start",
    false,
);
const END_SINGLE: Binding<TextCmd> = b(
    Chord::key(KeyCode::End),
    TextCmd::Move(Motion::End, Extend::No),
    "End",
    false,
);
const END_MULTI: Binding<TextCmd> = b(
    Chord::key(KeyCode::End),
    TextCmd::Move(Motion::End, Extend::No),
    "Line end",
    false,
);
const SELECT_HOME_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, SHIFT),
    TextCmd::Move(Motion::Home, Extend::Select),
    "Select to start",
    false,
);
const SELECT_HOME_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, SHIFT),
    TextCmd::Move(Motion::Home, Extend::Select),
    "Select to line start",
    false,
);
const SELECT_END_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, SHIFT),
    TextCmd::Move(Motion::End, Extend::Select),
    "Select to end",
    false,
);
const SELECT_END_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, SHIFT),
    TextCmd::Move(Motion::End, Extend::Select),
    "Select to line end",
    false,
);
const DOC_START_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, CTRL),
    TextCmd::Move(Motion::DocStart, Extend::No),
    "Document start",
    false,
);
const DOC_START_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, CTRL),
    TextCmd::Move(Motion::DocStart, Extend::No),
    "Start",
    false,
);
const DOC_END_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, CTRL),
    TextCmd::Move(Motion::DocEnd, Extend::No),
    "Document end",
    false,
);
const DOC_END_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, CTRL),
    TextCmd::Move(Motion::DocEnd, Extend::No),
    "End",
    false,
);
// Baseline `edit_key`: `Home if ctrl` ignores Shift — Shift+Ctrl+Home/End
// move to the document edge and never extend.
const DOC_START_SHIFT_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, CTRL_SHIFT),
    TextCmd::Move(Motion::DocStart, Extend::No),
    "Document start (Shift)",
    false,
);
const DOC_START_SHIFT_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::Home, CTRL_SHIFT),
    TextCmd::Move(Motion::DocStart, Extend::No),
    "Start (Shift)",
    false,
);
const DOC_END_SHIFT_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, CTRL_SHIFT),
    TextCmd::Move(Motion::DocEnd, Extend::No),
    "Document end (Shift)",
    false,
);
const DOC_END_SHIFT_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::End, CTRL_SHIFT),
    TextCmd::Move(Motion::DocEnd, Extend::No),
    "End (Shift)",
    false,
);
const HOME_CTRL_A_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('a'), CTRL),
    TextCmd::Move(Motion::Home, Extend::No),
    "Start (Ctrl+A)",
    false,
);
const HOME_CTRL_A_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('a'), CTRL),
    TextCmd::Move(Motion::Home, Extend::No),
    "Line start (Ctrl+A)",
    false,
);
const END_CTRL_E_SINGLE: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('e'), CTRL),
    TextCmd::Move(Motion::End, Extend::No),
    "End (Ctrl+E)",
    false,
);
const END_CTRL_E_MULTI: Binding<TextCmd> = b(
    Chord::with(KeyCode::Char('e'), CTRL),
    TextCmd::Move(Motion::End, Extend::No),
    "Line end (Ctrl+E)",
    false,
);

/// The single-line flavour of the shared edit table (the legacy
/// `field_common::edit_key(key, multiline = false)`): `Esc` drops the draft,
/// `Enter` commits it, and there is no vertical motion.
const SINGLE: &[Binding<TextCmd>] = &[
    ESC_CANCEL,
    ENTER_COMMIT,
    LEFT,
    RIGHT,
    SELECT_LEFT,
    SELECT_RIGHT,
    WORD_LEFT,
    WORD_RIGHT,
    WORD_LEFT_ALT,
    WORD_RIGHT_ALT,
    HOME_SINGLE,
    END_SINGLE,
    SELECT_HOME_SINGLE,
    SELECT_END_SINGLE,
    DOC_START_SINGLE,
    DOC_END_SINGLE,
    DOC_START_SHIFT_SINGLE,
    DOC_END_SHIFT_SINGLE,
    BACKSPACE,
    DELETE_WORD,
    DELETE_WORD_ALT,
    DELETE,
    HOME_CTRL_A_SINGLE,
    END_CTRL_E_SINGLE,
    DELETE_TO_START,
    DELETE_TO_END,
    DELETE_WORD_CTRL_W,
    SELECT_ALL,
    WORD_LEFT_ALT_B,
    WORD_RIGHT_ALT_F,
];

/// The multi-line flavour of the shared edit table (the legacy
/// `field_common::edit_key(key, multiline = true)`): `Enter` inserts a
/// newline, `Esc` **commits** — a document is not cancelled by leaving it —
/// and `↑`/`↓`/`PgUp`/`PgDn` move the cursor by line and by page.
const MULTI: &[Binding<TextCmd>] = &[
    ESC_DONE,
    ENTER_NEWLINE,
    LEFT,
    RIGHT,
    UP,
    DOWN,
    SELECT_LEFT,
    SELECT_RIGHT,
    SELECT_UP,
    SELECT_DOWN,
    WORD_LEFT,
    WORD_RIGHT,
    WORD_LEFT_ALT,
    WORD_RIGHT_ALT,
    PAGE_UP,
    PAGE_DOWN,
    HOME_MULTI,
    END_MULTI,
    SELECT_HOME_MULTI,
    SELECT_END_MULTI,
    DOC_START_MULTI,
    DOC_END_MULTI,
    DOC_START_SHIFT_MULTI,
    DOC_END_SHIFT_MULTI,
    BACKSPACE,
    DELETE_WORD,
    DELETE_WORD_ALT,
    DELETE,
    HOME_CTRL_A_MULTI,
    END_CTRL_E_MULTI,
    DELETE_TO_START,
    DELETE_TO_END,
    DELETE_WORD_CTRL_W,
    SELECT_ALL,
    WORD_LEFT_ALT_B,
    WORD_RIGHT_ALT_F,
];

/// The edit key table for the flavour: single-line ([`TextInput`](crate::input::TextInput))
/// for `false`, multi-line ([`TextArea`](crate::textarea::TextArea)) for `true`.
pub(crate) fn edit_bindings(multiline: bool) -> &'static [Binding<TextCmd>] {
    if multiline { MULTI } else { SINGLE }
}

/// Translate a binding-flavour [`TextCmd`] into the mutation-vocabulary
/// [`EditAction`] the text core applies.
///
/// [`Cancel`](TextCmd::Cancel), [`Commit`](TextCmd::Commit),
/// [`PageUp`](TextCmd::PageUp) and [`PageDown`](TextCmd::PageDown) have no
/// direct mutation: each control's `edit_command_*` handles them before this
/// mapper runs (lifecycle writes the draft, paging repeats line motion), so
/// they land on the defensive [`EditAction::ClearSelection`] fallback. A
/// single-line control never binds [`Newline`](TextCmd::Newline), but the
/// single shared mapper still translates it for the flavour that does.
pub(crate) fn edit_action_of(cmd: TextCmd) -> EditAction<'static> {
    match cmd {
        TextCmd::Move(m, e) => EditAction::Move(m, e),
        TextCmd::Newline => EditAction::Newline,
        TextCmd::Backspace => EditAction::Backspace,
        TextCmd::Delete => EditAction::Delete,
        TextCmd::DeleteWordLeft => EditAction::DeleteWordLeft,
        TextCmd::DeleteToLineEnd => EditAction::DeleteToLineEnd,
        TextCmd::DeleteToLineStart => EditAction::DeleteToLineStart,
        TextCmd::SelectAll => EditAction::SelectAll,
        TextCmd::Cancel | TextCmd::Commit | TextCmd::PageUp | TextCmd::PageDown => {
            EditAction::ClearSelection
        }
    }
}
