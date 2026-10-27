use core::fmt;
use core::ops::{ControlFlow, Range};

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
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::hit::ScrollRouteKey;
use termrock_runtime::runtime::{WheelRouteAttempt, WheelRouteResult};
use termrock_runtime::scroll_region::ScrollRegion;

const TAB_WIDTH: u16 = 4;

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
    paint: Option<PaintStyle>,
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
            self.push_run(value.as_ref(), role, modifier, None);
        }
        self.lines.push(start..self.runs.len());
    }

    pub fn push_styled_line<I, S>(&mut self, runs: I)
    where
        I: IntoIterator<Item = (S, PaintStyle)>,
        S: AsRef<str>,
    {
        let start = self.runs.len();
        for (value, paint) in runs {
            self.push_run(value.as_ref(), None, Modifier::empty(), Some(paint));
        }
        self.lines.push(start..self.runs.len());
    }

    fn push_run(
        &mut self,
        value: &str,
        role: Option<Role>,
        modifier: Modifier,
        paint: Option<PaintStyle>,
    ) {
        let text_start = self.text.len();
        self.text.push_str(value);
        self.runs.push(ProjectedRun {
            text: text_start..self.text.len(),
            role,
            modifier,
            paint,
        });
    }

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
enum FadePolicy {
    LegacyRect,
    LogicalWindow,
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

pub const HORIZONTAL_PAN_LEFT: crate::ActionKey =
    crate::ActionKey::custom("viewport.horizontal-pan-left");

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

pub const SELECTION_LEFT: crate::ActionKey = crate::ActionKey::custom("viewport.selection-left");

pub const SELECTION_RIGHT: crate::ActionKey = crate::ActionKey::custom("viewport.selection-right");

pub const SELECTION_UP: crate::ActionKey = crate::ActionKey::custom("viewport.selection-up");

pub const SELECTION_DOWN: crate::ActionKey = crate::ActionKey::custom("viewport.selection-down");

pub const SELECTION_HOME: crate::ActionKey = crate::ActionKey::custom("viewport.selection-home");

pub const SELECTION_END: crate::ActionKey = crate::ActionKey::custom("viewport.selection-end");

pub const SELECTION_WORD_LEFT_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-left-ctrl");

pub const SELECTION_WORD_RIGHT_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-right-ctrl");

pub const SELECTION_WORD_LEFT_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-left-alt");

pub const SELECTION_WORD_RIGHT_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-word-right-alt");

pub const SELECTION_UP_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-up-ctrl");

pub const SELECTION_DOWN_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-down-ctrl");

pub const SELECTION_UP_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-up-alt");

pub const SELECTION_DOWN_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-down-alt");

pub const SELECTION_DOCUMENT_HOME_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-home-ctrl");

pub const SELECTION_DOCUMENT_END_CTRL: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-end-ctrl");

pub const SELECTION_DOCUMENT_HOME_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-home-alt");

pub const SELECTION_DOCUMENT_END_ALT: crate::ActionKey =
    crate::ActionKey::custom("viewport.selection-document-end-alt");

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

fn run_style(line: LineRef<'_>, ix: usize) -> (Option<Role>, Modifier, Option<PaintStyle>) {
    match line {
        LineRef::Borrowed(line) => runs(line)
            .get(ix)
            .map_or((None, Modifier::empty(), None), |s| (s.role, s.add, None)),
        LineRef::Projected(text, line) => text
            .lines
            .get(line)
            .and_then(|line| text.runs.get(line.start.saturating_add(ix)))
            .map_or((None, Modifier::empty(), None), |run| {
                (run.role, run.modifier, run.paint)
            }),
    }
}

fn run_paint_style(base: PaintStyle, run: PaintStyle, selected: bool) -> PaintStyle {
    let composed = base.patch(run);
    if selected {
        composed.with_fg_from(base).with_bg_from(base)
    } else {
        composed
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
    source_revision: Option<u64>,
}

#[derive(Clone, Copy)]
struct SelectionLayout {
    key: LayoutKey,
    text_height: usize,
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
    fn ensure_with_widest<'a>(
        &mut self,
        lines: impl Into<LineSet<'a>>,
        key: LayoutKey,
        measure_widest: bool,
    ) -> usize {
        let lines = lines.into();
        let reusable = key.generation != u32::MAX
            && key.layout_revision != u64::MAX
            && key
                .source_revision
                .is_none_or(|revision| revision != u64::MAX);
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
    const fn allows_autoscroll(self) -> bool {
        matches!(
            self,
            PointerGesture::Plain { .. }
                | PointerGesture::Word { .. }
                | PointerGesture::Shift { .. }
                | PointerGesture::Line { .. }
        )
    }

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

    pub const fn horizontal_offset(&self) -> usize {
        self.horizontal.offset()
    }

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

    pub const fn follow_cursor(&self) -> FollowCursor {
        self.follow
    }

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

    fn reconcile_text_positions(&mut self, lines: LineSet<'_>) {
        if lines.is_empty() {
            return;
        }

        self.selection = self.selection.and_then(|(anchor, head)| {
            Some((clamp_position(lines, anchor)?, clamp_position(lines, head)?))
        });
        self.anchor = self.anchor.and_then(|anchor| clamp_position(lines, anchor));
        self.browse = if let Some((_, head)) = self.selection {
            Some(head)
        } else {
            self.browse.and_then(|pos| clamp_position(lines, pos))
        };
        if let PointerGesture::Word { range, dragged } = self.pointer_gesture {
            self.pointer_gesture = match (
                clamp_position(lines, range.0),
                clamp_position(lines, range.1),
            ) {
                (Some(start), Some(end)) => PointerGesture::Word {
                    range: (start, end),
                    dragged,
                },
                _ => PointerGesture::Idle,
            };
        }
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

    reserve_gutter: bool,

    scroll_gap: u16,
    caret_when_unfocused: bool,

    bar_cursor: bool,
    click_selects_word: bool,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
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
    text_height: usize,
    horizontal_overflow: bool,
    horizontal_bar: bool,
}

#[derive(Clone, Copy)]
struct PreparedGeometry {
    body_window: VerticalWindow,
    logical_rows: usize,
    text_window: Option<VerticalWindow>,
    text: Rect,
    hit_text: Rect,
    key: LayoutKey,
    total: usize,
    widest: usize,
    dimensions: PanDimensions,
    horizontal_track: Rect,
    authoritative: bool,
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
        }
    }

    fn note_indexed(count: usize) {
        let _ = count;
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn wrap(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    #[must_use]
    pub const fn horizontal_pan(mut self, enabled: bool) -> Self {
        self.horizontal_pan = enabled;
        self
    }

    #[must_use]
    pub const fn layout_revision(mut self, revision: u64) -> Self {
        self.layout_revision = revision;
        self
    }

    #[must_use]
    pub const fn reserve_gutter(mut self, yes: bool) -> Self {
        self.reserve_gutter = yes;
        self
    }

    #[must_use]
    pub const fn scroll_gap(mut self, cols: u16) -> Self {
        self.scroll_gap = cols;
        self
    }

    #[must_use]
    pub const fn caret_when_unfocused(mut self, enabled: bool) -> Self {
        self.caret_when_unfocused = enabled;
        self
    }

    #[must_use]
    pub const fn bar_cursor(mut self, yes: bool) -> Self {
        self.bar_cursor = yes;
        self
    }

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

    const fn pan_active(&self) -> bool {
        self.horizontal_pan && !self.wrap
    }

    fn pan_dimensions(
        &self,
        width: u16,
        body_rows: usize,
        content_rows: usize,
        widest_cols: usize,
    ) -> PanDimensions {
        let mut vertical = false;
        let mut horizontal = false;
        for _ in 0..3 {
            let rows = body_rows.saturating_sub(usize::from(horizontal && body_rows > 1));
            let next_vertical = content_rows > rows;
            let text_width = if self.scroll_gap > 0 && !next_vertical && content_rows <= rows {
                width.saturating_sub(u16::from(self.reserve_gutter))
            } else {
                Self::text_width(width, self.reserve_gutter, self.scroll_gap)
            };
            let next_horizontal = widest_cols > usize::from(text_width);
            if next_vertical == vertical && next_horizontal == horizontal {
                break;
            }
            vertical |= next_vertical;
            horizontal |= next_horizontal;
        }

        let horizontal_bar = horizontal && body_rows > 1;
        let text_height = body_rows.saturating_sub(usize::from(horizontal_bar));
        let vertical_overflow = content_rows > text_height;
        let text_width = if self.scroll_gap > 0 && !vertical_overflow && content_rows <= text_height
        {
            width.saturating_sub(u16::from(self.reserve_gutter))
        } else {
            Self::text_width(width, self.reserve_gutter, self.scroll_gap)
        };

        PanDimensions {
            text_width,
            text_height,
            horizontal_overflow: widest_cols > usize::from(text_width),
            horizontal_bar,
        }
    }

    fn fallback_width(&self, width: u16, body_rows: usize, lines: LineSet<'_>) -> u16 {
        if self.scroll_gap > 0 && !self.wrap && lines.len() <= body_rows {
            width.saturating_sub(u16::from(self.reserve_gutter))
        } else {
            Self::text_width(width, self.reserve_gutter, self.scroll_gap)
        }
    }

    const fn layout_key(
        &self,
        st: &ViewportState,
        width: u16,
        source_revision: Option<u64>,
    ) -> LayoutKey {
        LayoutKey {
            width,
            wrap: self.wrap,
            generation: st.generation,
            layout_revision: self.layout_revision,
            source_revision,
        }
    }

    fn prepared_geometry(
        &self,
        body_window: VerticalWindow,
        st: &ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
        layout: &mut ViewportLayout,
        authoritative: bool,
    ) -> PreparedGeometry {
        let area = body_window.visible_body();
        let fallback_width = self.fallback_width(area.width, body_window.logical_rows(), lines);
        let initial_key = self.layout_key(st, fallback_width, source_revision);
        let indexed = layout.ensure_with_widest(lines, initial_key, self.pan_active());
        let total = layout.total();

        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            self.pan_dimensions(
                area.width,
                body_window.logical_rows(),
                total,
                layout.widest_cols().unwrap_or(0),
            )
        } else {
            PanDimensions {
                text_width: fallback_width,
                text_height: body_window.logical_rows(),
                horizontal_overflow: false,
                horizontal_bar: false,
            }
        };
        let key = self.layout_key(st, dimensions.text_width, source_revision);
        let indexed = layout.ensure_with_widest(lines, key, self.pan_active());
        let total = layout.total();
        let widest = layout.widest_cols().unwrap_or(0);

        Self::note_indexed(indexed);
        let dimensions = if self.pan_active() {
            self.pan_dimensions(area.width, body_window.logical_rows(), total, widest)
        } else {
            dimensions
        };
        let logical_rows = dimensions.text_height;
        let text_window = self.text_window(body_window, logical_rows);
        let text = self.text_rect(body_window, text_window, dimensions, total);
        let horizontal_track = self.horizontal_track(body_window, dimensions);
        PreparedGeometry {
            body_window,
            logical_rows,
            text_window,
            text,
            hit_text: Rect::ZERO,
            key,
            total,
            widest,
            dimensions,
            horizontal_track,
            authoritative,
        }
    }

    fn text_window(
        &self,
        body_window: VerticalWindow,
        logical_rows: usize,
    ) -> Option<VerticalWindow> {
        if logical_rows == 0 {
            return None;
        }
        let body = body_window.visible_body();
        let gutter = u16::from(self.reserve_gutter);
        let x = body.x.checked_add(gutter)?;
        let width = body.width.checked_sub(gutter)?;
        let visible = body_window.visible_source_range();
        let start = visible.start;
        let end = visible.end.min(logical_rows);
        if start >= end {
            return None;
        }
        let y = body_window.physical_row_for_source(start)?;
        let height = u16::try_from(end.checked_sub(start)?).ok()?;
        VerticalWindow::new(Rect::new(x, y, width, height), logical_rows, start).ok()
    }

    fn text_rect(
        &self,
        body_window: VerticalWindow,
        text_window: Option<VerticalWindow>,
        dimensions: PanDimensions,
        content_rows: usize,
    ) -> Rect {
        let Some(text_window) = text_window else {
            let body = body_window.visible_body();
            let visible = body_window.visible_source_range();
            let start = visible.start;
            let end = visible.end.min(dimensions.text_height);
            if body.width.saturating_sub(u16::from(self.reserve_gutter)) == 0 && start < end {
                let y = body_window.physical_row_for_source(start).unwrap_or(body.y);
                return Rect::new(
                    body.x.saturating_add(u16::from(self.reserve_gutter)),
                    y,
                    0,
                    u16::try_from(end - start).unwrap_or(0),
                );
            }
            return Rect::ZERO;
        };
        let body = text_window.visible_body();
        let content_width = body
            .width
            .saturating_sub(u16::from(content_rows > dimensions.text_height));
        Rect {
            width: dimensions.text_width.min(content_width),
            ..body
        }
    }

    fn update_layout(
        &self,
        cx: &mut Cx<'_>,
        st: &ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
    ) -> PreparedGeometry {
        let facts = cx.layout(self.id);
        let text_part = cx.area_of_part(self.id, PartRef::of(Part::TEXT));
        let body_window = facts
            .and_then(|facts| facts.vertical_body_window)
            .or_else(|| {
                facts
                    .and_then(|facts| facts.logical_area)
                    .map(VerticalWindow::full)
            })
            .or_else(|| {
                text_part
                    .and_then(|_| cx.area(self.id))
                    .map(VerticalWindow::full)
            });
        let Some(body_window) = body_window else {
            return PreparedGeometry {
                body_window: VerticalWindow::full(Rect::ZERO),
                logical_rows: 0,
                text_window: None,
                text: Rect::ZERO,
                hit_text: Rect::ZERO,
                key: self.layout_key(st, 0, source_revision),
                total: 0,
                widest: 0,
                dimensions: PanDimensions::default(),
                horizontal_track: Rect::ZERO,
                authoritative: false,
            };
        };
        let mut prepared = {
            let layout = cx.cache::<ViewportLayout>(self.id);
            self.prepared_geometry(body_window, st, lines, source_revision, layout, true)
        };
        prepared.hit_text = text_part.unwrap_or(Rect::ZERO);
        prepared
    }

    fn pos_at(
        &self,
        cx: &mut Cx<'_>,
        lines: LineSet<'_>,
        geometry: &PreparedGeometry,
        offset: usize,
        horizontal_offset: usize,
        pos: Position,
    ) -> Option<CellPos> {
        let text_window = geometry.text_window?;
        let hit_text = geometry.hit_text;
        let key = geometry.key;
        if hit_text.is_empty() || lines.is_empty() || !hit_text.contains(pos) {
            return None;
        }
        let layout = cx.cache::<ViewportLayout>(self.id);
        let indexed = layout.ensure_with_widest(lines, key, self.pan_active());

        Self::note_indexed(indexed);
        let source_row = text_window.source_row_for_physical(pos.y)?;
        let visual = offset.checked_add(source_row)?;
        let (li, row) = layout.row_start(lines, visual);
        let line = lines.get(li)?;
        let (start, end) = row_cols(line, key.width, self.wrap, row);
        let dx = usize::from(pos.x.checked_sub(hit_text.x)?);
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
        self.update_lines(cx, st, LineSet::Borrowed(lines), None)
    }

    pub fn update_projected(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &ProjectedText,
    ) -> Response<ViewportAction> {
        self.update_lines(cx, st, LineSet::Projected(lines), None)
    }

    pub fn update_routed(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.update_source_lines(cx, st, LineSet::Borrowed(lines), source_revision)
    }

    pub fn update_projected_routed(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.update_source_lines(cx, st, LineSet::Projected(lines), source_revision)
    }

    pub fn service_routed_wheel(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.service_source_wheel(cx, st, LineSet::Borrowed(lines), source_revision)
    }

    pub fn service_projected_routed_wheel(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.service_source_wheel(cx, st, LineSet::Projected(lines), source_revision)
    }

    fn update_source_lines(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        source_revision: u64,
    ) -> Response<ViewportAction> {
        if let Some(attempt) = self.active_wheel_attempt(cx) {
            if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Event {
                let intents = cx.intents(self.id);
                let route_wheel_count = Self::count_route_wheel_intents(intents.clone(), attempt);
                if route_wheel_count == 1 {
                    let other_intents = Self::without_route_wheel_intent(intents, attempt);
                    let has_other_intents = other_intents.clone().next().is_some();
                    let routed =
                        self.service_source_wheel_attempt(cx, st, lines, source_revision, attempt);
                    if has_other_intents {
                        let other = self.update_lines_with(
                            cx,
                            st,
                            lines,
                            Some(source_revision),
                            other_intents,
                        );
                        return Self::fold_routed_and_other(routed, other, self.id);
                    }
                    return routed;
                }
                if route_wheel_count > 1 {
                    let other_intents = Self::without_route_wheel_intent(intents, attempt);
                    if other_intents.clone().next().is_some() {
                        return self.update_lines_with(
                            cx,
                            st,
                            lines,
                            Some(source_revision),
                            other_intents,
                        );
                    }
                    return Response::ignored().for_id(self.id);
                }

                return self.update_lines_with(cx, st, lines, Some(source_revision), intents);
            }
            return self.service_source_wheel_attempt(cx, st, lines, source_revision, attempt);
        }
        self.update_lines(cx, st, lines, Some(source_revision))
    }

    fn fold_routed_and_other(
        routed: Response<ViewportAction>,
        mut other: Response<ViewportAction>,
        id: Id,
    ) -> Response<ViewportAction> {
        let action = other.take_action();
        let mut acc = Acc::new();
        acc.fold(&routed.erase());
        acc.fold(&other.erase());
        if let Some(action) = action {
            acc.action(action);
        }
        acc.finish(id)
    }

    fn is_route_wheel_intent(intent: &Intent<'_>, attempt: WheelRouteAttempt) -> bool {
        matches!(
            intent,
            Intent::Wheel {
                axis,
                delta,
                part,
                pos,
            } if *axis == attempt.axis()
                && *delta == attempt.delta()
                && *part == attempt.part()
                && *pos == attempt.position()
        )
    }

    fn count_route_wheel_intents<'f>(
        intents: impl Iterator<Item = Intent<'f>>,
        attempt: WheelRouteAttempt,
    ) -> usize {
        intents
            .filter(|intent| Self::is_route_wheel_intent(intent, attempt))
            .count()
    }

    fn without_route_wheel_intent<'f, I>(
        intents: I,
        attempt: WheelRouteAttempt,
    ) -> impl Iterator<Item = Intent<'f>> + Clone + 'f
    where
        I: Iterator<Item = Intent<'f>> + Clone + 'f,
    {
        intents.filter(move |intent| !Self::is_route_wheel_intent(intent, attempt))
    }

    fn active_wheel_attempt(&self, cx: &Cx<'_>) -> Option<WheelRouteAttempt> {
        cx.wheel_route_attempt(self.id, PartRef::of(Part::CONTAINER), Axis::V)
            .or_else(|| cx.wheel_route_attempt(self.id, PartRef::of(Part::CONTAINER), Axis::H))
    }

    fn service_source_wheel(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        source_revision: u64,
    ) -> Response<ViewportAction> {
        let Some(attempt) = self.active_wheel_attempt(cx) else {
            return Response::ignored().for_id(self.id);
        };
        self.service_source_wheel_attempt(cx, st, lines, source_revision, attempt)
    }

    fn service_source_wheel_attempt(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        source_revision: u64,
        attempt: WheelRouteAttempt,
    ) -> Response<ViewportAction> {
        if attempt.source_revision() != source_revision {
            return Response::ignored().for_id(self.id);
        }

        let before = st.clone();
        let prepared = self.update_layout(cx, st, lines, Some(source_revision));
        if !prepared.authoritative || prepared.text_window.is_none() || prepared.hit_text.is_empty()
        {
            return Response::ignored().for_id(self.id);
        }
        let eligible = match attempt.axis() {
            Axis::V => prepared.total > prepared.logical_rows,
            Axis::H => self.pan_active() && prepared.dimensions.horizontal_overflow,
        };
        if !eligible {
            if !cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Ignored) {
                return Response::ignored().for_id(self.id);
            }
            return Response::ignored().for_id(self.id);
        }

        let result = match attempt.axis() {
            Axis::V => {
                st.scroll
                    .apply_layout(prepared.logical_rows, prepared.total);
                let before_wheel = st.scroll.offset();
                let _ = st.scroll.wheel(attempt.delta());
                if st.scroll.offset() != before_wheel {
                    st.follow = if st.scroll.at_end() {
                        FollowCursor::Always
                    } else {
                        FollowCursor::OnMove
                    };
                    if st.follow == FollowCursor::Always {
                        st.scroll.jump_end();
                    }
                    WheelRouteResult::Changed
                } else {
                    WheelRouteResult::Consumed
                }
            }
            Axis::H => {
                st.horizontal.set_content(prepared.widest);
                st.horizontal
                    .set_viewport(usize::from(prepared.dimensions.text_width));
                let before_wheel = st.horizontal.offset();
                st.horizontal
                    .scroll_by(isize::from(attempt.delta().signum()).saturating_mul(4));
                if st.horizontal.offset() != before_wheel {
                    WheelRouteResult::Changed
                } else {
                    WheelRouteResult::Consumed
                }
            }
        };

        if !cx.acknowledge_wheel_route(&attempt, result) {
            *st = before;
            return Response::ignored().for_id(self.id);
        }
        match result {
            WheelRouteResult::Ignored => Response::ignored().for_id(self.id),
            WheelRouteResult::Consumed => Response::consumed().for_id(self.id),
            WheelRouteResult::Changed => Response::changed().for_id(self.id),
            _ => Response::ignored().for_id(self.id),
        }
    }

    fn update_lines(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
    ) -> Response<ViewportAction> {
        self.update_lines_with(cx, st, lines, source_revision, cx.intents(self.id))
    }

    fn update_lines_with<'f>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
        intents: impl Iterator<Item = Intent<'f>>,
    ) -> Response<ViewportAction> {
        if lines.is_empty() {
            st.anchor = None;
            st.pointer_gesture = PointerGesture::Idle;
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
        }
        let prepared = self.update_layout(cx, st, lines, source_revision);
        let mut acc = Acc::<ViewportAction>::new();
        if !prepared.authoritative {
            for intent in intents {
                if matches!(
                    intent,
                    Intent::Cancel
                        | Intent::Pointer {
                            phase: Phase::Release | Phase::DragEnd,
                            ..
                        }
                ) {
                    st.pointer_gesture = PointerGesture::SuppressClick;
                    st.anchor = None;
                    st.scroll.release_track();
                    st.horizontal.release_track();
                    if cx.capture_owner() == Some(self.id) {
                        cx.release_capture();
                    }
                    acc.consumed();
                }
            }
            return acc.finish(self.id);
        }
        let visible_text = prepared.text_window.is_some() && !prepared.hit_text.is_empty();
        if !visible_text {
            for intent in intents {
                match intent {
                    Intent::Cancel
                    | Intent::Pointer {
                        phase: Phase::Release | Phase::DragEnd,
                        ..
                    } => {
                        st.pointer_gesture = PointerGesture::SuppressClick;
                        st.anchor = None;
                        st.scroll.release_track();
                        st.horizontal.release_track();
                        if cx.capture_owner() == Some(self.id) {
                            cx.release_capture();
                        }
                        acc.consumed();
                    }
                    Intent::Pointer { part, .. }
                        if part.part == HORIZONTAL_TRACK || part.part == HORIZONTAL_THUMB =>
                    {
                        let mut horizontal = st.horizontal;
                        horizontal.set_content(if self.pan_active() {
                            prepared.widest
                        } else {
                            0
                        });
                        horizontal.set_viewport(usize::from(prepared.dimensions.text_width));
                        self.handle_horizontal_pointer(
                            cx,
                            &mut horizontal,
                            prepared.horizontal_track,
                            intent,
                            &mut acc,
                        );
                        st.horizontal = horizontal;
                    }
                    _ => {}
                }
            }
            return acc.finish(self.id);
        }
        let total = prepared.total;
        let widest = prepared.widest;
        let dimensions = prepared.dimensions;
        st.scroll.apply_layout(prepared.logical_rows, total);
        st.horizontal
            .set_content(if self.pan_active() { widest } else { 0 });
        st.horizontal
            .set_viewport(usize::from(dimensions.text_width));
        let before = st.scroll.offset();
        let mut selection_moved = false;
        let bar = self.bar();
        let track_len = prepared.logical_rows.min(usize::from(u16::MAX)) as u16;
        let mut edge_drag_scrolled = false;
        for it in intents {
            let horizontal_wheel = matches!(it, Intent::Wheel { axis: Axis::H, .. });
            if !horizontal_wheel || !self.pan_active() {
                acc.fold(&bar.handle_intent_with_window(
                    cx,
                    &mut st.scroll,
                    track_len,
                    prepared.text_window,
                    it,
                ));
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
                        let moved = self.selection_command(
                            cx,
                            st,
                            lines,
                            SelectionLayout {
                                key: prepared.key,
                                text_height: prepared.logical_rows,
                            },
                            cmd,
                            &mut acc,
                        );
                        selection_moved |= moved;
                        if moved && cx.capture_owner() == Some(self.id) {
                            cx.cancel_text_drag_autoscroll(self.id, PartRef::of(Part::TEXT));
                            st.anchor = None;
                            st.pointer_gesture = PointerGesture::SuppressClick;
                        }
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
                        &mut st.horizontal,
                        prepared.horizontal_track,
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
                    let scroll_before_pointer = st.scroll.offset();
                    self.handle_text_pointer(
                        cx,
                        st,
                        lines,
                        prepared,
                        (phase, part, pos, mods),
                        &mut acc,
                    );
                    edge_drag_scrolled |=
                        phase == Phase::Drag && st.scroll.offset() != scroll_before_pointer;
                }
                _ => {}
            }
        }
        if !selection_moved && !edge_drag_scrolled {
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

    fn horizontal_track(&self, body_window: VerticalWindow, dimensions: PanDimensions) -> Rect {
        if !self.pan_active() || !dimensions.horizontal_overflow || !dimensions.horizontal_bar {
            return Rect::ZERO;
        }
        let body = body_window.visible_body();
        let Some(y) = body_window.physical_row_for_source(dimensions.text_height) else {
            return Rect::ZERO;
        };
        let gutter = u16::from(self.reserve_gutter);
        let Some(x) = body.x.checked_add(gutter) else {
            return Rect::ZERO;
        };
        let width = dimensions.text_width.min(body.width.saturating_sub(gutter));
        if width == 0 {
            return Rect::ZERO;
        }
        Rect::new(x, y, width, 1)
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

        st.reconcile_text_positions(lines);
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
                    .saturating_add(text_height.saturating_sub(1))
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
        if matches!(st.pointer_gesture, PointerGesture::Word { .. }) {
            st.pointer_gesture = PointerGesture::SuppressClick;
        }
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
        horizontal: &mut ScrollState,
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
            horizontal.release_track();
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            acc.consumed();
            return;
        }

        let before = horizontal.offset();
        match phase {
            Phase::Press if part.part == HORIZONTAL_THUMB => {
                let track_pos = usize::from(pos.x.saturating_sub(track.x));
                horizontal.press_track(track_pos, usize::from(track.width));
                if !cx.capture(self.id, PartRef::of(HORIZONTAL_THUMB)) {
                    horizontal.release_track();
                }
                acc.consumed();
            }
            Phase::Press => {
                let track_pos = usize::from(local.x);
                horizontal.press_track_with(track_pos, usize::from(track.width), TrackClick::Page);
                if before == horizontal.offset() {
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
                    let (_, thumb_len) = horizontal.thumb(usize::from(track.width));
                    usize::from(pos.x.saturating_sub(track.x).saturating_sub(grab))
                        .saturating_add(thumb_len / 2)
                } else {
                    usize::from(pos.x.saturating_sub(track.x))
                };
                horizontal.scroll_to(
                    horizontal.offset_for_track_pos(track_pos, usize::from(track.width)),
                );
                if before == horizontal.offset() {
                    acc.consumed();
                } else {
                    acc.changed();
                }
            }
            Phase::Release | Phase::DragEnd => {
                horizontal.release_track();
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
        geometry: PreparedGeometry,
        pointer: (Phase, PartRef, Position, KeyModifiers),
        acc: &mut Acc<ViewportAction>,
    ) {
        let hit_text = geometry.hit_text;
        let (phase, part, pos, mods) = pointer;
        if matches!(
            phase,
            Phase::Press | Phase::DragStart | Phase::Click | Phase::DoubleClick
        ) && !hit_text.contains(pos)
        {
            acc.consumed();
            return;
        }
        let mut horizontal_offset = st.horizontal.offset();
        let selection_before = st.selection;
        st.reconcile_text_positions(lines);
        let selection_reconciled = selection_before != st.selection;
        match phase {
            Phase::Press | Phase::DragStart => {
                cx.focus(self.id);
                let _ = cx.capture(self.id, PartRef::of(Part::TEXT));
                let at = self.pos_at(
                    cx,
                    lines,
                    &geometry,
                    st.scroll.offset(),
                    horizontal_offset,
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
                let vertical_delta = if !hit_text.is_empty() && pos.y < hit_text.y {
                    -isize::try_from(hit_text.y - pos.y).unwrap_or(isize::MAX)
                } else if !hit_text.is_empty() && pos.y >= hit_text.bottom() {
                    isize::try_from(pos.y.saturating_add(1).saturating_sub(hit_text.bottom()))
                        .unwrap_or(isize::MAX)
                } else {
                    0
                };
                if vertical_delta != 0 {
                    let before = st.scroll.offset();
                    st.scroll.scroll_by(vertical_delta);
                    if st.scroll.offset() != before {
                        st.follow = FollowCursor::OnMove;
                        acc.changed();
                    }
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
                    &geometry,
                    st.scroll.offset(),
                    horizontal_offset,
                    clamped,
                );
                let selected = match (st.pointer_gesture, st.anchor, at) {
                    (PointerGesture::Word { range, .. }, _, Some(head)) => {
                        Some(extend_word_range(range, head, word_range_at(lines, head)))
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
                } else if selection_reconciled {
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
                    &geometry,
                    st.scroll.offset(),
                    horizontal_offset,
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
                } else if selection_reconciled {
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
                if selection_reconciled {
                    acc.action(ViewportAction::SelectionChanged);
                } else {
                    acc.consumed();
                }
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
        self.draw_lines(
            ui,
            VerticalWindow::full(area),
            FadePolicy::LegacyRect,
            st,
            LineSet::Borrowed(lines),
            None,
        )
        .0
    }

    pub fn draw_projected(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &ProjectedText,
    ) -> Rect {
        self.draw_lines(
            ui,
            VerticalWindow::full(area),
            FadePolicy::LegacyRect,
            st,
            LineSet::Projected(lines),
            None,
        )
        .0
    }

    pub fn draw_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
    ) -> Rect {
        self.draw_lines(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Borrowed(lines),
            None,
        )
        .0
    }

    pub fn draw_projected_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &ProjectedText,
    ) -> Rect {
        self.draw_lines(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Projected(lines),
            None,
        )
        .0
    }

    pub fn draw_routed(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        self.draw_lines(
            ui,
            VerticalWindow::full(area),
            FadePolicy::LegacyRect,
            st,
            LineSet::Borrowed(lines),
            Some(source_revision),
        )
    }

    pub fn draw_window_routed(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        self.draw_lines(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Borrowed(lines),
            Some(source_revision),
        )
    }

    pub fn draw_window_routed_with_view(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>, Option<ScrollState>) {
        self.draw_lines_with_view(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Borrowed(lines),
            Some(source_revision),
        )
    }

    pub fn draw_projected_routed(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        self.draw_lines(
            ui,
            VerticalWindow::full(area),
            FadePolicy::LegacyRect,
            st,
            LineSet::Projected(lines),
            Some(source_revision),
        )
    }

    pub fn draw_projected_window_routed(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>) {
        self.draw_lines(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Projected(lines),
            Some(source_revision),
        )
    }

    pub fn draw_projected_window_routed_with_view(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        st: &ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> (Rect, Option<ScrollRouteKey>, Option<ScrollState>) {
        self.draw_lines_with_view(
            ui,
            window,
            FadePolicy::LogicalWindow,
            st,
            LineSet::Projected(lines),
            Some(source_revision),
        )
    }

    fn draw_lines(
        &self,
        ui: &mut Ui<'_>,
        body_window: VerticalWindow,
        fade_policy: FadePolicy,
        st: &ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
    ) -> (Rect, Option<ScrollRouteKey>) {
        let (text, route_key, _) =
            self.draw_lines_with_view(ui, body_window, fade_policy, st, lines, source_revision);
        (text, route_key)
    }

    fn draw_lines_with_view(
        &self,
        ui: &mut Ui<'_>,
        body_window: VerticalWindow,
        fade_policy: FadePolicy,
        st: &ViewportState,
        lines: LineSet<'_>,
        source_revision: Option<u64>,
    ) -> (Rect, Option<ScrollRouteKey>, Option<ScrollState>) {
        let area = body_window.visible_body();
        if area.is_empty() {
            return (area, None, None);
        }
        let ov = self.overrides();
        let id = self.id;
        let runtime = ui.state(id).difference(StateFlags::SELECTED);
        let live = PartStyle::flags(runtime, StateFlags::empty());
        let mut geometry = {
            let layout = ui.cache::<ViewportLayout>(id);
            self.prepared_geometry(body_window, st, lines, source_revision, layout, true)
        };
        let body_window = geometry.body_window;
        let area = body_window.visible_body();
        let total = geometry.total;
        let logical_rows = geometry.logical_rows;
        let text_width = geometry.dimensions.text_width;
        let mut view = st.scroll;
        view.apply_layout(logical_rows, total);
        if st.follow == FollowCursor::Always {
            view.jump_end();
        }
        let mut horizontal = st.horizontal;
        horizontal.set_content(if self.pan_active() {
            geometry.widest
        } else {
            0
        });
        horizontal.set_viewport(usize::from(text_width));
        let caret_line_start = st.caret.map(|caret| {
            let layout = ui.cache::<ViewportLayout>(id);
            layout.line_start(caret.line)
        });

        ui.register_control(id, area, Focusability::Focusable);
        ui.publish_bindings(id, live, BINDINGS);
        ui.publish_additional_bindings(id, runtime, SELECTION_BINDINGS);
        if st.selection.is_some() {
            ui.publish_additional_bindings(id, runtime, SELECTION_CLEAR_BINDINGS);
        }
        if self.pan_active() && geometry.dimensions.horizontal_overflow {
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
        if self.reserve_gutter {
            self.gutter(ui, area, live, &ov);
        }

        let bar = self.bar();
        if let Some(text_window) = geometry.text_window {
            if source_revision.is_some() {
                let _ = bar.draw_window_without_wheel_registration(ui, text_window, &view, total);
            } else {
                let _ = bar.draw_window(ui, text_window, &view, total);
            }
        }

        let text = geometry.text;
        geometry.hit_text = text;
        let mut facts = LayoutFacts::new(
            logical_rows,
            total,
            logical_rows.min(usize::from(u16::MAX)) as u16,
            text_width,
        )
        .with_vertical_body_window(body_window);
        if let Some(text_window) = geometry.text_window {
            facts = facts.with_vertical_window(text_window);
        }
        if body_window.first_source_row() == 0
            && usize::from(area.height) == body_window.logical_rows()
        {
            facts = facts.with_logical_area(area);
        }
        ui.report_layout(id, facts);

        let mut route_key = None;
        if let Some(source_revision) = source_revision
            && let Some(text_window) = geometry.text_window
            && !text.is_empty()
        {
            let vertical = view.headroom_v();
            let horizontal_headroom = horizontal.headroom_v();
            let axes = if self.pan_active() {
                Axes::Both
            } else {
                Axes::V
            };
            if view.overflows() {
                let body = text_window.visible_body();
                if body.width > 0 && body.height > 0 {
                    let bar_strip =
                        Rect::new(body.right().saturating_sub(1), body.y, 1, body.height);
                    if bar_strip.intersection(text).is_empty() {
                        ui.register_scroll(id, bar_strip, Axes::V, vertical);
                    }
                }
            }
            route_key = ui.register_routed_scroll(
                id,
                text,
                axes,
                Headroom {
                    up: vertical.up,
                    down: vertical.down,
                    left: if axes.handles(Axis::H) {
                        horizontal_headroom.up
                    } else {
                        0
                    },
                    right: if axes.handles(Axis::H) {
                        horizontal_headroom.down
                    } else {
                        0
                    },
                },
                source_revision,
            );
        } else if source_revision.is_none() && self.pan_active() && !text.is_empty() {
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
        let text_window = geometry.text_window;
        if !text.is_empty()
            && let Some(text_window) = text_window
        {
            ui.register_part(id, PartRef::of(Part::TEXT), text);
            let vertical_headroom = view.headroom_v();
            let horizontal_headroom = horizontal.headroom_v();
            let drag_axes = if self.pan_active() && geometry.dimensions.horizontal_overflow {
                Axes::Both
            } else {
                Axes::V
            };
            if st.pointer_gesture.allows_autoscroll() {
                ui.register_text_drag_autoscroll(
                    id,
                    PartRef::of(Part::TEXT),
                    text,
                    drag_axes,
                    Headroom {
                        up: vertical_headroom.up,
                        down: vertical_headroom.down,
                        left: if drag_axes.handles(Axis::H) {
                            horizontal_headroom.up
                        } else {
                            0
                        },
                        right: if drag_axes.handles(Axis::H) {
                            horizontal_headroom.down
                        } else {
                            0
                        },
                    },
                    st.generation,
                );
            }
            let first_visual = view.offset().saturating_add(text_window.first_source_row());
            if first_visual < total {
                let (first_line, first_row) = {
                    let layout = ui.cache::<ViewportLayout>(id);
                    layout.row_start(lines, first_visual)
                };
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
            }
            self.caret(
                ui,
                geometry,
                st,
                lines,
                (
                    text_width,
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
            let mut faded = text;
            let extra = self
                .scroll_gap
                .min(area.right().saturating_sub(text.right()));
            faded.width = faded.width.saturating_add(extra);
            match fade_policy {
                FadePolicy::LegacyRect => ui.scroll_edges(faded, &view),
                FadePolicy::LogicalWindow => {
                    if let Ok(faded_window) =
                        VerticalWindow::new(faded, logical_rows, text_window.first_source_row())
                    {
                        ui.scroll_edges_window(faded_window, &view);
                    }
                }
            }
        }
        if !geometry.horizontal_track.is_empty() {
            self.paint_horizontal_bar(ui, geometry.horizontal_track, &horizontal, live, &ov);
        }
        (text, route_key, Some(view))
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

        for y in text.rows() {
            let Some(line) = lines.get(li) else { break };

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
        geometry: PreparedGeometry,
        st: &ViewportState,
        lines: LineSet<'_>,
        view: (u16, usize, Option<usize>, usize),
        live: StateFlags,
    ) {
        let text = geometry.text;
        let Some(text_window) = geometry.text_window else {
            return;
        };
        let (width, offset, line_start, horizontal_offset) = view;
        let Some(c) = st.caret else { return };
        if !live.contains(StateFlags::FOCUSED)
            && !(self.caret_when_unfocused && st.follow() && st.selection().is_none())
        {
            return;
        }
        let visible = text.intersection(ui.full());
        if visible.is_empty() {
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
        let Some(source_row) = visual.checked_sub(offset) else {
            return;
        };
        let Some(y) = text_window.physical_row_for_source(source_row) else {
            return;
        };
        let (start, _) = row_cols(at, width, self.wrap, within);
        let caret_col = c.col.saturating_sub(start);
        if caret_col < horizontal_offset {
            return;
        }
        let dx = caret_col.saturating_sub(horizontal_offset);
        if dx >= usize::from(text.width) {
            return;
        }
        let Ok(dx) = u16::try_from(dx) else {
            return;
        };
        let Some(x) = visible.x.checked_add(dx) else {
            return;
        };
        let pos = Position::new(x, y);
        if !visible.contains(pos) {
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
        let (role, add, paint) = run_style(line, r.span);
        let base = if r.sel { styles.1 } else { styles.0 };
        let area = Rect {
            x: *x,
            y: rect.y,
            width: right.saturating_sub(*x),
            height: 1,
        };
        let w = if let Some(paint) = paint {
            ui.paint_str(area, text, run_paint_style(base, paint, r.sel))
        } else {
            let sp = Span {
                text,
                role: if r.sel { None } else { role },
                add,
            };
            ui.paint_spans(area, &[sp], base)
        };
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
            let base = if is_sel { styles.1 } else { styles.0 };
            let paint = run_style(line, c.span)
                .2
                .map_or(base, |paint| run_paint_style(base, paint, is_sel));
            ui.fill(
                Rect {
                    x,
                    y: rect.y,
                    width: 1,
                    height: 1,
                },
                paint,
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
