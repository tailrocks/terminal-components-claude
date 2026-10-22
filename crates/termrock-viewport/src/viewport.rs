use core::fmt;
use core::ops::{ControlFlow, Range};

#[cfg(feature = "testing")]
use std::cell::Cell;

use crate::theme::PaintStyle;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{Acc, PartStyle, SlotFn};
use crate::event::{Axis, Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::{FollowCursor, ScrollState, TrackClick};
use crate::text::Span;
use crate::text::measure::{grapheme_width, graphemes};
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Axes, Cx, FrameRead, Headroom, LayoutFacts, Ui};
use termrock_runtime::scroll_region::ScrollRegion;

const TAB_WIDTH: u16 = 4;

#[cfg(feature = "testing")]
#[derive(Debug, Default)]
pub struct ViewportWorkProbe {
    indexed_lines: Cell<usize>,
    visible_rows: Cell<usize>,
}

#[cfg(feature = "testing")]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ViewportWorkSnapshot {
    pub indexed_lines: usize,
    pub visible_rows: usize,
}

#[cfg(feature = "testing")]
impl ViewportWorkProbe {
    pub fn reset(&self) {
        self.indexed_lines.set(0);
        self.visible_rows.set(0);
    }

    pub fn snapshot(&self) -> ViewportWorkSnapshot {
        ViewportWorkSnapshot {
            indexed_lines: self.indexed_lines.get(),
            visible_rows: self.visible_rows.get(),
        }
    }

    fn add_indexed(&self, count: usize) {
        self.indexed_lines
            .set(self.indexed_lines.get().saturating_add(count));
    }

    fn add_visible(&self, count: usize) {
        self.visible_rows
            .set(self.visible_rows.get().saturating_add(count));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewportLine<'a> {
    Plain(&'a str),
    Spans(&'a [Span<'a>]),
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ProjectedText {
    text: String,
    lines: Vec<Range<usize>>,
    runs: Vec<ProjectedRun>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct ProjectedRun {
    text: Range<usize>,
    role: Option<Role>,
    modifier: Modifier,
}

impl ProjectedText {
    pub fn clear(&mut self) {
        self.text.clear();
        self.lines.clear();
        self.runs.clear();
    }

    pub fn push_line<I, S>(&mut self, runs: I)
    where
        I: IntoIterator<Item = (S, Option<Role>, Modifier)>,
        S: AsRef<str>,
    {
        let start = self.runs.len();
        for (value, role, modifier) in runs {
            let text_start = self.text.len();
            self.text.push_str(value.as_ref());
            self.runs.push(ProjectedRun {
                text: text_start..self.text.len(),
                role,
                modifier,
            });
        }
        self.lines.push(start..self.runs.len());
    }

    /// Widest line in terminal columns. Horizontal bars use this, not the
    /// visible window.
    pub fn widest_cols(&self) -> usize {
        (0..self.lines.len())
            .map(|index| line_cols(LineRef::Projected(self, index)))
            .max()
            .unwrap_or(0)
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[derive(Clone, Copy)]
enum LineRef<'a> {
    Borrowed(ViewportLine<'a>),
    Projected(&'a ProjectedText, usize),
}

impl<'a> From<ViewportLine<'a>> for LineRef<'a> {
    fn from(line: ViewportLine<'a>) -> Self {
        LineRef::Borrowed(line)
    }
}

#[derive(Clone, Copy)]
enum LineSet<'a> {
    Borrowed(&'a [ViewportLine<'a>]),
    Projected(&'a ProjectedText),
}

impl<'a> From<&'a [ViewportLine<'a>]> for LineSet<'a> {
    fn from(lines: &'a [ViewportLine<'a>]) -> Self {
        LineSet::Borrowed(lines)
    }
}

impl<'a, const N: usize> From<&'a [ViewportLine<'a>; N]> for LineSet<'a> {
    fn from(lines: &'a [ViewportLine<'a>; N]) -> Self {
        LineSet::Borrowed(lines)
    }
}

impl<'a> LineSet<'a> {
    fn len(self) -> usize {
        match self {
            LineSet::Borrowed(lines) => lines.len(),
            LineSet::Projected(text) => text.lines.len(),
        }
    }

    fn is_empty(self) -> bool {
        self.len() == 0
    }

    fn get(self, index: usize) -> Option<LineRef<'a>> {
        match self {
            LineSet::Borrowed(lines) => lines.get(index).copied().map(LineRef::Borrowed),
            LineSet::Projected(text) => text
                .lines
                .get(index)
                .map(|_| LineRef::Projected(text, index)),
        }
    }
}

impl<'a> From<&'a str> for ViewportLine<'a> {
    fn from(s: &'a str) -> Self {
        ViewportLine::Plain(s)
    }
}

impl<'a> From<&'a [Span<'a>]> for ViewportLine<'a> {
    fn from(s: &'a [Span<'a>]) -> Self {
        ViewportLine::Spans(s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct CellPos {
    pub line: usize,
    pub col: usize,
}

impl CellPos {
    pub const fn new(line: usize, col: usize) -> Self {
        CellPos { line, col }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ViewportAction {
    Copy(String),
    SelectionChanged,
    FollowChanged(bool),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewportCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    ToggleFollow,
    Copy,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: ViewportCmd,
    label: &'static str,
    visible: bool,
) -> Binding<ViewportCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 50 } else { 10 },
        visible,
    }
}

const BINDINGS: &[Binding<ViewportCmd>] = &[
    b(
        "viewport.up",
        Chord::key(KeyCode::Up),
        ViewportCmd::Up,
        "Up",
        true,
    ),
    b(
        "viewport.down",
        Chord::key(KeyCode::Down),
        ViewportCmd::Down,
        "Down",
        true,
    ),
    b(
        "viewport.up-vim",
        Chord::key(KeyCode::Char('k')),
        ViewportCmd::Up,
        "Up",
        false,
    ),
    b(
        "viewport.down-vim",
        Chord::key(KeyCode::Char('j')),
        ViewportCmd::Down,
        "Down",
        false,
    ),
    b(
        "viewport.page-up",
        Chord::key(KeyCode::PageUp),
        ViewportCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "viewport.page-down",
        Chord::key(KeyCode::PageDown),
        ViewportCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "viewport.home",
        Chord::key(KeyCode::Home),
        ViewportCmd::Home,
        "Top",
        false,
    ),
    b(
        "viewport.home-vim",
        Chord::key(KeyCode::Char('g')),
        ViewportCmd::Home,
        "Top",
        false,
    ),
    b(
        "viewport.end",
        Chord::key(KeyCode::End),
        ViewportCmd::End,
        "Tail",
        true,
    ),
    b(
        "viewport.end-vim",
        Chord::key(KeyCode::Char('G')),
        ViewportCmd::End,
        "Tail",
        false,
    ),
    b(
        "viewport.toggle-follow",
        Chord::key(KeyCode::Char('f')),
        ViewportCmd::ToggleFollow,
        "Follow",
        true,
    ),
    b(
        "viewport.copy",
        Chord::key(KeyCode::Char('y')),
        ViewportCmd::Copy,
        "Copy",
        true,
    ),
];

/// Semantic action for panning an opted-in, unwrapped [`TextViewport`] left.
/// Consumers can remap it on the viewport's owner with the runtime keymap.
pub const HORIZONTAL_PAN_LEFT: crate::ActionKey =
    crate::ActionKey::custom("viewport.horizontal-pan-left");

/// Semantic action for panning an opted-in, unwrapped [`TextViewport`] right.
/// Consumers can remap it on the viewport's owner with the runtime keymap.
pub const HORIZONTAL_PAN_RIGHT: crate::ActionKey =
    crate::ActionKey::custom("viewport.horizontal-pan-right");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HorizontalPanCmd {
    Left,
    Right,
}

const fn pan_binding(
    action: crate::ActionKey,
    chord: Option<Chord>,
    cmd: HorizontalPanCmd,
    label: &'static str,
    visible: bool,
) -> Binding<HorizontalPanCmd> {
    Binding {
        action,
        chord,
        cmd,
        label,
        priority: if visible { 50 } else { 10 },
        visible,
    }
}

const HORIZONTAL_PAN_BINDINGS: &[Binding<HorizontalPanCmd>] = &[
    pan_binding(
        HORIZONTAL_PAN_LEFT,
        Some(Chord::key(KeyCode::Left)),
        HorizontalPanCmd::Left,
        "Pan left",
        true,
    ),
    pan_binding(
        HORIZONTAL_PAN_RIGHT,
        Some(Chord::key(KeyCode::Right)),
        HorizontalPanCmd::Right,
        "Pan right",
        true,
    ),
];

const HORIZONTAL_PAN_SEMANTIC_BINDINGS: &[Binding<HorizontalPanCmd>] = &[
    pan_binding(HORIZONTAL_PAN_LEFT, None, HorizontalPanCmd::Left, "", false),
    pan_binding(
        HORIZONTAL_PAN_RIGHT,
        None,
        HorizontalPanCmd::Right,
        "",
        false,
    ),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SelectionCmd {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    WordLeft,
    WordRight,
    DocumentHome,
    DocumentEnd,
    Clear,
}

/// Shift+Left selection extension action, remappable on the viewport owner.
pub const SELECTION_LEFT: crate::ActionKey = crate::ActionKey::custom("viewport.selection-left");
/// Shift+Right selection extension action, remappable on the viewport owner.
pub const SELECTION_RIGHT: crate::ActionKey = crate::ActionKey::custom("viewport.selection-right");
/// Shift+Up selection extension action, remappable on the viewport owner.
pub const SELECTION_UP: crate::ActionKey = crate::ActionKey::custom("viewport.selection-up");
/// Shift+Down selection extension action, remappable on the viewport owner.
pub const SELECTION_DOWN: crate::ActionKey = crate::ActionKey::custom("viewport.selection-down");
/// Shift+Home selection extension action, remappable on the viewport owner.
pub const SELECTION_HOME: crate::ActionKey = crate::ActionKey::custom("viewport.selection-home");
/// Shift+End selection extension action, remappable on the viewport owner.
pub const SELECTION_END: crate::ActionKey = crate::ActionKey::custom("viewport.selection-end");
/// Ctrl+Shift+Left selection action that moves by word.
pub const SELECTION_WORD_LEFT_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-left-ctrl");
/// Ctrl+Shift+Right selection action that moves by word.
pub const SELECTION_WORD_RIGHT_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-right-ctrl");
/// Alt+Shift+Left selection action that moves by word.
pub const SELECTION_WORD_LEFT_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-left-alt");
/// Alt+Shift+Right selection action that moves by word.
pub const SELECTION_WORD_RIGHT_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-right-alt");
/// Ctrl+Shift+Up selection action that moves to the adjacent logical line.
pub const SELECTION_UP_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-up-ctrl");
/// Ctrl+Shift+Down selection action that moves to the adjacent logical line.
pub const SELECTION_DOWN_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-down-ctrl");
/// Alt+Shift+Up selection action that moves to the adjacent logical line.
pub const SELECTION_UP_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-up-alt");
/// Alt+Shift+Down selection action that moves to the adjacent logical line.
pub const SELECTION_DOWN_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-down-alt");
/// Ctrl+Shift+Home selection action that moves to the start of the document.
pub const SELECTION_DOCUMENT_HOME_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-home-ctrl");
/// Ctrl+Shift+End selection action that moves to the end of the document.
pub const SELECTION_DOCUMENT_END_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-end-ctrl");
/// Alt+Shift+Home selection action that moves to the start of the document.
pub const SELECTION_DOCUMENT_HOME_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-home-alt");
/// Alt+Shift+End selection action that moves to the end of the document.
pub const SELECTION_DOCUMENT_END_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-end-alt");
/// Escape action that clears an active selection.
pub const SELECTION_CLEAR: crate::ActionKey = crate::ActionKey::custom("viewport.selection-clear");

const fn selection_binding(
    action: crate::ActionKey,
    chord: Chord,
    cmd: SelectionCmd,
) -> Binding<SelectionCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label: "",
        priority: 10,
        visible: false,
    }
}

const SELECTION_BINDINGS: &[Binding<SelectionCmd>] = &[
    selection_binding(
        SELECTION_LEFT,
        Chord::with(KeyCode::Left, KeyModifiers::SHIFT),
        SelectionCmd::Left,
    ),
    selection_binding(
        SELECTION_RIGHT,
        Chord::with(KeyCode::Right, KeyModifiers::SHIFT),
        SelectionCmd::Right,
    ),
    selection_binding(
        SELECTION_UP,
        Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
        SelectionCmd::Up,
    ),
    selection_binding(
        SELECTION_DOWN,
        Chord::with(KeyCode::Down, KeyModifiers::SHIFT),
        SelectionCmd::Down,
    ),
    selection_binding(
        SELECTION_HOME,
        Chord::with(KeyCode::Home, KeyModifiers::SHIFT),
        SelectionCmd::Home,
    ),
    selection_binding(
        SELECTION_END,
        Chord::with(KeyCode::End, KeyModifiers::SHIFT),
        SelectionCmd::End,
    ),
    selection_binding(
        SELECTION_WORD_LEFT_CTRL,
        Chord::with(
            KeyCode::Left,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::WordLeft,
    ),
    selection_binding(
        SELECTION_WORD_RIGHT_CTRL,
        Chord::with(
            KeyCode::Right,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::WordRight,
    ),
    selection_binding(
        SELECTION_WORD_LEFT_ALT,
        Chord::with(KeyCode::Left, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::WordLeft,
    ),
    selection_binding(
        SELECTION_WORD_RIGHT_ALT,
        Chord::with(KeyCode::Right, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::WordRight,
    ),
    selection_binding(
        SELECTION_UP_CTRL,
        Chord::with(
            KeyCode::Up,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::Up,
    ),
    selection_binding(
        SELECTION_DOWN_CTRL,
        Chord::with(
            KeyCode::Down,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::Down,
    ),
    selection_binding(
        SELECTION_UP_ALT,
        Chord::with(KeyCode::Up, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::Up,
    ),
    selection_binding(
        SELECTION_DOWN_ALT,
        Chord::with(KeyCode::Down, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::Down,
    ),
    selection_binding(
        SELECTION_DOCUMENT_HOME_CTRL,
        Chord::with(
            KeyCode::Home,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::DocumentHome,
    ),
    selection_binding(
        SELECTION_DOCUMENT_END_CTRL,
        Chord::with(
            KeyCode::End,
            KeyModifiers::SHIFT.union(KeyModifiers::CONTROL),
        ),
        SelectionCmd::DocumentEnd,
    ),
    selection_binding(
        SELECTION_DOCUMENT_HOME_ALT,
        Chord::with(KeyCode::Home, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::DocumentHome,
    ),
    selection_binding(
        SELECTION_DOCUMENT_END_ALT,
        Chord::with(KeyCode::End, KeyModifiers::SHIFT.union(KeyModifiers::ALT)),
        SelectionCmd::DocumentEnd,
    ),
];

const SELECTION_CLEAR_BINDINGS: &[Binding<SelectionCmd>] = &[selection_binding(
    SELECTION_CLEAR,
    Chord::key(KeyCode::Esc),
    SelectionCmd::Clear,
)];

const HORIZONTAL_TRACK: Part = Part::custom("viewport.horizontal-track");
const HORIZONTAL_THUMB: Part = Part::custom("viewport.horizontal-thumb");

#[derive(Debug)]
struct VCell {
    span: usize,
    at: usize,
    len: usize,
    w: u16,
    tab: bool,
    /// A grapheme that crosses styled-run boundaries. Its style belongs to
    /// `span`, the run containing its first scalar.
    joined: Option<Box<str>>,
}

const fn runs(line: ViewportLine<'_>) -> &[Span<'_>] {
    match line {
        ViewportLine::Plain(_) => &[],
        ViewportLine::Spans(s) => s,
    }
}

fn run_count(line: LineRef<'_>) -> usize {
    match line {
        LineRef::Borrowed(ViewportLine::Plain(_)) => 1,
        LineRef::Borrowed(ViewportLine::Spans(spans)) => spans.len(),
        LineRef::Projected(text, line) => text.lines.get(line).map_or(0, Range::len),
    }
}

fn run_text(line: LineRef<'_>, ix: usize) -> &str {
    match line {
        LineRef::Borrowed(ViewportLine::Plain(s)) => s,
        LineRef::Borrowed(ViewportLine::Spans(sp)) => sp.get(ix).map_or("", |s| s.text),
        LineRef::Projected(text, line) => text
            .lines
            .get(line)
            .and_then(|line| text.runs.get(line.start.saturating_add(ix)))
            .and_then(|run| text.text.get(run.text.clone()))
            .unwrap_or(""),
    }
}

fn run_style(line: LineRef<'_>, ix: usize) -> (Option<Role>, Modifier) {
    match line {
        LineRef::Borrowed(line) => runs(line)
            .get(ix)
            .map_or((None, Modifier::empty()), |s| (s.role, s.add)),
        LineRef::Projected(text, line) => text
            .lines
            .get(line)
            .and_then(|line| text.runs.get(line.start.saturating_add(ix)))
            .map_or((None, Modifier::empty()), |run| (run.role, run.modifier)),
    }
}

fn cell_text<'line, 'cell>(line: impl Into<LineRef<'line>>, c: &'cell VCell) -> &'cell str
where
    'line: 'cell,
{
    let line = line.into();
    if c.tab {
        " "
    } else if let Some(joined) = &c.joined {
        joined
    } else {
        run_text(line, c.span)
            .get(c.at..c.at.saturating_add(c.len))
            .unwrap_or("")
    }
}

fn walk_text(line: LineRef<'_>, text: &str, f: &mut impl FnMut(VCell) -> ControlFlow<()>) {
    let count = run_count(line);
    if count == 0 {
        return;
    }
    let mut span = 0usize;
    let mut run_start = 0usize;
    let mut run_end = run_text(line, span).len();
    for (at, g) in graphemes(text) {
        while span + 1 < count && at >= run_end {
            span += 1;
            run_start = run_end;
            run_end = run_end.saturating_add(run_text(line, span).len());
        }
        let end = at.saturating_add(g.len());
        let run_at = at.saturating_sub(run_start);
        let joined = (end > run_end).then(|| Box::<str>::from(&text[at..end]));
        if g == "\t" {
            for _ in 0..TAB_WIDTH {
                if f(VCell {
                    span,
                    at: run_at,
                    len: g.len(),
                    w: 1,
                    tab: true,
                    joined: None,
                })
                .is_break()
                {
                    return;
                }
            }
            continue;
        }
        let w = grapheme_width(g);
        if w == 0 {
            continue;
        }
        if f(VCell {
            span,
            at: run_at,
            len: g.len(),
            w,
            tab: false,
            joined,
        })
        .is_break()
        {
            return;
        }
    }
}

fn walk_line<'a>(line: impl Into<LineRef<'a>>, mut f: impl FnMut(VCell) -> ControlFlow<()>) {
    let line = line.into();
    match line {
        LineRef::Borrowed(ViewportLine::Plain(text)) => walk_text(line, text, &mut f),
        LineRef::Borrowed(ViewportLine::Spans(spans)) => {
            if spans.len() <= 1 {
                walk_text(line, spans.first().map_or("", |span| span.text), &mut f);
            } else {
                let capacity = spans
                    .iter()
                    .fold(0usize, |size, span| size.saturating_add(span.text.len()));
                let mut joined = String::with_capacity(capacity);
                for span in spans {
                    joined.push_str(span.text);
                }
                walk_text(line, &joined, &mut f);
            }
        }
        LineRef::Projected(projected, index) => {
            let Some(runs) = projected.lines.get(index) else {
                return;
            };
            if runs.is_empty() {
                return;
            }
            let Some(first) = projected.runs.get(runs.start) else {
                return;
            };
            let Some(last) = runs
                .end
                .checked_sub(1)
                .and_then(|ix| projected.runs.get(ix))
            else {
                return;
            };
            if let Some(text) = projected.text.get(first.text.start..last.text.end) {
                walk_text(line, text, &mut f);
            }
        }
    }
}

fn line_cols<'a>(line: impl Into<LineRef<'a>>) -> usize {
    let line = line.into();
    let mut n = 0usize;
    walk_line(line, |c| {
        n = n.saturating_add(usize::from(c.w));
        ControlFlow::Continue(())
    });
    n
}

fn line_rows<'a>(line: impl Into<LineRef<'a>>, w: u16, wrap: bool) -> usize {
    let line = line.into();
    if !wrap || w == 0 {
        return 1;
    }
    let limit = usize::from(w);
    let mut rows = 1usize;
    let mut acc = 0usize;
    walk_line(line, |c| {
        let cw = usize::from(c.w);
        if acc > 0 && acc.saturating_add(cw) > limit {
            rows = rows.saturating_add(1);
            acc = cw;
        } else {
            acc = acc.saturating_add(cw);
        }
        ControlFlow::Continue(())
    });
    rows
}

fn row_cols<'a>(line: impl Into<LineRef<'a>>, w: u16, wrap: bool, row: usize) -> (usize, usize) {
    let line = line.into();
    if !wrap || w == 0 {
        return (0, usize::MAX);
    }
    let limit = usize::from(w);
    let mut r = 0usize;
    let mut acc = 0usize;
    let mut start = 0usize;
    let mut col = 0usize;
    let mut out: Option<(usize, usize)> = None;
    walk_line(line, |c| {
        let cw = usize::from(c.w);
        if acc > 0 && acc.saturating_add(cw) > limit {
            if r == row {
                out = Some((start, col));
                return ControlFlow::Break(());
            }
            r = r.saturating_add(1);
            start = col;
            acc = cw;
        } else {
            acc = acc.saturating_add(cw);
        }
        col = col.saturating_add(cw);
        ControlFlow::Continue(())
    });
    match out {
        Some(v) => v,
        None if r == row => (start, usize::MAX),
        None => (0, 0),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct LayoutKey {
    width: u16,
    wrap: bool,
    generation: u32,
    layout_revision: u64,
}

#[derive(Clone, Copy)]
struct SelectionLayout {
    key: LayoutKey,
    text_height: u16,
}

#[derive(Debug, Default)]
struct ViewportLayout {
    key: LayoutKey,
    prefix: Vec<usize>,
    lines: usize,
    widest_cols: usize,
    widest_valid: bool,
}

impl ViewportLayout {
    #[cfg(test)]
    fn ensure<'a>(&mut self, lines: impl Into<LineSet<'a>>, key: LayoutKey) -> usize {
        self.ensure_with_widest(lines, key, false)
    }

    fn ensure_with_widest<'a>(
        &mut self,
        lines: impl Into<LineSet<'a>>,
        key: LayoutKey,
        measure_widest: bool,
    ) -> usize {
        let lines = lines.into();
        let reusable = key.generation != u32::MAX && key.layout_revision != u64::MAX;
        if reusable
            && self.key == key
            && self.lines == lines.len()
            && (!measure_widest || self.widest_valid)
        {
            return 0;
        }
        let append = reusable && self.key == key && self.lines <= lines.len();
        if !append {
            self.key = key;
            self.prefix.clear();
            self.lines = 0;
            self.widest_cols = 0;
            self.widest_valid = false;
        }
        if !key.wrap {
            if measure_widest {
                let first = if self.widest_valid { self.lines } else { 0 };
                for index in first..lines.len() {
                    if let Some(line) = lines.get(index) {
                        self.widest_cols = self.widest_cols.max(line_cols(line));
                    }
                }
                self.widest_valid = true;
            } else if lines.len() != self.lines {
                self.widest_valid = false;
            }
            self.lines = lines.len();
            return 0;
        }
        if self.prefix.is_empty() {
            self.prefix.reserve(
                lines
                    .len()
                    .saturating_add(lines.len().checked_div(8).unwrap_or_default())
                    .saturating_add(1),
            );
            self.prefix.push(0);
        } else {
            self.prefix.reserve(
                lines
                    .len()
                    .saturating_add(1)
                    .saturating_sub(self.prefix.len()),
            );
        }
        let mut acc = self.prefix.last().copied().unwrap_or(0);
        let mut indexed = 0usize;
        for index in self.lines..lines.len() {
            let Some(line) = lines.get(index) else { break };
            let r = line_rows(line, key.width, true);
            acc = acc.saturating_add(r);
            self.prefix.push(acc);
            indexed = indexed.saturating_add(1);
        }
        self.lines = lines.len();
        indexed
    }

    fn total(&self) -> usize {
        if !self.key.wrap {
            return self.lines;
        }
        self.prefix.last().copied().unwrap_or(0)
    }

    fn widest_cols(&self) -> Option<usize> {
        self.widest_valid.then_some(self.widest_cols)
    }

    fn row_start<'a>(&self, lines: impl Into<LineSet<'a>>, row: usize) -> (usize, usize) {
        let lines = lines.into();
        if !self.key.wrap {
            return (row.min(lines.len().saturating_sub(1)), 0);
        }
        let prefix = self.prefix.as_slice();
        if prefix.len() < 2 {
            return (0, 0);
        }
        let target = row;
        let i = match prefix.binary_search(&target) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let i = i.min(prefix.len().saturating_sub(2));
        let base = prefix.get(i).copied().unwrap_or(0);
        (i, target.saturating_sub(base))
    }

    fn line_start(&self, line: usize) -> usize {
        if !self.key.wrap {
            return line.min(self.lines);
        }
        self.prefix.get(line).copied().unwrap_or(0)
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.'
}

fn word_at<'a>(line: impl Into<LineRef<'a>>, col: usize) -> Option<(usize, usize)> {
    let line = line.into();
    let mut cur: Option<usize> = None;
    let mut found: Option<(usize, usize)> = None;
    let mut at = 0usize;
    walk_line(line, |c| {
        let text = cell_text(line, &c);
        let word = !text.is_empty() && text.chars().all(is_word);
        if word {
            if cur.is_none() {
                cur = Some(at);
            }
        } else if let Some(s) = cur.take()
            && col >= s
            && col < at
        {
            found = Some((s, at));
            return ControlFlow::Break(());
        }
        at = at.saturating_add(usize::from(c.w));
        ControlFlow::Continue(())
    });
    if found.is_none()
        && let Some(s) = cur
        && col >= s
        && col < at
    {
        found = Some((s, at));
    }
    found
}

#[derive(Clone, Copy)]
struct LogicalCell {
    start: usize,
    end: usize,
    word: bool,
}

fn logical_cells(line: LineRef<'_>) -> Vec<LogicalCell> {
    let mut cells = Vec::new();
    let mut col = 0usize;
    walk_line(line, |cell| {
        let end = col.saturating_add(usize::from(cell.w));
        cells.push(LogicalCell {
            start: col,
            end,
            word: cell_text(line, &cell).chars().all(is_word),
        });
        col = end;
        ControlFlow::Continue(())
    });
    cells
}

fn snap_column(cells: &[LogicalCell], col: usize) -> usize {
    for cell in cells {
        if col < cell.end {
            return cell.start;
        }
    }
    cells.last().map_or(0, |cell| cell.end)
}

fn cell_at_column(cells: &[LogicalCell], col: usize) -> usize {
    cells
        .iter()
        .position(|cell| col < cell.end)
        .unwrap_or(cells.len())
}

fn clamp_position(lines: LineSet<'_>, pos: CellPos) -> Option<CellPos> {
    let line_index = pos.line.min(lines.len().checked_sub(1)?);
    let line = lines.get(line_index)?;
    let cells = logical_cells(line);
    Some(CellPos::new(line_index, snap_column(&cells, pos.col)))
}

fn word_range_at(lines: LineSet<'_>, pos: CellPos) -> Option<(CellPos, CellPos)> {
    let line = lines.get(pos.line)?;
    let (start, end) = word_at(line, pos.col)?;
    Some((CellPos::new(pos.line, start), CellPos::new(pos.line, end)))
}

fn logical_line_range(lines: LineSet<'_>, pos: CellPos) -> Option<(CellPos, CellPos)> {
    let line = lines.get(pos.line)?;
    Some((
        CellPos::new(pos.line, 0),
        CellPos::new(pos.line, line_cols(line)),
    ))
}

fn extend_word_range(
    original: (CellPos, CellPos),
    head: CellPos,
    target: Option<(CellPos, CellPos)>,
) -> (CellPos, CellPos) {
    let (start, end) = original;
    let (target_start, target_end) = target.unwrap_or((head, head));
    if target_start < start {
        (end, target_start)
    } else {
        (start, target_end.max(end))
    }
}

fn selection_destination(
    lines: LineSet<'_>,
    from: CellPos,
    command: SelectionCmd,
) -> Option<CellPos> {
    let last_line = lines.len().checked_sub(1)?;
    let line_index = from.line.min(last_line);
    let line = lines.get(line_index)?;
    let cells = logical_cells(line);
    let width = cells.last().map_or(0, |cell| cell.end);
    let col = snap_column(&cells, from.col);
    let index = cell_at_column(&cells, col);
    let at = |line: usize, col: usize| Some(CellPos::new(line, col));

    match command {
        SelectionCmd::Left | SelectionCmd::WordLeft => {
            if index == 0 {
                if line_index > 0 {
                    let previous = logical_cells(lines.get(line_index - 1)?);
                    return at(line_index - 1, previous.last().map_or(0, |cell| cell.end));
                }
                return at(line_index, col);
            }
            let target = if command == SelectionCmd::Left {
                index - 1
            } else {
                let mut target = index;
                while target > 0 && !cells[target - 1].word {
                    target -= 1;
                }
                while target > 0 && cells[target - 1].word {
                    target -= 1;
                }
                target
            };
            at(line_index, cells[target].start)
        }
        SelectionCmd::Right | SelectionCmd::WordRight => {
            if index >= cells.len() {
                if line_index < last_line {
                    return at(line_index + 1, 0);
                }
                return at(line_index, col);
            }
            let target = if command == SelectionCmd::Right {
                index + 1
            } else {
                let mut target = index;
                while target < cells.len() && !cells[target].word {
                    target += 1;
                }
                while target < cells.len() && cells[target].word {
                    target += 1;
                }
                target
            };
            at(
                line_index,
                cells.get(target).map_or(width, |cell| cell.start),
            )
        }
        SelectionCmd::Up => {
            if line_index == 0 {
                return at(0, 0);
            }
            let previous = logical_cells(lines.get(line_index - 1)?);
            let previous_width = previous.last().map_or(0, |cell| cell.end);
            at(
                line_index - 1,
                snap_column(&previous, col.min(previous_width)),
            )
        }
        SelectionCmd::Down => {
            if line_index == last_line {
                return at(line_index, width);
            }
            let next = logical_cells(lines.get(line_index + 1)?);
            let next_width = next.last().map_or(0, |cell| cell.end);
            at(line_index + 1, snap_column(&next, col.min(next_width)))
        }
        SelectionCmd::Home => at(line_index, 0),
        SelectionCmd::End => at(line_index, width),
        SelectionCmd::DocumentHome => at(0, 0),
        SelectionCmd::DocumentEnd => {
            let last = logical_cells(lines.get(last_line)?);
            at(last_line, last.last().map_or(0, |cell| cell.end))
        }
        SelectionCmd::Clear => None,
    }
}

fn visual_row_for_position(
    layout: &ViewportLayout,
    lines: LineSet<'_>,
    key: LayoutKey,
    pos: CellPos,
) -> Option<usize> {
    let line = lines.get(pos.line)?;
    let rows = line_rows(line, key.width, key.wrap).max(1);
    for row in 0..rows {
        let (start, end) = row_cols(line, key.width, key.wrap, row);
        if pos.col >= start && (pos.col < end || (row + 1 == rows && pos.col <= end)) {
            return Some(layout.line_start(pos.line).saturating_add(row));
        }
    }
    Some(
        layout
            .line_start(pos.line)
            .saturating_add(rows.saturating_sub(1)),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewportState {
    scroll: ScrollState,
    horizontal: ScrollState,
    follow: FollowCursor,
    selection: Option<(CellPos, CellPos)>,
    anchor: Option<CellPos>,
    pointer_gesture: PointerGesture,
    browse: Option<CellPos>,
    caret: Option<CellPos>,
    generation: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum PointerGesture {
    #[default]
    Idle,
    Plain {
        dragged: bool,
    },
    Word {
        range: (CellPos, CellPos),
        dragged: bool,
    },
    Shift {
        dragged: bool,
    },
    Line {
        dragged: bool,
    },
    SuppressClick,
}

impl PointerGesture {
    const fn dragged(self) -> bool {
        matches!(
            self,
            PointerGesture::Plain { dragged: true }
                | PointerGesture::Word { dragged: true, .. }
                | PointerGesture::Shift { dragged: true }
                | PointerGesture::Line { dragged: true }
        )
    }

    fn mark_dragged(&mut self) {
        match self {
            PointerGesture::Plain { dragged }
            | PointerGesture::Word { dragged, .. }
            | PointerGesture::Shift { dragged }
            | PointerGesture::Line { dragged } => *dragged = true,
            PointerGesture::Idle | PointerGesture::SuppressClick => {}
        }
    }

    fn retain(&mut self, dropped: usize) -> bool {
        let PointerGesture::Word { range, .. } = self else {
            return false;
        };
        if range.0.line < dropped && range.1.line < dropped {
            *self = PointerGesture::Idle;
            true
        } else {
            shift(&mut range.0, dropped);
            shift(&mut range.1, dropped);
            false
        }
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        ViewportState {
            scroll: ScrollState::default(),
            horizontal: ScrollState::default(),
            follow: FollowCursor::Always,
            selection: None,
            anchor: None,
            pointer_gesture: PointerGesture::Idle,
            browse: None,
            caret: None,
            generation: 0,
        }
    }
}

impl ViewportState {
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    /// The current horizontal display-cell offset of an opted-in viewport.
    ///
    /// It stays zero when horizontal panning is disabled or wrapping is on.
    pub const fn horizontal_offset(&self) -> usize {
        self.horizontal.offset()
    }

    /// Reset the vertical and horizontal offsets to the leading edge.
    ///
    /// Consumers that replace the displayed source can use this before the
    /// next draw so the new source is never painted at the previous position.
    pub fn reset_scroll_offsets(&mut self) {
        self.scroll.jump_start();
        self.scroll.release_track();
        self.horizontal.jump_start();
        self.horizontal.release_track();
    }

    pub const fn follow(&self) -> bool {
        match self.follow {
            FollowCursor::Always => true,
            FollowCursor::OnMove => false,
        }
    }

    pub const fn set_follow(&mut self, on: bool) {
        self.follow = if on {
            FollowCursor::Always
        } else {
            FollowCursor::OnMove
        };
    }

    /// How the view follows the end of its output.
    pub const fn follow_cursor(&self) -> FollowCursor {
        self.follow
    }

    /// Set how the view follows the end of its output.
    pub const fn set_follow_cursor(&mut self, follow: FollowCursor) {
        self.follow = follow;
    }

    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        let (a, b) = self.selection?;
        if a == b {
            return None;
        }
        Some((a.min(b), a.max(b)))
    }

    pub const fn select(&mut self, a: CellPos, b: CellPos) {
        self.selection = Some((a, b));
        self.anchor = None;
        self.pointer_gesture = PointerGesture::Idle;
        self.browse = Some(b);
    }

    pub const fn clear_selection(&mut self) -> bool {
        let had = self.selection.is_some();
        self.selection = None;
        self.anchor = None;
        self.pointer_gesture = PointerGesture::Idle;
        self.browse = None;
        had
    }

    pub const fn caret(&self) -> Option<CellPos> {
        self.caret
    }

    pub const fn set_caret(&mut self, c: Option<CellPos>) {
        self.caret = c;
    }

    pub fn retained(&mut self, dropped: usize) {
        if dropped == 0 {
            return;
        }
        let selection_dropped = self
            .selection
            .is_some_and(|(a, b)| a.line < dropped && b.line < dropped);
        if selection_dropped {
            self.selection = None;
            self.browse = None;
        } else if let Some((a, b)) = self.selection.as_mut() {
            shift(a, dropped);
            shift(b, dropped);
        }
        let word_gesture_dropped = self.pointer_gesture.retain(dropped);
        if selection_dropped && matches!(self.pointer_gesture, PointerGesture::Shift { .. }) {
            self.pointer_gesture = PointerGesture::Idle;
            self.anchor = None;
        } else if word_gesture_dropped {
            self.anchor = None;
        } else if let Some(anchor) = self.anchor.as_mut() {
            shift(anchor, dropped);
        }
        if !selection_dropped && let Some(b) = self.browse.as_mut() {
            shift(b, dropped);
        }
        if let Some(c) = self.caret.as_mut() {
            shift(c, dropped);
        }
        self.invalidate();
    }

    pub const fn invalidate(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }

    pub fn copy_into(&self, lines: &[ViewportLine<'_>], out: &mut String) -> bool {
        self.copy_from(LineSet::Borrowed(lines), out)
    }

    pub fn copy_from_projected(&self, text: &ProjectedText, out: &mut String) -> bool {
        self.copy_from(LineSet::Projected(text), out)
    }

    fn copy_from(&self, lines: LineSet<'_>, out: &mut String) -> bool {
        let Some((a, b)) = self.selection() else {
            return false;
        };
        let mut first = true;
        for li in a.line..=b.line {
            let Some(line) = lines.get(li) else { break };
            if !first {
                out.push('\n');
            }
            first = false;
            let from = if li == a.line { a.col } else { 0 };
            let to = if li == b.line { b.col } else { usize::MAX };
            let start = out.len();
            let mut col = 0usize;
            walk_line(line, |c| {
                if col >= to {
                    return ControlFlow::Break(());
                }
                if col >= from {
                    out.push_str(cell_text(line, &c));
                }
                col = col.saturating_add(usize::from(c.w));
                ControlFlow::Continue(())
            });
            let kept = out.get(start..).unwrap_or("").trim_end().len();
            out.truncate(start.saturating_add(kept));
        }
        true
    }
}

fn shift(p: &mut CellPos, dropped: usize) {
    if p.line < dropped {
        *p = CellPos::new(0, 0);
    } else {
        p.line = p.line.saturating_sub(dropped);
    }
}

pub struct TextViewport<'a> {
    id: Id,
    wrap: bool,
    horizontal_pan: bool,
    layout_revision: u64,
    /// When set, column 0 is the focus gutter and text starts one cell in.
    /// Terminal panes that already draw their prompt turn this off.
    reserve_gutter: bool,
    /// Blank columns kept between the text and the vertical bar. Baseline
    /// terminals reserve one.
    scroll_gap: u16,
    caret_when_unfocused: bool,
    /// Steady bar cursor. The default viewport caret is a block.
    bar_cursor: bool,
    click_selects_word: bool,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
    #[cfg(feature = "testing")]
    work_probe: Option<&'a ViewportWorkProbe>,
}

impl fmt::Debug for TextViewport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextViewport")
            .field("id", &self.id)
            .field("wrap", &self.wrap)
            .field("horizontal_pan", &self.horizontal_pan)
            .field("layout_revision", &self.layout_revision)
            .field("reserve_gutter", &self.reserve_gutter)
            .field("scroll_gap", &self.scroll_gap)
            .field("caret_when_unfocused", &self.caret_when_unfocused)
            .field("bar_cursor", &self.bar_cursor)
            .field("click_selects_word", &self.click_selects_word)
            .field("slot", &self.slot.map(|(p, _)| p))
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PanDimensions {
    text_width: u16,
    text_height: u16,
    horizontal_overflow: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct RowPosition {
    line: usize,
    wrapped_row: usize,
    horizontal_offset: usize,
}

impl<'a> TextViewport<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TEXT,
        Part::GUTTER,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const PREFERRED_TEXT_WIDTH: u16 = 20;

    pub const fn new(id: Id) -> Self {
        TextViewport {
            id,
            wrap: false,
            horizontal_pan: false,
            layout_revision: 0,
            reserve_gutter: true,
            scroll_gap: 0,
            caret_when_unfocused: false,
            bar_cursor: false,
            click_selects_word: false,
            patch: None,
            parts: &[],
            slot: None,
            #[cfg(feature = "testing")]
            work_probe: None,
        }
    }

    #[cfg(feature = "testing")]
    #[must_use]
    pub const fn work_probe(mut self, probe: &'a ViewportWorkProbe) -> Self {
        self.work_probe = Some(probe);
        self
    }

    #[cfg(feature = "testing")]
    fn note_indexed(&self, count: usize) {
        if let Some(probe) = self.work_probe {
            probe.add_indexed(count);
        }
    }

    #[cfg(not(feature = "testing"))]
    fn note_indexed(count: usize) {
        let _ = count;
    }

    #[cfg(feature = "testing")]
    fn note_visible(&self, count: usize) {
        if let Some(probe) = self.work_probe {
            probe.add_visible(count);
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn wrap(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    /// Allow display-cell horizontal panning when wrapping is disabled.
    ///
    /// The default is off, preserving the original viewport bindings and
    /// rendered cells. Pan actions use plain Left/Right only when no producer
    /// caret owns those keys; consumers can remap the public semantic action
    /// keys without installing a raw-key fallback.
    #[must_use]
    pub const fn horizontal_pan(mut self, enabled: bool) -> Self {
        self.horizontal_pan = enabled;
        self
    }

    /// Distinguish changed borrowed content that keeps the same viewport
    /// state generation and dimensions. Saturation disables layout reuse.
    #[must_use]
    pub const fn layout_revision(mut self, revision: u64) -> Self {
        self.layout_revision = revision;
        self
    }

    /// Keep a focus column at the left of the text. Capsule terminals leave
    /// this off so the prompt sits against the pane frame.
    #[must_use]
    pub const fn reserve_gutter(mut self, yes: bool) -> Self {
        self.reserve_gutter = yes;
        self
    }

    /// Reserve blank columns between the text and the vertical bar.
    #[must_use]
    pub const fn scroll_gap(mut self, cols: u16) -> Self {
        self.scroll_gap = cols;
        self
    }

    /// Allows this viewport to publish its state caret while it lacks
    /// keyboard focus. The caret remains subject to follow, selection,
    /// viewport and clipping rules; this setting does not make the viewport
    /// editable or change its focus styles.
    #[must_use]
    pub const fn caret_when_unfocused(mut self, enabled: bool) -> Self {
        self.caret_when_unfocused = enabled;
        self
    }

    /// Draw the caret as a steady bar. Terminal panes use this shape.
    #[must_use]
    pub const fn bar_cursor(mut self, yes: bool) -> Self {
        self.bar_cursor = yes;
        self
    }

    /// Select a word on a single click when enabled.
    ///
    /// A double click selects a word and three or more clicks select the
    /// logical line, regardless of this setting.
    #[must_use]
    pub const fn click_selects_word(mut self, yes: bool) -> Self {
        self.click_selects_word = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.slot = Some((p, f));
        self
    }

    fn overrides(&self) -> PartStyle<'a> {
        let mut ov = PartStyle::new().part(self.parts);
        if let Some(p) = self.patch {
            ov = ov.global(p);
        }
        if let Some((p, f)) = self.slot {
            ov = ov.slot(p, f);
        }
        ov
    }

    fn bar(&self) -> ScrollRegion<'a> {
        let mut sr = ScrollRegion::new(self.id)
            .inherit_family(Family::VIEWPORT)
            .patch_part(self.parts);
        if let Some(p) = self.patch {
            sr = sr.patch(p);
        }
        if let Some((p, f)) = self.slot {
            sr = sr.slot(p, f);
        }
        sr
    }

    fn text_width(width: u16, gutter: bool, gap: u16) -> u16 {
        let reserved = 1u16.saturating_add(u16::from(gutter)).saturating_add(gap);
        width.saturating_sub(reserved)
    }

    fn text_area(&self, area: Rect, width: u16) -> Rect {
        let gutter = u16::from(self.reserve_gutter);
        Rect {
            x: area.x.saturating_add(gutter),
            width: width.min(area.width.saturating_sub(gutter)),
            ..area
        }
    }

    const fn pan_active(&self) -> bool {
        self.horizontal_pan && !self.wrap
    }

    fn pan_dimensions(&self, area: Rect, content_rows: usize, widest_cols: usize) -> PanDimensions {
        let mut vertical = false;
        let mut horizontal = false;
        for _ in 0..3 {
            let rows = area
                .height
                .saturating_sub(u16::from(horizontal && area.height > 1));
            let next_vertical = content_rows > usize::from(rows);
            let text_width =
                if self.scroll_gap > 0 && !next_vertical && content_rows <= usize::from(rows) {
                    area.width.saturating_sub(u16::from(self.reserve_gutter))
                } else {
                    Self::text_width(area.width, self.reserve_gutter, self.scroll_gap)
                };
            let next_horizontal = widest_cols > usize::from(text_width);
            if next_vertical == vertical && next_horizontal == horizontal {
                break;
            }
            vertical |= next_vertical;
            horizontal |= next_horizontal;
        }

        let horizontal_bar = horizontal && area.height > 1;
        let text_height = area.height.saturating_sub(u16::from(horizontal_bar));
        let vertical_overflow = content_rows > usize::from(text_height);
        let text_width = if self.scroll_gap > 0
            && !vertical_overflow
            && content_rows <= usize::from(text_height)
        {
            area.width.saturating_sub(u16::from(self.reserve_gutter))
        } else {
            Self::text_width(area.width, self.reserve_gutter, self.scroll_gap)
        };

        PanDimensions {
            text_width,
            text_height,
            horizontal_overflow: widest_cols > usize::from(text_width),
        }
    }

    const fn layout_key(&self, st: &ViewportState, width: u16) -> LayoutKey {
        LayoutKey {
            width,
            wrap: self.wrap,
            generation: st.generation,
            layout_revision: self.layout_revision,
        }
    }

    fn pos_at(
        &self,
        cx: &mut Cx<'_>,
        lines: LineSet<'_>,
        geometry: (Rect, LayoutKey, usize, usize),
        pos: Position,
    ) -> Option<CellPos> {
        let (area, key, offset, horizontal_offset) = geometry;
        if area.is_empty() || lines.is_empty() {
            return None;
        }
        let layout = cx.cache::<ViewportLayout>(self.id);
        let indexed = layout.ensure_with_widest(lines, key, self.pan_active());
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dy = usize::from(pos.y.saturating_sub(area.y));
        let visual = offset.saturating_add(dy);
        let (li, row) = layout.row_start(lines, visual);
        let line = lines.get(li)?;
        let (start, end) = row_cols(line, key.width, self.wrap, row);
        let dx = usize::from(pos.x.saturating_sub(area.x));
        let last = end.min(line_cols(line));
        let col = if self.pan_active() {
            start.saturating_add(horizontal_offset).saturating_add(dx)
        } else {
            start.saturating_add(dx)
        };
        Some(CellPos {
            line: li,
            col: col.min(last),
        })
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
    ) -> Response<ViewportAction> {
        self.update_lines(cx, st, LineSet::Borrowed(lines))
    }

    pub fn update_projected(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &ProjectedText,
    ) -> Response<ViewportAction> {
        self.update_lines(cx, st, LineSet::Projected(lines))
    }

    fn update_lines(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
    ) -> Response<ViewportAction> {
        if lines.is_empty() {
            st.anchor = None;
            st.pointer_gesture = PointerGesture::Idle;
        }
        let layout = cx.layout(self.id);
        let area = layout
            .and_then(|layout| layout.logical_area)
            .or_else(|| cx.area(self.id))
            .unwrap_or(Rect::ZERO);
        let fallback_width = layout.map_or_else(
            || Self::text_width(area.width, self.reserve_gutter, self.scroll_gap),
            |layout| layout.cols,
        );
        let initial_key = self.layout_key(st, fallback_width);
        let (total, widest, indexed) = {
            let viewport_layout = cx.cache::<ViewportLayout>(self.id);
            let indexed = viewport_layout.ensure_with_widest(lines, initial_key, self.pan_active());
            (
                viewport_layout.total(),
                viewport_layout.widest_cols().unwrap_or(0),
                indexed,
            )
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            self.pan_dimensions(area, total, widest)
        } else {
            PanDimensions {
                text_width: fallback_width,
                text_height: area.height,
                horizontal_overflow: false,
            }
        };
        let width = dimensions.text_width;
        let mut logical_text = self.text_area(area, width);
        logical_text.height = dimensions.text_height;
        let hit_text = cx
            .area_of_part(self.id, PartRef::of(Part::TEXT))
            .unwrap_or(logical_text);
        // The registry clips the hit area. Keep its horizontal visible origin for pointer
        // mapping, while retaining the logical row origin and wrapping width from draw.
        let pointer_text = Rect {
            x: hit_text.x,
            y: logical_text.y,
            ..logical_text
        };
        let key = self.layout_key(st, width);
        let (total, indexed) = {
            let viewport_layout = cx.cache::<ViewportLayout>(self.id);
            let indexed = viewport_layout.ensure_with_widest(lines, key, self.pan_active());
            (viewport_layout.total(), indexed)
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            let widest = cx
                .cache::<ViewportLayout>(self.id)
                .widest_cols()
                .unwrap_or(0);
            self.pan_dimensions(area, total, widest)
        } else {
            dimensions
        };
        st.scroll
            .apply_layout(usize::from(dimensions.text_height), total);
        st.horizontal
            .set_content(if self.pan_active() { widest } else { 0 });
        st.horizontal
            .set_viewport(usize::from(dimensions.text_width));
        let before = st.scroll.offset();
        let mut acc = Acc::<ViewportAction>::new();
        let mut selection_moved = false;
        let bar = self.bar();
        let track_len = bar.prepare(cx, &mut st.scroll, total);
        for it in cx.intents(self.id) {
            let horizontal_wheel = matches!(it, Intent::Wheel { axis: Axis::H, .. });
            if !horizontal_wheel || !self.pan_active() {
                acc.fold(&bar.handle_intent(cx, &mut st.scroll, track_len, it));
            }
            match it {
                Intent::Binding(action) => {
                    if let Some(cmd) = Binding::command(BINDINGS, action) {
                        Self::command(st, lines, cmd, &mut acc);
                    } else if self.pan_active()
                        && let Some(cmd) = Binding::command(HORIZONTAL_PAN_BINDINGS, action)
                            .or_else(|| Binding::command(HORIZONTAL_PAN_SEMANTIC_BINDINGS, action))
                    {
                        Self::horizontal_command(st, cmd, dimensions.horizontal_overflow, &mut acc);
                    } else if let Some(cmd) = Binding::command(SELECTION_BINDINGS, action)
                        .or_else(|| Binding::command(SELECTION_CLEAR_BINDINGS, action))
                    {
                        selection_moved |= self.selection_command(
                            cx,
                            st,
                            lines,
                            SelectionLayout {
                                key,
                                text_height: dimensions.text_height,
                            },
                            cmd,
                            &mut acc,
                        );
                    }
                }
                Intent::Wheel {
                    axis: Axis::H,
                    delta,
                    ..
                } if self.pan_active() && dimensions.horizontal_overflow => {
                    let horizontal_before = st.horizontal.offset();
                    st.horizontal
                        .scroll_by(isize::from(delta.signum()).saturating_mul(4));
                    if horizontal_before == st.horizontal.offset() {
                        acc.consumed();
                    } else {
                        acc.changed();
                    }
                }
                Intent::Pointer { part, .. }
                    if self.pan_active()
                        && (part.part == HORIZONTAL_TRACK || part.part == HORIZONTAL_THUMB) =>
                {
                    self.handle_horizontal_pointer(
                        cx,
                        st,
                        self.horizontal_track(area, dimensions),
                        it,
                        &mut acc,
                    );
                }
                Intent::Cancel => {
                    st.pointer_gesture = PointerGesture::SuppressClick;
                    st.anchor = None;
                    st.horizontal.release_track();
                    if cx.capture_owner() == Some(self.id) {
                        cx.release_capture();
                    }
                }
                Intent::Pointer {
                    part,
                    phase,
                    pos,
                    mods,
                    ..
                } if part.part == Part::TEXT => {
                    self.handle_text_pointer(
                        cx,
                        st,
                        lines,
                        (area, pointer_text, hit_text, key, st.horizontal.offset()),
                        (phase, part, pos, mods),
                        &mut acc,
                    );
                }
                _ => {}
            }
        }
        if !selection_moved {
            if st.scroll.offset() != before {
                st.follow = if st.scroll.at_end() {
                    FollowCursor::Always
                } else {
                    FollowCursor::OnMove
                };
            }
            if st.follow == FollowCursor::Always {
                st.scroll.jump_end();
            }
        }
        acc.finish(self.id)
    }

    fn horizontal_track(&self, area: Rect, dimensions: PanDimensions) -> Rect {
        if !self.pan_active() || !dimensions.horizontal_overflow || area.height <= 1 {
            return Rect::ZERO;
        }
        Rect::new(
            area.x.saturating_add(u16::from(self.reserve_gutter)),
            area.y.saturating_add(dimensions.text_height),
            dimensions.text_width,
            1,
        )
    }

    fn horizontal_command(
        st: &mut ViewportState,
        cmd: HorizontalPanCmd,
        overflows: bool,
        acc: &mut Acc<ViewportAction>,
    ) {
        if !overflows {
            acc.consumed();
            return;
        }
        let before = st.horizontal.offset();
        match cmd {
            HorizontalPanCmd::Left => st.horizontal.scroll_by(-4),
            HorizontalPanCmd::Right => st.horizontal.scroll_by(4),
        }
        if before == st.horizontal.offset() {
            acc.consumed();
        } else {
            acc.changed();
        }
    }

    fn selection_command(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        layout: SelectionLayout,
        cmd: SelectionCmd,
        acc: &mut Acc<ViewportAction>,
    ) -> bool {
        let SelectionLayout { key, text_height } = layout;
        if cmd == SelectionCmd::Clear {
            if st.clear_selection() {
                acc.action(ViewportAction::SelectionChanged);
            }
            return false;
        }
        let previous_selection = st.selection;
        let previous_browse = st.browse;
        let previous_head = previous_selection.map(|(_, head)| head).or(previous_browse);
        if lines.is_empty() {
            st.selection = None;
            st.browse = None;
            if previous_head.is_some() {
                st.follow = FollowCursor::OnMove;
            }
            if previous_selection.is_some() {
                acc.action(ViewportAction::SelectionChanged);
            } else if previous_browse.is_some() {
                acc.changed();
            } else {
                acc.consumed();
            }
            return previous_head.is_some();
        }

        let selection = previous_selection.and_then(|(anchor, head)| {
            Some((clamp_position(lines, anchor)?, clamp_position(lines, head)?))
        });
        let browse = if let Some((_, head)) = selection {
            Some(head)
        } else {
            previous_browse.and_then(|pos| clamp_position(lines, pos))
        };
        st.selection = selection;
        st.browse = browse;
        let selection_changed = previous_selection != st.selection;
        let browse_changed = previous_browse != st.browse;
        let reconciled_head = st.selection.map(|(_, head)| head).or(st.browse);
        let head_changed = previous_head != reconciled_head;

        let current = reconciled_head;
        let from = current.or_else(|| {
            let layout = cx.cache::<ViewportLayout>(self.id);
            let line = if text_height > 0 && layout.total() > 0 {
                let last_visible = st
                    .scroll
                    .offset()
                    .saturating_add(usize::from(text_height).saturating_sub(1))
                    .min(layout.total().saturating_sub(1));
                layout.row_start(lines, last_visible).0
            } else {
                lines.len().saturating_sub(1)
            };
            let line_ref = lines.get(line)?;
            Some(CellPos::new(line, line_cols(line_ref)))
        });
        let Some(from) = from else {
            if selection_changed {
                acc.action(ViewportAction::SelectionChanged);
            } else if browse_changed {
                acc.changed();
            } else {
                acc.consumed();
            }
            if head_changed {
                st.follow = FollowCursor::OnMove;
                return true;
            }
            return false;
        };
        let anchor = st.selection.map_or(from, |(anchor, _)| anchor);
        let Some(to) = selection_destination(lines, from, cmd) else {
            if selection_changed {
                acc.action(ViewportAction::SelectionChanged);
            } else if browse_changed {
                acc.changed();
            } else {
                acc.consumed();
            }
            if head_changed {
                st.follow = FollowCursor::OnMove;
                return true;
            }
            return false;
        };
        let row = {
            let layout = cx.cache::<ViewportLayout>(self.id);
            visual_row_for_position(layout, lines, key, to)
        };

        if to == from {
            if selection_changed {
                acc.action(ViewportAction::SelectionChanged);
            } else if browse_changed {
                acc.changed();
            } else {
                acc.consumed();
            }
            if head_changed {
                if let Some(row) = row {
                    st.scroll.ensure_visible(row);
                }
                if self.pan_active() {
                    st.horizontal.ensure_visible(to.col);
                }
                st.follow = FollowCursor::OnMove;
                return true;
            }
            return false;
        }

        st.selection = Some((anchor, to));
        st.anchor = None;
        st.browse = Some(to);
        if let Some(row) = row {
            st.scroll.ensure_visible(row);
        }
        if self.pan_active() {
            st.horizontal.ensure_visible(to.col);
        }
        st.follow = FollowCursor::OnMove;
        acc.action(ViewportAction::SelectionChanged);
        true
    }

    fn handle_horizontal_pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        track: Rect,
        intent: Intent<'_>,
        acc: &mut Acc<ViewportAction>,
    ) {
        let Intent::Pointer {
            phase,
            part,
            pos,
            local,
            ..
        } = intent
        else {
            return;
        };
        if track.is_empty() {
            st.horizontal.release_track();
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            acc.consumed();
            return;
        }

        let before = st.horizontal.offset();
        match phase {
            Phase::Press if part.part == HORIZONTAL_THUMB => {
                let track_pos = usize::from(pos.x.saturating_sub(track.x));
                st.horizontal
                    .press_track(track_pos, usize::from(track.width));
                if !cx.capture(self.id, PartRef::of(HORIZONTAL_THUMB)) {
                    st.horizontal.release_track();
                }
                acc.consumed();
            }
            Phase::Press => {
                let track_pos = usize::from(local.x);
                st.horizontal.press_track_with(
                    track_pos,
                    usize::from(track.width),
                    TrackClick::Page,
                );
                if before == st.horizontal.offset() {
                    acc.consumed();
                } else {
                    acc.changed();
                }
            }
            Phase::Drag => {
                let track_pos = if cx.capture_owner() == Some(self.id) {
                    let origin = cx.capture_origin().unwrap_or(pos);
                    let capture_area = cx.capture_area().unwrap_or(track);
                    let grab = origin.x.saturating_sub(capture_area.x);
                    let (_, thumb_len) = st.horizontal.thumb(usize::from(track.width));
                    usize::from(pos.x.saturating_sub(track.x).saturating_sub(grab))
                        .saturating_add(thumb_len / 2)
                } else {
                    usize::from(pos.x.saturating_sub(track.x))
                };
                st.horizontal.scroll_to(
                    st.horizontal
                        .offset_for_track_pos(track_pos, usize::from(track.width)),
                );
                if before == st.horizontal.offset() {
                    acc.consumed();
                } else {
                    acc.changed();
                }
            }
            Phase::Release | Phase::DragEnd => {
                st.horizontal.release_track();
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                acc.consumed();
            }
            _ => acc.consumed(),
        }
    }

    fn handle_text_pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        geometry: (Rect, Rect, Rect, LayoutKey, usize),
        pointer: (Phase, PartRef, Position, KeyModifiers),
        acc: &mut Acc<ViewportAction>,
    ) {
        let (area, logical_text, hit_text, key, mut horizontal_offset) = geometry;
        let (phase, part, pos, mods) = pointer;
        match phase {
            Phase::Press | Phase::DragStart => {
                cx.focus(self.id);
                let _ = cx.capture(self.id, PartRef::of(Part::TEXT));
                let at = self.pos_at(
                    cx,
                    lines,
                    (logical_text, key, st.scroll.offset(), horizontal_offset),
                    pos,
                );
                let Some(at) = at else {
                    let had = st.selection.take().is_some();
                    st.browse = None;
                    st.anchor = None;
                    st.pointer_gesture = PointerGesture::SuppressClick;
                    if had {
                        acc.action(ViewportAction::SelectionChanged);
                    } else {
                        acc.consumed();
                    }
                    return;
                };

                if mods.contains(KeyModifiers::SHIFT) {
                    if let Some(anchor) = st.selection.map(|(anchor, _)| anchor).or(st.anchor) {
                        st.selection = Some((anchor, at));
                        st.browse = Some(at);
                        st.anchor = Some(anchor);
                        st.pointer_gesture = PointerGesture::Shift { dragged: false };
                        acc.action(ViewportAction::SelectionChanged);
                    } else {
                        st.pointer_gesture = PointerGesture::SuppressClick;
                        acc.consumed();
                    }
                } else {
                    st.browse = None;
                    let count = cx.pointer_press_count(self.id, part).unwrap_or(1);
                    if count >= 3 {
                        if let Some((start, end)) = logical_line_range(lines, at) {
                            st.select(start, end);
                            st.pointer_gesture = PointerGesture::Line { dragged: false };
                            acc.action(ViewportAction::SelectionChanged);
                        } else {
                            acc.consumed();
                        }
                    } else if count == 2
                        && let Some((start, end)) = word_range_at(lines, at)
                    {
                        st.select(start, end);
                        st.anchor = Some(start);
                        st.pointer_gesture = PointerGesture::Word {
                            range: (start, end),
                            dragged: false,
                        };
                        acc.action(ViewportAction::SelectionChanged);
                    } else {
                        let had = st.selection.take().is_some();
                        st.anchor = Some(at);
                        st.pointer_gesture = PointerGesture::Plain { dragged: false };
                        if had {
                            acc.action(ViewportAction::SelectionChanged);
                        } else {
                            acc.consumed();
                        }
                    }
                }
            }
            Phase::Drag => {
                st.pointer_gesture.mark_dragged();
                if self.pan_active() && !hit_text.is_empty() {
                    let before = st.horizontal.offset();
                    if pos.x < hit_text.x {
                        st.horizontal.scroll_by(-1);
                    } else if pos.x >= hit_text.right() {
                        st.horizontal.scroll_by(1);
                    }
                    horizontal_offset = st.horizontal.offset();
                    if before != horizontal_offset {
                        acc.changed();
                    }
                }
                if pos.y < area.y {
                    st.scroll.scroll_by(-1);
                    st.follow = FollowCursor::OnMove;
                } else if pos.y >= area.bottom() {
                    st.scroll.scroll_by(1);
                    st.follow = if st.scroll.at_end() {
                        FollowCursor::Always
                    } else {
                        FollowCursor::OnMove
                    };
                }
                let clamped = if hit_text.is_empty() {
                    pos
                } else {
                    Position::new(
                        pos.x.clamp(
                            hit_text.x,
                            hit_text.right().saturating_sub(1).max(hit_text.x),
                        ),
                        pos.y.clamp(
                            hit_text.y,
                            hit_text.bottom().saturating_sub(1).max(hit_text.y),
                        ),
                    )
                };
                let at = self.pos_at(
                    cx,
                    lines,
                    (logical_text, key, st.scroll.offset(), horizontal_offset),
                    clamped,
                );
                let selected = match (st.pointer_gesture, st.anchor, at) {
                    (PointerGesture::Word { range, .. }, _, Some(head)) => {
                        clamp_position(lines, range.0)
                            .zip(clamp_position(lines, range.1))
                            .map(|original| {
                                extend_word_range(original, head, word_range_at(lines, head))
                            })
                    }
                    (
                        PointerGesture::Plain { .. } | PointerGesture::Shift { .. },
                        Some(anchor),
                        Some(head),
                    ) => Some((anchor, head)),
                    _ => None,
                };
                if let Some((anchor, head)) = selected {
                    st.selection = Some((anchor, head));
                    st.browse = Some(head);
                    acc.action(ViewportAction::SelectionChanged);
                } else {
                    acc.consumed();
                }
            }
            Phase::Click | Phase::DoubleClick => {
                cx.focus(self.id);
                let gesture = st.pointer_gesture;
                if gesture.dragged()
                    || matches!(
                        gesture,
                        PointerGesture::Shift { .. } | PointerGesture::SuppressClick
                    )
                {
                    st.anchor = None;
                    st.pointer_gesture = PointerGesture::Idle;
                    acc.consumed();
                    return;
                }

                let count = cx.pointer_click_count(self.id, part).unwrap_or(1);
                let at = self.pos_at(
                    cx,
                    lines,
                    (logical_text, key, st.scroll.offset(), horizontal_offset),
                    pos,
                );
                let selected = at.and_then(|at| {
                    if count >= 3 {
                        logical_line_range(lines, at)
                    } else if count == 2 || self.click_selects_word {
                        word_range_at(lines, at)
                    } else {
                        None
                    }
                });
                if let Some((start, end)) = selected {
                    st.select(start, end);
                    acc.action(ViewportAction::SelectionChanged);
                } else {
                    acc.consumed();
                }
                st.anchor = None;
                st.pointer_gesture = PointerGesture::Idle;
            }
            Phase::Release | Phase::DragEnd => {
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                st.anchor = None;
                acc.consumed();
            }
            Phase::Secondary => acc.consumed(),
            Phase::Move => {}

            _ => {}
        }
    }

    fn command<'b>(
        st: &mut ViewportState,
        lines: impl Into<LineSet<'b>>,
        cmd: ViewportCmd,
        acc: &mut Acc<ViewportAction>,
    ) {
        let lines = lines.into();
        let before = st.scroll.offset();
        match cmd {
            ViewportCmd::Up => st.scroll.scroll_by(-1),
            ViewportCmd::Down => st.scroll.scroll_by(1),
            ViewportCmd::PageUp => st.scroll.page_up(),
            ViewportCmd::PageDown => st.scroll.page_down(),
            ViewportCmd::Home => st.scroll.jump_start(),
            ViewportCmd::End => {
                st.scroll.jump_end();
                let was = st.follow;
                st.follow = FollowCursor::Always;
                if was == FollowCursor::Always {
                    acc.changed();
                } else {
                    acc.action(ViewportAction::FollowChanged(true));
                }
                return;
            }
            ViewportCmd::ToggleFollow => {
                st.follow = match st.follow {
                    FollowCursor::Always => FollowCursor::OnMove,
                    FollowCursor::OnMove => FollowCursor::Always,
                };
                if st.follow == FollowCursor::Always {
                    st.scroll.jump_end();
                }
                acc.action(ViewportAction::FollowChanged(
                    st.follow == FollowCursor::Always,
                ));
                return;
            }
            ViewportCmd::Copy => {
                let mut text = String::new();
                if st.copy_from(lines, &mut text) {
                    acc.action(ViewportAction::Copy(text));
                } else {
                    acc.consumed();
                }
                return;
            }
        }
        if st.scroll.offset() == before {
            acc.consumed();
        } else {
            st.follow = if st.scroll.at_end() {
                FollowCursor::Always
            } else {
                FollowCursor::OnMove
            };
            acc.changed();
        }
    }

    pub fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
    ) -> Rect {
        self.draw_lines(ui, area, st, LineSet::Borrowed(lines))
    }

    pub fn draw_projected(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &ProjectedText,
    ) -> Rect {
        self.draw_lines(ui, area, st, LineSet::Projected(lines))
    }

    fn draw_lines(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: LineSet<'_>,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let ov = self.overrides();
        let id = self.id;
        let runtime = ui.state(id).difference(StateFlags::SELECTED);
        let live = PartStyle::flags(runtime, StateFlags::empty());
        let fallback_width =
            if self.scroll_gap > 0 && !self.wrap && lines.len() <= usize::from(area.height) {
                area.width.saturating_sub(u16::from(self.reserve_gutter))
            } else {
                Self::text_width(area.width, self.reserve_gutter, self.scroll_gap)
            };
        let initial_key = self.layout_key(st, fallback_width);
        let (total, widest, indexed) = {
            let layout = ui.cache::<ViewportLayout>(id);
            let indexed = layout.ensure_with_widest(lines, initial_key, self.pan_active());
            (layout.total(), layout.widest_cols().unwrap_or(0), indexed)
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            self.pan_dimensions(area, total, widest)
        } else {
            PanDimensions {
                text_width: fallback_width,
                text_height: area.height,
                horizontal_overflow: false,
            }
        };
        let text_w = dimensions.text_width;
        let key = self.layout_key(st, text_w);
        let (total, indexed) = {
            let layout = ui.cache::<ViewportLayout>(id);
            let indexed = layout.ensure_with_widest(lines, key, self.pan_active());
            (layout.total(), indexed)
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            let widest = ui.cache::<ViewportLayout>(id).widest_cols().unwrap_or(0);
            self.pan_dimensions(area, total, widest)
        } else {
            dimensions
        };
        let rows = dimensions.text_height;
        let text_w = dimensions.text_width;
        let mut view = st.scroll;
        view.apply_layout(usize::from(rows), total);
        if st.follow == FollowCursor::Always {
            view.jump_end();
        }
        let mut horizontal = st.horizontal;
        horizontal.set_content(if self.pan_active() { widest } else { 0 });
        horizontal.set_viewport(usize::from(text_w));
        let (first_line, first_row) = {
            let lay = ui.cache::<ViewportLayout>(id);
            lay.row_start(lines, view.offset())
        };
        let caret_line_start = st.caret.map(|c| {
            let lay = ui.cache::<ViewportLayout>(id);
            lay.line_start(c.line)
        });
        ui.register_control(id, area, Focusability::Focusable);
        ui.publish_bindings(id, live, BINDINGS);
        ui.publish_additional_bindings(id, runtime, SELECTION_BINDINGS);
        if st.selection.is_some() {
            ui.publish_additional_bindings(id, runtime, SELECTION_CLEAR_BINDINGS);
        }
        if self.pan_active() && dimensions.horizontal_overflow {
            let pan_bindings = if st.caret.is_some() {
                HORIZONTAL_PAN_SEMANTIC_BINDINGS
            } else {
                HORIZONTAL_PAN_BINDINGS
            };
            ui.publish_additional_bindings(id, runtime, pan_bindings);
        }
        let container = ov.style(
            ui,
            id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        let gutter_w = u16::from(self.reserve_gutter);
        if self.reserve_gutter {
            self.gutter(ui, area, live, &ov);
        }
        let body = Rect {
            x: area.x.saturating_add(gutter_w),
            y: area.y,
            width: area.width.saturating_sub(gutter_w),
            height: rows,
        };
        let content = self.bar().draw(ui, body, &view, total);
        let mut desired_text = self.text_area(area, text_w);
        desired_text.height = rows;
        let text = Rect {
            width: desired_text.width.min(content.width),
            ..desired_text
        };
        ui.report_layout(
            id,
            LayoutFacts::new(usize::from(rows), total, rows, text_w).with_logical_area(area),
        );
        if self.pan_active() {
            let vertical = view.headroom_v();
            let horizontal_headroom = horizontal.headroom_v();
            ui.register_scroll(
                id,
                text,
                Axes::Both,
                Headroom {
                    up: vertical.up,
                    down: vertical.down,
                    left: horizontal_headroom.up,
                    right: horizontal_headroom.down,
                },
            );
        }
        if text.is_empty() {
            return text;
        }
        ui.register_part(id, PartRef::of(Part::TEXT), text);
        let base = ov.style(ui, id, Family::VIEWPORT, Variant::DEFAULT, Part::TEXT, live);
        let selected = ov.style(
            ui,
            id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::TEXT,
            live | StateFlags::SELECTED,
        );
        self.rows(
            ui,
            text,
            lines,
            RowPosition {
                line: first_line,
                wrapped_row: first_row,
                horizontal_offset: if self.pan_active() {
                    horizontal.offset()
                } else {
                    0
                },
            },
            st.selection(),
            (base.style, selected.style),
        );
        self.caret(
            ui,
            text,
            st,
            lines,
            (
                text_w,
                view.offset(),
                caret_line_start,
                if self.pan_active() {
                    horizontal.offset()
                } else {
                    0
                },
            ),
            live,
        );
        let horizontal_track = self.horizontal_track(area, dimensions);
        if !horizontal_track.is_empty() {
            self.paint_horizontal_bar(ui, horizontal_track, &horizontal, live, &ov);
        }
        // Overflowing panes fade the scrolled edge. The frozen capsule keeps
        // 0.55 on the outer row and 0.8 on the next when the viewport is tall.
        // The gap column beside the bar fades with the text.
        let mut faded = text;
        let extra = self
            .scroll_gap
            .min(area.right().saturating_sub(text.right()));
        faded.width = faded.width.saturating_add(extra);
        ui.scroll_edges(faded, &view);
        text
    }

    fn paint_horizontal_bar(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &ScrollState,
        live: StateFlags,
        overrides: &PartStyle<'a>,
    ) {
        if area.is_empty() {
            return;
        }
        let track = overrides.style(
            ui,
            self.id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::TRACK,
            live,
        );
        let thumb_live = if ui.pressed_part(self.id) == Some(PartRef::of(HORIZONTAL_THUMB)) {
            live | StateFlags::PRESSED
        } else {
            live
        };
        let thumb = overrides.style(
            ui,
            self.id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::THUMB,
            thumb_live,
        );
        let (start, len) = state.thumb(usize::from(area.width));
        let end = start.saturating_add(len);
        ui.register_part(self.id, PartRef::of(HORIZONTAL_TRACK), area);
        if len > 0 {
            ui.register_part(
                self.id,
                PartRef::of(HORIZONTAL_THUMB),
                Rect::new(
                    area.x
                        .saturating_add(start.min(usize::from(u16::MAX)) as u16),
                    area.y,
                    len.min(usize::from(u16::MAX)) as u16,
                    1,
                ),
            );
        }
        for column in 0..area.width {
            let index = usize::from(column);
            let on_thumb = (start..end).contains(&index);
            ui.paint_str(
                Rect::new(area.x.saturating_add(column), area.y, 1, 1),
                if on_thumb { "━" } else { "┄" },
                if on_thumb { thumb.style } else { track.style },
            );
        }
    }

    /// Paint a horizontal scrollbar on a row the caller reserved.
    ///
    /// Baseline panes use `━` for the thumb and `┄` for the track, in the
    /// viewport track and thumb inks. The last column stays empty so the
    /// vertical bar's corner stays blank. A pane whose lines fit is left
    /// as the caller filled it.
    pub fn paint_horizontal_overflow(
        &self,
        ui: &mut Ui<'_>,
        row: Rect,
        lines: &ProjectedText,
        viewport_rows: u16,
    ) {
        if row.width < 2 || row.height == 0 {
            return;
        }
        let v_bar = !self.wrap && lines.line_count() > usize::from(viewport_rows);
        let text_w = if v_bar {
            Self::text_width(row.width, self.reserve_gutter, self.scroll_gap)
        } else {
            row.width.saturating_sub(u16::from(self.reserve_gutter))
        };
        let widest = lines.widest_cols();
        if text_w == 0 || widest <= usize::from(text_w) {
            return;
        }
        let track_w = usize::from(if v_bar {
            row.width.saturating_sub(1)
        } else {
            row.width
        });
        let mut across = ScrollState::default();
        across.set_content(widest);
        across.set_viewport(usize::from(text_w));
        let (start, len) = across.thumb(track_w);
        let ov = self.overrides();
        let live = StateFlags::empty();
        let thumb = ov
            .style(
                ui,
                self.id,
                Family::VIEWPORT,
                Variant::DEFAULT,
                Part::THUMB,
                live,
            )
            .style;
        let track = ov
            .style(
                ui,
                self.id,
                Family::VIEWPORT,
                Variant::DEFAULT,
                Part::TRACK,
                live,
            )
            .style;
        let end = start.saturating_add(len);
        for i in 0..track_w {
            let on_thumb = i >= start && i < end;
            ui.paint_str(
                Rect::new(row.x.saturating_add(i as u16), row.y, 1, 1),
                if on_thumb { "━" } else { "┄" },
                if on_thumb { thumb } else { track },
            );
        }
    }

    fn rows(
        &self,
        ui: &mut Ui<'_>,
        text: Rect,
        lines: LineSet<'_>,
        from: RowPosition,
        sel: Option<(CellPos, CellPos)>,
        styles: (PaintStyle, PaintStyle),
    ) {
        let mut li = from.line;
        let mut row = from.wrapped_row;
        let text_w = text.width;
        let mut rows_here = lines.get(li).map_or(0, |l| line_rows(l, text_w, self.wrap));
        #[cfg(feature = "testing")]
        let mut visible = 0usize;
        for y in text.rows() {
            let Some(line) = lines.get(li) else { break };
            #[cfg(feature = "testing")]
            {
                visible = visible.saturating_add(1);
            }
            let cols = if self.pan_active() {
                (
                    from.horizontal_offset,
                    from.horizontal_offset.saturating_add(usize::from(text_w)),
                )
            } else {
                row_cols(line, text_w, self.wrap, row)
            };
            paint_row(ui, y, line, li, cols, sel, styles);
            row = row.saturating_add(1);
            if row >= rows_here {
                row = 0;
                li = li.saturating_add(1);
                rows_here = lines.get(li).map_or(0, |l| line_rows(l, text_w, self.wrap));
            }
        }
        #[cfg(feature = "testing")]
        self.note_visible(visible);
    }

    fn gutter(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags, ov: &PartStyle<'a>) {
        let col = Rect {
            x: area.x,
            y: area.y,
            width: 1,
            height: area.height,
        };
        if let Some(f) = ov.slot_for(Part::GUTTER) {
            f(ui, col);
            return;
        }
        let g = ov.style(
            ui,
            self.id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::GUTTER,
            live,
        );
        match g.glyph {
            Slot::Set(glyph) => {
                for r in col.rows() {
                    ui.glyph(r, glyph, g.style);
                }
            }
            Slot::Inherit if live.contains(StateFlags::FOCUSED) => {
                for r in col.rows() {
                    ui.glyph(r, GlyphRole::FocusBar, g.style);
                }
            }
            Slot::Inherit | Slot::Clear => ui.fill(col, g.style),
        }
    }

    fn caret(
        &self,
        ui: &mut Ui<'_>,
        text: Rect,
        st: &ViewportState,
        lines: LineSet<'_>,
        view: (u16, usize, Option<usize>, usize),
        live: StateFlags,
    ) {
        let (width, offset, line_start, horizontal_offset) = view;
        let Some(c) = st.caret else { return };
        if !live.contains(StateFlags::FOCUSED)
            && !(self.caret_when_unfocused && st.follow() && st.selection().is_none())
        {
            return;
        }
        let Some(at) = lines.get(c.line) else {
            return;
        };
        let before = line_start.unwrap_or(c.line);
        let mut within = 0usize;
        if self.wrap {
            for i in 0..line_rows(at, width, true) {
                let (a, b) = row_cols(at, width, true, i);
                if c.col >= a && c.col < b {
                    within = i;
                    break;
                }
            }
        }
        let visual = before.saturating_add(within);
        if visual < offset {
            return;
        }
        let dy = visual.saturating_sub(offset);
        let (start, _) = row_cols(at, width, self.wrap, within);
        let caret_col = c.col.saturating_sub(start);
        if caret_col < horizontal_offset {
            return;
        }
        let dx = caret_col.saturating_sub(horizontal_offset);
        if dy >= usize::from(text.height) || dx >= usize::from(text.width) {
            return;
        }
        let visible = text.intersection(ui.full());
        if visible.is_empty() {
            return;
        }
        let pos = Position::new(
            visible.x.saturating_add(dx as u16),
            text.y.saturating_add(dy as u16),
        );
        if pos.x < visible.x
            || pos.x >= visible.right()
            || pos.y < visible.y
            || pos.y >= visible.bottom()
        {
            return;
        }
        if self.bar_cursor {
            ui.set_bar_cursor(self.id, pos);
        } else {
            ui.set_cursor(self.id, pos);
        }
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let reserved = 1u16
            .saturating_add(u16::from(self.reserve_gutter))
            .saturating_add(self.scroll_gap);
        Size {
            min: (3, 1),
            preferred: (Self::PREFERRED_TEXT_WIDTH.saturating_add(reserved), c.max.1),
        }
        .fit(c)
    }
}

#[derive(Clone)]
struct Run {
    span: usize,
    sel: bool,
    a: usize,
    b: usize,
    joined: Option<Box<str>>,
}

fn paint_row(
    ui: &mut Ui<'_>,
    rect: Rect,
    line: LineRef<'_>,
    line_ix: usize,
    cols: (usize, usize),
    sel: Option<(CellPos, CellPos)>,
    styles: (PaintStyle, PaintStyle),
) {
    let (from, to) = cols;
    let right = rect.right();
    let mut x = rect.x;
    let mut col = 0usize;
    let mut run: Option<Run> = None;
    let flush = |ui: &mut Ui<'_>, r: Option<Run>, x: &mut u16| {
        let Some(r) = r else { return };
        let text = r
            .joined
            .as_deref()
            .or_else(|| run_text(line, r.span).get(r.a..r.b))
            .unwrap_or("");
        if text.is_empty() || *x >= right {
            return;
        }
        let (role, add) = run_style(line, r.span);
        let sp = Span {
            text,
            role: if r.sel { None } else { role },
            add,
        };
        let base = if r.sel { styles.1 } else { styles.0 };
        let w = ui.paint_spans(
            Rect {
                x: *x,
                y: rect.y,
                width: right.saturating_sub(*x),
                height: 1,
            },
            &[sp],
            base,
        );
        *x = x.saturating_add(w);
    };
    walk_line(line, |c| {
        if col >= to || x >= right {
            return ControlFlow::Break(());
        }
        let next = col.saturating_add(usize::from(c.w));
        if next <= from {
            col = next;
            return ControlFlow::Continue(());
        }
        if col < from {
            flush(ui, run.take(), &mut x);
            let blank = next
                .saturating_sub(from)
                .min(usize::from(right.saturating_sub(x)));
            let is_sel = sel.is_some_and(|(a, b)| {
                let p = CellPos { line: line_ix, col };
                p >= a && p < b
            });
            if blank > 0 {
                ui.fill(
                    Rect {
                        x,
                        y: rect.y,
                        width: blank.min(usize::from(u16::MAX)) as u16,
                        height: 1,
                    },
                    if is_sel { styles.1 } else { styles.0 },
                );
                x = x.saturating_add(blank.min(usize::from(u16::MAX)) as u16);
            }
            col = next;
            return ControlFlow::Continue(());
        }
        if next > to {
            return ControlFlow::Break(());
        }
        let is_sel = sel.is_some_and(|(a, b)| {
            let p = CellPos { line: line_ix, col };
            p >= a && p < b
        });
        if c.tab {
            flush(ui, run.take(), &mut x);
            ui.fill(
                Rect {
                    x,
                    y: rect.y,
                    width: 1,
                    height: 1,
                },
                if is_sel { styles.1 } else { styles.0 },
            );
            x = x.saturating_add(1);
        } else if c.joined.is_some() {
            flush(ui, run.take(), &mut x);
            run = Some(Run {
                span: c.span,
                sel: is_sel,
                a: c.at,
                b: c.at.saturating_add(c.len),
                joined: c.joined,
            });
        } else {
            let end = c.at.saturating_add(c.len);
            match run.take() {
                Some(r)
                    if r.joined.is_none() && r.span == c.span && r.sel == is_sel && r.b == c.at =>
                {
                    run = Some(Run { b: end, ..r });
                }
                other => {
                    flush(ui, other, &mut x);
                    run = Some(Run {
                        span: c.span,
                        sel: is_sel,
                        a: c.at,
                        b: end,
                        joined: None,
                    });
                }
            }
        }
        col = next;
        ControlFlow::Continue(())
    });
    flush(ui, run.take(), &mut x);
}

impl Bindings for TextViewport<'_> {
    type Cmd = ViewportCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ViewportCmd>] {
        BINDINGS
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::{Buffer, Cell};

    use super::*;
    use crate::event::{Axis, MouseKind};
    use crate::runtime::stub::{SCREEN, Stub, key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::{ColorLevel, Surface, Theme};
    use crate::{ReferenceState, ReferenceTarget};
    use termrock_runtime::LayerSpec;

    const ID: Id = Id::root("viewport.tests");
    const SELECTION_ID: Id = Id::root("viewport.selection.tests");
    const OTHER_ID: Id = Id::root("viewport.selection.other");
    const MODAL_LAYER: Id = Id::root("viewport.selection.modal");
    const MODAL_CONTROL: Id = Id::root("viewport.selection.modal-control");
    const POINTER_AREA: Rect = Rect::new(0, 0, 12, 8);
    const MANY_LINES: [ViewportLine<'static>; 100] = [ViewportLine::Plain("row"); 100];
    const LONG_LINES: [ViewportLine<'static>; 100] =
        [ViewportLine::Plain("012345678901234567890123456789"); 100];

    struct SelectionViewportApp<'a> {
        state: ViewportState,
        lines: Vec<ViewportLine<'a>>,
        area: Rect,
        wrap: bool,
        pan: bool,
        click_selects_word: bool,
        keymap: crate::keymap::KeyMap,
        actions: Vec<ViewportAction>,
        other_focusable: bool,
        other_pointer: bool,
        other_pointer_state: ViewportState,
        modal_open: bool,
        reference_only: bool,
    }

    impl<'a> SelectionViewportApp<'a> {
        fn new(lines: Vec<ViewportLine<'a>>) -> Self {
            let mut state = ViewportState::default();
            state.set_follow(false);
            SelectionViewportApp {
                state,
                lines,
                area: Rect::new(0, 0, 12, 5),
                wrap: false,
                pan: false,
                click_selects_word: false,
                keymap: crate::keymap::KeyMap::new(),
                actions: Vec::new(),
                other_focusable: false,
                other_pointer: false,
                other_pointer_state: ViewportState::default(),
                modal_open: false,
                reference_only: false,
            }
        }

        fn viewport(&self) -> TextViewport<'static> {
            TextViewport::new(SELECTION_ID)
                .wrap(self.wrap)
                .horizontal_pan(self.pan)
                .click_selects_word(self.click_selects_word)
        }
    }

    impl App for SelectionViewportApp<'_> {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if self.modal_open {
                cx.open_layer(
                    MODAL_LAYER,
                    LayerSpec::modal(SELECTION_ID).initial_focus(MODAL_CONTROL),
                );
            }
            let response = self.viewport().update(cx, &mut self.state, &self.lines);
            if self.other_pointer {
                let _ = TextViewport::new(OTHER_ID).update(
                    cx,
                    &mut self.other_pointer_state,
                    &self.lines,
                );
            }
            if let Some(action) = response.action_ref() {
                self.actions.push(action.clone());
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            if self.reference_only {
                ui.reference(
                    Some(ReferenceTarget::new(SELECTION_ID, ReferenceState::FOCUSED)),
                    |ui| {
                        self.viewport()
                            .draw(ui, self.area, &self.state, &self.lines);
                    },
                );
            } else {
                self.viewport()
                    .draw(ui, self.area, &self.state, &self.lines);
            }
            if self.other_focusable {
                ui.register_control(OTHER_ID, Rect::new(40, 0, 8, 2), Focusability::Focusable);
            }
            if self.other_pointer {
                TextViewport::new(OTHER_ID).draw(
                    ui,
                    Rect::new(26, 0, 12, 5),
                    &self.other_pointer_state,
                    &self.lines,
                );
            }
            let _ = ui.layer(MODAL_LAYER, |ui, area| {
                ui.register_control(MODAL_CONTROL, area, Focusability::Focusable);
            });
        }

        fn keymap(&self) -> &crate::keymap::KeyMap {
            &self.keymap
        }
    }

    fn selection_key(code: KeyCode, mods: KeyModifiers) -> crate::event::Input {
        crate::event::Input::Key(crate::event::Key { code, mods })
    }

    fn selection_runtime<'a>(app: SelectionViewportApp<'a>) -> Runtime<SelectionViewportApp<'a>> {
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        runtime
    }

    fn selection_mouse(kind: MouseKind, x: u16, y: u16, mods: KeyModifiers) -> crate::event::Input {
        crate::event::Input::Mouse(crate::event::Mouse {
            kind,
            pos: Position::new(x, y),
            mods,
        })
    }

    fn deliver_click_count(
        runtime: &mut Runtime<SelectionViewportApp<'_>>,
        count: u8,
        x: u16,
        y: u16,
    ) {
        for _ in 0..count {
            let _ = crate::runtime::stub::deliver(runtime, mouse(MouseKind::Down, x, y));
            let _ = crate::runtime::stub::deliver(runtime, mouse(MouseKind::Up, x, y));
        }
    }

    fn copy_from_viewport(runtime: &mut Runtime<SelectionViewportApp<'_>>) -> String {
        let _ = crate::runtime::stub::deliver(
            runtime,
            selection_key(KeyCode::Char('y'), KeyModifiers::NONE),
        );
        match runtime.app().actions.last() {
            Some(ViewportAction::Copy(text)) => text.clone(),
            other => panic!("copy action was not produced: {other:?}"),
        }
    }

    #[test]
    fn pointer_click_counts_select_words_and_wrapped_lines() {
        for click_selects_word in [false, true] {
            for count in [2, 3, 4] {
                let mut app =
                    SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma")]);
                app.area = Rect::new(0, 0, 12, 4);
                app.wrap = true;
                app.click_selects_word = click_selects_word;
                let mut runtime = selection_runtime(app);

                deliver_click_count(&mut runtime, count, 7, 0);

                let expected = if count == 2 {
                    (CellPos::new(0, 6), CellPos::new(0, 10))
                } else {
                    (CellPos::new(0, 0), CellPos::new(0, 16))
                };
                assert_eq!(
                    runtime.app().state.selection,
                    Some(expected),
                    "count {count} selected the wrong logical range with click_selects_word={click_selects_word}"
                );
                assert_eq!(
                    copy_from_viewport(&mut runtime),
                    if count == 2 {
                        "beta"
                    } else {
                        "alpha beta gamma"
                    },
                    "count {count} copied the wrong source text"
                );
            }
        }
    }

    #[test]
    fn single_click_word_selection_stays_opt_in_and_never_moves_the_caret() {
        for click_selects_word in [false, true] {
            let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta")]);
            app.area = Rect::new(0, 0, 16, 4);
            app.click_selects_word = click_selects_word;
            app.state.set_caret(Some(CellPos::new(0, 7)));
            let mut runtime = selection_runtime(app);
            deliver_click_count(&mut runtime, 1, 3, 0);

            let expected = click_selects_word.then_some((CellPos::new(0, 0), CellPos::new(0, 5)));
            assert_eq!(runtime.app().state.selection(), expected);
            assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 7)));
        }

        let mut blocked = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta")]);
        blocked.area = Rect::new(0, 0, 16, 4);
        blocked.click_selects_word = true;
        blocked.modal_open = true;
        let mut runtime = selection_runtime(blocked);
        assert_eq!(runtime.focus(), Some(MODAL_CONTROL));
        deliver_click_count(&mut runtime, 1, 3, 0);
        assert_eq!(runtime.app().state.selection(), None);
        assert_eq!(runtime.capture_owner(), None);

        let mut reference = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta")]);
        reference.area = Rect::new(0, 0, 16, 4);
        reference.click_selects_word = true;
        reference.reference_only = true;
        let mut runtime = selection_runtime(reference);
        deliver_click_count(&mut runtime, 1, 3, 0);
        assert_eq!(runtime.app().state.selection(), None);
        assert_eq!(runtime.capture_owner(), None);
    }

    #[test]
    fn cancelling_a_drag_suppresses_the_stale_click_after_capture_release() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma")]);
        app.area = Rect::new(0, 0, 24, 4);
        app.click_selects_word = true;
        let mut runtime = selection_runtime(app);
        deliver_click_count(&mut runtime, 1, 7, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 7, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 13, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 16)))
        );
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));

        runtime.app_mut().modal_open = true;
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Char('x'), KeyModifiers::NONE),
        );
        runtime.app_mut().modal_open = false;
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert_eq!(runtime.capture_owner(), None);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 13, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 16))),
            "a cancelled drag must not be replaced by its stale count-two click"
        );
    }

    #[test]
    fn counted_press_starts_word_drag_and_drag_extends_whole_words_both_ways() {
        let mut forward =
            SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma delta")]);
        forward.area = Rect::new(0, 0, 12, 4);
        forward.wrap = true;
        let mut runtime = selection_runtime(forward);
        deliver_click_count(&mut runtime, 1, 7, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 7, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 10))),
            "the second press must select its whole word before dragging"
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 3, 1));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 1));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 16)))
        );
        assert_eq!(copy_from_viewport(&mut runtime), "beta gamma");
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 15))),
            "keyboard extension must continue from the pointer head"
        );

        let mut reverse =
            SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma delta")]);
        reverse.area = Rect::new(0, 0, 12, 4);
        reverse.wrap = true;
        let mut runtime = selection_runtime(reverse);
        deliver_click_count(&mut runtime, 1, 4, 1);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 4, 1));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 11), CellPos::new(0, 16)))
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 3, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 16), CellPos::new(0, 0))),
            "reverse word drag must retain its oriented anchor"
        );
        assert_eq!(copy_from_viewport(&mut runtime), "alpha beta gamma");
    }

    #[test]
    fn word_drag_range_tracks_retained_lines_and_shrunk_source() {
        let mut retained = SelectionViewportApp::new(vec![
            ViewportLine::Plain("old row"),
            ViewportLine::Plain("one two three"),
        ]);
        retained.area = Rect::new(0, 0, 24, 4);
        let mut runtime = selection_runtime(retained);
        deliver_click_count(&mut runtime, 1, 6, 1);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 6, 1));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(1, 4), CellPos::new(1, 7)))
        );

        runtime.app_mut().state.retained(1);
        let _ = runtime.app_mut().lines.remove(0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 10, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 10, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 4), CellPos::new(0, 13)))
        );
        assert_eq!(copy_from_viewport(&mut runtime), "two three");

        let mut shrunk = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta")]);
        shrunk.area = Rect::new(0, 0, 16, 4);
        let mut runtime = selection_runtime(shrunk);
        deliver_click_count(&mut runtime, 1, 3, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 3, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 5)))
        );
        runtime.app_mut().lines[0] = ViewportLine::Plain("abc");
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 2, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 3)))
        );
        assert_eq!(copy_from_viewport(&mut runtime), "abc");
    }

    #[test]
    fn clearing_source_while_pointer_is_down_discards_its_anchor() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta")]);
        app.area = Rect::new(0, 0, 16, 4);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 3, 0));
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 2)));

        runtime.app_mut().lines.clear();
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 0));
        assert_eq!(runtime.app().state.anchor, None);
        assert_eq!(runtime.app().state.pointer_gesture, PointerGesture::Idle);
        assert_eq!(runtime.app().state.selection(), None);

        runtime.app_mut().lines = vec![ViewportLine::Plain("new words")];
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 0, KeyModifiers::SHIFT),
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 5, 0));
        assert_eq!(runtime.app().state.selection(), None);
    }

    #[test]
    fn shift_pointer_click_extends_oriented_collapsed_and_reverse_selections() {
        let cases = [
            (
                (CellPos::new(0, 5), CellPos::new(0, 10)),
                17usize,
                (CellPos::new(0, 5), CellPos::new(0, 17)),
                (CellPos::new(0, 5), CellPos::new(0, 16)),
            ),
            (
                (CellPos::new(0, 16), CellPos::new(0, 11)),
                2usize,
                (CellPos::new(0, 16), CellPos::new(0, 2)),
                (CellPos::new(0, 16), CellPos::new(0, 1)),
            ),
            (
                (CellPos::new(0, 5), CellPos::new(0, 5)),
                8usize,
                (CellPos::new(0, 5), CellPos::new(0, 8)),
                (CellPos::new(0, 5), CellPos::new(0, 7)),
            ),
        ];

        for (initial, target_col, expected, after_keyboard) in cases {
            let mut app =
                SelectionViewportApp::new(vec![ViewportLine::Plain("zero alpha beta omega")]);
            app.area = Rect::new(0, 0, 24, 4);
            app.click_selects_word = true;
            app.state.select(initial.0, initial.1);
            app.state.set_caret(Some(CellPos::new(0, 3)));
            let mut runtime = selection_runtime(app);

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(
                    MouseKind::Down,
                    u16::try_from(target_col.saturating_add(1)).expect("test coordinate fits"),
                    0,
                    KeyModifiers::SHIFT,
                ),
            );
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                mouse(
                    MouseKind::Up,
                    u16::try_from(target_col.saturating_add(1)).expect("test coordinate fits"),
                    0,
                ),
            );
            assert_eq!(runtime.app().state.selection, Some(expected));
            assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 3)));

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_key(KeyCode::Left, KeyModifiers::SHIFT),
            );
            assert_eq!(runtime.app().state.selection, Some(after_keyboard));
        }
    }

    #[test]
    fn shift_click_without_selection_ignores_caret_and_a_prior_plain_press() {
        let line = ViewportLine::Plain("zero alpha beta omega");
        for starts_selected in [false, true] {
            let mut app = SelectionViewportApp::new(vec![line]);
            app.area = Rect::new(0, 0, 24, 4);
            app.click_selects_word = !starts_selected;
            app.state.set_caret(Some(CellPos::new(0, 2)));
            if starts_selected {
                app.state.select(CellPos::new(0, 1), CellPos::new(0, 5));
            }
            let mut runtime = selection_runtime(app);
            if starts_selected {
                let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 8, 0));
                let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 8, 0));
            }

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 12, 0, KeyModifiers::SHIFT),
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 12, 0));
            assert_eq!(runtime.app().state.selection(), None);
            assert_eq!(runtime.app().state.browse, None);

            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_key(KeyCode::Left, KeyModifiers::SHIFT),
            );
            assert_eq!(
                runtime.app().state.selection,
                Some((CellPos::new(0, 21), CellPos::new(0, 20))),
                "Shift-click without an oriented anchor must use the visible-line fallback, not caret or stale press"
            );
            assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 2)));
        }
    }

    #[test]
    fn pointer_click_count_resets_after_time_slop_owner_and_part_changes() {
        let mut timed = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma")]);
        timed.area = Rect::new(0, 0, 24, 4);
        let mut runtime = selection_runtime(timed);
        deliver_click_count(&mut runtime, 1, 3, 0);
        runtime.advance_clock(5_000);
        deliver_click_count(&mut runtime, 1, 3, 0);
        assert_eq!(
            runtime.app().state.selection(),
            None,
            "elapsed time must reset the click count"
        );

        let mut slop = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma")]);
        slop.area = Rect::new(0, 0, 24, 4);
        let mut runtime = selection_runtime(slop);
        runtime.set_pointer_click_slop(0);
        deliver_click_count(&mut runtime, 1, 3, 0);
        deliver_click_count(&mut runtime, 1, 4, 0);
        assert_eq!(
            runtime.app().state.selection(),
            None,
            "click slop must be measured against the same point"
        );

        let mut owner = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma")]);
        owner.area = Rect::new(0, 0, 24, 4);
        owner.other_pointer = true;
        let mut runtime = selection_runtime(owner);
        deliver_click_count(&mut runtime, 1, 3, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 30, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 30, 0));
        deliver_click_count(&mut runtime, 1, 3, 0);
        assert_eq!(
            runtime.app().state.selection(),
            None,
            "changing pointer owner must reset the count"
        );

        let mut part = SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma"); 20]);
        part.area = Rect::new(0, 0, 12, 5);
        let mut runtime = selection_runtime(part);
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(
            runtime
                .area_of_part(SELECTION_ID, PartRef::of(Part::TRACK))
                .is_some(),
            "the overflowing viewport must register its vertical track"
        );
        deliver_click_count(&mut runtime, 1, 3, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 11, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 11, 0));
        deliver_click_count(&mut runtime, 1, 3, 0);
        assert_eq!(
            runtime.app().state.selection(),
            None,
            "changing part must reset the count"
        );
    }

    #[test]
    fn pointer_selection_copy_preserves_styled_unicode_and_viewport_offsets() {
        let spans = [
            Span::new("e").modifier(Modifier::BOLD),
            Span::new("\u{301} ").modifier(Modifier::ITALIC),
            Span::new("界").role(Role::Success),
            Span::new(" "),
            Span::new("👩").modifier(Modifier::BOLD),
            Span::new("\u{200d}💻 ").modifier(Modifier::ITALIC),
            Span::new("🇺").role(Role::Info),
            Span::new("🇸 alpha").role(Role::Warning),
        ];
        let line_text = "e\u{301} 界 👩\u{200d}💻 🇺🇸 alpha";
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Spans(&spans)]);
        app.area = Rect::new(0, 0, 24, 4);
        let mut runtime = selection_runtime(app);
        deliver_click_count(&mut runtime, 2, 3, 0);
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 2), CellPos::new(0, 4))),
            "double click must map a CJK word through styled grapheme runs and the gutter"
        );
        assert_eq!(copy_from_viewport(&mut runtime), "界");

        let mut runtime =
            selection_runtime(SelectionViewportApp::new(vec![ViewportLine::Spans(&spans)]));
        deliver_click_count(&mut runtime, 3, 3, 0);
        assert_eq!(
            runtime.app().state.selection,
            Some((
                CellPos::new(0, 0),
                CellPos::new(0, line_cols(ViewportLine::Spans(&spans)))
            )),
            "triple click selects the logical line, not the painted gutter"
        );
        assert_eq!(copy_from_viewport(&mut runtime), line_text);

        let mut panned =
            SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma delta")]);
        panned.area = Rect::new(0, 0, 12, 4);
        panned.pan = true;
        panned.state.horizontal.set_content(22);
        panned.state.horizontal.set_viewport(10);
        panned.state.horizontal.scroll_to(4);
        let mut runtime = selection_runtime(panned);
        deliver_click_count(&mut runtime, 2, 1, 0);
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 5))),
            "word mapping must include the horizontal content offset"
        );
        assert_eq!(copy_from_viewport(&mut runtime), "alpha");
    }

    #[test]
    fn shift_selection_starts_at_the_last_visible_logical_line_end() {
        let mut app = SelectionViewportApp::new(
            ["zero", "one", "two", "three", "four", "visible"]
                .into_iter()
                .map(ViewportLine::Plain)
                .collect(),
        );
        app.area = Rect::new(0, 0, 12, 4);
        app.state.scroll.apply_layout(4, app.lines.len());
        app.state.scroll.scroll_to(2);
        app.state.set_caret(Some(CellPos::new(2, 1)));
        let mut runtime = selection_runtime(app);

        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(5, 7), CellPos::new(5, 6)))
        );
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(2, 1)));
        assert!(response.is_changed());
        assert_eq!(
            runtime.app().actions.last(),
            Some(&ViewportAction::SelectionChanged)
        );
    }

    #[test]
    fn plain_pointer_press_clears_selection_without_becoming_keyboard_origin() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("clicked"),
            ViewportLine::Plain("visible"),
        ]);
        app.state.select(CellPos::new(0, 0), CellPos::new(0, 3));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 1, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 1, 0));
        assert_eq!(runtime.app().state.selection, None);
        assert_eq!(runtime.app().state.browse, None);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(1, 7), CellPos::new(1, 6))),
            "a plain click must not replace the documented first-Shift fallback"
        );
    }

    #[test]
    fn wrapped_shift_fallback_uses_the_last_visible_logical_line() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("abcdefghijklmnop"),
            ViewportLine::Plain("tail"),
        ]);
        app.area = Rect::new(0, 0, 8, 2);
        app.wrap = true;
        app.state.scroll.apply_layout(2, 4);
        app.state.scroll.scroll_to(1);
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 16), CellPos::new(0, 15)))
        );
    }

    #[test]
    fn horizontal_selection_moves_across_logical_lines_and_line_ends() {
        let mut app =
            SelectionViewportApp::new(vec![ViewportLine::Plain("abc"), ViewportLine::Plain("de")]);
        app.state.select(CellPos::new(0, 3), CellPos::new(0, 3));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(1, 0)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 3)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 2)))
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Home, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 0)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::End, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 3)))
        );
    }

    #[test]
    fn vertical_selection_clamps_the_current_column_on_each_logical_line() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("abcdef"),
            ViewportLine::Plain("xy"),
            ViewportLine::Plain("uvwxyz"),
        ]);
        app.state.select(CellPos::new(0, 4), CellPos::new(0, 4));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Down, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 4), CellPos::new(1, 2)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Down, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 4), CellPos::new(2, 2))),
            "the next line starts from the clamped logical column"
        );
    }

    #[test]
    fn pointer_drag_selection_continues_through_collapse_and_reexpansion() {
        let app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdef")]);
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 1, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 3, 0));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 0));
        assert_eq!(
            runtime.app().state.selection(),
            Some((CellPos::new(0, 0), CellPos::new(0, 2)))
        );

        for _ in 0..2 {
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_key(KeyCode::Left, KeyModifiers::SHIFT),
            );
        }
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 0))),
            "the oriented anchor must survive a collapsed visible range"
        );
        assert_eq!(runtime.app().state.selection(), None);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 1)))
        );
    }

    #[test]
    fn selection_word_and_document_chords_move_over_logical_cells() {
        let app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("alpha  beta"),
            ViewportLine::Plain("last"),
        ]);
        let mut runtime = selection_runtime(app);
        runtime
            .app_mut()
            .state
            .select(CellPos::new(0, 0), CellPos::new(0, 0));

        let ctrl_shift = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
        let _ =
            crate::runtime::stub::deliver(&mut runtime, selection_key(KeyCode::Right, ctrl_shift));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 5)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 11)))
        );

        let _ =
            crate::runtime::stub::deliver(&mut runtime, selection_key(KeyCode::End, ctrl_shift));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(1, 4)))
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Home, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 0)))
        );
    }

    #[test]
    fn empty_viewport_consumes_shift_selection_without_creating_a_range() {
        let mut runtime = selection_runtime(SelectionViewportApp::new(Vec::new()));

        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed());
        assert!(!response.is_changed());
        assert_eq!(runtime.app().state.selection, None);
        assert_eq!(runtime.app().state.browse, None);
    }

    #[test]
    fn a_shrunk_multiline_selection_reconciles_before_a_boundary_command() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("first"),
            ViewportLine::Plain("second"),
        ]);
        app.area = Rect::new(0, 0, 12, 1);
        app.state.select(CellPos::new(0, 0), CellPos::new(1, 6));
        app.state.scroll.apply_layout(1, 2);
        app.state.scroll.scroll_to(1);
        app.state.set_follow(true);
        app.state.set_caret(Some(CellPos::new(1, 3)));
        let mut runtime = selection_runtime(app);
        let mut buffer = Buffer::empty(SCREEN);

        runtime.app_mut().lines = vec![ViewportLine::Plain("x")];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::End, KeyModifiers::SHIFT),
        );

        let expected = Some((CellPos::new(0, 0), CellPos::new(0, 1)));
        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(runtime.app().state.selection, expected);
        assert_eq!(runtime.app().state.selection(), expected);
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 1)));
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert_eq!(runtime.app().state.scroll.offset(), 0);
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(1, 3)));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&ViewportAction::SelectionChanged)
        );

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert!(
            runtime
                .app()
                .actions
                .contains(&ViewportAction::Copy("x".to_owned()))
        );

        runtime.app_mut().lines = vec![ViewportLine::Plain("x"), ViewportLine::Plain("appended")];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 0)))
        );
        assert_eq!(runtime.app().state.selection(), None);
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 0)));
    }

    #[test]
    fn a_wide_cell_shrink_reconciles_both_oriented_endpoints_before_noop() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abc漢q")]);
        app.state.select(CellPos::new(0, 4), CellPos::new(0, 6));
        app.state.set_follow(true);
        app.state.set_caret(Some(CellPos::new(0, 1)));
        let mut runtime = selection_runtime(app);
        let mut buffer = Buffer::empty(SCREEN);

        runtime.app_mut().lines = vec![ViewportLine::Plain("abc漢")];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::End, KeyModifiers::SHIFT),
        );

        let expected = Some((CellPos::new(0, 3), CellPos::new(0, 5)));
        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(runtime.app().state.selection, expected);
        assert_eq!(runtime.app().state.selection(), expected);
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 5)));
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 1)));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&ViewportAction::SelectionChanged)
        );

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert!(
            runtime
                .app()
                .actions
                .contains(&ViewportAction::Copy("漢".to_owned()))
        );

        runtime.app_mut().lines = vec![ViewportLine::Plain("abc漢q")];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 3)))
        );
        assert_eq!(runtime.app().state.selection(), None);
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 3)));
    }

    #[test]
    fn browse_only_reconciliation_marks_change_without_selection_action() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdef")]);
        app.state.browse = Some(CellPos::new(0, 6));
        app.state.set_follow(true);
        let mut runtime = selection_runtime(app);
        let mut buffer = Buffer::empty(SCREEN);

        runtime.app_mut().lines = vec![ViewportLine::Plain("abc")];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::End, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(runtime.app().state.selection, None);
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 3)));
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert!(runtime.app().actions.is_empty());
    }

    #[test]
    fn an_empty_source_clears_keyboard_continuation_before_repopulation() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("old-first"),
            ViewportLine::Plain("obsolete"),
        ]);
        app.area = Rect::new(0, 0, 12, 1);
        app.state.select(CellPos::new(0, 2), CellPos::new(1, 8));
        app.state.scroll.apply_layout(1, 2);
        app.state.scroll.scroll_to(1);
        app.state.set_follow(true);
        app.state.set_caret(Some(CellPos::new(1, 2)));
        let mut runtime = selection_runtime(app);
        let mut buffer = Buffer::empty(SCREEN);

        runtime.app_mut().lines.clear();
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(runtime.app().state.selection, None);
        assert_eq!(runtime.app().state.selection(), None);
        assert_eq!(runtime.app().state.browse, None);
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(1, 2)));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&ViewportAction::SelectionChanged)
        );

        runtime.app_mut().lines = vec![
            ViewportLine::Plain("new-first"),
            ViewportLine::Plain("last"),
        ];
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed() && response.is_changed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 9), CellPos::new(0, 8)))
        );
        assert_eq!(
            runtime.app().state.selection(),
            Some((CellPos::new(0, 8), CellPos::new(0, 9)))
        );
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 8)));
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(1, 2)));

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert!(
            runtime
                .app()
                .actions
                .contains(&ViewportAction::Copy("t".to_owned()))
        );
    }

    #[test]
    fn selection_boundary_noop_is_consumed_without_changing_selection() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abc")]);
        app.state.select(CellPos::new(0, 3), CellPos::new(0, 3));
        let mut runtime = selection_runtime(app);

        let response = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );

        assert!(response.is_consumed());
        assert!(!response.is_changed());
        assert!(runtime.app().actions.is_empty());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 3)))
        );
    }

    #[test]
    fn reversed_multiline_selection_copies_source_text_after_keyboard_extension() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("abc"),
            ViewportLine::Plain("defgh"),
        ]);
        app.state.select(CellPos::new(1, 4), CellPos::new(0, 1));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(1, 4), CellPos::new(0, 2)))
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));

        assert!(
            runtime
                .app()
                .actions
                .contains(&ViewportAction::Copy("c\ndefg".to_owned()))
        );
    }

    #[test]
    fn keyboard_extension_uses_graphemes_across_runs_and_copies_utf8_source() {
        static RUNS: [Span<'static>; 5] = [
            Span::new("e"),
            Span::new("\u{301}👩"),
            Span::new("\u{200d}"),
            Span::new("💻🇺"),
            Span::new("🇸x"),
        ];
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Spans(&RUNS)]);
        app.state.select(CellPos::new(0, 0), CellPos::new(0, 0));
        let mut runtime = selection_runtime(app);

        for expected_col in [1, 3, 5] {
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_key(KeyCode::Right, KeyModifiers::SHIFT),
            );
            assert_eq!(
                runtime.app().state.selection.map(|(_, head)| head.col),
                Some(expected_col)
            );
        }
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));

        assert!(
            runtime
                .app()
                .actions
                .contains(&ViewportAction::Copy("e\u{301}👩\u{200d}💻🇺🇸".to_owned()))
        );
    }

    #[test]
    fn vertical_extension_snaps_a_wide_destination_to_its_leading_cell() {
        let mut app =
            SelectionViewportApp::new(vec![ViewportLine::Plain("x"), ViewportLine::Plain("漢z")]);
        app.state.select(CellPos::new(0, 0), CellPos::new(0, 1));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Down, KeyModifiers::SHIFT),
        );

        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(1, 0)))
        );
    }

    #[test]
    fn selection_actions_are_remappable_without_changing_plain_pan_arrows() {
        let mut app =
            SelectionViewportApp::new(vec![ViewportLine::Plain("012345678901234567890123456789")]);
        app.pan = true;
        app.state.select(CellPos::new(0, 0), CellPos::new(0, 1));
        app.keymap.remap_component(
            SELECTION_ID,
            SELECTION_RIGHT,
            Chord::key(KeyCode::Char('p')),
        );
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 1)))
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('p')));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 2)))
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));

        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 0), CellPos::new(0, 2))),
            "plain Right remains owned by horizontal panning"
        );
    }

    #[test]
    fn selection_bindings_do_not_run_outside_the_viewport_focus_or_modal_scope() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdef")]);
        app.other_focusable = true;
        let mut runtime = selection_runtime(app);
        runtime.set_focus(Some(OTHER_ID));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(runtime.app().state.selection, None);

        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdef")]);
        app.modal_open = true;
        let mut runtime = selection_runtime(app);
        assert_eq!(runtime.focus(), Some(MODAL_CONTROL));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert_eq!(runtime.app().state.selection, None);
    }

    #[test]
    fn selection_reveals_both_axes_and_pauses_follow_without_moving_the_caret() {
        let mut app =
            SelectionViewportApp::new(vec![
                ViewportLine::Plain("012345678901234567890123456789");
                20
            ]);
        app.area = Rect::new(0, 0, 12, 4);
        app.pan = true;
        app.state.scroll.apply_layout(4, app.lines.len());
        app.state.scroll.scroll_to(5);
        app.state.set_caret(Some(CellPos::new(3, 2)));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Down, KeyModifiers::SHIFT),
        );

        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(7, 30), CellPos::new(8, 30)))
        );
        assert!(runtime.app().state.scroll.offset() > 5);
        assert!(runtime.app().state.horizontal_offset() > 0);
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(3, 2)));

        let mut tail = SelectionViewportApp::new(vec![
            ViewportLine::Plain("long enough to pan"),
            ViewportLine::Plain("long enough to pan"),
            ViewportLine::Plain("long enough to pan"),
        ]);
        tail.area = Rect::new(0, 0, 12, 4);
        tail.pan = true;
        tail.state.set_follow(true);
        let mut runtime = selection_runtime(tail);
        assert!(runtime.app().state.scroll.at_end());
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(runtime.app().state.follow_cursor(), FollowCursor::OnMove);
        assert!(runtime.app().state.scroll.at_end());
    }

    #[test]
    fn retained_selection_survives_append_resize_wrap_and_source_rows() {
        let mut app = SelectionViewportApp::new(vec![
            ViewportLine::Plain("zero"),
            ViewportLine::Plain("one"),
            ViewportLine::Plain("target"),
        ]);
        app.state.select(CellPos::new(1, 1), CellPos::new(2, 4));
        let mut runtime = selection_runtime(app);

        runtime
            .app_mut()
            .lines
            .push(ViewportLine::Plain("appended"));
        runtime.app_mut().area = Rect::new(0, 0, 9, 3);
        runtime.app_mut().wrap = true;
        runtime.app_mut().state.invalidate();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(1, 1), CellPos::new(2, 3)))
        );

        runtime.app_mut().state.retained(1);
        runtime.app_mut().lines.remove(0);
        runtime.app_mut().state.invalidate();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 1), CellPos::new(1, 2)))
        );
    }

    #[test]
    fn retention_drops_keyboard_browse_with_a_fully_dropped_selection() {
        let mut state = ViewportState::default();
        state.select(CellPos::new(0, 0), CellPos::new(1, 2));

        state.retained(2);

        assert_eq!(state.selection, None);
        assert_eq!(state.browse, None);
    }

    #[test]
    fn escape_clears_keyboard_browse_and_unselected_escape_stays_unhandled() {
        let mut runtime = selection_runtime(SelectionViewportApp::new(vec![ViewportLine::Plain(
            "abcdef",
        )]));
        runtime
            .app_mut()
            .state
            .select(CellPos::new(0, 1), CellPos::new(0, 3));
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let response = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        assert!(response.is_consumed());
        assert_eq!(runtime.app().state.selection, None);
        assert_eq!(runtime.app().state.browse, None);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 5)))
        );

        let mut empty = selection_runtime(SelectionViewportApp::new(vec![ViewportLine::Plain(
            "abcdef",
        )]));
        let response = crate::runtime::stub::deliver(&mut empty, key(KeyCode::Esc));
        assert!(!response.is_consumed());
        assert_eq!(empty.app().state.selection, None);
    }

    struct ViewportApp {
        state: ViewportState,
    }

    impl Default for ViewportApp {
        fn default() -> Self {
            let mut state = ViewportState::default();
            state.set_follow(false);
            ViewportApp { state }
        }
    }

    impl App for ViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .update(cx, &mut self.state, &MANY_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextViewport::new(ID).draw(ui, POINTER_AREA, &self.state, &MANY_LINES);
        }
    }

    struct HorizontalWheelProbe {
        state: ViewportState,
        keymap: crate::keymap::KeyMap,
    }

    impl HorizontalWheelProbe {
        fn new(state: ViewportState) -> Self {
            HorizontalWheelProbe {
                state,
                keymap: crate::keymap::KeyMap::new(),
            }
        }
    }

    impl App for HorizontalWheelProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .horizontal_pan(true)
                .update(cx, &mut self.state, &LONG_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextViewport::new(ID).horizontal_pan(true).draw(
                ui,
                POINTER_AREA,
                &self.state,
                &LONG_LINES,
            );
        }

        fn keymap(&self) -> &crate::keymap::KeyMap {
            &self.keymap
        }
    }

    struct MutableViewportApp {
        state: ViewportState,
        lines: Vec<&'static str>,
        area: Rect,
        wrap: bool,
    }

    impl MutableViewportApp {
        fn view_lines(lines: &[&'static str]) -> Vec<ViewportLine<'static>> {
            lines.iter().copied().map(ViewportLine::Plain).collect()
        }
    }

    impl App for MutableViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let lines = Self::view_lines(&self.lines);
            TextViewport::new(ID)
                .wrap(self.wrap)
                .update(cx, &mut self.state, &lines)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let lines = Self::view_lines(&self.lines);
            TextViewport::new(ID)
                .wrap(self.wrap)
                .draw(ui, self.area, &self.state, &lines);
        }
    }

    struct GutterlessViewportApp {
        state: ViewportState,
    }

    const GUTTERLESS_LINES: [ViewportLine<'static>; 5] = [
        ViewportLine::Plain("aa bb"),
        ViewportLine::Plain("row 1"),
        ViewportLine::Plain("row 2"),
        ViewportLine::Plain("row 3"),
        ViewportLine::Plain("row 4"),
    ];

    impl App for GutterlessViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .reserve_gutter(false)
                .click_selects_word(true)
                .update(cx, &mut self.state, &GUTTERLESS_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextViewport::new(ID)
                .reserve_gutter(false)
                .click_selects_word(true)
                .draw(ui, Rect::new(0, 0, 12, 4), &self.state, &GUTTERLESS_LINES);
        }
    }

    struct ExtremeGapViewportApp {
        state: ViewportState,
    }

    struct ClippedWrappedViewportApp {
        state: ViewportState,
        clip: Rect,
    }

    struct ClippedWordViewportApp {
        state: ViewportState,
        clip: Rect,
    }

    const CLIPPED_WRAPPED_AREA: Rect = Rect::new(0, 0, 24, 4);
    const CLIPPED_WRAPPED_LINES: [ViewportLine<'static>; 1] =
        [ViewportLine::Plain("abcdefghijklmnopqrstuv WORD")];
    const CLIPPED_WORD_AREA: Rect = Rect::new(0, 0, 12, 4);
    const CLIPPED_WORD_LINES: [ViewportLine<'static>; 1] = [ViewportLine::Plain("aa bb")];

    impl App for ClippedWrappedViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .wrap(true)
                .click_selects_word(true)
                .update(cx, &mut self.state, &CLIPPED_WRAPPED_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.with_area(self.clip, |ui| {
                TextViewport::new(ID)
                    .wrap(true)
                    .click_selects_word(true)
                    .draw(
                        ui,
                        CLIPPED_WRAPPED_AREA,
                        &self.state,
                        &CLIPPED_WRAPPED_LINES,
                    );
            });
        }
    }

    impl App for ClippedWordViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .click_selects_word(true)
                .update(cx, &mut self.state, &CLIPPED_WORD_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.with_area(self.clip, |ui| {
                TextViewport::new(ID).click_selects_word(true).draw(
                    ui,
                    CLIPPED_WORD_AREA,
                    &self.state,
                    &CLIPPED_WORD_LINES,
                );
            });
        }
    }

    impl App for ExtremeGapViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            TextViewport::new(ID)
                .scroll_gap(u16::MAX)
                .update(cx, &mut self.state, &GUTTERLESS_LINES)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            TextViewport::new(ID).scroll_gap(u16::MAX).draw(
                ui,
                Rect::new(0, 0, 1, 1),
                &self.state,
                &GUTTERLESS_LINES,
            );
        }
    }

    #[test]
    fn track_registration_uses_the_full_bar_geometry() {
        let mut runtime = Runtime::new(ViewportApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let track = runtime
            .area_of_part(ID, PartRef::of(Part::TRACK))
            .expect("the overflowing viewport must register its track");
        assert_eq!(track, Rect::new(11, 0, 1, POINTER_AREA.height));

        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Down, track.x, track.bottom() - 1),
        );
        assert!(runtime.app().state.scroll.at_end());
    }

    #[test]
    fn horizontal_wheel_is_not_reapplied_as_vertical_scroll() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Wheel(Axis::H, 1), 4, 3));

        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), 0);
    }

    #[test]
    fn horizontal_pan_keys_move_four_cells_and_leave_vertical_position_alone() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));
        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), 0);

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Left));
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
        assert_eq!(runtime.app().state.scroll().offset(), 0);
    }

    #[test]
    fn pan_remap_disables_the_old_arrow_and_caret_ownership_has_no_raw_fallback() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        state.set_caret(Some(CellPos::new(0, 0)));
        let mut keymap = crate::keymap::KeyMap::new();
        keymap.remap_component(ID, HORIZONTAL_PAN_RIGHT, Chord::key(KeyCode::Char('p')));
        let mut app = HorizontalWheelProbe::new(state);
        app.keymap = keymap;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('p')));
        assert_eq!(runtime.app().state.horizontal_offset(), 4);
    }

    #[test]
    fn horizontal_edge_drag_pans_without_changing_logical_selection_or_copy() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        state.horizontal.set_content(30);
        state.horizontal.set_viewport(10);
        state.horizontal.scroll_to(4);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 3, 1));
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 11, 1));
        assert_eq!(runtime.app().state.horizontal_offset(), 5);
        assert_eq!(
            runtime.app().state.selection(),
            Some((CellPos::new(1, 6), CellPos::new(1, 14)))
        );
        let mut copied = String::new();
        assert!(runtime.app().state.copy_into(&LONG_LINES, &mut copied));
        assert_eq!(copied, "67890123");
    }

    #[test]
    fn vertical_wheel_remains_vertical_inside_the_combined_region() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Wheel(Axis::V, 1), 4, 3));

        assert_eq!(runtime.app().state.scroll().offset(), 3);
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
    }

    #[test]
    fn opted_in_text_registers_both_axes_with_independent_headroom_and_a_blank_corner() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        state.scroll.apply_layout(7, LONG_LINES.len());
        state.scroll.scroll_to(6);
        state.horizontal.set_content(30);
        state.horizontal.set_viewport(10);
        state.horizontal.scroll_to(4);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let text_point = Position::new(4, 2);
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(text_point, Axis::V)
                .map(|hit| hit.owner),
            Some(ID)
        );
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(text_point, Axis::H)
                .map(|hit| hit.owner),
            Some(ID)
        );
        let both = runtime
            .registry()
            .regions()
            .iter()
            .find(|region| {
                region.owner == ID
                    && region.kind == crate::ui::hit::RegionKind::Scroll
                    && region.scroll().is_some_and(|(axes, _)| axes == Axes::Both)
            })
            .expect("opted-in body registers one combined-axis scroll region");
        let (axes, headroom) = both.scroll().expect("combined region carries headroom");
        assert_eq!(axes, Axes::Both);
        assert_eq!((headroom.up, headroom.down), (6, 87));
        assert_eq!((headroom.left, headroom.right), (4, 16));

        let track = runtime
            .area_of_part(ID, PartRef::of(HORIZONTAL_TRACK))
            .expect("horizontal overflow paints and registers its bottom track");
        assert_eq!(track, Rect::new(1, 7, 10, 1));
        assert!(
            !track.contains(Position::new(11, 7)),
            "the bars' corner stays blank"
        );
    }

    #[test]
    fn horizontal_thumb_drag_tracks_the_logical_offset_and_releases_outside() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let track = runtime
            .area_of_part(ID, PartRef::of(HORIZONTAL_TRACK))
            .expect("horizontal track is registered");
        let thumb = runtime
            .area_of_part(ID, PartRef::of(HORIZONTAL_THUMB))
            .expect("horizontal thumb is registered separately from the vertical bar");
        let grab = Position::new(thumb.x, thumb.y);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, grab.x, grab.y));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Drag, track.right().saturating_sub(1), track.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 20);
        assert_eq!(runtime.app().state.scroll().offset(), 0);

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 30, 20));
        assert_eq!(runtime.app().state.horizontal_offset(), 20);
    }

    #[test]
    fn horizontal_track_page_press_reports_repaint_only_when_offset_changes() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(HorizontalWheelProbe::new(state), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(POINTER_AREA);
        runtime
            .draw_buffer(POINTER_AREA, &mut buffer)
            .commit_presented();

        let track = runtime
            .area_of_part(ID, PartRef::of(HORIZONTAL_TRACK))
            .expect("horizontal track is registered");
        let thumb = runtime
            .area_of_part(ID, PartRef::of(HORIZONTAL_THUMB))
            .expect("horizontal thumb is registered");
        let page = Position::new(track.right().saturating_sub(1), track.y);
        assert!(page.x >= thumb.right(), "press lands beyond the thumb");

        let response =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, page.x, page.y));
        assert!(runtime.app().state.horizontal_offset() > 0);
        assert!(response.is_consumed());
        assert!(response.is_changed(), "a changed page press must repaint");

        let mut edge_state = ViewportState::default();
        edge_state.horizontal.set_content(30);
        edge_state.horizontal.set_viewport(10);
        edge_state
            .horizontal
            .scroll_to(edge_state.horizontal.max_offset());
        let intents = crate::intent::IntentQueue::new();
        let mut services = crate::ui::cx::FrameServices::default();
        let mut core = crate::ui::UiCore::default();
        let last = crate::ui::cx::LastFrame::default();
        let theme = Theme::junie();
        let mut cx = crate::ui::Cx::new_with_cause(
            &intents,
            &mut services,
            &mut core,
            &last,
            &theme,
            None,
            crate::runtime::UpdateCause::Event,
        );
        let mut acc = Acc::new();
        TextViewport::new(ID)
            .horizontal_pan(true)
            .handle_horizontal_pointer(
                &mut cx,
                &mut edge_state,
                Rect::new(0, 0, 10, 1),
                crate::intent::Intent::Pointer {
                    phase: Phase::Press,
                    part: PartRef::of(HORIZONTAL_TRACK),
                    pos: Position::new(9, 0),
                    local: Position::new(9, 0),
                    mods: crate::event::KeyModifiers::NONE,
                },
                &mut acc,
            );
        let response = acc.finish(ID);
        assert!(response.is_consumed());
        assert!(
            !response.is_changed(),
            "an unchanged clamped track target needs no repaint"
        );
    }

    #[test]
    fn pan_window_omits_wide_graphemes_crossed_by_either_edge() {
        let render = |text: &str, offset: usize| {
            let lines = [ViewportLine::Plain(text)];
            let mut state = ViewportState::default();
            state.horizontal.set_content(line_cols(lines[0]));
            state.horizontal.set_viewport(3);
            state.horizontal.scroll_to(offset);
            let area = Rect::new(0, 0, 5, 3);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    TextViewport::new(ID)
                        .horizontal_pan(true)
                        .draw(ui, area, &state, &lines);
                })
                .commit_presented();
            (0..3)
                .map(|x| {
                    buffer
                        .cell(Position::new(x + 1, 0))
                        .map_or_else(String::new, |cell| cell.symbol().to_owned())
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(render("a漢b界c", 2), [" ", "b", " "]);
        assert_eq!(render("ab漢c", 0), ["a", "b", " "]);
        assert_eq!(render("xe\u{301}yz", 1), ["e\u{301}", "y", "z"]);
        assert_eq!(render("x👩\u{200d}💻yz", 2), [" ", "y", "z"]);
        assert_eq!(render("x🇺🇸yz", 2), [" ", "y", "z"]);
        assert_eq!(render("ab\tcd", 4), [" ", " ", "c"]);
    }

    #[test]
    fn horizontal_pan_keeps_the_style_of_the_visible_projected_run() {
        let spans = [
            Span::new("ab").role(Role::Success).modifier(Modifier::BOLD),
            Span::new("漢cdef")
                .role(Role::Danger)
                .modifier(Modifier::UNDERLINED),
        ];
        let lines = [ViewportLine::Spans(&spans)];
        let mut state = ViewportState::default();
        state.horizontal.set_content(8);
        state.horizontal.set_viewport(4);
        state.horizontal.scroll_to(2);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                TextViewport::new(ID).horizontal_pan(true).draw(
                    ui,
                    Rect::new(0, 0, 6, 3),
                    &state,
                    &lines,
                );
            })
            .commit_presented();

        let visible = buffer
            .cell(Position::new(1, 0))
            .expect("the first panned cell is visible");
        assert_eq!(visible.symbol(), "漢");
        assert!(visible.modifier.contains(Modifier::UNDERLINED));
        assert!(!visible.modifier.contains(Modifier::BOLD));
        let following = buffer
            .cell(Position::new(3, 0))
            .expect("the following styled cell is visible");
        assert_eq!(following.symbol(), "c");
        assert!(following.modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn graphemes_split_across_styled_runs_keep_the_leading_style() {
        let check = |first: &'static str, rest: &'static str, grapheme: &'static str| {
            let spans = [
                Span::new(first)
                    .role(Role::Success)
                    .modifier(Modifier::BOLD),
                Span::new(rest)
                    .role(Role::Danger)
                    .modifier(Modifier::UNDERLINED),
            ];
            let lines = [ViewportLine::Spans(&spans)];
            assert_eq!(
                line_slice(lines[0], 0, grapheme_width(grapheme) as usize),
                grapheme
            );

            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    TextViewport::new(ID).draw(
                        ui,
                        Rect::new(0, 0, 8, 2),
                        &ViewportState::default(),
                        &lines,
                    );
                })
                .commit_presented();

            let combined = buffer
                .cell(Position::new(1, 0))
                .expect("the combined grapheme is visible");
            assert_eq!(combined.symbol(), grapheme);
            assert!(combined.modifier.contains(Modifier::BOLD));
            assert!(!combined.modifier.contains(Modifier::UNDERLINED));
            let next_x = 1 + grapheme_width(grapheme);
            let following = buffer
                .cell(Position::new(next_x, 0))
                .expect("the next grapheme is visible");
            assert_eq!(following.symbol(), "x");
            assert!(following.modifier.contains(Modifier::UNDERLINED));
            assert!(!following.modifier.contains(Modifier::BOLD));
        };

        check("e", "\u{301}x", "e\u{301}");
        check("👩", "\u{200d}💻x", "👩\u{200d}💻");
        check("🇺", "🇸x", "🇺🇸");
    }

    #[test]
    fn pointer_mapping_uses_current_lines_after_invalidation() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(
            MutableViewportApp {
                state,
                lines: vec!["012345678901234567890", "old second line"],
                area: Rect::new(0, 0, 12, 4),
                wrap: true,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let app = runtime.app_mut();
        app.lines = vec!["short", "target line"];
        app.state.invalidate();

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 1, 1));
        assert!(runtime.app().state.anchor.is_some(), "press did not anchor");
        assert_eq!(runtime.capture_owner(), Some(ID), "press did not capture");
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 4, 1));

        assert!(
            runtime.app().state.selection.is_some(),
            "drag did not set a selection"
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(1, 0), CellPos::new(1, 3)))
        );
        assert_eq!(
            runtime.app().state.selection(),
            Some((CellPos::new(1, 0), CellPos::new(1, 3)))
        );
    }

    #[test]
    fn update_uses_the_current_content_length() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(
            MutableViewportApp {
                state,
                lines: vec!["row"; 2],
                area: POINTER_AREA,
                wrap: false,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        runtime.app_mut().lines = vec!["row"; 100];
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::End));

        assert_eq!(runtime.app().state.scroll.offset(), 92);
    }

    #[test]
    fn bottom_track_click_and_thumb_drag_reach_the_end() {
        let mut runtime = Runtime::new(ViewportApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let track = runtime
            .area_of_part(ID, PartRef::of(Part::TRACK))
            .unwrap_or(Rect::ZERO);
        let bottom = track.bottom().saturating_sub(1);
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, track.x, bottom));
        assert!(runtime.app().state.scroll.at_end());

        runtime.app_mut().state.scroll.scroll_to(40);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let thumb = runtime
            .area_of_part(ID, PartRef::of(Part::THUMB))
            .unwrap_or(Rect::ZERO);
        let grab_y = thumb.bottom().saturating_sub(1);
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, thumb.x, grab_y));
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, track.x, bottom));
        assert!(runtime.app().state.scroll.at_end());
    }

    #[test]
    fn offscreen_caret_emits_no_cursor_while_visible_caret_does() {
        let lines: Vec<ViewportLine<'_>> = (0..30).map(|_| ViewportLine::Plain("row")).collect();
        let area = Rect::new(0, 0, 12, 4);
        let render = |caret: CellPos, level: ColorLevel| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(level));
            let mut buffer = Buffer::empty(SCREEN);
            let mut state = ViewportState::default();
            state.set_follow(false);
            state
                .scroll
                .apply_layout(usize::from(area.height), lines.len());
            state.scroll.scroll_to(5);
            state.set_caret(Some(caret));
            for _ in 0..2 {
                runtime
                    .draw_scene(SCREEN, &mut buffer, |ui, _| {
                        TextViewport::new(ID).draw(ui, area, &state, &lines);
                    })
                    .commit_presented();
            }
            runtime.cursor()
        };
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            assert_eq!(
                render(CellPos::new(0, 0), level),
                None,
                "caret above viewport"
            );
            assert_eq!(
                render(CellPos::new(20, 0), level),
                None,
                "caret below viewport"
            );
            assert_eq!(
                render(CellPos::new(6, 1), level),
                Some(Position::new(2, 1)),
                "caret position changed at {level:?}"
            );
        }
    }

    #[test]
    fn append_extends_the_wrapped_index_without_rebuilding_its_prefix() {
        let key = LayoutKey {
            width: 4,
            wrap: true,
            generation: 0,
            layout_revision: 0,
        };
        let mut layout = ViewportLayout::default();
        let first = [ViewportLine::Plain("abcdefgh"), ViewportLine::Plain("ij")];
        layout.ensure(&first, key);
        layout.prefix[1] = 777;
        let appended = [
            ViewportLine::Plain("abcdefgh"),
            ViewportLine::Plain("ij"),
            ViewportLine::Plain("klmnop"),
        ];
        layout.ensure(&appended, key);
        assert_eq!(layout.prefix[1], 777, "append rebuilt the existing prefix");
        assert_eq!(layout.prefix.len(), 4);
    }

    #[test]
    fn ensure_work_is_cold_zero_warm_suffix_on_append_and_full_on_reflow() {
        let lines = [ViewportLine::Plain("abcdefgh"); 4];
        let mut layout = ViewportLayout::default();
        let key = LayoutKey {
            width: 4,
            wrap: true,
            generation: 0,
            layout_revision: 0,
        };

        assert_eq!(layout.ensure(&lines, key), lines.len());
        assert_eq!(layout.ensure(&lines, key), 0);

        let appended = [ViewportLine::Plain("abcdefgh"); 7];
        assert_eq!(layout.ensure(&appended, key), 3);

        assert_eq!(
            layout.ensure(
                &appended,
                LayoutKey {
                    generation: 1,
                    ..key
                },
            ),
            appended.len()
        );

        assert_eq!(
            layout.ensure(
                &appended,
                LayoutKey {
                    width: 5,
                    generation: 1,
                    ..key
                },
            ),
            appended.len()
        );
    }

    #[cfg(feature = "testing")]
    #[test]
    fn work_probe_is_isolated_resettable_and_reports_cold_warm_work() {
        let first = ViewportWorkProbe::default();
        let other = ViewportWorkProbe::default();
        let lines = [ViewportLine::Plain("abcdefgh"); 7];
        let viewport = TextViewport::new(ID).wrap(true).work_probe(&first);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                viewport.draw(ui, area, &ViewportState::default(), &lines);
            })
            .commit_presented();
        let cold = first.snapshot();
        assert_eq!(cold.indexed_lines, lines.len());
        assert!(cold.visible_rows > 0);
        assert_eq!(other.snapshot(), ViewportWorkSnapshot::default());

        first.reset();
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                viewport.draw(ui, area, &ViewportState::default(), &lines);
            })
            .commit_presented();
        let warm = first.snapshot();
        assert_eq!(warm.indexed_lines, 0);
        assert!(warm.visible_rows > 0);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn work_probes_are_parallel_caller_owned() {
        let run = || {
            let probe = ViewportWorkProbe::default();
            let lines = [ViewportLine::Plain("parallel"); 5];
            let viewport = TextViewport::new(ID).wrap(true).work_probe(&probe);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, area| {
                    viewport.draw(ui, area, &ViewportState::default(), &lines);
                })
                .commit_presented();
            probe.snapshot()
        };
        let a = std::thread::spawn(run);
        let b = std::thread::spawn(run);
        assert_eq!(a.join().expect("first probe thread").indexed_lines, 5);
        assert_eq!(b.join().expect("second probe thread").indexed_lines, 5);
    }

    #[test]
    fn wrapped_prefix_matches_bruteforce() {
        let spans = [
            Span::new("ab\t").role(Role::Info),
            Span::new("漢e\u{301}").modifier(Modifier::BOLD),
        ];
        let lines = [
            ViewportLine::Plain(""),
            ViewportLine::Plain("ascii"),
            ViewportLine::Plain("漢字"),
            ViewportLine::Plain("e\u{301}clair"),
            ViewportLine::Spans(&spans),
        ];
        for width in 1..=12 {
            let key = LayoutKey {
                width,
                wrap: true,
                generation: u32::from(width),
                layout_revision: 0,
            };
            let mut layout = ViewportLayout::default();
            layout.ensure(&lines, key);
            let mut expected = vec![0usize];
            for line in lines {
                let next = expected
                    .last()
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(line_rows(line, width, true));
                expected.push(next);
            }
            assert_eq!(layout.prefix, expected, "width {width}");
            for (line, pair) in expected.windows(2).enumerate() {
                for visual in pair[0]..pair[1] {
                    assert_eq!(
                        layout.row_start(&lines, visual),
                        (line, visual.saturating_sub(pair[0])),
                        "width {width}, visual row {visual}"
                    );
                }
            }
        }
    }

    #[test]
    fn generation_saturation_never_reuses_stale_layout() {
        let mut layout = ViewportLayout::default();
        let key = LayoutKey {
            width: 4,
            wrap: true,
            generation: u32::MAX,
            layout_revision: 0,
        };
        let before = [ViewportLine::Plain("a"), ViewportLine::Plain("b")];
        let after = [ViewportLine::Plain("abcdefgh"), ViewportLine::Plain("b")];
        layout.ensure(&before, key);
        assert_eq!(layout.total(), 2);
        assert_eq!(layout.ensure(&after, key), after.len());
        assert_eq!(layout.total(), 3);

        let mut state = ViewportState {
            generation: u32::MAX - 1,
            ..ViewportState::default()
        };
        state.invalidate();
        state.invalidate();
        assert_eq!(state.generation, u32::MAX);
    }

    #[test]
    fn layout_revision_remeasures_changed_content_and_saturation_never_reuses() {
        let short = [ViewportLine::Plain("x")];
        let medium = [ViewportLine::Plain("012345")];
        let long = [ViewportLine::Plain("0123456789")];
        let mut layout = ViewportLayout::default();
        let key = LayoutKey {
            width: 4,
            wrap: false,
            generation: 7,
            layout_revision: 1,
        };

        layout.ensure_with_widest(&short, key, true);
        assert_eq!(layout.widest_cols, 1);

        let next = LayoutKey {
            layout_revision: 2,
            ..key
        };
        layout.ensure_with_widest(&medium, next, true);
        assert_eq!(layout.widest_cols, 6);

        let saturated = LayoutKey {
            layout_revision: u64::MAX,
            ..next
        };
        layout.ensure_with_widest(&short, saturated, true);
        assert_eq!(layout.widest_cols, 1);
        layout.ensure_with_widest(&long, saturated, true);
        assert_eq!(layout.widest_cols, 10);
    }

    #[test]
    fn same_length_offscreen_rewrap_after_invalidate_updates_total_end_thumb_and_mapping() {
        let before = [ViewportLine::Plain("row"); 8];
        let mut after = before;
        after[5] = ViewportLine::Plain("01234567890123456789");
        let width = 4;
        let viewport = 4;
        let track = 6;
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut layout = ViewportLayout::default();
        let old_key = LayoutKey {
            width,
            wrap: true,
            generation: state.generation,
            layout_revision: 0,
        };
        layout.ensure(&before, old_key);
        let old_total = layout.total();
        let mut old_scroll = state.scroll;
        old_scroll.apply_layout(viewport, old_total);
        let (_, old_thumb) = old_scroll.thumb(track);

        state.invalidate();
        let new_key = LayoutKey {
            generation: state.generation,
            ..old_key
        };
        layout.ensure(&after, new_key);
        let new_total = layout.total();
        state.scroll.apply_layout(viewport, new_total);
        let (_, new_thumb) = state.scroll.thumb(track);
        assert_eq!(old_total, 8);
        assert_eq!(new_total, 12);
        assert_eq!(state.scroll.max_offset(), new_total - viewport);
        assert!(
            new_thumb < old_thumb,
            "a larger total did not shorten the thumb"
        );

        let mut acc = Acc::new();
        TextViewport::command(&mut state, &after, ViewportCmd::End, &mut acc);
        assert!(state.follow());
        assert_eq!(state.scroll.offset(), state.scroll.max_offset());

        state.scroll.scroll_to(0);
        let track_pos = track.saturating_sub(1);
        state
            .scroll
            .scroll_to(state.scroll.offset_for_track_pos(track_pos, track));
        assert!(
            state.scroll.at_end(),
            "bottom track cap did not reach the end"
        );
        assert_eq!(layout.row_start(&after, 7), (5, 2));
    }

    #[test]
    fn copy_command_carries_the_selected_text_in_the_exact_action() {
        let lines = [ViewportLine::Plain("alpha beta")];
        let mut st = ViewportState::default();
        st.select(CellPos::new(0, 6), CellPos::new(0, 10));
        let mut acc = Acc::new();
        TextViewport::command(&mut st, &lines, ViewportCmd::Copy, &mut acc);
        assert_eq!(
            acc.finish(ID).into_action(),
            Some(ViewportAction::Copy("beta".to_owned()))
        );
    }

    #[test]
    fn first_end_emits_follow_changed() {
        let mut st = ViewportState::default();
        st.set_follow(false);
        let mut acc = Acc::new();

        TextViewport::command(&mut st, &[], ViewportCmd::End, &mut acc);

        assert_eq!(
            acc.finish(ID).into_action(),
            Some(ViewportAction::FollowChanged(true))
        );
    }

    #[test]
    fn repeated_end_repaints_without_emitting_follow_changed() {
        let mut st = ViewportState::default();
        let mut acc = Acc::new();

        TextViewport::command(&mut st, &[], ViewportCmd::End, &mut acc);
        let response = acc.finish(ID);

        assert!(response.is_consumed());
        assert!(response.is_changed());
        assert_eq!(response.into_action(), None);
    }

    #[test]
    fn parts_match_the_exact_styling_surface() {
        assert_eq!(
            TextViewport::PARTS,
            &[
                Part::CONTAINER,
                Part::TEXT,
                Part::GUTTER,
                Part::TRACK,
                Part::THUMB,
            ]
        );
    }

    #[test]
    fn a_reference_viewport_makes_its_nested_scroll_region_inert() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = ViewportState::default();
        let lines: Vec<ViewportLine<'_>> = (0..30).map(|_| ViewportLine::Plain("row")).collect();
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            ui.reference(
                Some(ReferenceTarget::new(ID, ReferenceState::FOCUSED)),
                |ui| {
                    TextViewport::new(ID).draw(ui, Rect::new(0, 0, 12, 3), &st, &lines);
                },
            );
        })
        .commit_presented();
        assert!(rt.area_of(ID).is_none());
    }

    #[test]
    fn nested_scrollbar_resolves_through_the_viewport_family() {
        let theme = Theme::junie()
            .override_family(Family::VIEWPORT, |recipe| {
                recipe
                    .part(Part::TRACK)
                    .base(StylePatch::new().set_fg(Role::Danger));
            })
            .override_family(Family::SCROLLBAR, |recipe| {
                recipe
                    .part(Part::TRACK)
                    .base(StylePatch::new().set_fg(Role::Info));
            });
        let expected = theme.resolve(
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::TRACK,
            StateFlags::empty(),
            Surface::Canvas,
        );
        let scrollbar = theme.resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::TRACK,
            StateFlags::empty(),
            Surface::Canvas,
        );
        let mut rt = Runtime::new(Stub::default(), theme);
        let mut buf = Buffer::empty(SCREEN);
        let st = ViewportState::default();
        let lines = [ViewportLine::Plain("row"); 30];
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            TextViewport::new(ID).draw(ui, POINTER_AREA, &st, &lines);
        })
        .commit_presented();

        assert_eq!(rt.resolved(ID, Part::TRACK), expected);
        assert_ne!(rt.resolved(ID, Part::TRACK), scrollbar);
    }

    #[test]
    fn retention_fixes_up_selection_and_caret() {
        let mut st = ViewportState::default();
        st.select(CellPos::new(7, 2), CellPos::new(9, 4));
        st.anchor = Some(CellPos::new(7, 2));
        st.set_caret(Some(CellPos::new(9, 1)));

        st.retained(5);
        assert_eq!(
            st.selection(),
            Some((CellPos::new(2, 2), CellPos::new(4, 4))),
            "the selection did not move with the buffer"
        );
        assert_eq!(st.anchor, Some(CellPos::new(2, 2)));
        assert_eq!(st.caret(), Some(CellPos::new(4, 1)));

        let mut partial = ViewportState::default();
        partial.select(CellPos::new(1, 3), CellPos::new(4, 6));
        partial.retained(3);
        assert_eq!(
            partial.selection(),
            Some((CellPos::new(0, 0), CellPos::new(1, 6)))
        );

        st.retained(10);
        assert_eq!(
            st.selection(),
            None,
            "a fully dropped selection must collapse, not name line 0 twice"
        );
        assert_eq!(st.caret(), Some(CellPos::new(0, 0)));

        let before = st.generation;
        st.retained(1);
        assert_ne!(st.generation, before);

        let g = st.generation;
        st.retained(0);
        assert_eq!(st.generation, g);
    }

    #[test]
    fn wrapped_rows_partition_the_line() {
        let text = "the quick brown fox jumps over the lazy dog and keeps going";
        let line = ViewportLine::Plain(text);
        let cols = line_cols(line);
        for w in [1u16, 2, 7, 13, 40, 200] {
            let rows = line_rows(line, w, true);
            assert!(rows >= 1);
            let mut next = 0usize;
            for r in 0..rows {
                let (a, b) = row_cols(line, w, true, r);
                assert_eq!(a, next, "w={w} row {r}: starts at {a}, expected {next}");
                let end = b.min(cols);
                assert!(end > a || cols == 0, "w={w} row {r}: empty row");
                assert!(
                    end.saturating_sub(a) <= usize::from(w).max(1),
                    "w={w} row {r}: {a}..{end} is wider than the viewport"
                );
                next = end;
            }
            assert_eq!(next, cols, "w={w}: the rows do not cover the line");
        }
    }

    #[test]
    fn copy_joins_lines_and_trims_each() {
        let lines = [
            ViewportLine::Plain("alpha beta   "),
            ViewportLine::Plain("gamma delta"),
            ViewportLine::Plain("epsilon"),
        ];
        let mut st = ViewportState::default();
        st.select(CellPos::new(0, 6), CellPos::new(2, 3));
        let mut out = String::new();
        assert!(st.copy_into(&lines, &mut out));
        assert_eq!(out, "beta\ngamma delta\neps");

        let mut st2 = ViewportState::default();
        let mut out2 = String::new();
        assert!(!st2.copy_into(&lines, &mut out2));
        assert!(out2.is_empty());
        assert!(!st2.clear_selection());
    }

    #[test]
    fn a_double_click_selects_a_path_like_word() {
        let line = ViewportLine::Plain("see src/main.rs:42 for the rest");
        assert_eq!(word_at(line, 5), Some((4, 15)));
        assert_eq!(
            "src/main.rs",
            line_slice(line, 4, 15),
            "the word bounds do not name the path"
        );

        assert_eq!(word_at(line, 3), None);

        assert_eq!(word_at(line, 999), None);
    }

    fn line_slice(line: ViewportLine<'_>, a: usize, b: usize) -> String {
        let mut out = String::new();
        let mut col = 0usize;
        walk_line(line, |c| {
            if col >= b {
                return ControlFlow::Break(());
            }
            if col >= a {
                out.push_str(cell_text(line, &c));
            }
            col = col.saturating_add(usize::from(c.w));
            ControlFlow::Continue(())
        });
        out
    }

    #[test]
    fn the_text_width_does_not_depend_on_whether_the_bar_is_painted() {
        let short = [ViewportLine::Plain("one"), ViewportLine::Plain("two")];
        let long: Vec<ViewportLine<'_>> = (0..50)
            .map(|_| ViewportLine::Plain("this line is wider than the viewport"))
            .collect();
        let themes = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ]
        .map(|level| Theme::junie().downgrade(level));
        let areas = [
            Rect::ZERO,
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 24, 1),
            Rect::new(0, 0, 24, 4),
        ];

        for theme in themes {
            for area in areas {
                let dimensions_and_last_row = |lines: &[ViewportLine<'_>]| {
                    let mut rt = Runtime::new(Stub::default(), theme.clone());
                    let mut buf = Buffer::empty(SCREEN);
                    let st = ViewportState::default();
                    let mut dimensions = (0, 0);
                    rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                        let text = TextViewport::new(ID).draw(ui, area, &st, lines);
                        dimensions = (text.width, text.height);
                    })
                    .commit_presented();
                    let last_cell = buf
                        .cell(Position::new(1, area.bottom().saturating_sub(1)))
                        .map_or_else(String::new, |cell| cell.symbol().to_owned());
                    (dimensions, last_cell)
                };
                let (short_dimensions, _) = dimensions_and_last_row(&short);
                let (long_dimensions, last_cell) = dimensions_and_last_row(&long);
                assert_eq!(short_dimensions, long_dimensions, "area={area:?}");
                assert_eq!(short_dimensions.1, area.height, "area={area:?}");
                if area == Rect::new(0, 0, 24, 1) || area == Rect::new(0, 0, 24, 4) {
                    assert_eq!(last_cell, "t", "area={area:?}: last row remains text");
                }
                assert_eq!(
                    short_dimensions.0,
                    TextViewport::text_width(area.width, true, 0),
                    "area={area:?}"
                );
            }
        }
    }

    #[test]
    fn gutterless_pointer_selection_uses_the_drawn_text_origin() {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut state = ViewportState::default();
            state.set_follow(false);
            let theme = Theme::junie().downgrade(level);
            let mut runtime = Runtime::new(GutterlessViewportApp { state }, theme);
            let _ = runtime.initialize();
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

            assert_eq!(
                buffer.cell(Position::new(3, 0)).map(Cell::symbol),
                Some("b"),
                "the visible b must occupy column 3 without a gutter ({level:?})"
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 3, 0));
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 0));

            assert_eq!(
                runtime.app().state.selection(),
                Some((CellPos::new(0, 3), CellPos::new(0, 5))),
                "hit mapping changed at {level:?}"
            );
        }
    }

    #[test]
    fn clipped_wrapped_pointer_uses_the_logical_draw_layout() {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for clip in [Rect::new(0, 0, 10, 4), Rect::new(0, 1, 10, 3)] {
                let mut state = ViewportState::default();
                state.set_follow(false);
                let mut runtime = Runtime::new(
                    ClippedWrappedViewportApp { state, clip },
                    Theme::junie().downgrade(level),
                );
                let _ = runtime.initialize();
                let mut buffer = Buffer::empty(SCREEN);
                runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

                assert_eq!(
                    buffer.cell(Position::new(2, 1)).map(Cell::symbol),
                    Some("W"),
                    "the visible W must be in the second visual row ({level:?}, {clip:?})"
                );
                let visible_text = runtime
                    .area_of_part(ID, PartRef::of(Part::TEXT))
                    .expect("the visible text hit area must be registered");
                assert_eq!(visible_text, clip.intersection(Rect::new(1, 0, 22, 4)));

                let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 2, 1));
                let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 1));

                assert_eq!(
                    runtime.app().state.selection(),
                    Some((CellPos::new(0, 23), CellPos::new(0, 27))),
                    "the clipped hit must map through the logical wrapped layout ({level:?}, {clip:?})"
                );
            }
        }
    }

    #[test]
    fn horizontally_clipped_pointer_uses_the_logical_text_origin() {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut state = ViewportState::default();
            state.set_follow(false);
            let mut runtime = Runtime::new(
                ClippedWordViewportApp {
                    state,
                    clip: Rect::new(4, 0, 4, 4),
                },
                Theme::junie().downgrade(level),
            );
            let _ = runtime.initialize();
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

            assert_eq!(
                buffer.cell(Position::new(4, 0)).map(Cell::symbol),
                Some("a"),
                "the clipped draw origin must remain visible ({level:?})"
            );
            let visible_text = runtime
                .area_of_part(ID, PartRef::of(Part::TEXT))
                .expect("the clipped text hit area must be registered");
            assert_eq!(visible_text, Rect::new(4, 0, 4, 4));

            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 4, 0));
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 4, 0));

            assert_eq!(
                runtime.app().state.selection(),
                Some((CellPos::new(0, 0), CellPos::new(0, 2))),
                "the visible hit must retain the logical column offset ({level:?})"
            );
        }
    }

    #[test]
    fn visible_caret_matches_wrapped_text_under_horizontal_clipping() {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut state = ViewportState::default();
            state.set_follow(false);
            state.set_caret(Some(CellPos::new(0, 23)));
            let mut runtime = Runtime::new(
                ClippedWrappedViewportApp {
                    state,
                    clip: Rect::new(4, 1, 8, 3),
                },
                Theme::junie().downgrade(level),
            );
            let _ = runtime.initialize();
            let area = Rect::new(0, 0, 30, 6);
            let mut buffer = Buffer::empty(area);
            for _ in 0..2 {
                runtime.draw_buffer(area, &mut buffer).commit_presented();
            }

            assert_eq!(
                buffer.cell(Position::new(5, 1)).map(Cell::symbol),
                Some("W"),
                "the caret target must remain rendered at {level:?}"
            );
            assert_eq!(runtime.focus(), Some(ID));
            assert_eq!(
                runtime.cursor_position(),
                Some(Position::new(5, 1)),
                "the cursor must follow the rendered wrapped character ({level:?})"
            );
        }
    }

    #[test]
    fn maximum_scroll_gap_saturates_for_measurement_update_and_draw() {
        let viewport = TextViewport::new(ID).scroll_gap(u16::MAX);
        assert_eq!(TextViewport::text_width(0, true, u16::MAX), 0);
        assert_eq!(TextViewport::text_width(u16::MAX, false, u16::MAX), 0);

        let lines = [ViewportLine::Plain("long content"); 32];
        let mut state = ViewportState::default();
        state.set_follow(false);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                assert_eq!(
                    viewport.measure(ui, Constraints::loose(0, 0)).preferred,
                    (0, 0)
                );
                assert_eq!(
                    viewport
                        .measure(ui, Constraints::loose(u16::MAX, u16::MAX))
                        .preferred
                        .0,
                    u16::MAX
                );
                for area in [Rect::ZERO, Rect::new(0, 0, 1, 1), Rect::new(0, 0, 12, 4)] {
                    viewport.draw(ui, area, &state, &lines);
                }
            })
            .commit_presented();

        let mut app_runtime = Runtime::new(
            ExtremeGapViewportApp {
                state: state.clone(),
            },
            Theme::junie(),
        );
        let _ = app_runtime.initialize();
        app_runtime
            .draw_scene(SCREEN, &mut buffer, |ui, _| {
                viewport.draw(ui, Rect::new(0, 0, 1, 1), &state, &lines);
            })
            .commit_presented();
        let _ = crate::runtime::stub::deliver(&mut app_runtime, key(KeyCode::End));
    }

    #[test]
    fn the_focus_gutter_is_the_only_focus_affordance_and_it_is_painted() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 3,
        };
        let lines = [ViewportLine::Plain("hello"), ViewportLine::Plain("world")];
        let bar = Theme::junie().design.glyphs.get(GlyphRole::FocusBar);
        let gutter = |state: ReferenceState| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ViewportState::default();
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                ui.reference(Some(ReferenceTarget::new(ID, state)), |ui| {
                    TextViewport::new(ID).draw(ui, area, &st, &lines);
                });
            })
            .commit_presented();
            buf.cell(Position::new(0, 0))
                .map_or_else(String::new, |c| c.symbol().to_owned())
        };
        assert_eq!(gutter(ReferenceState::FOCUSED), bar);
        assert_ne!(gutter(ReferenceState::default()), bar);
    }

    #[test]
    fn mono_selection_paints_the_resolved_inverse_underlined_style() {
        let area = Rect::new(0, 0, 8, 1);
        let lines = [ViewportLine::Plain("abcdef")];
        let mut state = ViewportState::default();
        state.select(CellPos::new(0, 1), CellPos::new(0, 3));

        for base in [Theme::junie(), Theme::paper()] {
            let theme = base.downgrade(ColorLevel::Mono);
            let expected = theme
                .resolve(
                    Family::VIEWPORT,
                    Variant::DEFAULT,
                    Part::TEXT,
                    StateFlags::SELECTED,
                    Surface::Canvas,
                )
                .style;
            let mut runtime = Runtime::new(Stub::default(), theme);
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    TextViewport::new(ID).draw(ui, area, &state, &lines);
                })
                .commit_presented();

            let base_cell = buffer.cell(Position::new(1, 0));
            let selected_cell = buffer.cell(Position::new(2, 0));
            let selected_end = buffer.cell(Position::new(3, 0));
            let after = buffer.cell(Position::new(4, 0));
            assert_ne!(selected_cell.map(Cell::style), base_cell.map(Cell::style));
            assert_eq!(
                selected_end.map(Cell::style),
                selected_cell.map(Cell::style)
            );
            assert_eq!(after.map(Cell::style), base_cell.map(Cell::style));
            assert_eq!(selected_cell.map(|cell| cell.fg), expected.fg);
            assert_eq!(selected_cell.map(|cell| cell.bg), expected.bg);
            assert!(selected_cell.is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED)));
        }
    }

    #[test]
    fn a_rebuilt_layout_paints_what_the_warm_one_did() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 18,
            height: 4,
        };
        let text = "a wrapped line long enough to need several visual rows";
        let lines = [ViewportLine::Plain(text), ViewportLine::Plain("tail")];
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut st = ViewportState::default();
        st.set_follow(false);
        let before = st.clone();

        let mut frames = Vec::new();
        for f in 0..3 {
            if f == 2 {
                st.invalidate();
            }
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                TextViewport::new(ID).wrap(true).draw(ui, area, &st, &lines);
            })
            .commit_presented();
            frames.push(buf);
        }
        assert_eq!(
            frames.get(1),
            frames.get(2),
            "a rebuilt layout paints a different frame from the warm one"
        );
        assert_eq!(
            st.scroll(),
            before.scroll(),
            "draw moved the caller's offset"
        );
        assert_eq!(st.selection(), before.selection());
    }

    #[test]
    fn tabs_expand_to_a_fixed_width_everywhere() {
        let line = ViewportLine::Plain("\tif x:");
        assert_eq!(line_cols(line), usize::from(TAB_WIDTH) + 5);
        let lines = [line];
        let mut st = ViewportState::default();
        st.select(
            CellPos::new(0, 0),
            CellPos::new(0, usize::from(TAB_WIDTH) + 2),
        );
        let mut out = String::new();
        assert!(st.copy_into(&lines, &mut out));
        assert_eq!(out, "    if", "the tab did not copy as its expansion");
    }

    #[test]
    fn every_declared_part_is_one_a_drawn_viewport_styles() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 3,
        };
        let lines: Vec<ViewportLine<'_>> = (0..30).map(|_| ViewportLine::Plain("row")).collect();
        let render = |patched: Option<Part>| {
            let ps: [(Part, StylePatch); 1] = [(
                patched.unwrap_or(Part::CONTAINER),
                StylePatch::new().set_fg(Role::Warning).set_bg(Role::Danger),
            )];
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ViewportState::default();
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut v = TextViewport::new(ID);
                if patched.is_some() {
                    v = v.patch_part(&ps);
                }
                v.draw(ui, area, &st, &lines);
            })
            .commit_presented();
            buf
        };
        let plain = render(None);
        for part in TextViewport::PARTS {
            assert_ne!(
                render(Some(*part)),
                plain,
                "TextViewport declares {part:?} and paints nothing with it"
            );
        }
    }

    #[test]
    fn the_slot_addressable_parts_are_exactly_the_documented_ones() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 3,
        };
        let lines: Vec<ViewportLine<'_>> = (0..30).map(|_| ViewportLine::Plain("row")).collect();
        let marker = |ui: &mut Ui<'_>, r: Rect| {
            let s = ui.surface_style();
            ui.paint_str(r, "ZZZZ", s);
        };
        let render = |slot: Option<Part>| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ViewportState::default();
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut v = TextViewport::new(ID);
                if let Some(part) = slot {
                    v = v.slot(part, &marker);
                }
                v.draw(ui, area, &st, &lines);
            })
            .commit_presented();
            buf
        };
        let plain = render(None);
        for part in [Part::GUTTER, Part::TRACK, Part::THUMB] {
            assert_ne!(
                render(Some(part)),
                plain,
                "`## Overrides` grants a slot on {part:?} and it is dropped"
            );
        }
        for part in [Part::CONTAINER, Part::TEXT] {
            assert_eq!(
                render(Some(part)),
                plain,
                "a slot on {part:?} changes cells, and `## Overrides` says it does not"
            );
        }
    }
}
