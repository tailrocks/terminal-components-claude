//! Code editor: a document editor with a gutter (focus bar, block marker,
//! line numbers, diagnostics), caller-supplied highlighting and block
//! segmentation, horizontal scrolling, selection and an inline find bar.
//! Language knowledge stays outside: the widget only receives spans.
//!
//! Two modes like every text control: focused (navigation) and editing.
//! How focus becomes an edit is the editor's [`EditEntry`] (default the
//! theme's, [`EditEntry::Immediate`]: focus starts the edit and the caret
//! shows at once). While editing Tab indents (unless
//! [`CodeEditor::tab_leaves`]), Esc ends the edit keeping the text
//! ("Done"; [`CodeEditor::esc_reverts`] reverts instead), and then Tab or
//! Shift+Tab leave. Ctrl+Tab and Ctrl+Shift+Tab always leave where the
//! terminal delivers them (the kitty keyboard protocol); hints never
//! advertise them.

use std::ops::Range;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::edit::{EditEntry, EditKind, FieldKeys};
use crate::core::event::{Key, Outcome};
use crate::core::focus::request_focus_next;
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::scroll::{HScroll, ScrollPane, ScrollState, Scrollable};
use crate::core::text::TextBuffer;
use crate::theme::SyntaxTone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{find_ranges, width};
use crate::widgets::edit::{EditField, Editable, LineLook};
use crate::widgets::field_common::{EditAction, PointerField};
use crate::widgets::scrollbar;

pub type Highlighter = fn(&str) -> Vec<(Range<usize>, SyntaxTone)>;
pub type Segmenter = fn(&str) -> Vec<Range<usize>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range<usize>,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct FindState {
    /// The query: the shared one-line [`EditField`] (caret, Select All,
    /// the clipboard keys, undo).
    pub needle: EditField,
    /// Whole-grapheme byte ranges in the original document, including when
    /// case-insensitive matching expands characters such as `İ`.
    pub matches: Vec<Range<usize>>,
    pub current: usize,
    pub editing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    Changed,
    CursorMoved,
    /// Esc or modified Enter from editing: back to navigation (document kept).
    Committed,
    /// Tab from editing when `tab_leaves` is set.
    Leave {
        backward: bool,
    },
}

#[derive(Debug, Clone)]
pub struct CodeEditor {
    pub id: WidgetId,
    /// The document: its text, caret and selection, undo history, keys
    /// and pointer selection (the shared [`EditField`]).
    pub field: EditField,
    pub editing: bool,
    pub read_only: bool,
    /// Rows (`scroll`) and the horizontal bar (`scroll.h`, kept in step
    /// with [`Self::hscroll`]) ([`Scrollable`]).
    pub scroll: ScrollPane,
    pub hscroll: usize,
    pub indent: usize,
    pub highlighter: Option<Highlighter>,
    pub segmenter: Option<Segmenter>,
    pub diagnostics: Vec<Diagnostic>,
    /// Block currently executing (spinner in the marker column).
    pub running: Option<Range<usize>>,
    pub find: Option<FindState>,
    pub placeholder: String,
    /// When true, Tab in editing mode commits and leaves (form-like);
    /// otherwise Tab indents.
    pub tab_leaves: bool,
    pub area: Rect,
    text_area: Rect,
    gutter_w: u16,
    /// Cached wanted column for vertical motion.
    cached_spans: Vec<(Range<usize>, SyntaxTone)>,
    cached_for: u64,
    /// Manual scrolling survives redraws until editing/cursor/content changes.
    rendered_state: Option<(usize, u64, bool)>,
    /// Blank columns between the text and the scrollbar (default
    /// [`scrollbar::GAP`]).
    pub scrollbar_gap: u16,
    /// Horizontal scrolling policy; `None` follows [`RenderCtx::hscroll`].
    pub horizontal: Option<HScroll>,
    /// Which keys move the cursor while reading (default
    /// [`NavKeys::default`]; ←/→ scroll sideways). Editing keys are the
    /// text buffer's own.
    pub nav: NavKeys,
    /// The reading-mode action keys (default
    /// [`CodeCmd::default_keymap`]).
    pub keymap: KeyMap<CodeCmd>,
    /// How focus starts an edit. `None` (default) follows the theme's
    /// [`crate::theme::Theme::edit_entry`] ([`EditEntry::Immediate`]).
    /// A read-only editor never edits.
    pub edit_entry: Option<EditEntry>,
    entry: EditEntry,
    /// Esc reverts the document to what it was when the edit began
    /// instead of keeping it. Default `false` (Esc is "Done"; Ctrl+Z
    /// undoes).
    pub esc_reverts: bool,
    /// The control-key preset. `None` (default) follows the theme's
    /// [`crate::theme::Theme::field_keys`].
    pub field_keys: Option<FieldKeys>,
    /// The document as the edit began, for [`Self::esc_reverts`].
    snapshot: Option<String>,
    focus_seen: bool,
}

/// A [`CodeEditor`] reading-mode action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeCmd {
    /// Start editing at the cursor. Default Enter and `i`.
    Edit,
    /// Start editing after the cursor. Default `a`.
    Append,
    /// Jump to the previous block. Default `{`.
    PrevBlock,
    /// Jump to the next block. Default `}`.
    NextBlock,
    /// Open the find bar. Default `/`.
    Find,
    /// Next find match. Default `n`.
    NextMatch,
    /// Previous find match. Default `N`.
    PrevMatch,
}

impl Actions for CodeCmd {
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::key(KeyCode::Enter), CodeCmd::Edit),
            (KeyBinding::char('i'), CodeCmd::Edit),
            (KeyBinding::char('a'), CodeCmd::Append),
            (KeyBinding::char('{'), CodeCmd::PrevBlock),
            (KeyBinding::char('}'), CodeCmd::NextBlock),
            (KeyBinding::char('/'), CodeCmd::Find),
            (KeyBinding::char('n'), CodeCmd::NextMatch),
            (KeyBinding::char('N'), CodeCmd::PrevMatch),
        ]
    }
}

fn hash_text(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ s.len() as u64
}

impl CodeEditor {
    pub fn new(id: WidgetId, text: &str) -> Self {
        let mut field = EditField::multi(text);
        field.buffer.move_doc_start(false);
        Self {
            id,
            field,
            editing: false,
            read_only: false,
            scroll: ScrollPane::default(),
            hscroll: 0,
            indent: 2,
            highlighter: None,
            segmenter: None,
            diagnostics: vec![],
            running: None,
            find: None,
            placeholder: String::new(),
            tab_leaves: false,
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            gutter_w: 0,
            cached_spans: vec![],
            cached_for: 0,
            rendered_state: None,
            scrollbar_gap: scrollbar::GAP,
            horizontal: None,
            nav: NavKeys::default(),
            keymap: CodeCmd::default_keymap(),
            edit_entry: None,
            entry: crate::theme::Theme::DEFAULT_EDIT_ENTRY,
            esc_reverts: false,
            field_keys: None,
            snapshot: None,
            focus_seen: false,
        }
    }

    /// See [`Self::edit_entry`].
    pub fn edit_entry(mut self, entry: EditEntry) -> Self {
        self.edit_entry = Some(entry);
        self.entry = entry;
        self
    }
    /// See [`Self::esc_reverts`].
    pub fn esc_reverts(mut self, on: bool) -> Self {
        self.esc_reverts = on;
        self
    }
    /// See [`Self::tab_leaves`].
    pub fn tab_leaves(mut self, on: bool) -> Self {
        self.tab_leaves = on;
        self
    }
    /// See [`Self::text_keymap`].
    pub fn text_keymap(
        mut self,
        keymap: crate::core::keymap::KeyMap<crate::core::edit::TextCmd>,
    ) -> Self {
        self.field.text_keymap = Some(keymap);
        self
    }
    /// The edit action `key` means under the resolved preset and text keys.
    fn edit_action(&self, key: &Key) -> EditAction {
        self.field.action(key)
    }
    /// See [`Self::field_keys`].
    pub fn field_keys(mut self, keys: FieldKeys) -> Self {
        self.field_keys = Some(keys);
        self.field.keys = keys;
        self
    }
    /// The entry policy in effect.
    pub fn entry(&self) -> EditEntry {
        self.edit_entry.unwrap_or(self.entry)
    }
    /// The footer hints for the editor's own keys while it holds focus:
    /// while editing, how to leave ([`crate::widgets::keyhint::EDIT_CODE`]
    /// `Tab Indent · Esc Done` when Tab indents, `Tab Next field · Esc
    /// Done` under [`Self::tab_leaves`]); after Esc, `Enter Edit · Tab
    /// Next`. Ctrl+Tab is never advertised (most terminals send plain Tab).
    pub fn hints(&self) -> Vec<crate::widgets::keyhint::Hint> {
        use crate::widgets::keyhint::{EDIT_CODE, hint};
        let esc = if self.esc_reverts { "Cancel" } else { "Done" };
        if self.find.as_ref().is_some_and(|f| f.editing) {
            vec![hint("Enter", "Find"), hint("Esc", "Close")]
        } else if self.editing && self.tab_leaves {
            vec![hint("Tab", "Next field"), hint("Esc", esc)]
        } else if self.editing && !self.esc_reverts {
            EDIT_CODE.to_vec()
        } else if self.editing {
            vec![hint("Tab", "Indent"), hint("Esc", esc)]
        } else if self.read_only {
            vec![hint("Tab", "Next")]
        } else {
            vec![hint("Enter", "Edit"), hint("Tab", "Next")]
        }
    }

    /// The focused editor takes printable keys before page and app letter
    /// shortcuts: the document or its find bar is being edited.
    pub fn wants_text_input(&self) -> bool {
        self.find.as_ref().is_some_and(|f| f.editing) || (self.editing && !self.read_only)
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<CodeCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    /// Blank columns between the text and the scrollbar.
    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    /// Override the app-wide horizontal scrolling policy.
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    /// The horizontal scroll position: what the bar shows.
    pub fn hbar(&self) -> &ScrollState {
        &self.scroll.h
    }

    /// Move sideways by `cells`, clamped to the widest line once it has
    /// been measured. Returns whether the view moved.
    fn scroll_h(&mut self, cells: isize) -> bool {
        let before = self.hscroll;
        let max = if self.scroll.h.viewport_len > 0 {
            self.scroll.h.max_offset()
        } else {
            usize::MAX
        };
        self.hscroll = self.hscroll.saturating_add_signed(cells).min(max);
        self.scroll.h.offset = self.hscroll;
        self.hscroll != before
    }

    pub fn highlighter(mut self, h: Highlighter) -> Self {
        self.highlighter = Some(h);
        self
    }
    pub fn segmenter(mut self, s: Segmenter) -> Self {
        self.segmenter = Some(s);
        self
    }
    pub fn read_only(mut self, ro: bool) -> Self {
        self.read_only = ro;
        self
    }
    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }

    pub fn text(&self) -> &str {
        self.field.buffer.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.field.set_text(text);
        self.field.buffer.move_doc_start(false);
        self.diagnostics.clear();
        self.hscroll = 0;
        self.scroll.jump_start();
        self.refind();
    }

    pub fn is_empty(&self) -> bool {
        self.field.buffer.is_empty()
    }

    /// Block (statement) containing the cursor, per the segmenter.
    pub fn current_block(&self) -> Option<Range<usize>> {
        let seg = self.segmenter?;
        let cur = self.field.buffer.cursor_offset();
        let blocks = seg(self.field.buffer.text());
        blocks
            .iter()
            .find(|b| cur >= b.start && cur <= b.end)
            .or_else(|| blocks.iter().rev().find(|b| b.end <= cur))
            .cloned()
    }

    /// Selection text and range, or the current block.
    pub fn selection_or_block(&self) -> Option<(String, Range<usize>)> {
        if let Some(r) = self.field.buffer.selection() {
            return Some((self.field.buffer.text()[r.clone()].to_owned(), r));
        }
        let b = self.current_block()?;
        Some((self.field.buffer.text()[b.clone()].to_owned(), b))
    }

    /// Byte ranges of all blocks.
    pub fn blocks(&self) -> Vec<Range<usize>> {
        self.segmenter
            .map(|s| s(self.field.buffer.text()))
            .unwrap_or_default()
    }

    pub fn jump_to(&mut self, offset: usize) {
        let offset = offset.min(self.field.buffer.text().len());
        let pos = TextBuffer::pos_of(self.field.buffer.text(), offset);
        self.field.buffer.set_cursor_line_col(pos.line, pos.col);
        self.scroll.ensure_visible(pos.line);
    }

    pub fn cursor_offset(&self) -> usize {
        self.field.buffer.cursor_offset()
    }

    /// Screen cell of the cursor (anchor for a completion popup).
    pub fn cursor_cell(&self) -> Option<Rect> {
        let pos = self.field.buffer.cursor_pos();
        if pos.line < self.scroll.offset
            || pos.line >= self.scroll.offset + self.scroll.viewport_len
            || pos.col < self.hscroll
            || pos.col >= self.hscroll.saturating_add(self.text_area.width as usize)
        {
            return None;
        }
        let y = self.text_area.y + (pos.line - self.scroll.offset) as u16;
        let x = self.text_area.x + pos.col.saturating_sub(self.hscroll) as u16;
        Some(Rect::new(x, y, 1, 1))
    }

    pub fn begin_edit(&mut self) {
        if !self.read_only {
            if !self.editing {
                self.snapshot = Some(self.field.buffer.text().to_owned());
            }
            self.editing = true;
        }
    }

    /// Mark a block as executing (spinner in the marker column).
    pub fn set_running(&mut self, block: Option<Range<usize>>) {
        self.running = block;
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.field.buffer.clear_selection();
    }

    // ---- find -------------------------------------------------------

    /// Open a single-line query. Uppercase queries match case exactly;
    /// lowercase queries use Unicode lowercase matching and source graphemes.
    /// Long queries scroll within the footer to keep their insertion point visible.
    pub fn open_find(&mut self) {
        let mut needle = self
            .find
            .as_ref()
            .map(|f| f.needle.clone())
            .unwrap_or_else(|| EditField::single(""));
        needle.keys = self.field.keys;
        self.find = Some(FindState {
            needle,
            matches: vec![],
            current: 0,
            editing: true,
        });
        self.refind();
    }

    fn refind(&mut self) {
        let Some(f) = self.find.as_mut() else {
            return;
        };
        f.matches.clear();
        f.current = 0;
        let needle = f.needle.text();
        if needle.is_empty() {
            return;
        }
        let case_sensitive = needle.chars().any(|c| c.is_uppercase());
        f.matches = find_ranges(self.field.buffer.text(), needle, case_sensitive);
        let cur = self.field.buffer.cursor_offset();
        f.current = f.matches.iter().position(|m| m.start >= cur).unwrap_or(0);
    }

    fn goto_match(&mut self, delta: isize) {
        let Some(f) = self.find.as_mut() else {
            return;
        };
        if f.matches.is_empty() {
            return;
        }
        let n = f.matches.len() as isize;
        f.current = ((f.current as isize + delta).rem_euclid(n)) as usize;
        let m = f.matches[f.current].clone();
        self.jump_to(m.start);
    }

    // ---- keys -------------------------------------------------------

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<EditorAction>) {
        // Ctrl+C copies only a selection; with nothing selected it is not
        // the editor's, so it reaches the app's quit guard (as in
        // TextInput and TextArea), editing or not
        if key.ctrl_char('c') && self.buffer.selected_text().is_none() {
            return (Outcome::Ignored, None);
        }
        // find bar
        if let Some(f) = self.find.as_mut()
            && f.editing
        {
            {
                match key.code {
                    KeyCode::Esc => {
                        self.find = None;
                        return (Outcome::Changed, None);
                    }
                    KeyCode::Enter => {
                        if key.shift() {
                            self.goto_match(-1);
                        } else if f.needle.text().is_empty() {
                            f.editing = false;
                        } else {
                            f.editing = false;
                            self.goto_match(0);
                        }
                        return (Outcome::Changed, Some(EditorAction::CursorMoved));
                    }
                    // the query is a text field: typing, the caret keys,
                    // Select All, the clipboard keys and undo
                    _ => {
                        let r = f.needle.on_key(key);
                        if r.edited {
                            self.refind();
                            self.goto_match(0);
                            return (Outcome::Changed, Some(EditorAction::CursorMoved));
                        }
                        return (
                            if r.outcome == Outcome::Changed {
                                Outcome::Changed
                            } else {
                                Outcome::Consumed
                            },
                            None,
                        );
                    }
                }
            }
        }
        // Public read_only can change between events. Reconcile the document
        // mode before dispatch so indentation cannot bypass mutation guards.
        if self.read_only && self.editing {
            self.commit();
        }
        // F6 / Shift+F6 leave the editor from any mode (K17): the pane key
        // terminals deliver, where Ctrl+Tab usually never arrives
        if key.code == KeyCode::F(6) && !key.ctrl() && !key.alt() {
            let backward = key.shift();
            self.commit();
            request_focus_next(backward);
            return (Outcome::Changed, Some(EditorAction::Leave { backward }));
        }
        if !self.editing {
            if self.entry() == EditEntry::Immediate && !self.focus_seen && !self.read_only {
                // focus arrived and this key beat the first render
                self.begin_edit();
            } else {
                return self.nav_key(key);
            }
        }
        // Ctrl+Tab / Ctrl+Shift+Tab always leave, where the terminal
        // delivers them
        if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) && key.ctrl() {
            let backward = key.code == KeyCode::BackTab || key.shift();
            self.commit();
            request_focus_next(backward);
            return (Outcome::Changed, Some(EditorAction::Leave { backward }));
        }
        let action = self.edit_action(key);
        match action {
            EditAction::Cancel if self.esc_reverts => {
                if let Some(snap) = self.snapshot.take()
                    && snap != self.field.buffer.text()
                {
                    self.field.record(EditKind::Other);
                    self.field.buffer.set_text(snap);
                    self.after_change();
                }
                self.commit();
                (Outcome::Changed, Some(EditorAction::Committed))
            }
            EditAction::Cancel | EditAction::Commit => {
                self.commit();
                (Outcome::Changed, Some(EditorAction::Committed))
            }
            EditAction::Tab { backward } => {
                if self.tab_leaves {
                    self.commit();
                    request_focus_next(backward);
                    return (Outcome::Changed, Some(EditorAction::Leave { backward }));
                }
                self.field.record(EditKind::Other);
                if backward {
                    self.dedent_selection();
                } else if self.field.buffer.has_selection_lines() {
                    self.indent_selection();
                } else {
                    let spaces = " ".repeat(self.indent);
                    self.field.buffer.insert_str(&spaces);
                }
                self.after_change();
                (Outcome::Changed, Some(EditorAction::Changed))
            }
            EditAction::Apply(_) | EditAction::Insert(_) if self.read_only => {
                (Outcome::Consumed, None)
            }
            EditAction::None => match key.code {
                KeyCode::PageUp if !key.chord() => {
                    for _ in 0..self.scroll.viewport_len.max(1) {
                        self.field.buffer.move_up(false);
                    }
                    self.after_change();
                    (Outcome::Changed, Some(EditorAction::CursorMoved))
                }
                KeyCode::PageDown if !key.chord() => {
                    for _ in 0..self.scroll.viewport_len.max(1) {
                        self.field.buffer.move_down(false);
                    }
                    self.after_change();
                    (Outcome::Changed, Some(EditorAction::CursorMoved))
                }
                _ => (Outcome::Ignored, None),
            },
            action => {
                let edited = self.field.apply_as(action, key, self.read_only).edited;
                self.after_change();
                (
                    Outcome::Changed,
                    Some(if edited {
                        EditorAction::Changed
                    } else {
                        EditorAction::CursorMoved
                    }),
                )
            }
        }
    }

    /// The text command `key` is bound to in the editor's text key map
    /// (its own, or the preset's [`FieldKeys::text_keymap`]).
    fn text_cmd(&self, key: &Key) -> Option<crate::core::edit::TextCmd> {
        self.field.text_cmd(key)
    }

    fn nav_key(&mut self, key: &Key) -> (Outcome, Option<EditorAction>) {
        use crate::core::edit::TextCmd;
        // The text commands work outside an edit too: Select All, Copy and
        // the Cmd caret moves select and copy from a read-only document;
        // Cmd+X/V/Z pick the edit up where it can edit. The preset's
        // control keys for those (Ctrl+W, Ctrl+Y, Ctrl+Z) and a Copy with
        // nothing selected stay with the page, as before.
        if let Some(cmd) = self.text_cmd(key) {
            match cmd {
                TextCmd::Cut | TextCmd::Paste | TextCmd::Undo | TextCmd::Redo if !key.cmd() => {}
                TextCmd::Copy if self.field.buffer.selected_text().is_none() => {}
                TextCmd::Cut | TextCmd::Paste | TextCmd::Undo | TextCmd::Redo => {
                    if self.read_only {
                        return (Outcome::Consumed, None);
                    }
                    self.begin_edit();
                    return self.on_key(key);
                }
                _ => {
                    let action = crate::widgets::field_common::text_cmd_action(cmd, true);
                    if matches!(
                        action,
                        EditAction::Commit
                            | EditAction::Cancel
                            | EditAction::Tab { .. }
                            | EditAction::None
                    ) {
                        return (Outcome::Ignored, None);
                    }
                    self.field.apply_as(action, key, true);
                    self.after_change();
                    return (Outcome::Changed, Some(EditorAction::CursorMoved));
                }
            }
        }
        if let Some(action) = lookup(&self.keymap, key) {
            return self.on_action(action);
        }
        let plain = key.plain();
        match self.nav.action(key) {
            Some(Nav::Up) if plain => {
                self.field.buffer.move_up(false);
                self.after_change();
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            Some(Nav::Down) if plain => {
                self.field.buffer.move_down(false);
                self.after_change();
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            Some(Nav::Left) if plain => {
                self.scroll_h(-8);
                (Outcome::Changed, None)
            }
            Some(Nav::Right) if plain => {
                self.scroll_h(8);
                (Outcome::Changed, None)
            }
            Some(Nav::PageUp) => {
                self.scroll.page_up();
                (Outcome::Changed, None)
            }
            Some(Nav::PageDown) => {
                self.scroll.page_down();
                (Outcome::Changed, None)
            }
            Some(Nav::Home) if plain => {
                self.field.buffer.move_doc_start(false);
                self.after_change();
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            Some(Nav::End) => {
                self.field.buffer.move_doc_end(false);
                self.after_change();
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            _ if key.code == KeyCode::Esc => {
                if self.find.is_some() {
                    self.find = None;
                    (Outcome::Changed, None)
                } else {
                    (Outcome::Ignored, None)
                }
            }
            _ => (Outcome::Ignored, None),
        }
    }

    fn on_action(&mut self, action: CodeCmd) -> (Outcome, Option<EditorAction>) {
        match action {
            CodeCmd::Edit => {
                self.begin_edit();
                (Outcome::Changed, None)
            }
            CodeCmd::Append => {
                self.begin_edit();
                self.field.buffer.move_right(false);
                (Outcome::Changed, None)
            }
            CodeCmd::PrevBlock => {
                let cur = self.field.buffer.cursor_offset();
                if let Some(b) = self.blocks().iter().rev().find(|b| b.start < cur) {
                    self.jump_to(b.start);
                }
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            CodeCmd::NextBlock => {
                let cur = self.field.buffer.cursor_offset();
                if let Some(b) = self.blocks().iter().find(|b| b.start > cur) {
                    self.jump_to(b.start);
                }
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            CodeCmd::Find => {
                self.open_find();
                (Outcome::Changed, None)
            }
            CodeCmd::NextMatch => {
                self.goto_match(1);
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
            CodeCmd::PrevMatch => {
                self.goto_match(-1);
                (Outcome::Changed, Some(EditorAction::CursorMoved))
            }
        }
    }

    fn after_change(&mut self) {
        let pos = self.field.buffer.cursor_pos();
        self.scroll.set_content(self.field.buffer.line_count());
        self.scroll.ensure_visible(pos.line);
        let w = self.text_area.width.max(8) as usize;
        if pos.col < self.hscroll + 4 {
            self.hscroll = pos.col.saturating_sub(4);
        } else if pos.col + 4 >= self.hscroll + w {
            self.hscroll = pos.col + 5 - w;
        }
        if self.find.is_some() {
            self.refind();
        }
    }

    fn indent_selection(&mut self) {
        let spaces = " ".repeat(self.indent);
        let (a, b) = self.field.buffer.selection_lines();
        for line in a..=b {
            let off = self.field.buffer.offset_at(line, 0);
            self.field.buffer.insert_at(off, &spaces);
        }
    }

    fn dedent_selection(&mut self) {
        let (a, b) = self.field.buffer.selection_lines();
        for line in a..=b {
            let off = self.field.buffer.offset_at(line, 0);
            let text = self.field.buffer.text();
            let n = text[off..]
                .chars()
                .take(self.indent)
                .take_while(|c| *c == ' ')
                .count();
            self.field.buffer.remove_range(off..off + n);
        }
    }

    // ---- mouse ------------------------------------------------------

    /// The byte offset under `pos`. A pointer above or left of the text
    /// names the line or column before the first visible one, so a drag
    /// past the edge scrolls the editor and the selection runs on (M2).
    fn offset_at_pos(&self, pos: Position) -> usize {
        let line = (self.scroll.offset as isize + (pos.y as isize - self.text_area.y as isize))
            .max(0) as usize;
        let line = line.min(self.field.buffer.line_count().saturating_sub(1));
        let col =
            (self.hscroll as isize + (pos.x as isize - self.text_area.x as isize)).max(0) as usize;
        self.field.buffer.offset_at(line, col)
    }

    /// A click enters editing (one click, never two) and places the caret
    /// at the pointer; a read-only editor only moves the caret. Same as
    /// [`PointerField::on_click_count`] with a count of 1.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        PointerField::on_click_count(self, pos, 1)
    }

    /// A drag held since a press on the text selects from the press.
    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        PointerField::on_drag(self, pos)
    }

    /// Paste into the active find query, or into the editable document.
    /// Find stays available for read-only documents; its single-line query
    /// uses the same paste normalization as other single-line text controls.
    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(find) = self.find.as_mut()
            && find.editing
        {
            find.needle.paste(text);
            self.refind();
            self.goto_match(0);
            return Outcome::Changed;
        }
        if !self.editing || self.read_only {
            return Outcome::Ignored;
        }
        self.field.record(EditKind::Other);
        self.field.buffer.insert_str(text);
        self.after_change();
        Outcome::Changed
    }

    // ---- render -----------------------------------------------------

    fn spans(&mut self) -> &[(Range<usize>, SyntaxTone)] {
        let h = hash_text(self.field.buffer.text());
        if h != self.cached_for {
            self.cached_spans = self
                .highlighter
                .map(|f| f(self.field.buffer.text()))
                .unwrap_or_default();
            self.cached_for = h;
        }
        &self.cached_spans
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.scroll.track_click = Some(click);
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.scroll.begin(ctx.track_click);
        self.entry = self.edit_entry.unwrap_or(ctx.theme.edit_entry);
        self.field.keys = self.field_keys.or(ctx.theme.field_keys).unwrap_or_default();
        // focus newly gained starts an immediate edit: the caret shows the
        // moment Tab lands here
        let shown = ctx.interaction.focused(self.id) && !ctx.inert;
        if shown && !self.focus_seen && self.entry == EditEntry::Immediate && !self.read_only {
            self.begin_edit();
        }
        self.focus_seen = shown;
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        // The parent buffer can be much larger than this editor's pane.
        let put = |buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style| {
            if area.contains(Position::new(x, y)) {
                buf.set_stringn(x, y, text, (area.right() - x) as usize, style);
            }
        };
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.editing = self.editing && s.focused;
        // Read-only documents still accept focus, navigation, selection and find.
        if (!s.focused || self.read_only) && self.editing {
            self.commit();
            s.editing = false;
        }
        let focused = s.focused;
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        ctx.pointer_shape(self.id, crate::core::shape::PointerShape::Text);

        let fs = if self.read_only {
            Style::new().fg(t.text_primary).bg(bg)
        } else {
            t.field_style(s)
        };
        fill(buf, area, fs);
        let body_h = area.height.saturating_sub(1);
        let spans: Vec<(Range<usize>, SyntaxTone)> = self.spans().to_vec();
        let text_owned = self.field.buffer.text().to_owned();
        let lines: Vec<&str> = text_owned.split('\n').collect();
        let line_count = lines.len();
        let num_w = (line_count.to_string().len() as u16).max(2);
        // gutter: bar(1) marker(1) space(1) numbers(num_w) space(1)
        let gutter_w = (1 + 1 + num_w + 1 + 1).min(area.width);
        self.gutter_w = gutter_w;
        // both bars are settled before the viewport is set: the horizontal
        // bar takes the last text row, which can make the lines overflow
        // and cost a column for the vertical bar
        self.scroll.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        let widest = lines.iter().map(|l| width(l)).max().unwrap_or(0);
        let gap = self.scrollbar_gap;
        let text_w = |has_sb: bool| {
            area.width
                .saturating_sub(gutter_w + if has_sb { scrollbar::reserve(gap) } else { 0 })
        };
        let mut has_sb = line_count > body_h as usize;
        // one spare cell past the widest line, where the caret can sit
        self.scroll.h.offset = self.hscroll;
        self.scroll.h.set_content(widest + 1);
        self.scroll.h.set_viewport(text_w(has_sb) as usize);
        let has_hb =
            self.scroll.h_cfg.enabled && body_h > 1 && self.scroll.h_cfg.shows_bar(&self.scroll.h);
        let rows_h = body_h - u16::from(has_hb);
        if !has_sb && line_count > rows_h as usize {
            has_sb = true;
            self.scroll.h.set_viewport(text_w(true) as usize);
        }
        // never scrolled past the widest line (the arrows used to run on)
        self.hscroll = self.scroll.h.offset;
        let rows = rows_h as usize;
        self.scroll.set_content(line_count);
        self.scroll.set_viewport(rows);
        let text_area = Rect::new(area.x + gutter_w, area.y, text_w(has_sb), rows_h);
        let viewport_changed =
            self.text_area.width != text_area.width || self.text_area.height != text_area.height;
        self.text_area = text_area;
        if self.field.sel.drag {
            // a drag held past an edge keeps selecting and scrolls (M2)
            ctx.autoscroll(text_area);
        }
        let cur = self.field.buffer.cursor_pos();
        let cur_off = self.field.buffer.cursor_offset();
        let rendered_state = (cur_off, self.cached_for, s.editing);
        if s.editing && (viewport_changed || self.rendered_state != Some(rendered_state)) {
            self.scroll.ensure_visible(cur.line);
        }
        self.rendered_state = Some(rendered_state);
        let block = if self.field.buffer.selection().is_none() {
            self.current_block()
        } else {
            None
        };
        let running = self.running.clone();
        let sel = self.field.buffer.selection();
        let find = self.find.clone();
        let diags = self.diagnostics.clone();
        let bracket = self.bracket_pair();

        // line offsets
        let mut line_starts = Vec::with_capacity(line_count);
        let mut acc = 0;
        for l in &lines {
            line_starts.push(acc);
            acc += l.len() + 1;
        }
        let tone_at = |off: usize| -> SyntaxTone {
            spans
                .iter()
                .find(|(r, _)| r.contains(&off))
                .map(|(_, tn)| *tn)
                .unwrap_or(SyntaxTone::Plain)
        };
        let gutter_bar = t.gutter(s, fs.bg.unwrap_or(bg), false);
        for row in 0..rows {
            let li = self.scroll.offset + row;
            let y = area.y + row as u16;
            // the bar marks the line the cursor is on, like a row in a list
            if li == cur.line {
                put(buf, area.x, y, t.gutter_symbol(s), gutter_bar);
            }
            if li >= line_count {
                continue;
            }
            let line = lines[li];
            let ls = line_starts[li];
            let le = ls + line.len();
            // marker column
            let mut marker = " ";
            let mut marker_style = fs;
            if let Some(r) = &running {
                if r.start >= ls && r.start <= le {
                    marker = t.glyphs.spinner_frame(ctx.interaction.tick);
                    marker_style = fs.fg(t.accent);
                }
            } else if let Some(b) = &block {
                let first_content = b.start;
                if first_content >= ls && first_content <= le {
                    marker = t.glyphs.current;
                    // the block marker is a position aid beside the
                    // focus bar, never a second green cue
                    marker_style = fs.fg(if focused {
                        t.text_primary
                    } else {
                        t.text_secondary
                    });
                }
            }
            // a diagnostic owns the marker slot
            let diag_here = diags
                .iter()
                .find(|d| d.range.start >= ls && d.range.start <= le);
            if let Some(d) = diag_here {
                let c = if d.severity == Severity::Error {
                    t.error
                } else {
                    t.warning
                };
                marker = "!";
                marker_style = fs.fg(c).add_modifier(Modifier::BOLD);
            }
            put(buf, area.x + 1, y, marker, marker_style);
            // line number
            let in_block = block.as_ref().is_some_and(|b| le >= b.start && ls <= b.end);
            let ns = if li == cur.line && focused {
                fs.fg(t.text_primary).add_modifier(Modifier::BOLD)
            } else if in_block {
                fs.fg(t.text_secondary)
            } else {
                fs.fg(t.text_muted)
            };
            let ns = if self.read_only {
                fs.fg(t.text_faint)
            } else {
                ns
            };
            put(
                buf,
                area.x + 3,
                y,
                &crate::ui::text::fit_right(&(li + 1).to_string(), num_w as usize),
                ns,
            );
            if text_area.width == 0 {
                continue;
            }
            // text
            let mut x = text_area.x;
            let mut col = 0usize;
            if self.hscroll > 0 && !line.is_empty() {
                put(buf, x, y, t.glyphs.ellipsis, fs.fg(t.text_muted));
            }
            let underline_line = s.editing && li == cur.line;
            for (gi, g) in line.grapheme_indices(true) {
                let gw = width(g);
                if col + gw <= self.hscroll {
                    col += gw;
                    continue;
                }
                if col >= self.hscroll && col == self.hscroll && self.hscroll > 0 {
                    // first visible cell is the `…`
                    col += gw;
                    x += gw as u16;
                    continue;
                }
                if x + gw as u16 > text_area.right() {
                    put(
                        buf,
                        text_area.right().saturating_sub(1),
                        y,
                        t.glyphs.ellipsis,
                        fs.fg(t.text_muted),
                    );
                    break;
                }
                let off = ls + gi;
                let mut st = fs.patch(t.syntax(tone_at(off)));
                if let Some(r) = &sel
                    && r.contains(&off)
                {
                    let sel = t.selection();
                    st = st.bg(t.selection_bg).add_modifier(sel.add_modifier);
                }
                if let Some(f) = &find
                    && let Some(mi) = f.matches.iter().position(|m| m.contains(&off))
                {
                    st = if mi == f.current {
                        st.patch(t.match_current())
                    } else {
                        st.add_modifier(Modifier::UNDERLINED)
                            .underline_color(t.border_strong)
                    };
                }
                if let Some(d) = diags.iter().find(|d| {
                    d.range.contains(&off) || (d.range.is_empty() && d.range.start == off)
                }) {
                    let c = if d.severity == Severity::Error {
                        t.error
                    } else {
                        t.warning
                    };
                    st = st.add_modifier(Modifier::UNDERLINED).underline_color(c);
                }
                if bracket.is_some_and(|(a, b)| off == a || off == b) {
                    st = st
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                        .underline_color(t.border_strong);
                }
                if underline_line && !st.add_modifier.contains(Modifier::UNDERLINED) {
                    st = st
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.border_strong);
                }
                put(buf, x, y, g, st);
                x += gw as u16;
                col += gw;
            }
            if underline_line {
                for xx in x..text_area.right() {
                    if let Some(c) = buf.cell_mut(Position::new(xx, y)) {
                        c.set_style(
                            c.style()
                                .add_modifier(Modifier::UNDERLINED)
                                .underline_color(t.border_strong),
                        );
                    }
                }
            }
        }
        // the placeholder stays until the first character, with the caret on
        // it while editing (macOS)
        if self.field.buffer.is_empty() && !self.placeholder.is_empty() {
            put(
                buf,
                text_area.x,
                text_area.y,
                &crate::ui::text::truncate(&self.placeholder, text_area.width as usize),
                fs.fg(t.text_muted),
            );
        }
        // a read-only document is navigated by its caret: it shows while
        // the editor has focus, so a click or an arrow visibly moves it
        // (M39)
        if (s.editing || (focused && self.read_only))
            && let Some(cursor) = self.cursor_cell()
        {
            ctx.set_cursor(Position::new(cursor.x, cursor.y));
        }
        if has_sb {
            let body = Rect::new(area.x, area.y, area.width, rows_h);
            self.scroll
                .draw_with_fade(body, buf, ctx, self.id, focused, &[]);
        }
        if has_hb {
            // under the text, the gutter stays put; the corner stays blank
            let y = area.y + rows_h;
            fill(buf, Rect::new(area.x, y, area.width, 1), fs);
            let hb = Rect::new(text_area.x, y, text_area.width, 1);
            self.scroll.draw_h(hb, buf, ctx, self.id, focused);
        }
        // footer row
        let fy = area.y + body_h;
        if let Some(f) = &self.find {
            let left = area.x + u16::from(area.width > 2);
            let available = (area.right() - left) as usize;
            let label = if available >= 8 { "find " } else { "" };
            put(buf, left, fy, label, fs.fg(t.text_muted));
            let nx = left + label.len() as u16;
            let ns = if f.editing {
                fs.add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.accent)
            } else {
                fs
            };
            let count = if f.matches.is_empty() {
                if f.needle.text().is_empty() {
                    String::new()
                } else {
                    "no matches".into()
                }
            } else {
                format!("{}/{}", f.current + 1, f.matches.len())
            };
            let count_width = width(&count);
            let count_room = if !count.is_empty() && available >= label.len() + count_width + 4 {
                count_width + 1
            } else {
                0
            };
            let query_room = available.saturating_sub(label.len() + count_room);
            // Keep the query's insertion point visible without splitting a
            // grapheme. Footer position readout yields to active search.
            let line =
                Rect::new(nx, fy, query_room.min(u16::MAX as usize) as u16, 1).intersection(area);
            let look = LineLook {
                text: ns,
                selection: t.selection(),
                ellipsis: fs.fg(t.text_muted),
                ellipsis_glyph: t.glyphs.ellipsis,
                underline: None,
                follow_caret: true,
            };
            let caret = self
                .find
                .as_mut()
                .map(|f| f.needle.draw_line(buf, line, &look));
            let editing = self.find.as_ref().is_some_and(|f| f.editing);
            if editing {
                ctx.cursor = None;
                if focused
                    && query_room > 0
                    && let Some(caret) = caret
                {
                    ctx.set_cursor(caret);
                }
            }
            if count_room > 0 {
                put(
                    buf,
                    area.right() - count_width as u16,
                    fy,
                    &count,
                    fs.fg(t.text_muted),
                );
            }
            return;
        }
        let pos = if s.editing || focused {
            format!(
                "ln {}/{}{}col {}",
                cur.line + 1,
                line_count,
                t.glyphs.separator,
                cur.col + 1
            )
        } else if self.scroll.overflows() {
            scrollbar::position_label(&self.scroll)
        } else {
            String::new()
        };
        let right_padding = u16::from(area.width > 1);
        let pos = crate::ui::text::truncate(&pos, (area.width - right_padding) as usize);
        let px = area
            .right()
            .saturating_sub(width(&pos) as u16 + right_padding)
            .max(area.x);
        if let Some(d) = diags.iter().min_by_key(|d| d.range.start.abs_diff(cur_off)) {
            let c = if d.severity == Severity::Error {
                t.error
            } else {
                t.warning
            };
            // the diagnostic yields to the position readout on its right
            let room = px.saturating_sub(area.x + 2) as usize;
            put(
                buf,
                area.x + 1,
                fy,
                &crate::ui::text::truncate(&d.message, room),
                fs.fg(c),
            );
        }
        if !pos.is_empty() {
            put(buf, px, fy, &pos, fs.fg(t.text_faint));
        }
    }

    fn bracket_pair(&self) -> Option<(usize, usize)> {
        let text = self.field.buffer.text().as_bytes();
        let cur = self.field.buffer.cursor_offset();
        let at = |i: usize| text.get(i).copied();
        let probe = [cur, cur.wrapping_sub(1)];
        for &p in &probe {
            let Some(c) = at(p) else { continue };
            let (open, close, forward) = match c {
                b'(' => (b'(', b')', true),
                b')' => (b'(', b')', false),
                b'[' => (b'[', b']', true),
                b']' => (b'[', b']', false),
                _ => continue,
            };
            let mut depth = 0i32;
            if forward {
                for (i, &b) in text.iter().enumerate().skip(p) {
                    if b == open {
                        depth += 1;
                    } else if b == close {
                        depth -= 1;
                        if depth == 0 {
                            return Some((p, i));
                        }
                    }
                }
            } else {
                for i in (0..=p).rev() {
                    let b = text[i];
                    if b == close {
                        depth += 1;
                    } else if b == open {
                        depth -= 1;
                        if depth == 0 {
                            return Some((i, p));
                        }
                    }
                }
            }
        }
        None
    }
}

impl Scrollable for CodeEditor {
    /// The pane, its horizontal offset brought in step with
    /// [`CodeEditor::hscroll`] (the caret moves that one).
    fn pane(&mut self) -> &mut ScrollPane {
        self.scroll.h.offset = self.hscroll;
        &mut self.scroll
    }

    fn after_scroll(&mut self, axis: Axis, _moved: bool) -> bool {
        if axis == Axis::Horizontal {
            self.hscroll = self.scroll.h.offset;
        }
        false
    }
}

/// The editor's text, for the pointer ([`PointerField`] through
/// [`Editable`]): a press anchors a drag at the pointer (the second press
/// of a double click selects the word), a drag selects from the press and
/// scrolls past an edge, the release keeps the selection, a completed
/// click places the caret (2: the word, 3: the line) and enters editing
/// unless the document is read-only (M2).
impl Editable for CodeEditor {
    fn field(&mut self) -> &mut EditField {
        &mut self.field
    }
    fn edit_id(&self) -> WidgetId {
        self.id
    }
    fn offset_at(&self, pos: Position) -> usize {
        self.offset_at_pos(pos)
    }
    fn pointer_begin(&mut self) {
        self.begin_edit();
    }
    fn pointer_after(&mut self) {
        self.after_change();
    }
}

/// The editor's pointer gestures, as the apps route them: a wheel or a
/// bar press scrolls it; the text takes press, drag and multi-click
/// through [`PointerField`] (the [`Editable`] impl above).
impl crate::core::pointer::MouseTarget for CodeEditor {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::intent::Phase;
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r;
        }
        match ev.phase() {
            Some(Phase::Press) => PointerField::on_press_count(self, ev.pos(), ev.count()).into(),
            Some(Phase::Click) => PointerField::on_click_count(self, ev.pos(), ev.count()).into(),
            Some(Phase::Drag) => PointerField::on_drag(self, ev.pos()).into(),
            _ => crate::core::response::Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn scrollbar_drag_reaches_the_end_of_the_drawn_track() {
        let text: Vec<String> = (1..=40).map(|i| format!("line {i}")).collect();
        let mut editor = CodeEditor::new(WidgetId::of("c"), &text.join("\n"));
        render(&mut editor, 40, 6);
        assert!(editor.scroll.overflows());
        let track = editor.scroll.v_track();
        assert_eq!(track.height, 5, "the track is the body, not the status row");
        assert_eq!(
            editor
                .bar_drag(Position::new(track.x, track.bottom() - 1))
                .outcome(),
            Outcome::Changed
        );
        assert_eq!(editor.scroll.offset, editor.scroll.max_offset());
        assert_eq!(
            editor.bar_drag(Position::new(track.x, track.y)).outcome(),
            Outcome::Changed
        );
        assert_eq!(editor.scroll.offset, 0);
        assert_eq!(
            editor.wheel(Axis::Vertical, -1).outcome(),
            Outcome::Consumed
        );
    }

    /// Long lines stop a blank column short of the scrollbar instead of
    /// running into its track (layout audit #14).
    #[test]
    fn long_lines_keep_the_gap_before_the_scrollbar() {
        let text: Vec<String> = (1..=40)
            .map(|i| format!("{i} {}", "x".repeat(60)))
            .collect();
        let mut editor = CodeEditor::new(WidgetId::of("c"), &text.join("\n"));
        render(&mut editor, 40, 6);
        let track = editor.scroll.v_track();
        assert_eq!(track.x, 39);
        assert_eq!(
            editor.text_area.right(),
            38,
            "one blank column before the bar"
        );
        let tight = CodeEditor::new(WidgetId::of("c"), &text.join("\n")).scrollbar_gap(0);
        let mut tight = tight;
        render(&mut tight, 40, 6);
        assert_eq!(tight.text_area.right(), 39);
    }

    fn render(editor: &mut CodeEditor, width: u16, height: u16) -> Option<Position> {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(editor.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        editor.render(area, &mut buf, &mut ctx, theme.surface);
        ctx.cursor
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    /// K17 (S11): F6 leaves the editor while editing (Tab indents there)
    /// and while browsing, Shift+F6 backward; the document is kept.
    #[test]
    fn f6_leaves_the_editor_from_editing_and_browsing() {
        let mut e = CodeEditor::new(WidgetId::of("c"), "SELECT 1;");
        e.begin_edit();
        e.on_key(&key(KeyCode::Char('x')));
        assert_eq!(
            e.on_key(&key(KeyCode::F(6))).1,
            Some(EditorAction::Leave { backward: false })
        );
        assert!(!e.editing);
        assert_eq!(e.text(), "xSELECT 1;", "the document is kept");
        let back = Key {
            code: KeyCode::F(6),
            mods: KeyModifiers::SHIFT,
        };
        assert_eq!(
            e.on_key(&back).1,
            Some(EditorAction::Leave { backward: true }),
            "from browsing too"
        );
    }

    /// M2 (S11): the drag anchor is set by the press, through the router.
    /// A drag selects from where the button went down (it started at the
    /// previous click), the release away from the press keeps the
    /// selection (the click's `jump_to` cleared it), a double click
    /// selects the word, a triple click the line, and a drag above the
    /// text runs the selection on to the start (the offset was unsigned).
    /// A focused read-only document shows its caret (M39).
    #[test]
    fn a_drag_selects_from_the_press_and_multi_clicks_select_words_and_lines() {
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let text = "SELECT id FROM users;\nSELECT name FROM orders;\nSELECT 1;";
        let mut editor = CodeEditor::new(WidgetId::of("c"), text);
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let area = Rect::new(0, 0, 50, 6);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        {
            let mut buf = Buffer::empty(area);
            let mut ctx = RenderCtx::new(
                &theme,
                Interaction {
                    focus: Some(editor.id),
                    ..Default::default()
                },
                &mut hits,
                &mut ring,
            );
            editor.render(area, &mut buf, &mut ctx, theme.surface);
        }
        let (ta, id) = (editor.text_area, editor.id);
        let ev = |phase, x: u16, y: u16, count| {
            let pos = Position::new(x, y);
            PointerEvent::new(&hits, phase, id, pos, count, KeyModifiers::NONE)
        };
        let sel = |e: &CodeEditor| e.field.buffer.selected_text().map(str::to_owned);
        // an earlier click elsewhere must not become the drag's anchor
        let _ = editor.pointer(&ev(Phase::Press, ta.x + 2, ta.y + 2, 1));
        let _ = editor.pointer(&ev(Phase::Click, ta.x + 2, ta.y + 2, 1));
        let _ = editor.pointer(&ev(Phase::Press, ta.x + 7, ta.y + 1, 1));
        let _ = editor.pointer(&ev(Phase::Drag, ta.x + 11, ta.y + 1, 1));
        assert_eq!(sel(&editor).as_deref(), Some("name"), "from the press");
        let _ = editor.pointer(&ev(Phase::Release, ta.x + 11, ta.y + 1, 1));
        assert_eq!(sel(&editor).as_deref(), Some("name"), "kept on release");
        // a double click selects the word, a triple click the line
        let _ = editor.pointer(&ev(Phase::Press, ta.x + 14, ta.y + 1, 2));
        let _ = editor.pointer(&ev(Phase::Click, ta.x + 14, ta.y + 1, 2));
        assert_eq!(sel(&editor).as_deref(), Some("FROM"));
        let _ = editor.pointer(&ev(Phase::Press, ta.x + 14, ta.y + 1, 3));
        let _ = editor.pointer(&ev(Phase::Click, ta.x + 14, ta.y + 1, 3));
        assert!(sel(&editor).is_some_and(|s| s.starts_with("SELECT name FROM orders;")));
        // a drag above the text runs the selection on to its start
        let _ = editor.pointer(&ev(Phase::Press, ta.x + 6, ta.y + 1, 1));
        let _ = editor.pointer(&ev(Phase::Drag, ta.x, ta.y.saturating_sub(1), 1));
        assert_eq!(editor.field.buffer.selection(), Some(0..28));
        // a focused read-only document shows its caret where a click put it
        let mut ro = CodeEditor::new(WidgetId::of("r"), text).read_only(true);
        render(&mut ro, 50, 6);
        ro.on_click(Position::new(ro.text_area.x + 3, ro.text_area.y + 1));
        let caret = render(&mut ro, 50, 6).expect("the caret shows");
        assert_eq!(caret, Position::new(ro.text_area.x + 3, ro.text_area.y + 1));
    }

    /// Keyboard audit #5, #16 and #17: focus starts the edit (caret
    /// shown), Tab indents while editing and the hint says so, Esc is
    /// "Done" and keeps the text, then Tab leaves through the ring; with
    /// `tab_leaves` Tab leaves at once; Ctrl+Tab always leaves; the find
    /// bar counts as text input; `esc_reverts` restores the document.
    /// Ctrl+C with nothing selected is not the editor's while it edits,
    /// under every field-key preset, so the app's quit guard sees it (the
    /// key router hands an editing field the key first).
    #[test]
    fn ctrl_c_without_a_selection_passes_on_while_editing() {
        let mut e = CodeEditor::new(WidgetId::of("c"), "");
        render(&mut e, 40, 6);
        assert!(e.editing);
        e.on_key(&key(KeyCode::Char('q')));
        let ctrl_c = Key {
            code: KeyCode::Char('c'),
            mods: KeyModifiers::CONTROL,
        };
        assert_eq!(e.on_key(&ctrl_c).0, Outcome::Ignored);
        assert_eq!(e.text(), "q");
    }

    #[test]
    fn immediate_entry_tab_indents_esc_is_done_and_then_tab_leaves() {
        let mut e = CodeEditor::new(WidgetId::of("c"), "");
        assert!(render(&mut e, 40, 6).is_some(), "the caret shows on focus");
        assert!(e.editing && e.wants_text_input());
        e.on_key(&key(KeyCode::Char('q')));
        assert_eq!(e.text(), "q", "letters edit at once");
        let (_, ev) = e.on_key(&key(KeyCode::Tab));
        assert_eq!(ev, Some(EditorAction::Changed), "Tab indents");
        assert_eq!(e.text(), "q  ");
        let keys: Vec<&str> = e.hints().iter().map(|h| h.key).collect();
        assert_eq!(keys, ["Tab", "Esc"], "Tab Indent · Esc Done");
        assert_eq!(
            e.on_key(&key(KeyCode::Esc)).1,
            Some(EditorAction::Committed)
        );
        assert!(!e.editing, "Esc is Done");
        assert_eq!(e.text(), "q  ", "and keeps the text");
        assert_eq!(
            e.on_key(&key(KeyCode::Tab)).0,
            Outcome::Ignored,
            "Tab leaves"
        );
        assert!(!e.wants_text_input());
        e.on_key(&key(KeyCode::Char('/')));
        assert!(e.wants_text_input(), "the find bar takes typed keys");

        let mut leaves = CodeEditor::new(WidgetId::of("l"), "x").tab_leaves(true);
        render(&mut leaves, 40, 6);
        assert_eq!(
            leaves.on_key(&key(KeyCode::Tab)).1,
            Some(EditorAction::Leave { backward: false })
        );
        let mut ctrl = CodeEditor::new(WidgetId::of("t"), "x");
        render(&mut ctrl, 40, 6);
        let ctrl_tab = Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::CONTROL,
        };
        assert_eq!(
            ctrl.on_key(&ctrl_tab).1,
            Some(EditorAction::Leave { backward: false }),
            "Ctrl+Tab leaves where the terminal delivers it"
        );
        let mut revert = CodeEditor::new(WidgetId::of("r"), "keep").esc_reverts(true);
        render(&mut revert, 40, 6);
        revert.on_key(&key(KeyCode::Char('x')));
        revert.on_key(&key(KeyCode::Esc));
        assert_eq!(revert.text(), "keep", "esc_reverts restores the document");
        let undo = Key {
            code: KeyCode::Char('z'),
            mods: KeyModifiers::CONTROL,
        };
        revert.begin_edit();
        revert.on_key(&undo);
        assert_eq!(revert.text(), "xkeep", "and the revert is undoable");
        let mut explicit = CodeEditor::new(WidgetId::of("x"), "a").edit_entry(EditEntry::Explicit);
        render(&mut explicit, 40, 6);
        assert!(!explicit.editing, "Explicit waits for Enter or i");
    }

    #[test]
    fn the_block_marker_beside_the_focus_bar_is_not_accent() {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        fn whole(text: &str) -> Vec<std::ops::Range<usize>> {
            std::iter::once(0..text.len()).collect()
        }
        let mut editor = CodeEditor::new(WidgetId::of("c"), "select 1;").segmenter(whole);
        let area = Rect::new(0, 0, 40, 6);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let inter = Interaction {
            focus: Some(editor.id),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
        editor.render(area, &mut buf, &mut ctx, theme.surface);
        let marker = buf
            .content()
            .iter()
            .find(|c| c.symbol() == "›")
            .expect("block marker");
        assert_eq!(marker.fg, theme.text_primary);
    }

    #[test]
    fn modified_enter_commits_without_changing_document() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "hello");
        editor.begin_edit();
        assert_eq!(
            editor.on_key(&Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::CONTROL
            }),
            (Outcome::Changed, Some(EditorAction::Committed))
        );
        assert!(!editor.editing);
        assert_eq!(editor.text(), "hello");
    }

    #[test]
    fn find_uses_original_grapheme_ranges_after_case_expansion() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "İé é\u{301}");
        editor.open_find();
        editor.on_key(&key(KeyCode::Char('é')));
        assert_eq!(editor.find.as_ref().unwrap().matches, vec![2..4, 5..9]);
        assert_eq!(editor.cursor_offset(), 2);
    }

    #[test]
    fn paste_and_backspace_edit_find_query_in_each_document_mode() {
        for (editing, read_only) in [(false, false), (true, false), (false, true)] {
            let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "a\u{301}");
            editor.editing = editing;
            editor.read_only = read_only;
            editor.open_find();
            assert_eq!(editor.on_paste("a\u{301}"), Outcome::Changed);
            assert_eq!(editor.text(), "a\u{301}");
            assert_eq!(editor.find.as_ref().unwrap().needle.text(), "a\u{301}");
            editor.on_key(&key(KeyCode::Backspace));
            assert_eq!(editor.find.as_ref().unwrap().needle.text(), "");
        }
    }

    #[test]
    fn replacing_document_refreshes_existing_find_matches() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "abc");
        editor.open_find();
        editor.on_paste("a");
        editor.set_text("xyz");
        assert!(editor.find.as_ref().unwrap().matches.is_empty());
    }

    #[test]
    fn manual_scroll_survives_render_then_typing_reveals_cursor() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"line\n".repeat(30));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.wheel(Axis::Vertical, 3).outcome();
        let offset = editor.scroll.offset;
        assert!(offset > 0);
        assert_eq!(render(&mut editor, 40, 8), None);
        assert_eq!(editor.scroll.offset, offset);
        assert_eq!(editor.cursor_cell(), None);
        editor.on_key(&key(KeyCode::Char('x')));
        assert!(render(&mut editor, 40, 8).is_some());
        assert_eq!(editor.scroll.offset, 0);
    }

    fn draw_buf(editor: &mut CodeEditor, width: u16, height: u16) -> (Buffer, HitRegistry) {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            editor.render(area, &mut buf, &mut ctx, theme.surface);
        }
        (buf, hits)
    }

    #[test]
    fn a_wide_document_shows_a_horizontal_bar_and_stops_at_its_widest_line() {
        let text = format!("SELECT {} FROM t;\nshort", "c, ".repeat(30));
        let mut editor = CodeEditor::new(WidgetId::of("test.wide"), &text).read_only(true);
        let (buf, hits) = draw_buf(&mut editor, 40, 8);
        // body rows 0..=6, footer on 7: the bar takes the last body row
        let y = 6;
        let thumb = |buf: &Buffer| (0..40).find(|&x| buf[(x, y)].symbol() == scrollbar::H_THUMB);
        assert!(thumb(&buf).is_some(), "the bar is drawn above the footer");
        assert_eq!(
            hits.hit(Position::new(editor.scroll.h_track().x + 1, y)),
            Some(scrollbar::h_id_for(editor.id))
        );
        let start = thumb(&buf).unwrap();

        assert_eq!(
            editor.wheel(Axis::Horizontal, 1).outcome(),
            Outcome::Changed
        );
        assert_eq!(editor.hscroll, HScroll::default().step);
        let (buf, _) = draw_buf(&mut editor, 40, 8);
        assert!(thumb(&buf).unwrap() > start, "the thumb follows");

        // → runs no further than the widest line (it used to run on)
        for _ in 0..40 {
            editor.on_key(&key(KeyCode::Right));
        }
        draw_buf(&mut editor, 40, 8);
        assert_eq!(editor.hscroll, editor.hbar().max_offset());

        // the bar drags back to the start
        let (buf, _) = draw_buf(&mut editor, 40, 8);
        let grab = thumb(&buf).unwrap();
        assert_eq!(
            editor
                .bar_press(Position::new(grab, y))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Consumed
        );
        assert_eq!(
            editor.bar_drag(Position::new(0, y)).outcome(),
            Outcome::Changed
        );
        assert_eq!(editor.hscroll, 0);
        assert_eq!(editor.scroll.offset, 0);

        let mut off = CodeEditor::new(WidgetId::of("test.off"), &text).horizontal(HScroll::off());
        let (buf, _) = draw_buf(&mut off, 40, 8);
        assert!(thumb(&buf).is_none());
        assert_eq!(off.wheel(Axis::Horizontal, 1).outcome(), Outcome::Ignored);
    }

    #[test]
    fn horizontal_scroll_hides_offscreen_cursor() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"x".repeat(100));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.wheel(Axis::Horizontal, 3).outcome();
        assert_eq!(render(&mut editor, 40, 8), None);
        assert_eq!(editor.cursor_cell(), None);
    }

    #[test]
    fn focused_read_only_editor_keeps_monochrome_navigation_gutter() {
        let theme = Theme::for_level(ColorLevel::Mono);
        let area = Rect::new(0, 0, 40, 8);
        let mut buf = Buffer::empty(area);
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "hello").read_only(true);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(editor.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        editor.render(area, &mut buf, &mut ctx, theme.surface);
        assert_eq!(buf[(0, 0)].symbol(), "▎");
    }

    #[test]
    fn switching_to_read_only_disables_all_document_edit_actions() {
        for code in [KeyCode::Tab, KeyCode::BackTab, KeyCode::Char('x')] {
            let mut editor = CodeEditor::new(WidgetId::of("test.editor"), "  text");
            editor.begin_edit();
            editor.read_only = true;
            editor.on_key(&key(code));
            assert_eq!(editor.text(), "  text");
            assert!(!editor.editing);
        }
    }

    #[test]
    fn smaller_viewport_reveals_cursor_after_manual_scroll() {
        let mut editor = CodeEditor::new(WidgetId::of("test.editor"), &"line\n".repeat(30));
        editor.begin_edit();
        render(&mut editor, 40, 8);
        editor.wheel(Axis::Vertical, 3).outcome();
        render(&mut editor, 40, 8);
        assert!(editor.scroll.offset > 0);
        assert!(render(&mut editor, 20, 4).is_some());
        assert_eq!(editor.scroll.offset, 0);
    }

    #[test]
    fn narrow_editor_and_long_find_stay_inside_nonzero_area() {
        for width in [1, 3, 7, 16, 40] {
            for find in [false, true] {
                let theme = Theme::for_level(ColorLevel::Mono);
                let outer = Rect::new(0, 0, 60, 10);
                let area = Rect::new(8, 2, width, 5);
                let mut buf = Buffer::empty(outer);
                for cell in &mut buf.content {
                    cell.set_symbol("·");
                }
                let mut editor = CodeEditor::new(
                    WidgetId::of("test.editor"),
                    "界🙂 abcdefghijklmnopqrstuvwxyz",
                );
                editor.begin_edit();
                if find {
                    editor.open_find();
                    editor.on_paste(&"界🙂a\u{301}".repeat(20));
                }
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(
                    &theme,
                    Interaction {
                        focus: Some(editor.id),
                        ..Default::default()
                    },
                    &mut hits,
                    &mut ring,
                );
                editor.render(area, &mut buf, &mut ctx, theme.surface);
                if let Some(cursor) = ctx.cursor {
                    assert!(area.contains(cursor), "cursor {cursor:?}, area {area:?}");
                }
                for y in 0..outer.height {
                    for x in 0..outer.width {
                        if !area.contains(Position::new(x, y)) {
                            assert_eq!(
                                buf[(x, y)].symbol(),
                                "·",
                                "spill at {x},{y}, area {area:?}, find {find}"
                            );
                        }
                    }
                }
            }
        }
    }
}
