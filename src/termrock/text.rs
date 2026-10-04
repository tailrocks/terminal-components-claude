//! Termrock shared text operations, text source, and grapheme-safe editor core.
//!
//! Provides Unicode-width calculation, horizontal cell slicing, truncation,
//! fuzzy ranking, and the shared `TextEditorCore` buffer.

use std::borrow::Cow;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::termrock::identity::{ItemKey, Revision};
use crate::termrock::theme::{StylePatch, Tone};

/// Borrowed revisioned text source trait.
pub trait TextSource {
    fn revision(&self) -> Revision;
    fn line_count(&self) -> usize;
    fn line(&self, index: usize) -> Option<TextLine<'_>>;
}

/// A single borrowed line from a TextSource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLine<'a> {
    pub key: ItemKey,
    pub text: &'a str,
    pub spans: &'a [StyleSpan],
}

impl<'a> TextLine<'a> {
    pub const fn new(key: ItemKey, text: &'a str, spans: &'a [StyleSpan]) -> Self {
        Self { key, text, spans }
    }

    pub const fn plain(key: ItemKey, text: &'a str) -> Self {
        Self {
            key,
            text,
            spans: &[],
        }
    }
}

/// Logical text position identified by stable line key and byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TextPosition {
    pub line: ItemKey,
    pub byte: usize,
}

impl TextPosition {
    pub const fn new(line: ItemKey, byte: usize) -> Self {
        Self { line, byte }
    }
}

/// Revision-anchored source range across stable lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRange {
    pub revision: Revision,
    pub start: TextPosition,
    pub end: TextPosition,
}

impl TextRange {
    pub const fn new(revision: Revision, start: TextPosition, end: TextPosition) -> Self {
        Self {
            revision,
            start,
            end,
        }
    }
}

/// Styled span within a logical text line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSpan {
    pub start: usize,
    pub end: usize,
    pub tone: Tone,
    pub patch: Option<StylePatch>,
}

impl StyleSpan {
    pub const fn new(start: usize, end: usize, tone: Tone) -> Self {
        Self {
            start,
            end,
            tone,
            patch: None,
        }
    }

    pub const fn with_patch(start: usize, end: usize, tone: Tone, patch: StylePatch) -> Self {
        Self {
            start,
            end,
            tone,
            patch: Some(patch),
        }
    }
}

/// Result of fuzzy matching query against text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub score: u32,
    pub ranges: Vec<Range<usize>>,
    pub matched: bool,
}

impl MatchResult {
    pub const fn none() -> Self {
        Self {
            score: u32::MAX,
            ranges: Vec::new(),
            matched: false,
        }
    }

    pub fn is_match(&self) -> bool {
        self.matched
    }

    pub fn byte_offsets(&self) -> Vec<usize> {
        self.ranges.iter().map(|r| r.start).collect()
    }
}

/// Returns the display column width of a string in terminal cells.
pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Truncate text to at most `columns` display cells, appending `…` when cut.
pub fn truncate(text: &str, columns: usize) -> Cow<'_, str> {
    if width(text) <= columns {
        return Cow::Borrowed(text);
    }
    if columns == 0 {
        return Cow::Borrowed("");
    }
    let mut out = String::new();
    let mut w = 0;
    for g in text.graphemes(true) {
        let gw = width(g);
        if w + gw > columns.saturating_sub(1) {
            break;
        }
        out.push_str(g);
        w += gw;
    }
    out.push('…');
    Cow::Owned(out)
}

/// Show a horizontal window addressed in display cells, never splitting a grapheme.
///
/// Preserves baseline behavior from `crate::ui::text::slice_cells`.
pub fn slice_cells(text: &str, start_col: usize, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    let clipped_right = width(text) > start_col.saturating_add(max_cols);
    let visible = max_cols - usize::from(clipped_right);
    if visible == 0 {
        return "…".to_owned();
    }
    let end = start_col.saturating_add(visible);
    let mut out = String::new();
    let mut col = 0;
    let mut out_width = 0;
    for g in text.graphemes(true) {
        let start = col;
        col += width(g);
        if col <= start_col {
            continue;
        }
        if start >= end {
            break;
        }
        if start <= start_col && start_col > 0 {
            let cells = col.min(end) - start_col;
            out.push('…');
            out.push_str(&" ".repeat(cells - 1));
            out_width += cells;
        } else if col <= end {
            out.push_str(g);
            out_width += col - start;
        } else {
            break;
        }
    }
    if clipped_right {
        out.push_str(&" ".repeat(visible.saturating_sub(out_width)));
        out.push('…');
    }
    out
}

/// Search text helper retaining source grapheme boundaries.
struct SearchText {
    text: String,
    origins: Vec<(Range<usize>, Range<usize>)>,
}

impl SearchText {
    fn new(source: &str, lowercase: bool) -> Self {
        let text = if lowercase {
            source.to_lowercase()
        } else {
            source.to_owned()
        };
        let mut chars = text.char_indices().peekable();
        let mut origins = Vec::new();
        for (start, grapheme) in source.grapheme_indices(true) {
            let mapped_start = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            let count = if lowercase {
                grapheme.chars().map(|c| c.to_lowercase().count()).sum()
            } else {
                grapheme.chars().count()
            };
            for _ in 0..count {
                chars.next();
            }
            let mapped_end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            origins.push((mapped_start..mapped_end, start..start + grapheme.len()));
        }
        Self { text, origins }
    }

    fn original_range(&self, range: Range<usize>) -> Range<usize> {
        let first = self
            .origins
            .partition_point(|(mapped, _)| mapped.end <= range.start);
        let end = self
            .origins
            .partition_point(|(mapped, _)| mapped.start < range.end);
        if first >= self.origins.len() || end == 0 || first >= end {
            return range;
        }
        self.origins[first].1.start..self.origins[end - 1].1.end
    }

    fn matched_ranges(&self, range: Range<usize>) -> Vec<Range<usize>> {
        self.origins
            .iter()
            .filter(|(mapped, _)| mapped.start < range.end && range.start < mapped.end)
            .map(|(_, original)| original.clone())
            .collect()
    }
}

/// Fuzzy match `query` against `text`: prefix wins, then boundary substring, then subsequence.
/// Returns score (lower is better penalty) and matched original grapheme byte ranges.
pub fn fuzzy(query: &str, text: &str) -> MatchResult {
    if query.is_empty() {
        return MatchResult {
            score: 0,
            ranges: Vec::new(),
            matched: true,
        };
    }
    let hay = SearchText::new(text, true);
    let q = query.to_lowercase();
    if hay.text.starts_with(&q) {
        return MatchResult {
            score: 0,
            ranges: hay.matched_ranges(0..q.len()),
            matched: true,
        };
    }
    if let Some(p) = hay.text.find(&q) {
        let boundary = p == 0 || matches!(hay.text.as_bytes()[p - 1], b'_' | b'.' | b'-' | b' ');
        return MatchResult {
            score: if boundary { 10 } else { 30 },
            ranges: hay.matched_ranges(p..p + q.len()),
            matched: true,
        };
    }
    let mut matched = Vec::new();
    let mut chars = hay.text.char_indices();
    for qc in q.chars() {
        let (offset, ch) = match chars.find(|(_, ch)| *ch == qc) {
            Some(pair) => pair,
            None => return MatchResult::none(),
        };
        let original = hay.original_range(offset..offset + ch.len_utf8());
        if matched.last() != Some(&original) {
            matched.push(original);
        }
    }
    let last_offset = matched.last().map(|r| r.start as u32).unwrap_or(0);
    MatchResult {
        score: 60 + last_offset,
        ranges: matched,
        matched: true,
    }
}

/// Commands for editing and navigating in `TextEditorCore`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextCommand {
    InsertChar(char),
    InsertStr(String),
    DeleteBackward,
    DeleteForward,
    DeleteWordLeft,
    DeleteToLineStart,
    DeleteToLineEnd,
    MoveLeft { select: bool },
    MoveRight { select: bool },
    MoveHome { select: bool },
    MoveEnd { select: bool },
    MoveWordLeft { select: bool },
    MoveWordRight { select: bool },
    SelectAll,
    ClearSelection,
    Undo,
    Redo,
}

/// Outcome of applying a `TextCommand` to `TextEditorCore`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditOutcome {
    pub modified: bool,
    pub selection_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextSnapshot {
    text: String,
    caret: usize,
    anchor: Option<usize>,
}

/// Grapheme-safe text editing buffer with undo/redo history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditorCore {
    text: String,
    caret: usize,
    anchor: Option<usize>,
    undo_stack: Vec<TextSnapshot>,
    redo_stack: Vec<TextSnapshot>,
    max_history: usize,
    multiline: bool,
}

impl Default for TextEditorCore {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditorCore {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            caret: 0,
            anchor: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 128,
            multiline: false,
        }
    }

    pub fn with_multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn set_caret(&mut self, pos: usize) {
        self.caret = Self::ceil_boundary(&self.text, pos.min(self.text.len()));
        self.anchor = None;
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        let a = self.anchor?;
        if a == self.caret {
            None
        } else {
            Some(a.min(self.caret)..a.max(self.caret))
        }
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selection().map(|r| &self.text[r])
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.push_undo();
        self.text = text.into();
        self.caret = self.text.len();
        self.anchor = None;
        self.normalize_positions();
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(TextSnapshot {
            text: self.text.clone(),
            caret: self.caret,
            anchor: self.anchor,
        });
        self.redo_stack.clear();
    }

    fn normalize_positions(&mut self) {
        self.caret = Self::ceil_boundary(&self.text, self.caret);
        self.anchor = self.anchor.map(|a| Self::floor_boundary(&self.text, a));
    }

    fn floor_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i <= offset)
            .last()
            .unwrap_or(0)
    }

    fn ceil_boundary(text: &str, offset: usize) -> usize {
        if offset >= text.len() {
            return text.len();
        }
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|i| *i >= offset)
            .unwrap_or(text.len())
    }

    fn prev_boundary(&self, from: usize) -> usize {
        self.text[..from]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, from: usize) -> usize {
        self.text[from..]
            .graphemes(true)
            .next()
            .map(|g| from + g.len())
            .unwrap_or(from)
    }

    fn line_start(&self, from: usize) -> usize {
        self.text[..from].rfind('\n').map(|i| i + 1).unwrap_or(0)
    }

    fn line_end(&self, from: usize) -> usize {
        self.text[from..]
            .find('\n')
            .map(|i| from + i)
            .unwrap_or(self.text.len())
    }

    fn prev_word(&self, from: usize) -> usize {
        let mut graphemes = self.text[..from].grapheme_indices(true).rev().peekable();
        while graphemes
            .peek()
            .is_some_and(|(_, g)| !g.chars().any(char::is_alphanumeric))
        {
            graphemes.next();
        }
        let mut start = 0;
        while let Some(&(i, g)) = graphemes.peek() {
            if !g.chars().any(char::is_alphanumeric) {
                break;
            }
            start = i;
            graphemes.next();
        }
        start
    }

    fn next_word(&self, from: usize) -> usize {
        let after = &self.text[from..];
        let mut it = after.grapheme_indices(true);
        let mut i = 0;
        while let Some((idx, g)) = it.next() {
            i = idx + g.len();
            if !g.chars().any(char::is_alphanumeric) {
                continue;
            }
            for (idx2, g2) in it.by_ref() {
                if !g2.chars().any(char::is_alphanumeric) {
                    i = idx2;
                    return from + i;
                }
                i = idx2 + g2.len();
            }
            break;
        }
        from + i
    }

    fn begin_move(&mut self, select: bool) {
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.caret);
            }
        } else {
            self.anchor = None;
        }
    }

    fn delete_selection_internal(&mut self) -> bool {
        if let Some(r) = self.selection() {
            self.push_undo();
            self.text.replace_range(r.clone(), "");
            self.caret = r.start;
            self.anchor = None;
            self.normalize_positions();
            true
        } else {
            false
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if matches!(c, '\r' | '\n') && !self.multiline {
            return;
        }
        let c = if c == '\r' { '\n' } else { c };
        let mut buf = [0u8; 4];
        self.insert_str(c.encode_utf8(&mut buf));
    }

    pub fn insert_str(&mut self, s: &str) {
        self.push_undo();
        if let Some(r) = self.selection() {
            self.caret = r.start + s.len();
            self.text.replace_range(r, s);
        } else {
            self.text.insert_str(self.caret, s);
            self.caret += s.len();
        }
        self.anchor = None;
        self.normalize_positions();
    }

    pub fn delete_backward(&mut self) -> bool {
        if self.delete_selection_internal() {
            return true;
        }
        if self.caret == 0 {
            return false;
        }
        self.push_undo();
        let start = self.prev_boundary(self.caret);
        self.text.replace_range(start..self.caret, "");
        self.caret = start;
        self.normalize_positions();
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        if self.delete_selection_internal() {
            return true;
        }
        if self.caret >= self.text.len() {
            return false;
        }
        self.push_undo();
        let end = self.next_boundary(self.caret);
        self.text.replace_range(self.caret..end, "");
        self.normalize_positions();
        true
    }

    pub fn delete_word_left(&mut self) -> bool {
        if self.delete_selection_internal() {
            return true;
        }
        if self.caret == 0 {
            return false;
        }
        self.push_undo();
        let start = self.prev_word(self.caret);
        self.text.replace_range(start..self.caret, "");
        self.caret = start;
        self.normalize_positions();
        true
    }

    pub fn delete_to_line_start(&mut self) -> bool {
        if self.delete_selection_internal() {
            return true;
        }
        let start = self.line_start(self.caret);
        if start == self.caret {
            return false;
        }
        self.push_undo();
        self.text.replace_range(start..self.caret, "");
        self.caret = start;
        self.normalize_positions();
        true
    }

    pub fn delete_to_line_end(&mut self) -> bool {
        if self.delete_selection_internal() {
            return true;
        }
        let end = self.line_end(self.caret);
        if end == self.caret {
            return false;
        }
        self.push_undo();
        self.text.replace_range(self.caret..end, "");
        self.normalize_positions();
        true
    }

    pub fn move_left(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.caret = r.start;
            return;
        }
        self.begin_move(select);
        self.caret = self.prev_boundary(self.caret);
    }

    pub fn move_right(&mut self, select: bool) {
        if !select && let Some(r) = self.selection() {
            self.anchor = None;
            self.caret = r.end;
            return;
        }
        self.begin_move(select);
        self.caret = self.next_boundary(self.caret);
    }

    pub fn move_home(&mut self, select: bool) {
        self.begin_move(select);
        self.caret = self.line_start(self.caret);
    }

    pub fn move_end(&mut self, select: bool) {
        self.begin_move(select);
        self.caret = self.line_end(self.caret);
    }

    pub fn move_word_left(&mut self, select: bool) {
        self.begin_move(select);
        self.caret = self.prev_word(self.caret);
    }

    pub fn move_word_right(&mut self, select: bool) {
        self.begin_move(select);
        self.caret = self.next_word(self.caret);
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(0);
        self.caret = self.text.len();
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(TextSnapshot {
                text: self.text.clone(),
                caret: self.caret,
                anchor: self.anchor,
            });
            self.text = snapshot.text;
            self.caret = snapshot.caret;
            self.anchor = snapshot.anchor;
            self.normalize_positions();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(TextSnapshot {
                text: self.text.clone(),
                caret: self.caret,
                anchor: self.anchor,
            });
            self.text = snapshot.text;
            self.caret = snapshot.caret;
            self.anchor = snapshot.anchor;
            self.normalize_positions();
            true
        } else {
            false
        }
    }

    /// Applies a text editing/navigation command and returns the outcome.
    pub fn apply(&mut self, cmd: TextCommand) -> EditOutcome {
        let prev_len = self.text.len();
        let prev_sel = self.selection();
        match cmd {
            TextCommand::InsertChar(c) => self.insert_char(c),
            TextCommand::InsertStr(s) => self.insert_str(&s),
            TextCommand::DeleteBackward => {
                self.delete_backward();
            }
            TextCommand::DeleteForward => {
                self.delete_forward();
            }
            TextCommand::DeleteWordLeft => {
                self.delete_word_left();
            }
            TextCommand::DeleteToLineStart => {
                self.delete_to_line_start();
            }
            TextCommand::DeleteToLineEnd => {
                self.delete_to_line_end();
            }
            TextCommand::MoveLeft { select } => self.move_left(select),
            TextCommand::MoveRight { select } => self.move_right(select),
            TextCommand::MoveHome { select } => self.move_home(select),
            TextCommand::MoveEnd { select } => self.move_end(select),
            TextCommand::MoveWordLeft { select } => self.move_word_left(select),
            TextCommand::MoveWordRight { select } => self.move_word_right(select),
            TextCommand::SelectAll => self.select_all(),
            TextCommand::ClearSelection => self.clear_selection(),
            TextCommand::Undo => {
                self.undo();
            }
            TextCommand::Redo => {
                self.redo();
            }
        }
        EditOutcome {
            modified: self.text.len() != prev_len,
            selection_changed: self.selection() != prev_sel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_width_and_truncation() {
        assert_eq!(width("hello"), 5);
        assert_eq!(width("日本"), 4);
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("日本", 3), "日…");
    }

    #[test]
    fn slice_cells_preserves_baseline_and_graphemes() {
        assert_eq!(slice_cells("ab日本cd", 3, 5), "…本cd");
        assert_eq!(slice_cells("ab日本cd", 2, 5), "… 本…");
        assert_eq!(slice_cells("日本語", 0, 5), "日本…");
        assert_eq!(slice_cells("abc", 3, 5), "");
        assert_eq!(slice_cells("日本", 1, 1), "…");
    }

    #[test]
    fn fuzzy_ranking_and_ranges() {
        let res = fuzzy("al", "alpha");
        assert!(res.is_match());
        assert_eq!(res.score, 0);
        assert_eq!(res.ranges, vec![0..1, 1..2]);

        let res2 = fuzzy("be", "a_beta");
        assert!(res2.is_match());
        assert_eq!(res2.score, 10);

        let res3 = fuzzy("xyz", "alpha");
        assert!(!res3.is_match());
        assert_eq!(res3.score, u32::MAX);
    }

    #[test]
    fn editor_core_editing_and_undo_redo() {
        let mut ed = TextEditorCore::new();
        ed.insert_str("hello");
        assert_eq!(ed.text(), "hello");
        assert_eq!(ed.caret(), 5);

        ed.insert_str(" world");
        assert_eq!(ed.text(), "hello world");

        assert!(ed.undo());
        assert_eq!(ed.text(), "hello");

        assert!(ed.redo());
        assert_eq!(ed.text(), "hello world");

        ed.move_word_left(false);
        assert_eq!(ed.caret(), 6);
        ed.move_word_right(true);
        assert_eq!(ed.selected_text(), Some("world"));

        ed.delete_backward();
        assert_eq!(ed.text(), "hello ");
    }

    #[test]
    fn editor_core_commands_and_selection() {
        let mut ed = TextEditorCore::new();
        ed.apply(TextCommand::InsertStr("abcdef".to_string()));
        assert_eq!(ed.text(), "abcdef");
        assert_eq!(ed.caret(), 6);

        ed.apply(TextCommand::MoveHome { select: false });
        assert_eq!(ed.caret(), 0);

        ed.apply(TextCommand::MoveRight { select: true });
        ed.apply(TextCommand::MoveRight { select: true });
        assert_eq!(ed.selected_text(), Some("ab"));

        ed.apply(TextCommand::DeleteForward);
        assert_eq!(ed.text(), "cdef");
        assert_eq!(ed.caret(), 0);

        ed.apply(TextCommand::MoveEnd { select: false });
        assert_eq!(ed.caret(), 4);

        ed.apply(TextCommand::SelectAll);
        assert_eq!(ed.selected_text(), Some("cdef"));

        ed.apply(TextCommand::ClearSelection);
        assert_eq!(ed.selection(), None);

        ed.apply(TextCommand::InsertChar('!'));
        assert_eq!(ed.text(), "cdef!");
    }

    #[test]
    fn text_source_and_range_types() {
        struct MockSource {
            rev: Revision,
            lines: Vec<String>,
        }

        impl TextSource for MockSource {
            fn revision(&self) -> Revision {
                self.rev
            }
            fn line_count(&self) -> usize {
                self.lines.len()
            }
            fn line(&self, index: usize) -> Option<TextLine<'_>> {
                self.lines
                    .get(index)
                    .map(|s| TextLine::plain(ItemKey::new(index as u64), s))
            }
        }

        let src = MockSource {
            rev: Revision::new(42),
            lines: vec!["first line".to_string(), "second line".to_string()],
        };

        assert_eq!(src.revision().as_u64(), 42);
        assert_eq!(src.line_count(), 2);
        let l0 = src.line(0).unwrap();
        assert_eq!(l0.key.as_u64(), 0);
        assert_eq!(l0.text, "first line");

        let pos1 = TextPosition::new(ItemKey::new(0), 0);
        let pos2 = TextPosition::new(ItemKey::new(0), 5);
        let range = TextRange::new(src.revision(), pos1, pos2);
        assert_eq!(range.revision.as_u64(), 42);
        assert_eq!(range.start.byte, 0);
        assert_eq!(range.end.byte, 5);

        let span = StyleSpan::new(0, 5, Tone::Primary);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.tone, Tone::Primary);
    }

    #[test]
    fn editor_core_grapheme_boundaries() {
        let mut ed = TextEditorCore::new();
        ed.insert_str("e\u{301}日👩‍💻");
        assert_eq!(ed.text(), "e\u{301}日👩‍💻");

        ed.move_left(false);
        // Should move past the entire 👩‍💻 cluster (7 bytes)
        assert_eq!(ed.caret(), "e\u{301}日".len());

        ed.delete_backward();
        assert_eq!(ed.text(), "e\u{301}👩‍💻");
    }
}
