//! Selectable read-only text viewport: styled lines with bounded retention,
//! tail-follow, optional wrapping, wheel/scrollbar scrolling, drag, word and
//! keyboard selection, copy, search marks, and an optional caret exposed as
//! the hardware cursor. Serves log bodies and simulated terminals alike; it
//! knows nothing about what the text means.
//!
//! Ownership contract:
//!
//! - Content is only reachable through the mutation methods (`set_lines`,
//!   `push`, `extend`, `replace_last`, `pop`, `clear`, `set_max_lines`).
//!   Every mutation bumps [`TextViewport::revision`] and marks exactly the
//!   changed logical lines for re-segmentation, so text, copy and cursor
//!   agree on the next frame while unchanged frames reuse their layout.
//! - The retention cap is enforced by one transaction at construction,
//!   replacement, append, tail replacement and limit change. Every line has
//!   a stable identity independent of its index; selection, drag anchor,
//!   producer caret and the retained reading position are reconciled through
//!   that identity, never by clamping an index.
//! - A logical line is segmented into graphemes as one string; span
//!   boundaries never split a cluster. A cluster that crosses two spans takes
//!   the style of the span containing its first byte. Tabs expand to four
//!   cells, other C0 controls render as `^X`, and copy returns the source
//!   bytes (tabs and controls included), never the display text.

use std::ops::Range;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::Tone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::width;
use crate::widgets::scrollbar;

/// One styled run. Tone maps through `Theme::tone`; bold/italic/underline
/// are the only modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Draw reversed (canvas on text-primary): a terminal's cursor block.
    pub reversed: bool,
}

impl Span {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
            italic: false,
            underline: false,
            reversed: false,
        }
    }
    pub fn plain(text: impl Into<String>) -> Self {
        Self::new(text, Tone::Normal)
    }
    pub fn muted(text: impl Into<String>) -> Self {
        Self::new(text, Tone::Muted)
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
}

pub type Line = Vec<Span>;

pub fn line_text(line: &[Span]) -> String {
    line.iter().map(|s| s.text.as_str()).collect()
}

/// Logical position: line index and display column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellPos {
    pub line: usize,
    pub col: usize,
}

/// A search or emphasis mark over a source byte range of a logical line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub line: usize,
    pub range: Range<usize>,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportEvent {
    /// `y` with a selection: the owner puts the text on its clipboard.
    Copy(String),
    SelectionChanged,
    FollowChanged(bool),
}

/// Layout work performed since construction, for performance proofs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkCounters {
    /// Logical lines segmented into cells.
    pub segmented_lines: usize,
    /// Logical lines whose wrapped rows were recomputed.
    pub reflowed_lines: usize,
    /// Visual row index rebuilds from scratch (dataset replacement, width or
    /// wrap change, removal from the tail).
    pub index_rebuilds: usize,
    /// Visual row index extensions: appended or replaced tail lines added
    /// their rows without touching retained ones.
    pub index_extends: usize,
    /// Visual row index shifts: leading rows dropped by retention, retained
    /// rows renumbered, nothing re-laid out.
    pub index_shifts: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellStyle {
    tone: Tone,
    bold: bool,
    italic: bool,
    underline: bool,
    reversed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Text,
    /// One of the cells a tab expands to.
    TabPad,
    /// A visible stand-in for a control byte (`^X`).
    Control,
}

#[derive(Debug, Clone)]
struct Cell {
    /// Display grapheme (or a stand-in).
    g: String,
    w: usize,
    /// Source byte range this cell stands for (shared by tab pads).
    src: Range<usize>,
    style: CellStyle,
    kind: CellKind,
}

/// Wrapped row boundaries (cell index ranges) at one `(width, wrap)` key.
type RowCache = ((u16, bool), Vec<(usize, usize)>);

#[derive(Debug, Clone)]
struct Logical {
    id: u64,
    spans: Line,
    /// Concatenated source text of the spans.
    text: String,
    cells: Vec<Cell>,
    /// Whether `cells` reflects `spans`.
    segmented: bool,
    /// Wrapped row boundaries (cell index ranges) per layout width; the
    /// scrollbar decision alternates between two widths, so both are kept.
    rows: Vec<RowCache>,
}

impl Logical {
    fn new(id: u64, spans: Line) -> Self {
        let text = line_text(&spans);
        Self {
            id,
            spans,
            text,
            cells: vec![],
            segmented: false,
            rows: vec![],
        }
    }

    fn set(&mut self, spans: Line) {
        self.text = line_text(&spans);
        self.spans = spans;
        self.segmented = false;
        self.rows.clear();
    }

    fn has_rows(&self, key: (u16, bool)) -> bool {
        self.rows.iter().any(|(k, _)| *k == key)
    }

    fn rows_at(&self, key: (u16, bool)) -> &[(usize, usize)] {
        self.rows
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, r)| r.as_slice())
            .unwrap_or(&[])
    }

    fn segment(&mut self) {
        let mut cells: Vec<Cell> = vec![];
        // span byte boundaries in the concatenated text
        let mut bounds: Vec<(usize, CellStyle)> = vec![];
        let mut at = 0;
        for sp in &self.spans {
            bounds.push((
                at,
                CellStyle {
                    tone: sp.tone,
                    bold: sp.bold,
                    italic: sp.italic,
                    underline: sp.underline,
                    reversed: sp.reversed,
                },
            ));
            at += sp.text.len();
        }
        let style_at = |b: usize| -> CellStyle {
            bounds
                .iter()
                .rev()
                .find(|(s, _)| *s <= b)
                .map(|(_, st)| *st)
                .unwrap_or(CellStyle {
                    tone: Tone::Normal,
                    bold: false,
                    italic: false,
                    underline: false,
                    reversed: false,
                })
        };
        for (start, g) in self.text.grapheme_indices(true) {
            let src = start..start + g.len();
            let style = style_at(start);
            if g == "\t" {
                for _ in 0..crate::ui::text::TAB_SPACES.len() {
                    cells.push(Cell {
                        g: " ".into(),
                        w: 1,
                        src: src.clone(),
                        style,
                        kind: CellKind::TabPad,
                    });
                }
                continue;
            }
            let first = g.chars().next().unwrap_or(' ');
            if g.chars().count() == 1 && (first.is_control() || first == '\u{7f}') {
                let shown = if first == '\u{7f}' {
                    "^?".to_owned()
                } else {
                    format!("^{}", ((first as u8) + 0x40) as char)
                };
                cells.push(Cell {
                    g: shown,
                    w: 2,
                    src,
                    style,
                    kind: CellKind::Control,
                });
                continue;
            }
            let w = width(g);
            if w == 0 {
                // a lone zero-width cluster attaches to the cell before it
                if let Some(prev) = cells.last_mut() {
                    prev.g.push_str(g);
                    prev.src.end = src.end;
                    continue;
                }
                cells.push(Cell {
                    g: g.to_owned(),
                    w: 1,
                    src,
                    style,
                    kind: CellKind::Text,
                });
                continue;
            }
            cells.push(Cell {
                g: g.to_owned(),
                w,
                src,
                style,
                kind: CellKind::Text,
            });
        }
        self.cells = cells;
        self.segmented = true;
        self.rows.clear();
    }

    fn reflow(&mut self, text_w: u16, wrap: bool) {
        let w = text_w.max(1) as usize;
        let mut rows = vec![];
        if !wrap || self.cells.is_empty() {
            rows.push((0, self.cells.len()));
        } else {
            let mut start = 0;
            let mut acc = 0;
            for (ci, c) in self.cells.iter().enumerate() {
                if acc + c.w > w && ci > start {
                    rows.push((start, ci));
                    start = ci;
                    acc = 0;
                }
                acc += c.w;
            }
            rows.push((start, self.cells.len()));
        }
        if self.rows.len() >= 2 {
            self.rows.remove(0);
        }
        self.rows.push(((text_w, wrap), rows));
    }

    fn col_of(&self, cell: usize) -> usize {
        self.cells.iter().take(cell).map(|c| c.w).sum()
    }

    fn cell_at(&self, col: usize) -> usize {
        let mut acc = 0;
        for (i, c) in self.cells.iter().enumerate() {
            if acc + c.w > col {
                return i;
            }
            acc += c.w;
        }
        self.cells.len()
    }

    fn total_width(&self) -> usize {
        self.cells.iter().map(|c| c.w).sum()
    }
}

#[derive(Debug, Clone, Copy)]
struct VisualRow {
    line: usize,
    start: usize,
    end: usize,
}

/// A position anchored to a line identity rather than an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Anchor {
    id: u64,
    col: usize,
}

#[derive(Debug, Clone)]
pub struct TextViewport {
    pub id: WidgetId,
    pub scroll: ScrollState,
    pub follow: bool,
    pub wrap: bool,
    /// Caret exposed as the hardware cursor while following and focused.
    /// Owned by the producer; reconciled through eviction, cleared when its
    /// line leaves retention.
    pub caret: Option<CellPos>,
    pub caret_visible: bool,
    pub area: Rect,
    lines: Vec<Logical>,
    max_lines: Option<usize>,
    next_id: u64,
    revision: u64,
    selection: Option<(Anchor, Anchor)>,
    drag_anchor: Option<Anchor>,
    /// Keyboard browse caret for selection extension.
    browse: Option<Anchor>,
    marks: Vec<Mark>,
    visual: Vec<VisualRow>,
    layout_width: u16,
    layout_wrap: bool,
    index_valid: bool,
    /// Leading logical lines whose rows the index holds; lines beyond it were
    /// appended since and extend the index on the next layout.
    indexed_lines: usize,
    layout_size: Option<(u16, u16, bool)>,
    /// Retained reading position while follow is off: the line identity and
    /// wrapped sub-row at the top of the viewport.
    reading: Option<(u64, usize)>,
    work: WorkCounters,
}

impl TextViewport {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            scroll: ScrollState::default(),
            follow: true,
            wrap: false,
            caret: None,
            caret_visible: true,
            area: Rect::ZERO,
            lines: vec![],
            max_lines: None,
            next_id: 1,
            revision: 0,
            selection: None,
            drag_anchor: None,
            browse: None,
            marks: vec![],
            visual: vec![],
            layout_width: 0,
            layout_wrap: false,
            index_valid: false,
            indexed_lines: 0,
            layout_size: None,
            reading: None,
            work: WorkCounters::default(),
        }
    }

    pub fn with_lines(id: WidgetId, lines: Vec<Line>) -> Self {
        let mut v = Self::new(id);
        v.set_lines(lines);
        v
    }

    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    pub fn max_lines(mut self, n: usize) -> Self {
        self.set_max_lines(Some(n));
        self
    }

    // ------------------------------------------------------------ content

    /// Monotonic content revision: bumps on every mutation.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn work_counters(&self) -> WorkCounters {
        self.work
    }

    pub fn retention(&self) -> Option<usize> {
        self.max_lines
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().map(|l| &l.spans)
    }

    pub fn line(&self, i: usize) -> Option<&Line> {
        self.lines.get(i).map(|l| &l.spans)
    }

    /// Source text of a logical line.
    pub fn line_text(&self, i: usize) -> Option<&str> {
        self.lines.get(i).map(|l| l.text.as_str())
    }

    pub fn last(&self) -> Option<&Line> {
        self.lines.last().map(|l| &l.spans)
    }

    fn alloc(&mut self, spans: Line) -> Logical {
        let id = self.next_id;
        self.next_id += 1;
        Logical::new(id, spans)
    }

    /// A structural change: the whole index is rebuilt on the next layout.
    fn touched(&mut self) {
        self.revision += 1;
        self.index_valid = false;
    }

    /// Lines were appended: retained rows stay valid, the tail extends.
    fn appended(&mut self) {
        self.revision += 1;
    }

    /// Retention dropped `drop` leading lines: their rows leave the index
    /// and the remaining rows are renumbered; nothing is re-laid out.
    fn shift_index(&mut self, drop: usize) {
        if !self.index_valid {
            return;
        }
        let gone = self.visual.iter().take_while(|v| v.line < drop).count();
        self.visual.drain(..gone);
        for v in &mut self.visual {
            v.line -= drop;
        }
        self.indexed_lines = self.indexed_lines.saturating_sub(drop);
        self.work.index_shifts += 1;
        self.scroll.set_content(self.visual.len());
        if !self.follow {
            // the reading row keeps its content, not its number
            self.scroll.offset = self.scroll.offset.saturating_sub(gone);
        }
    }

    /// Enforce the retention cap: drop the oldest lines and reconcile every
    /// retained identity. Returns how many lines were evicted.
    fn enforce_cap(&mut self) -> usize {
        let Some(max) = self.max_lines else {
            return 0;
        };
        if self.lines.len() <= max {
            return 0;
        }
        let drop = self.lines.len() - max;
        let evicted: Vec<u64> = self.lines.drain(..drop).map(|l| l.id).collect();
        self.reconcile(drop, &evicted);
        self.shift_index(drop);
        drop
    }

    /// Reconcile anchored state after `drop` leading lines were evicted.
    fn reconcile(&mut self, drop: usize, evicted: &[u64]) {
        let gone = |id: u64| evicted.contains(&id);
        // a selection whose ends are both evicted disappears; a partially
        // surviving range clips to the first retained line start
        if let Some((a, b)) = self.selection {
            let (lo, hi) = if (a.id, a.col) <= (b.id, b.col) {
                (a, b)
            } else {
                (b, a)
            };
            if gone(hi.id) {
                self.selection = None;
            } else if gone(lo.id) {
                let first = self.lines.first().map(|l| l.id).unwrap_or(hi.id);
                let clipped = Anchor { id: first, col: 0 };
                self.selection = Some(if a.id == lo.id {
                    (clipped, b)
                } else {
                    (a, clipped)
                });
                if self.selection.is_some_and(|(x, y)| x == y) {
                    self.selection = None;
                }
            }
        }
        if self.drag_anchor.is_some_and(|d| gone(d.id)) {
            // the press began on a line that no longer exists: the drag
            // continues from the first retained line, never a random one
            self.drag_anchor = self.lines.first().map(|l| Anchor { id: l.id, col: 0 });
        }
        if self.browse.is_some_and(|b| gone(b.id)) {
            self.browse = None;
        }
        if let Some(c) = self.caret {
            if c.line < drop {
                self.caret = None;
            } else {
                self.caret = Some(CellPos {
                    line: c.line - drop,
                    col: c.col,
                });
            }
        }
        if self.reading.is_some_and(|(id, _)| gone(id)) {
            self.reading = self.lines.first().map(|l| (l.id, 0));
        }
        self.marks.retain(|m| m.line >= drop);
        for m in &mut self.marks {
            m.line -= drop;
        }
    }

    pub fn set_lines(&mut self, lines: Vec<Line>) {
        // dataset replacement: identities do not survive, retained state resets
        let mut fresh = Vec::with_capacity(lines.len());
        for l in lines {
            let lg = self.alloc(l);
            fresh.push(lg);
        }
        self.lines = fresh;
        self.selection = None;
        self.drag_anchor = None;
        self.browse = None;
        self.caret = None;
        self.reading = None;
        self.marks.clear();
        self.touched();
        self.enforce_cap();
    }

    pub fn push(&mut self, line: Line) {
        let lg = self.alloc(line);
        self.lines.push(lg);
        self.appended();
        self.enforce_cap();
    }

    pub fn extend(&mut self, lines: Vec<Line>) {
        if lines.is_empty() {
            return;
        }
        for l in lines {
            let lg = self.alloc(l);
            self.lines.push(lg);
        }
        self.appended();
        self.enforce_cap();
    }

    /// Replace the last line in place (a terminal updating its live row).
    /// The identity is retained; selection on the line is clipped to its
    /// new width on the next layout.
    pub fn replace_last(&mut self, line: Line) {
        match self.lines.last_mut() {
            Some(last) => {
                if last.spans == line {
                    return;
                }
                last.set(line);
                self.touched();
            }
            None => self.push(line),
        }
    }

    pub fn pop(&mut self) -> Option<Line> {
        let l = self.lines.pop()?;
        let evicted = [l.id];
        self.reconcile_removed_tail(&evicted, self.lines.len());
        self.touched();
        Some(l.spans)
    }

    fn reconcile_removed_tail(&mut self, evicted: &[u64], new_len: usize) {
        let gone = |id: u64| evicted.contains(&id);
        if self
            .selection
            .is_some_and(|(a, b)| gone(a.id) || gone(b.id))
        {
            self.selection = None;
        }
        if self.drag_anchor.is_some_and(|d| gone(d.id)) {
            self.drag_anchor = None;
        }
        if self.browse.is_some_and(|b| gone(b.id)) {
            self.browse = None;
        }
        if self.caret.is_some_and(|c| c.line >= new_len) {
            self.caret = None;
        }
        if self.reading.is_some_and(|(id, _)| gone(id)) {
            self.reading = self.lines.last().map(|l| (l.id, 0));
        }
        self.marks.retain(|m| m.line < new_len);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.selection = None;
        self.drag_anchor = None;
        self.browse = None;
        self.caret = None;
        self.reading = None;
        self.marks.clear();
        self.touched();
    }

    pub fn set_max_lines(&mut self, n: Option<usize>) {
        self.max_lines = n;
        if self.enforce_cap() > 0 {
            self.touched();
        }
    }

    // ------------------------------------------------------------ marks

    pub fn set_marks(&mut self, marks: Vec<Mark>) {
        self.marks = marks;
    }

    pub fn clear_marks(&mut self) {
        self.marks.clear();
    }

    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    /// Scroll so a logical line is visible; pauses follow.
    pub fn reveal_line(&mut self, line: usize) {
        self.ensure_index();
        if let Some(vi) = self.visual.iter().position(|v| v.line == line) {
            self.follow = false;
            self.scroll.ensure_visible(vi);
            self.remember_reading();
        }
    }

    // ------------------------------------------------------------ layout

    fn ensure_layout(&mut self, text_w: u16) {
        let wrap = self.wrap;
        let same_key = self.layout_width == text_w && self.layout_wrap == wrap;
        if self.index_valid && same_key {
            // incremental: only lines appended since the index was built
            let from = self.indexed_lines.min(self.lines.len());
            let key = (text_w, wrap);
            let mut extended = false;
            for li in from..self.lines.len() {
                let l = &mut self.lines[li];
                if !l.segmented {
                    l.segment();
                    self.work.segmented_lines += 1;
                }
                if !l.has_rows(key) {
                    l.reflow(text_w, wrap);
                    self.work.reflowed_lines += 1;
                }
                for &(s, e) in self.lines[li].rows_at(key) {
                    self.visual.push(VisualRow {
                        line: li,
                        start: s,
                        end: e,
                    });
                }
                extended = true;
            }
            self.indexed_lines = self.lines.len();
            if extended {
                self.work.index_extends += 1;
                self.scroll.set_content(self.visual.len());
                self.clip_selection();
            }
            return;
        }
        for l in &mut self.lines {
            if !l.segmented {
                l.segment();
                self.work.segmented_lines += 1;
            }
            if !l.has_rows((text_w, wrap)) {
                l.reflow(text_w, wrap);
                self.work.reflowed_lines += 1;
            }
        }
        self.layout_width = text_w;
        self.layout_wrap = wrap;
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        let key = (self.layout_width, self.layout_wrap);
        let mut visual = Vec::with_capacity(self.lines.len());
        for (li, l) in self.lines.iter().enumerate() {
            for &(s, e) in l.rows_at(key) {
                visual.push(VisualRow {
                    line: li,
                    start: s,
                    end: e,
                });
            }
        }
        self.visual = visual;
        self.index_valid = true;
        self.indexed_lines = self.lines.len();
        self.work.index_rebuilds += 1;
        self.scroll.set_content(self.visual.len());
        self.clip_selection();
    }

    fn clip_selection(&mut self) {
        // clip a selection whose columns overflow a replaced line
        if let Some((a, b)) = self.selection {
            let clip = |x: Anchor, lines: &[Logical]| -> Anchor {
                match lines.iter().find(|l| l.id == x.id) {
                    Some(l) => Anchor {
                        id: x.id,
                        col: x.col.min(l.total_width()),
                    },
                    None => x,
                }
            };
            let na = clip(a, &self.lines);
            let nb = clip(b, &self.lines);
            self.selection = if na == nb { None } else { Some((na, nb)) };
        }
        if !self.follow {
            self.restore_reading();
        }
    }

    fn ensure_index(&mut self) {
        if !self.index_valid || self.indexed_lines < self.lines.len() {
            let w = if self.layout_width == 0 {
                self.area.width.max(1)
            } else {
                self.layout_width
            };
            self.ensure_layout(w);
        }
    }

    fn remember_reading(&mut self) {
        if self.follow {
            self.reading = None;
            return;
        }
        let Some(vr) = self.visual.get(self.scroll.offset) else {
            self.reading = None;
            return;
        };
        let key = (self.layout_width, self.layout_wrap);
        let sub = self.lines[vr.line]
            .rows_at(key)
            .iter()
            .position(|&(s, _)| s == vr.start)
            .unwrap_or(0);
        self.reading = Some((self.lines[vr.line].id, sub));
    }

    fn restore_reading(&mut self) {
        let Some((id, sub)) = self.reading else {
            return;
        };
        let Some(li) = self.lines.iter().position(|l| l.id == id) else {
            self.scroll.jump_start();
            return;
        };
        let key = (self.layout_width, self.layout_wrap);
        let rows = self.lines[li].rows_at(key);
        let sub = sub.min(rows.len().saturating_sub(1));
        let start = rows.get(sub).map(|r| r.0).unwrap_or(0);
        let target = self
            .visual
            .iter()
            .position(|v| v.line == li && v.start == start)
            .unwrap_or(0);
        self.scroll.scroll_to(target);
    }

    /// Lay the text out for `area` without drawing: owners that render a
    /// copy of the viewport call this before routing mouse or key events to
    /// the original so positions and page sizes match the last frame.
    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
        let size = (area.width, area.height, self.wrap);
        let dirty = !self.index_valid || self.indexed_lines < self.lines.len();
        if dirty || self.layout_size != Some(size) {
            // Cache the final layout, including the scrollbar decision. The
            // width that settled last time is tried first, so a live stream
            // that already overflows extends its index at the scrollbar
            // width instead of rebuilding at both widths every frame.
            let narrow = area.width.saturating_sub(1);
            let preferred = if self.layout_width == narrow && self.wrap == self.layout_wrap {
                narrow
            } else {
                area.width
            };
            self.ensure_layout(preferred);
            self.scroll.set_viewport(area.height as usize);
            if self.scroll.overflows() && preferred == area.width {
                self.ensure_layout(narrow);
            } else if !self.scroll.overflows() && preferred == narrow {
                self.ensure_layout(area.width);
                self.scroll.set_viewport(area.height as usize);
                if self.scroll.overflows() {
                    self.ensure_layout(narrow);
                }
            }
            self.layout_size = Some(size);
        } else {
            self.scroll.set_viewport(area.height as usize);
        }
        if self.follow {
            self.scroll.jump_end();
        }
    }

    // -------------------------------------------------------- positions

    fn index_of(&self, id: u64) -> Option<usize> {
        self.lines.iter().position(|l| l.id == id)
    }

    fn resolve(&self, a: Anchor) -> Option<CellPos> {
        let line = self.index_of(a.id)?;
        Some(CellPos { line, col: a.col })
    }

    fn anchor_at(&self, p: CellPos) -> Option<Anchor> {
        let l = self.lines.get(p.line)?;
        Some(Anchor {
            id: l.id,
            col: p.col,
        })
    }

    /// True after a press anchored a selection and before it was cleared.
    pub fn has_anchor(&self) -> bool {
        self.drag_anchor.is_some()
    }

    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        let (a, b) = self.selection?;
        let a = self.resolve(a)?;
        let b = self.resolve(b)?;
        if a == b {
            return None;
        }
        Some((a.min(b), a.max(b)))
    }

    pub fn clear_selection(&mut self) -> Outcome {
        self.browse = None;
        if self.selection.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn is_at_tail(&self) -> bool {
        self.scroll.offset >= self.scroll.max_offset()
    }

    /// Lines behind the live tail (scrollback depth).
    pub fn scrollback_depth(&self) -> usize {
        self.scroll.max_offset().saturating_sub(self.scroll.offset)
    }

    pub fn set_follow(&mut self, on: bool) {
        self.follow = on;
        if on {
            self.scroll.jump_end();
            self.reading = None;
        } else {
            self.remember_reading();
        }
    }

    fn col_of(&self, line: usize, cell: usize) -> usize {
        self.lines.get(line).map(|l| l.col_of(cell)).unwrap_or(0)
    }

    fn cell_at(&self, line: usize, col: usize) -> usize {
        self.lines.get(line).map(|l| l.cell_at(col)).unwrap_or(0)
    }

    /// Map a screen position to a logical position (line, column).
    pub fn pos_at(&self, pos: Position) -> Option<CellPos> {
        if self.area.is_empty() || self.visual.is_empty() {
            return None;
        }
        let row = (pos.y.saturating_sub(self.area.y) as usize + self.scroll.offset)
            .min(self.visual.len().saturating_sub(1));
        let vr = self.visual.get(row)?;
        let x = pos.x.saturating_sub(self.area.x) as usize;
        let base = self.col_of(vr.line, vr.start);
        let line_w = self.col_of(vr.line, vr.end);
        Some(CellPos {
            line: vr.line,
            col: (base + x).min(line_w),
        })
    }

    /// Source bytes of a cell range within a line.
    fn source_slice(&self, line: usize, from: usize, to: usize) -> String {
        let Some(l) = self.lines.get(line) else {
            return String::new();
        };
        let from = from.min(l.cells.len());
        let to = to.min(l.cells.len());
        if from >= to {
            return String::new();
        }
        let start = l.cells[from].src.start;
        let end = l.cells[to - 1].src.end;
        l.text[start..end].to_owned()
    }

    /// The selected text as source bytes (tabs and controls preserved);
    /// trailing spaces of each line are dropped.
    pub fn selected_text(&self) -> Option<String> {
        let (a, b) = self.selection()?;
        let mut out = String::new();
        for li in a.line..=b.line {
            let Some(l) = self.lines.get(li) else { break };
            let from = if li == a.line {
                self.cell_at(li, a.col)
            } else {
                0
            };
            let to = if li == b.line {
                self.cell_at(li, b.col)
            } else {
                l.cells.len()
            };
            let text = self.source_slice(li, from, to);
            out.push_str(text.trim_end_matches(' '));
            if li != b.line {
                out.push('\n');
            }
        }
        Some(out)
    }

    fn is_word(c: &Cell) -> bool {
        c.kind == CellKind::Text
            && c.g
                .chars()
                .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '/' || ch == '.')
    }

    fn word_bounds(&self, line: usize, ci: usize) -> Option<(usize, usize)> {
        let cs = &self.lines.get(line)?.cells;
        if cs.is_empty() {
            return None;
        }
        let ci = ci.min(cs.len() - 1);
        if !Self::is_word(&cs[ci]) {
            return None;
        }
        let mut s = ci;
        while s > 0 && Self::is_word(&cs[s - 1]) {
            s -= 1;
        }
        let mut e = ci + 1;
        while e < cs.len() && Self::is_word(&cs[e]) {
            e += 1;
        }
        Some((s, e))
    }

    /// Double-click: select the word under the pointer.
    pub fn select_word_at(&mut self, pos: Position) -> Outcome {
        let Some(p) = self.pos_at(pos) else {
            return Outcome::Ignored;
        };
        let ci = self.cell_at(p.line, p.col);
        let Some((s, e)) = self.word_bounds(p.line, ci) else {
            return if self.lines.get(p.line).is_some_and(|l| l.cells.is_empty()) {
                Outcome::Ignored
            } else {
                Outcome::Consumed
            };
        };
        let a = CellPos {
            line: p.line,
            col: self.col_of(p.line, s),
        };
        let b = CellPos {
            line: p.line,
            col: self.col_of(p.line, e),
        };
        self.selection = self.anchor_at(a).zip(self.anchor_at(b));
        self.drag_anchor = None;
        self.browse = self.anchor_at(b);
        Outcome::Changed
    }

    /// Mouse down: anchor a drag; no selection yet.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        let had = self.selection.is_some();
        self.selection = None;
        self.browse = None;
        self.drag_anchor = self.pos_at(pos).and_then(|p| self.anchor_at(p));
        if had {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Drag: extend the selection from the anchor; auto-scroll at the
    /// vertical edges.
    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        let Some(anchor) = self.drag_anchor else {
            return Outcome::Ignored;
        };
        if pos.y < self.area.y {
            self.scroll.scroll_by(-1);
            self.follow = false;
            self.remember_reading();
        } else if pos.y >= self.area.bottom() {
            self.scroll.scroll_by(1);
            self.remember_reading();
        }
        let clamped = Position::new(
            pos.x
                .clamp(self.area.x, self.area.right().saturating_sub(1)),
            pos.y
                .clamp(self.area.y, self.area.bottom().saturating_sub(1)),
        );
        let Some(head) = self.pos_at(clamped).and_then(|p| self.anchor_at(p)) else {
            return Outcome::Consumed;
        };
        self.selection = Some((anchor, head));
        self.browse = Some(head);
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        self.follow = self.is_at_tail();
        self.remember_reading();
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        self.follow = self.is_at_tail();
        self.remember_reading();
        Outcome::Changed
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id)
    }

    // ------------------------------------------------- keyboard selection

    /// The browse caret: the selection head, else the last cell of the last
    /// visible line.
    fn browse_caret(&self) -> Option<CellPos> {
        if let Some(b) = self.browse.and_then(|b| self.resolve(b)) {
            return Some(b);
        }
        if self.lines.is_empty() {
            return None;
        }
        let last_visible = self
            .visual
            .get(self.scroll.visible_range().end.saturating_sub(1))
            .map(|v| v.line)
            .unwrap_or(self.lines.len() - 1);
        Some(CellPos {
            line: last_visible,
            col: self.lines[last_visible].total_width(),
        })
    }

    fn move_caret(&self, from: CellPos, code: KeyCode, word: bool) -> CellPos {
        let n = self.lines.len();
        let line_w = |li: usize| self.lines.get(li).map(|l| l.total_width()).unwrap_or(0);
        match code {
            KeyCode::Left => {
                let ci = self.cell_at(from.line, from.col);
                if ci == 0 {
                    if from.line > 0 {
                        return CellPos {
                            line: from.line - 1,
                            col: line_w(from.line - 1),
                        };
                    }
                    return from;
                }
                let target = if word {
                    let cs = &self.lines[from.line].cells;
                    let mut i = ci;
                    while i > 0 && !Self::is_word(&cs[i - 1]) {
                        i -= 1;
                    }
                    while i > 0 && Self::is_word(&cs[i - 1]) {
                        i -= 1;
                    }
                    i
                } else {
                    ci - 1
                };
                CellPos {
                    line: from.line,
                    col: self.col_of(from.line, target),
                }
            }
            KeyCode::Right => {
                let cs_len = self
                    .lines
                    .get(from.line)
                    .map(|l| l.cells.len())
                    .unwrap_or(0);
                let ci = self.cell_at(from.line, from.col);
                if ci >= cs_len {
                    if from.line + 1 < n {
                        return CellPos {
                            line: from.line + 1,
                            col: 0,
                        };
                    }
                    return from;
                }
                let target = if word {
                    let cs = &self.lines[from.line].cells;
                    let mut i = ci;
                    while i < cs.len() && !Self::is_word(&cs[i]) {
                        i += 1;
                    }
                    while i < cs.len() && Self::is_word(&cs[i]) {
                        i += 1;
                    }
                    i
                } else {
                    ci + 1
                };
                CellPos {
                    line: from.line,
                    col: self.col_of(from.line, target),
                }
            }
            KeyCode::Up => {
                if from.line == 0 {
                    return CellPos { line: 0, col: 0 };
                }
                CellPos {
                    line: from.line - 1,
                    col: from.col.min(line_w(from.line - 1)),
                }
            }
            KeyCode::Down => {
                if from.line + 1 >= n {
                    return CellPos {
                        line: from.line,
                        col: line_w(from.line),
                    };
                }
                CellPos {
                    line: from.line + 1,
                    col: from.col.min(line_w(from.line + 1)),
                }
            }
            KeyCode::Home => {
                if word {
                    CellPos { line: 0, col: 0 }
                } else {
                    CellPos {
                        line: from.line,
                        col: 0,
                    }
                }
            }
            KeyCode::End => {
                if word {
                    CellPos {
                        line: n.saturating_sub(1),
                        col: line_w(n.saturating_sub(1)),
                    }
                } else {
                    CellPos {
                        line: from.line,
                        col: line_w(from.line),
                    }
                }
            }
            _ => from,
        }
    }

    fn extend_selection(&mut self, code: KeyCode, word: bool) -> Outcome {
        self.ensure_index();
        let Some(from) = self.browse_caret() else {
            return Outcome::Consumed;
        };
        let to = self.move_caret(from, code, word);
        let anchor = match self.selection {
            Some((a, _)) => a,
            None => match self.anchor_at(from) {
                Some(a) => a,
                None => return Outcome::Consumed,
            },
        };
        let Some(head) = self.anchor_at(to) else {
            return Outcome::Consumed;
        };
        self.selection = Some((anchor, head));
        self.browse = Some(head);
        self.drag_anchor = None;
        // the caret stays on screen; selection pauses follow
        if let Some(vi) = self.visual.iter().position(|v| {
            v.line == to.line
                && self.col_of(v.line, v.start) <= to.col
                && (to.col < self.col_of(v.line, v.end) || v.end == self.lines[v.line].cells.len())
        }) {
            self.follow = false;
            self.scroll.ensure_visible(vi);
            self.remember_reading();
        }
        Outcome::Changed
    }

    /// ↑↓ j k · PgUp PgDn · Home End g G · `f` follow · `y` copy ·
    /// Shift+arrows extend a keyboard selection (Ctrl/Alt by word,
    /// Ctrl+Shift+Home/End to the document ends) · Esc clears.
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ViewportEvent>) {
        if key.shift()
            && matches!(
                key.code,
                KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
            )
        {
            let word = key.ctrl() || key.alt();
            let o = self.extend_selection(key.code, word);
            return (o, Some(ViewportEvent::SelectionChanged));
        }
        let before = self.scroll.offset;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => self.scroll.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.scroll.scroll_by(1),
            KeyCode::PageUp if key.plain() => self.scroll.page_up(),
            KeyCode::PageDown if key.plain() => self.scroll.page_down(),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.scroll.jump_start(),
            KeyCode::End | KeyCode::Char('G') if key.plain() => {
                self.scroll.jump_end();
                self.follow = true;
                self.reading = None;
                return (Outcome::Changed, Some(ViewportEvent::FollowChanged(true)));
            }
            KeyCode::Char('f') if key.plain() => {
                self.set_follow(!self.follow);
                return (
                    Outcome::Changed,
                    Some(ViewportEvent::FollowChanged(self.follow)),
                );
            }
            KeyCode::Char('y') if key.plain() => {
                return match self.selected_text() {
                    Some(t) => (Outcome::Changed, Some(ViewportEvent::Copy(t))),
                    None => (Outcome::Consumed, None),
                };
            }
            KeyCode::Esc if key.plain() => {
                return match self.clear_selection() {
                    Outcome::Changed => (Outcome::Changed, Some(ViewportEvent::SelectionChanged)),
                    _ => (Outcome::Ignored, None),
                };
            }
            _ => return (Outcome::Ignored, None),
        }
        if self.scroll.offset != before {
            self.follow = self.is_at_tail();
            self.remember_reading();
        }
        (Outcome::Changed, None)
    }

    // ------------------------------------------------------------ render

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.set_area(area);
        let has_sb = self.scroll.overflows();
        let text_w = self.layout_width;
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        let sel = self.selection();
        fill(buf, area, Style::new().bg(bg));
        for (k, vi) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let Some(vr) = self.visual.get(vi).copied() else {
                break;
            };
            let l = &self.lines[vr.line];
            let line_marks: Vec<&Mark> = self.marks.iter().filter(|m| m.line == vr.line).collect();
            let mut x = area.x;
            let mut col = l.col_of(vr.start);
            for c in &l.cells[vr.start..vr.end] {
                if x + c.w as u16 > area.x + text_w {
                    break;
                }
                let mut st = Style::new().fg(t.tone(c.style.tone)).bg(bg);
                if c.style.bold {
                    st = st.add_modifier(Modifier::BOLD);
                }
                if c.style.italic {
                    st = st.add_modifier(Modifier::ITALIC);
                }
                if c.style.underline {
                    st = st.add_modifier(Modifier::UNDERLINED);
                }
                if c.kind == CellKind::Control {
                    st = st.fg(t.text_muted);
                }
                if c.style.reversed {
                    st = Style::new().fg(t.canvas).bg(t.text_primary);
                }
                if let Some(m) = line_marks
                    .iter()
                    .find(|m| m.range.start < c.src.end && c.src.start < m.range.end)
                {
                    st = if m.current {
                        Style::new()
                            .fg(t.canvas)
                            .bg(t.accent)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        st.add_modifier(Modifier::UNDERLINED).bg(t.lift(bg))
                    };
                }
                let p = CellPos { line: vr.line, col };
                if let Some((a, b)) = sel
                    && p >= a
                    && p < b
                {
                    st = t.selection().add_modifier(st.add_modifier);
                }
                buf.set_string(x, y, &c.g, st);
                x += c.w as u16;
                col += c.w;
            }
            // a selection that spans to the line end paints the trailing gap
            if let Some((a, b)) = sel
                && vr.line >= a.line
                && vr.line < b.line
                && vr.end == l.cells.len()
            {
                let tail = Rect::new(x, y, (area.x + text_w).saturating_sub(x).min(1), 1);
                fill(buf, tail, t.selection());
            }
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        // the hardware cursor: the keyboard selection head while selecting,
        // else the producer caret while following
        let cursor_pos = if focused {
            match self.browse.and_then(|b| self.resolve(b)) {
                Some(b) if self.selection.is_some() => Some(b),
                _ if self.follow && self.caret_visible => self.caret,
                _ => None,
            }
        } else {
            None
        };
        if let Some(c) = cursor_pos
            && let Some(vi) = self.visual.iter().position(|v| {
                v.line == c.line
                    && self.col_of(v.line, v.start) <= c.col
                    && (c.col < self.col_of(v.line, v.end)
                        || v.end == self.lines.get(v.line).map_or(0, |l| l.cells.len()))
            })
            && self.scroll.visible_range().contains(&vi)
        {
            let vr = self.visual[vi];
            let base = self.col_of(vr.line, vr.start);
            let cx = area.x + (c.col.saturating_sub(base) as u16).min(text_w.saturating_sub(1));
            let cy = area.y + (vi - self.scroll.offset) as u16;
            ctx.set_cursor(Position::new(cx, cy));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn lines(n: usize) -> Vec<Line> {
        (0..n)
            .map(|i| vec![Span::plain(format!("line {i} alpha beta"))])
            .collect()
    }

    fn render(v: &mut TextViewport, w: u16, h: u16) -> Buffer {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        v.render(Rect::new(0, 0, w, h), &mut buf, &mut ctx, t.canvas);
        buf
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect::<String>()
            .trim_end_matches([' ', '│', '┃'])
            .to_owned()
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn follows_tail_and_wheel_leaves_it() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(50));
        render(&mut v, 40, 10);
        assert!(v.is_at_tail());
        assert_eq!(v.scroll.offset, 40);
        v.on_wheel(-3);
        assert!(!v.follow);
        assert_eq!(v.scrollback_depth(), 3);
        v.on_key(&key(KeyCode::End, KeyModifiers::NONE));
        assert!(v.follow);
    }

    #[test]
    fn drag_selects_and_copies_text() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(5));
        v.follow = false;
        render(&mut v, 40, 10);
        v.on_click(Position::new(0, 1));
        v.on_drag(Position::new(6, 2));
        assert_eq!(
            v.selected_text().as_deref(),
            Some("line 1 alpha beta\nline 2")
        );
        let (_, ev) = v.on_key(&key(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(matches!(ev, Some(ViewportEvent::Copy(t)) if t.starts_with("line 1")));
        v.select_word_at(Position::new(8, 0));
        assert_eq!(v.selected_text().as_deref(), Some("alpha"));
    }

    #[test]
    fn wraps_long_lines_and_bounds_retention() {
        let mut v = TextViewport::new(WidgetId::of("v")).wrap(true).max_lines(3);
        for i in 0..5 {
            v.push(vec![Span::plain(format!("{i} {}", "x".repeat(30)))]);
        }
        assert_eq!(v.len(), 3);
        render(&mut v, 20, 10);
        assert_eq!(v.visual.len(), 6, "each line wraps into two visual rows");
    }

    #[test]
    fn unchanged_overflow_redraw_reuses_layout() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(4000)).wrap(true);
        let expected = render(&mut v, 100, 30);
        let work = v.work_counters();
        let started = std::time::Instant::now();
        for _ in 0..10 {
            v.set_area(Rect::new(0, 0, 100, 30));
            assert_eq!(render(&mut v, 100, 30), expected);
        }
        eprintln!(
            "4000-line viewport, 10 unchanged redraws: {:?}",
            started.elapsed()
        );
        assert_eq!(
            v.work_counters(),
            work,
            "unchanged scrollback must not reflow"
        );
    }

    #[test]
    fn cached_layout_reflows_after_resize_wrap_and_content_changes() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(20)).wrap(true);
        for (w, h, wrap) in [(40, 10, true), (8, 6, true), (40, 30, true), (8, 30, false)] {
            v.wrap = wrap;
            let mut fresh = TextViewport::with_lines(v.id, v.lines().cloned().collect()).wrap(wrap);
            assert_eq!(render(&mut v, w, h), render(&mut fresh, w, h));
            assert_eq!(v.scroll, fresh.scroll);
        }
        v.set_lines(lines(2));
        let mut fresh = TextViewport::with_lines(v.id, lines(2));
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.push(vec![Span::plain("appended")]);
        fresh.push(vec![Span::plain("appended")]);
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.replace_last(vec![Span::plain("replaced")]);
        fresh.replace_last(vec![Span::plain("replaced")]);
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.clear();
        fresh.clear();
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
    }

    // ---------------------------------------------------------- F10

    #[test]
    fn split_styles_never_split_a_grapheme_cluster() {
        // one span
        let mut one =
            TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("👩\u{200d}💻X")]]);
        one.follow = false;
        let a = render(&mut one, 10, 1);
        // the same cluster split across a ZWJ boundary
        let mut split = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("👩"),
                Span::new("\u{200d}💻", Tone::Warning),
                Span::plain("X"),
            ]],
        );
        split.follow = false;
        let b = render(&mut split, 10, 1);
        assert_eq!(row(&a, 0), row(&b, 0));
        assert_eq!(one.lines[0].cells.len(), 2);
        assert_eq!(split.lines[0].cells.len(), 2);
        assert_eq!(split.lines[0].col_of(1), 2, "X sits at column 2 both ways");
        // combining accent split from its base survives copy
        let mut acc = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("a"),
                Span::new("\u{301}", Tone::Error),
                Span::plain("X"),
            ]],
        );
        acc.follow = false;
        render(&mut acc, 10, 1);
        acc.selection = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 2 }));
        assert_eq!(acc.selected_text().as_deref(), Some("a\u{301}X"));
        // the cluster takes the style of the span holding its first byte
        assert_eq!(acc.lines[0].cells[0].style.tone, Tone::Normal);
        // flags and variation selectors keep one cell each
        let mut flags = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("🇯"),
                Span::new("🇵", Tone::Muted),
                Span::plain("☕\u{fe0f}!"),
            ]],
        );
        flags.follow = false;
        render(&mut flags, 12, 1);
        assert_eq!(flags.lines[0].cells.len(), 3);
        assert_eq!(flags.lines[0].total_width(), 5);
    }

    #[test]
    fn narrow_clipping_and_nonzero_origin_keep_cells_whole() {
        let mut v =
            TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("ab日本cd")]]);
        v.follow = false;
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 3));
        v.render(Rect::new(5, 1, 3, 1), &mut buf, &mut ctx, t.canvas);
        assert_eq!(
            row(&buf, 1).trim(),
            "ab",
            "a wide cell never paints past the edge"
        );
        assert_eq!(
            v.pos_at(Position::new(6, 1)),
            Some(CellPos { line: 0, col: 1 })
        );
    }

    // ---------------------------------------------------------- F14

    #[test]
    fn tabs_and_controls_display_visibly_and_copy_as_source() {
        let mut v = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![Span::plain("A\tB\u{1b}[0m\u{7}C\r")]],
        );
        v.follow = false;
        let buf = render(&mut v, 30, 1);
        assert_eq!(row(&buf, 0), "A    B^[[0m^GC^M");
        assert_eq!(v.lines[0].cells[1].kind, CellKind::TabPad);
        assert_eq!(
            v.pos_at(Position::new(3, 0)),
            Some(CellPos { line: 0, col: 3 })
        );
        v.selection = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 30 }));
        assert_eq!(
            v.selected_text().as_deref(),
            Some("A\tB\u{1b}[0m\u{7}C\r"),
            "copy returns the source bytes"
        );
        // selecting inside a tab pad selects the whole tab
        v.selection = Some((Anchor { id: 1, col: 2 }, Anchor { id: 1, col: 4 }));
        assert_eq!(v.selected_text().as_deref(), Some("\t"));
    }

    // ---------------------------------------------------------- F11

    #[test]
    fn retention_is_enforced_on_every_ingestion_path() {
        let eight = lines(8);
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.set_lines(eight.clone());
        assert_eq!(v.len(), 3);
        assert_eq!(v.line_text(0), Some("line 5 alpha beta"));
        let mut w = TextViewport::with_lines(WidgetId::of("v"), eight.clone()).max_lines(3);
        assert_eq!(w.len(), 3);
        w.set_max_lines(Some(1));
        assert_eq!(w.line_text(0), Some("line 7 alpha beta"));
        w.extend(lines(4));
        assert_eq!(w.len(), 1);
        assert_eq!(w.line_text(0), Some("line 3 alpha beta"));
        let mut z = TextViewport::new(WidgetId::of("v")).max_lines(0);
        z.push(vec![Span::plain("x")]);
        z.replace_last(vec![Span::plain("y")]);
        assert_eq!(z.len(), 0, "a zero cap retains nothing");
        let mut e = TextViewport::new(WidgetId::of("v")).max_lines(2);
        e.replace_last(vec![Span::plain("first")]);
        assert_eq!(e.len(), 1, "replace on empty appends");
        let mut n = TextViewport::new(WidgetId::of("v"));
        n.extend(lines(5000));
        assert_eq!(n.len(), 5000, "no cap keeps everything");
        assert!(v.revision() > 0);
    }

    // ---------------------------------------------------------- F12

    #[test]
    fn eviction_never_retargets_a_selection() {
        // cap 3, A/B/C: select A, append D → the selection is gone, never B
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 5);
        v.on_click(Position::new(0, 0));
        v.on_drag(Position::new(1, 0));
        assert_eq!(v.selected_text().as_deref(), Some("A"));
        v.push(vec![Span::plain("D")]);
        render(&mut v, 10, 5);
        assert_eq!(v.selected_text(), None, "an evicted selection is cleared");
        // partial eviction clips to the first retained line
        let mut p = TextViewport::new(WidgetId::of("v")).max_lines(3);
        p.follow = false;
        for s in ["A", "B", "C"] {
            p.push(vec![Span::plain(s)]);
        }
        render(&mut p, 10, 5);
        p.on_click(Position::new(0, 0));
        p.on_drag(Position::new(1, 2));
        assert_eq!(p.selected_text().as_deref(), Some("A\nB\nC"));
        p.push(vec![Span::plain("D")]);
        render(&mut p, 10, 5);
        assert_eq!(p.selected_text().as_deref(), Some("B\nC"));
    }

    #[test]
    fn active_drag_survives_eviction_with_the_right_source() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 5);
        v.on_click(Position::new(0, 1)); // press on B
        v.push(vec![Span::plain("D")]);
        render(&mut v, 10, 5);
        v.on_drag(Position::new(1, 2)); // now B C D; drag to D
        assert_eq!(v.selected_text().as_deref(), Some("B\nC\nD"));
        let mut a = TextViewport::new(WidgetId::of("v")).max_lines(3);
        a.follow = false;
        for s in ["A", "B", "C"] {
            a.push(vec![Span::plain(s)]);
        }
        render(&mut a, 10, 5);
        a.on_click(Position::new(0, 0)); // press on A
        a.push(vec![Span::plain("D")]);
        a.push(vec![Span::plain("E")]);
        render(&mut a, 10, 5);
        a.on_drag(Position::new(1, 0));
        assert_eq!(
            a.selected_text().as_deref(),
            Some("C"),
            "an evicted anchor restarts at the first retained line"
        );
    }

    #[test]
    fn reading_position_is_retained_while_follow_is_off() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 2);
        v.scroll.scroll_to(1);
        v.remember_reading();
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "B");
        v.push(vec![Span::plain("D")]);
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "B", "B survives, so B stays on top");
        v.push(vec![Span::plain("E")]);
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "C", "B is gone: the first retained line");
        // wrapped rows: the sub-row of the reading position is kept
        let mut w = TextViewport::new(WidgetId::of("w")).wrap(true);
        w.follow = false;
        w.push(vec![Span::plain("aaaaaaaaa bbbbbbbbb ccccccccc")]);
        w.push(vec![Span::plain("second")]);
        render(&mut w, 11, 2);
        w.scroll.scroll_to(1);
        w.remember_reading();
        w.push(vec![Span::plain("third")]);
        let buf = render(&mut w, 11, 2);
        assert_eq!(row(&buf, 0), "bbbbbbbbb");
    }

    #[test]
    fn producer_caret_follows_eviction_and_replacement() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(2);
        v.push(vec![Span::plain("A")]);
        v.push(vec![Span::plain("B")]);
        v.caret = Some(CellPos { line: 1, col: 1 });
        v.push(vec![Span::plain("C")]);
        assert_eq!(v.caret, Some(CellPos { line: 0, col: 1 }));
        v.push(vec![Span::plain("D")]);
        assert_eq!(v.caret, None, "the caret's line was evicted");
        v.caret = Some(CellPos { line: 1, col: 0 });
        v.pop();
        assert_eq!(v.caret, None);
    }

    #[test]
    fn replace_last_keeps_identity_and_clips_selection() {
        let mut v = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![Span::plain("first")], vec![Span::plain("second line")]],
        );
        v.follow = false;
        render(&mut v, 20, 2);
        v.on_click(Position::new(0, 1));
        v.on_drag(Position::new(10, 1));
        assert_eq!(v.selected_text().as_deref(), Some("second lin"));
        let rev = v.revision();
        v.replace_last(vec![Span::plain("short")]);
        assert!(v.revision() > rev);
        render(&mut v, 20, 2);
        assert_eq!(v.selected_text().as_deref(), Some("short"));
        let rev = v.revision();
        v.replace_last(vec![Span::plain("short")]);
        assert_eq!(
            v.revision(),
            rev,
            "an identical replacement is not a mutation"
        );
    }

    // ---------------------------------------------------------- F13

    #[test]
    fn every_mutation_is_visible_next_frame() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("OLD")]]);
        v.follow = false;
        let a = render(&mut v, 10, 1);
        assert_eq!(row(&a, 0), "OLD");
        v.replace_last(vec![Span::plain("NEW")]);
        let b = render(&mut v, 10, 1);
        assert_eq!(row(&b, 0), "NEW", "same byte length edit is rendered");
        v.selection = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 3 }));
        assert_eq!(v.selected_text().as_deref(), Some("NEW"));
        v.set_lines(vec![vec![Span::plain("x")], vec![Span::plain("y")]]);
        assert!(v.selection().is_none(), "replacement resets identity");
        let c = render(&mut v, 10, 2);
        assert_eq!(row(&c, 1), "y");
        v.pop();
        let d = render(&mut v, 10, 2);
        assert_eq!(row(&d, 1), "");
    }

    // ---------------------------------------------------------- F15

    #[test]
    fn tail_update_reparses_only_the_changed_line() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(1000)).wrap(true);
        render(&mut v, 80, 20);
        let w0 = v.work_counters();
        for i in 0..20 {
            v.replace_last(vec![Span::plain(format!("tail {i}"))]);
            render(&mut v, 80, 20);
        }
        let w1 = v.work_counters();
        assert_eq!(w1.segmented_lines - w0.segmented_lines, 20);
        // the scrollbar decision lays out two widths: at most two reflows
        // per changed line, none for the unchanged 999
        assert!(w1.reflowed_lines - w0.reflowed_lines <= 40);
        for i in 0..20 {
            v.push(vec![Span::plain(format!("append {i}"))]);
            render(&mut v, 80, 20);
        }
        let w2 = v.work_counters();
        assert_eq!(w2.segmented_lines - w1.segmented_lines, 20);
        // fresh-layout equality after the churn
        let mut fresh =
            TextViewport::with_lines(WidgetId::of("v"), v.lines().cloned().collect()).wrap(true);
        assert_eq!(render(&mut v, 80, 20), render(&mut fresh, 80, 20));
        // a resize reflows every line once, without re-segmenting any
        let w3 = v.work_counters();
        render(&mut v, 60, 20);
        let w4 = v.work_counters();
        assert_eq!(w4.segmented_lines, w3.segmented_lines);
        assert!(w4.reflowed_lines - w3.reflowed_lines <= 2 * v.len());
        assert!(w4.reflowed_lines - w3.reflowed_lines >= v.len());
    }

    // ---------------------------------------------------------- F19

    #[test]
    fn keyboard_selection_matches_mouse_selection() {
        let text = vec![
            vec![Span::plain("café 日本 👩\u{200d}💻 done")],
            vec![Span::plain("second línea here")],
        ];
        let mut k = TextViewport::with_lines(WidgetId::of("v"), text.clone());
        k.follow = false;
        render(&mut k, 40, 4);
        let mut m = TextViewport::with_lines(WidgetId::of("v"), text);
        m.follow = false;
        render(&mut m, 40, 4);
        // mouse: from col 5 line 0 to col 6 line 1
        m.on_click(Position::new(5, 0));
        m.on_drag(Position::new(6, 1));
        // keyboard: put the caret at line 0 col 5 with a click, then extend
        k.on_click(Position::new(5, 0));
        k.browse = k.anchor_at(CellPos { line: 0, col: 5 });
        k.on_key(&key(KeyCode::Down, KeyModifiers::SHIFT));
        k.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(k.selected_text(), m.selected_text());
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond")
        );
        // word extension and line ends
        k.on_key(&key(
            KeyCode::Right,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        ));
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond línea")
        );
        k.on_key(&key(KeyCode::End, KeyModifiers::SHIFT));
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond línea here")
        );
        k.on_key(&key(
            KeyCode::Home,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        ));
        assert_eq!(k.selected_text().as_deref(), Some("café"));
        // Esc clears, the second Esc is ignored so the owner may act on it
        let (o, _) = k.on_key(&key(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Changed);
        assert!(!k.has_selection());
        let (o, _) = k.on_key(&key(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Ignored);
        // empty viewport: selection keys are consumed, nothing panics
        let mut e = TextViewport::new(WidgetId::of("e"));
        render(&mut e, 10, 2);
        let (o, _) = e.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(o, Outcome::Consumed);
        // copy without a selection is consumed without an event
        let (o, ev) = e.on_key(&key(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!((o, ev), (Outcome::Consumed, None));
    }

    #[test]
    fn keyboard_selection_pauses_follow_and_survives_append_and_resize() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(30)).wrap(true);
        render(&mut v, 40, 5);
        assert!(v.follow);
        v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT));
        v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT | KeyModifiers::ALT));
        assert!(!v.follow, "selecting pauses the tail");
        assert_eq!(v.selected_text().as_deref(), Some("beta"));
        v.push(vec![Span::plain("appended")]);
        render(&mut v, 40, 5);
        assert_eq!(v.selected_text().as_deref(), Some("beta"));
        render(&mut v, 12, 5);
        assert_eq!(
            v.selected_text().as_deref(),
            Some("beta"),
            "wrapped rows keep identity"
        );
        let buf = render(&mut v, 40, 5);
        let sel = v.selection().unwrap();
        assert!(
            v.scroll
                .visible_range()
                .any(|vi| v.visual[vi].line == sel.1.line)
        );
        let _ = buf;
    }

    // ---------------------------------------------------------- marks

    #[test]
    fn marks_paint_and_survive_eviction() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["alpha", "beta", "gamma"] {
            v.push(vec![Span::plain(s)]);
        }
        v.set_marks(vec![
            Mark {
                line: 1,
                range: 0..2,
                current: true,
            },
            Mark {
                line: 2,
                range: 1..3,
                current: false,
            },
        ]);
        let buf = render(&mut v, 10, 3);
        assert!(buf[(0, 1)].modifier.contains(Modifier::REVERSED));
        assert!(buf[(1, 2)].modifier.contains(Modifier::UNDERLINED));
        v.push(vec![Span::plain("delta")]);
        assert_eq!(v.marks().len(), 2);
        assert_eq!(v.marks()[0].line, 0);
        v.reveal_line(2);
        assert!(!v.follow);
    }
}
