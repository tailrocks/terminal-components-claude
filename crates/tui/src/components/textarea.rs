//! `TextArea` — the multi-line text control (`COMPONENT_ARCHITECTURE.md`
//! §15, §17.0 A7/A10, §18.2, Appendix A 4B).

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::input::{BlurPolicy, EditPhase, TextAction, TextCmd, byte_at_col};
use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn, cell_at, first_row};
use crate::event::{Chord, Key, KeyCode, KeyModifiers};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::text::{EditAction, EditOutcome, Extend, Motion, TextEditorCore, width};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

const fn b(chord: Chord, cmd: TextCmd, label: &'static str, visible: bool) -> Binding<TextCmd> {
    Binding {
        chord,
        cmd,
        label,
        priority: 50,
        visible,
    }
}

/// The multi-line flavour of the shared edit table (the legacy
/// `field_common::edit_key(key, multiline = true)`): `Enter` inserts a
/// newline, `Esc` **commits** — a document is not cancelled by leaving it —
/// and `↑`/`↓`/`PgUp`/`PgDn` move the cursor by line and by page.
const BINDINGS: &[Binding<TextCmd>] = &[
    b(Chord::key(KeyCode::Esc), TextCmd::Commit, "Done", true),
    b(
        Chord::key(KeyCode::Enter),
        TextCmd::Newline,
        "New line",
        true,
    ),
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
        Chord::key(KeyCode::Up),
        TextCmd::Move(Motion::Up, Extend::No),
        "Up",
        false,
    ),
    b(
        Chord::key(KeyCode::Down),
        TextCmd::Move(Motion::Down, Extend::No),
        "Down",
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
        Chord::with(KeyCode::Up, SHIFT),
        TextCmd::Move(Motion::Up, Extend::Select),
        "Select up",
        false,
    ),
    b(
        Chord::with(KeyCode::Down, SHIFT),
        TextCmd::Move(Motion::Down, Extend::Select),
        "Select down",
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
        Chord::key(KeyCode::PageUp),
        TextCmd::PageUp,
        "Page up",
        true,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        TextCmd::PageDown,
        "Page down",
        true,
    ),
    b(
        Chord::key(KeyCode::Home),
        TextCmd::Move(Motion::Home, Extend::No),
        "Line start",
        false,
    ),
    b(
        Chord::key(KeyCode::End),
        TextCmd::Move(Motion::End, Extend::No),
        "Line end",
        false,
    ),
    b(
        Chord::with(KeyCode::Home, SHIFT),
        TextCmd::Move(Motion::Home, Extend::Select),
        "Select to line start",
        false,
    ),
    b(
        Chord::with(KeyCode::End, SHIFT),
        TextCmd::Move(Motion::End, Extend::Select),
        "Select to line end",
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
        "Line start",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('e'), CTRL),
        TextCmd::Move(Motion::End, Extend::No),
        "Line end",
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

/// Durable state of a [`TextArea`]: the in-flight draft, the phase, the
/// vertical scroll and the last validation error. `Debug` redacts the draft.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct TextAreaState {
    draft: TextEditorCore,
    phase: EditPhase,
    scroll: ScrollState,
    error: Option<FieldError>,
}

impl fmt::Debug for TextAreaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaState")
            .field("draft", &"[redacted]")
            .field("draft_len", &self.draft.text().len())
            .field("phase", &self.phase)
            .field("scroll", &self.scroll)
            .field("error", &self.error)
            .finish()
    }
}

impl TextAreaState {
    /// Whether a draft is in flight.
    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    /// The phase.
    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    /// The vertical scroll.
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    /// The vertical scroll, mutably.
    pub const fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
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
        self.draft = TextEditorCore::multi(current);
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
    /// The validator's error under [`BlurPolicy::CommitAndValidate`]; the
    /// default policy is [`BlurPolicy::Commit`], which never validates.
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

/// A multi-line text control over the shared [`TextEditorCore`], with an
/// explicit edit lifecycle, a scroll region and grapheme-correct editing.
///
/// ## Construction
/// `TextArea::new(id, rows)` — `rows` is the height of the text region. The
/// controlled value is passed per phase: `&mut String` to `update`,
/// `.value(&str)` for `draw`.
///
/// ## Ownership
/// The caller owns the value and a [`TextAreaState`] (draft, phase, scroll,
/// error). The runtime owns focus, hover, the cursor write, wheel routing
/// and the scrollbar capture. Controlled is the default (S4): the value
/// changes only on commit.
///
/// ## Configuration
/// `.value(&str)` (draw), `.placeholder(&str)`, `.validate(&dyn Validate)`
/// (`NoValidate`), `.blur(BlurPolicy)` (**`Commit`** — a document is
/// committed, not cancelled, when focus leaves it, §15), `.rows(u16)`,
/// `.read_only(bool)`, `.disabled(bool)`, `.status(Status)`, `.patch`,
/// `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::TEXTAREA`, `DEFAULT` only.
///
/// ## States
/// `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` from the runtime; `EDITING` is
/// owned by the state and declared every frame; `ERROR` from the state's
/// error or `.status(Error)`; `READ_ONLY`, `DISABLED`, `BUSY`, `LOADING`.
///
/// ## Actions
/// [`TextAction`]: `Changed` (the draft changed), `Committed` (written to
/// the value — Esc, or focus loss under `Commit`/`CommitAndValidate`),
/// `Cancelled` (draft dropped), `MoveNext` / `MovePrev` (reserved).
///
/// ## Focus
/// `Focusable` (`FocusableReadOnly` / `Disabled`); swallows typing. Focus
/// arriving begins an edit; focus leaving applies the blur policy.
///
/// ## Keyboard
/// The multi-line edit table: `Esc` commits (the legacy semantics — a
/// document is not cancelled by leaving it), `Enter` inserts a newline,
/// `←`/`→`/`↑`/`↓` (`Shift` selects, `Ctrl`/`Alt` move by word),
/// `PgUp`/`PgDn` move by page, `Home`/`End` (`Shift` selects, `Ctrl` the
/// document), `Backspace` (`Ctrl`/`Alt` word), `Del`, `Ctrl+a`/`Ctrl+e`
/// line start/end, `Ctrl+u`/`Ctrl+k` delete to start/end, `Ctrl+w` delete
/// word, `Ctrl+l` select all, `Alt+b`/`Alt+f` word motion.
///
/// ## Mouse
/// `PartRef::of(Part::CONTAINER)`: a press begins editing and places the
/// cursor at the clicked line and column. `TRACK` / `THUMB` and the wheel
/// go to the embedded [`ScrollRegion`].
///
/// ## Layout
/// `rows` rows (clamped to `area`): a gutter column, a two-cell indent, the
/// text window, a reserved trailing pad column (the error marker and the
/// readiness spinner share it), and a scrollbar column while the document
/// overflows.
/// `measure` is `(12…40, rows)`; `draw` paints the rows it used and returns
/// them; `0×0` registers nothing (R5).
///
/// ## Parts
/// `FIELD` (the body fill), `TEXT` (the value / draft), `PLACEHOLDER`,
/// `ROW` (the selection run), `MARKER` (the trailing error glyph), `GUTTER`
/// (the focus bar), `ICON` (the readiness spinner, in that same trailing
/// cell), `TRACK` / `THUMB` (the scrollbar).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER`, `ICON` and
/// `PLACEHOLDER`; `FIELD` and `TEXT` cannot be replaced.
///
/// ## Identity
/// One `Id`; no items. The scrollbar is `TRACK` / `THUMB` of the same id.
///
/// ## Testing
/// `TextAreaCase` with `FOCUSABLE | EDITS | CURSOR | TYPES | SCROLLS |
/// DISABLEABLE`; `render::components::text_area::*`;
/// `textarea::blur_commits_without_validation`;
/// `textarea::busy_and_loading_paint_the_readiness_spinner`;
/// `textarea::the_icon_slot_replaces_the_readiness_spinner`.
///
/// ## Invariants
/// `draw` never commits, cancels or validates (it takes `&TextAreaState`);
/// the hardware cursor is written only while editing and focused; the
/// vertical offset is owned by [`ScrollState`] and clamped by it, never by
/// arithmetic in `draw`.
pub struct TextArea<'a> {
    id: Id,
    rows: u16,
    value: Option<&'a str>,
    placeholder: Option<&'a str>,
    validate: Option<&'a dyn Validate>,
    blur: BlurPolicy,
    read_only: bool,
    disabled: bool,
    status: crate::collection::Status,
    ov: Overrides<'a>,
}

impl fmt::Debug for TextArea<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextArea")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .field("value", &self.value.map(|_| "[redacted]"))
            .field("placeholder", &self.placeholder)
            .field("blur", &self.blur)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> TextArea<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::FIELD,
        Part::TEXT,
        Part::PLACEHOLDER,
        Part::ROW,
        Part::MARKER,
        Part::GUTTER,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
    ];

    /// A text area `rows` rows tall.
    pub const fn new(id: Id, rows: u16) -> Self {
        TextArea {
            id,
            rows,
            value: None,
            placeholder: None,
            validate: None,
            blur: BlurPolicy::Commit,
            read_only: false,
            disabled: false,
            status: crate::collection::Status::Ready,
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

    /// The validator run on commit under [`BlurPolicy::CommitAndValidate`].
    #[must_use]
    pub const fn validate(mut self, v: &'a dyn Validate) -> Self {
        self.validate = Some(v);
        self
    }

    /// What focus loss does to a draft; the default is
    /// [`BlurPolicy::Commit`].
    #[must_use]
    pub const fn blur(mut self, p: BlurPolicy) -> Self {
        self.blur = p;
        self
    }

    /// The height of the text region, in rows.
    #[must_use]
    pub const fn rows(mut self, n: u16) -> Self {
        self.rows = n;
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
    pub const fn status(mut self, s: crate::collection::Status) -> Self {
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

    /// Rows of text `area` can hold.
    const fn body_rows(&self, area: Rect) -> u16 {
        if self.rows < area.height {
            self.rows
        } else {
            area.height
        }
    }

    /// Columns between the gutter indent and the right pad.
    ///
    /// The three columns are the gutter, its one-cell indent, and the
    /// trailing pad — reserved unconditionally, on every frame, so the
    /// error marker and the readiness spinner that share it never move the
    /// text (§29 Q1's geometry discipline).
    const fn inner_width(width: u16) -> u16 {
        width.saturating_sub(3)
    }

    /// The update phase: drains this control's intents and drives the edit
    /// lifecycle. The controlled `value` is written on commit only.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextAreaState,
        value: &mut String,
    ) -> Response<TextAction> {
        let mut acc = Acc::<TextAction>::new();
        let editable = self.editable();
        let lines = if st.is_editing() {
            st.draft.line_count()
        } else {
            line_count(value)
        };
        let scroll = ScrollRegion::new(self.id);
        let track_len = if self.disabled {
            None
        } else {
            Some(scroll.prepare(cx, &mut st.scroll, lines))
        };
        let page = st.scroll.viewport_len().max(1);
        for it in cx.intents(self.id) {
            if let Some(track_len) = track_len {
                let bar = scroll.handle_intent(cx, &mut st.scroll, track_len, it);
                acc.fold(&bar);
            }
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
                    if !st.is_editing() && !k.is(KeyCode::Esc) {
                        st.begin(value);
                    }
                    if st.is_editing() {
                        self.edit_key(st, value, k, page, &mut acc);
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
                Intent::Pointer { part, .. }
                    if part.part == Part::TRACK || part.part == Part::THUMB => {}
                Intent::Pointer {
                    phase: Phase::Press | Phase::Click,
                    local,
                    ..
                } if editable => {
                    if !st.is_editing() {
                        st.begin(value);
                    }
                    let line = st
                        .scroll
                        .offset()
                        .saturating_add(usize::from(local.y))
                        .min(st.draft.line_count().saturating_sub(1));
                    let col = usize::from(local.x.saturating_sub(2))
                        .saturating_add(usize::from(st.draft.hscroll()));
                    st.draft.set_cursor_line_col(line, col);
                    acc.changed();
                }
                Intent::Pointer { .. } => acc.consumed(),
                Intent::Cancel if st.is_editing() => {
                    // Esc reaching the control after a layer closed keeps the
                    // document: the `Commit` blur policy is what a text area
                    // means by "leaving" (§15).
                    let _ = st.blur(value, &self.validator(), self.blur);
                    acc.action(TextAction::Committed);
                }
                _ => {}
            }
        }
        if st.is_editing() {
            let cur = st.draft.cursor_pos();
            if !self.disabled {
                st.scroll.set_content(st.draft.line_count());
                st.scroll.ensure_visible(cur.line);
            }
            if let Some(a) = cx.area(self.id) {
                st.draft.scroll_into_view(Self::inner_width(a.width));
            }
        }
        acc.finish(self.id)
    }

    fn live_validate(&self, st: &mut TextAreaState) {
        if st.error.is_some() {
            st.error = self.validator().check(st.draft.text()).err();
        }
    }

    fn edit_key(
        &self,
        st: &mut TextAreaState,
        value: &mut String,
        k: Key,
        page: usize,
        acc: &mut Acc<TextAction>,
    ) {
        match Binding::lookup(BINDINGS, &k) {
            Some(TextCmd::Cancel) => {
                st.cancel();
                acc.action(TextAction::Cancelled);
            }
            Some(TextCmd::Commit) => {
                // Esc commits a document (legacy `textarea::on_key`), and the
                // policy decides whether the validator runs.
                let _ = match self.blur {
                    BlurPolicy::CommitAndValidate => st.commit(value, &self.validator()),
                    _ => st.blur(value, &self.validator(), BlurPolicy::Commit),
                };
                acc.action(TextAction::Committed);
            }
            Some(cmd) => {
                let outcome = match cmd {
                    TextCmd::PageUp | TextCmd::PageDown => {
                        let up = cmd == TextCmd::PageUp;
                        let m = if up { Motion::Up } else { Motion::Down };
                        let mut out = EditOutcome::Ignored;
                        for _ in 0..page {
                            let step = st.apply(EditAction::Move(m, Extend::No));
                            if step.is_visible() {
                                out = step;
                            }
                        }
                        out
                    }
                    other => {
                        let action = match other {
                            TextCmd::Move(m, e) => EditAction::Move(m, e),
                            TextCmd::Newline => EditAction::Newline,
                            TextCmd::Backspace => EditAction::Backspace,
                            TextCmd::Delete => EditAction::Delete,
                            TextCmd::DeleteWordLeft => EditAction::DeleteWordLeft,
                            TextCmd::DeleteToLineEnd => EditAction::DeleteToLineEnd,
                            TextCmd::DeleteToLineStart => EditAction::DeleteToLineStart,
                            TextCmd::SelectAll => EditAction::SelectAll,
                            TextCmd::Cancel
                            | TextCmd::Commit
                            | TextCmd::PageUp
                            | TextCmd::PageDown => EditAction::ClearSelection,
                        };
                        st.apply(action)
                    }
                };
                match outcome {
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
                    if st.apply(EditAction::Insert(c)).changed() {
                        self.live_validate(st);
                        acc.action(TextAction::Changed);
                    } else {
                        acc.consumed();
                    }
                }
            }
        }
    }

    /// The draw phase: the body fill, the gutter column, the visible lines,
    /// the selection run, the scrollbar, the cursor request and the trailing
    /// readiness affordance.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the body, the visible lines and the shared trailing cell"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextAreaState) -> Rect {
        let rows = self.body_rows(area);
        let body = Rect {
            height: rows,
            ..area
        };
        if body.is_empty() {
            return first_row(body);
        }
        let editing = st.is_editing();
        let error = st.error.is_some() || matches!(self.status, crate::collection::Status::Error);
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
        let ov = self.ov;
        let id = self.id;
        let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
            ov.style(ui, id, Family::TEXTAREA, Variant::DEFAULT, part, flags)
        };
        let field = style(ui, Part::FIELD, live);
        ui.fill(body, field.style);
        let shown = if editing {
            st.draft.text()
        } else {
            self.value.unwrap_or("")
        };
        let lines = line_count(shown);
        let content = ScrollRegion::new(self.id).draw(ui, body, &st.scroll, lines);
        let inner = Rect {
            x: content.x.saturating_add(2),
            y: content.y,
            width: Self::inner_width(content.width),
            height: content.height,
        };
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        if !self.ov.is_forced() {
            ui.register_editor(self.id, body, focusability, declared);
        }
        // gutter: one cell per row, the focus bar when the recipe says so
        for row in content.rows() {
            let gutter_cell = cell_at(row, content.x);
            if let Some(f) = ov.slot_for(Part::GUTTER) {
                f(ui, gutter_cell);
            } else {
                let g = style(ui, Part::GUTTER, live);
                match g.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(gutter_cell, glyph, g.style);
                    }
                    Slot::Inherit | Slot::Clear => ui.fill(gutter_cell, g.style),
                }
            }
        }
        if inner.is_empty() {
            return body;
        }
        if shown.is_empty() && !editing {
            if let Some(p) = self.placeholder {
                let ps = style(ui, Part::PLACEHOLDER, live);
                match ov.slot_for(Part::PLACEHOLDER) {
                    Some(f) => f(ui, first_row(inner)),
                    None => {
                        ui.paint_str(first_row(inner), p, ps.style);
                    }
                }
            }
        } else {
            let ts = style(ui, Part::TEXT, live);
            let sel_style = style(ui, Part::ROW, live | StateFlags::SELECTED).style;
            let view = ScrollRegion::view(&st.scroll, content, lines);
            let hs = usize::from(if editing { st.draft.hscroll() } else { 0 });
            let sel = if editing { st.draft.selection() } else { None };
            let mut start = 0usize;
            for (i, line) in shown.split('\n').enumerate() {
                let end = start.saturating_add(line.len());
                if view.visible_range().contains(&i) {
                    let y = inner.y.saturating_add(
                        i.saturating_sub(view.offset()).min(usize::from(u16::MAX)) as u16,
                    );
                    let row = Rect {
                        y,
                        height: 1,
                        ..inner
                    };
                    let total = usize::from(width(line));
                    let overflow = total > hs.saturating_add(usize::from(inner.width));
                    let run = Rect {
                        width: if overflow {
                            row.width.saturating_sub(1)
                        } else {
                            row.width
                        },
                        ..row
                    };
                    let from = byte_at_col(line, hs);
                    ui.paint_str(run, line.get(from..).unwrap_or(""), ts.style);
                    if overflow {
                        ui.glyph(
                            cell_at(row, row.right().saturating_sub(1)),
                            GlyphRole::Ellipsis,
                            ts.style,
                        );
                    }
                    if let Some(r) = &sel
                        && r.start < end.saturating_add(1)
                        && r.end > start
                    {
                        let a = r.start.max(start).saturating_sub(start);
                        let b = r.end.min(end).saturating_sub(start);
                        let x0 = usize::from(width(line.get(..a).unwrap_or("")));
                        let x1 = usize::from(width(line.get(..b).unwrap_or("")));
                        let sub = column_span(run, x0.saturating_sub(hs), x1.saturating_sub(hs));
                        ui.paint_style(sub, sel_style);
                    }
                }
                start = end.saturating_add(1);
            }
            if editing && live.contains(StateFlags::FOCUSED) && self.editable() {
                let cur = st.draft.cursor_pos();
                if cur.line >= view.offset() {
                    let y = inner.y.saturating_add(
                        cur.line
                            .saturating_sub(view.offset())
                            .min(usize::from(u16::MAX)) as u16,
                    );
                    let x = inner.x.saturating_add(
                        cur.col.saturating_sub(hs).min(usize::from(u16::MAX)) as u16,
                    );
                    if y < inner.bottom() {
                        ui.set_cursor(self.id, Position::new(x.min(inner.right()), y));
                    }
                }
            }
        }
        // The trailing pad column `inner_width` reserves on every frame
        // carries the readiness affordance §11.4 obliges a component that
        // accepts `.status(…)` to render. The error glyph and the spinner
        // share it, error winning, exactly as in `TextInput`; the spinner is
        // a *symbol*, so it survives `Mono` without a theme rule.
        let trailing = cell_at(
            first_row(content),
            content.right().saturating_sub(1).max(content.x),
        );
        if error {
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, trailing);
            } else {
                let ms = style(ui, Part::MARKER, live);
                match ms.glyph {
                    Slot::Set(g) => {
                        ui.glyph(trailing, g, ms.style);
                    }
                    Slot::Inherit | Slot::Clear => {}
                }
            }
        } else if live.intersects(StateFlags::BUSY | StateFlags::LOADING) {
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, trailing);
            } else {
                let is = style(ui, Part::ICON, live);
                let frames = ui.design().motion.spinner_frames;
                let frame = frames.first().copied().unwrap_or("");
                ui.paint_str(trailing, frame, is.style);
            }
        }
        body
    }

    /// The natural size: `rows` rows, twelve columns minimum, forty
    /// preferred.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, self.rows.max(1)),
            preferred: (40, self.rows.max(1)),
        }
        .fit(c)
    }
}

/// `line`'s sub-rect between display columns `a` and `b`.
fn column_span(row: Rect, a: usize, b: usize) -> Rect {
    let a = a.min(usize::from(u16::MAX)) as u16;
    let b = b.min(usize::from(u16::MAX)) as u16;
    Rect {
        x: row.x.saturating_add(a),
        y: row.y,
        width: b.saturating_sub(a),
        height: 1,
    }
    .intersection(row)
}

/// Lines in `s`, counting the trailing empty line a trailing newline makes.
fn line_count(s: &str) -> usize {
    s.split('\n').count()
}

/// A borrowed validator behind the blanket-impl bound.
struct Dyn<'a>(&'a dyn Validate);

impl Validate for Dyn<'_> {
    fn check(&self, s: &str) -> Result<(), FieldError> {
        self.0.check(s)
    }
}

impl Bindings for TextArea<'_> {
    type Cmd = TextCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TextCmd>] {
        BINDINGS
    }
}

impl FieldControl for TextArea<'_> {
    type State = TextAreaState;

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextAreaState) -> Rect {
        TextArea::draw(self, ui, area, st)
    }

    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        TextArea::measure(self, ui, c)
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

    const ID: Id = Id::root("textarea.tests");

    fn always_bad(_s: &str) -> Result<(), FieldError> {
        Err(FieldError::new("never valid"))
    }

    /// §16.1: a text area's blur policy is `Commit`, not
    /// `CommitAndValidate` — a document is written out when focus leaves it
    /// and the validator does not run, so a half-typed paragraph is never
    /// rejected on the way out (§15).
    #[test]
    fn blur_commits_without_validation() {
        let mut value = "first".to_owned();
        let mut st = TextAreaState::default();
        st.begin(&value);
        assert!(st.is_editing());
        assert_eq!(st.apply(EditAction::Newline), EditOutcome::Changed);
        assert_eq!(st.apply(EditAction::Insert('x')), EditOutcome::Changed);
        assert!(
            st.blur(&mut value, &always_bad, BlurPolicy::Commit).is_ok(),
            "the default policy must not run the validator"
        );
        assert_eq!(value, "first\nx");
        assert!(st.error().is_none());
        assert!(!st.is_editing());
        // the opt-in policy does validate, and records the error
        st.begin(&value);
        assert!(
            st.blur(&mut value, &always_bad, BlurPolicy::CommitAndValidate)
                .is_err()
        );
        assert!(st.error().is_some());
        assert!(!format!("{st:?}").contains("first"));
    }

    #[test]
    fn vertical_motion_and_page_moves_stay_in_the_document() {
        let mut st = TextAreaState::default();
        st.begin("a\nbb\nccc");
        assert_eq!(st.draft.line_count(), 3);
        assert_eq!(
            st.apply(EditAction::Move(Motion::DocStart, Extend::No)),
            EditOutcome::Moved
        );
        assert_eq!(st.draft.cursor_pos().line, 0);
        assert_eq!(
            st.apply(EditAction::Move(Motion::DocEnd, Extend::No)),
            EditOutcome::Moved
        );
        assert_eq!(st.draft.cursor_pos().line, 2);
        assert_eq!(
            st.apply(EditAction::Move(Motion::Up, Extend::No)),
            EditOutcome::Moved
        );
        assert_eq!(st.draft.cursor_pos().line, 1);
        assert_eq!(line_count("a\nb\n"), 3);
        assert_eq!(column_span(Rect::new(4, 0, 10, 1), 2, 5).x, 6);
        assert_eq!(column_span(Rect::new(4, 0, 10, 1), 2, 5).width, 3);
    }

    /// Draw a four-row text area at `status` over the stub screen, with an
    /// optional forced state (`.state_override`).
    fn draw_with_forced(status: crate::collection::Status, forced: Option<StateFlags>) -> Buffer {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextAreaState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            let mut t = TextArea::new(ID, 4).value("hello").status(status);
            if let Some(f) = forced {
                t = t.state_override(f);
            }
            t.draw(ui, a, &st);
        });
        buf
    }

    /// Draw a four-row text area at `status` over the stub screen.
    fn draw_with(status: crate::collection::Status) -> Buffer {
        draw_with_forced(status, None)
    }

    /// The symbol painted at `(x, y)`.
    fn symbol_at(buf: &Buffer, x: u16, y: u16) -> String {
        buf.cell(Position::new(x, y))
            .map_or_else(String::new, |c| c.symbol().to_owned())
    }

    /// The columns of row 0 left of the reserved trailing pad.
    fn text_run(buf: &Buffer) -> String {
        (0..SCREEN.width - 1)
            .map(|x| symbol_at(buf, x, 0))
            .collect()
    }

    /// The trailing pad column of the first body row — the cell
    /// `inner_width`'s `- 3` already reserves and the error marker already
    /// uses.
    const READINESS_X: u16 = SCREEN.width - 1;

    /// §11.4: a component that accepts `.status(…)` must render readiness.
    /// `BUSY` and `LOADING` paint `design.motion.spinner_frames[0]` into the
    /// trailing pad column, which `inner_width` reserves on **every** frame,
    /// so the text run does not move (§29 Q1's geometry discipline).
    #[test]
    fn busy_and_loading_paint_the_readiness_spinner() {
        let design = Theme::junie().design;
        let frame = design.motion.spinner_frames.first().copied().unwrap();
        let ready = draw_with(crate::collection::Status::Ready);
        for status in [
            crate::collection::Status::Busy,
            crate::collection::Status::Loading,
        ] {
            let buf = draw_with(status);
            assert_eq!(
                symbol_at(&buf, READINESS_X, 0),
                frame,
                "{status:?}: the readiness affordance was not painted"
            );
            assert_eq!(
                text_run(&buf),
                text_run(&ready),
                "{status:?}: the affordance moved the text"
            );
            // `Overrides::flags` *replaces* the live flags with the forced
            // state, so a rule that read only those flags would be
            // unreachable under `.state_override`. `draw` or-s the status
            // flags back in afterwards, so the affordance still fires.
            let forced = draw_with_forced(status, Some(StateFlags::FOCUSED));
            assert_eq!(
                symbol_at(&forced, READINESS_X, 0),
                frame,
                "{status:?}: unreachable under a forced state"
            );
        }
    }

    /// §11.4: `ERROR` keeps the marker glyph in that same cell — the error
    /// glyph wins the shared cell, as it does in `TextInput`.
    ///
    /// This assertion held **before** the readiness spinner was added (the
    /// `MARKER` path is pre-existing), so it certifies the existing error
    /// affordance and guards it against the spinner stealing the cell; it is
    /// not evidence for the spinner itself.
    #[test]
    fn error_keeps_the_marker_glyph_in_the_readiness_cell() {
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);
        let buf = draw_with(crate::collection::Status::Error);
        assert_eq!(symbol_at(&buf, READINESS_X, 0), glyph);
    }

    /// §11.4: a ready text area paints no readiness affordance at all — the
    /// reserved pad column stays blank.
    ///
    /// Like the `ERROR` case this held before the spinner was added; its
    /// value is as the negative half of the busy assertion.
    #[test]
    fn ready_paints_no_readiness_affordance() {
        let design = Theme::junie().design;
        let cell = symbol_at(&draw_with(crate::collection::Status::Ready), READINESS_X, 0);
        assert_ne!(cell, design.motion.spinner_frames.first().copied().unwrap());
        assert_ne!(cell, design.glyphs.get(GlyphRole::Error));
        assert_eq!(cell, " ", "the reserved pad column must stay blank");
    }

    /// §12.1: the readiness affordance resolves through the slot path, so
    /// `.slot(Part::ICON, …)` replaces it.
    #[test]
    fn the_icon_slot_replaces_the_readiness_spinner() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextAreaState::default();
        let icon = |ui: &mut Ui<'_>, r: Rect| {
            let s = ui.surface_style();
            ui.paint_str(r, "Z", s);
        };
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            TextArea::new(ID, 4)
                .value("hello")
                .status(crate::collection::Status::Busy)
                .slot(Part::ICON, &icon)
                .draw(ui, a, &st);
        });
        assert_eq!(
            symbol_at(&buf, READINESS_X, 0),
            "Z",
            "the ICON slot did not replace the readiness affordance"
        );
    }
}
