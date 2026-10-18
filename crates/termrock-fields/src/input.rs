#![doc = r" `TextInput` and its explicit edit lifecycle (`COMPONENT_ARCHITECTURE.md`"]
#![doc = r" §15, §17.0 A7, Appendix A 4B)."]

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{PartStyle, SlotFn, cell_at, first_row, shift};
use crate::action::ActionKey;
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
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};
use termrock_runtime::Status;
use termrock_runtime::rowui::CellUi;

mod text_target {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for crate::secret::Secret {}
}

#[doc = r" A controlled text value accepted by the crate-internal form bridge."]
#[doc = r""]
#[doc = r" Sealing keeps the public standalone controls on `String` while allowing"]
#[doc = r" forms to edit `Secret` in place, without cloning it or widening the API."]
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

#[doc = r" What happens to an in-flight edit when focus leaves the control."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BlurPolicy {
    #[doc = r" Write the draft to the value and run the validator."]
    #[default]
    CommitAndValidate,
    #[doc = r" Write the draft to the value without validating."]
    Commit,
    #[doc = r" Drop the draft."]
    Cancel,
    #[doc = r" Keep editing; the draft survives until focus returns."]
    Keep,
}

#[doc = r" The edit lifecycle phase."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EditPhase {
    #[doc = r" Showing the controlled value."]
    #[default]
    Idle,
    #[doc = r" A draft is in flight."]
    Editing,
}

#[doc = r" What a text input reports."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAction {
    #[doc = r" The draft changed."]
    Changed,
    #[doc = r" The draft was written to the value."]
    Committed,
    #[doc = r" The draft was dropped."]
    Cancelled,
    #[doc = r" Reserved: focus should move forward (Tab is runtime focus policy and"]
    #[doc = r" never reaches the control; a form may emit this itself)."]
    MoveNext,
    #[doc = r" Reserved: focus should move backward."]
    MovePrev,
}

#[doc = r" The const-constructible commands of the edit keymap (the legacy"]
#[doc = r" `field_common::edit_key` table), shared by every text control."]
#[doc = r""]
#[doc = r" [`Newline`](TextCmd::Newline), [`PageUp`](TextCmd::PageUp) and"]
#[doc = r" [`PageDown`](TextCmd::PageDown) belong to the multi-line flavour of the"]
#[doc = r" table ([`TextArea`](crate::TextArea)); a single-line"]
#[doc = r" [`TextInput`] never binds them, exactly as the legacy"]
#[doc = r" table selected its arms on a `multiline` flag."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCmd {
    #[doc = r" Drop the draft (`Esc`)."]
    Cancel,
    #[doc = r" Write the draft (`Enter`); while idle, begin editing."]
    Commit,
    #[doc = r" A cursor motion."]
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
    #[doc = r" Insert a newline (multi-line controls only)."]
    Newline,
    #[doc = r" Move the cursor up one viewport (multi-line controls only)."]
    PageUp,
    #[doc = r" Move the cursor down one viewport (multi-line controls only)."]
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

#[doc = r" The in-flight editor with its sensitivity encoded in the variant."]
#[doc = r""]
#[doc = r" A secret editor is never represented as an untagged `String` plus a flag:"]
#[doc = r" every operation that can copy, compare or format the draft sees the secret"]
#[doc = r" variant explicitly."]
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

    #[doc = r" Create a redacted snapshot. A secret snapshot is intentionally not a"]
    #[doc = r" semantic continuation: committing it writes mask glyphs, never bytes"]
    #[doc = r" from the original draft."]
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

#[doc = r" A retained validation error tagged with the sensitivity of its source."]
#[doc = r" Sensitive errors carry no caller message or code."]
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

#[doc = r" Durable state of a [`TextInput`]: the in-flight draft, the phase and the"]
#[doc = r" last validation error. `Debug` redacts the draft. `Clone` makes a redacted"]
#[doc = r" snapshot for secret state, not a continuation that can commit the secret."]
#[derive(Default)]
pub struct TextInputState {
    draft: EditorDraft,
    phase: EditPhase,
    error: Option<ErrorState>,
    redacted_snapshot: bool,
    sensitivity: Option<bool>,
}

impl Clone for TextInputState {
    fn clone(&self) -> Self {
        TextInputState {
            draft: self.draft.clone_snapshot(),
            phase: self.phase,
            error: self.error.as_ref().map(ErrorState::clone_snapshot),
            redacted_snapshot: self.is_sensitive(),
            sensitivity: self.sensitivity,
        }
    }
}

impl PartialEq for TextInputState {
    fn eq(&self, other: &Self) -> bool {
        if self.is_sensitive() || other.is_sensitive() {
            self.is_sensitive() == other.is_sensitive()
                && self.phase == other.phase
                && self.error.as_ref().map(ErrorState::is_sensitive)
                    == other.error.as_ref().map(ErrorState::is_sensitive)
        } else {
            self.draft.same(&other.draft)
                && self.phase == other.phase
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
            .field("sensitive", &self.is_sensitive())
            .finish()
    }
}

impl TextInputState {
    #[doc = r" Construct state for direct editing of a secret `String`."]
    #[doc = r""]
    #[doc = r" Use this instead of [`TextInputState::default`] when calling the"]
    #[doc = r" public lifecycle methods directly, without [`TextInput::update`]"]
    #[doc = r" establishing sensitivity first. The returned state is sensitive before"]
    #[doc = r" [`TextInputState::begin`] can copy caller data into its draft."]
    #[must_use]
    pub fn sensitive() -> Self {
        let mut state = Self::default();
        state.set_sensitive(true);
        state
    }

    #[doc = r" Whether a draft is in flight."]
    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    #[doc = r" The phase."]
    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    #[doc = r" Return the current non-secret draft while editing."]
    #[doc = r""]
    #[doc = r" Secret drafts intentionally return `None`; callers that need to react"]
    #[doc = r" to a non-secret transient command can inspect this without exposing"]
    #[doc = r" credential material through a public state accessor."]
    pub fn draft_text(&self) -> Option<&str> {
        if self.is_sensitive() || !self.is_editing() {
            None
        } else {
            Some(self.draft.text())
        }
    }

    #[doc = r" Replace `replace_len` bytes before the draft cursor with `insertion`."]
    #[doc = r""]
    #[doc = r" Idle states refuse so a stale accept cannot rewrite a committed value."]
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

    #[doc = r" Compare the text the editor currently displays without exposing its draft."]
    #[doc = r" Redacted snapshots cannot authorize an action on the original secret."]
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

    #[doc = r" The last validation error."]
    #[doc = r""]
    #[doc = r" Before the first update reconciles the control's sensitivity, an"]
    #[doc = r" externally supplied error is exposed as `Invalid value`. A plain"]
    #[doc = r" control resolves that pending error during reconciliation and restores"]
    #[doc = r" its detail; a secret control discards the detail."]
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

    #[doc = r" Set (or clear) the error from an external / async validation."]
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

    #[doc = r" Begin an edit over `current` (a no-op while editing)."]
    pub fn begin(&mut self, current: &str) {
        if self.is_editing() {
            return;
        }
        self.redacted_snapshot = false;
        self.draft.begin_single(current);
        self.phase = EditPhase::Editing;
    }

    #[doc = r" Write the draft to `value`, end the edit and validate."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r" The validator's error; it is also recorded in the state."]
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

    #[doc = r" Drop the draft."]
    pub fn cancel(&mut self) {
        self.phase = EditPhase::Idle;
        self.redacted_snapshot = false;
        self.draft.zeroize();
        if self.is_sensitive() {
            self.clear_error();
        }
    }

    #[doc = r" Apply the blur policy."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r" The validator's error under `CommitAndValidate`."]
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

    #[doc = r" Overwrite the draft bytes."]
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

#[doc = r" A single-line text control with an explicit edit lifecycle."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `TextInput::new(id)`. The controlled value is passed per phase:"]
#[doc = r" `&mut String` to `update`, `.value(&str)` for `draw`."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" The caller owns the value and a [`TextInputState`] (draft, phase,"]
#[doc = r" error). The runtime owns focus, hover, the cursor write and paste"]
#[doc = r" routing. Controlled is the default (S4): the value changes only on"]
#[doc = r" commit."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.value(&str)` (draw), `.placeholder(&str)`, `.validate(&dyn Validate)`"]
#[doc = r" (`NoValidate`), `.blur(BlurPolicy)` (`CommitAndValidate`),"]
#[doc = r" `.secret(SecretPolicy)`, `.read_only(bool)`, `.disabled(bool)`,"]
#[doc = r" `.status(Status)`, `.patch`, `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::INPUT`, `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` from the runtime; `EDITING` is"]
#[doc = r" owned by the state and declared every frame; `ERROR` from the state's"]
#[doc = r" error or `.status(Error)`; `READ_ONLY`, `DISABLED`, `BUSY`, `LOADING`."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" `Changed` (the draft changed), `Committed` (written to the value),"]
#[doc = r" `Cancelled` (draft dropped), `MoveNext` / `MovePrev` (reserved)."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" `Focusable` (`FocusableReadOnly` / `Disabled`); swallows typing."]
#[doc = r" Focus arriving begins an edit; focus leaving applies the blur policy."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" The edit table (every state; while idle only `Enter` and typing apply):"]
#[doc = r" `Esc` Cancel, `Enter` Commit, `←`/`→` (`Shift` selects, `Ctrl`/`Alt`"]
#[doc = r" words), `Home`/`End` (`Shift`, `Ctrl` document), `Backspace`"]
#[doc = r" (`Ctrl`/`Alt` word), `Del`, `Ctrl+a/e` start/end, `Ctrl+u/k` delete to"]
#[doc = r" start/end, `Ctrl+w` delete word, `Ctrl+l` select all, `Alt+b/f` words."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::of(Part::CONTAINER)`: a press begins editing and places the"]
#[doc = r" cursor at the clicked column."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" One row: gutter, one-cell indent, the text run and one trailing readiness"]
#[doc = r" cell. `measure` is `(8…, 1)`; `draw` paints the first"]
#[doc = r" row of `area` and returns it; `0×0` registers nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `FIELD` (the row fill), `TEXT` (the value / draft), `PLACEHOLDER`,"]
#[doc = r" `MARKER` (the trailing validation glyph), `GUTTER` (the focus bar), `ICON`"]
#[doc = r" (the status error glyph or spinner, in the same trailing cell)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` on any part; `.slot` on exactly `GUTTER`,"]
#[doc = r" `MARKER`, `PLACEHOLDER` and `ICON`. `FIELD` and `TEXT` are not"]
#[doc = r" slot-addressable: the first is the field's own fill and the second is the"]
#[doc = r" edited text, which the caller owns through `.value` and the draft."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id`; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `TextInputCase` with `FOCUSABLE | EDITS | CURSOR | TYPES | SECRET |"]
#[doc = r" DISABLEABLE | REPORTS_STATUS`; `render::components::text_input::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" `draw` never commits, cancels or validates (it takes `&TextInputState`);"]
#[doc = r" a secret draft is masked while editing and never reaches `Debug`; the"]
#[doc = r" hardware cursor belongs to focus by default; an explicit fallback policy"]
#[doc = r" may publish the active draft as cursor owner without moving focus."]
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
    typing_policy: crate::TypingPolicy,
    secret: Option<SecretPolicy>,
    read_only: bool,
    disabled: bool,
    pointer_enabled: bool,
    status: Status,
    ov: PartStyle<'a>,
}

impl fmt::Debug for TextInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInput")
            .field("id", &self.id)
            .field("value", &self.value.map(|_| "[redacted]"))
            .field("placeholder", &self.placeholder)
            .field("blur", &self.blur)
            .field("typing_policy", &self.typing_policy)
            .field("secret", &self.secret)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("pointer_enabled", &self.pointer_enabled)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> TextInput<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::FIELD,
        Part::TEXT,
        Part::PLACEHOLDER,
        Part::MARKER,
        Part::GUTTER,
        Part::ICON,
    ];

    #[doc = r" A text input."]
    pub const fn new(id: Id) -> Self {
        TextInput {
            id,
            value: None,
            placeholder: None,
            placeholder_while_editing: false,
            validate: None,
            blur: BlurPolicy::CommitAndValidate,
            typing_policy: crate::TypingPolicy::Focused,
            secret: None,
            read_only: false,
            disabled: false,
            pointer_enabled: true,
            status: Status::Ready,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The controlled value, for `draw`."]
    #[must_use]
    pub const fn value(mut self, v: &'a str) -> Self {
        self.value = Some(v);
        self
    }

    #[doc = r" Placeholder shown while the value is empty and no edit is in flight."]
    #[must_use]
    pub const fn placeholder(mut self, s: &'a str) -> Self {
        self.placeholder = Some(s);
        self
    }

    #[doc = r" Also show the placeholder for an empty active draft, preserving its cursor."]
    #[doc = r" Disabled by default; this changes painting only, not editing ownership."]
    #[must_use]
    pub const fn placeholder_while_editing(mut self, enabled: bool) -> Self {
        self.placeholder_while_editing = enabled;
        self
    }

    #[doc = r" The validator run on commit."]
    #[must_use]
    pub const fn validate(mut self, v: &'a dyn Validate) -> Self {
        self.validate = Some(v);
        self
    }

    #[doc = r" What focus loss does to a draft."]
    #[must_use]
    pub const fn blur(mut self, p: BlurPolicy) -> Self {
        self.blur = p;
        self
    }

    #[doc = r" Publish an alternate typing target without moving navigation focus."]
    #[doc = r" Fallback routing requires an active editable draft and compatible"]
    #[doc = r" presented geometry. Commit and cancel remain primary-focus actions."]
    #[must_use]
    pub const fn typing_policy(mut self, policy: crate::TypingPolicy) -> Self {
        self.typing_policy = policy;
        self
    }

    #[doc = r" Mask the text (§15): every grapheme paints the policy's mask glyph."]
    #[must_use]
    pub const fn secret(mut self, policy: SecretPolicy) -> Self {
        self.secret = Some(policy);
        self
    }

    pub const fn is_secret(&self) -> bool {
        self.secret.is_some()
    }

    #[doc = r" Read-only: stays in the ring, never edits."]
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[doc = r" Disabled: registered, never reachable."]
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[doc = r" Allow pointer activation and caret positioning (enabled by default)."]
    #[doc = r" Disabling pointer input preserves keyboard editing, focus and geometry."]
    #[must_use]
    pub const fn pointer_enabled(mut self, enabled: bool) -> Self {
        self.pointer_enabled = enabled;
        self
    }

    #[doc = r" Data readiness."]
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        TextInput {
            id: self.id,
            value: self.value,
            placeholder: self.placeholder,
            placeholder_while_editing: self.placeholder_while_editing,
            validate: self.validate,
            blur: self.blur,
            typing_policy: self.typing_policy,
            secret: self.secret,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            pointer_enabled: self.pointer_enabled,
            status: self.status,
            ov: self.ov,
        }
    }

    fn validator(&self) -> Dyn<'_> {
        Dyn(self.validate.unwrap_or(&NoValidate))
    }

    #[doc = r" Columns between the gutter indent and the always-reserved trailing cell."]
    const fn inner_width(area_width: u16) -> u16 {
        area_width.saturating_sub(3)
    }

    #[doc = r" The update phase: drains this control's intents and drives the edit"]
    #[doc = r" lifecycle. The controlled `value` is written on commit only."]
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut String,
    ) -> Response<TextAction> {
        self.update_target(cx, st, value)
    }

    #[doc = r" Form bridge with inherited disabled state. The configured component"]
    #[doc = r" remains the source of every other prop."]
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
        for it in cx.intents(self.id) {
            match it {
                Intent::FocusIn { .. } => {
                    if editable {
                        st.begin(value.expose());
                    }
                }
                Intent::FocusOut { .. } => {
                    if st.is_editing() {
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
                    phase: Phase::Press | Phase::Click,
                    local,
                    ..
                } if editable && self.pointer_enabled => {
                    if !st.is_editing() {
                        st.begin(value.expose());
                    }
                    let col = usize::from(local.x.saturating_sub(2))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    st.draft.set_cursor_line_col(0, col);
                    acc.changed();
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
            let w = Self::inner_width(a.width);
            st.draft.scroll_into_view(w);
        }
        acc.finish(self.id)
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

    #[doc = r" The draw phase: the field row, the placeholder or the text, the"]
    #[doc = r" error marker and the cursor request."]
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
        let focusability = if self.disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        let declared = if editing {
            StateFlags::EDITING
        } else {
            StateFlags::empty()
        };

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
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y,
            width: Self::inner_width(area.width),
            height: 1,
        };
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        if self.pointer_enabled {
            ui.register_editor(self.id, area, focusability, declared);
        } else {
            ui.register_keyboard_editor(self.id, area, focusability, declared);
        }
        ui.publish_bindings(self.id, live, BINDINGS);
        if let crate::TypingPolicy::Fallback { cursor } = self.typing_policy {
            ui.publish_typing_target(
                self.id,
                BINDINGS,
                |cmd| !matches!(cmd, TextCmd::Commit | TextCmd::Cancel),
                cursor,
            );
        }
        let owns_cursor = live.contains(StateFlags::FOCUSED)
            || (editing
                && matches!(
                    self.typing_policy,
                    crate::TypingPolicy::Fallback { cursor: true }
                ));
        let ov = self.ov;
        let id = self.id;
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, id, Family::INPUT, Variant::DEFAULT, part, live)
        };
        let field = style(ui, Part::FIELD);
        ui.fill(area, field.style);
        let gutter_cell = cell_at(area, area.x);
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
                if hs > 0 {
                    let used = ui.glyph(run, GlyphRole::Ellipsis, ts.style);
                    run = shift(run, used);
                }
                let skip = if hs > 0 { hs.saturating_add(1) } else { 0 };
                let overflow_right = total > hs.saturating_add(usize::from(inner.width));
                if overflow_right {
                    run.width = run.width.saturating_sub(1);
                }
                if let Some(policy) = secret_policy {
                    paint_masked(ui, run, shown, skip, editing, policy, ts.style);
                } else {
                    let start = byte_at_col(shown, skip);
                    ui.paint_str(run, shown.get(start..).unwrap_or(""), ts.style);
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
                    if matches!(
                        self.typing_policy,
                        crate::TypingPolicy::Fallback { cursor: true }
                    ) {
                        ui.offer_typing_cursor(self.id, Position::new(cx, inner.y));
                    } else {
                        ui.set_cursor(self.id, Position::new(cx, inner.y));
                    }
                }
            } else if owns_cursor && self.editable() {
                if matches!(
                    self.typing_policy,
                    crate::TypingPolicy::Fallback { cursor: true }
                ) {
                    ui.offer_typing_cursor(self.id, Position::new(inner.x, inner.y));
                } else {
                    ui.set_cursor(self.id, Position::new(inner.x, inner.y));
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

    #[doc = r" The natural size: one row, eight columns minimum, thirty preferred."]
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

#[doc = r" A borrowed validator behind the blanket-impl bound."]
struct Dyn<'a>(&'a dyn Validate);

impl Validate for Dyn<'_> {
    fn check(&self, s: &str) -> Result<(), FieldError> {
        self.0.check(s)
    }
}

#[doc = r" Byte offset of display column `col` in `s` (the whole length past the end)."]
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::{Input, Key, KeyCode, KeyModifiers};
    use crate::runtime::App;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const ID: Id = Id::root("input.tests");

    struct SecretInputApp {
        state: TextInputState,
        value: String,
    }

    impl App for SecretInputApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextInput::new(ID)
                .secret(SecretPolicy::default())
                .update(cx, &mut self.state, &mut self.value)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextInput::new(ID)
                .secret(SecretPolicy::default())
                .value(&self.value)
                .draw(ui, SCREEN, &self.state);
        }
    }

    fn secret_state() -> TextInputState {
        let mut runtime = Runtime::new(
            SecretInputApp {
                state: TextInputState::default(),
                value: "hunter2".to_owned(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::NONE,
            }),
        );
        runtime.app().state.clone()
    }

    #[test]
    fn sensitive_constructor_makes_direct_lifecycle_safe() {
        const SECRET: &str = "hunter2";
        let mut state = TextInputState::sensitive();
        assert!(state.is_sensitive());

        state.begin(SECRET);
        assert!(!format!("{state:?}").contains(SECRET));

        let mut copy = state.clone();
        let mut copied_value = String::new();
        copy.commit(&mut copied_value, &NoValidate)
            .expect("a redacted snapshot must not fail validation");
        assert!(copied_value.is_empty());

        let mut value = String::new();
        state
            .commit(&mut value, &NoValidate)
            .expect("the sensitive draft must commit");
        assert_eq!(value, SECRET);
    }

    fn draw_status(status: Option<Status>) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        let state = TextInputState::default();
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                let mut input = TextInput::new(ID).value("value");
                if let Some(status) = status {
                    input = input.status(status);
                }
                input.draw(ui, area, &state);
            })
            .commit_presented();
        buffer
    }

    fn rule(s: &str) -> Result<(), FieldError> {
        if s.contains('@') {
            Ok(())
        } else {
            Err(FieldError::new("Enter a valid address"))
        }
    }

    #[doc = r" §16.1: `begin` snapshots the controlled value into the draft, and the"]
    #[doc = r" value itself does not move until a commit."]
    #[test]
    fn begin_snapshots_the_value() {
        let mut st = TextInputState::default();
        let value = "hello".to_owned();
        st.begin(&value);
        assert!(st.is_editing());
        assert_eq!(st.phase(), EditPhase::Editing);
        assert_eq!(st.draft.text(), "hello");
        assert_eq!(st.apply(EditAction::Insert('!')), EditOutcome::Changed);
        assert_eq!(st.draft.text(), "hello!");
        assert_eq!(value, "hello", "the value moves only on commit");

        st.begin("other");
        assert_eq!(st.draft.text(), "hello!");
    }

    #[test]
    fn splice_completion_replaces_the_bytes_before_the_cursor() {
        let mut st = TextInputState::default();
        assert!(!st.splice_completion(5, "world"));
        st.begin("hello");
        assert!(st.splice_completion(5, "world"));
        assert_eq!(st.draft_text(), Some("world"));
    }

    #[doc = r" §16.1: commit is the only writer of the controlled value."]
    #[test]
    fn commit_writes_the_controlled_value() {
        let mut st = TextInputState::default();
        let mut value = "a".to_owned();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('@'));
        assert!(st.commit(&mut value, &rule).is_ok());
        assert_eq!(value, "a@");
        assert!(!st.is_editing(), "commit ends the edit");
        assert_eq!(st.draft.text(), "", "the draft is zeroized on commit");

        assert!(st.commit(&mut value, &rule).is_ok());
        assert_eq!(value, "a@");
    }

    #[doc = r" §16.1: one commit runs the validator exactly once, over the value it"]
    #[doc = r" just wrote."]
    #[test]
    fn commit_runs_validation_once() {
        use core::cell::Cell;
        let calls = Cell::new(0usize);
        let seen = Cell::new(String::new());
        let counting = |s: &str| {
            calls.set(calls.get().saturating_add(1));
            seen.set(s.to_owned());
            rule(s)
        };
        let mut st = TextInputState::default();
        let mut value = "a".to_owned();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('@'));
        assert!(st.commit(&mut value, &counting).is_ok());
        assert_eq!(calls.get(), 1);
        assert_eq!(seen.take(), "a@", "the validator sees the committed value");
        assert!(st.error().is_none());
    }

    #[test]
    fn cancel_restores_the_snapshot() {
        let mut st = TextInputState::default();
        let mut value = "hello".to_owned();
        st.begin(&value);
        assert!(st.is_editing());
        assert_eq!(st.apply(EditAction::Insert('!')), EditOutcome::Changed);
        st.cancel();
        assert!(!st.is_editing());
        assert_eq!(value, "hello");
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('!'));
        assert!(st.commit(&mut value, &rule).is_err());
        assert_eq!(value, "hello!");
        assert!(st.error().is_some());
        assert!(!format!("{st:?}").contains("hello"));
    }

    #[doc = r" §16.1: the default policy writes the draft **and** validates it."]
    #[test]
    fn blur_commit_and_validate_policy() {
        let mut value = "a".to_owned();
        let mut st = TextInputState::default();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('@'));
        assert!(
            st.blur(&mut value, &rule, BlurPolicy::CommitAndValidate)
                .is_ok()
        );
        assert_eq!(value, "a@");
        assert!(st.error().is_none());
        st.begin(&value);
        let _ = st.apply(EditAction::Backspace);
        assert!(
            st.blur(&mut value, &rule, BlurPolicy::CommitAndValidate)
                .is_err(),
            "the value is written, then rejected"
        );
        assert_eq!(value, "a");
        assert!(st.error().is_some());
    }

    #[doc = r" §16.1: `Cancel` drops the draft and leaves the value untouched."]
    #[test]
    fn blur_cancel_policy() {
        let mut value = "a".to_owned();
        let mut st = TextInputState::default();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('b'));
        assert!(st.blur(&mut value, &rule, BlurPolicy::Cancel).is_ok());
        assert_eq!(value, "a");
        assert!(!st.is_editing());
        assert_eq!(st.draft.text(), "");
    }

    #[doc = r" §16.1: `Keep` leaves the edit in flight, so focus can return to it."]
    #[test]
    fn blur_keep_policy_leaves_the_draft() {
        let mut value = "a".to_owned();
        let mut st = TextInputState::default();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('b'));
        assert!(st.blur(&mut value, &rule, BlurPolicy::Keep).is_ok());
        assert!(st.is_editing(), "the draft survives the blur");
        assert_eq!(st.draft.text(), "ab");
        assert_eq!(value, "a");

        assert!(st.blur(&mut value, &rule, BlurPolicy::Commit).is_ok());
        assert_eq!(value, "ab");
    }

    #[test]
    fn secret_blur_keep_policy_leaves_the_draft() {
        let mut st = secret_state();
        st.cancel();
        let mut value = "a".to_owned();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('b'));
        assert!(st.blur(&mut value, &rule, BlurPolicy::Keep).is_ok());
        assert!(st.is_editing(), "secret Keep must preserve the draft");
        assert_eq!(st.draft.text(), "ab");
        assert_eq!(value, "a");
    }

    #[doc = r" §16.1: an error set from outside (an async / server-side check) is"]
    #[doc = r" state, not a derived value, so redrawing never clears it."]
    #[test]
    fn external_error_survives_a_redraw() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = TextInputState::default();
        st.set_sensitive(false);
        st.set_error(Some(FieldError::coded("Already taken", "dup")));
        for _ in 0..3 {
            rt.draw_scene(SCREEN, &mut buf, |ui, a| {
                TextInput::new(ID).value("ada").draw(ui, a, &st);
            })
            .commit_presented();
        }
        assert_eq!(st.error().map(|e| e.code), Some(Some("dup")));
        st.set_error(None);
        assert!(st.error().is_none());
    }

    #[test]
    fn sensitive_string_target_replaces_the_old_allocation() {
        let mut st = TextInputState::default();
        st.set_sensitive(true);
        st.begin("new secret");

        let mut value = String::with_capacity(1024);
        value.push_str("old secret");
        let old_capacity = value.capacity();

        st.commit(&mut value, &NoValidate)
            .expect("the replacement must not fail validation");

        assert_eq!(value, "new secret");
        assert!(
            value.capacity() < old_capacity,
            "sensitive replacement retained the old caller allocation"
        );
    }

    #[test]
    fn pending_error_is_masked_until_sensitivity_is_reconciled() {
        const DETAIL: &str = "secret validation detail";

        let mut plain = TextInputState::default();
        plain.set_error(Some(FieldError::new(DETAIL.to_owned())));
        assert!(matches!(plain.error, Some(ErrorState::Pending(_))));
        assert_eq!(
            plain.error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        plain.set_sensitive(false);
        assert!(matches!(plain.error, Some(ErrorState::Plain(_))));
        assert_eq!(
            plain.error().map(|error| error.message.as_ref()),
            Some(DETAIL)
        );

        let mut secret = TextInputState::default();
        secret.set_error(Some(FieldError::new(DETAIL.to_owned())));
        assert!(matches!(secret.error, Some(ErrorState::Pending(_))));
        assert_eq!(
            secret.error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        secret.set_sensitive(true);
        assert!(matches!(secret.error, Some(ErrorState::Sensitive)));
        assert_eq!(
            secret.error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
    }

    #[test]
    fn editing_style_comes_only_from_the_real_edit_state() {
        let render = |stale_runtime_editing: bool, real_editing: bool| {
            let theme = Theme::junie().override_family(Family::INPUT, |recipe| {
                recipe.part(Part::FIELD).when(
                    StateFlags::EDITING,
                    StylePatch::new().set_bg(crate::theme::Role::Danger),
                );
            });
            let mut runtime = Runtime::new(Stub::default(), theme);
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    if stale_runtime_editing {
                        ui.declare_state(ID, StateFlags::EDITING);
                    }
                })
                .commit_presented();
            let mut state = TextInputState::default();
            if real_editing {
                state.begin("value");
            }
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, area| {
                    TextInput::new(ID).value("value").draw(ui, area, &state);
                })
                .commit_presented();
            buffer
                .cell(Position::new(1, 0))
                .map(|cell| cell.bg)
                .unwrap_or_default()
        };
        let idle = render(false, false);
        assert_eq!(render(true, false), idle, "stale runtime EDITING leaked");
        assert_ne!(render(false, true), idle, "real edit state was not styled");
    }

    #[doc = r" §16.1 (P5): a masked field paints mask glyphs and a **synthetic**"]
    #[doc = r" tail derived from the fingerprint — never the real characters, and"]
    #[doc = r" never a `String` of them."]
    #[test]
    fn write_mask_is_synthetic() {
        const SECRET: &str = "hunter2";
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextInputState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            TextInput::new(ID)
                .secret(SecretPolicy::default())
                .value(SECRET)
                .draw(ui, a, &st);
        })
        .commit_presented();
        let mut row = String::new();
        for x in 0..SCREEN.width {
            if let Some(c) = buf.cell(Position::new(x, 0)) {
                row.push_str(c.symbol());
            }
        }
        assert!(
            !row.contains(SECRET),
            "the secret reached the buffer: {row}"
        );
        let policy = SecretPolicy::default();
        let mask = Theme::junie().design.glyphs.get(policy.mask);
        let painted = row.trim();
        let masked = SECRET.chars().count().saturating_sub(policy.synthetic_tail);
        assert!(
            painted.starts_with(&mask.repeat(masked)),
            "the mask run is wrong: {painted}"
        );

        let tail: String = painted.chars().skip(masked).collect();
        assert_eq!(tail.chars().count(), policy.synthetic_tail);
        assert_ne!(tail, "r2", "the real tail was painted");
        assert!(
            tail.chars().all(|c| c.is_ascii_alphanumeric()),
            "tail {tail} is not synthetic"
        );
    }

    #[test]
    fn sensitive_state_clone_keeps_shape_without_copying_draft() {
        let mut state = secret_state();
        state.begin("hunter2");
        let copy = state.clone();
        assert_eq!(copy.draft.text(), "•••••••");
        assert!(!copy.draft.text().contains("hunter2"));
    }

    #[test]
    fn sensitive_state_clone_cannot_commit_redacted_draft() {
        let mut state = TextInputState::default();
        state.set_sensitive(true);
        state.begin("hunter2");
        let _ = state.apply(EditAction::Insert('!'));

        let mut copy = state.clone();
        let mut value = String::new();
        copy.commit(&mut value, &NoValidate)
            .expect("a snapshot commit must not validate a redacted draft");
        assert!(value.is_empty());
        assert!(!copy.is_editing());
    }

    #[test]
    fn sensitive_state_equality_ignores_draft_contents() {
        let mut left = secret_state();
        left.cancel();
        left.begin("hunter2");
        let mut right = secret_state();
        right.cancel();
        right.begin("different");
        assert_eq!(left, right);
    }

    #[test]
    fn sensitive_state_masks_when_control_policy_is_removed() {
        const SECRET: &str = "hunter2";
        let mut state = secret_state();
        state.begin(SECRET);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                TextInput::new(ID).value(SECRET).draw(ui, area, &state);
            })
            .commit_presented();
        let frame: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        let mask = Theme::junie()
            .design
            .glyphs
            .get(SecretPolicy::default().mask);
        assert!(
            !frame.contains(SECRET),
            "the sensitive state reached the frame"
        );
        assert!(
            frame.matches(mask).count() >= SECRET.chars().count(),
            "the sensitive state did not paint its mask: {frame}"
        );
    }

    #[test]
    fn sensitive_validator_error_is_generic_and_not_retained() {
        const SECRET: &str = "hunter2";
        let validator = |value: &str| Err(FieldError::new(format!("invalid {value}")));
        let mut state = secret_state();
        state.begin(SECRET);
        state.set_error(Some(FieldError::new(SECRET)));
        assert_eq!(
            state.error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        let mut value = String::new();
        let error = state
            .commit(&mut value, &validator)
            .expect_err("the validator must reject the secret");
        assert_eq!(error.message, "Invalid value");
        assert_eq!(
            state.error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains(SECRET));
        state.zeroize();
        assert!(state.error().is_none());
        value.clear();
    }

    #[test]
    fn byte_offsets_follow_display_columns() {
        assert_eq!(byte_at_col("日本語", 2), 3);
        assert_eq!(byte_at_col("ab", 9), 2);
        assert_eq!(byte_at_col("ab", 0), 0);
    }

    #[test]
    fn one_trailing_lane_prioritizes_validation_over_status() {
        assert!(TextInput::PARTS.contains(&Part::ICON));
        assert_eq!(draw_status(None), draw_status(Some(Status::Ready)));
        let marker_calls = Cell::new(0usize);
        let icon_calls = Cell::new(0usize);
        let seen = Cell::new(None);
        let marker = |ui: &mut Ui<'_>, area: Rect| {
            marker_calls.set(marker_calls.get().saturating_add(1));
            let style = ui.surface_style();
            ui.paint_str(area, "M", style);
        };
        let icon = |ui: &mut Ui<'_>, area: Rect| {
            icon_calls.set(icon_calls.get().saturating_add(1));
            seen.set(Some(area));
            let style = ui.surface_style();
            ui.paint_str(area, "I", style);
        };
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let area = Rect::new(0, 0, 12, 1);
        let mut buffer = Buffer::empty(area);
        let mut st = TextInputState::default();
        st.set_error(Some(FieldError::new("invalid")));
        let patches = [
            (
                Part::MARKER,
                StylePatch::new().set_glyph(GlyphRole::WarningMark),
            ),
            (Part::ICON, StylePatch::new().set_glyph(GlyphRole::NewTab)),
        ];
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                TextInput::new(ID)
                    .value("value")
                    .status(Status::Error)
                    .patch_part(&patches)
                    .draw(ui, area, &st);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(11, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::WarningMark))
        );

        st.set_error(None);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                TextInput::new(ID)
                    .value("value")
                    .status(Status::Error)
                    .patch_part(&patches)
                    .draw(ui, area, &st);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(11, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::NewTab))
        );

        st.set_error(Some(FieldError::new("invalid")));
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                TextInput::new(ID)
                    .value("value")
                    .status(Status::Error)
                    .slot(Part::MARKER, &marker)
                    .draw(ui, area, &st);
            })
            .commit_presented();
        assert_eq!(marker_calls.get(), 1);
        st.set_error(None);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                TextInput::new(ID)
                    .value("value")
                    .status(Status::Error)
                    .slot(Part::ICON, &icon)
                    .draw(ui, area, &st);
            })
            .commit_presented();
        assert_eq!(icon_calls.get(), 1);
        assert_eq!(seen.get(), Some(Rect::new(11, 0, 1, 1)));
        assert_eq!(
            buffer
                .cell(Position::new(11, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("I")
        );
    }
}
