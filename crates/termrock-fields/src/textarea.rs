use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::SecretPolicy;
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::field_control::FieldControl;
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::input::{
    BlurPolicy, EditCapabilities, EditMenuRequest, EditPhase, EditorDraft, ErrorState, TextAction,
    TextCmd, TextEditCommand, TextTarget, byte_at_col, discard_error,
};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::text::measure::graphemes;
use crate::text::{EditAction, EditOutcome, Extend, Motion, width};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use crate::validate::{FieldError, NoValidate, Validate};
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::layer::LayerOpening;
use termrock_runtime::rowui::CellUi;
use termrock_runtime::scroll_region::ScrollRegion;

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

/// Geometry shared by TextArea drawing and coordinate-bearing input.
///
/// `window` describes the component body's visible rows. `text` is the
/// registered text part inside that body, so pointer, cursor and menu mapping
/// use the same physical origin and clipping bounds.
#[derive(Clone, Copy, Debug)]
struct TextAreaGeometry {
    window: VerticalWindow,
    text: Rect,
}

impl TextAreaGeometry {
    fn new(window: VerticalWindow, text: Rect) -> Option<Self> {
        let text = text.intersection(window.visible_body());
        (!text.is_empty()).then_some(Self { window, text })
    }

    fn document_point(
        self,
        pos: Position,
        child_scroll: usize,
        horizontal_scroll: u16,
    ) -> Option<(usize, usize)> {
        if !self.text.contains(pos) {
            return None;
        }
        let source_row = self.window.source_row_for_physical(pos.y)?;
        let line = child_scroll.checked_add(source_row)?;
        let col = usize::from(pos.x.saturating_sub(self.text.x))
            .saturating_add(usize::from(horizontal_scroll));
        Some((line, col))
    }

    /// Project an active captured drag through the parent window. Unlike a
    /// press, a captured drag remains meaningful after leaving `TEXT`; the
    /// old logical child behavior extended to the document edge from the
    /// captured origin. Clamp above the visible window to its first row, but
    /// preserve vertical extrapolation below it so the caller can clamp to the
    /// document edge rather than the parent's logical viewport.
    fn captured_document_point(
        self,
        pos: Position,
        child_scroll: usize,
        horizontal_scroll: u16,
    ) -> Option<(usize, usize)> {
        let logical_rows = self.window.logical_rows();
        if logical_rows == 0 {
            return None;
        }
        let visible_body = self.window.visible_body();
        let physical_delta = usize::from(pos.y.saturating_sub(visible_body.y));
        let source_row = self
            .window
            .first_source_row()
            .saturating_add(physical_delta);
        let line = child_scroll.checked_add(source_row)?;
        let col = usize::from(pos.x.saturating_sub(self.text.x))
            .saturating_add(usize::from(horizontal_scroll));
        Some((line, col))
    }

    fn physical_row(self, document_row: usize, child_scroll: usize) -> Option<u16> {
        let source_row = document_row.checked_sub(child_scroll)?;
        let y = self.window.physical_row_for_source(source_row)?;
        if y < self.text.y || y >= self.text.bottom() {
            return None;
        }
        Some(y)
    }

    fn cursor_position(
        self,
        cursor: crate::text::CursorPos,
        child_scroll: usize,
        horizontal_scroll: u16,
    ) -> Option<Position> {
        let y = self.physical_row(cursor.line, child_scroll)?;
        let x = self.text.x.checked_add(
            u16::try_from(cursor.col.saturating_sub(usize::from(horizontal_scroll)))
                .unwrap_or(u16::MAX),
        )?;
        (x <= self.text.right()).then_some(Position::new(x, y))
    }

    fn logical_inner_width(self) -> u16 {
        // Keep the established editing contract: horizontal scrolling uses
        // the body width minus the left gutter and the trailing cell. The
        // vertical scrollbar does not reduce this logical editor width.
        self.window.visible_body().width.saturating_sub(3)
    }
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
        "Line start (Ctrl+A)",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('e'), CTRL),
        TextCmd::Move(Motion::End, Extend::No),
        "Line end (Ctrl+E)",
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

#[derive(Default)]
pub struct TextAreaState {
    draft: EditorDraft,
    phase: EditPhase,
    scroll: ScrollState,
    error: Option<ErrorState>,
    redacted_snapshot: bool,
    sensitivity: Option<bool>,
    edit_menu_request: Option<EditMenuRequest>,
    pointer_anchor: Option<usize>,
    focus_continuation: Option<LayerOpening>,
}

impl Clone for TextAreaState {
    fn clone(&self) -> Self {
        TextAreaState {
            draft: self.draft.clone_snapshot(),
            phase: self.phase,
            scroll: self.scroll,
            error: self.error.as_ref().map(ErrorState::clone_snapshot),
            redacted_snapshot: self.is_sensitive(),
            sensitivity: self.sensitivity,
            edit_menu_request: self.edit_menu_request,
            pointer_anchor: self.pointer_anchor,
            focus_continuation: self.focus_continuation,
        }
    }
}

impl PartialEq for TextAreaState {
    fn eq(&self, other: &Self) -> bool {
        if self.is_sensitive() || other.is_sensitive() {
            self.is_sensitive() == other.is_sensitive()
                && self.phase == other.phase
                && self.scroll == other.scroll
                && self.pointer_anchor == other.pointer_anchor
                && self.focus_continuation == other.focus_continuation
                && self.error.as_ref().map(ErrorState::is_sensitive)
                    == other.error.as_ref().map(ErrorState::is_sensitive)
        } else {
            self.draft.same(&other.draft)
                && self.phase == other.phase
                && self.scroll == other.scroll
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

impl Eq for TextAreaState {}

impl fmt::Debug for TextAreaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaState")
            .field("draft", &"[redacted]")
            .field("draft_len", &self.draft.text().len())
            .field("phase", &self.phase)
            .field("scroll", &self.scroll)
            .field("error", &self.error.as_ref().map(|_| "[redacted]"))
            .field("redacted_snapshot", &self.redacted_snapshot)
            .field("sensitivity", &self.sensitivity)
            .field("pointer_anchor", &self.pointer_anchor)
            .field("focus_continuation", &self.focus_continuation)
            .field("sensitive", &self.is_sensitive())
            .finish()
    }
}

impl TextAreaState {
    #[must_use]
    pub fn sensitive() -> Self {
        let mut state = Self::default();
        state.set_sensitive(true);
        state
    }

    pub const fn is_editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing)
    }

    /// Take a pending contextual edit-menu request, if one was made this update.
    pub fn take_edit_menu_request(&mut self) -> Option<EditMenuRequest> {
        self.edit_menu_request.take()
    }

    pub const fn phase(&self) -> EditPhase {
        self.phase
    }

    pub(crate) const fn is_sensitive(&self) -> bool {
        self.draft.is_sensitive()
    }

    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    pub const fn scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll
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
        self.draft.begin_multi(current);
        self.phase = EditPhase::Editing;
    }

    pub fn commit(&mut self, value: &mut String, v: &impl Validate) -> Result<(), FieldError> {
        self.commit_target(value, v)
    }

    fn commit_target<T: TextTarget + ?Sized>(
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

    fn blur_target<T: TextTarget + ?Sized>(
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

pub struct TextArea<'a> {
    id: Id,
    rows: u16,
    value: Option<&'a str>,
    placeholder: Option<&'a str>,
    validate: Option<&'a dyn Validate>,
    blur: BlurPolicy,
    secret: Option<SecretPolicy>,
    read_only: bool,
    disabled: bool,
    status: termrock_runtime::Status,
    ov: PartStyle<'a>,
}

impl fmt::Debug for TextArea<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextArea")
            .field("id", &self.id)
            .field("rows", &self.rows)
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

impl<'a> TextArea<'a> {
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
        Part::CONTAINER,
    ];

    pub const fn new(id: Id, rows: u16) -> Self {
        TextArea {
            id,
            rows,
            value: None,
            placeholder: None,
            validate: None,
            blur: BlurPolicy::Commit,
            secret: None,
            read_only: false,
            disabled: false,
            status: termrock_runtime::Status::Ready,
            ov: PartStyle::new(),
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
    pub const fn secret(mut self, policy: SecretPolicy) -> Self {
        self.secret = Some(policy);
        self
    }

    pub const fn is_secret(&self) -> bool {
        self.secret.is_some()
    }

    #[must_use]
    pub const fn rows(mut self, n: u16) -> Self {
        self.rows = n;
        self
    }

    pub const fn rows_in_form(&self) -> u16 {
        self.rows
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
    pub const fn status(mut self, s: termrock_runtime::Status) -> Self {
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

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn with_inherited_disabled(&self, inherited: bool) -> Self {
        TextArea {
            id: self.id,
            rows: self.rows,
            value: self.value,
            placeholder: self.placeholder,
            validate: self.validate,
            blur: self.blur,
            secret: self.secret,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            status: self.status,
            ov: self.ov,
        }
    }

    const fn scroll_region(&self) -> ScrollRegion<'a> {
        ScrollRegion::new(self.id)
    }

    fn validator(&self) -> Dyn<'_> {
        Dyn(self.validate.unwrap_or(&NoValidate))
    }

    const fn body_rows(&self, area: Rect) -> u16 {
        if self.rows < area.height {
            self.rows
        } else {
            area.height
        }
    }

    const fn inner_width(width: u16) -> u16 {
        width.saturating_sub(3)
    }

    fn prepared_geometry(cx: &Cx<'_>, id: Id) -> Option<TextAreaGeometry> {
        cx.layout(id)
            .and_then(|facts| facts.vertical_window)
            .zip(cx.area_of_part(id, PartRef::of(Part::TEXT)))
            .and_then(|(window, text)| TextAreaGeometry::new(window, text))
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextAreaState,
        value: &mut String,
    ) -> Response<TextAction> {
        self.update_target(cx, st, value)
    }

    pub fn update_in_form<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextAreaState,
        value: &mut T,
        inherited_disabled: bool,
    ) -> Response<TextAction> {
        self.with_inherited_disabled(inherited_disabled)
            .update_target(cx, st, value)
    }

    pub fn commit_in_form<T: TextTarget + ?Sized>(
        &self,
        st: &mut TextAreaState,
        value: &mut T,
    ) -> bool {
        if !st.is_editing() {
            return false;
        }
        let _ = st.blur_target(value, &self.validator(), BlurPolicy::Commit);
        true
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over focus, binding, paste, pointer and scroll intents"
    )]
    fn update_target<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextAreaState,
        value: &mut T,
    ) -> Response<TextAction> {
        st.set_sensitive(self.secret.is_some() || value.is_sensitive());
        let mut acc = Acc::<TextAction>::new();
        let editable = self.editable();
        st.edit_menu_request = None;
        let lines = if st.is_editing() {
            st.draft.line_count()
        } else {
            line_count(value.expose())
        };
        let scroll = self.scroll_region();
        let geometry = Self::prepared_geometry(cx, self.id);
        // Pointer coordinates target the last presented frame. FocusIn can
        // begin an edit and reveal a different document row during this same
        // update, so retain the viewport origin before processing intents.
        let displayed_scroll = st.scroll.offset();
        let displayed_hscroll = st.draft.hscroll();
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
                        st.begin(value.expose());
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
                        if !st.is_editing() && cmd != TextCmd::Commit {
                            st.begin(value.expose());
                        }
                        if st.is_editing() {
                            self.edit_command(st, value, cmd, page, &mut acc);
                        }
                    }
                }
                Intent::Key(k)
                    if !self.disabled
                        && (k.code == KeyCode::Menu
                            || (k.code == KeyCode::F(10)
                                && k.mods.contains(KeyModifiers::SHIFT))) =>
                {
                    if let Some(anchor) = geometry.and_then(|g| {
                        g.cursor_position(
                            st.draft.cursor_pos(),
                            st.scroll.offset(),
                            st.draft.hscroll(),
                        )
                    }) {
                        st.edit_menu_request = Some(EditMenuRequest {
                            owner: self.id,
                            part: PartRef::of(Part::TEXT),
                            anchor,
                        });
                    }
                    acc.consumed();
                }
                Intent::Key(k) if editable => {
                    if !st.is_editing() {
                        st.begin(value.expose());
                    }
                    if let Some(c) = k.bare_char()
                        && st.apply(EditAction::Insert(c)).changed()
                    {
                        self.live_validate(st);
                        acc.action(TextAction::Changed);
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
                    phase: Phase::Press,
                    part,
                    pos,
                    ..
                } if editable || (self.read_only && !self.disabled) => {
                    if let Some((line, col)) = geometry.and_then(|geometry| {
                        geometry.document_point(pos, displayed_scroll, displayed_hscroll)
                    }) {
                        if !st.is_editing() {
                            st.begin(value.expose());
                        }
                        st.pointer_anchor = Some(st.draft.pointer_select(
                            line.min(st.draft.line_count().saturating_sub(1)),
                            col,
                            cx.pointer_press_count(self.id, part).unwrap_or(1),
                        ));
                        acc.changed();
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase: Phase::DragStart | Phase::Drag,
                    pos,
                    ..
                } if st.is_editing() => {
                    if let Some((line, col)) = geometry.and_then(|geometry| {
                        geometry.captured_document_point(pos, displayed_scroll, displayed_hscroll)
                    }) {
                        if let Some(anchor) = st.pointer_anchor {
                            let line = line.min(st.draft.line_count().saturating_sub(1));
                            st.draft.pointer_extend(anchor, line, col);
                            acc.changed();
                        } else {
                            acc.consumed();
                        }
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase: Phase::Release | Phase::DragEnd,
                    ..
                } => {
                    st.pointer_anchor = None;
                    acc.consumed();
                }
                Intent::Pointer {
                    phase: Phase::Secondary,
                    part,
                    pos,
                    ..
                } if !self.disabled => {
                    if let Some((line, col)) = geometry.and_then(|geometry| {
                        geometry.document_point(pos, displayed_scroll, displayed_hscroll)
                    }) {
                        if !st.is_editing() {
                            st.begin(value.expose());
                        }
                        let changed = st
                            .draft
                            .pointer_point(line.min(st.draft.line_count().saturating_sub(1)), col);
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
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer { .. } => acc.consumed(),
                Intent::Cancel if st.is_editing() => {
                    let _ = st.blur_target(value, &self.validator(), self.blur);
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
            if let Some(geometry) = geometry {
                st.draft.scroll_into_view(geometry.logical_inner_width());
            }
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

    /// Report available edit commands without exposing the field's buffer.
    pub fn edit_capabilities<T: TextTarget + ?Sized>(
        &self,
        st: &TextAreaState,
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

    /// Return the focused caret's source row inside this TextArea's logical
    /// visible body, after the document's own scroll offset.
    ///
    /// A containing Form can use this row to reveal the caret in its parent
    /// viewport without reading or changing the document state.
    pub fn caret_source_row(&self, st: &TextAreaState) -> Option<usize> {
        st.is_editing().then(|| {
            st.draft
                .cursor_pos()
                .line
                .saturating_sub(st.scroll.offset())
        })
    }

    /// Execute one contextual edit command through the field's shared editor.
    pub fn execute_edit_command<T: TextTarget + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut TextAreaState,
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
        if applied {
            let cursor = st.draft.cursor_pos();
            st.scroll.set_content(st.draft.line_count());
            st.scroll.ensure_visible(cursor.line);
            if let Some(geometry) = Self::prepared_geometry(cx, self.id) {
                st.draft.scroll_into_view(geometry.logical_inner_width());
            }
        }
        applied
    }

    fn live_validate(&self, st: &mut TextAreaState) {
        if st.error.is_some() {
            let _ = st.finish_validation(self.validator().check(st.draft.text()));
        }
    }

    fn edit_command<T: TextTarget + ?Sized>(
        &self,
        st: &mut TextAreaState,
        value: &mut T,
        cmd: TextCmd,
        page: usize,
        acc: &mut Acc<TextAction>,
    ) {
        match cmd {
            TextCmd::Cancel => {
                st.cancel();
                acc.action(TextAction::Cancelled);
            }
            TextCmd::Commit => {
                let _ = match self.blur {
                    BlurPolicy::CommitAndValidate => st.commit_target(value, &self.validator()),
                    _ => st.blur_target(value, &self.validator(), BlurPolicy::Commit),
                };
                acc.action(TextAction::Committed);
            }
            cmd => {
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
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextAreaState) -> Rect {
        let rows = self.body_rows(area);
        let body = Rect {
            height: rows,
            ..area
        };
        self.draw_window(ui, VerticalWindow::full(body), st)
    }

    /// Draw this TextArea through a physical parent window while preserving
    /// its full logical viewport and document row numbering.
    pub fn draw_window(&self, ui: &mut Ui<'_>, window: VerticalWindow, st: &TextAreaState) -> Rect {
        let body = window.visible_body();
        if body.is_empty() {
            return first_row(body);
        }
        let editing = st.is_editing();
        let validation_error = st.error.is_some();
        let status_error = matches!(self.status, termrock_runtime::Status::Error);
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
        let content = self
            .scroll_region()
            .draw_window(ui, window, &st.scroll, lines);
        let inner = Rect {
            x: content.x.saturating_add(2),
            y: content.y,
            width: Self::inner_width(content.width),
            height: content.height,
        };
        ui.register_decor(self.id, PartRef::of(Part::TEXT), inner);
        let geometry = TextAreaGeometry::new(window, inner);
        ui.register_editor(self.id, body, focusability, declared);
        ui.publish_bindings(self.id, live, BINDINGS);

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
                if let Some(row) = geometry.and_then(|geometry| {
                    geometry.physical_row(0, st.scroll.offset()).map(|y| Rect {
                        y,
                        height: 1,
                        ..inner
                    })
                }) {
                    match ov.slot_for(Part::PLACEHOLDER) {
                        Some(f) => f(ui, first_row(row)),
                        None => {
                            ui.paint_str(first_row(row), p, ps.style);
                        }
                    }
                }
            }
        } else {
            let ts = style(ui, Part::TEXT, live);
            let sel_style = style(ui, Part::ROW, live | StateFlags::SELECTED).style;
            let mut view = st.scroll;
            view.apply_layout(window.logical_rows(), lines);
            let hs = usize::from(if editing { st.draft.hscroll() } else { 0 });
            let sel = if editing { st.draft.selection() } else { None };
            let mut start = 0usize;
            for (i, line) in shown.split('\n').enumerate() {
                let end = start.saturating_add(line.len());
                if view.visible_range().contains(&i)
                    && let Some(y) = geometry.and_then(|g| g.physical_row(i, view.offset()))
                {
                    let row = Rect {
                        y,
                        height: 1,
                        ..inner
                    };
                    let secret_policy = self
                        .secret
                        .or_else(|| st.is_sensitive().then_some(SecretPolicy::default()));
                    let total = if secret_policy.is_some() {
                        graphemes(line).count()
                    } else {
                        usize::from(width(line))
                    };
                    let overflow = total > hs.saturating_add(usize::from(inner.width));
                    let run = Rect {
                        width: if overflow {
                            row.width.saturating_sub(1)
                        } else {
                            row.width
                        },
                        ..row
                    };
                    if let Some(policy) = secret_policy {
                        paint_masked_line(ui, run, line, hs, policy, ts.style);
                    } else {
                        let from = byte_at_col(line, hs);
                        ui.paint_str(run, line.get(from..).unwrap_or(""), ts.style);
                    }
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
            if editing
                && live.contains(StateFlags::FOCUSED)
                && self.editable()
                && let Some(cursor) = geometry.and_then(|g| {
                    g.cursor_position(st.draft.cursor_pos(), view.offset(), hs as u16)
                })
            {
                ui.set_cursor(self.id, cursor);
            }
        }

        let trailing = cell_at(
            first_row(content),
            content.right().saturating_sub(1).max(content.x),
        );
        if validation_error {
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
        } else if status_error {
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, trailing);
            } else {
                let is = style(ui, Part::ICON, live);
                match is.glyph {
                    Slot::Set(g) => {
                        ui.glyph(trailing, g, is.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(trailing, GlyphRole::Error, is.style);
                    }
                    Slot::Clear => ui.fill(trailing, is.style),
                }
            }
        } else if matches!(
            self.status,
            termrock_runtime::Status::Busy | termrock_runtime::Status::Loading
        ) {
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

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &TextAreaState,
        value: &str,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .value(value)
            .draw(ui, area, st)
    }

    /// Draw a non-secret form value through its parent's clipped body window.
    pub fn draw_in_form_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &TextAreaState,
        value: &str,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .value(value)
            .draw_window(ui, window, st)
    }

    pub fn draw_secret_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &TextAreaState,
        value: &crate::secret::Secret,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .value(value.expose())
            .secret(self.secret.unwrap_or_default())
            .draw(ui, area, st)
    }

    /// Draw a secret form value through its parent's clipped body window.
    pub fn draw_secret_in_form_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &TextAreaState,
        value: &crate::secret::Secret,
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .value(value.expose())
            .secret(self.secret.unwrap_or_default())
            .draw_window(ui, window, st)
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, self.rows.max(1)),
            preferred: (40, self.rows.max(1)),
        }
        .fit(c)
    }
}

fn paint_masked_line(
    ui: &mut Ui<'_>,
    run: Rect,
    line: &str,
    skip: usize,
    policy: SecretPolicy,
    style: crate::theme::PaintStyle,
) {
    let total = graphemes(line).count().saturating_sub(skip);
    let mut cells = CellUi::new(ui.reborrow(), run, style);
    cells.glyphs(policy.mask, total);
}

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

fn line_count(s: &str) -> usize {
    s.split('\n').count()
}

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
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::{Input, Key, KeyCode, KeyModifiers};
    use crate::runtime::App;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;
    use crate::ui::layer::{Anchor, LayerOpening, LayerSize, LayerSpec};
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("textarea.tests");
    const CONTEXT_LAYER: Id = Id::root("textarea.context-layer");
    const CONTEXT_TARGET: Id = Id::root("textarea.context-target");

    struct ContinuationTextAreaApp {
        state: TextAreaState,
        value: String,
        opening: Option<LayerOpening>,
    }

    impl App for ContinuationTextAreaApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = TextArea::new(ID, 3).blur(BlurPolicy::Commit).update(
                cx,
                &mut self.state,
                &mut self.value,
            );
            if self.state.take_edit_menu_request().is_some() {
                self.opening = cx.open_layer_with_token(
                    CONTEXT_LAYER,
                    LayerSpec::popover(CONTEXT_LAYER, Anchor::Point(Position::new(3, 0)))
                        .size(LayerSize::Fixed(10, 3))
                        .initial_focus(CONTEXT_TARGET)
                        .continues_focus(ID),
                );
            }
            if cx
                .intents(CONTEXT_TARGET)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')))
            {
                cx.close_layer(CONTEXT_LAYER, None);
            }
            if let Some(opening) = self.opening
                && cx.take_layer_completion(opening).is_some()
            {
                self.opening = None;
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextArea::new(ID, 3)
                .value(&self.value)
                .blur(BlurPolicy::Commit)
                .draw(ui, SCREEN, &self.state);
            let _ = ui.layer(CONTEXT_LAYER, |ui, area| {
                ui.register_control(CONTEXT_TARGET, area, Focusability::Focusable);
            });
        }
    }

    fn publish<A: App>(runtime: &mut Runtime<A>) {
        let prepared = runtime
            .prepare(SCREEN)
            .expect("textarea continuation frame prepares");
        let mut terminal = Terminal::new(TestBackend::new(SCREEN.width, SCREEN.height))
            .expect("test terminal initializes");
        terminal
            .draw(|frame| {
                runtime
                    .draw_prepared(prepared, frame)
                    .expect("prepared textarea continuation frame draws");
            })
            .expect("test backend draws");
        runtime
            .acknowledge_presented(prepared)
            .expect("test backend acknowledges textarea continuation frame");
    }

    #[test]
    fn contextual_edit_capabilities_match_single_line_field_contract() {
        let value = "read only".to_owned();
        let input = TextArea::new(ID, 3).read_only(true);
        let idle = input.edit_capabilities(&TextAreaState::default(), &value);
        assert!(!idle.editable);
        assert!(idle.can_select_all);
        assert!(!idle.can_paste);

        let mut selected = TextAreaState::default();
        selected.begin(&value);
        selected.draft.select_range(0, value.len());
        let selected_caps = input.edit_capabilities(&selected, &value);
        assert!(selected_caps.can_copy);
        assert!(!selected_caps.can_cut);
        assert!(!selected_caps.can_select_all);

        let disabled = TextArea::new(ID, 3)
            .disabled(true)
            .edit_capabilities(&selected, &value);
        assert!(!disabled.editable);
        assert!(!disabled.can_copy);
        assert!(!disabled.can_paste);

        let mut sensitive = TextAreaState::sensitive();
        sensitive.begin(&value);
        sensitive.draft.select_range(0, value.len());
        let secret = TextArea::new(ID, 3).edit_capabilities(&sensitive, &value);
        assert!(!secret.can_copy);
        assert!(!secret.can_cut);
    }

    struct SecretTextAreaApp {
        state: TextAreaState,
        value: String,
    }

    struct SecondaryPasteTextAreaApp {
        state: TextAreaState,
        value: String,
        pasted: bool,
    }

    impl App for SecondaryPasteTextAreaApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let area = TextArea::new(ID, 3);
            let response = area.update(cx, &mut self.state, &mut self.value);
            if self.state.take_edit_menu_request().is_some() && !self.pasted {
                cx.clipboard_copy("X");
                self.pasted = area.execute_edit_command(
                    cx,
                    &mut self.state,
                    &mut self.value,
                    TextEditCommand::Paste,
                );
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextArea::new(ID, 3)
                .value(&self.value)
                .draw(ui, SCREEN, &self.state);
        }
    }

    impl App for SecretTextAreaApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextArea::new(ID, 3)
                .secret(SecretPolicy::default())
                .update(cx, &mut self.state, &mut self.value)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextArea::new(ID, 3)
                .secret(SecretPolicy::default())
                .value(&self.value)
                .draw(ui, SCREEN, &self.state);
        }
    }

    fn secret_state() -> TextAreaState {
        let mut runtime = Runtime::new(
            SecretTextAreaApp {
                state: TextAreaState::default(),
                value: "one\ntwo".to_owned(),
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
        const SECRET: &str = "one\ntwo";
        let mut state = TextAreaState::sensitive();
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

    #[test]
    fn parts_include_every_owned_scroll_region_part() {
        for part in ScrollRegion::PARTS {
            assert!(
                TextArea::PARTS.contains(part),
                "TextArea::PARTS omits owned ScrollRegion part {part:?}"
            );
        }
    }

    #[test]
    fn secondary_pointer_paste_inserts_at_the_clicked_textarea_point() {
        let mut runtime = Runtime::new(
            SecondaryPasteTextAreaApp {
                state: TextAreaState::default(),
                value: "one\ntwo".to_owned(),
                pasted: false,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Mouse(crate::event::Mouse {
                kind: crate::event::MouseKind::Secondary,
                pos: Position::new(3, 1),
                mods: KeyModifiers::NONE,
            }),
        );
        assert!(runtime.app().pasted);
        assert_eq!(runtime.app().state.draft.text(), "one\ntXwo");
    }

    #[test]
    fn continued_edit_menu_restoration_preserves_multiline_field_selection() {
        let mut state = TextAreaState::default();
        state.begin("value");
        state.draft.select_range(1, 4);
        let mut runtime = Runtime::new(
            ContinuationTextAreaApp {
                state,
                value: "value".to_owned(),
                opening: None,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        publish(&mut runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Mouse(crate::event::Mouse {
                kind: crate::event::MouseKind::Secondary,
                pos: Position::new(3, 0),
                mods: KeyModifiers::NONE,
            }),
        );
        publish(&mut runtime);
        assert!(runtime.app().state.is_editing());
        assert_eq!(runtime.app().state.draft.selection(), Some(1..4));
        assert!(runtime.app().opening.is_some());

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            }),
        );
        publish(&mut runtime);
        assert!(runtime.app().state.is_editing());
        assert_eq!(runtime.app().state.draft.selection(), Some(1..4));
        assert_eq!(runtime.app().value, "value");
    }

    #[test]
    fn a_reference_text_area_leaves_the_owned_scroll_region_inert() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextAreaState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ui.reference(
                Some(ReferenceTarget::new(ID, ReferenceState::FOCUSED)),
                |ui| {
                    TextArea::new(ID, 4)
                        .value("one\ntwo\nthree\nfour\nfive")
                        .draw(ui, Rect::new(0, 0, 20, 4), &st);
                },
            );
        })
        .commit_presented();
        for part in ScrollRegion::PARTS {
            assert!(
                rt.area_of_part(ID, PartRef::of(*part)).is_none(),
                "reference TextArea registered owned ScrollRegion part {part:?}"
            );
        }
        assert!(
            !rt.registry().delivers_to(ID),
            "reference TextArea left its owned ScrollRegion interactive"
        );
    }

    fn always_bad(_s: &str) -> Result<(), FieldError> {
        Err(FieldError::new("never valid"))
    }

    #[test]
    fn sensitive_state_masks_when_control_policy_is_removed() {
        const SECRET: &str = "one\ntwo";
        let mut state = secret_state();
        state.begin(SECRET);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let area = Rect::new(0, 0, 24, 3);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                TextArea::new(ID, 3).value(SECRET).draw(ui, area, &state);
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
            "the sensitive area reached the frame"
        );
        assert!(
            frame.matches(mask).count() >= 6,
            "the sensitive area did not paint its mask: {frame}"
        );
    }

    #[test]
    fn sensitive_state_equality_ignores_draft_contents() {
        let mut left = secret_state();
        left.cancel();
        left.begin("one");
        let mut right = secret_state();
        right.cancel();
        right.begin("different");
        assert_eq!(left, right);
        let copy = left.clone();
        assert_eq!(copy.draft.text(), "•••");
        assert!(!copy.draft.text().contains("one"));
    }

    #[test]
    fn sensitive_state_clone_cannot_commit_redacted_draft() {
        let mut state = TextAreaState::default();
        state.set_sensitive(true);
        state.begin("one\ntwo");
        let _ = state.apply(EditAction::Insert('!'));

        let mut copy = state.clone();
        let mut value = String::new();
        copy.commit(&mut value, &NoValidate)
            .expect("a snapshot commit must not validate a redacted draft");
        assert!(value.is_empty());
        assert!(!copy.is_editing());
    }

    #[test]
    fn sensitive_validator_error_is_generic_and_not_retained() {
        const SECRET: &str = "one\ntwo";
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
    fn sensitive_string_target_replaces_the_old_allocation() {
        let mut state = TextAreaState::default();
        state.set_sensitive(true);
        state.begin("new\nsecret");

        let mut value = String::with_capacity(1024);
        value.push_str("old\nsecret");
        let old_capacity = value.capacity();

        state
            .commit(&mut value, &NoValidate)
            .expect("the replacement must not fail validation");

        assert_eq!(value, "new\nsecret");
        assert!(
            value.capacity() < old_capacity,
            "sensitive replacement retained the old caller allocation"
        );
    }

    #[test]
    fn pending_error_is_masked_until_sensitivity_is_reconciled() {
        const DETAIL: &str = "secret validation detail";

        let mut plain = TextAreaState::default();
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

        let mut secret = TextAreaState::default();
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

        st.begin(&value);
        assert!(
            st.blur(&mut value, &always_bad, BlurPolicy::CommitAndValidate)
                .is_err()
        );
        assert!(st.error().is_some());
        assert!(!format!("{st:?}").contains("first"));
    }

    #[test]
    fn secret_blur_keep_policy_leaves_the_draft() {
        let mut st = secret_state();
        st.cancel();
        let mut value = "one".to_owned();
        st.begin(&value);
        let _ = st.apply(EditAction::Insert('!'));
        assert!(st.blur(&mut value, &always_bad, BlurPolicy::Keep).is_ok());
        assert!(st.is_editing(), "secret Keep must preserve the draft");
        assert_eq!(st.draft.text(), "one!");
        assert_eq!(value, "one");
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

    #[test]
    fn selected_style_is_painted_only_for_a_real_selection_range() {
        let render = |selected: bool| {
            let theme = Theme::junie().override_family(Family::TEXTAREA, |recipe| {
                recipe.part(Part::ROW).when(
                    StateFlags::SELECTED,
                    StylePatch::new().set_bg(crate::theme::Role::Danger),
                );
            });
            let mut runtime = Runtime::new(Stub::default(), theme);
            let area = Rect::new(0, 0, 12, 3);
            let mut buffer = Buffer::empty(area);
            let mut state = TextAreaState::default();
            state.begin("hello");
            if selected {
                let _ = state.draft.apply(EditAction::SelectAll);
            }
            runtime
                .draw_scene(area, &mut buffer, |ui, area| {
                    TextArea::new(ID, 3).value("hello").draw(ui, area, &state);
                })
                .commit_presented();
            buffer
        };
        let plain = render(false);
        let selected = render(true);
        for x in 0..12 {
            let changed = plain
                .cell(Position::new(x, 0))
                .map(ratatui_core::buffer::Cell::style)
                != selected
                    .cell(Position::new(x, 0))
                    .map(ratatui_core::buffer::Cell::style);
            assert_eq!(changed, (2..7).contains(&x), "column {x}");
        }
    }

    fn draw_with(status: termrock_runtime::Status) -> Buffer {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = TextAreaState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            TextArea::new(ID, 4)
                .value("hello")
                .status(status)
                .draw(ui, a, &st);
        })
        .commit_presented();
        buf
    }

    fn symbol_at(buf: &Buffer, x: u16, y: u16) -> String {
        buf.cell(Position::new(x, y))
            .map_or_else(String::new, |c| c.symbol().to_owned())
    }

    fn text_run(buf: &Buffer) -> String {
        (0..SCREEN.width - 1)
            .map(|x| symbol_at(buf, x, 0))
            .collect()
    }

    const READINESS_X: u16 = SCREEN.width - 1;

    #[test]
    fn busy_and_loading_paint_the_readiness_spinner() {
        let design = Theme::junie().design;
        let frame = design.motion.spinner_frames.first().copied().unwrap();
        let ready = draw_with(termrock_runtime::Status::Ready);
        for status in [
            termrock_runtime::Status::Busy,
            termrock_runtime::Status::Loading,
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
        }
    }

    #[test]
    fn status_error_paints_the_icon_fallback_in_the_readiness_cell() {
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);
        let buf = draw_with(termrock_runtime::Status::Error);
        assert_eq!(symbol_at(&buf, READINESS_X, 0), glyph);
    }

    #[test]
    fn ready_paints_no_readiness_affordance() {
        let design = Theme::junie().design;
        let cell = symbol_at(&draw_with(termrock_runtime::Status::Ready), READINESS_X, 0);
        assert_ne!(cell, design.motion.spinner_frames.first().copied().unwrap());
        assert_ne!(cell, design.glyphs.get(GlyphRole::Error));
        assert_eq!(cell, " ", "the reserved pad column must stay blank");
    }

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
                .status(termrock_runtime::Status::Busy)
                .slot(Part::ICON, &icon)
                .draw(ui, a, &st);
        })
        .commit_presented();
        assert_eq!(
            symbol_at(&buf, READINESS_X, 0),
            "Z",
            "the ICON slot did not replace the readiness affordance"
        );
    }

    #[test]
    fn validation_marker_wins_the_shared_lane_over_status_error() {
        assert!(TextArea::PARTS.contains(&Part::ICON));
        let marker_calls = Cell::new(0usize);
        let icon_calls = Cell::new(0usize);
        let marker = |ui: &mut Ui<'_>, area: Rect| {
            marker_calls.set(marker_calls.get().saturating_add(1));
            let style = ui.surface_style();
            ui.paint_str(area, "M", style);
        };
        let icon = |ui: &mut Ui<'_>, area: Rect| {
            icon_calls.set(icon_calls.get().saturating_add(1));
            let style = ui.surface_style();
            ui.paint_str(area, "I", style);
        };
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let mut st = TextAreaState::default();
        st.set_error(Some(FieldError::new("invalid")));
        rt.draw_scene(SCREEN, &mut buf, |ui, area| {
            TextArea::new(ID, 4)
                .value("hello")
                .status(termrock_runtime::Status::Error)
                .slot(Part::MARKER, &marker)
                .draw(ui, area, &st);
        })
        .commit_presented();
        assert_eq!(marker_calls.get(), 1);
        assert_eq!(icon_calls.get(), 0);
        assert_eq!(symbol_at(&buf, READINESS_X, 0), "M");

        st.set_error(None);
        rt.draw_scene(SCREEN, &mut buf, |ui, area| {
            TextArea::new(ID, 4)
                .value("hello")
                .status(termrock_runtime::Status::Error)
                .slot(Part::ICON, &icon)
                .draw(ui, area, &st);
        })
        .commit_presented();
        assert_eq!(icon_calls.get(), 1);
        assert_eq!(symbol_at(&buf, READINESS_X, 0), "I");
    }
}
