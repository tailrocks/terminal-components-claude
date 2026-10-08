//! `TextInput` and its explicit edit lifecycle (`COMPONENT_ARCHITECTURE.md`
//! §15, §17.0 A7, Appendix A 4B).

use core::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{PartStyle, SlotFn, cell_at, first_row, shift};
use crate::collection::{CellUi, Status};
use crate::edit_keys::{edit_action_of, edit_bindings};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::secret::{Secret, SecretPolicy, wipe_string};
use crate::text::measure::{byte_at_col, graphemes};
use crate::text::{EditAction, EditOutcome, Extend, Motion, TextEditorCore, width};
use crate::theme::{ColorLevel, Family, FgStep, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};

mod text_target {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for crate::secret::Secret {}
}

/// A controlled text value accepted by the crate-internal form bridge.
///
/// Sealing keeps the public standalone controls on `String` while allowing
/// forms to edit `Secret` in place, without cloning it or widening the API.
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

/// What happens to an in-flight edit when focus leaves the control.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BlurPolicy {
    /// Write the draft to the value and run the validator.
    #[default]
    CommitAndValidate,
    /// Write the draft to the value without validating.
    Commit,
    /// Drop the draft.
    Cancel,
    /// Keep editing; the draft survives until focus returns.
    Keep,
}

/// The edit lifecycle phase.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EditPhase {
    /// Showing the controlled value.
    #[default]
    Idle,
    /// A draft is in flight.
    Editing,
}

/// What a text input reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAction {
    /// The draft changed.
    Changed,
    /// The draft was written to the value.
    Committed,
    /// The draft was dropped.
    Cancelled,
    /// Reserved: focus should move forward (Tab is runtime focus policy and
    /// never reaches the control; a form may emit this itself).
    MoveNext,
    /// Reserved: focus should move backward.
    MovePrev,
}

/// The const-constructible commands of the edit keymap (the legacy
/// `field_common::edit_key` table), shared by every text control.
///
/// [`Newline`](TextCmd::Newline), [`PageUp`](TextCmd::PageUp) and
/// [`PageDown`](TextCmd::PageDown) belong to the multi-line flavour of the
/// table ([`TextArea`](crate::textarea::TextArea)); a single-line
/// [`TextInput`] never binds them, exactly as the legacy
/// table selected its arms on a `multiline` flag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCmd {
    /// Drop the draft (`Esc`).
    Cancel,
    /// Write the draft (`Enter`); while idle, begin editing.
    Commit,
    /// A cursor motion.
    Move(Motion, Extend),
    /// Delete the grapheme before the cursor.
    Backspace,
    /// Delete the grapheme after the cursor.
    Delete,
    /// Delete the word before the cursor.
    DeleteWordLeft,
    /// Delete to the line end.
    DeleteToLineEnd,
    /// Delete to the line start.
    DeleteToLineStart,
    /// Select everything.
    SelectAll,
    /// Insert a newline (multi-line controls only).
    Newline,
    /// Move the cursor up one viewport (multi-line controls only).
    PageUp,
    /// Move the cursor down one viewport (multi-line controls only).
    PageDown,
}

/// The in-flight editor with its sensitivity encoded in the variant.
///
/// A secret editor is never represented as an untagged `String` plus a flag:
/// every operation that can copy, compare or format the draft sees the secret
/// variant explicitly.
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
        if sensitive {
            // Escalation: a secret control attaches over a begun plain
            // draft — re-wrap the live text and caret instead of
            // dropping the edit. The transient is wiped immediately;
            // the old plain bytes are zeroized first.
            let text = self.text().to_owned();
            let caret = self.cursor_offset();
            self.zeroize();
            let mut editor = TextEditorCore::sensitive_single(&text);
            editor.set_cursor_offset(caret);
            wipe_string(text);
            *self = EditorDraft::Secret(editor);
        } else {
            self.zeroize();
            *self = EditorDraft::Plain(TextEditorCore::default());
        }
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

    pub(crate) fn set_cursor_offset(&mut self, at: usize) {
        match self {
            EditorDraft::Plain(editor) | EditorDraft::Secret(editor) => {
                editor.set_cursor_offset(at);
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

    /// Create a redacted snapshot. A secret snapshot is intentionally not a
    /// semantic continuation: committing it writes mask glyphs, never bytes
    /// from the original draft.
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

/// A retained validation error tagged with the sensitivity of its source.
/// Sensitive errors carry no caller message or code.
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

    pub fn clone_snapshot(&self) -> Self {
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

/// Durable state of a [`TextInput`]: the in-flight draft, the phase and the
/// last validation error. `Debug` redacts the draft. `Clone` makes a redacted
/// snapshot for secret state, not a continuation that can commit the secret.
#[derive(Default)]
pub struct TextInputState {
    draft: EditorDraft,
    phase: EditPhase,
    error: Option<ErrorState>,
    redacted_snapshot: bool,
    sensitivity: Option<bool>,
    /// FNV-1a of the caller value at `begin` (W08-07): conflict detection
    /// without retaining caller bytes. `None` while idle.
    base: Option<u64>,
}

/// FNV-1a fingerprint of a begin value: change detection that retains no
/// caller bytes (secret-safe by construction).
fn hash_str(s: &str) -> u64 {
    crate::id::fnv1a(0xcbf2_9ce4_8422_2325, s.as_bytes())
}

impl Clone for TextInputState {
    fn clone(&self) -> Self {
        TextInputState {
            draft: self.draft.clone_snapshot(),
            phase: self.phase,
            error: self.error.as_ref().map(ErrorState::clone_snapshot),
            redacted_snapshot: self.is_sensitive(),
            sensitivity: self.sensitivity,
            base: self.base,
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
    /// Construct state for direct editing of a secret `String`.
    ///
    /// Use this instead of [`TextInputState::default`] when calling the
    /// public lifecycle methods directly, without [`TextInput::update`]
    /// establishing sensitivity first. The returned state is sensitive before
    /// [`TextInputState::begin`] can copy caller data into its draft.
    #[must_use]
    pub fn sensitive() -> Self {
        let mut state = Self::default();
        state.set_sensitive(true);
        state
    }

    /// Whether a draft is in flight.
    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    /// The phase.
    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    /// Return the current non-secret draft while editing.
    ///
    /// Secret drafts intentionally return `None`; callers that need to react
    /// to a non-secret transient command can inspect this without exposing
    /// credential material through a public state accessor.
    pub fn draft_text(&self) -> Option<&str> {
        if self.is_sensitive() || !self.is_editing() {
            None
        } else {
            Some(self.draft.text())
        }
    }

    /// Byte offset of the draft cursor while editing; `None` when idle.
    ///
    /// Completion owners anchor trigger and accept ranges on this offset.
    /// Secret drafts report their real offset: position is not content.
    pub fn draft_cursor(&self) -> Option<usize> {
        self.is_editing().then(|| self.draft.cursor_offset())
    }

    /// Replace `replace_len` bytes before the draft cursor with `insertion`.
    ///
    /// Completion accept targets the live draft only: idle states refuse so a
    /// stale accept can never rewrite a committed value.
    pub fn splice_completion(&mut self, replace_len: usize, insertion: &str) -> bool {
        if !self.is_editing() {
            return false;
        }
        let end = self.draft.cursor_offset();
        let start = end.saturating_sub(replace_len);
        self.draft.select_range(start, end);
        self.draft.apply(EditAction::Paste(insertion)).changed()
    }

    /// Compare the text the editor currently displays without exposing its draft.
    /// Redacted snapshots cannot authorize an action on the original secret.
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

    /// The last validation error.
    ///
    /// Before the first update reconciles the control's sensitivity, an
    /// externally supplied error is exposed as `Invalid value`. A plain
    /// control resolves that pending error during reconciliation and restores
    /// its detail; a secret control discards the detail.
    pub const fn error(&self) -> Option<&FieldError> {
        match &self.error {
            Some(error) => Some(error.as_ref()),
            None => None,
        }
    }

    pub fn set_sensitive(&mut self, sensitive: bool) {
        let changed = self.is_sensitive() != sensitive;
        let first_attach = self.sensitivity.is_none();
        let pending_error = self
            .error
            .as_ref()
            .is_some_and(|error| matches!(error, ErrorState::Pending(_)));
        self.draft.set_sensitive(sensitive);
        self.sensitivity = Some(sensitive);
        if changed && !first_attach {
            // A genuine mid-edit policy flip drops the edit; a first
            // attach reconciles around the live draft instead.
            self.base = None;
            self.phase = EditPhase::Idle;
            self.redacted_snapshot = false;
            self.draft.zeroize();
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

    /// Set (or clear) the error from an external / async validation.
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

    /// Begin an edit over `current` (a no-op while editing).
    pub fn begin(&mut self, current: &str) {
        if self.is_editing() {
            return;
        }
        self.redacted_snapshot = false;
        self.base = Some(hash_str(current));
        self.draft.begin_single(current);
        self.phase = EditPhase::Editing;
    }

    /// Write the draft to `value`, end the edit and validate.
    ///
    /// # Errors
    /// The validator's error; it is also recorded in the state.
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

    /// Whether the caller-owned value moved under the active draft: the
    /// begin fingerprint no longer matches. The widget layer — the only
    /// place where the commit target is the same live binding the draft
    /// was seeded from — rebases on this; direct `commit` into a fresh
    /// target always writes.
    pub(crate) fn external_changed(&self, current: &str) -> bool {
        self.is_editing() && self.base.is_some_and(|base| base != hash_str(current))
    }

    fn write_target<T: TextTarget + ?Sized>(&mut self, value: &mut T) {
        if self.is_editing() && !self.redacted_snapshot {
            value.set(self.draft.text(), self.is_sensitive());
        }
        self.base = None;
        self.phase = EditPhase::Idle;
        self.redacted_snapshot = false;
        self.draft.zeroize();
    }

    /// Drop the draft.
    pub fn cancel(&mut self) {
        self.base = None;
        self.phase = EditPhase::Idle;
        self.redacted_snapshot = false;
        self.draft.zeroize();
        if self.is_sensitive() {
            self.clear_error();
        }
    }

    /// Apply the blur policy.
    ///
    /// # Errors
    /// The validator's error under `CommitAndValidate`.
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

    /// Overwrite the draft bytes.
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

/// A single-line text control with an explicit edit lifecycle.
///
/// ## Construction
/// `TextInput::new(id)`. The controlled value is passed per phase:
/// `&mut String` to `update`, `.value(&str)` for `draw`.
///
/// ## Ownership
/// The caller owns the value and a [`TextInputState`] (draft, phase,
/// error). The runtime owns focus, hover, the cursor write and paste
/// routing. Controlled is the default (S4): the value changes only on
/// commit.
///
/// ## Configuration
/// `.value(&str)` (draw), `.placeholder(&str)`, `.validate(&dyn Validate)`
/// (`NoValidate`), `.blur(BlurPolicy)` (`CommitAndValidate`),
/// `.secret(SecretPolicy)`, `.read_only(bool)`, `.disabled(bool)`,
/// `.status(Status)`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::INPUT`, `DEFAULT` only.
///
/// ## States
/// `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` from the runtime; `EDITING` is
/// owned by the state and declared every frame; `ERROR` from the state's
/// error or `.status(Error)`; `READ_ONLY`, `DISABLED`, `BUSY`, `LOADING`.
///
/// ## Actions
/// `Changed` (the draft changed), `Committed` (written to the value),
/// `Cancelled` (draft dropped), `MoveNext` / `MovePrev` (reserved).
///
/// ## Focus
/// `Focusable` (`FocusableReadOnly` / `Disabled`); swallows typing.
/// Focus arriving begins an edit; focus leaving applies the blur policy.
///
/// ## Keyboard
/// The edit table (every state; while idle only `Enter` and typing apply):
/// `Esc` Cancel, `Enter` Commit, `←`/`→` (`Shift` selects, `Ctrl`/`Alt`
/// words), `Home`/`End` (`Shift`, `Ctrl` document), `Backspace`
/// (`Ctrl`/`Alt` word), `Del`, `Ctrl+a/e` start/end, `Ctrl+u/k` delete to
/// start/end, `Ctrl+w` delete word, `Ctrl+l` select all, `Alt+b/f` words.
///
/// ## Mouse
/// `PartRef::of(Part::CONTAINER)`: a press begins editing and places the
/// cursor at the clicked column.
///
/// ## Layout
/// One row: gutter, one-cell indent, the text run and one trailing readiness
/// cell. `measure` is `(8…, 1)`; `draw` paints the first
/// row of `area` and returns it; `0×0` registers nothing (R5).
///
/// ## Parts
/// `FIELD` (the row fill), `TEXT` (the value / draft), `PLACEHOLDER`,
/// `MARKER` (the trailing validation glyph), `GUTTER` (the focus bar), `ICON`
/// (the status error glyph or spinner, in the same trailing cell).
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part; `.slot` on exactly `GUTTER`,
/// `MARKER`, `PLACEHOLDER` and `ICON`. `FIELD` and `TEXT` are not
/// slot-addressable: the first is the field's own fill and the second is the
/// edited text, which the caller owns through `.value` and the draft.
///
/// ## Identity
/// One `Id`; no items.
///
/// ## Testing
/// `TextInputCase` with `FOCUSABLE | EDITS | CURSOR | TYPES | SECRET |
/// DISABLEABLE | REPORTS_STATUS`; `render::components::text_input::*`.
///
/// ## Invariants
/// `draw` never commits, cancels or validates (it takes `&TextInputState`);
/// a secret draft is masked while editing and never reaches `Debug`; the
/// hardware cursor belongs to focus by default; an explicit fallback policy
/// may publish the active draft as cursor owner without moving focus.
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
    inline: bool,
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
            .field("inline", &self.inline)
            .finish_non_exhaustive()
    }
}

impl<'a> TextInput<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::FIELD,
        Part::TEXT,
        Part::PLACEHOLDER,
        Part::MARKER,
        Part::GUTTER,
        Part::ICON,
    ];

    /// A text input.
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
            inline: false,
        }
    }

    /// Whether the editor is drawn inline in a cell, without standalone gutter or indent.
    #[must_use]
    pub const fn inline(mut self, enabled: bool) -> Self {
        self.inline = enabled;
        self
    }

    /// The controlled value, for `draw`.
    #[must_use]
    pub const fn value(mut self, v: &'a str) -> Self {
        self.value = Some(v);
        self
    }

    /// Placeholder shown while the value is empty and no edit is in flight.
    #[must_use]
    pub const fn placeholder(mut self, s: &'a str) -> Self {
        self.placeholder = Some(s);
        self
    }

    /// Also show the placeholder for an empty active draft, preserving its cursor.
    /// Disabled by default; this changes painting only, not editing ownership.
    #[must_use]
    pub const fn placeholder_while_editing(mut self, enabled: bool) -> Self {
        self.placeholder_while_editing = enabled;
        self
    }

    /// The validator run on commit.
    #[must_use]
    pub const fn validate(mut self, v: &'a dyn Validate) -> Self {
        self.validate = Some(v);
        self
    }

    /// What focus loss does to a draft.
    #[must_use]
    pub const fn blur(mut self, p: BlurPolicy) -> Self {
        self.blur = p;
        self
    }

    /// Publish an alternate typing target without moving navigation focus.
    /// Fallback routing requires an active editable draft and compatible
    /// presented geometry. Commit and cancel remain primary-focus actions.
    #[must_use]
    pub const fn typing_policy(mut self, policy: crate::TypingPolicy) -> Self {
        self.typing_policy = policy;
        self
    }

    /// Mask the text (§15): every grapheme paints the policy's mask glyph.
    #[must_use]
    pub const fn secret(mut self, policy: SecretPolicy) -> Self {
        self.secret = Some(policy);
        self
    }

    pub const fn is_secret(&self) -> bool {
        self.secret.is_some()
    }

    /// Read-only: stays in the ring, never edits.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered, never reachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// Allow pointer activation and caret positioning (enabled by default).
    /// Disabling pointer input preserves keyboard editing, focus and geometry.
    #[must_use]
    pub const fn pointer_enabled(mut self, enabled: bool) -> Self {
        self.pointer_enabled = enabled;
        self
    }

    /// Data readiness.
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
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
            inline: self.inline,
        }
    }

    fn validator(&self) -> Dyn<'_> {
        Dyn(self.validate.unwrap_or(&NoValidate))
    }

    /// Columns between the gutter indent and the always-reserved trailing cell.
    const fn inner_width(area_width: u16) -> u16 {
        area_width.saturating_sub(3)
    }

    /// The update phase: drains this control's intents and drives the edit
    /// lifecycle. The controlled `value` is written on commit only.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut String,
    ) -> Response<TextAction> {
        self.update_target(cx, st, value)
    }

    /// Form bridge with inherited disabled state. The configured component
    /// remains the source of every other prop.
    pub fn update_in_form<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
        inherited_disabled: bool,
    ) -> Response<TextAction> {
        let field = self.with_inherited_disabled(inherited_disabled);
        let focused = cx.state(field.id).contains(StateFlags::FOCUSED);
        if focused && field.editable() {
            st.begin(value.expose());
        }
        field.update_target(cx, st, value)
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

    /// The update phase over any [`TextTarget`], without the form
    /// bridge's focus pre-begin. Sensitivity reconciles from this
    /// control's `.secret()` policy only: a `Secret` value never
    /// re-sensitizes the state, so plaintext acknowledgement echo keeps a
    /// secret-typed store. Every other arm matches [`TextInput::update`].
    pub fn update_value<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
    ) -> Response<TextAction> {
        self.update_core(cx, st, value, false)
    }

    fn update_target<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
    ) -> Response<TextAction> {
        let value_sensitive = value.is_sensitive();
        self.update_core(cx, st, value, value_sensitive)
    }

    fn update_core<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut T,
        value_sensitive: bool,
    ) -> Response<TextAction> {
        st.set_sensitive(self.secret.is_some() || value_sensitive);
        let mut acc = super::Acc::<TextAction>::new();
        let editable = self.editable();
        // W08-07: the caller owns the value and may rewrite it under an
        // active draft (the widget writes on commit only, so a mismatch
        // is always external). The newer write wins: drop the stale draft
        // so no later commit silently overwrites it; the next begin
        // reseeds from the current value.
        if st.external_changed(value.expose()) {
            st.cancel();
            acc.changed();
        }
        for it in cx.intents(self.id) {
            match it {
                Intent::FocusIn { .. } => {}
                Intent::FocusOut { .. } => {
                    if st.is_editing() {
                        let policy = self.blur;
                        // W11-04: focus transit through a pristine field
                        // ends the edit silently — the reference never
                        // auto-begins, so tabbing past an untouched field
                        // reports nothing there either.
                        let dirty = st.draft.text() != value.expose();
                        let _ = st.blur_target(value, &self.validator(), policy);
                        match policy {
                            BlurPolicy::CommitAndValidate | BlurPolicy::Commit => {
                                if dirty {
                                    acc.action(TextAction::Committed);
                                } else {
                                    acc.consumed();
                                }
                            }
                            BlurPolicy::Cancel => acc.action(TextAction::Cancelled),
                            BlurPolicy::Keep => {}
                        }
                    }
                }
                Intent::Binding(action) if editable => {
                    if let Some(cmd) = Binding::command(edit_bindings(false), action) {
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
                    } else {
                        // Baseline `edit_key … => None`: keys with no edit
                        // meaning (verticals, ctrl/alt chords) are swallowed
                        // while editing, never ignored.
                        acc.consumed();
                    }
                }
                Intent::Paste(s) if editable => {
                    if !st.is_editing() {
                        // W08-03: paste while navigating starts editing
                        // (reference `on_paste` calls `begin_edit` first).
                        st.begin(value.expose());
                    }
                    if st.apply(EditAction::Paste(s)).changed() {
                        self.live_validate(st);
                        acc.action(TextAction::Changed);
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    local,
                    ..
                } if editable && self.pointer_enabled => {
                    if !st.is_editing() {
                        st.begin(value.expose());
                    }
                    let col = usize::from(local.x.saturating_sub(2))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    // W08-02: a press without release edits nothing (the
                    // legacy widget has no press handler — `on_click`
                    // only), so `Phase::Press` falls through to the
                    // consume arm below.
                    if self.secret.is_some() || st.is_sensitive() {
                        // W08-02: masked paint shows one cell per grapheme,
                        // so the click column counts graphemes, not
                        // plaintext cells (reference `on_click` walks
                        // `display_graphemes`).
                        let text = st.draft.text();
                        let at = graphemes(text).nth(col).map_or(text.len(), |(i, _)| i);
                        st.draft.set_cursor_offset(at);
                    } else {
                        st.draft.set_cursor_line_col(0, col);
                    }
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
                let action = edit_action_of(cmd);
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

    /// The draw phase: the field row, the placeholder or the text, the
    /// error marker and the cursor request.
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
        // runtime: the frame's own focus/hover/press; derived: `.status`,
        // the edit phase, the error, `.read_only` and `.disabled`
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
        // the readiness affordance is a *symbol*, so it survives `Mono`
        // without a theme rule (§11.4's `BUSY`/`LOADING` row); it shares the
        // trailing cell with the error glyph, which wins.
        let busy = matches!(self.status, Status::Busy | Status::Loading);
        let inner = if self.inline {
            Rect {
                width: area.width.saturating_sub(1),
                ..area
            }
        } else {
            Rect {
                x: area.x.saturating_add(2),
                y: area.y,
                width: Self::inner_width(area.width),
                height: 1,
            }
        };
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        if self.pointer_enabled {
            ui.register_editor(self.id, area, focusability, declared);
        } else {
            ui.register_keyboard_editor(self.id, area, focusability, declared);
        }
        ui.publish_bindings(self.id, live, edit_bindings(false));
        if let crate::TypingPolicy::Fallback { cursor } = self.typing_policy {
            ui.publish_typing_target(
                self.id,
                edit_bindings(false),
                |cmd| !matches!(cmd, TextCmd::Commit | TextCmd::Cancel),
                cursor,
            );
        }
        let owns_cursor = editing
            && (live.contains(StateFlags::FOCUSED)
                || matches!(
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
        if !self.inline {
            let gutter_cell = cell_at(area, area.x);
            if let Some(f) = ov.slot_for(Part::GUTTER) {
                f(ui, gutter_cell);
            } else {
                let g = style(ui, Part::GUTTER);
                match g.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(gutter_cell, glyph, g.style);
                    }
                    Slot::Inherit if live.contains(StateFlags::FOCUSED) => {
                        ui.glyph(
                            gutter_cell,
                            GlyphRole::FocusBar,
                            g.style.with_bg_from(field.style),
                        );
                    }
                    Slot::Inherit | Slot::Clear => {
                        ui.fill(gutter_cell, field.style.with_fg_from_bg(field.style));
                    }
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
                let secret_policy = self
                    .secret
                    .or_else(|| st.is_sensitive().then_some(SecretPolicy::default()));
                let cursor_col = if editing {
                    if secret_policy.is_some() {
                        // W08-02: masked paint shows one cell per grapheme,
                        // so the caret column counts graphemes before the
                        // caret, not plaintext cells — clicks and cursor
                        // share the display geometry.
                        let at = st.draft.cursor_offset();
                        graphemes(shown).take_while(|(i, _)| *i < at).count()
                    } else {
                        st.draft.cursor_pos().col
                    }
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
                let total = match secret_policy {
                    Some(_) => graphemes(shown).count(),
                    None => usize::from(width(shown)),
                };
                let ellipsis_style = if self.inline {
                    ts.style
                } else {
                    ts.style
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))))
                };
                let mut run = inner;
                if hs > 0 {
                    let used = ui.glyph(run, GlyphRole::Ellipsis, ellipsis_style);
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
                    let sel = if editing { st.draft.selection() } else { None };
                    if let Some(r) = sel {
                        let text_start = start;
                        let text_end = shown.len();
                        let s_start = r.start.max(text_start).min(text_end);
                        let s_end = r.end.max(text_start).min(text_end);
                        let sel_style = if ui.theme().capability.color == ColorLevel::Mono {
                            ts.style
                                .patch(
                                    ui.paint_patch(
                                        &StylePatch::new()
                                            .set_bg(Role::SelectionBg)
                                            .set_fg(Role::SelectionFg),
                                    ),
                                )
                                .add_modifier(Modifier::REVERSED)
                        } else {
                            ts.style.patch(
                                ui.paint_patch(
                                    &StylePatch::new()
                                        .set_bg(Role::SelectionBg)
                                        .set_fg(Role::SelectionFg),
                                ),
                            )
                        };
                        let mut cur_run = run;
                        if s_start > text_start {
                            let before = &shown[text_start..s_start];
                            let used = ui.paint_str(cur_run, before, ts.style);
                            cur_run = shift(cur_run, used);
                        }
                        if s_end > s_start {
                            let selected_text = &shown[s_start..s_end];
                            let used = ui.paint_str(cur_run, selected_text, sel_style);
                            cur_run = shift(cur_run, used);
                        }
                        if text_end > s_end {
                            let after = &shown[s_end..text_end];
                            ui.paint_str(cur_run, after, ts.style);
                        }
                    } else {
                        ui.paint_str(run, shown.get(start..).unwrap_or(""), ts.style);
                    }
                }
                if overflow_right {
                    let last = cell_at(inner, inner.right().saturating_sub(1));
                    ui.glyph(last, GlyphRole::Ellipsis, ellipsis_style);
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
                    let text = crate::text::truncate(p, inner.width);
                    ui.paint_str(inner, &text, ps.style);
                }
            }
        }
        if !self.inline {
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

    /// The natural size: one row, eight columns minimum, thirty preferred.
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
        // Oracle reveal rule (TI-MASK-007, masked && !editing): the
        // committed tail shows the real last `tail` graphemes, painted
        // slice-by-slice so no allocation of the secret is built.
        let start = total.saturating_sub(tail);
        for (_, g) in graphemes(shown).skip(skip).skip(start).take(tail) {
            cell.text(g);
        }
    } else {
        cell.glyphs(policy.mask, total);
    }
}

/// A borrowed validator behind the blanket-impl bound.
struct Dyn<'a>(&'a dyn Validate);

impl Validate for Dyn<'_> {
    fn check(&self, s: &str) -> Result<(), FieldError> {
        self.0.check(s)
    }
}

impl Bindings for TextInput<'_> {
    type Cmd = TextCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TextCmd>] {
        edit_bindings(false)
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

    struct ValueApp {
        state: TextInputState,
        value: Secret,
        secret_control: bool,
        last: Option<TextAction>,
    }

    impl App for ValueApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let input = TextInput::new(ID);
            let input = if self.secret_control {
                input.secret(SecretPolicy::default())
            } else {
                input
            };
            let r = input.update_value(cx, &mut self.state, &mut self.value);
            self.last = r.action_ref().copied();
            r.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let input = TextInput::new(ID);
            let input = if self.secret_control {
                input.secret(SecretPolicy::default())
            } else {
                input
            };
            input
                .value(self.value.expose())
                .draw(ui, SCREEN, &self.state);
        }
    }

    /// `update_value` is the plain generic path: no focus pre-begin, the
    /// `update` lifecycle over a `Secret` value, and sensitivity from the
    /// control policy only — a secret-typed store never re-sensitizes.
    #[test]
    fn update_value_is_plain_and_control_sensitive_only() {
        let mut rt = Runtime::new(
            ValueApp {
                state: TextInputState::default(),
                value: Secret::new(String::new()),
                secret_control: false,
                last: None,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.set_focus(Some(ID));
        let key = |code| {
            Input::Key(Key {
                code,
                mods: KeyModifiers::NONE,
            })
        };
        let step = |rt: &mut Runtime<ValueApp>, buf: &mut Buffer, input: Input| {
            let _ = crate::runtime::stub::deliver(rt, input);
            rt.draw_buffer(SCREEN, &mut *buf).commit_presented();
        };
        step(&mut rt, &mut buf, Input::Tick);
        assert!(
            !rt.app().state.is_editing(),
            "update_value never pre-begins on focus"
        );
        assert!(
            !rt.app().state.is_sensitive(),
            "a Secret value must not re-sensitize a plain control"
        );
        step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.app().state.is_editing(), "idle Enter begins");
        assert_eq!(rt.app().last, None, "begin reports no commit");
        step(&mut rt, &mut buf, key(KeyCode::Char('a')));
        assert_eq!(rt.app().last, Some(TextAction::Changed));
        step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(rt.app().last, Some(TextAction::Committed));
        assert_eq!(rt.app().value.expose(), "a");
        assert!(!rt.app().state.is_editing(), "commit ends the edit");
        assert!(
            !rt.app().state.is_sensitive(),
            "committing into a Secret store stays non-sensitive"
        );
        step(&mut rt, &mut buf, key(KeyCode::Enter));
        step(&mut rt, &mut buf, key(KeyCode::Char('b')));
        step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert_eq!(rt.app().last, Some(TextAction::Cancelled));
        assert_eq!(rt.app().value.expose(), "a", "cancel drops the draft");
        assert!(!rt.app().state.is_editing());
        rt.app_mut().secret_control = true;
        step(&mut rt, &mut buf, Input::Tick);
        assert!(
            rt.app().state.is_sensitive(),
            "a .secret() control still sensitizes"
        );
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

    /// §16.1: `begin` snapshots the controlled value into the draft, and the
    /// value itself does not move until a commit.
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
        // a second begin while editing keeps the draft
        st.begin("other");
        assert_eq!(st.draft.text(), "hello!");
    }

    /// §16.1: commit is the only writer of the controlled value.
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
        // committing while idle leaves the value alone
        assert!(st.commit(&mut value, &rule).is_ok());
        assert_eq!(value, "a@");
    }

    /// §16.1: one commit runs the validator exactly once, over the value it
    /// just wrote.
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

    /// §16.1: the default policy writes the draft **and** validates it.
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

    /// §16.1: `Cancel` drops the draft and leaves the value untouched.
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

    /// §16.1: `Keep` leaves the edit in flight, so focus can return to it.
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
        // and `Commit` afterwards writes exactly that draft
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

    /// §16.1: an error set from outside (an async / server-side check) is
    /// state, not a derived value, so redrawing never clears it.
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

    /// Oracle reveal rule (TI-MASK-007, masked && !editing): a committed
    /// masked field paints mask glyphs plus the real last-`tail`
    /// graphemes — the reveal tail identifies which secret is stored.
    #[test]
    fn committed_mask_reveals_real_tail() {
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
        let policy = SecretPolicy::default();
        let mask = Theme::junie().design.glyphs.get(policy.mask);
        let painted = row.trim();
        let masked = SECRET.chars().count().saturating_sub(policy.synthetic_tail);
        assert!(
            painted.starts_with(&mask.repeat(masked)),
            "the mask run is wrong: {painted}"
        );
        // the tail reveals the real last graphemes, not a substitute
        let tail: String = painted.chars().skip(masked).collect();
        assert_eq!(tail.chars().count(), policy.synthetic_tail);
        assert_eq!(tail, "r2", "the reveal tail must be real: {painted}");
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
