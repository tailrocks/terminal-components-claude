//! `CodeEditor` — a language-agnostic document editor.

use core::fmt;
use core::ops::Range;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn, cell_at, first_row};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::text::measure::{grapheme_width, graphemes};
use crate::text::{EditAction, EditOutcome, Extend, Motion, TextEditorCore, width};
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, SyntaxRole, Variant};
use crate::ui::{Cx, FrameRead, Ui};

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

/// A caller-supplied syntax highlighter.
///
/// Returned byte ranges may be in any order; the editor sorts and clamps them
/// once per edit generation before painting.
pub trait Highlighter {
    /// Highlight `text` with byte ranges and semantic syntax roles.
    fn highlight(&self, text: &str) -> Vec<(Range<usize>, SyntaxRole)>;
}

impl<F> Highlighter for F
where
    F: Fn(&str) -> Vec<(Range<usize>, SyntaxRole)>,
{
    fn highlight(&self, text: &str) -> Vec<(Range<usize>, SyntaxRole)> {
        self(text)
    }
}

/// A caller-supplied statement/block segmenter.
pub trait Segmenter {
    /// Return block byte ranges for `text`.
    fn segments(&self, text: &str) -> Vec<Range<usize>>;
}

impl<F> Segmenter for F
where
    F: Fn(&str) -> Vec<Range<usize>>,
{
    fn segments(&self, text: &str) -> Vec<Range<usize>> {
        self(text)
    }
}

/// Diagnostic severity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeSeverity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// Informational feedback.
    Info,
}

/// What Tab does while the editor is active.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TabBehavior {
    /// Insert or remove indentation.
    #[default]
    Indent,
    /// Ask the surrounding focus scope to move.
    Leave,
}

impl CodeSeverity {
    const fn role(self) -> Role {
        Role::Syntax(match self {
            CodeSeverity::Error => SyntaxRole::DiagError,
            CodeSeverity::Warning => SyntaxRole::DiagWarning,
            CodeSeverity::Info => SyntaxRole::DiagInfo,
        })
    }
}

/// A byte-range diagnostic attached to a document.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CodeDiagnostic {
    /// A byte range in the current document.
    pub range: Range<usize>,
    /// Severity.
    pub severity: CodeSeverity,
    /// Human-readable detail.
    pub message: String,
}

impl CodeDiagnostic {
    /// A diagnostic.
    pub fn new(range: Range<usize>, severity: CodeSeverity, message: impl Into<String>) -> Self {
        CodeDiagnostic {
            range,
            severity,
            message: message.into(),
        }
    }
}

fn nearest_diagnostic(diagnostics: &[CodeDiagnostic], cursor: usize) -> Option<&CodeDiagnostic> {
    let after = diagnostics.partition_point(|diagnostic| diagnostic.range.start < cursor);
    [after.checked_sub(1), Some(after)]
        .into_iter()
        .flatten()
        .filter_map(|index| diagnostics.get(index))
        .min_by_key(|diagnostic| diagnostic.range.start.abs_diff(cursor))
}

/// What a [`CodeEditor`] reports.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CodeAction {
    /// The document changed.
    Changed,
    /// Navigation moved the cursor or selection.
    CursorMoved,
    /// Editing ended; the document remains in the state.
    Committed,
    /// Tab asked the surrounding form to move focus.
    Leave {
        /// Move backwards when true.
        backward: bool,
    },
}

/// Const-constructible commands used by the editor's default keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeCmd {
    /// Begin editing at the cursor.
    InsertMode,
    /// Begin editing after the cursor.
    AppendMode,
    /// End editing.
    Commit,
    /// Move the cursor.
    Move(Motion, Extend),
    /// Move a page vertically.
    Page(bool),
    /// Delete backwards.
    Backspace,
    /// Delete forwards.
    Delete,
    /// Delete the previous word.
    DeleteWordLeft,
    /// Delete to line end.
    DeleteToLineEnd,
    /// Delete to line start.
    DeleteToLineStart,
    /// Insert a newline.
    Newline,
    /// Indent or leave the control.
    Tab(bool),
    /// Open find.
    Find,
    /// Accept the current find query.
    FindAccept,
    /// Remove one character from the find query.
    FindBackspace,
    /// Close find without moving.
    FindCancel,
    /// Move to the next or previous find result.
    FindNext(bool),
    /// Move to the previous or next block.
    Block(bool),
}

const fn b(chord: Chord, cmd: CodeCmd, label: &'static str, visible: bool) -> Binding<CodeCmd> {
    Binding {
        action: crate::ActionKey::custom(label),
        chord: Some(chord),
        cmd,
        label,
        priority: 50,
        visible,
    }
}

const NAV_BINDINGS: &[Binding<CodeCmd>] = &[
    b(
        Chord::key(KeyCode::Enter),
        CodeCmd::InsertMode,
        "Edit (i)",
        true,
    ),
    b(
        Chord::key(KeyCode::Char('i')),
        CodeCmd::InsertMode,
        "Edit",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('a')),
        CodeCmd::AppendMode,
        "Append",
        false,
    ),
    b(
        Chord::key(KeyCode::Up),
        CodeCmd::Move(Motion::Up, Extend::No),
        "Up (k)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('k')),
        CodeCmd::Move(Motion::Up, Extend::No),
        "Up",
        false,
    ),
    b(
        Chord::key(KeyCode::Down),
        CodeCmd::Move(Motion::Down, Extend::No),
        "Down (j)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('j')),
        CodeCmd::Move(Motion::Down, Extend::No),
        "Down",
        false,
    ),
    b(
        Chord::key(KeyCode::Left),
        CodeCmd::Move(Motion::Left, Extend::No),
        "Left (h)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('h')),
        CodeCmd::Move(Motion::Left, Extend::No),
        "Left",
        false,
    ),
    b(
        Chord::key(KeyCode::Right),
        CodeCmd::Move(Motion::Right, Extend::No),
        "Right (l)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('l')),
        CodeCmd::Move(Motion::Right, Extend::No),
        "Right",
        false,
    ),
    b(
        Chord::key(KeyCode::PageUp),
        CodeCmd::Page(true),
        "Page up",
        false,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        CodeCmd::Page(false),
        "Page down",
        false,
    ),
    b(
        Chord::key(KeyCode::Home),
        CodeCmd::Move(Motion::DocStart, Extend::No),
        "Start (g)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('g')),
        CodeCmd::Move(Motion::DocStart, Extend::No),
        "Start",
        false,
    ),
    b(
        Chord::key(KeyCode::End),
        CodeCmd::Move(Motion::DocEnd, Extend::No),
        "End (G)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('G')),
        CodeCmd::Move(Motion::DocEnd, Extend::No),
        "End",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('{')),
        CodeCmd::Block(true),
        "Previous block",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('}')),
        CodeCmd::Block(false),
        "Next block",
        false,
    ),
    b(Chord::key(KeyCode::Char('/')), CodeCmd::Find, "Find", true),
    b(
        Chord::key(KeyCode::Char('n')),
        CodeCmd::FindNext(false),
        "Next match",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('N')),
        CodeCmd::FindNext(true),
        "Previous match",
        false,
    ),
];

const EDIT_BINDINGS: &[Binding<CodeCmd>] = &[
    b(Chord::key(KeyCode::Esc), CodeCmd::Commit, "Done", true),
    b(
        Chord::key(KeyCode::Enter),
        CodeCmd::Newline,
        "New line",
        true,
    ),
    b(
        Chord::key(KeyCode::Tab),
        CodeCmd::Tab(false),
        "Indent",
        true,
    ),
    b(
        Chord::with(KeyCode::BackTab, SHIFT),
        CodeCmd::Tab(true),
        "Dedent",
        false,
    ),
    b(
        Chord::key(KeyCode::Left),
        CodeCmd::Move(Motion::Left, Extend::No),
        "Left",
        false,
    ),
    b(
        Chord::key(KeyCode::Right),
        CodeCmd::Move(Motion::Right, Extend::No),
        "Right",
        false,
    ),
    b(
        Chord::key(KeyCode::Up),
        CodeCmd::Move(Motion::Up, Extend::No),
        "Up",
        false,
    ),
    b(
        Chord::key(KeyCode::Down),
        CodeCmd::Move(Motion::Down, Extend::No),
        "Down",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, SHIFT),
        CodeCmd::Move(Motion::Left, Extend::Select),
        "Select left",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, SHIFT),
        CodeCmd::Move(Motion::Right, Extend::Select),
        "Select right",
        false,
    ),
    b(
        Chord::with(KeyCode::Up, SHIFT),
        CodeCmd::Move(Motion::Up, Extend::Select),
        "Select up",
        false,
    ),
    b(
        Chord::with(KeyCode::Down, SHIFT),
        CodeCmd::Move(Motion::Down, Extend::Select),
        "Select down",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, CTRL),
        CodeCmd::Move(Motion::WordLeft, Extend::No),
        "Word left (Alt)",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, CTRL),
        CodeCmd::Move(Motion::WordRight, Extend::No),
        "Word right (Alt)",
        false,
    ),
    b(
        Chord::with(KeyCode::Left, ALT),
        CodeCmd::Move(Motion::WordLeft, Extend::No),
        "Word left",
        false,
    ),
    b(
        Chord::with(KeyCode::Right, ALT),
        CodeCmd::Move(Motion::WordRight, Extend::No),
        "Word right",
        false,
    ),
    b(
        Chord::key(KeyCode::PageUp),
        CodeCmd::Page(true),
        "Page up",
        false,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        CodeCmd::Page(false),
        "Page down",
        false,
    ),
    b(
        Chord::key(KeyCode::Home),
        CodeCmd::Move(Motion::Home, Extend::No),
        "Line start",
        false,
    ),
    b(
        Chord::key(KeyCode::End),
        CodeCmd::Move(Motion::End, Extend::No),
        "Line end",
        false,
    ),
    b(
        Chord::key(KeyCode::Backspace),
        CodeCmd::Backspace,
        "Backspace",
        false,
    ),
    b(
        Chord::with(KeyCode::Backspace, CTRL),
        CodeCmd::DeleteWordLeft,
        "Delete word (Alt)",
        false,
    ),
    b(
        Chord::with(KeyCode::Backspace, ALT),
        CodeCmd::DeleteWordLeft,
        "Delete word",
        false,
    ),
    b(
        Chord::key(KeyCode::Delete),
        CodeCmd::Delete,
        "Delete",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('u'), CTRL),
        CodeCmd::DeleteToLineStart,
        "Delete to start",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('k'), CTRL),
        CodeCmd::DeleteToLineEnd,
        "Delete to end",
        false,
    ),
];

const FIND_BINDINGS: &[Binding<CodeCmd>] = &[
    b(
        Chord::key(KeyCode::Esc),
        CodeCmd::FindCancel,
        "Close find",
        true,
    ),
    b(
        Chord::key(KeyCode::Enter),
        CodeCmd::FindAccept,
        "Accept find",
        true,
    ),
    b(
        Chord::key(KeyCode::Backspace),
        CodeCmd::FindBackspace,
        "Erase find",
        false,
    ),
];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct FindState {
    needle: String,
    matches: Vec<Range<usize>>,
    current: usize,
    typing: bool,
}

/// Durable state of a [`CodeEditor`]. `Debug` redacts document and find text.
#[derive(Clone, PartialEq, Eq)]
pub struct CodeEditorState {
    editor: TextEditorCore,
    editing: bool,
    scroll: ScrollState,
    edit_generation: u64,
    find: Option<FindState>,
    diagnostics: Vec<CodeDiagnostic>,
    has_error: bool,
    running: Option<Range<usize>>,
}

impl Default for CodeEditorState {
    fn default() -> Self {
        Self::new("")
    }
}

impl fmt::Debug for CodeEditorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeEditorState")
            .field("text", &"[redacted]")
            .field("text_len", &self.editor.text().len())
            .field("editing", &self.editing)
            .field("scroll", &self.scroll)
            .field("edit_generation", &self.edit_generation)
            .field("find", &self.find.as_ref().map(|_| "[redacted]"))
            .field("diagnostics", &self.diagnostics)
            .field("has_error", &self.has_error)
            .field("running", &self.running)
            .finish()
    }
}

impl CodeEditorState {
    /// State containing `text`.
    pub fn new(text: &str) -> Self {
        let mut editor = TextEditorCore::multi(text);
        editor.set_cursor_line_col(0, 0);
        CodeEditorState {
            editor,
            editing: false,
            scroll: ScrollState::default(),
            edit_generation: 0,
            find: None,
            diagnostics: Vec::new(),
            has_error: false,
            running: None,
        }
    }

    /// The document.
    pub fn text(&self) -> &str {
        self.editor.text()
    }

    /// Replace the document, reset navigation, and invalidate highlighting.
    pub fn set_text(&mut self, text: &str) {
        if self.editor.apply(EditAction::SetText(text)).changed() {
            self.bump_generation();
        }
        self.editor.set_cursor_line_col(0, 0);
        self.scroll.jump_start();
        self.diagnostics.clear();
        self.has_error = false;
        self.refind();
    }

    /// Whether the editor is in insert mode.
    pub const fn is_editing(&self) -> bool {
        self.editing
    }

    /// The vertical scroll.
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    /// Cursor byte offset.
    pub const fn cursor_offset(&self) -> usize {
        self.editor.cursor_offset()
    }

    /// Selected bytes, if any.
    pub fn selected_text(&self) -> Option<&str> {
        self.editor.selected_text()
    }

    /// Replace and sort diagnostics.
    pub fn set_diagnostics(&mut self, mut diagnostics: Vec<CodeDiagnostic>) {
        diagnostics.sort_by_key(|d| d.range.start);
        self.has_error = diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == CodeSeverity::Error);
        self.diagnostics = diagnostics;
    }

    /// Current diagnostics, sorted by start offset.
    pub fn diagnostics(&self) -> &[CodeDiagnostic] {
        &self.diagnostics
    }

    /// Mark one block as running.
    pub fn set_running(&mut self, block: Option<Range<usize>>) {
        self.running = block;
    }

    /// Start insert mode.
    pub const fn begin_edit(&mut self) {
        self.editing = true;
    }

    /// End insert mode, preserving the document.
    pub fn commit(&mut self) {
        self.editing = false;
        let _ = self.editor.apply(EditAction::ClearSelection);
    }

    /// Jump to a byte offset.
    pub fn jump_to(&mut self, offset: usize) {
        let offset = floor_boundary(self.text(), offset.min(self.text().len()));
        let (line, col) = line_col(self.text(), offset);
        self.editor.set_cursor_line_col(line, col);
        self.scroll.ensure_visible(line);
    }

    /// Explicitly invalidate derived highlighting after caller-owned semantic
    /// inputs captured by a highlighter change.
    pub fn invalidate(&mut self) {
        self.bump_generation();
    }

    fn bump_generation(&mut self) {
        self.edit_generation = self.edit_generation.saturating_add(1);
    }

    fn after_edit(&mut self, outcome: EditOutcome) {
        if outcome.changed() {
            self.bump_generation();
            self.refind();
        }
        let cur = self.editor.cursor_pos();
        self.scroll.set_content(self.editor.line_count());
        self.scroll.ensure_visible(cur.line);
    }

    fn open_find(&mut self) {
        let mut find = self.find.take().unwrap_or_default();
        find.typing = true;
        self.find = Some(find);
        self.refind();
    }

    fn refind(&mut self) {
        let Some(find) = self.find.as_mut() else {
            return;
        };
        find.matches.clear();
        if find.needle.is_empty() {
            find.current = 0;
            return;
        }
        find.matches.extend(
            self.editor
                .text()
                .match_indices(find.needle.as_str())
                .map(|(at, text)| at..at.saturating_add(text.len())),
        );
        let cursor = self.editor.cursor_offset();
        find.current = find
            .matches
            .iter()
            .position(|range| range.start >= cursor)
            .unwrap_or(0);
    }

    fn goto_match(&mut self, previous: bool) {
        let Some(find) = self.find.as_mut() else {
            return;
        };
        if find.matches.is_empty() {
            return;
        }
        if previous {
            find.current = find
                .current
                .checked_sub(1)
                .unwrap_or_else(|| find.matches.len().saturating_sub(1));
        } else if find.current.saturating_add(1) >= find.matches.len() {
            find.current = 0;
        } else {
            find.current = find.current.saturating_add(1);
        }
        let target = find.matches.get(find.current).map(|range| range.start);
        if let Some(target) = target {
            self.jump_to(target);
        }
    }
}

#[derive(Debug, Default)]
struct HighlightCache {
    generation: u64,
    valid: bool,
    spans: Vec<(Range<usize>, SyntaxRole)>,
    line_starts: Vec<usize>,
}

impl HighlightCache {
    fn ensure(&mut self, generation: u64, text: &str, highlighter: Option<&dyn Highlighter>) {
        if generation != u64::MAX && self.valid && self.generation == generation {
            return;
        }
        self.spans.clear();
        self.line_starts.clear();
        self.line_starts.push(0);
        self.line_starts.extend(
            text.match_indices('\n')
                .map(|(offset, _)| offset.saturating_add(1)),
        );
        if let Some(highlighter) = highlighter {
            self.spans = highlighter.highlight(text);
            let len = text.len();
            for (range, _) in &mut self.spans {
                range.start = floor_boundary(text, range.start.min(len));
                range.end = floor_boundary(text, range.end.min(len)).max(range.start);
            }
            self.spans.sort_by_key(|(range, _)| range.start);
        }
        self.generation = generation;
        self.valid = true;
    }
}

/// A language-agnostic code editor over [`TextEditorCore`].
///
/// ## Construction
/// `CodeEditor::new(id, rows)` with [`CodeEditorState::new`] holding the
/// document.
///
/// ## Ownership
/// The caller owns `CodeEditorState` (document, cursor, selection, diagnostics,
/// find and scroll). The runtime owns focus, pointer routing, capture and the
/// derived highlight cache.
///
/// ## Configuration
/// `.highlighter`, `.segmenter`, `.placeholder`, `.indent`, `.tab_behavior`,
/// `.read_only`, `.disabled`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::CODE`, `Variant::DEFAULT` only.
///
/// ## States
/// Runtime focus/hover/press plus state-owned `EDITING`; `READ_ONLY`,
/// `DISABLED` and diagnostic `ERROR` are derived.
///
/// ## Actions
/// [`CodeAction::Changed`], `CursorMoved`, `Committed`, and `Leave`.
///
/// ## Focus
/// One focusable editor stop; it swallows typing. Read-only remains reachable,
/// disabled does not.
///
/// ## Keyboard
/// Navigation defaults to vim-compatible `i/a`, `h/j/k/l`, `g/G`, `{`/`}`,
/// `/`, `n/N`; insert mode uses the shared editor motions and mutations. The
/// table is ordinary [`Bindings`] metadata and can be remapped by `KeyMap`.
/// `/` enters find input: bare characters extend the query, `Backspace`
/// removes one character, `Enter` accepts it, and `Esc` closes it.
///
/// ## Mouse
/// Pressing `TEXT` places the cursor and enters insert mode; wheel and
/// `TRACK`/`THUMB` are delegated to [`ScrollRegion`].
///
/// ## Layout
/// `rows` includes a one-row find/status footer. The body has focus, marker,
/// line-number and scrollbar columns. `measure` returns a 20-column minimum;
/// zero area registers nothing.
///
/// ## Parts
/// `CONTAINER`, `GUTTER`, `MARKER`, `META` (numbers/footer), `TEXT`,
/// `PLACEHOLDER`, `ROW` (selection), `QUERY`, `DETAIL` (diagnostic), `ICON`,
/// `TRACK`, `THUMB`.
///
/// ## Overrides
/// `.patch`/`.patch_part` reach all parts. `.slot` replaces `GUTTER`, `MARKER`,
/// `ICON`, `PLACEHOLDER`, `QUERY`, `TRACK`, or `THUMB`; text geometry remains
/// owned by the editor.
///
/// ## Identity
/// One caller `Id`; diagnostics and byte ranges refer to the state's current
/// document.
///
/// ## Testing
/// `CodeEditorCase`; `render::components::code_editor::*`; unit coverage pins
/// the edit-generation cache and sorted-span walk.
///
/// ## Invariants
/// `draw(&self, ..., &CodeEditorState)` cannot commit or mutate semantic state.
/// Highlighting runs once per edit generation and painting advances sorted
/// span/diagnostic/find cursors alongside the grapheme walk.
pub struct CodeEditor<'a> {
    id: Id,
    rows: u16,
    highlighter: Option<&'a dyn Highlighter>,
    segmenter: Option<&'a dyn Segmenter>,
    placeholder: Option<&'a str>,
    indent: u8,
    tab_behavior: TabBehavior,
    read_only: bool,
    disabled: bool,
    ov: Overrides<'a>,
}

impl fmt::Debug for CodeEditor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeEditor")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .field("highlighter", &self.highlighter.is_some())
            .field("segmenter", &self.segmenter.is_some())
            .field("placeholder", &self.placeholder)
            .field("indent", &self.indent)
            .field("tab_behavior", &self.tab_behavior)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<'a> CodeEditor<'a> {
    /// Styled parts.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::META,
        Part::TEXT,
        Part::PLACEHOLDER,
        Part::ROW,
        Part::QUERY,
        Part::DETAIL,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
    ];

    /// An editor `rows` cells tall.
    pub const fn new(id: Id, rows: u16) -> Self {
        CodeEditor {
            id,
            rows,
            highlighter: None,
            segmenter: None,
            placeholder: None,
            indent: 2,
            tab_behavior: TabBehavior::Indent,
            read_only: false,
            disabled: false,
            ov: Overrides::new(),
        }
    }

    /// Syntax highlighter.
    #[must_use]
    pub const fn highlighter(mut self, highlighter: &'a dyn Highlighter) -> Self {
        self.highlighter = Some(highlighter);
        self
    }

    /// Block segmenter.
    #[must_use]
    pub const fn segmenter(mut self, segmenter: &'a dyn Segmenter) -> Self {
        self.segmenter = Some(segmenter);
        self
    }

    /// Empty-document placeholder.
    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Spaces inserted by Tab.
    #[must_use]
    pub const fn indent(mut self, spaces: u8) -> Self {
        self.indent = spaces;
        self
    }

    /// Configure whether Tab indents or leaves the editor.
    #[must_use]
    pub const fn tab_behavior(mut self, behavior: TabBehavior) -> Self {
        self.tab_behavior = behavior;
        self
    }

    /// Read-only: reachable but never edits.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered but skipped.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// Patch every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(patch);
        self
    }

    /// Patch selected parts.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(patches);
        self
    }

    /// Replace a part painter.
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    const fn bar(&self) -> ScrollRegion<'a> {
        ScrollRegion::new(self.id)
    }

    fn active_bindings(state: &CodeEditorState) -> &'static [Binding<CodeCmd>] {
        if state.find.as_ref().is_some_and(|find| find.typing) {
            FIND_BINDINGS
        } else if state.editing {
            EDIT_BINDINGS
        } else {
            NAV_BINDINGS
        }
    }

    fn gutter_width(lines: usize) -> u16 {
        let mut digits = 1u16;
        let mut n = lines.max(1);
        while n >= 10 {
            digits = digits.saturating_add(1);
            n /= 10;
        }
        digits.max(2).saturating_add(4)
    }

    /// Selection, or the block containing the cursor.
    pub fn selection_or_block(&self, st: &CodeEditorState) -> Option<(String, Range<usize>)> {
        if let Some(range) = st.editor.selection() {
            return Some((st.text().get(range.clone())?.to_owned(), range));
        }
        let range = self.current_block(st)?;
        Some((st.text().get(range.clone())?.to_owned(), range))
    }

    /// Blocks returned by the configured segmenter.
    pub fn blocks(&self, st: &CodeEditorState) -> Vec<Range<usize>> {
        let mut blocks = self
            .segmenter
            .map_or_else(Vec::new, |segmenter| segmenter.segments(st.text()));
        blocks.sort_by_key(|range| range.start);
        blocks
    }

    /// Block at or immediately before the cursor.
    pub fn current_block(&self, st: &CodeEditorState) -> Option<Range<usize>> {
        let cursor = st.cursor_offset();
        let blocks = self.blocks(st);
        blocks
            .iter()
            .find(|range| range.start <= cursor && cursor <= range.end)
            .or_else(|| blocks.iter().rev().find(|range| range.end <= cursor))
            .cloned()
    }

    /// Last-frame cursor cell, suitable for anchoring completion.
    pub fn cursor_cell(&self, cx: &Cx<'_>, st: &CodeEditorState) -> Option<Rect> {
        let area = cx.area(self.id)?;
        let cursor = st.editor.cursor_pos();
        if cursor.line < st.scroll.offset()
            || cursor.line >= st.scroll.offset().saturating_add(usize::from(area.height))
        {
            return None;
        }
        let lines = st.editor.line_count();
        let content_width = area
            .width
            .saturating_sub(u16::from(lines > usize::from(area.height)));
        let gutter = Self::gutter_width(lines);
        let text_area = Rect {
            x: area.x.saturating_add(gutter),
            width: content_width.saturating_sub(gutter),
            ..area
        };
        let y = cursor.line.saturating_sub(st.scroll.offset());
        let position = visible_cursor(text_area, cursor.col, y, usize::from(st.editor.hscroll()))?;
        Some(Rect {
            x: position.x,
            y: position.y,
            width: 1,
            height: 1,
        })
    }

    /// Update interaction and editing state.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut CodeEditorState) -> Response<CodeAction> {
        if self.disabled {
            return Response::ignored();
        }
        let mut acc = Acc::<CodeAction>::new();
        let lines = st.editor.line_count();
        let bar = self.bar();
        let track = bar.prepare(cx, &mut st.scroll, lines);
        let page = st.scroll.viewport_len().max(1);
        for intent in cx.intents(self.id) {
            acc.fold(&bar.handle_intent(cx, &mut st.scroll, track, intent));
            match intent {
                Intent::FocusOut { .. } | Intent::Cancel if st.editing => {
                    st.commit();
                    acc.action(CodeAction::Committed);
                }
                Intent::Binding(action) => {
                    let table = Self::active_bindings(st);
                    if let Some(command) = Binding::command(table, action) {
                        if st.find.as_ref().is_some_and(|find| find.typing) {
                            Self::find_command(st, command, &mut acc);
                        } else {
                            self.command(st, command, page, &mut acc);
                        }
                    }
                }
                Intent::Key(key)
                    if st.editing || st.find.as_ref().is_some_and(|find| find.typing) =>
                {
                    if let Some(c) = key.bare_char() {
                        if let Some(find) = st.find.as_mut().filter(|find| find.typing) {
                            find.needle.push(c);
                            st.refind();
                            acc.changed();
                        } else if !self.read_only {
                            let outcome = st.editor.apply(EditAction::Insert(c));
                            st.after_edit(outcome);
                            if outcome.changed() {
                                acc.action(CodeAction::Changed);
                            } else {
                                acc.consumed();
                            }
                        } else {
                            acc.consumed();
                        }
                    }
                }
                Intent::Paste(text) if st.editing && !self.read_only => {
                    let out = st.editor.apply(EditAction::Paste(text));
                    st.after_edit(out);
                    if out.changed() {
                        acc.action(CodeAction::Changed);
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer { part, .. }
                    if part.part == Part::TRACK || part.part == Part::THUMB => {}
                Intent::Pointer {
                    phase: Phase::Press | Phase::Click,
                    part:
                        PartRef {
                            part: Part::TEXT, ..
                        },
                    local,
                    ..
                } if !self.read_only && !self.disabled => {
                    st.editing = true;
                    let line = st
                        .scroll
                        .offset()
                        .saturating_add(usize::from(local.y))
                        .min(lines.saturating_sub(1));
                    let col = usize::from(local.x).saturating_add(usize::from(st.editor.hscroll()));
                    st.editor.set_cursor_line_col(line, col);
                    acc.action(CodeAction::CursorMoved);
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        if st.editing {
            let cursor = st.editor.cursor_pos();
            st.scroll.set_content(st.editor.line_count());
            st.scroll.ensure_visible(cursor.line);
            if let Some(area) = cx.area(self.id) {
                let text_width = area
                    .width
                    .saturating_sub(Self::gutter_width(lines).saturating_add(1));
                st.editor.scroll_into_view(text_width);
            }
        }
        acc.finish(self.id)
    }

    fn command(
        &self,
        st: &mut CodeEditorState,
        command: CodeCmd,
        page: usize,
        acc: &mut Acc<CodeAction>,
    ) {
        if self.read_only
            && matches!(
                command,
                CodeCmd::InsertMode
                    | CodeCmd::AppendMode
                    | CodeCmd::Backspace
                    | CodeCmd::Delete
                    | CodeCmd::DeleteWordLeft
                    | CodeCmd::DeleteToLineEnd
                    | CodeCmd::DeleteToLineStart
                    | CodeCmd::Newline
                    | CodeCmd::Tab(false)
            )
        {
            acc.consumed();
            return;
        }
        match command {
            CodeCmd::InsertMode => {
                st.editing = true;
                acc.changed();
            }
            CodeCmd::AppendMode => {
                st.editing = true;
                let out = st.editor.apply(EditAction::Move(Motion::Right, Extend::No));
                st.after_edit(out);
                acc.changed();
            }
            CodeCmd::Commit => {
                st.commit();
                acc.action(CodeAction::Committed);
            }
            CodeCmd::Find => {
                st.open_find();
                acc.changed();
            }
            CodeCmd::FindNext(previous) => {
                st.goto_match(previous);
                acc.action(CodeAction::CursorMoved);
            }
            CodeCmd::Block(previous) => {
                let cursor = st.cursor_offset();
                let blocks = self.blocks(st);
                let target = if previous {
                    blocks.iter().rev().find(|range| range.start < cursor)
                } else {
                    blocks.iter().find(|range| range.start > cursor)
                };
                if let Some(range) = target {
                    st.jump_to(range.start);
                    acc.action(CodeAction::CursorMoved);
                } else {
                    acc.consumed();
                }
            }
            CodeCmd::Page(up) => {
                let motion = if up { Motion::Up } else { Motion::Down };
                let mut out = EditOutcome::Ignored;
                for _ in 0..page {
                    let step = st.editor.apply(EditAction::Move(motion, Extend::No));
                    if step.is_visible() {
                        out = step;
                    }
                }
                st.after_edit(out);
                if out.is_visible() {
                    acc.action(CodeAction::CursorMoved);
                } else {
                    acc.consumed();
                }
            }
            CodeCmd::Tab(backward) if self.tab_behavior == TabBehavior::Leave => {
                st.commit();
                acc.action(CodeAction::Leave { backward });
            }
            CodeCmd::Tab(true)
            | CodeCmd::FindAccept
            | CodeCmd::FindBackspace
            | CodeCmd::FindCancel => acc.consumed(),
            CodeCmd::Tab(false) => {
                let spaces = "                ";
                let take = usize::from(self.indent).min(spaces.len());
                let out = st.editor.apply(EditAction::Paste(&spaces[..take]));
                st.after_edit(out);
                acc.action(CodeAction::Changed);
            }
            other => {
                Self::apply_edit_command(st, other, acc);
            }
        }
    }

    fn apply_edit_command(st: &mut CodeEditorState, command: CodeCmd, acc: &mut Acc<CodeAction>) {
        let action = match command {
            CodeCmd::Move(motion, extend) => EditAction::Move(motion, extend),
            CodeCmd::Backspace => EditAction::Backspace,
            CodeCmd::Delete => EditAction::Delete,
            CodeCmd::DeleteWordLeft => EditAction::DeleteWordLeft,
            CodeCmd::DeleteToLineEnd => EditAction::DeleteToLineEnd,
            CodeCmd::DeleteToLineStart => EditAction::DeleteToLineStart,
            CodeCmd::Newline => EditAction::Newline,
            _ => EditAction::ClearSelection,
        };
        let outcome = st.editor.apply(action);
        st.after_edit(outcome);
        match outcome {
            EditOutcome::Changed => acc.action(CodeAction::Changed),
            EditOutcome::Moved => acc.action(CodeAction::CursorMoved),
            EditOutcome::Ignored | EditOutcome::Rejected => acc.consumed(),
        }
    }

    fn find_command(st: &mut CodeEditorState, command: CodeCmd, acc: &mut Acc<CodeAction>) {
        match command {
            CodeCmd::FindCancel => {
                st.find = None;
                acc.changed();
            }
            CodeCmd::FindAccept => {
                if let Some(find) = st.find.as_mut() {
                    find.typing = false;
                }
                st.goto_match(false);
                acc.action(CodeAction::CursorMoved);
            }
            CodeCmd::FindBackspace => {
                if let Some(find) = st.find.as_mut() {
                    find.needle.pop();
                }
                st.refind();
                acc.changed();
            }
            _ => acc.consumed(),
        }
    }

    /// Draw the editor without semantic state mutation.
    #[expect(
        clippy::too_many_lines,
        reason = "single sorted cursor walk over visible code"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &CodeEditorState) -> Rect {
        let used = Rect {
            height: self.rows.min(area.height),
            ..area
        };
        if used.is_empty() {
            return used;
        }
        let mut cache = core::mem::take(ui.cache::<HighlightCache>(self.id));
        cache.ensure(st.edit_generation, st.text(), self.highlighter);
        let body_h = used.height.saturating_sub(1);
        let body = Rect {
            height: body_h,
            ..used
        };
        let lines = cache.line_starts.len();
        let gutter_w = Self::gutter_width(lines);
        let derived = (if st.editing {
            StateFlags::EDITING
        } else {
            StateFlags::empty()
        }) | (if self.read_only {
            StateFlags::READ_ONLY
        } else {
            StateFlags::empty()
        }) | (if self.disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::empty()
        }) | (if st.has_error {
            StateFlags::ERROR
        } else {
            StateFlags::empty()
        });
        let runtime = ui
            .state(self.id)
            .difference(StateFlags::EDITING | StateFlags::SELECTED);
        let live = Overrides::flags(runtime, derived);
        let style = |ui: &mut Ui<'_>, part: Part, flags: StateFlags| {
            self.ov
                .style(ui, self.id, Family::CODE, Variant::DEFAULT, part, flags)
        };
        let container = style(ui, Part::CONTAINER, live);
        ui.fill(used, container.style);
        let focusability = if self.disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        ui.register_editor(
            self.id,
            body,
            focusability,
            if st.editing {
                StateFlags::EDITING
            } else {
                StateFlags::empty()
            },
        );
        ui.publish_bindings(self.id, live, Self::active_bindings(st));
        let content = self.bar().draw(ui, body, &st.scroll, lines);
        let text_area = Rect {
            x: content.x.saturating_add(gutter_w),
            width: content.width.saturating_sub(gutter_w),
            ..content
        };
        ui.register_part(self.id, PartRef::of(Part::TEXT), text_area);
        let spans = cache.spans.as_slice();
        let selection = st.editor.selection();
        let cursor = st.editor.cursor_pos();
        let find = st.find.as_ref();
        let view = ScrollRegion::view(&st.scroll, content, lines);
        let text_style = style(ui, Part::TEXT, live).style;
        let selected_style = style(ui, Part::ROW, live | StateFlags::SELECTED).style;
        let number_style = style(ui, Part::META, live).style;
        let marker_style = style(ui, Part::MARKER, live).style;
        let gutter_style = style(ui, Part::GUTTER, live).style;
        let visible = view.visible_range();
        let first_offset = cache
            .line_starts
            .get(visible.start)
            .copied()
            .unwrap_or(st.text().len());
        let mut span_cursor = spans.partition_point(|(range, _)| range.end <= first_offset);
        let mut diagnostic_cursor = st
            .diagnostics
            .partition_point(|diagnostic| diagnostic.range.end <= first_offset);
        let mut find_cursor = find.map_or(0, |find| {
            find.matches
                .partition_point(|range| range.end <= first_offset)
        });
        for line_index in visible {
            let Some(&global) = cache.line_starts.get(line_index) else {
                break;
            };
            let next = cache.line_starts.get(line_index.saturating_add(1)).copied();
            let line_end = next.map_or(st.text().len(), |start| start.saturating_sub(1));
            let code_line = st.text().get(global..line_end).unwrap_or("");
            {
                let y = content.y.saturating_add(
                    line_index
                        .saturating_sub(view.offset())
                        .min(usize::from(u16::MAX)) as u16,
                );
                let row = Rect {
                    y,
                    height: 1,
                    ..content
                };
                if live.contains(StateFlags::FOCUSED) && line_index == cursor.line {
                    match self.ov.slot_for(Part::GUTTER) {
                        Some(slot) => slot(ui, cell_at(row, row.x)),
                        None => match self
                            .ov
                            .style(
                                ui,
                                self.id,
                                Family::CODE,
                                Variant::DEFAULT,
                                Part::GUTTER,
                                live,
                            )
                            .glyph
                        {
                            Slot::Set(glyph) => {
                                ui.glyph(cell_at(row, row.x), glyph, gutter_style);
                            }
                            Slot::Inherit => {
                                ui.glyph(cell_at(row, row.x), GlyphRole::FocusBar, gutter_style);
                            }
                            Slot::Clear => {}
                        },
                    }
                }
                let marker = cell_at(row, row.x.saturating_add(1));
                while st
                    .diagnostics
                    .get(diagnostic_cursor)
                    .is_some_and(|diagnostic| diagnostic.range.end <= global)
                {
                    diagnostic_cursor = diagnostic_cursor.saturating_add(1);
                }
                let diag_here = st.diagnostics.get(diagnostic_cursor).filter(|diagnostic| {
                    diagnostic.range.start <= line_end && diagnostic.range.end > global
                });
                if diag_here.is_some() {
                    if let Some(slot) = self.ov.slot_for(Part::MARKER) {
                        slot(ui, marker);
                    } else {
                        ui.paint_str(marker, "!", marker_style);
                    }
                } else if st
                    .running
                    .as_ref()
                    .is_some_and(|r| r.start >= global && r.start <= line_end)
                {
                    if let Some(slot) = self.ov.slot_for(Part::ICON) {
                        slot(ui, marker);
                    } else {
                        let icon = style(ui, Part::ICON, live | StateFlags::BUSY);
                        let glyph = ui
                            .design()
                            .motion
                            .spinner_frames
                            .first()
                            .copied()
                            .unwrap_or("");
                        ui.paint_str(marker, glyph, icon.style);
                    }
                }
                let number = Rect {
                    x: row.x.saturating_add(3),
                    width: gutter_w.saturating_sub(4),
                    ..row
                }
                .intersection(row);
                let mut digits = [0u8; 20];
                let number_text = usize_text(line_index.saturating_add(1), &mut digits);
                let number_width = width(number_text).min(number.width);
                ui.paint_str(
                    Rect {
                        x: number.right().saturating_sub(number_width),
                        width: number_width,
                        ..number
                    },
                    number_text,
                    number_style,
                );
                let hs = usize::from(st.editor.hscroll());
                let mut column = 0usize;
                let mut x = text_area.x;
                for (at, grapheme) in graphemes(code_line) {
                    let cells = usize::from(grapheme_width(grapheme));
                    if column.saturating_add(cells) <= hs {
                        column = column.saturating_add(cells);
                        continue;
                    }
                    if x.saturating_add(cells.min(usize::from(u16::MAX)) as u16) > text_area.right()
                    {
                        ui.glyph(
                            cell_at(row, text_area.right().saturating_sub(1)),
                            GlyphRole::Ellipsis,
                            text_style,
                        );
                        break;
                    }
                    let offset = global.saturating_add(at);
                    while spans.get(span_cursor).is_some_and(|(r, _)| r.end <= offset) {
                        span_cursor = span_cursor.saturating_add(1);
                    }
                    let syntax = spans
                        .get(span_cursor)
                        .filter(|(range, _)| range.start <= offset && offset < range.end)
                        .map(|(_, role)| *role);
                    let mut painted = text_style;
                    if let Some(role) = syntax
                        && !live.contains(StateFlags::DISABLED)
                    {
                        painted.fg = Some(syntax_color(ui, role));
                    }
                    if selection
                        .as_ref()
                        .is_some_and(|range| range.start <= offset && offset < range.end)
                    {
                        painted = painted.patch(selected_style);
                    }
                    while st
                        .diagnostics
                        .get(diagnostic_cursor)
                        .is_some_and(|diagnostic| diagnostic.range.end <= offset)
                    {
                        diagnostic_cursor = diagnostic_cursor.saturating_add(1);
                    }
                    if let Some(diagnostic) =
                        st.diagnostics.get(diagnostic_cursor).filter(|diagnostic| {
                            diagnostic.range.start <= offset && offset < diagnostic.range.end
                        })
                    {
                        painted.add_modifier = painted.add_modifier.union(Modifier::UNDERLINED);
                        painted.underline_color = Some(role_color(ui, diagnostic.severity.role()));
                    }
                    if let Some(find) = find {
                        while find
                            .matches
                            .get(find_cursor)
                            .is_some_and(|range| range.end <= offset)
                        {
                            find_cursor = find_cursor.saturating_add(1);
                        }
                    }
                    if let Some(find) = find
                        && find
                            .matches
                            .get(find_cursor)
                            .is_some_and(|range| range.start <= offset && offset < range.end)
                    {
                        let role = if find_cursor == find.current {
                            SyntaxRole::MatchCurrentBg
                        } else {
                            SyntaxRole::MatchBg
                        };
                        painted.bg = Some(syntax_color(ui, role));
                    }
                    ui.paint_str(
                        Rect {
                            x,
                            y,
                            width: cells.min(usize::from(u16::MAX)) as u16,
                            height: 1,
                        },
                        grapheme,
                        painted,
                    );
                    x = x.saturating_add(cells.min(usize::from(u16::MAX)) as u16);
                    column = column.saturating_add(cells);
                }
            }
        }
        *ui.cache::<HighlightCache>(self.id) = cache;
        if st.text().is_empty()
            && !st.editing
            && let Some(placeholder) = self.placeholder
        {
            let row = first_row(text_area);
            if let Some(slot) = self.ov.slot_for(Part::PLACEHOLDER) {
                slot(ui, row);
            } else {
                let placeholder_style = style(ui, Part::PLACEHOLDER, live).style;
                ui.paint_str(row, placeholder, placeholder_style);
            }
        }
        let footer = Rect {
            y: used.bottom().saturating_sub(1),
            height: 1,
            ..used
        };
        if let Some(find) = &st.find {
            let query = style(ui, Part::QUERY, live);
            ui.paint_str(footer, "find ", query.style);
            let at = footer.x.saturating_add(5);
            ui.paint_str(
                Rect {
                    x: at,
                    width: footer.right().saturating_sub(at),
                    ..footer
                },
                &find.needle,
                query.style,
            );
            if find.typing && live.contains(StateFlags::FOCUSED) {
                let x = at.saturating_add(width(&find.needle));
                if x < footer.right() {
                    ui.set_cursor(self.id, Position::new(x, footer.y));
                }
            }
        } else if let Some(diagnostic) = nearest_diagnostic(&st.diagnostics, st.cursor_offset()) {
            let detail_style = style(ui, Part::DETAIL, live).style;
            ui.paint_str(footer, &diagnostic.message, detail_style);
        }
        if live.contains(StateFlags::FOCUSED)
            && !self.read_only
            && !st.find.as_ref().is_some_and(|find| find.typing)
            && cursor.line >= view.offset()
            && let Some(position) = visible_cursor(
                text_area,
                cursor.col,
                cursor.line.saturating_sub(view.offset()),
                usize::from(st.editor.hscroll()),
            )
        {
            ui.set_cursor(self.id, position);
        }
        used
    }

    /// Natural editor size.
    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (20, 2),
            preferred: (80, self.rows.max(2)),
        }
        .fit(constraints)
    }
}

impl Bindings for CodeEditor<'_> {
    type Cmd = CodeCmd;

    fn bindings(&self, state: BindingState) -> &'static [Binding<CodeCmd>] {
        if state.flags.contains(StateFlags::EDITING) {
            EDIT_BINDINGS
        } else {
            NAV_BINDINGS
        }
    }
}

fn visible_cursor(
    text_area: Rect,
    column: usize,
    row: usize,
    horizontal_offset: usize,
) -> Option<Position> {
    if text_area.is_empty() || column < horizontal_offset || row >= usize::from(text_area.height) {
        return None;
    }
    let x = column.saturating_sub(horizontal_offset);
    if x >= usize::from(text_area.width) {
        return None;
    }
    Some(Position::new(
        text_area
            .x
            .saturating_add(x.min(usize::from(u16::MAX)) as u16),
        text_area
            .y
            .saturating_add(row.min(usize::from(u16::MAX)) as u16),
    ))
}

fn floor_boundary(text: &str, mut offset: usize) -> usize {
    offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset = offset.saturating_sub(1);
    }
    offset
}

fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let prefix = text.get(..offset).unwrap_or("");
    let line = prefix.bytes().filter(|b| *b == b'\n').count();
    let tail = prefix.rsplit_once('\n').map_or(prefix, |(_, tail)| tail);
    (line, usize::from(width(tail)))
}

fn role_color(ui: &Ui<'_>, role: Role) -> ratatui_core::style::Color {
    match role {
        Role::Syntax(role) => syntax_color(ui, role),
        _ => ui.theme().color.fg[0],
    }
}

fn syntax_color(ui: &Ui<'_>, role: SyntaxRole) -> ratatui_core::style::Color {
    let syntax = ui.theme().color.syntax;
    match role {
        SyntaxRole::Keyword => syntax.keyword,
        SyntaxRole::Ident => syntax.ident,
        SyntaxRole::Str => syntax.string,
        SyntaxRole::Number => syntax.number,
        SyntaxRole::Operator => syntax.operator,
        SyntaxRole::Punct => syntax.punct,
        SyntaxRole::Comment => syntax.comment,
        SyntaxRole::Plain => syntax.plain,
        SyntaxRole::TypeName => syntax.type_name,
        SyntaxRole::Function => syntax.function,
        SyntaxRole::Constant => syntax.constant,
        SyntaxRole::Invalid => syntax.invalid,
        SyntaxRole::Deprecated => syntax.deprecated,
        SyntaxRole::MatchBg => syntax.match_bg,
        SyntaxRole::MatchCurrentBg => syntax.match_current_bg,
        SyntaxRole::BracketMatch => syntax.bracket_match,
        SyntaxRole::DiagError => syntax.diagnostic_error,
        SyntaxRole::DiagWarning => syntax.diagnostic_warning,
        SyntaxRole::DiagInfo => syntax.diagnostic_info,
    }
}

fn usize_text(mut number: usize, buf: &mut [u8; 20]) -> &str {
    let mut at = buf.len();
    loop {
        at = at.saturating_sub(1);
        if let Some(slot) = buf.get_mut(at) {
            *slot = b'0'.saturating_add((number % 10) as u8);
        }
        number /= 10;
        if number == 0 {
            break;
        }
    }
    core::str::from_utf8(buf.get(at..).unwrap_or_default()).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{Stub, key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::builder::contrast;
    use crate::theme::{ColorLevel, Theme};

    const ID: Id = Id::root("code.tests");

    struct EditorApp {
        state: CodeEditorState,
        read_only: bool,
    }

    impl App for EditorApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            CodeEditor::new(ID, 5)
                .read_only(self.read_only)
                .update(cx, &mut self.state)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            CodeEditor::new(ID, 5)
                .read_only(self.read_only)
                .draw(ui, ui.full(), &self.state);
        }
    }

    fn focused_runtime(text: &str, read_only: bool) -> (Runtime<EditorApp>, Buffer) {
        let area = Rect::new(0, 0, 40, 5);
        let mut runtime = Runtime::new(
            EditorApp {
                state: CodeEditorState::new(text),
                read_only,
            },
            Theme::junie(),
        );
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Tab));
        runtime.draw_buffer(area, &mut buffer);
        (runtime, buffer)
    }

    fn send_key(runtime: &mut Runtime<EditorApp>, buffer: &mut Buffer, code: KeyCode) {
        let _ = runtime.handle(key(code));
        runtime.draw_buffer(buffer.area, buffer);
    }

    #[test]
    fn edit_counter_invalidates_the_highlight_cache() {
        let calls = Cell::new(0usize);
        let highlighter = |text: &str| {
            calls.set(calls.get().saturating_add(1));
            vec![(0..text.len(), SyntaxRole::Keyword)]
        };
        let editor = CodeEditor::new(ID, 4).highlighter(&highlighter);
        let mut state = CodeEditorState::new("select 1");
        let area = Rect::new(0, 0, 40, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        for _ in 0..2 {
            runtime.draw_scene(area, &mut buffer, |ui, rect| {
                editor.draw(ui, rect, &state);
            });
        }
        assert_eq!(calls.get(), 1, "unchanged draw re-highlighted the document");
        state.set_text("select 2");
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            editor.draw(ui, rect, &state);
        });
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn editing_style_comes_only_from_the_real_edit_state() {
        let render = |stale_runtime_editing: bool, real_editing: bool| {
            let theme = Theme::junie().override_family(Family::CODE, |recipe| {
                recipe
                    .part(Part::TEXT)
                    .when(StateFlags::EDITING, StylePatch::new().set_bg(Role::Danger));
            });
            let area = Rect::new(0, 0, 30, 4);
            let mut runtime = Runtime::new(Stub::default(), theme);
            let mut buffer = Buffer::empty(area);
            runtime.draw_scene(area, &mut buffer, |ui, _| {
                if stale_runtime_editing {
                    ui.declare_state(ID, StateFlags::EDITING);
                }
            });
            let mut state = CodeEditorState::new("hello");
            if real_editing {
                state.begin_edit();
            }
            runtime.draw_scene(area, &mut buffer, |ui, area| {
                CodeEditor::new(ID, 4).draw(ui, area, &state);
            });
            let text = runtime
                .registry()
                .area_of_part(ID, PartRef::of(Part::TEXT))
                .unwrap_or(Rect::ZERO);
            buffer
                .cell(Position::new(text.x, text.y))
                .map(|cell| cell.bg)
                .unwrap_or_default()
        };
        let idle = render(false, false);
        assert_eq!(render(true, false), idle, "stale runtime EDITING leaked");
        assert_ne!(render(false, true), idle, "real edit state was not styled");
    }

    #[test]
    fn saturation_never_reuses_stale_highlights() {
        let calls = Cell::new(0usize);
        let highlighter = |_text: &str| {
            calls.set(calls.get().saturating_add(1));
            Vec::new()
        };
        let editor = CodeEditor::new(ID, 3).highlighter(&highlighter);
        let mut state = CodeEditorState::new("a");
        state.edit_generation = u64::MAX;
        let area = Rect::new(0, 0, 20, 3);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        for _ in 0..2 {
            runtime.draw_scene(area, &mut buffer, |ui, rect| {
                editor.draw(ui, rect, &state);
            });
        }
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn captured_highlighter_and_segmenter_are_supported() {
        let keyword = "let".to_owned();
        let highlighter = |text: &str| {
            text.match_indices(&keyword)
                .map(|(at, found)| at..at + found.len())
                .map(|range| (range, SyntaxRole::Keyword))
                .collect()
        };
        let delimiter = ';';
        let segmenter = |text: &str| {
            text.match_indices(delimiter)
                .scan(0usize, |start, (end, _)| {
                    let range = *start..end.saturating_add(1);
                    *start = range.end;
                    Some(range)
                })
                .collect()
        };
        let editor = CodeEditor::new(ID, 4)
            .highlighter(&highlighter)
            .segmenter(&segmenter);
        let state = CodeEditorState::new("let a;let b;");
        assert_eq!(editor.blocks(&state), [0..6, 6..12]);
    }

    #[test]
    fn disabled_highlighted_text_keeps_contrast_and_dim_at_every_level() {
        let area = Rect::new(0, 0, 40, 5);
        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        let highlighter = |text: &str| vec![(0..text.len(), SyntaxRole::Keyword)];
        for base in [Theme::junie(), Theme::paper()] {
            for level in levels {
                let mut runtime = Runtime::new(Stub::default(), base.downgrade(level));
                let mut buffer = Buffer::empty(area);
                let state = CodeEditorState::new("let attempts = 5;");
                runtime.draw_scene(area, &mut buffer, |ui, rect| {
                    CodeEditor::new(ID, 5)
                        .disabled(true)
                        .highlighter(&highlighter)
                        .draw(ui, rect, &state);
                });
                let text = runtime
                    .area_of_part(ID, PartRef::of(Part::TEXT))
                    .expect("disabled editor registers its text part");
                for position in text.positions() {
                    let cell = buffer
                        .cell(position)
                        .expect("text part positions are inside the buffer");
                    assert!(
                        contrast(cell.fg, cell.bg) >= 3.0,
                        "{base:?}/{level:?}: disabled code at {position:?} has {:?} on {:?}",
                        cell.fg,
                        cell.bg
                    );
                    assert!(
                        cell.modifier.contains(Modifier::DIM),
                        "{base:?}/{level:?}: disabled code at {position:?} lost DIM"
                    );
                }
            }
        }
    }

    #[test]
    fn draw_cannot_commit_or_change_the_state() {
        let editor = CodeEditor::new(ID, 4);
        let mut state = CodeEditorState::new("one\ntwo");
        state.begin_edit();
        let before = state.clone();
        let area = Rect::new(0, 0, 30, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            editor.draw(ui, rect, &state);
        });
        assert_eq!(state, before);
    }

    #[test]
    fn find_input_routes_text_backspace_enter_and_escape() {
        let (mut runtime, mut buffer) = focused_runtime("alpha beta alpha", false);
        for code in [
            KeyCode::Char('/'),
            KeyCode::Char('a'),
            KeyCode::Char('l'),
            KeyCode::Backspace,
        ] {
            send_key(&mut runtime, &mut buffer, code);
        }
        let find = runtime.app().state.find.as_ref().expect("find is open");
        assert_eq!(find.needle, "a");
        assert!(find.typing);
        assert_eq!(runtime.cursor(), Some(Position::new(6, 4)));
        send_key(&mut runtime, &mut buffer, KeyCode::Enter);
        assert!(
            !runtime
                .app()
                .state
                .find
                .as_ref()
                .expect("find remains")
                .typing
        );
        send_key(&mut runtime, &mut buffer, KeyCode::Char('/'));
        send_key(&mut runtime, &mut buffer, KeyCode::Esc);
        assert!(runtime.app().state.find.is_none());

        let (mut read_only, mut read_only_buffer) = focused_runtime("alpha", true);
        send_key(&mut read_only, &mut read_only_buffer, KeyCode::Char('/'));
        send_key(&mut read_only, &mut read_only_buffer, KeyCode::Char('p'));
        assert_eq!(
            read_only
                .app()
                .state
                .find
                .as_ref()
                .map(|find| find.needle.as_str()),
            Some("p")
        );
    }

    #[test]
    fn text_hit_coordinates_are_relative_to_the_text_part() {
        let (mut runtime, mut buffer) = focused_runtime("a界b", false);
        let text = runtime
            .registry()
            .area_of_part(ID, PartRef::of(Part::TEXT))
            .expect("text part");
        for (column, expected) in [(0, 0), (1, 1), (2, 1), (3, 4)] {
            let x = text.x.saturating_add(column);
            let _ = runtime.handle(mouse(MouseKind::Down, x, text.y));
            runtime.draw_buffer(buffer.area, &mut buffer);
            assert_eq!(
                runtime.app().state.cursor_offset(),
                expected,
                "column {column}"
            );
            let _ = runtime.handle(mouse(MouseKind::Up, x, text.y));
            runtime.draw_buffer(buffer.area, &mut buffer);
        }
        let gutter_x = text.x.saturating_sub(1);
        let _ = runtime.handle(mouse(MouseKind::Down, gutter_x, text.y));
        runtime.draw_buffer(buffer.area, &mut buffer);
        assert_eq!(runtime.app().state.cursor_offset(), 4);
    }

    #[test]
    fn read_only_blocks_document_commands_even_if_state_is_editing() {
        let (mut runtime, mut buffer) = focused_runtime("alpha", true);
        runtime.app_mut().state.begin_edit();
        runtime.app_mut().state.jump_to(2);
        let before = runtime.app().state.clone();
        runtime.draw_buffer(buffer.area, &mut buffer);
        for code in [
            KeyCode::Char('x'),
            KeyCode::Backspace,
            KeyCode::Delete,
            KeyCode::Enter,
            KeyCode::Tab,
        ] {
            send_key(&mut runtime, &mut buffer, code);
        }
        assert_eq!(runtime.app().state, before);

        let editor = CodeEditor::new(ID, 4).read_only(true);
        for command in [CodeCmd::InsertMode, CodeCmd::AppendMode] {
            let mut state = CodeEditorState::new("alpha");
            let before = state.clone();
            let mut response = Acc::new();
            editor.command(&mut state, command, 1, &mut response);
            assert_eq!(state, before, "{command:?} changed read-only state");
        }
    }

    #[test]
    fn cursor_is_only_returned_inside_nonempty_text_area() {
        assert_eq!(visible_cursor(Rect::new(2, 3, 0, 1), 0, 0, 0), None);
        assert_eq!(visible_cursor(Rect::new(2, 3, 4, 1), 4, 0, 0), None);
        assert_eq!(visible_cursor(Rect::new(2, 3, 4, 1), 0, 0, 1), None);
        assert_eq!(
            visible_cursor(Rect::new(2, 3, 4, 1), 3, 0, 0),
            Some(Position::new(5, 3))
        );
    }
}
