use core::fmt;
use core::ops::Range;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::completion::{
    COMPLETION_NEXT_FOCUS_CHORD, COMPLETION_PREV_FOCUS_CHORD, CompletingCodeEditor,
    CompletionProvider,
};
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
use crate::theme::{
    Family, FgStep, GlyphRole, Role, Slot, StyleDefaults, StylePatch, SyntaxRole, Variant,
};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_core::edit_entry::EditEntry;
use termrock_runtime::scroll_region::{ScrollRegion, position_label};
use termrock_text::CursorPos;

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

pub trait Highlighter {
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

pub trait Segmenter {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeSeverity {
    Error,

    Warning,

    Info,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TabBehavior {
    #[default]
    Indent,

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CodeDiagnostic {
    pub range: Range<usize>,

    pub severity: CodeSeverity,

    pub message: String,
}

impl CodeDiagnostic {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CodeAction {
    Changed,

    CursorMoved,

    Committed,

    Leave { backward: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeCmd {
    InsertMode,

    AppendMode,

    Commit,

    Move(Motion, Extend),

    Page(bool),

    Backspace,

    Delete,

    DeleteWordLeft,

    DeleteToLineEnd,

    DeleteToLineStart,

    Newline,

    Tab(bool),

    Find,

    FindAccept,

    FindBackspace,

    FindCancel,

    FindNext(bool),

    Block(bool),

    FocusNext,

    FocusPrevious,
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
        COMPLETION_PREV_FOCUS_CHORD,
        CodeCmd::FocusPrevious,
        "focus.previous",
        false,
    ),
    b(
        COMPLETION_NEXT_FOCUS_CHORD,
        CodeCmd::FocusNext,
        "focus.next",
        false,
    ),
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
    b(
        COMPLETION_PREV_FOCUS_CHORD,
        CodeCmd::FocusPrevious,
        "focus.previous",
        false,
    ),
    b(
        COMPLETION_NEXT_FOCUS_CHORD,
        CodeCmd::FocusNext,
        "focus.next",
        false,
    ),
    b(
        Chord::with(KeyCode::Char('f'), CTRL),
        CodeCmd::Find,
        "Find",
        true,
    ),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeFindInfo<'a> {
    pub query: &'a str,

    pub current: Option<usize>,

    pub matches: usize,

    pub typing: bool,
}

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

    pub fn text(&self) -> &str {
        self.editor.text()
    }

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

    pub const fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn is_find_typing(&self) -> bool {
        self.find.as_ref().is_some_and(|find| find.typing)
    }

    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    pub const fn cursor_offset(&self) -> usize {
        self.editor.cursor_offset()
    }

    pub fn cursor_pos(&self) -> CursorPos {
        self.editor.cursor_pos()
    }

    pub fn find_info(&self) -> Option<CodeFindInfo<'_>> {
        self.find.as_ref().map(|find| CodeFindInfo {
            query: &find.needle,
            current: (!find.matches.is_empty()).then_some(find.current),
            matches: find.matches.len(),
            typing: find.typing,
        })
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.editor.selected_text()
    }

    pub fn splice_completion(&mut self, replace_len: usize, insertion: &str) -> bool {
        let end = self.editor.cursor_offset();
        let start = end.saturating_sub(replace_len);
        self.editor.select_range(start, end);
        let outcome = self.editor.apply(EditAction::Paste(insertion));
        self.after_edit(outcome);
        outcome.changed()
    }

    pub fn set_diagnostics(&mut self, mut diagnostics: Vec<CodeDiagnostic>) {
        diagnostics.sort_by_key(|d| d.range.start);
        self.has_error = diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == CodeSeverity::Error);
        self.diagnostics = diagnostics;
    }

    pub fn diagnostics(&self) -> &[CodeDiagnostic] {
        &self.diagnostics
    }

    pub fn set_running(&mut self, block: Option<Range<usize>>) {
        self.running = block;
    }

    pub const fn begin_edit(&mut self) {
        self.editing = true;
    }

    pub fn commit(&mut self) {
        self.editing = false;
        let _ = self.editor.apply(EditAction::ClearSelection);
    }

    pub fn jump_to(&mut self, offset: usize) {
        let offset = floor_boundary(self.text(), offset.min(self.text().len()));
        let (line, col) = line_col(self.text(), offset);
        self.editor.set_cursor_line_col(line, col);
        self.scroll.ensure_visible(line);
    }

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

    blocks: Vec<Range<usize>>,
    blocks_generation: u64,
    blocks_valid: bool,
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

fn current_block_in(blocks: &[Range<usize>], cursor: usize) -> Option<Range<usize>> {
    blocks
        .iter()
        .find(|range| range.start <= cursor && cursor <= range.end)
        .or_else(|| blocks.iter().rev().find(|range| range.end <= cursor))
        .cloned()
}

pub struct CodeEditor<'a> {
    id: Id,
    rows: u16,
    fill_height: bool,
    idle_scroll_position: bool,
    highlighter: Option<&'a dyn Highlighter>,
    segmenter: Option<&'a dyn Segmenter>,
    placeholder: Option<&'a str>,
    indent: u8,
    tab_behavior: TabBehavior,
    read_only: bool,
    disabled: bool,
    edit_entry: Option<EditEntry>,
    commit_on_blur: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for CodeEditor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeEditor")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .field("fill_height", &self.fill_height)
            .field("idle_scroll_position", &self.idle_scroll_position)
            .field("highlighter", &self.highlighter.is_some())
            .field("segmenter", &self.segmenter.is_some())
            .field("placeholder", &self.placeholder)
            .field("indent", &self.indent)
            .field("tab_behavior", &self.tab_behavior)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .field("edit_entry", &self.edit_entry)
            .field("commit_on_blur", &self.commit_on_blur)
            .finish_non_exhaustive()
    }
}

impl<'a> CodeEditor<'a> {
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

    pub const fn new(id: Id, rows: u16) -> Self {
        CodeEditor {
            id,
            rows,
            fill_height: false,
            idle_scroll_position: false,
            highlighter: None,
            segmenter: None,
            placeholder: None,
            indent: 2,
            tab_behavior: TabBehavior::Indent,
            read_only: false,
            disabled: false,
            edit_entry: None,
            commit_on_blur: true,
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn fill_height(mut self, fill: bool) -> Self {
        self.fill_height = fill;
        self
    }

    #[must_use]
    pub const fn idle_scroll_position(mut self, enabled: bool) -> Self {
        self.idle_scroll_position = enabled;
        self
    }

    pub fn with_completion(
        self,
        popup_id: Id,
        provider: &'a dyn CompletionProvider,
    ) -> CompletingCodeEditor<'a> {
        let id = self.id;
        CompletingCodeEditor::new(self, id, popup_id, provider)
    }

    pub(crate) const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub(crate) const fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[must_use]
    pub const fn highlighter(mut self, highlighter: &'a dyn Highlighter) -> Self {
        self.highlighter = Some(highlighter);
        self
    }

    #[must_use]
    pub const fn segmenter(mut self, segmenter: &'a dyn Segmenter) -> Self {
        self.segmenter = Some(segmenter);
        self
    }

    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    #[must_use]
    pub const fn indent(mut self, spaces: u8) -> Self {
        self.indent = spaces;
        self
    }

    #[must_use]
    pub const fn tab_behavior(mut self, behavior: TabBehavior) -> Self {
        self.tab_behavior = behavior;
        self
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
    pub const fn edit_entry(mut self, entry: EditEntry) -> Self {
        self.edit_entry = Some(entry);
        self
    }

    #[must_use]
    pub const fn commit_on_blur(mut self, commit: bool) -> Self {
        self.commit_on_blur = commit;
        self
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self
    }

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

    pub fn selection_or_block(&self, st: &CodeEditorState) -> Option<(String, Range<usize>)> {
        if let Some(range) = st.editor.selection() {
            return Some((st.text().get(range.clone())?.to_owned(), range));
        }
        let range = self.current_block(st)?;
        Some((st.text().get(range.clone())?.to_owned(), range))
    }

    pub fn blocks(&self, st: &CodeEditorState) -> Vec<Range<usize>> {
        let mut blocks = self
            .segmenter
            .map_or_else(Vec::new, |segmenter| segmenter.segments(st.text()));
        blocks.sort_by_key(|range| range.start);
        blocks
    }

    pub fn current_block(&self, st: &CodeEditorState) -> Option<Range<usize>> {
        current_block_in(&self.blocks(st), st.cursor_offset())
    }

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
        let hscroll = usize::from(st.editor.hscroll());
        if let Some(position) = visible_cursor(text_area, cursor.col, y, hscroll) {
            return Some(Rect {
                x: position.x,
                y: position.y,
                width: 1,
                height: 1,
            });
        }

        if text_area.width > 0
            && y < usize::from(text_area.height)
            && cursor.col >= hscroll
            && cursor.col.saturating_sub(hscroll) == usize::from(text_area.width)
        {
            return Some(Rect {
                x: text_area.right().saturating_sub(1),
                y: text_area
                    .y
                    .saturating_add(y.min(usize::from(u16::MAX)) as u16),
                width: 1,
                height: 1,
            });
        }
        None
    }

    pub fn open_find(&self, cx: &mut Cx<'_>, st: &mut CodeEditorState) -> Response<CodeAction> {
        if self.disabled {
            return Response::ignored();
        }
        self.begin_find(cx, st);
        Response::changed()
    }

    fn begin_find(&self, cx: &mut Cx<'_>, st: &mut CodeEditorState) {
        st.open_find();
        cx.focus(self.id);
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut CodeEditorState) -> Response<CodeAction> {
        if self.disabled {
            return Response::ignored();
        }
        let mut acc = Acc::<CodeAction>::new();
        let edit_entry = self.edit_entry.unwrap_or_else(|| cx.edit_entry());
        let lines = st.editor.line_count();
        let bar = self.bar();
        let track = bar.prepare(cx, &mut st.scroll, lines);
        let page = st.scroll.viewport_len().max(1);
        for intent in cx.intents(self.id) {
            acc.fold(&bar.handle_intent(cx, &mut st.scroll, track, intent));
            match intent {
                Intent::FocusIn { .. }
                    if edit_entry == EditEntry::Immediate && !self.read_only && !self.disabled =>
                {
                    if !st.editing {
                        st.begin_edit();
                        acc.changed();
                    }
                }
                Intent::FocusOut { .. } if st.editing && self.commit_on_blur => {
                    st.commit();
                    acc.action(CodeAction::Committed);
                }
                Intent::FocusOut { .. } if st.editing => acc.consumed(),
                Intent::Cancel if st.editing => {
                    st.commit();
                    acc.action(CodeAction::Committed);
                }
                Intent::Binding(action) => {
                    let table = Self::active_bindings(st);
                    if let Some(command) = Binding::command(table, action) {
                        if st.find.as_ref().is_some_and(|find| find.typing) {
                            Self::find_command(st, command, &mut acc);
                        } else {
                            self.dispatch_command(cx, st, command, page, &mut acc);
                        }
                    }
                }
                Intent::Key(key)
                    if st.editing
                        || st.find.as_ref().is_some_and(|find| find.typing)
                        || (edit_entry == EditEntry::TypeToEdit
                            && !self.read_only
                            && key.bare_char().is_some()) =>
                {
                    if !st.editing && !st.find.as_ref().is_some_and(|find| find.typing) {
                        st.begin_edit();
                        acc.changed();
                    }
                    let command = Self::active_bindings(st).iter().find(|binding| {
                        cx.effective_chord(self.id, binding.action, binding.chord)
                            .is_some_and(|chord| chord.matches(&key))
                    });
                    if let Some(binding) = command {
                        let already_dispatched = cx.intents(self.id).any(|intent| {
                            matches!(intent, Intent::Binding(action) if action == binding.action)
                        });
                        if !already_dispatched {
                            if st.find.as_ref().is_some_and(|find| find.typing) {
                                Self::find_command(st, binding.cmd, &mut acc);
                            } else {
                                self.dispatch_command(cx, st, binding.cmd, page, &mut acc);
                            }
                        }
                    } else if let Some(c) = key.bare_char() {
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
                } if !self.disabled => {
                    if !self.read_only {
                        st.editing = true;
                    }
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
                let gutter = Self::gutter_width(lines);
                let text_width = area.width.saturating_sub(gutter);
                st.editor.scroll_into_view(text_width);

                let widest = st.text().split('\n').map(width).max().unwrap_or(0);
                st.editor.clamp_hscroll(widest.saturating_sub(text_width));
            }
        }
        acc.finish(self.id)
    }

    fn dispatch_command(
        &self,
        cx: &mut Cx<'_>,
        st: &mut CodeEditorState,
        command: CodeCmd,
        page: usize,
        acc: &mut Acc<CodeAction>,
    ) {
        match command {
            CodeCmd::Find => {
                self.begin_find(cx, st);
                acc.changed();
            }
            CodeCmd::FocusNext => {
                st.commit();
                cx.focus_next();
                acc.action(CodeAction::Leave { backward: false });
            }
            CodeCmd::FocusPrevious => {
                st.commit();
                cx.focus_prev();
                acc.action(CodeAction::Leave { backward: true });
            }
            command => self.command(st, command, page, acc),
        }
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
                    | CodeCmd::Tab(true)
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
            CodeCmd::Find => acc.consumed(),
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
            CodeCmd::Tab(true) => {
                let text = st.editor.text();
                let selection = st.editor.selection();
                let cursor = st.editor.cursor_offset();
                let anchor = selection.as_ref().map(|range| {
                    if cursor == range.start {
                        range.end
                    } else {
                        range.start
                    }
                });
                let (first_line, last_line) = if let Some(range) = selection.as_ref() {
                    let first = text[..range.start]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count();
                    let before_end = &text[..range.end];
                    let newlines = before_end.bytes().filter(|byte| *byte == b'\n').count();
                    let last = newlines.saturating_sub(usize::from(before_end.ends_with('\n')));
                    (first, last)
                } else {
                    let line = st.editor.cursor_pos().line;
                    (line, line)
                };
                let mut line_starts = vec![0usize];
                line_starts.extend(
                    text.match_indices('\n')
                        .map(|(offset, _)| offset.saturating_add(1)),
                );
                let mut removals = Vec::new();
                for line in first_line..=last_line {
                    let Some(&start) = line_starts.get(line) else {
                        continue;
                    };
                    let end = text[start..]
                        .find('\n')
                        .map_or(text.len(), |relative| start.saturating_add(relative));
                    let count = text.as_bytes()[start..end]
                        .iter()
                        .take(usize::from(self.indent))
                        .take_while(|byte| **byte == b' ')
                        .count();
                    if count > 0 {
                        let removal_start = grapheme_floor_boundary(text, start);
                        let removal_end = grapheme_ceil_boundary(text, start.saturating_add(count));
                        if removal_start < removal_end {
                            removals.push(removal_start..removal_end);
                        }
                    }
                }
                if removals.is_empty() {
                    st.after_edit(EditOutcome::Ignored);
                    acc.action(CodeAction::Changed);
                    return;
                }
                let map_offset = |offset: usize| {
                    let mut removed = 0usize;
                    for range in &removals {
                        if offset >= range.end {
                            removed = removed.saturating_add(range.len());
                        } else if offset > range.start {
                            return range.start.saturating_sub(removed);
                        }
                    }
                    offset.saturating_sub(removed)
                };
                let cursor = map_offset(cursor);
                let anchor = anchor.map(map_offset);
                for range in removals.iter().rev() {
                    st.editor.select_range(range.start, range.end);
                    let _ = st.editor.apply(EditAction::Paste(""));
                }
                if let Some(anchor) = anchor {
                    st.editor.select_range(anchor, cursor);
                } else {
                    st.jump_to(cursor);
                }
                st.after_edit(EditOutcome::Changed);
                acc.action(CodeAction::Changed);
            }
            CodeCmd::FindAccept | CodeCmd::FindBackspace | CodeCmd::FindCancel => acc.consumed(),
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

    #[expect(
        clippy::too_many_lines,
        reason = "single sorted cursor walk over visible code"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &CodeEditorState) -> Rect {
        let used = Rect {
            height: if self.fill_height {
                area.height
            } else {
                self.rows.min(area.height)
            },
            ..area
        };
        if used.is_empty() {
            return used;
        }
        let mut cache = core::mem::take(ui.cache::<HighlightCache>(self.id));
        cache.ensure(st.edit_generation, st.text(), self.highlighter);
        if st.edit_generation != u64::MAX
            && (!cache.blocks_valid || cache.blocks_generation != st.edit_generation)
        {
            cache.blocks = self.blocks(st);
            cache.blocks_generation = st.edit_generation;
            cache.blocks_valid = true;
        }
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
        let live = PartStyle::flags(runtime, derived);
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
        let marker_style = style(ui, Part::MARKER, live).style;
        let gutter_style = style(ui, Part::GUTTER, live).style;

        let block = if st.editor.selection().is_none() {
            if cache.blocks_valid {
                current_block_in(&cache.blocks, st.cursor_offset())
            } else {
                self.current_block(st)
            }
        } else {
            None
        };
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
                if line_index == cursor.line {
                    if live.contains(StateFlags::FOCUSED) {
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
                                    ui.glyph(
                                        cell_at(row, row.x),
                                        GlyphRole::FocusBar,
                                        gutter_style,
                                    );
                                }
                                Slot::Clear => {}
                            },
                        }
                    } else {
                        ui.paint_str(
                            cell_at(row, row.x),
                            " ",
                            container.style.with_fg_from_bg(container.style),
                        );
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
                        let ink = marker_style.patch(
                            ui.paint_patch(
                                &StylePatch::new()
                                    .set_fg(Role::Syntax(SyntaxRole::DiagError))
                                    .add(Modifier::BOLD),
                            ),
                        );
                        ui.paint_str(marker, "!", ink);
                    }
                } else if block
                    .as_ref()
                    .is_some_and(|range| range.start >= global && range.start <= line_end)
                {
                    let ink = if live.contains(StateFlags::FOCUSED) {
                        Role::Fg(FgStep::Primary)
                    } else {
                        Role::Fg(FgStep::Secondary)
                    };
                    ui.paint_str(
                        marker,
                        "›",
                        marker_style.patch(ui.paint_patch(&StylePatch::new().set_fg(ink))),
                    );
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
                let line_start = global;
                let line_span_end = global.saturating_add(code_line.len());
                let in_block = block
                    .as_ref()
                    .is_some_and(|range| line_span_end >= range.start && line_start <= range.end);
                let focused_line = live.contains(StateFlags::FOCUSED) && line_index == cursor.line;
                let defaults = if self.read_only {
                    StylePatch::new().set_fg(Role::Fg(FgStep::Faint))
                } else if focused_line {
                    StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD)
                } else if in_block {
                    StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))
                } else {
                    StylePatch::new().set_fg(Role::Fg(FgStep::Muted))
                };
                let meta_patch = self.ov.part_patch(Part::META);
                let resolved = ui.style_defaults_with_explicit(
                    Family::CODE,
                    Variant::DEFAULT,
                    Part::META,
                    live,
                    StyleDefaults::new(defaults),
                    meta_patch.as_ref(),
                );
                self.ov.note(
                    ui,
                    self.id,
                    Family::CODE,
                    Variant::DEFAULT,
                    Part::META,
                    resolved.resolved,
                );
                let number_style = resolved.resolved.style;
                let mut digits = [0u8; 20];
                let number_text = usize_text(line_index.saturating_add(1), &mut digits);
                let number_width = width(number_text).min(number.width);
                let pad = number.width.saturating_sub(number_width);
                let mut digit_x = number.x;
                for _ in 0..pad {
                    ui.paint_str(
                        Rect {
                            x: digit_x,
                            width: 1,
                            ..number
                        },
                        " ",
                        number_style,
                    );
                    digit_x = digit_x.saturating_add(1);
                }
                ui.paint_str(
                    Rect {
                        x: digit_x,
                        width: number_width,
                        ..number
                    },
                    number_text,
                    number_style,
                );
                let hs = usize::from(st.editor.hscroll());
                let mut column = 0usize;
                let mut x = text_area.x;
                if hs > 0 && !code_line.is_empty() {
                    let ellipsis = text_style
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
                    ui.paint_str(cell_at(row, x), "…", ellipsis);
                }
                for (at, grapheme) in graphemes(code_line) {
                    let cells = usize::from(grapheme_width(grapheme));
                    if column.saturating_add(cells) <= hs {
                        column = column.saturating_add(cells);
                        continue;
                    }

                    if hs > 0 && column == hs {
                        column = column.saturating_add(cells);
                        x = x.saturating_add(cells.min(usize::from(u16::MAX)) as u16);
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
                        painted = painted
                            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Syntax(role))));
                        if role == SyntaxRole::Keyword {
                            painted = painted.add_modifier(Modifier::BOLD);
                        } else if role == SyntaxRole::Number || role == SyntaxRole::Comment {
                            painted = painted.add_modifier(Modifier::ITALIC);
                        }
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
                        painted = painted.add_modifier(Modifier::UNDERLINED);
                        painted = painted.patch(ui.paint_patch(
                            &StylePatch::new().set_underline(diagnostic.severity.role()),
                        ));
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
                        painted = painted
                            .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Syntax(role))));
                    }
                    if st.editing
                        && line_index == cursor.line
                        && !painted
                            .as_style()
                            .add_modifier
                            .contains(Modifier::UNDERLINED)
                    {
                        painted = painted.add_modifier(Modifier::UNDERLINED).patch(
                            ui.paint_patch(&StylePatch::new().set_underline(Role::BorderStrong)),
                        );
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
                if st.editing && line_index == cursor.line && x < text_area.right() {
                    ui.paint_style(
                        Rect {
                            x,
                            y,
                            width: text_area.right().saturating_sub(x),
                            height: 1,
                        },
                        ui.paint_patch(
                            &StylePatch::new()
                                .add(Modifier::UNDERLINED)
                                .set_underline(Role::BorderStrong),
                        ),
                    );
                }
            }
        }
        paint_code_hbar(
            ui,
            text_area,
            content.bottom().saturating_sub(1),
            content.height,
            st.text(),
            usize::from(st.editor.hscroll()),
            container.style,
        );
        *ui.cache::<HighlightCache>(self.id) = cache;
        if st.text().is_empty()
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
                    ui.set_bar_cursor(self.id, Position::new(x, footer.y));
                }
            }
        } else {
            let show_pos = st.editing || live.contains(StateFlags::FOCUSED);
            let pos = if show_pos {
                format!(
                    "ln {}/{} · col {}",
                    cursor.line.saturating_add(1),
                    lines.max(1),
                    cursor.col.saturating_add(1)
                )
            } else if self.idle_scroll_position {
                position_label(&view)
            } else {
                String::new()
            };
            let right_pad = u16::from(footer.width > 1);
            let room = footer.width.saturating_sub(right_pad);
            let pos_width = width(&pos).min(room);
            let pos_x = footer
                .right()
                .saturating_sub(pos_width.saturating_add(right_pad))
                .max(footer.x);
            if let Some(diagnostic) = nearest_diagnostic(&st.diagnostics, st.cursor_offset()) {
                let detail_style = style(ui, Part::DETAIL, live)
                    .style
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(diagnostic.severity.role())));
                let message_x = footer.x.saturating_add(1);
                let message_width = pos_x.saturating_sub(message_x);
                ui.paint_str(
                    Rect {
                        x: message_x,
                        width: message_width,
                        ..footer
                    },
                    &diagnostic.message,
                    detail_style,
                );
            }
            if pos_width > 0 {
                let defaults = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                let meta_patch = self.ov.part_patch(Part::META);
                let resolved = ui.style_defaults_with_explicit(
                    Family::CODE,
                    Variant::DEFAULT,
                    Part::META,
                    live,
                    StyleDefaults::new(defaults),
                    meta_patch.as_ref(),
                );
                self.ov.note(
                    ui,
                    self.id,
                    Family::CODE,
                    Variant::DEFAULT,
                    Part::META,
                    resolved.resolved,
                );
                let pos_style = resolved.resolved.style;
                ui.paint_str(
                    Rect {
                        x: pos_x,
                        width: pos_width,
                        ..footer
                    },
                    &pos,
                    pos_style,
                );
            }
        }

        let show_caret = live.contains(StateFlags::FOCUSED) && (st.editing || self.read_only);
        if show_caret
            && !st.find.as_ref().is_some_and(|find| find.typing)
            && cursor.line >= view.offset()
            && let Some(position) = visible_cursor(
                text_area,
                cursor.col,
                cursor.line.saturating_sub(view.offset()),
                usize::from(st.editor.hscroll()),
            )
        {
            ui.set_bar_cursor(self.id, position);
        }
        used
    }

    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (20, 2),
            preferred: (
                80,
                if self.fill_height {
                    constraints.max.1
                } else {
                    self.rows.max(2)
                },
            ),
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

fn paint_code_hbar(
    ui: &mut Ui<'_>,
    text_area: Rect,
    y: u16,
    body_rows: u16,
    text: &str,
    offset: usize,
    field: crate::theme::PaintStyle,
) {
    let text_w = usize::from(text_area.width);
    if body_rows < 2 || text_w == 0 {
        return;
    }
    let widest = usize::from(text.split('\n').map(width).max().unwrap_or(0));
    if widest.saturating_add(1) <= text_w {
        return;
    }
    let mut across = ScrollState::default();
    across.set_content(widest.saturating_add(1));
    across.set_viewport(text_w);
    across.scroll_to(offset);
    let (start, len) = across.thumb(text_w);
    let end = start.saturating_add(len);
    let track = field.patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::from_u32(0x0047_4747)))),
    );
    let thumb = field.patch(
        ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::from_u32(0x0080_8080)))),
    );
    for i in 0..text_w {
        let on_thumb = i >= start && i < end;
        ui.paint_str(
            Rect::new(text_area.x.saturating_add(i as u16), y, 1, 1),
            if on_thumb { "━" } else { "┄" },
            if on_thumb { thumb } else { track },
        );
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

fn grapheme_floor_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    graphemes(text)
        .map(|(at, _)| at)
        .take_while(|at| *at <= offset)
        .last()
        .unwrap_or(0)
}

fn grapheme_ceil_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    if offset == text.len() {
        return text.len();
    }
    graphemes(text)
        .map(|(at, _)| at)
        .find(|at| *at >= offset)
        .unwrap_or(text.len())
}

fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let prefix = text.get(..offset).unwrap_or("");
    let line = prefix.bytes().filter(|b| *b == b'\n').count();
    let tail = prefix.rsplit_once('\n').map_or(prefix, |(_, tail)| tail);
    (line, usize::from(width(tail)))
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
