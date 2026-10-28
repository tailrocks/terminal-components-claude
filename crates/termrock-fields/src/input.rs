use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Modifier, Style};

use super::{PartStyle, SlotFn, cell_at, first_row, shift};
use crate::action::ActionKey;
pub use crate::edit::{EditCapabilities, EditMenuRequest, TextEditCommand};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::secret::{Secret, SecretPolicy, wipe_string};
use crate::text::measure::graphemes;
use crate::text::{EditAction, EditOutcome, Extend, Motion, TextEditorCore, width};
use crate::theme::{Family, GlyphRole, PaintStyle, Role, Slot, StyleDefaults, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};
use termrock_runtime::Status;
use termrock_runtime::layer::LayerOpening;
use termrock_runtime::rowui::CellUi;

mod text_target {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for crate::secret::Secret {}
}

pub trait TextTarget: text_target::Sealed {
    fn expose(&self) -> &str;
    fn set(&mut self, value: &str, sensitive: bool);
    fn is_sensitive(&self) -> bool;
}

impl TextTarget for String {
    fn expose(&self) -> &str {
        self
    }

    fn set(&mut self, value: &str, sensitive: bool) {
        if sensitive {
            wipe_string(core::mem::take(self));
        } else {
            self.clear();
        }
        self.push_str(value);
    }

    fn is_sensitive(&self) -> bool {
        false
    }
}

impl TextTarget for Secret {
    fn expose(&self) -> &str {
        Secret::expose(self)
    }

    fn set(&mut self, value: &str, _sensitive: bool) {
        Secret::set(self, value);
    }

    fn is_sensitive(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BlurPolicy {
    #[default]
    CommitAndValidate,
    Commit,
    Cancel,
    Keep,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FocusSelect {
    CaretStart,

    #[default]
    CaretEnd,

    SelectAll,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EditPhase {
    #[default]
    Idle,
    Editing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAction {
    Changed,
    Committed,
    Cancelled,
    MoveNext,
    MovePrev,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCmd {
    Cancel,
    Commit,
    Move(Motion, Extend),
    Backspace,
    Delete,
    DeleteWordLeft,
    DeleteToLineEnd,
    DeleteToLineStart,
    SelectAll,
    Newline,
    PageUp,
    PageDown,
}

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

const BINDINGS: &[Binding<TextCmd>] = &[
    b(Chord::key(KeyCode::Esc), TextCmd::Cancel, "Cancel", true),
    b(Chord::key(KeyCode::Enter), TextCmd::Commit, "Commit", true),
    b(
        Chord::key(KeyCode::Left),
        TextCmd::Move(Motion::Left, Extend::No),
        "Left",
        false,
    ),
    b(
        Chord::key(KeyCode::Right),
        TextCmd::Move(Motion::Right, Extend::No),
        "Right",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, SHIFT),
        TextCmd::Move(Motion::Left, Extend::Select),
        "Select left",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, SHIFT),
        TextCmd::Move(Motion::Right, Extend::Select),
        "Select right",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, CTRL),
        TextCmd::Move(Motion::WordLeft, Extend::No),
        "Word left",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, CTRL),
        TextCmd::Move(Motion::WordRight, Extend::No),
        "Word right",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, ALT),
        TextCmd::Move(Motion::WordLeft, Extend::No),
        "Word left (Alt+Left)",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, ALT),
        TextCmd::Move(Motion::WordRight, Extend::No),
        "Word right (Alt+Right)",
        false,
    ),
    b(
        Chord::key(KeyCode::Home),
        TextCmd::Move(Motion::Home, Extend::No),
        "Start",
        false,
    ),
    b(
        Chord::key(KeyCode::End),
        TextCmd::Move(Motion::End, Extend::No),
        "End",
        false,
    ),
    b(
        Chord::with(KeyCode::Home, SHIFT),
        TextCmd::Move(Motion::Home, Extend::Select),
        "Select to start",
        false,
    ),
    b(
        Chord::with(KeyCode::End, SHIFT),
        TextCmd::Move(Motion::End, Extend::Select),
        "Select to end",
        false,
    ),
    b(
        Chord::with(KeyCode::Home, CTRL),
        TextCmd::Move(Motion::DocStart, Extend::No),
        "Document start",
        false,
    ),
    b(
        Chord::with(KeyCode::End, CTRL),
        TextCmd::Move(Motion::DocEnd, Extend::No),
        "Document end",
        false,
    ),
    b(
        Chord::key(KeyCode::Backspace),
        TextCmd::Backspace,
        "Backspace",
        false,
    ),
    b(
        Chord::with(KeyCode::Backspace, CTRL),
        TextCmd::DeleteWordLeft,
        "Delete word",
        false,
    ),
    b(
        Chord::with(KeyCode::Backspace, ALT),
        TextCmd::DeleteWordLeft,
        "Delete word (Alt+Backspace)",
        false,
    ),
    b(
        Chord::key(KeyCode::Delete),
        TextCmd::Delete,
        "Delete",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('a'), CTRL),
        TextCmd::Move(Motion::Home, Extend::No),
        "Start (Ctrl+A)",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('e'), CTRL),
        TextCmd::Move(Motion::End, Extend::No),
        "End (Ctrl+E)",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('u'), CTRL),
        TextCmd::DeleteToLineStart,
        "Delete to start",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('k'), CTRL),
        TextCmd::DeleteToLineEnd,
        "Delete to end",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('w'), CTRL),
        TextCmd::DeleteWordLeft,
        "Delete word (Ctrl+W)",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('l'), CTRL),
        TextCmd::SelectAll,
        "Select all",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('b'), ALT),
        TextCmd::Move(Motion::WordLeft, Extend::No),
        "Word left (Alt+B)",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('f'), ALT),
        TextCmd::Move(Motion::WordRight, Extend::No),
        "Word right (Alt+F)",
        false,
    ),
];

pub(crate) enum EditorDraft {
    Plain(TextEditorCore),
    Secret(TextEditorCore),
}

impl Default for EditorDraft {
    fn default() -> Self {
        EditorDraft::Plain(TextEditorCore::default())
    }
}

impl EditorDraft {
    pub(crate) const fn is_sensitive(&self) -> bool {
        matches!(self, EditorDraft::Secret(_))
    }

    pub(crate) fn set_sensitive(&mut self, sensitive: bool) {
        if self.is_sensitive() == sensitive {
            return;
        }
        self.zeroize();
        *self = if sensitive {
            EditorDraft::Secret(TextEditorCore::sensitive_single(""))
        } else {
            EditorDraft::Plain(TextEditorCore::default())
        };
    }

    pub(crate) fn begin_single(&mut self, current: &str) {
        let sensitive = self.is_sensitive();
        self.zeroize();
        let editor = if sensitive {
            TextEditorCore::sensitive_single(current)
        } else {
            TextEditorCore::single(current)
        };
        *self = if sensitive {
            EditorDraft::Secret(editor)
        } else {
            EditorDraft::Plain(editor)
        };
    }

    pub(crate) fn begin_multi(&mut self, current: &str) {
        let sensitive = self.is_sensitive();
        self.zeroize();
        let editor = if sensitive {
            TextEditorCore::sensitive_multi(current)
        } else {
            TextEditorCore::multi(current)
        };
        *self = if sensitive {
            EditorDraft::Secret(editor)
        } else {
            EditorDraft::Plain(editor)
        };
    }

    pub(crate) fn text(&self) -> &str {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.text(),
        }
    }

    pub(crate) fn cursor_pos(&self) -> crate::text::CursorPos {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.cursor_pos(),
        }
    }

    pub(crate) fn hscroll(&self) -> u16 {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.hscroll(),
        }
    }

    pub(crate) fn line_count(&self) -> usize {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.line_count(),
        }
    }

    pub(crate) fn selection(&self) -> Option<core::ops::Range<usize>> {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.selection(),
        }
    }

    pub(crate) fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                editor.set_cursor_line_col(line, col);
            }
        }
    }

    pub(crate) fn pointer_point(&mut self, line: usize, col: usize) -> bool {
        let offset = self.offset_at(line, col);
        if self
            .selection()
            .is_some_and(|range| range.start <= offset && offset <= range.end)
        {
            return false;
        }
        let before = (self.cursor_offset(), self.selection());
        self.set_cursor_line_col(line, col);
        before != (self.cursor_offset(), self.selection())
    }

    pub(crate) fn offset_at(&self, line: usize, col: usize) -> usize {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.offset_at(line, col),
        }
    }

    pub(crate) fn select_word_at(&mut self, offset: usize) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                let _ = editor.select_word_at(offset);
            }
        }
    }

    pub(crate) fn select_line_at(&mut self, offset: usize) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                let _ = editor.select_line_at(offset);
            }
        }
    }

    pub(crate) fn pointer_select(&mut self, line: usize, col: usize, click_count: u8) -> usize {
        let offset = self.offset_at(line, col);
        match click_count {
            3.. if !self.is_sensitive() => self.select_line_at(offset),
            2 if !self.is_sensitive() => self.select_word_at(offset),
            _ => self.set_cursor_line_col(line, col),
        }
        self.selection()
            .map_or_else(|| self.cursor_offset(), |range| range.start)
    }

    pub(crate) fn pointer_extend(&mut self, anchor: usize, line: usize, col: usize) {
        self.select_range(anchor, self.offset_at(line, col));
    }

    pub(crate) fn cursor_offset(&self) -> usize {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.cursor_offset(),
        }
    }

    pub(crate) fn select_range(&mut self, start: usize, end: usize) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                editor.select_range(start, end);
            }
        }
    }

    pub(crate) fn scroll_into_view(&mut self, width: u16) -> u16 {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                editor.scroll_into_view(width)
            }
        }
    }

    pub(crate) fn apply(&mut self, action: EditAction<'_>) -> EditOutcome {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.apply(action),
        }
    }

    pub(crate) fn zeroize(&mut self) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor.zeroize(),
        }
    }

    pub(crate) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (EditorDraft::Plain(left), EditorDraft::Plain(right)) => left == right,
            (EditorDraft::Secret(_), EditorDraft::Secret(_)) => true,
            _ => false,
        }
    }

    pub(crate) fn clone_snapshot(&self) -> Self {
        match self {
            EditorDraft::Plain(editor) => EditorDraft::Plain(editor.clone()),
            EditorDraft::Secret(editor) => {
                let mut snapshot = if editor.is_multiline() {
                    TextEditorCore::sensitive_multi(&redacted_text(editor.text()))
                } else {
                    TextEditorCore::sensitive_single(&redacted_text(editor.text()))
                };
                let cursor = editor.cursor_pos();
                snapshot.set_cursor_line_col(cursor.line, cursor.col);
                EditorDraft::Secret(snapshot)
            }
        }
    }
}

pub enum ErrorState {
    Plain(FieldError),
    Pending(FieldError),
    Sensitive,
}

impl ErrorState {
    pub fn sensitive() -> Self {
        ErrorState::Sensitive
    }

    pub const fn is_sensitive(&self) -> bool {
        matches!(self, ErrorState::Sensitive)
    }

    pub const fn as_ref(&self) -> &FieldError {
        match self {
            ErrorState::Plain(error) => error,
            ErrorState::Pending(_) | ErrorState::Sensitive => &INVALID_VALUE,
        }
    }

    pub(crate) fn clone_snapshot(&self) -> Self {
        match self {
            ErrorState::Plain(error) => ErrorState::Plain(error.clone()),
            ErrorState::Pending(_) | ErrorState::Sensitive => ErrorState::Sensitive,
        }
    }

    pub fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (ErrorState::Plain(left), ErrorState::Plain(right)) => left == right,
            (
                ErrorState::Pending(_) | ErrorState::Sensitive,
                ErrorState::Pending(_) | ErrorState::Sensitive,
            ) => true,
            _ => false,
        }
    }

    pub fn redact(self) -> Self {
        match self {
            ErrorState::Plain(error) | ErrorState::Pending(error) => {
                discard_error(error);
                ErrorState::Sensitive
            }
            ErrorState::Sensitive => ErrorState::Sensitive,
        }
    }

    pub fn resolve_plain(self) -> Self {
        match self {
            ErrorState::Pending(error) => ErrorState::Plain(error),
            other => other,
        }
    }

    pub fn discard(self) {
        match self {
            ErrorState::Plain(error) | ErrorState::Pending(error) => discard_error(error),
            ErrorState::Sensitive => {}
        }
    }
}

impl Clone for ErrorState {
    fn clone(&self) -> Self {
        self.clone_snapshot()
    }
}

static INVALID_VALUE: FieldError = FieldError {
    message: std::borrow::Cow::Borrowed("Invalid value"),
    code: None,
};

#[derive(Default)]
pub struct TextInputState {
    draft: EditorDraft,
    phase: EditPhase,
    error: Option<ErrorState>,
    redacted_snapshot: bool,
    sensitivity: Option<bool>,
    edit_menu_request: Option<EditMenuRequest>,
    pointer_anchor: Option<usize>,
    focus_continuation: Option<LayerOpening>,
}

impl Clone for TextInputState {
    fn clone(&self) -> Self {
        TextInputState {
            draft: self.draft.clone_snapshot(),
            phase: self.phase,
            error: self.error.as_ref().map(ErrorState::clone_snapshot),
            redacted_snapshot: self.is_sensitive(),
            sensitivity: self.sensitivity,
            edit_menu_request: self.edit_menu_request,
            pointer_anchor: self.pointer_anchor,
            focus_continuation: self.focus_continuation,
        }
    }
}

impl PartialEq for TextInputState {
    fn eq(&self, other: &Self) -> bool {
        if self.is_sensitive() || other.is_sensitive() {
            self.is_sensitive() == other.is_sensitive()
                && self.phase == other.phase
                && self.pointer_anchor == other.pointer_anchor
                && self.focus_continuation == other.focus_continuation
                && self.error.as_ref().map(ErrorState::is_sensitive)
                    == other.error.as_ref().map(ErrorState::is_sensitive)
        } else {
            self.draft.same(&other.draft)
                && self.phase == other.phase
                && self.pointer_anchor == other.pointer_anchor
                && self.focus_continuation == other.focus_continuation
                && match (&self.error, &other.error) {
                    (Some(left), Some(right)) => left.same(right),
                    (None, None) => true,
                    _ => false,
                }
        }
    }
}

impl Eq for TextInputState {}

impl fmt::Debug for TextInputState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInputState")
            .field("draft", &"[redacted]")
            .field("draft_len", &self.draft.text().len())
            .field("phase", &self.phase)
            .field("error", &self.error.as_ref().map(|_| "[redacted]"))
            .field("redacted_snapshot", &self.redacted_snapshot)
            .field("sensitivity", &self.sensitivity)
            .field("pointer_anchor", &self.pointer_anchor)
            .field("focus_continuation", &self.focus_continuation)
            .field("sensitive", &self.is_sensitive())
            .finish()
    }
}

impl TextInputState {
    #[must_use]
    pub fn sensitive() -> Self {
        let mut state = Self::default();
        state.set_sensitive(true);
        state
    }

    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    pub fn take_edit_menu_request(&mut self) -> Option<EditMenuRequest> {
        self.edit_menu_request.take()
    }

    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    pub fn draft_text(&self) -> Option<&str> {
        if self.is_sensitive() || !self.is_editing() {
            None
        } else {
            Some(self.draft.text())
        }
    }

    pub fn selection(&self) -> Option<core::ops::Range<usize>> {
        self.is_editing().then(|| self.draft.selection()).flatten()
    }

    pub fn cursor_offset(&self) -> Option<usize> {
        self.is_editing().then(|| self.draft.cursor_offset())
    }

    pub fn select_all(&mut self) {
        if self.is_editing() {
            let _ = self.apply(EditAction::SelectAll);
        }
    }

    pub fn splice_completion(&mut self, replace_len: usize, insertion: &str) -> bool {
        if !self.is_editing() {
            return false;
        }
        let editor = match &mut self.draft {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => editor,
        };
        let end = editor.cursor_offset();
        let start = end.saturating_sub(replace_len);
        editor.select_range(start, end);
        editor.apply(EditAction::Paste(insertion)).changed()
    }

    pub fn visible_text_equals(&self, committed: &str, expected: &str) -> bool {
        !self.redacted_snapshot
            && if self.is_editing() {
                self.draft.text() == expected
            } else {
                committed == expected
            }
    }

    pub const fn is_sensitive(&self) -> bool {
        self.draft.is_sensitive()
    }

    pub const fn error(&self) -> Option<&FieldError> {
        match &self.error {
            Some(error) => Some(error.as_ref()),
            None => None,
        }
    }

    pub fn set_sensitive(&mut self, sensitive: bool) {
        let changed = self.is_sensitive() != sensitive;
        let pending_error = self
            .error
            .as_ref()
            .is_some_and(|error| matches!(error, ErrorState::Pending(_)));
        self.draft.set_sensitive(sensitive);
        self.sensitivity = Some(sensitive);
        if changed {
            self.phase = EditPhase::Idle;
            self.redacted_snapshot = false;
            if sensitive && pending_error {
                self.redact_error();
            } else {
                self.clear_error();
            }
        } else if sensitive {
            self.redact_error();
        } else {
            self.resolve_pending_error();
        }
    }

    pub fn set_error(&mut self, e: Option<FieldError>) {
        self.clear_error();
        self.error = match (self.sensitivity, e) {
            (_, None) => None,
            (Some(true), Some(error)) => {
                discard_error(error);
                Some(ErrorState::sensitive())
            }
            (Some(false), Some(error)) => Some(ErrorState::Plain(error)),
            (None, Some(error)) => Some(ErrorState::Pending(error)),
        };
    }

    fn redact_error(&mut self) {
        if let Some(error) = self.error.take() {
            self.error = Some(error.redact());
        }
    }

    fn resolve_pending_error(&mut self) {
        if let Some(error) = self.error.take() {
            self.error = Some(error.resolve_plain());
        }
    }

    pub fn begin(&mut self, current: &str) {
        if self.is_editing() {
            return;
        }
        self.redacted_snapshot = false;
        self.draft.begin_single(current);
        self.phase = EditPhase::Editing;
    }

    pub fn place_cursor(&mut self, col: usize, visible_width: u16) -> bool {
        if !self.is_editing() {
            return false;
        }
        let before = self.draft.cursor_offset();
        let col = col.saturating_add(usize::from(self.draft.hscroll()));
        self.draft.set_cursor_line_col(0, col);
        self.draft.scroll_into_view(visible_width);
        before != self.draft.cursor_offset()
    }

    pub fn select_pointer_range(
        &mut self,
        anchor: usize,
        col: usize,
        visible_width: u16,
    ) -> Option<usize> {
        if !self.is_editing() {
            return None;
        }
        let col = col.saturating_add(usize::from(self.draft.hscroll()));
        let cursor = self.draft.offset_at(0, col);
        self.draft.select_range(anchor, cursor);
        self.draft.scroll_into_view(visible_width);
        Some(cursor)
    }

    pub fn backspace(&mut self) -> bool {
        self.is_editing() && self.apply(EditAction::Backspace).changed()
    }

    pub fn ensure_cursor_visible(&mut self, visible_width: u16) {
        if self.is_editing() {
            self.draft.scroll_into_view(visible_width);
        }
    }

    pub fn commit(&mut self, value: &mut String, v: &impl Validate) -> Result<(), FieldError> {
        self.commit_target(value, v)
    }

    pub(crate) fn commit_target<T: TextTarget + ?Sized>(
        &mut self,
        value: &mut T,
        v: &impl Validate,
    ) -> Result<(), FieldError> {
        self.write_target(value);
        self.finish_validation(v.check(value.expose()))
    }

    fn write_target<T: TextTarget + ?Sized>(&mut self, value: &mut T) {
        if self.is_editing() && !self.redacted_snapshot {
            value.set(self.draft.text(), self.is_sensitive());
        }
        self.phase = EditPhase::Idle;
        self.redacted_snapshot = false;
        self.draft.zeroize();
    }

    pub fn cancel(&mut self) {
        self.phase = EditPhase::Idle;
        self.redacted_snapshot = false;
        self.draft.zeroize();
        if self.is_sensitive() {
            self.clear_error();
        }
    }

    pub fn blur(
        &mut self,
        value: &mut String,
        v: &impl Validate,
        p: BlurPolicy,
    ) -> Result<(), FieldError> {
        self.blur_target(value, v, p)
    }

    pub(crate) fn blur_target<T: TextTarget + ?Sized>(
        &mut self,
        value: &mut T,
        v: &impl Validate,
        p: BlurPolicy,
    ) -> Result<(), FieldError> {
        match p {
            BlurPolicy::CommitAndValidate => self.commit_target(value, v),
            BlurPolicy::Commit => {
                self.write_target(value);
                Ok(())
            }
            BlurPolicy::Cancel => {
                self.cancel();
                Ok(())
            }
            BlurPolicy::Keep => Ok(()),
        }
    }

    pub fn zeroize(&mut self) {
        self.draft.zeroize();
        self.clear_error();
    }

    fn apply(&mut self, a: EditAction<'_>) -> EditOutcome {
        self.draft.apply(a)
    }

    fn finish_validation(&mut self, result: Result<(), FieldError>) -> Result<(), FieldError> {
        self.clear_error();
        match result {
            Ok(()) => Ok(()),
            Err(error) if self.is_sensitive() => {
                discard_error(error);
                self.error = Some(ErrorState::sensitive());
                Err(FieldError::new("Invalid value"))
            }
            Err(error) => {
                self.error = Some(ErrorState::Plain(error.clone()));
                Err(error)
            }
        }
    }

    fn clear_error(&mut self) {
        if let Some(error) = self.error.take() {
            error.discard();
        }
    }
}

pub fn discard_error(error: FieldError) {
    drop(error);
}

pub fn redacted_text(text: &str) -> String {
    let mut redacted = String::new();
    for (line, segment) in text.split('\n').enumerate() {
        if line > 0 {
            redacted.push('\n');
        }
        for _ in graphemes(segment) {
            redacted.push('•');
        }
    }
    redacted
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "placeholder visibility, editability, availability and pointer admission are independent borrowed configuration"
)]
pub struct TextInput<'a> {
    id: Id,
    value: Option<&'a str>,
    placeholder: Option<&'a str>,
    placeholder_while_editing: bool,
    validate: Option<&'a dyn Validate>,
    blur: BlurPolicy,
    focus_select: FocusSelect,
    typing_policy: crate::TypingPolicy,
    secret: Option<SecretPolicy>,
    read_only: bool,
    disabled: bool,
    pointer_enabled: bool,
    status: Status,
    ov: PartStyle<'a>,
    style_target: Option<(Family, Variant, Part)>,
    text_style_target: Option<(Family, Variant, Part)>,
    auxiliary_style_target: Option<(Family, Variant)>,
    embedded_cell: bool,
    excluded_commands: &'a [TextCmd],
}

#[derive(Clone, Copy)]
struct InputGeometry {
    text: Rect,
    gutter: Option<Rect>,
    leading: u16,
}

impl fmt::Debug for TextInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInput")
            .field("id", &self.id)
            .field("value", &self.value.map(|_| "[redacted]"))
            .field("placeholder", &self.placeholder)
            .field("blur", &self.blur)
            .field("focus_select", &self.focus_select)
            .field("typing_policy", &self.typing_policy)
            .field("secret", &self.secret)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("pointer_enabled", &self.pointer_enabled)
            .field("text_style_target", &self.text_style_target)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> TextInput<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::FIELD,
        Part::TEXT,
        Part::PLACEHOLDER,
        Part::MARKER,
        Part::GUTTER,
        Part::ICON,
    ];

    pub const fn new(id: Id) -> Self {
        TextInput {
            id,
            value: None,
            placeholder: None,
            placeholder_while_editing: false,
            validate: None,
            blur: BlurPolicy::CommitAndValidate,
            focus_select: FocusSelect::CaretEnd,
            typing_policy: crate::TypingPolicy::Focused,
            secret: None,
            read_only: false,
            disabled: false,
            pointer_enabled: true,
            status: Status::Ready,
            ov: PartStyle::new(),
            style_target: None,
            text_style_target: None,
            auxiliary_style_target: None,
            embedded_cell: false,
            excluded_commands: &[],
        }
    }

    #[must_use]
    pub const fn value(mut self, v: &'a str) -> Self {
        self.value = Some(v);
        self
    }

    #[must_use]
    pub const fn placeholder(mut self, s: &'a str) -> Self {
        self.placeholder = Some(s);
        self
    }

    #[must_use]
    pub const fn placeholder_while_editing(mut self, enabled: bool) -> Self {
        self.placeholder_while_editing = enabled;
        self
    }

    #[must_use]
    pub const fn validate(mut self, v: &'a dyn Validate) -> Self {
        self.validate = Some(v);
        self
    }

    #[must_use]
    pub const fn blur(mut self, p: BlurPolicy) -> Self {
        self.blur = p;
        self
    }

    #[must_use]
    pub const fn focus_select(mut self, policy: FocusSelect) -> Self {
        self.focus_select = policy;
        self
    }

    #[must_use]
    pub const fn typing_policy(mut self, policy: crate::TypingPolicy) -> Self {
        self.typing_policy = policy;
        self
    }

    #[must_use]
    pub const fn exclude_commands(mut self, commands: &'a [TextCmd]) -> Self {
        self.excluded_commands = commands;
        self
    }

    #[must_use]
    pub const fn secret(mut self, policy: SecretPolicy) -> Self {
        self.secret = Some(policy);
        self
    }

    pub const fn is_secret(&self) -> bool {
        self.secret.is_some()
    }

    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn pointer_enabled(mut self, enabled: bool) -> Self {
        self.pointer_enabled = enabled;
        self
    }

    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn style_target(mut self, family: Family, variant: Variant, part: Part) -> Self {
        self.style_target = Some((family, variant, part));
        self
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn text_style_target(mut self, family: Family, variant: Variant, part: Part) -> Self {
        self.text_style_target = Some((family, variant, part));
        self
    }

    #[must_use]
    pub const fn auxiliary_style_target(mut self, family: Family, variant: Variant) -> Self {
        self.auxiliary_style_target = Some((family, variant));
        self
    }

    #[must_use]
    pub const fn embedded_cell(mut self) -> Self {
        self.embedded_cell = true;
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }

    fn includes_command(&self, command: TextCmd) -> bool {
        !self.excluded_commands.contains(&command)
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        TextInput {
            id: self.id,
            value: self.value,
            placeholder: self.placeholder,
            placeholder_while_editing: self.placeholder_while_editing,
            validate: self.validate,
            blur: self.blur,
            focus_select: self.focus_select,
            typing_policy: self.typing_policy,
            secret: self.secret,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            pointer_enabled: self.pointer_enabled,
            status: self.status,
            ov: self.ov,
            style_target: self.style_target,
            text_style_target: self.text_style_target,
            auxiliary_style_target: self.auxiliary_style_target,
            embedded_cell: self.embedded_cell,
            excluded_commands: self.excluded_commands,
        }
    }

    fn validator(&self) -> Dyn<'_> {
        Dyn(self.validate.unwrap_or(&NoValidate))
    }

    fn geometry(&self, area: Rect) -> InputGeometry {
        let area = first_row(area);
        let leading = if self.embedded_cell { 0 } else { 2 };
        let trailing = if self.embedded_cell { 1 } else { 3 };
        InputGeometry {
            text: Rect {
                x: area.x.saturating_add(leading),
                y: area.y,
                width: area.width.saturating_sub(trailing),
                height: 1.min(area.height),
            },
            gutter: (!self.embedded_cell).then(|| cell_at(area, area.x)),
            leading,
        }
    }

    pub fn register_editor(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) {
        self.register_editor_area(ui, area, st);
    }

    fn register_editor_area(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) {
        if area.is_empty() {
            return;
        }
        let editing = st.is_editing();
        let focusability = self.form_focusability(false);
        let declared = if editing {
            StateFlags::EDITING
        } else {
            StateFlags::empty()
        };
        let mut derived = self.status.flags();
        if editing {
            derived |= StateFlags::EDITING;
        }
        if st.error.is_some() || matches!(self.status, Status::Error) {
            derived |= StateFlags::ERROR;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let runtime = ui
            .state(self.id)
            .difference(StateFlags::EDITING | StateFlags::SELECTED);
        let mut live = PartStyle::flags(runtime, derived);
        if self.disabled {
            live = live.difference(StateFlags::HOVERED);
        }

        if self.pointer_enabled {
            ui.register_editor(self.id, area, focusability, declared);
        } else {
            ui.register_keyboard_editor(self.id, area, focusability, declared);
        }
        termrock_runtime::author::publish_bindings_filtered(ui, self.id, live, BINDINGS, |cmd| {
            self.includes_command(cmd)
        });
        if let crate::TypingPolicy::Fallback { cursor } = self.typing_policy {
            ui.publish_typing_target(
                self.id,
                BINDINGS,
                |cmd| {
                    self.includes_command(cmd) && !matches!(cmd, TextCmd::Commit | TextCmd::Cancel)
                },
                cursor,
            );
        }
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut String,
    ) -> Response<TextAction> {
        self.update_target(cx, st, value)
    }

    pub fn update_in_form<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
        inherited_disabled: bool,
    ) -> Response<TextAction> {
        self.with_inherited_disabled(inherited_disabled)
            .update_target(cx, st, value)
    }

    pub fn commit_in_form<T: TextTarget + ?Sized>(
        &self,
        st: &mut TextInputState,
        value: &mut T,
    ) -> bool {
        if !st.is_editing() {
            return false;
        }
        let _ = st.commit_target(value, &self.validator());
        true
    }

    fn update_target<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
    ) -> Response<TextAction> {
        st.set_sensitive(self.secret.is_some() || value.is_sensitive());
        let mut acc = super::Acc::<TextAction>::new();
        let editable = self.editable();
        st.edit_menu_request = None;
        for it in cx.intents(self.id) {
            match it {
                Intent::FocusIn { via } => {
                    if editable {
                        let starts_edit = !st.is_editing();
                        st.begin(value.expose());
                        if starts_edit
                            && matches!(
                                via,
                                crate::intent::FocusVia::Keyboard
                                    | crate::intent::FocusVia::Programmatic
                                    | crate::intent::FocusVia::Restore
                            )
                        {
                            match self.focus_select {
                                FocusSelect::CaretStart => {
                                    let _ = st.apply(EditAction::Move(Motion::Home, Extend::No));
                                }
                                FocusSelect::CaretEnd => {}
                                FocusSelect::SelectAll => {
                                    st.select_all();
                                }
                            }
                        }
                    }
                }
                Intent::FocusOut { to } => {
                    if st.is_editing() {
                        if crate::edit::retain_focus_continuation(
                            cx,
                            self.id,
                            to,
                            &mut st.focus_continuation,
                        ) {
                            acc.consumed();
                        } else {
                            let policy = self.blur;
                            let _ = st.blur_target(value, &self.validator(), policy);
                            match policy {
                                BlurPolicy::CommitAndValidate | BlurPolicy::Commit => {
                                    acc.action(TextAction::Committed);
                                }
                                BlurPolicy::Cancel => acc.action(TextAction::Cancelled),
                                BlurPolicy::Keep => {}
                            }
                        }
                    }
                }
                Intent::Binding(action) if editable => {
                    if let Some(cmd) = Binding::command(BINDINGS, action) {
                        if st.is_editing() {
                            self.edit_command_target(st, value, cmd, &mut acc);
                        } else if cmd == TextCmd::Commit {
                            st.begin(value.expose());
                            acc.changed();
                        }
                    }
                }
                Intent::Key(k)
                    if !self.disabled
                        && (k.code == KeyCode::Menu
                            || (k.code == KeyCode::F(10) && k.mods.contains(SHIFT))) =>
                {
                    let anchor = cx.area(self.id).map_or(Position::new(0, 0), |area| {
                        let col = st.draft.cursor_pos().col;
                        let geometry = self.geometry(area);
                        Position::new(
                            geometry.text.x.saturating_add(
                                u16::try_from(col.saturating_sub(usize::from(st.draft.hscroll())))
                                    .unwrap_or(u16::MAX),
                            ),
                            area.y,
                        )
                    });
                    st.edit_menu_request = Some(EditMenuRequest {
                        owner: self.id,
                        part: PartRef::of(Part::TEXT),
                        anchor,
                    });
                    acc.consumed();
                }
                Intent::Key(k) if editable && st.is_editing() => {
                    if let Some(c) = k.bare_char() {
                        self.insert(st, c, &mut acc);
                    }
                }
                Intent::Paste(s) if editable && st.is_editing() => {
                    if st.apply(EditAction::Paste(s)).changed() {
                        self.live_validate(st);
                        acc.action(TextAction::Changed);
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase: Phase::Press,
                    part,
                    local,
                    ..
                } if part.item.is_none()
                    && (editable || (self.read_only && !self.disabled))
                    && self.pointer_enabled =>
                {
                    if !st.is_editing() {
                        st.begin(value.expose());
                    }
                    let leading = self.geometry(cx.area(self.id).unwrap_or_default()).leading;
                    let col = usize::from(local.x.saturating_sub(leading))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    st.pointer_anchor = Some(st.draft.pointer_select(
                        0,
                        col,
                        cx.pointer_press_count(self.id, part).unwrap_or(1),
                    ));
                    acc.changed();
                }
                Intent::Pointer {
                    phase: Phase::DragStart | Phase::Drag,
                    part,
                    local,
                    ..
                } if part.item.is_none() && st.is_editing() && self.pointer_enabled => {
                    if let (Some(anchor), Some(area)) = (st.pointer_anchor, cx.area(self.id)) {
                        let geometry = self.geometry(area);
                        let col = usize::from(local.x.saturating_sub(geometry.leading));
                        let _ = st.select_pointer_range(anchor, col, geometry.text.width);
                        acc.changed();
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase: Phase::Release | Phase::DragEnd,
                    part,
                    ..
                } if part.item.is_none() => {
                    st.pointer_anchor = None;
                    acc.consumed();
                }
                Intent::Pointer {
                    phase: Phase::Secondary,
                    part,
                    pos,
                    local,
                    ..
                } if part.item.is_none() && !self.disabled => {
                    if !st.is_editing() {
                        st.begin(value.expose());
                    }
                    let leading = self.geometry(cx.area(self.id).unwrap_or_default()).leading;
                    let col = usize::from(local.x.saturating_sub(leading))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    let changed = st.draft.pointer_point(0, col);
                    cx.focus(self.id);
                    st.edit_menu_request = Some(EditMenuRequest {
                        owner: self.id,
                        part,
                        anchor: pos,
                    });
                    if changed {
                        acc.changed();
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer { .. } if self.pointer_enabled => acc.consumed(),
                Intent::Cancel if st.is_editing() => {
                    st.cancel();
                    acc.action(TextAction::Cancelled);
                }
                _ => {}
            }
        }
        if st.is_editing()
            && let Some(a) = cx.area(self.id)
        {
            let w = self.geometry(a).text.width;
            st.draft.scroll_into_view(w);
        }
        if let Some(opening) = st.focus_continuation
            && let Some(outcome) = crate::edit::continuation_result(cx, self.id, opening)
        {
            st.focus_continuation = None;
            let restored = outcome.restored_focus() == Some(self.id);
            if !(outcome.continued() && restored) && st.is_editing() {
                let policy = self.blur;
                let _ = st.blur_target(value, &self.validator(), policy);
                match policy {
                    BlurPolicy::CommitAndValidate | BlurPolicy::Commit => {
                        acc.action(TextAction::Committed);
                    }
                    BlurPolicy::Cancel => acc.action(TextAction::Cancelled),
                    BlurPolicy::Keep => {}
                }
                if restored && editable {
                    st.begin(value.expose());
                }
            }
            let _ = crate::edit::acknowledge_continuation_result(cx, self.id, opening);
        }
        acc.finish(self.id)
    }

    pub fn edit_capabilities<T: TextTarget + ?Sized>(
        &self,
        st: &TextInputState,
        value: &T,
    ) -> EditCapabilities {
        crate::edit::capabilities(
            &st.draft,
            value.expose(),
            st.is_editing(),
            self.disabled,
            self.editable(),
            self.secret.is_some() || st.is_sensitive(),
            value.is_sensitive(),
        )
    }

    pub fn execute_edit_command<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
        command: TextEditCommand,
    ) -> bool {
        st.set_sensitive(self.secret.is_some() || value.is_sensitive());
        if command == TextEditCommand::SelectAll && !st.is_editing() && !self.disabled {
            st.begin(value.expose());
        }
        if command == TextEditCommand::Paste
            && self.editable()
            && !st.is_editing()
            && cx.clipboard_text().is_some()
        {
            st.begin(value.expose());
        }
        let capabilities = self.edit_capabilities(st, value);
        let applied = crate::edit::execute(cx, &mut st.draft, capabilities, command);
        if applied && matches!(command, TextEditCommand::Cut | TextEditCommand::Paste) {
            self.live_validate(st);
        }
        if applied && let Some(area) = cx.area(self.id) {
            st.ensure_cursor_visible(self.geometry(area).text.width);
        }
        applied
    }

    fn live_validate(&self, st: &mut TextInputState) {
        if st.error.is_some() {
            let _ = st.finish_validation(self.validator().check(st.draft.text()));
        }
    }

    fn insert(&self, st: &mut TextInputState, c: char, acc: &mut super::Acc<TextAction>) {
        if st.apply(EditAction::Insert(c)).changed() {
            self.live_validate(st);
            acc.action(TextAction::Changed);
        } else {
            acc.consumed();
        }
    }

    fn edit_command_target<T: TextTarget + ?Sized>(
        &self,
        st: &mut TextInputState,
        value: &mut T,
        cmd: TextCmd,
        acc: &mut super::Acc<TextAction>,
    ) {
        match cmd {
            TextCmd::Cancel => {
                st.cancel();
                acc.action(TextAction::Cancelled);
            }
            TextCmd::Commit => {
                let _ = st.commit_target(value, &self.validator());
                acc.action(TextAction::Committed);
            }
            cmd => {
                let action = match cmd {
                    TextCmd::Move(m, e) => EditAction::Move(m, e),
                    TextCmd::Backspace => EditAction::Backspace,
                    TextCmd::Delete => EditAction::Delete,
                    TextCmd::DeleteWordLeft => EditAction::DeleteWordLeft,
                    TextCmd::DeleteToLineEnd => EditAction::DeleteToLineEnd,
                    TextCmd::DeleteToLineStart => EditAction::DeleteToLineStart,
                    TextCmd::SelectAll => EditAction::SelectAll,

                    TextCmd::Newline
                    | TextCmd::PageUp
                    | TextCmd::PageDown
                    | TextCmd::Cancel
                    | TextCmd::Commit => EditAction::ClearSelection,
                };
                match st.apply(action) {
                    EditOutcome::Changed => {
                        self.live_validate(st);
                        acc.action(TextAction::Changed);
                    }
                    EditOutcome::Moved => acc.changed(),
                    EditOutcome::Ignored | EditOutcome::Rejected => acc.consumed(),
                }
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over gutter, text window, marker and cursor"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let editing = st.is_editing();
        let validation_error = st.error.is_some();
        let status_error = matches!(self.status, Status::Error);
        let error = validation_error || status_error;
        let mut derived = self.status.flags();
        if editing {
            derived |= StateFlags::EDITING;
        }
        if error {
            derived |= StateFlags::ERROR;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let runtime = ui
            .state(self.id)
            .difference(StateFlags::EDITING | StateFlags::SELECTED);
        let mut live = PartStyle::flags(runtime, derived);
        if self.disabled {
            live = live.difference(StateFlags::HOVERED);
        }

        let busy = matches!(self.status, Status::Busy | Status::Loading);
        let geometry = self.geometry(area);
        let inner = geometry.text;
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        self.register_editor_area(ui, area, st);
        let owns_cursor = live.contains(StateFlags::FOCUSED)
            || (editing
                && matches!(
                    self.typing_policy,
                    crate::TypingPolicy::Fallback { cursor: true }
                ));
        let ov = self.ov;
        let id = self.id;
        let style_target = self.style_target;
        let text_style_target = self.text_style_target;
        let auxiliary_style_target = self.auxiliary_style_target;
        let style = |ui: &mut Ui<'_>, part: Part| {
            let base_target = style_target
                .filter(|_| matches!(part, Part::FIELD | Part::TEXT))
                .or_else(|| {
                    auxiliary_style_target
                        .filter(|_| matches!(part, Part::GUTTER | Part::PLACEHOLDER))
                        .map(|(family, variant)| (family, variant, part))
                })
                .unwrap_or((Family::INPUT, Variant::DEFAULT, part));
            let Some((family, variant, target_part)) =
                text_style_target.filter(|_| part == Part::TEXT)
            else {
                let local_patch = ov.part_patch(part);
                let resolved = match local_patch {
                    Some(patch) => {
                        ui.style_patched(base_target.0, base_target.1, base_target.2, live, &patch)
                    }
                    None => ui.style(base_target.0, base_target.1, base_target.2, live),
                };
                ov.note(
                    ui,
                    id,
                    base_target.0,
                    base_target.1,
                    base_target.2,
                    resolved,
                );
                return resolved;
            };

            let local_patch = ov.part_patch(part);
            let mut resolved = match local_patch {
                Some(patch) => {
                    ui.style_patched(base_target.0, base_target.1, base_target.2, live, &patch)
                }
                None => ui.style(base_target.0, base_target.1, base_target.2, live),
            };
            let base_underline = resolved
                .style
                .underline_role()
                .map(|(role, _)| role)
                .or_else(|| resolved.style.as_style().underline_color.map(Role::Custom));
            let defaults =
                StyleDefaults::new(base_underline.map_or_else(StylePatch::new, |role| {
                    StylePatch::new().set_underline(role)
                }));
            let decoration =
                ui.style_defaults_with_explicit(family, variant, target_part, live, defaults, None);
            let mut decoration_style = Style::new();
            let explicit_underline = local_patch.is_some_and(|patch| patch.underline.speaks());
            if explicit_underline {
                if resolved.style.underline_cleared() {
                    decoration_style.underline_color = Some(ratatui_core::style::Color::Reset);
                }
            } else {
                let target_style = decoration.resolved.style;
                if target_style.underline_cleared() {
                    decoration_style.underline_color = Some(ratatui_core::style::Color::Reset);
                } else if target_style.underline_role().is_some() {
                    decoration_style.underline_color = target_style.as_style().underline_color;
                }
            }
            let explicit_underline_modifier = local_patch.is_some_and(|patch| {
                patch.add.contains(Modifier::UNDERLINED)
                    || patch.remove.contains(Modifier::UNDERLINED)
            });
            if !explicit_underline_modifier {
                decoration_style.add_modifier = decoration
                    .resolved
                    .style
                    .as_style()
                    .add_modifier
                    .intersection(Modifier::UNDERLINED);
                decoration_style.sub_modifier = decoration
                    .resolved
                    .style
                    .as_style()
                    .sub_modifier
                    .intersection(Modifier::UNDERLINED);
            }
            resolved.style = resolved.style.patch(PaintStyle::from(decoration_style));
            let mut styled = decoration.resolved;
            styled.style = resolved.style;
            ov.note(ui, id, family, variant, target_part, styled);
            resolved = styled;
            resolved
        };
        let field = style(ui, Part::FIELD);
        ui.fill(area, field.style);
        if let Some(gutter_cell) = geometry.gutter {
            if let Some(f) = ov.slot_for(Part::GUTTER) {
                f(ui, gutter_cell);
            } else {
                let g = style(ui, Part::GUTTER);
                match g.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(gutter_cell, glyph, g.style);
                    }
                    Slot::Inherit | Slot::Clear => ui.fill(gutter_cell, g.style),
                }
            }
        }
        let shown = if editing {
            st.draft.text()
        } else {
            self.value.unwrap_or("")
        };
        if !shown.is_empty() || editing {
            if inner.width > 0 {
                let ts = style(ui, Part::TEXT);
                let cursor_col = if editing {
                    st.draft.cursor_pos().col
                } else {
                    0
                };
                let hs = if editing {
                    let w = usize::from(inner.width);
                    let hs = usize::from(st.draft.hscroll());
                    if cursor_col < hs {
                        cursor_col
                    } else if cursor_col >= hs.saturating_add(w) {
                        cursor_col.saturating_add(1).saturating_sub(w)
                    } else {
                        hs
                    }
                } else {
                    0
                };
                let secret_policy = self
                    .secret
                    .or_else(|| st.is_sensitive().then_some(SecretPolicy::default()));
                let total = match secret_policy {
                    Some(_) => graphemes(shown).count(),
                    None => usize::from(width(shown)),
                };
                let mut run = inner;
                if hs > 0 && !self.embedded_cell {
                    let used = ui.glyph(run, GlyphRole::Ellipsis, ts.style);
                    run = shift(run, used);
                }
                let skip = if hs > 0 && !self.embedded_cell {
                    hs.saturating_add(1)
                } else {
                    hs
                };
                let overflow_right =
                    !self.embedded_cell && total > hs.saturating_add(usize::from(inner.width));
                if overflow_right {
                    run.width = run.width.saturating_sub(1);
                }
                if let Some(policy) = secret_policy {
                    paint_masked(ui, run, shown, skip, editing, policy, ts.style);
                } else {
                    let start = byte_at_col(shown, skip);
                    ui.paint_str(run, shown.get(start..).unwrap_or(""), ts.style);
                    if let Some(selection) = st.selection() {
                        let start_col =
                            usize::from(width(shown.get(..selection.start).unwrap_or("")));
                        let end_col = usize::from(width(shown.get(..selection.end).unwrap_or("")));
                        let visible_start = start_col.max(skip);
                        let visible_limit = usize::from(inner.width)
                            .saturating_add(hs)
                            .saturating_sub(usize::from(overflow_right));
                        let visible_end = end_col.min(visible_limit);
                        if visible_start < visible_end {
                            let selected_area = Rect {
                                x: inner.x.saturating_add(
                                    visible_start.saturating_sub(hs).min(usize::from(u16::MAX))
                                        as u16,
                                ),
                                y: inner.y,
                                width: visible_end
                                    .saturating_sub(visible_start)
                                    .min(usize::from(u16::MAX))
                                    as u16,
                                height: 1,
                            };
                            let (family, variant, part) =
                                (Family::INPUT, Variant::DEFAULT, Part::ROW);
                            let selected = ov.style(
                                ui,
                                id,
                                family,
                                variant,
                                part,
                                live | StateFlags::SELECTED,
                            );
                            ui.fill(selected_area, selected.style);
                            let selection_start = byte_at_col(shown, visible_start);
                            let selection_end = byte_at_col(shown, visible_end);
                            ui.paint_str(
                                selected_area,
                                shown.get(selection_start..selection_end).unwrap_or(""),
                                selected.style,
                            );
                        }
                    }
                }
                if overflow_right {
                    let last = cell_at(inner, inner.right().saturating_sub(1));
                    ui.glyph(last, GlyphRole::Ellipsis, ts.style);
                }
                if owns_cursor && self.editable() {
                    let cursor_col = if editing {
                        cursor_col
                    } else {
                        usize::from(width(shown))
                    };
                    let cx = inner
                        .x
                        .saturating_add(
                            cursor_col.saturating_sub(hs).min(usize::from(u16::MAX)) as u16
                        )
                        .min(inner.right());
                    let pos = Position::new(cx, inner.y);
                    if area.contains(pos) {
                        if matches!(
                            self.typing_policy,
                            crate::TypingPolicy::Fallback { cursor: true }
                        ) {
                            ui.offer_typing_cursor(self.id, pos);
                        } else {
                            ui.set_bar_cursor(self.id, pos);
                        }
                    }
                }
            } else if owns_cursor && self.editable() && !self.embedded_cell {
                let pos = Position::new(inner.x, inner.y);
                if area.contains(pos) {
                    if matches!(
                        self.typing_policy,
                        crate::TypingPolicy::Fallback { cursor: true }
                    ) {
                        ui.offer_typing_cursor(self.id, pos);
                    } else {
                        ui.set_bar_cursor(self.id, pos);
                    }
                }
            }
        }
        if let Some(p) = self
            .placeholder
            .filter(|_| shown.is_empty() && (!editing || self.placeholder_while_editing))
        {
            let ps = style(ui, Part::PLACEHOLDER);
            match ov.slot_for(Part::PLACEHOLDER) {
                Some(f) => f(ui, inner),
                None => {
                    ui.paint_str(inner, p, ps.style);
                }
            }
        }
        let readiness_cell = cell_at(area, area.right().saturating_sub(1));
        if validation_error {
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, readiness_cell);
            } else {
                let ms = style(ui, Part::MARKER);
                if let Slot::Set(g) = ms.glyph {
                    ui.glyph(readiness_cell, g, ms.style);
                }
            }
        } else if status_error {
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, readiness_cell);
            } else {
                let is = style(ui, Part::ICON);
                match is.glyph {
                    Slot::Set(g) => {
                        ui.glyph(readiness_cell, g, is.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(readiness_cell, GlyphRole::Error, is.style);
                    }
                    Slot::Clear => ui.fill(readiness_cell, is.style),
                }
            }
        } else if busy {
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, readiness_cell);
            } else {
                let is = style(ui, Part::ICON);
                let frames = ui.design().motion.spinner_frames;
                let frame = frames.first().copied().unwrap_or("");
                ui.paint_str(readiness_cell, frame, is.style);
            }
        }
        area
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &TextInputState,
        value: &str,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .value(value)
            .draw(ui, area, st)
    }

    pub fn draw_secret_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &TextInputState,
        value: &Secret,
        inherited_disabled: bool,
    ) -> Rect {
        let control = self.with_inherited_disabled(inherited_disabled);
        let policy = control.secret.unwrap_or_default();
        control
            .value(value.expose())
            .secret(policy)
            .draw(ui, area, st)
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (8, 1),
            preferred: (30, 1),
        }
        .fit(c)
    }
}

fn paint_masked(
    ui: &mut Ui<'_>,
    run: Rect,
    shown: &str,
    skip: usize,
    editing: bool,
    policy: SecretPolicy,
    style: crate::theme::PaintStyle,
) {
    let total = graphemes(shown).count().saturating_sub(skip);
    let tail = if editing {
        0
    } else {
        policy.synthetic_tail.min(total)
    };
    let mut cell = CellUi::new(ui.reborrow(), run, style);
    if tail > 0 {
        cell.glyphs(policy.mask, total.saturating_sub(tail));
        let fp = crate::id::fnv1a(0xcbf2_9ce4_8422_2325, shown.as_bytes()).to_le_bytes();
        let mut buf = [0u8; 8];
        for (slot, byte) in buf.iter_mut().zip(fp) {
            let value = byte % 36;
            *slot = if value < 10 {
                b'0'.saturating_add(value)
            } else {
                b'a'.saturating_add(value.saturating_sub(10))
            };
        }
        cell.text(core::str::from_utf8(buf.get(..tail).unwrap_or(&[])).unwrap_or(""));
    } else {
        cell.glyphs(policy.mask, total);
    }
}

struct Dyn<'a>(&'a dyn Validate);

impl Validate for Dyn<'_> {
    fn check(&self, s: &str) -> Result<(), FieldError> {
        self.0.check(s)
    }
}

pub(super) fn byte_at_col(s: &str, col: usize) -> usize {
    let mut w = 0usize;
    for (i, g) in graphemes(s) {
        if w >= col {
            return i;
        }
        w = w.saturating_add(usize::from(width(g)));
    }
    s.len()
}

impl Bindings for TextInput<'_> {
    type Cmd = TextCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TextCmd>] {
        BINDINGS
    }
}

impl FieldControl for TextInput<'_> {
    type State = TextInputState;

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect {
        TextInput::draw(self, ui, area, st)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        TextInput::measure(self, ui, c)
    }
}
