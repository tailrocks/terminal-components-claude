//! `TextInput` and its explicit edit lifecycle (`COMPONENT_ARCHITECTURE.md`
//! §15, §17.0 A7, Appendix A 4B).

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Overrides, SlotFn, cell_at, first_row, shift};
use crate::collection::{CellUi, Status};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::secret::{Secret, SecretPolicy};
use crate::text::measure::graphemes;
use crate::text::{EditAction, EditOutcome, Extend, Motion, TextEditorCore, width};
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};

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
/// table ([`TextArea`](crate::TextArea)); a single-line
/// [`TextInput`](crate::TextInput) never binds them, exactly as the legacy
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

const fn b(chord: Chord, cmd: TextCmd, label: &'static str, visible: bool) -> Binding<TextCmd> {
    Binding {
        chord,
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
        "Word left",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, ALT),
        TextCmd::Move(Motion::WordRight, Extend::No),
        "Word right",
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
        "Start",
        false,
    ),
    b(
        Chord::with(KeyCode::End, CTRL),
        TextCmd::Move(Motion::DocEnd, Extend::No),
        "End",
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
        "Delete word",
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
        "Start",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('e'), CTRL),
        TextCmd::Move(Motion::End, Extend::No),
        "End",
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
        "Delete word",
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
        "Word left",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('f'), ALT),
        TextCmd::Move(Motion::WordRight, Extend::No),
        "Word right",
        false,
    ),
];

/// Durable state of a [`TextInput`]: the in-flight draft, the phase and the
/// last validation error. `Debug` redacts the draft.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct TextInputState {
    draft: TextEditorCore,
    phase: EditPhase,
    error: Option<FieldError>,
}

impl fmt::Debug for TextInputState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInputState")
            .field("draft", &"[redacted]")
            .field("draft_len", &self.draft.text().len())
            .field("phase", &self.phase)
            .field("error", &self.error)
            .finish()
    }
}

impl TextInputState {
    /// Whether a draft is in flight.
    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    /// The phase.
    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    /// The last validation error.
    pub const fn error(&self) -> Option<&FieldError> {
        self.error.as_ref()
    }

    /// Set (or clear) the error from an external / async validation.
    pub fn set_error(&mut self, e: Option<FieldError>) {
        self.error = e;
    }

    /// Begin an edit over `current` (a no-op while editing).
    pub fn begin(&mut self, current: &str) {
        if self.is_editing() {
            return;
        }
        self.draft = TextEditorCore::single(current);
        self.phase = EditPhase::Editing;
    }

    /// Write the draft to `value`, end the edit and validate.
    ///
    /// # Errors
    /// The validator's error; it is also recorded in the state.
    pub fn commit(&mut self, value: &mut String, v: &impl Validate) -> Result<(), FieldError> {
        self.write(value);
        let r = v.check(value);
        self.error = r.clone().err();
        r
    }

    fn write(&mut self, value: &mut String) {
        if self.is_editing() {
            value.clear();
            value.push_str(self.draft.text());
        }
        self.phase = EditPhase::Idle;
        self.draft.zeroize();
    }

    /// Drop the draft.
    pub fn cancel(&mut self) {
        self.phase = EditPhase::Idle;
        self.draft.zeroize();
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
        match p {
            BlurPolicy::CommitAndValidate => self.commit(value, v),
            BlurPolicy::Commit => {
                self.write(value);
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
    }

    fn apply(&mut self, a: EditAction<'_>) -> EditOutcome {
        self.draft.apply(a)
    }
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
/// `.status(Status)`, `.patch`, `.patch_part`, `.slot`, `.state_override`.
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
/// One row: gutter, two-cell indent, the text run, a trailing cell (two
/// with the error marker or the readiness spinner). `measure` is `(8…, 1)`; `draw` paints the first
/// row of `area` and returns it; `0×0` registers nothing (R5).
///
/// ## Parts
/// `FIELD` (the row fill), `TEXT` (the value / draft), `PLACEHOLDER`,
/// `MARKER` (the trailing error glyph), `GUTTER` (the focus bar), `ICON`
/// (the readiness spinner, in the same trailing cell).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER` and `PLACEHOLDER`;
/// `FIELD` and `TEXT` cannot be replaced.
///
/// ## Identity
/// One `Id`; no items.
///
/// ## Testing
/// `TextInputCase` with `FOCUSABLE | EDITS | CURSOR | TYPES | SECRET |
/// DISABLEABLE`; `render::components::text_input::*`.
///
/// ## Invariants
/// `draw` never commits, cancels or validates (it takes `&TextInputState`);
/// a secret draft is masked while editing and never reaches `Debug`; the
/// hardware cursor is written only while editing and focused.
pub struct TextInput<'a> {
    id: Id,
    value: Option<&'a str>,
    placeholder: Option<&'a str>,
    validate: Option<&'a dyn Validate>,
    blur: BlurPolicy,
    secret: Option<SecretPolicy>,
    read_only: bool,
    disabled: bool,
    status: Status,
    ov: Overrides<'a>,
}

impl fmt::Debug for TextInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInput")
            .field("id", &self.id)
            .field("value", &self.value.map(|_| "[redacted]"))
            .field("placeholder", &self.placeholder)
            .field("blur", &self.blur)
            .field("secret", &self.secret)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("status", &self.status)
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
            validate: None,
            blur: BlurPolicy::CommitAndValidate,
            secret: None,
            read_only: false,
            disabled: false,
            status: Status::Ready,
            ov: Overrides::new(),
        }
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

    /// Mask the text (§15): every grapheme paints the policy's mask glyph.
    #[must_use]
    pub const fn secret(mut self, policy: SecretPolicy) -> Self {
        self.secret = Some(policy);
        self
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

    /// Data readiness.
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn validator(&self) -> Dyn<'_> {
        Dyn(self.validate.unwrap_or(&NoValidate))
    }

    /// Columns between the gutter indent and the trailing cells. `marker`
    /// reserves the trailing cell, which carries either the error glyph or
    /// the readiness spinner.
    fn inner_width(area_width: u16, marker: bool) -> u16 {
        area_width
            .saturating_sub(3)
            .saturating_sub(if marker { 2 } else { 0 })
    }

    /// The update phase: drains this control's intents and drives the edit
    /// lifecycle. The controlled `value` is written on commit only.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextInputState,
        value: &mut String,
    ) -> Response<TextAction> {
        let mut acc = super::Acc::<TextAction>::new();
        let editable = self.editable();
        for it in cx.intents(self.id) {
            match it {
                Intent::FocusIn { .. } => {
                    if editable {
                        st.begin(value);
                    }
                }
                Intent::FocusOut { .. } => {
                    if st.is_editing() {
                        let policy = self.blur;
                        let _ = st.blur(value, &self.validator(), policy);
                        match policy {
                            BlurPolicy::CommitAndValidate | BlurPolicy::Commit => {
                                acc.action(TextAction::Committed);
                            }
                            BlurPolicy::Cancel => acc.action(TextAction::Cancelled),
                            BlurPolicy::Keep => {}
                        }
                    }
                }
                Intent::Key(k) if editable => {
                    if st.is_editing() {
                        self.edit_key(st, value, k, &mut acc);
                    } else if k.is(KeyCode::Enter) {
                        st.begin(value);
                        acc.changed();
                    } else if let Some(c) = k.bare_char() {
                        st.begin(value);
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
                } if editable => {
                    if !st.is_editing() {
                        st.begin(value);
                    }
                    let col = usize::from(local.x.saturating_sub(2))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    st.draft.set_cursor_line_col(0, col);
                    acc.changed();
                }
                Intent::Pointer { .. } => acc.consumed(),
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
            let w = Self::inner_width(
                a.width,
                st.error.is_some() || !matches!(self.status, Status::Ready),
            );
            st.draft.scroll_into_view(w);
        }
        acc.finish(self.id)
    }

    fn live_validate(&self, st: &mut TextInputState) {
        if st.error.is_some() {
            st.error = self.validator().check(st.draft.text()).err();
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

    fn edit_key(
        &self,
        st: &mut TextInputState,
        value: &mut String,
        k: crate::event::Key,
        acc: &mut super::Acc<TextAction>,
    ) {
        match Binding::lookup(BINDINGS, &k) {
            Some(TextCmd::Cancel) => {
                st.cancel();
                acc.action(TextAction::Cancelled);
            }
            Some(TextCmd::Commit) => {
                let _ = st.commit(value, &self.validator());
                acc.action(TextAction::Committed);
            }
            Some(cmd) => {
                let action = match cmd {
                    TextCmd::Move(m, e) => EditAction::Move(m, e),
                    TextCmd::Backspace => EditAction::Backspace,
                    TextCmd::Delete => EditAction::Delete,
                    TextCmd::DeleteWordLeft => EditAction::DeleteWordLeft,
                    TextCmd::DeleteToLineEnd => EditAction::DeleteToLineEnd,
                    TextCmd::DeleteToLineStart => EditAction::DeleteToLineStart,
                    TextCmd::SelectAll => EditAction::SelectAll,
                    // never bound by the single-line table above
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
            None => {
                if let Some(c) = k.bare_char() {
                    self.insert(st, c, acc);
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
        let error = st.error.is_some() || matches!(self.status, Status::Error);
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
        let mut live = self.ov.flags(ui.state(self.id)) | self.status.flags();
        if editing {
            live |= StateFlags::EDITING;
        }
        if error {
            live |= StateFlags::ERROR;
        }
        if self.read_only {
            live |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            live |= StateFlags::DISABLED;
            live = live.difference(StateFlags::HOVERED);
        }
        // the readiness affordance is a *symbol*, so it survives `Mono`
        // without a theme rule (§11.4's `BUSY`/`LOADING` row); it shares the
        // trailing cell with the error glyph, which wins.
        let busy = live.intersects(StateFlags::BUSY | StateFlags::LOADING);
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y,
            width: Self::inner_width(area.width, error || busy),
            height: 1,
        };
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        if !self.ov.is_forced() {
            ui.register_editor(self.id, area, focusability, declared);
        }
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
                Some(glyph) => {
                    ui.glyph(gutter_cell, glyph, g.style);
                }
                None => ui.fill(gutter_cell, g.style),
            }
        }
        let shown = if editing {
            st.draft.text()
        } else {
            self.value.unwrap_or("")
        };
        if shown.is_empty() && !editing {
            if let Some(p) = self.placeholder {
                let ps = style(ui, Part::PLACEHOLDER);
                match ov.slot_for(Part::PLACEHOLDER) {
                    Some(f) => f(ui, inner),
                    None => {
                        ui.paint_str(inner, p, ps.style);
                    }
                }
            }
        } else if inner.width > 0 {
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
            let total = match self.secret {
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
            if let Some(policy) = self.secret {
                paint_masked(ui, run, shown, skip, editing, policy, ts.style);
            } else {
                let start = byte_at_col(shown, skip);
                ui.paint_str(run, shown.get(start..).unwrap_or(""), ts.style);
            }
            if overflow_right {
                let last = cell_at(inner, inner.right().saturating_sub(1));
                ui.glyph(last, GlyphRole::Ellipsis, ts.style);
            }
            if live.contains(StateFlags::FOCUSED) && self.editable() {
                let cursor_col = if editing {
                    cursor_col
                } else {
                    usize::from(width(shown))
                };
                let cx = inner
                    .x
                    .saturating_add(cursor_col.saturating_sub(hs).min(usize::from(u16::MAX)) as u16)
                    .min(inner.right());
                ui.set_cursor(self.id, Position::new(cx, inner.y));
            }
        } else if live.contains(StateFlags::FOCUSED) && self.editable() {
            ui.set_cursor(self.id, Position::new(inner.x, inner.y));
        }
        if error {
            let marker_cell = cell_at(area, area.right().saturating_sub(2));
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, marker_cell);
            } else {
                let ms = style(ui, Part::MARKER);
                if let Some(g) = ms.glyph {
                    ui.glyph(marker_cell, g, ms.style);
                }
            }
        } else if busy {
            let icon_cell = cell_at(area, area.right().saturating_sub(2));
            let is = style(ui, Part::ICON);
            let frames = ui.design().motion.spinner_frames;
            let frame = frames.first().copied().unwrap_or("");
            ui.paint_str(icon_cell, frame, is.style);
        }
        area
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
    style: ratatui_core::style::Style,
) {
    let total = graphemes(shown).count().saturating_sub(skip);
    let tail = if editing {
        0
    } else {
        policy.synthetic_tail.min(total)
    };
    let mut cell = CellUi::new(ui.reborrow(), run, style);
    if tail > 0 {
        let secret = Secret::new(shown.to_owned());
        secret.write_mask(&mut cell, total.saturating_sub(tail), policy);
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

/// Byte offset of display column `col` in `s` (the whole length past the end).
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

    fn inherit_forced(mut self, s: Option<StateFlags>) -> Self {
        self.ov = self.ov.inherit_forced(s);
        self
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const ID: Id = Id::root("input.tests");

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

    /// §16.1: an error set from outside (an async / server-side check) is
    /// state, not a derived value, so redrawing never clears it.
    #[test]
    fn external_error_survives_a_redraw() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = TextInputState::default();
        st.set_error(Some(FieldError::coded("Already taken", "dup")));
        for _ in 0..3 {
            rt.draw_scene(SCREEN, &mut buf, |ui, a| {
                TextInput::new(ID).value("ada").draw(ui, a, &st);
            });
        }
        assert_eq!(st.error().map(|e| e.code), Some(Some("dup")));
        st.set_error(None);
        assert!(st.error().is_none());
    }

    /// §16.1 (P5): a masked field paints mask glyphs and a **synthetic**
    /// tail derived from the fingerprint — never the real characters, and
    /// never a `String` of them.
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
        });
        let mut row = String::new();
        for x in 0..SCREEN.width {
            if let Some(c) = buf.cell(Position::new(x, 0)) {
                row.push_str(c.symbol());
            }
        }
        assert!(!row.contains(SECRET), "the secret reached the buffer: {row}");
        assert!(!row.contains('h') && !row.contains('u'), "{row}");
        let mask = Theme::junie().design.glyphs.get(SecretPolicy::default().mask);
        assert!(row.contains(mask), "no mask glyph in {row}");
        // the tail is the fingerprint alphabet, and it is stable per secret
        let tail: String = row
            .trim_end()
            .chars()
            .rev()
            .take(SecretPolicy::default().synthetic_tail)
            .collect();
        assert!(
            tail.chars().all(|c| c.is_ascii_alphanumeric()),
            "tail {tail} is not synthetic"
        );
    }

    #[test]
    fn byte_offsets_follow_display_columns() {
        assert_eq!(byte_at_col("日本語", 2), 3);
        assert_eq!(byte_at_col("ab", 9), 2);
        assert_eq!(byte_at_col("ab", 0), 0);
    }
}
