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
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::hit::ScrollRouteKey;
use termrock_runtime::runtime::{WheelRouteAttempt, WheelRouteResult};
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

    /// Add a line with complete paint styles for each text run.
    ///
    /// The viewport still measures, wraps, selects, copies, clips, and paints
    /// the line. A grapheme that crosses run boundaries uses the style of the
    /// run containing its first scalar, matching [`ViewportLine::Spans`].
    /// On selected cells, the viewport's selected foreground and background
    /// remain authoritative while producer modifiers and underline channels
    /// compose over that selected style. Styles are stored as supplied; rebuild
    /// this projection from the current theme and paint context when those
    /// inputs change. The viewport does not use its source revision as a
    /// theme-style cache key.
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
        // Rich producer modifiers and underline channels compose just like
        // ordinary span additions over the selected base. Only selected ink
        // is authoritative, and these helpers preserve its role metadata.
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
            #[cfg(feature = "testing")]
            work_probe: None,
        }
    }

    /// Returns the default, always-published supplemental selection bindings.
    ///
    /// This testing-only view is derived from the same table that the
    /// viewport publishes and dispatches. It excludes the conditional
    /// selection-clear and horizontal-pan tables, and does not describe
    /// consumer remaps or every conditional viewport command.
    #[cfg(feature = "testing")]
    pub fn selection_binding_chords() -> impl Iterator<Item = (crate::ActionKey, Chord)> {
        SELECTION_BINDINGS
            .iter()
            .filter_map(|binding| binding.chord.map(|chord| (binding.action, chord)))
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
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
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
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
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
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
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

    /// Update a viewport whose caller supplies an identity for the live text
    /// source. A source-aware draw and update must use the same revision. The
    /// caller must advance it for replacements, reorderings, and layout-affecting
    /// changes, and must not reuse it while an older route can still arrive.
    /// Revisions must fail closed at exhaustion instead of wrapping.
    pub fn update_routed(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.update_source_lines(cx, st, LineSet::Borrowed(lines), source_revision)
    }

    /// Projected-text form of [`TextViewport::update_routed`].
    pub fn update_projected_routed(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &ProjectedText,
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.update_source_lines(cx, st, LineSet::Projected(lines), source_revision)
    }

    /// Service only this viewport's active routed wheel attempt. A stale source
    /// revision remains unacknowledged and does not prepare or mutate state.
    pub fn service_routed_wheel(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
        source_revision: u64,
    ) -> Response<ViewportAction> {
        self.service_source_wheel(cx, st, LineSet::Borrowed(lines), source_revision)
    }

    /// Projected-text form of [`TextViewport::service_routed_wheel`].
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
                    // Duplicate matches cannot be attributed to one physical input. Consume
                    // only those ambiguous wheel intents, leave the attempt unacknowledged,
                    // and still deliver every unrelated intent through the ordinary path.
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
                // If the active attempt has no matching physical intent, it has no
                // authority to acknowledge or mutate for that event. Preserve the actual
                // queue through the ordinary update path.
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

    /// Draw a viewport into a visible logical window.
    ///
    /// The window's logical row count supplies the original body height. The
    /// returned rectangle is only the physically visible text area after
    /// gutter, scrollbar, and horizontal-row reservations.
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

    /// Draw projected text into a visible logical window.
    ///
    /// The window's logical row count supplies the original body height. The
    /// returned rectangle is only the physically visible text area after
    /// gutter, scrollbar, and horizontal-row reservations.
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

    /// Draw a viewport as an opt-in wheel target with explicit live-source
    /// identity. Its key is valid only for the current draw frame and must be
    /// linked explicitly to a containing parent. The caller must advance the
    /// revision when source order, contents, or layout-affecting source data
    /// changes, and must not wrap or reuse it while an old route can arrive.
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

    /// Draw a visible logical window as an opt-in wheel target.
    ///
    /// The window's logical row count supplies the original body height. The
    /// returned rectangle is only the visible text area. The route key is
    /// valid for this draw frame and must be linked to a containing parent.
    /// Keep the source revision unique while an older registration can still
    /// be serviced. Do not derive it from content length or state generation;
    /// fail closed rather than wrapping at exhaustion.
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

    /// Draw a visible logical window as an opt-in wheel target and return the
    /// exact vertical view used by this draw.
    ///
    /// The returned view includes the current logical layout clamp and follow
    /// behavior. It is an owned snapshot; drawing does not mutate `st`. The
    /// route key remains frame-local and must be linked to a containing parent.
    /// The view is `None` only when the visible body is empty; a nonempty body
    /// with no text columns still returns its projected view.
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

    /// Projected-text form of [`TextViewport::draw_routed`].
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

    /// Draw projected text in a visible logical window as an opt-in wheel target.
    ///
    /// The returned rectangle is only the visible text area. The route key is
    /// frame-local and carries the source revision; link it to a containing
    /// parent. Keep revisions unique while an older registration can still be
    /// serviced, and fail closed rather than wrapping at exhaustion.
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

    /// Projected-text form of [`TextViewport::draw_window_routed_with_view`].
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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::{Buffer, Cell};

    use super::*;
    use crate::event::{Axis, MouseKind};
    use crate::intent::FocusVia;
    use crate::runtime::stub::{SCREEN, Stub, key, mouse};
    use crate::runtime::{App, Runtime, UpdateCause};
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
    const ROUTED_PARENT: Id = Id::root("viewport.routed.parent");
    const ROUTED_CHILD: Id = Id::root("viewport.routed.child");
    const ROUTED_FRONT: Id = Id::root("viewport.routed.front");
    const ROUTED_AREA: Rect = Rect::new(2, 2, 24, 8);
    const ROUTED_SHORT: [ViewportLine<'static>; 2] =
        [ViewportLine::Plain("short"), ViewportLine::Plain("text")];
    const ROUTED_SHORT_32: [ViewportLine<'static>; 32] = [ViewportLine::Plain("short"); 32];
    const ROUTED_LONG: [ViewportLine<'static>; 32] =
        [ViewportLine::Plain("0123456789012345678901234567890123456789"); 32];

    type ObservedRouteAttempt = (UpdateCause, Option<(Id, Axis, u64)>);

    struct RoutedViewportApp {
        state: ViewportState,
        parent: ScrollState,
        long: bool,
        empty: bool,
        same_count: bool,
        source_revision: u64,
        replace_on_event: bool,
        replacement_long: bool,
        front_router: bool,
        preack_child_route: bool,
        wrap: bool,
        pan: bool,
        projected: Option<ProjectedText>,
        linked: std::cell::Cell<bool>,
        child_key: std::cell::Cell<Option<ScrollRouteKey>>,
        front_key: std::cell::Cell<Option<ScrollRouteKey>>,
        event_updates: usize,
        tick_updates: usize,
        route_callbacks: usize,
        route_owners: Vec<Id>,
        route_callbacks_empty: bool,
        observed_route_attempts: Vec<ObservedRouteAttempt>,
        observed_update_state: Vec<(UpdateCause, u64, usize, usize)>,
        last_event_attempt: Option<WheelRouteAttempt>,
    }

    impl RoutedViewportApp {
        fn new(long: bool, wrap: bool, pan: bool) -> Self {
            let mut state = ViewportState::default();
            state.set_follow(false);
            state.scroll.apply_layout(7, ROUTED_LONG.len());
            let mut parent = ScrollState::new(200);
            parent.set_viewport(usize::from(SCREEN.height));
            Self {
                state,
                parent,
                long,
                empty: false,
                same_count: false,
                source_revision: 7,
                replace_on_event: false,
                replacement_long: true,
                front_router: false,
                preack_child_route: false,
                wrap,
                pan,
                projected: None,
                linked: std::cell::Cell::new(false),
                child_key: std::cell::Cell::new(None),
                front_key: std::cell::Cell::new(None),
                event_updates: 0,
                tick_updates: 0,
                route_callbacks: 0,
                route_owners: Vec::new(),
                route_callbacks_empty: true,
                observed_route_attempts: Vec::new(),
                observed_update_state: Vec::new(),
                last_event_attempt: None,
            }
        }

        fn borrowed_lines(&self) -> &'static [ViewportLine<'static>] {
            if self.empty {
                &[]
            } else if self.long {
                &ROUTED_LONG
            } else if self.same_count {
                &ROUTED_SHORT_32
            } else {
                &ROUTED_SHORT
            }
        }

        fn viewport(&self) -> TextViewport<'static> {
            TextViewport::new(ROUTED_CHILD)
                .wrap(self.wrap)
                .horizontal_pan(self.pan)
        }

        fn update_viewport(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let viewport = self.viewport();
            let source_revision = self.source_revision;
            let borrowed_lines = self.borrowed_lines();
            if let Some(projected) = &self.projected {
                viewport
                    .update_projected_routed(cx, &mut self.state, projected, source_revision)
                    .erase()
            } else {
                viewport
                    .update_routed(cx, &mut self.state, borrowed_lines, source_revision)
                    .erase()
            }
        }

        fn service_viewport(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let viewport = self.viewport();
            let source_revision = self.source_revision;
            let borrowed_lines = self.borrowed_lines();
            if let Some(projected) = &self.projected {
                viewport
                    .service_projected_routed_wheel(cx, &mut self.state, projected, source_revision)
                    .erase()
            } else {
                viewport
                    .service_routed_wheel(cx, &mut self.state, borrowed_lines, source_revision)
                    .erase()
            }
        }
    }

    impl App for RoutedViewportApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let cause = cx.update_cause();
            let viewport_before = self.state.scroll.viewport_len();
            if let Some(attempt) =
                cx.wheel_route_attempt(ROUTED_FRONT, PartRef::of(Part::CONTAINER), Axis::V)
            {
                for _ in cx.intents(ROUTED_FRONT) {}
                assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Ignored));
            }
            let route_attempt = cx
                .wheel_route_attempt(ROUTED_CHILD, PartRef::of(Part::CONTAINER), Axis::V)
                .or_else(|| {
                    cx.wheel_route_attempt(ROUTED_CHILD, PartRef::of(Part::CONTAINER), Axis::H)
                });
            if cause == UpdateCause::Event && route_attempt.is_some() {
                self.last_event_attempt = route_attempt;
            }
            self.observed_route_attempts.push((
                cause,
                route_attempt
                    .map(|attempt| (attempt.owner(), attempt.axis(), attempt.source_revision())),
            ));
            match cause {
                UpdateCause::Event => {
                    self.event_updates = self.event_updates.saturating_add(1);
                    if self.replace_on_event {
                        self.long = self.replacement_long;
                        self.source_revision = self.source_revision.saturating_add(1);
                        self.replace_on_event = false;
                    }
                }
                UpdateCause::Tick => self.tick_updates = self.tick_updates.saturating_add(1),
                _ => {}
            }
            let parent = ScrollRegion::new(ROUTED_PARENT)
                .scrollbar_visible(false)
                .update_routed(cx, &mut self.parent, 200, 11);
            if self.preack_child_route
                && cause == UpdateCause::Event
                && let Some(attempt) =
                    cx.wheel_route_attempt(ROUTED_CHILD, PartRef::of(Part::CONTAINER), Axis::V)
            {
                assert!(cx.acknowledge_wheel_route(&attempt, WheelRouteResult::Consumed));
                self.preack_child_route = false;
            }
            let viewport_response = self.update_viewport(cx);
            self.observed_update_state.push((
                cause,
                self.source_revision,
                viewport_before,
                self.state.scroll.viewport_len(),
            ));
            parent | viewport_response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let parent = ScrollRegion::new(ROUTED_PARENT).scrollbar_visible(false);
            let (_, parent_key) = parent.draw_routed(ui, ui.full(), &self.parent, 200, 11);
            let viewport = self.viewport();
            let (text, child_key) = if let Some(projected) = &self.projected {
                viewport.draw_projected_routed(
                    ui,
                    ROUTED_AREA,
                    &self.state,
                    projected,
                    self.source_revision,
                )
            } else {
                viewport.draw_routed(
                    ui,
                    ROUTED_AREA,
                    &self.state,
                    self.borrowed_lines(),
                    self.source_revision,
                )
            };
            self.child_key.set(child_key);
            let front_key = self
                .front_router
                .then(|| {
                    ui.register_routed_scroll(
                        ROUTED_FRONT,
                        text,
                        Axes::V,
                        Headroom {
                            up: 1,
                            down: 1,
                            left: 0,
                            right: 0,
                        },
                        self.source_revision,
                    )
                })
                .flatten();
            self.front_key.set(front_key);
            if let (Some(front), Some(child)) = (front_key, child_key) {
                let _ = ui.link_scroll_parent(front, child);
            }
            let linked = child_key
                .zip(parent_key)
                .is_some_and(|(child, parent)| ui.link_scroll_parent(child, parent));
            self.linked.set(linked);
            let _ = text;
        }

        fn route_wheel_attempt(
            &mut self,
            cx: &mut Cx<'_>,
            attempt: WheelRouteAttempt,
        ) -> Response<()> {
            self.route_callbacks = self.route_callbacks.saturating_add(1);
            self.route_owners.push(attempt.owner());
            self.route_callbacks_empty &= cx.update_cause() == UpdateCause::ScrollRoute
                && cx.intents(ROUTED_PARENT).next().is_none()
                && cx.intents(ROUTED_CHILD).next().is_none()
                && cx.intents(ROUTED_FRONT).next().is_none();
            if attempt.owner() == ROUTED_PARENT {
                ScrollRegion::new(ROUTED_PARENT).service_routed_wheel(cx, &mut self.parent, 200, 11)
            } else if attempt.owner() == ROUTED_CHILD {
                self.service_viewport(cx)
            } else {
                Response::ignored()
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ModalControlEvent {
        FocusIn(FocusVia),
        FocusOut(Option<Id>),
        Key(KeyCode),
    }

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
        show_viewport: bool,
        request_repaint_after_ms: Option<u64>,
        cancel_drag_autoscroll: Option<(Id, PartRef)>,
        update_causes: Vec<UpdateCause>,
        drag_events: Vec<(UpdateCause, PartRef, Position, KeyModifiers, Option<u8>)>,
        modal_layer_receipts: Vec<(termrock_runtime::LayerOpening, termrock_runtime::LayerEvent)>,
        modal_owner_events: Vec<termrock_runtime::LayerEvent>,
        selection_focus_outs: Vec<Option<Id>>,
        modal_control_events: Vec<ModalControlEvent>,
        unhandled_selection_intents: Vec<&'static str>,
        unhandled_modal_control_intents: Vec<&'static str>,
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
                show_viewport: true,
                request_repaint_after_ms: None,
                cancel_drag_autoscroll: None,
                update_causes: Vec::new(),
                drag_events: Vec::new(),
                modal_layer_receipts: Vec::new(),
                modal_owner_events: Vec::new(),
                selection_focus_outs: Vec::new(),
                modal_control_events: Vec::new(),
                unhandled_selection_intents: Vec::new(),
                unhandled_modal_control_intents: Vec::new(),
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
            let cause = cx.update_cause();
            self.update_causes.push(cause);
            if let Some(ms) = self.request_repaint_after_ms.take() {
                cx.request_repaint_after(core::time::Duration::from_millis(ms));
            }
            if let Some((owner, part)) = self.cancel_drag_autoscroll.take() {
                cx.cancel_text_drag_autoscroll(owner, part);
            }
            while let Some(receipt) = cx.take_layer_intent_receipt(MODAL_LAYER) {
                self.modal_layer_receipts
                    .push((receipt.opening(), receipt.event()));
            }
            for intent in cx.intents(SELECTION_ID) {
                match intent {
                    Intent::Pointer {
                        phase: Phase::Drag,
                        part,
                        pos,
                        mods,
                        ..
                    } => self.drag_events.push((
                        cause,
                        part,
                        pos,
                        mods,
                        cx.pointer_press_count(SELECTION_ID, part),
                    )),
                    Intent::Pointer { .. } | Intent::Binding(_) | Intent::Key(_) => {}
                    Intent::FocusIn { .. } => {}
                    Intent::FocusOut { to } if self.modal_open && to == Some(MODAL_CONTROL) => {
                        self.selection_focus_outs.push(to);
                    }
                    Intent::Layer(event)
                        if self.modal_layer_receipts.iter().any(|(opening, received)| {
                            opening.owner() == SELECTION_ID && *received == event
                        }) =>
                    {
                        self.modal_owner_events.push(event);
                    }
                    Intent::FocusOut { .. }
                    | Intent::Paste(_)
                    | Intent::Wheel { .. }
                    | Intent::Layer(_)
                    | Intent::Cancel => self.unhandled_selection_intents.push(match intent {
                        Intent::FocusOut { .. } => "FocusOut",
                        Intent::Paste(_) => "Paste",
                        Intent::Wheel { .. } => "Wheel",
                        Intent::Layer(_) => "unmatched Layer event",
                        Intent::Cancel => "Cancel",
                        _ => unreachable!("the outer match selected this intent"),
                    }),
                    _ => self
                        .unhandled_selection_intents
                        .push("other selection intent"),
                }
            }
            for intent in cx.intents(MODAL_CONTROL) {
                match intent {
                    Intent::FocusIn { via } => {
                        self.modal_control_events
                            .push(ModalControlEvent::FocusIn(via));
                    }
                    Intent::FocusOut { to } => {
                        self.modal_control_events
                            .push(ModalControlEvent::FocusOut(to));
                    }
                    Intent::Key(key) => {
                        self.modal_control_events
                            .push(ModalControlEvent::Key(key.code));
                    }
                    _ => self
                        .unhandled_modal_control_intents
                        .push("unexpected modal-control intent"),
                }
            }
            if self.modal_open {
                cx.open_layer(
                    MODAL_LAYER,
                    LayerSpec::modal(SELECTION_ID).initial_focus(MODAL_CONTROL),
                );
            }
            let response = if self.show_viewport {
                self.viewport().update(cx, &mut self.state, &self.lines)
            } else {
                Response::ignored()
            };
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
            if !self.show_viewport {
                // The owner is intentionally absent from this registry frame.
            } else if self.reference_only {
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
    fn captured_text_drag_autoscroll_replays_once_after_fifty_milliseconds() {
        let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        app.state.set_caret(Some(CellPos::new(0, 1)));
        let mut runtime = selection_runtime(app);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );

        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        let before_replay = runtime.app().state.scroll().offset();
        assert_eq!(before_replay, 3, "edge movement follows outside distance");
        let deadline = runtime.next_deadline();
        assert_eq!(deadline, Some(crate::runtime::Moment::from_millis(50)));

        runtime
            .advance_to(crate::runtime::Moment::from_millis(49))
            .expect("the simulated clock is monotonic");
        assert!(
            !runtime.needs_settle(),
            "49 ms is before the replay deadline"
        );
        assert_eq!(runtime.app().state.scroll().offset(), before_replay);
        runtime.app_mut().state.set_follow(true);

        runtime
            .advance_to(crate::runtime::Moment::from_millis(50))
            .expect("the simulated clock reaches the replay deadline");
        assert!(
            runtime.needs_settle(),
            "the due replay schedules one runtime tick"
        );
        runtime.app_mut().update_causes.clear();
        runtime.app_mut().drag_events.clear();
        let _ = runtime.settle();

        assert!(
            runtime.app().state.scroll().offset() > before_replay,
            "a stationary pointer continues the captured selection drag"
        );
        assert!(runtime.app().state.selection().is_some());
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 1)));
        assert!(
            !runtime.app().state.follow(),
            "edge selection pauses follow mode"
        );
        assert_eq!(runtime.focus(), Some(SELECTION_ID));
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(runtime.app().update_causes, [UpdateCause::Tick]);
        assert_eq!(runtime.app().drag_events.len(), 1);
        assert_eq!(
            runtime.app().drag_events[0],
            (
                UpdateCause::Tick,
                PartRef::of(Part::TEXT),
                Position::new(5, 9),
                KeyModifiers::SHIFT,
                Some(1),
            ),
            "replay preserves pointer, modifiers, and press count"
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(100)),
            "the next replay is based on the actual tick time, with no catch-up burst"
        );
        assert!(runtime.diagnostics().is_empty());

        let offset_after_replay = runtime.app().state.scroll().offset();
        let _ = runtime.settle();
        assert_eq!(runtime.app().state.scroll().offset(), offset_after_replay);
        assert_eq!(runtime.app().update_causes, [UpdateCause::Tick]);
        assert_eq!(
            runtime.app().drag_events.len(),
            1,
            "settle cannot replay twice"
        );

        let copied = copy_from_viewport(&mut runtime);
        assert!(
            copied.lines().count() > 1,
            "copy follows the replayed selection"
        );
    }

    #[test]
    fn held_text_drag_replays_once_per_due_tick_with_competing_deadlines() {
        let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );
        runtime.app_mut().request_repaint_after_ms = Some(25);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Char('q'), KeyModifiers::NONE),
        );

        runtime.app_mut().update_causes.clear();
        runtime.app_mut().drag_events.clear();
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(25))
        );
        runtime
            .advance_to(crate::runtime::Moment::from_millis(25))
            .expect("the simulated clock is monotonic");
        let _ = runtime.settle();
        assert_eq!(runtime.app().update_causes, [UpdateCause::Tick]);
        assert!(runtime.app().drag_events.is_empty());
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50))
        );

        runtime.app_mut().update_causes.clear();
        runtime
            .advance_to(crate::runtime::Moment::from_millis(250))
            .expect("the simulated clock is monotonic");
        let _ = runtime.settle();
        assert_eq!(runtime.app().update_causes, [UpdateCause::Tick]);
        assert_eq!(
            runtime.app().drag_events.len(),
            1,
            "a late tick never catches up in a replay storm"
        );
        assert_eq!(runtime.app().drag_events[0].0, UpdateCause::Tick);
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(300))
        );
        let offset_after_one_late_replay = runtime.app().state.scroll().offset();
        let _ = runtime.settle();
        assert_eq!(
            runtime.app().state.scroll().offset(),
            offset_after_one_late_replay
        );
        assert_eq!(runtime.app().drag_events.len(), 1);
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn held_text_drag_reconciles_a_new_source_revision_before_replaying() {
        let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50))
        );

        runtime
            .advance_to(crate::runtime::Moment::from_millis(20))
            .unwrap();
        runtime.app_mut().lines = MANY_LINES
            .iter()
            .map(|_| ViewportLine::Plain("new"))
            .collect();
        runtime.app_mut().state.invalidate();
        let area = runtime.screen();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(70)),
            "the replacement source registration starts a fresh held-drag interval"
        );

        let drag_events = runtime.app().drag_events.len();
        let tick_count = runtime
            .app()
            .update_causes
            .iter()
            .filter(|cause| **cause == UpdateCause::Tick)
            .count();
        runtime
            .advance_to(crate::runtime::Moment::from_millis(50))
            .unwrap();
        assert!(!runtime.needs_settle());
        let _ = runtime.settle();
        assert_eq!(runtime.app().drag_events.len(), drag_events);
        assert_eq!(
            runtime
                .app()
                .update_causes
                .iter()
                .filter(|cause| **cause == UpdateCause::Tick)
                .count(),
            tick_count
        );

        runtime
            .advance_to(crate::runtime::Moment::from_millis(70))
            .unwrap();
        let _ = runtime.settle();
        assert_eq!(runtime.app().drag_events.len(), drag_events + 1);
        assert_eq!(
            runtime.app().drag_events.last().unwrap().0,
            UpdateCause::Tick
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(120))
        );
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn keyboard_selection_takes_over_and_cancels_held_text_drag() {
        let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        app.click_selects_word = true;
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50))
        );
        let pointer_selection = runtime.app().state.selection();

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert_ne!(runtime.app().state.selection(), pointer_selection);
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(runtime.next_deadline(), None);
        let keyboard_selection = runtime.app().state.selection();
        let stale_drag = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Move, 5, 9, KeyModifiers::SHIFT),
        );
        assert!(stale_drag.is_consumed());
        assert_eq!(runtime.app().state.selection(), keyboard_selection);
        let drag_events = runtime.app().drag_events.len();
        runtime
            .advance_to(crate::runtime::Moment::from_millis(50))
            .unwrap();
        assert!(!runtime.needs_settle());
        let _ = runtime.settle();
        assert_eq!(runtime.app().drag_events.len(), drag_events);
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn text_drag_cancellation_is_exact_and_a_fresh_press_rearms_it() {
        let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );
        let captured_drag_count = runtime.app().drag_events.len();
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50))
        );

        runtime.app_mut().request_repaint_after_ms = Some(25);
        runtime.app_mut().cancel_drag_autoscroll = Some((OTHER_ID, PartRef::of(Part::TEXT)));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Char('q'), KeyModifiers::NONE),
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(25)),
            "a wrong owner cannot cancel the gesture or its independent repaint deadline"
        );
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));

        runtime
            .advance_to(crate::runtime::Moment::from_millis(25))
            .unwrap();
        let _ = runtime.settle();
        assert_eq!(runtime.app().drag_events.len(), captured_drag_count);
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50)),
            "the exact gesture deadline remains after the unrelated repaint tick"
        );

        runtime.app_mut().cancel_drag_autoscroll = Some((SELECTION_ID, PartRef::of(Part::TRACK)));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Char('q'), KeyModifiers::NONE),
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50)),
            "a wrong part cannot cancel the captured text gesture"
        );

        let before_keyboard_selection = runtime.app().state.selection();
        let takeover = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert!(takeover.is_changed());
        assert_ne!(runtime.app().state.selection(), before_keyboard_selection);
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(runtime.next_deadline(), None);
        let selection_before_stale_move = runtime.app().state.selection();
        let captured_move = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Move, 5, 9, KeyModifiers::SHIFT),
        );
        assert!(captured_move.is_consumed());
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(runtime.app().state.selection(), selection_before_stale_move);
        let ticks_after_cancel = runtime
            .app()
            .update_causes
            .iter()
            .filter(|cause| **cause == UpdateCause::Tick)
            .count();
        let drags_after_cancel = runtime.app().drag_events.len();
        runtime
            .advance_to(crate::runtime::Moment::from_millis(50))
            .unwrap();
        assert!(!runtime.needs_settle());
        let _ = runtime.settle();
        assert_eq!(runtime.app().drag_events.len(), drags_after_cancel);
        assert_eq!(
            runtime
                .app()
                .update_causes
                .iter()
                .filter(|cause| **cause == UpdateCause::Tick)
                .count(),
            ticks_after_cancel
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Up, 5, 9, KeyModifiers::SHIFT),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
        );
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(100)),
            "a new press starts a fresh, independently timed drag"
        );
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn held_text_drag_cancels_after_release_inside_resize_modal_clear_or_missing_part() {
        for cancellation in ["release", "inside", "resize", "modal", "clear", "missing"] {
            let mut app = SelectionViewportApp::new(MANY_LINES.to_vec());
            app.area = Rect::new(2, 2, 12, 5);
            let mut runtime = selection_runtime(app);
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
            );
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Drag, 5, 9, KeyModifiers::SHIFT),
            );
            assert_eq!(
                runtime.next_deadline(),
                Some(crate::runtime::Moment::from_millis(50))
            );

            match cancellation {
                "release" => {
                    let _ = crate::runtime::stub::deliver(
                        &mut runtime,
                        selection_mouse(MouseKind::Up, 5, 9, KeyModifiers::SHIFT),
                    );
                    assert_eq!(runtime.capture_owner(), None);
                }
                "inside" => {
                    let _ = crate::runtime::stub::deliver(
                        &mut runtime,
                        selection_mouse(MouseKind::Move, 5, 3, KeyModifiers::SHIFT),
                    );
                    assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
                }
                "resize" => {
                    let _ = crate::runtime::stub::deliver(
                        &mut runtime,
                        crate::event::Input::Resize(40, 16),
                    );
                    assert_eq!(runtime.capture_owner(), None);
                }
                "modal" => {
                    runtime.app_mut().modal_open = true;
                    let _ = crate::runtime::stub::deliver(
                        &mut runtime,
                        selection_key(KeyCode::Char('q'), KeyModifiers::NONE),
                    );
                    assert_eq!(runtime.capture_owner(), None);
                    assert_eq!(runtime.next_deadline(), None);
                    assert!(
                        runtime
                            .app()
                            .modal_layer_receipts
                            .iter()
                            .any(|(opening, event)| {
                                opening.owner() == SELECTION_ID
                                    && *event == termrock_runtime::LayerEvent::Opened
                            })
                    );
                    assert_eq!(
                        runtime.app().selection_focus_outs,
                        [Some(MODAL_CONTROL)],
                        "the owner applies the exact FocusOut into the modal control"
                    );
                    assert_eq!(
                        runtime
                            .app()
                            .modal_owner_events
                            .iter()
                            .filter(|event| **event == termrock_runtime::LayerEvent::Opened)
                            .count(),
                        1,
                        "the viewport owner acknowledges the same modal opening once"
                    );
                    assert!(
                        runtime
                            .app()
                            .modal_control_events
                            .contains(&ModalControlEvent::FocusIn(FocusVia::Programmatic))
                    );
                    assert!(runtime.app().unhandled_selection_intents.is_empty());
                    assert!(runtime.app().unhandled_modal_control_intents.is_empty());
                }
                "clear" => {
                    runtime.app_mut().lines.clear();
                    let _ = crate::runtime::stub::deliver(
                        &mut runtime,
                        selection_key(KeyCode::Char('q'), KeyModifiers::NONE),
                    );
                    assert_eq!(runtime.capture_owner(), None);
                }
                "missing" => {
                    runtime.app_mut().show_viewport = false;
                    let area = runtime.screen();
                    let mut buffer = Buffer::empty(area);
                    runtime.draw_buffer(area, &mut buffer).commit_presented();
                    assert_eq!(runtime.capture_owner(), None);
                }
                _ => unreachable!(),
            }

            assert_eq!(
                runtime.next_deadline(),
                None,
                "{cancellation} must cancel the held timer"
            );
            let offset_after_cancel = runtime.app().state.scroll().offset();
            let drag_events_after_cancel = runtime.app().drag_events.len();
            let ticks_after_cancel = runtime
                .app()
                .update_causes
                .iter()
                .filter(|cause| **cause == UpdateCause::Tick)
                .count();
            runtime
                .advance_to(crate::runtime::Moment::from_millis(50))
                .expect("the simulated clock is monotonic");
            let _ = runtime.settle();
            assert_eq!(
                runtime
                    .app()
                    .update_causes
                    .iter()
                    .filter(|cause| **cause == UpdateCause::Tick)
                    .count(),
                ticks_after_cancel,
                "{cancellation} must not leave a due tick"
            );
            assert_eq!(runtime.app().state.scroll().offset(), offset_after_cancel);
            assert!(
                runtime.app().drag_events.len() == drag_events_after_cancel,
                "{cancellation} must not revive a held drag"
            );
            assert!(
                runtime.diagnostics().is_empty(),
                "{cancellation}: {:?}",
                runtime.diagnostics()
            );
        }
    }

    #[test]
    fn held_text_drag_scrolls_horizontally_only_for_opted_in_unwrapped_pan() {
        let mut app = SelectionViewportApp::new(LONG_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        app.pan = true;
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 20, 3, KeyModifiers::NONE),
        );
        let before = runtime.app().state.horizontal_offset();
        assert!(before > 0, "the real drag reaches the horizontal edge");
        assert_eq!(
            runtime.next_deadline(),
            Some(crate::runtime::Moment::from_millis(50))
        );
        runtime
            .advance_to(crate::runtime::Moment::from_millis(50))
            .expect("the simulated clock is monotonic");
        let _ = runtime.settle();
        assert_eq!(runtime.app().state.horizontal_offset(), before + 1);

        let mut app = SelectionViewportApp::new(LONG_LINES.to_vec());
        app.area = Rect::new(2, 2, 12, 5);
        app.pan = true;
        app.wrap = true;
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Down, 5, 3, KeyModifiers::NONE),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            selection_mouse(MouseKind::Drag, 20, 3, KeyModifiers::NONE),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
        assert_eq!(
            runtime.next_deadline(),
            None,
            "wrapped text has no horizontal autoscroll"
        );
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
    fn shift_pointer_reconciles_and_persists_oriented_selection_after_width_shrink() {
        let cases = [
            (
                (CellPos::new(0, 5), CellPos::new(0, 8)),
                (CellPos::new(0, 5), CellPos::new(0, 1)),
                (CellPos::new(0, 5), CellPos::new(0, 9)),
                "fghi",
            ),
            (
                (CellPos::new(0, 8), CellPos::new(0, 5)),
                (CellPos::new(0, 6), CellPos::new(0, 1)),
                (CellPos::new(0, 6), CellPos::new(0, 9)),
                "ghi",
            ),
        ];

        for (initial, after_shrink, after_restore, copied) in cases {
            let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdefghij")]);
            app.area = Rect::new(0, 0, 20, 3);
            app.state.select(initial.0, initial.1);
            let mut runtime = selection_runtime(app);

            runtime.app_mut().lines[0] = ViewportLine::Plain("abcdef");
            runtime.app_mut().state.invalidate();
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 2, 0, KeyModifiers::SHIFT),
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
            assert_eq!(
                runtime.app().state.selection,
                Some(after_shrink),
                "the surviving oriented anchor must clamp before Shift press"
            );

            runtime.app_mut().lines[0] = ViewportLine::Plain("abcdefghij");
            runtime.app_mut().state.invalidate();
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 10, 0, KeyModifiers::SHIFT),
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 10, 0));
            assert_eq!(runtime.app().state.selection, Some(after_restore));
            assert_eq!(copy_from_viewport(&mut runtime), copied);
        }
    }

    #[test]
    fn shift_pointer_reconciles_and_persists_anchor_after_line_count_shrink() {
        let cases = [
            (CellPos::new(1, 2), CellPos::new(3, 4)),
            (CellPos::new(3, 2), CellPos::new(1, 3)),
        ];
        for (anchor, head) in cases {
            let mut app = SelectionViewportApp::new(vec![
                ViewportLine::Plain("zero"),
                ViewportLine::Plain("one"),
                ViewportLine::Plain("two"),
                ViewportLine::Plain("three"),
            ]);
            app.area = Rect::new(0, 0, 16, 4);
            app.state.select(anchor, head);
            let mut runtime = selection_runtime(app);

            runtime.app_mut().lines.truncate(2);
            runtime.app_mut().state.invalidate();
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 2, 0, KeyModifiers::SHIFT),
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
            assert_eq!(
                runtime.app().state.selection,
                Some((CellPos::new(1, 2), CellPos::new(0, 1))),
                "the forward and reverse anchors must clamp to the surviving final line"
            );
            assert_eq!(copy_from_viewport(&mut runtime), "ero\non");

            runtime.app_mut().lines = vec![
                ViewportLine::Plain("zero"),
                ViewportLine::Plain("one"),
                ViewportLine::Plain("two"),
                ViewportLine::Plain("three"),
            ];
            runtime.app_mut().state.invalidate();
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                selection_mouse(MouseKind::Down, 3, 2, KeyModifiers::SHIFT),
            );
            let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 3, 2));
            assert_eq!(
                runtime.app().state.selection,
                Some((CellPos::new(1, 2), CellPos::new(2, 2)))
            );
            assert_eq!(copy_from_viewport(&mut runtime), "e\ntw");
        }
    }

    #[test]
    fn plain_pointer_drag_reconciles_active_anchor_after_source_shrink() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdefghij")]);
        app.area = Rect::new(0, 0, 16, 3);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 7, 0));
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 6)));

        runtime.app_mut().lines[0] = ViewportLine::Plain("abc");
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 2, 0));
        assert_eq!(
            runtime.app().state.anchor,
            Some(CellPos::new(0, 3)),
            "the active ordinary anchor must persist its clamped endpoint"
        );
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 1)))
        );
        assert_eq!(copy_from_viewport(&mut runtime), "bc");

        runtime.app_mut().lines[0] = ViewportLine::Plain("abcdefghij");
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 2, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 1))),
            "restoring source must not revive the pre-shrink anchor"
        );
        assert_eq!(copy_from_viewport(&mut runtime), "bc");
    }

    #[test]
    fn pointer_anchor_inside_wide_grapheme_snaps_and_stays_snapped_after_restore() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("a界b")]);
        app.area = Rect::new(0, 0, 12, 3);
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 3, 0));
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 2)));

        runtime.app_mut().lines[0] = ViewportLine::Plain("a界");
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 2, 0));
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 1)));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 1), CellPos::new(0, 1)))
        );

        runtime.app_mut().lines[0] = ViewportLine::Plain("a界bb");
        runtime.app_mut().state.invalidate();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 4, 0));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 1), CellPos::new(0, 3)))
        );
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 4, 0));
        assert_eq!(copy_from_viewport(&mut runtime), "界");
    }

    #[test]
    fn no_op_keyboard_boundary_clamps_plain_anchor_before_source_restore() {
        let mut app = SelectionViewportApp::new(vec![ViewportLine::Plain("abcdefghij")]);
        app.area = Rect::new(0, 0, 16, 3);
        app.state.set_caret(Some(CellPos::new(0, 2)));
        let mut runtime = selection_runtime(app);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 9, 0));
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 8)));

        runtime.app_mut().lines[0] = ViewportLine::Plain("abc");
        runtime.app_mut().state.invalidate();
        let boundary = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Right, KeyModifiers::SHIFT),
        );
        assert!(boundary.is_consumed());
        assert!(!boundary.is_changed(), "boundary selection remains a no-op");
        assert_eq!(runtime.app().state.anchor, Some(CellPos::new(0, 3)));
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 2)));

        runtime.app_mut().lines[0] = ViewportLine::Plain("abcdefghijklm");
        runtime.app_mut().state.invalidate();
        let moved = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 11, 0));
        assert!(moved.is_changed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 3), CellPos::new(0, 10)))
        );
        assert_eq!(copy_from_viewport(&mut runtime), "defghij");
    }

    #[test]
    fn keyboard_takeover_ends_held_word_gesture_without_changing_producer_caret() {
        let mut app =
            SelectionViewportApp::new(vec![ViewportLine::Plain("alpha beta gamma delta")]);
        app.area = Rect::new(0, 0, 24, 3);
        app.state.set_caret(Some(CellPos::new(0, 2)));
        let mut runtime = selection_runtime(app);
        deliver_click_count(&mut runtime, 1, 7, 0);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, 7, 0));
        assert_eq!(runtime.capture_owner(), Some(SELECTION_ID));
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 10)))
        );

        let extended = crate::runtime::stub::deliver(
            &mut runtime,
            selection_key(KeyCode::Left, KeyModifiers::SHIFT),
        );
        assert!(extended.is_changed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 9)))
        );
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 9)));
        assert_eq!(runtime.app().state.caret(), Some(CellPos::new(0, 2)));

        let moved = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Drag, 14, 0));
        assert!(moved.is_consumed());
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 6), CellPos::new(0, 9))),
            "captured pointer movement must not resume the ended word gesture"
        );
        assert_eq!(runtime.app().state.browse, Some(CellPos::new(0, 9)));
        assert_eq!(copy_from_viewport(&mut runtime), "bet");

        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, 14, 0));
        deliver_click_count(&mut runtime, 2, 12, 0);
        assert_eq!(
            runtime.app().state.selection,
            Some((CellPos::new(0, 11), CellPos::new(0, 16))),
            "a fresh double press must start a new word gesture"
        );
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
                &mut edge_state.horizontal,
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
            source_revision: None,
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
            source_revision: None,
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
    fn routed_window_view_keeps_borrowed_and_projected_cold_warm_work_unchanged() {
        fn snapshots(
            projected_source: bool,
            with_view: bool,
        ) -> (ViewportWorkSnapshot, ViewportWorkSnapshot) {
            let probe = ViewportWorkProbe::default();
            let viewport = TextViewport::new(ID).wrap(true).work_probe(&probe);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let lines = [ViewportLine::Plain("abcdefgh"); 7];
            let mut projected = ProjectedText::default();
            for _ in 0..lines.len() {
                projected.push_line([("abcdefgh", None, Modifier::empty())]);
            }
            let window = VerticalWindow::new(Rect::new(2, 2, 8, 4), 8, 2)
                .expect("the native test window is inside its logical body");
            let state = ViewportState::default();
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    match (projected_source, with_view) {
                        (false, false) => {
                            let _ = viewport.draw_window_routed(ui, window, &state, &lines, 1);
                        }
                        (false, true) => {
                            let _ = viewport
                                .draw_window_routed_with_view(ui, window, &state, &lines, 1);
                        }
                        (true, false) => {
                            let _ = viewport
                                .draw_projected_window_routed(ui, window, &state, &projected, 1);
                        }
                        (true, true) => {
                            let _ = viewport.draw_projected_window_routed_with_view(
                                ui, window, &state, &projected, 1,
                            );
                        }
                    }
                })
                .commit_presented();
            let cold = probe.snapshot();
            probe.reset();
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    match (projected_source, with_view) {
                        (false, false) => {
                            let _ = viewport.draw_window_routed(ui, window, &state, &lines, 1);
                        }
                        (false, true) => {
                            let _ = viewport
                                .draw_window_routed_with_view(ui, window, &state, &lines, 1);
                        }
                        (true, false) => {
                            let _ = viewport
                                .draw_projected_window_routed(ui, window, &state, &projected, 1);
                        }
                        (true, true) => {
                            let _ = viewport.draw_projected_window_routed_with_view(
                                ui, window, &state, &projected, 1,
                            );
                        }
                    }
                })
                .commit_presented();
            (cold, probe.snapshot())
        }

        for projected_source in [false, true] {
            let legacy = snapshots(projected_source, false);
            let with_view = snapshots(projected_source, true);
            assert_eq!(with_view, legacy, "projected source={projected_source}");
            assert_eq!(legacy.0.indexed_lines, 7);
            assert_eq!(legacy.1.indexed_lines, 0);
            assert!(legacy.0.visible_rows > 0);
            assert!(legacy.1.visible_rows > 0);
        }
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
                source_revision: None,
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
            source_revision: None,
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
            source_revision: None,
        };

        layout.ensure_with_widest(&short, key, true);
        assert_eq!(layout.widest_cols, 1);

        let next = LayoutKey {
            layout_revision: 2,
            source_revision: None,
            ..key
        };
        layout.ensure_with_widest(&medium, next, true);
        assert_eq!(layout.widest_cols, 6);

        let saturated = LayoutKey {
            layout_revision: u64::MAX,
            source_revision: None,
            ..next
        };
        layout.ensure_with_widest(&short, saturated, true);
        assert_eq!(layout.widest_cols, 1);
        layout.ensure_with_widest(&long, saturated, true);
        assert_eq!(layout.widest_cols, 10);
    }

    #[test]
    fn source_revision_saturation_never_reuses_stale_layout() {
        let short = [ViewportLine::Plain("x")];
        let long = [ViewportLine::Plain("0123456789")];
        let key = LayoutKey {
            width: 4,
            wrap: true,
            generation: 7,
            layout_revision: 2,
            source_revision: Some(u64::MAX),
        };
        let mut layout = ViewportLayout::default();

        assert_eq!(layout.ensure_with_widest(&short, key, true), 1);
        assert_eq!(layout.total(), 1);
        assert_eq!(layout.ensure_with_widest(&long, key, true), 1);
        assert_eq!(layout.total(), 3);
        assert_eq!(
            layout.widest_cols(),
            None,
            "wrapped layouts do not cache width"
        );
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
            source_revision: None,
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

    fn routed_runtime(app: RoutedViewportApp) -> Runtime<RoutedViewportApp> {
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        for _ in 0..16 {
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            if !runtime.needs_settle() {
                return runtime;
            }
            let _ = runtime.settle();
        }
        panic!(
            "routed viewport fixture did not settle: {:?}",
            runtime.diagnostics()
        );
    }

    fn routed_text_point(runtime: &Runtime<RoutedViewportApp>) -> Position {
        let text = runtime
            .area_of_part(ROUTED_CHILD, PartRef::of(Part::TEXT))
            .expect("routed viewport paints its text part");
        Position::new(text.x, text.y)
    }

    #[test]
    fn routed_text_ignores_a_short_child_then_services_one_parent_attempt() {
        let mut runtime = routed_runtime(RoutedViewportApp::new(false, false, false));
        assert!(runtime.app().linked.get());
        assert!(runtime.app().child_key.get().is_some());
        let point = routed_text_point(&runtime);
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::V)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );

        assert_eq!(runtime.app().state.scroll().offset(), 0);
        assert!(runtime.app().parent.offset() > 0);
        assert_eq!(runtime.app().event_updates, 1);
        assert_eq!(runtime.app().route_callbacks, 1);
        assert_eq!(runtime.app().route_owners, [ROUTED_PARENT]);
        assert!(runtime.app().route_callbacks_empty);
        assert_eq!(runtime.app().tick_updates, 0);
    }

    #[test]
    fn failed_child_acknowledgement_rolls_back_prepared_scroll_state() {
        let mut app = RoutedViewportApp::new(true, false, false);
        app.preack_child_route = true;
        let mut runtime = routed_runtime(app);
        let point = routed_text_point(&runtime);
        let before = runtime.app().state.clone();

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );

        assert_eq!(
            runtime.app().state,
            before,
            "a failed route acknowledgement restores both prepared axes"
        );
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn empty_routed_viewport_ignores_wheel_and_services_the_containing_parent() {
        let mut app = RoutedViewportApp::new(false, false, false);
        app.empty = true;
        let mut runtime = routed_runtime(app);
        assert!(runtime.app().child_key.get().is_some());
        let point = routed_text_point(&runtime);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );

        assert_eq!(runtime.app().state.scroll().offset(), 0);
        assert!(runtime.app().parent.offset() > 0);
        assert_eq!(runtime.app().route_owners, [ROUTED_PARENT]);
        assert!(runtime.app().route_callbacks_empty);
        assert!(
            runtime.diagnostics().is_empty(),
            "{:?}",
            runtime.diagnostics()
        );
    }

    #[test]
    fn route_wheel_partition_consumes_only_the_exact_physical_intent() {
        let mut runtime = routed_runtime(RoutedViewportApp::new(true, false, false));
        let point = routed_text_point(&runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        let attempt = runtime
            .app()
            .last_event_attempt
            .expect("the real wheel event exposed its exact route attempt");
        let exact_wheel = Intent::Wheel {
            axis: attempt.axis(),
            delta: attempt.delta(),
            part: attempt.part(),
            pos: attempt.position(),
        };
        let different_axis = Intent::Wheel {
            axis: Axis::H,
            delta: attempt.delta(),
            part: attempt.part(),
            pos: attempt.position(),
        };
        let different_position = Intent::Wheel {
            axis: attempt.axis(),
            delta: attempt.delta(),
            part: attempt.part(),
            pos: Position::new(attempt.position().x.saturating_add(1), attempt.position().y),
        };
        let unrelated = Intent::FocusOut {
            to: Some(ROUTED_PARENT),
        };
        let cancel = Intent::Cancel;
        let queued = [
            unrelated,
            exact_wheel,
            different_axis,
            cancel,
            different_position,
        ];

        assert_eq!(
            TextViewport::count_route_wheel_intents(queued.iter().copied(), attempt),
            1
        );
        assert_eq!(
            TextViewport::without_route_wheel_intent(queued.iter().copied(), attempt)
                .collect::<Vec<_>>(),
            vec![unrelated, different_axis, cancel, different_position],
            "only the exact axis/delta/part/position is consumed; unrelated order is kept"
        );
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn routed_pan_target_and_vertical_bar_are_disjoint_and_keep_pointer_behavior() {
        let mut runtime = routed_runtime(RoutedViewportApp::new(true, false, true));
        assert!(runtime.app().linked.get());
        assert!(runtime.app().child_key.get().is_some());
        let text = runtime
            .area_of_part(ROUTED_CHILD, PartRef::of(Part::TEXT))
            .expect("text body is visible");
        let point = Position::new(text.x, text.y);
        let child_scrolls = runtime
            .registry()
            .regions()
            .iter()
            .filter(|region| {
                region.owner == ROUTED_CHILD && region.kind == crate::ui::hit::RegionKind::Scroll
            })
            .collect::<Vec<_>>();
        assert_eq!(
            child_scrolls.len(),
            2,
            "text route plus a separate bar strip"
        );
        let body_route = child_scrolls
            .iter()
            .find(|region| region.area == text)
            .expect("one routed target covers the clipped text body");
        assert_eq!(body_route.scroll().map(|(axes, _)| axes), Some(Axes::Both));
        let bar = child_scrolls
            .iter()
            .find(|region| region.area != text)
            .expect("the vertical scrollbar keeps an ordinary wheel strip");
        assert_eq!(bar.scroll().map(|(axes, _)| axes), Some(Axes::V));
        assert!(bar.area.intersection(text).is_empty());
        assert_eq!(
            child_scrolls
                .iter()
                .filter(|region| region.area.contains(point))
                .count(),
            1,
            "the text point has exactly one candidate for this owner"
        );
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::H)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::V)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );

        let vertical_before = runtime.app().state.scroll().offset();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), vertical_before);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);

        let track = runtime
            .area_of_part(ROUTED_CHILD, PartRef::of(Part::TRACK))
            .expect("suppression keeps the painted vertical track registered");
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Down, track.x, track.bottom().saturating_sub(1)),
        );
        assert!(runtime.app().state.scroll().at_end());
    }

    #[test]
    fn routed_draw_preserves_cells_and_non_scroll_geometry() {
        let mut state = ViewportState::default();
        state.set_follow(false);
        state.scroll.apply_layout(7, ROUTED_LONG.len());
        state.horizontal.set_content(40);
        state.horizontal.set_viewport(22);
        let render = |routed: bool| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let mut content = None;
            let mut route_key = None;
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    let viewport = TextViewport::new(ROUTED_CHILD).horizontal_pan(true);
                    if routed {
                        let (rect, key) =
                            viewport.draw_routed(ui, ROUTED_AREA, &state, &ROUTED_LONG, 7);
                        content = Some(rect);
                        route_key = key;
                    } else {
                        content = Some(viewport.draw(ui, ROUTED_AREA, &state, &ROUTED_LONG));
                    }
                })
                .commit_presented();
            let parts = TextViewport::PARTS
                .iter()
                .map(|part| runtime.area_of_part(ROUTED_CHILD, PartRef::of(*part)))
                .collect::<Vec<_>>();
            let non_scroll_regions = runtime
                .registry()
                .regions()
                .iter()
                .filter(|region| region.kind != crate::ui::hit::RegionKind::Scroll)
                .map(|region| (region.owner, region.part, region.kind, region.area))
                .collect::<Vec<_>>();
            let scroll_regions = runtime
                .registry()
                .regions()
                .iter()
                .filter(|region| region.owner == ROUTED_CHILD)
                .filter(|region| region.kind == crate::ui::hit::RegionKind::Scroll)
                .map(|region| {
                    (
                        region.part,
                        region.area,
                        region.scroll().map(|(axes, _)| axes),
                    )
                })
                .collect::<Vec<_>>();
            (
                buffer,
                content,
                parts,
                non_scroll_regions,
                scroll_regions,
                route_key,
                runtime.diagnostics().to_vec(),
            )
        };

        let ordinary = render(false);
        let routed = render(true);
        assert_eq!(routed.0, ordinary.0, "all painted cells stay identical");
        assert_eq!(routed.1, ordinary.1, "content rectangles stay identical");
        assert_eq!(
            &routed.2[1..],
            &ordinary.2[1..],
            "text, gutter, and scrollbar pointer parts stay identical"
        );
        assert_eq!(
            routed.2[0], routed.1,
            "route-mode CONTAINER stays on the clipped text target, not the later bar strip"
        );
        assert_eq!(
            ordinary.2[0], ordinary.1,
            "ordinary CONTAINER remains the visible inner ScrollRegion body"
        );
        assert_eq!(
            routed.3, ordinary.3,
            "focus, pointer, and decoration registrations stay identical"
        );
        assert_eq!(routed.4.len(), 2);
        assert_eq!(ordinary.4.len(), 2);
        let routed_content = routed.1.expect("route-mode text content is present");
        let routed_text_scroll = routed
            .4
            .iter()
            .find(|(_, area, _)| *area == routed_content)
            .expect("the clipped text body has its unique routed scroll target");
        assert_eq!(routed_text_scroll.2, Some(Axes::Both));
        let routed_bar_scroll = routed
            .4
            .iter()
            .find(|(_, area, _)| *area != routed_content)
            .expect("the vertical bar keeps an ordinary wheel region");
        assert_eq!(routed_bar_scroll.2, Some(Axes::V));
        assert!(routed_bar_scroll.1.intersection(routed_content).is_empty());
        let ordinary_pan_scroll = ordinary
            .4
            .iter()
            .find(|(_, _, axes)| *axes == Some(Axes::Both))
            .expect("ordinary pan keeps its combined-axis text registration");
        assert_eq!(
            ordinary_pan_scroll.1,
            ordinary.1.expect("ordinary text is present")
        );
        let ordinary_vertical_scroll = ordinary
            .4
            .iter()
            .find(|(_, _, axes)| *axes == Some(Axes::V))
            .expect("ordinary ScrollRegion keeps its broad vertical registration");
        assert_eq!(
            ordinary_vertical_scroll.1.intersection(routed_content),
            routed_content
        );
        assert_ne!(routed.4, ordinary.4, "only wheel routing geometry changes");
        assert!(routed.5.is_some());
        assert_eq!(routed.6, ordinary.6);
    }

    #[test]
    fn clipped_and_too_small_routed_text_geometry_fails_closed() {
        let state = ViewportState::default();
        let render = |clip: Rect, area: Rect| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            let mut key = None;
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    ui.with_area(clip, |ui| {
                        key = TextViewport::new(ROUTED_CHILD)
                            .horizontal_pan(true)
                            .draw_routed(ui, area, &state, &ROUTED_LONG, 7)
                            .1;
                    });
                })
                .commit_presented();
            let scroll_count = runtime
                .registry()
                .regions()
                .iter()
                .filter(|region| {
                    region.owner == ROUTED_CHILD
                        && region.kind == crate::ui::hit::RegionKind::Scroll
                })
                .count();
            (key, scroll_count)
        };

        assert_eq!(render(Rect::new(50, 20, 1, 1), ROUTED_AREA), (None, 0));
        assert_eq!(
            render(Rect::new(0, 0, 40, 20), Rect::new(2, 2, 1, 1)),
            (None, 0)
        );
    }

    #[test]
    fn targeted_text_viewport_fallback_services_only_its_exact_wheel_attempt() {
        let mut app = RoutedViewportApp::new(true, false, false);
        app.front_router = true;
        let mut runtime = routed_runtime(app);
        assert!(runtime.app().linked.get());
        assert!(runtime.app().front_key.get().is_some());
        let point = routed_text_point(&runtime);
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::V)
                .map(|hit| hit.owner),
            Some(ROUTED_FRONT)
        );
        let before = runtime.app().state.scroll().offset();

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );

        assert!(runtime.app().state.scroll().offset() > before);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 1);
        assert_eq!(runtime.app().route_owners, [ROUTED_CHILD]);
        assert!(runtime.app().route_callbacks_empty);
        assert_eq!(runtime.app().event_updates, 1);
        assert_eq!(runtime.app().tick_updates, 0);
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn active_pan_without_horizontal_overflow_acknowledges_ignored_without_state_change() {
        let mut app = RoutedViewportApp::new(false, false, true);
        app.same_count = true;
        let mut runtime = routed_runtime(app);
        let point = routed_text_point(&runtime);
        assert!(runtime.app().child_key.get().is_some());
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::H)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD),
            "active unwrapped pan registers H even before overflow"
        );
        let state_before = runtime.app().state.clone();

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );

        assert_eq!(runtime.app().state, state_before);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
        assert!(
            runtime
                .app()
                .observed_route_attempts
                .iter()
                .any(|(cause, attempt)| {
                    *cause == UpdateCause::Event && *attempt == Some((ROUTED_CHILD, Axis::H, 7))
                })
        );
    }

    #[test]
    fn routed_horizontal_wheel_keeps_four_cell_steps_and_consumes_both_edges() {
        let mut runtime = routed_runtime(RoutedViewportApp::new(true, false, true));
        let point = routed_text_point(&runtime);
        let vertical_before = runtime.app().state.scroll().offset();

        for delta in [i16::MIN, 0] {
            let _ = crate::runtime::stub::deliver(
                &mut runtime,
                mouse(MouseKind::Wheel(Axis::H, delta), point.x, point.y),
            );
            assert_eq!(runtime.app().state.horizontal_offset(), 0);
            assert_eq!(runtime.app().state.scroll().offset(), vertical_before);
            assert_eq!(runtime.app().route_callbacks, 0);
        }

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, i16::MAX), point.x, point.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), vertical_before);

        let max_horizontal_offset = runtime.app().state.horizontal.max_offset();
        runtime
            .app_mut()
            .state
            .horizontal
            .scroll_to(max_horizontal_offset);
        let end = runtime.app().state.horizontal_offset();
        assert!(end > 4);
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let point = routed_text_point(&runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, i16::MAX), point.x, point.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), end);
        assert_eq!(runtime.app().state.scroll().offset(), vertical_before);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
    }

    #[test]
    fn routed_vertical_wheel_changes_interior_and_consumes_both_edges() {
        let mut runtime = routed_runtime(RoutedViewportApp::new(true, false, false));
        let point = routed_text_point(&runtime);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, -1), point.x, point.y),
        );
        assert_eq!(runtime.app().state.scroll().offset(), 0);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        let inside = runtime.app().state.scroll().offset();
        assert!(inside > 0);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);

        runtime.app_mut().state.scroll.jump_end();
        let end = runtime.app().state.scroll().offset();
        assert!(end > inside);
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let point = routed_text_point(&runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        assert_eq!(runtime.app().state.scroll().offset(), end);
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
    }

    #[test]
    fn wrapped_or_disabled_pan_registers_only_vertical_routes() {
        for app in [
            RoutedViewportApp::new(true, true, true),
            RoutedViewportApp::new(true, false, false),
        ] {
            let runtime = routed_runtime(app);
            let point = routed_text_point(&runtime);
            assert!(runtime.app().child_key.get().is_some());
            assert_eq!(
                runtime
                    .registry()
                    .hit_scroll(point, Axis::V)
                    .map(|hit| hit.owner),
                Some(ROUTED_CHILD)
            );
            assert_eq!(runtime.registry().hit_scroll(point, Axis::H), None);
        }
    }

    #[test]
    fn same_length_source_revision_rejects_old_horizontal_attempt_then_remeasures() {
        let mut app = RoutedViewportApp::new(false, false, true);
        app.same_count = true;
        app.replace_on_event = true;
        let mut runtime = routed_runtime(app);
        let point = routed_text_point(&runtime);
        let before = runtime.app().state.clone();
        assert!(runtime.app().child_key.get().is_some());
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::H)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );

        assert!(
            runtime
                .app()
                .observed_route_attempts
                .iter()
                .any(|(cause, attempt)| {
                    *cause == UpdateCause::Event && *attempt == Some((ROUTED_CHILD, Axis::H, 7))
                })
        );
        assert_eq!(
            runtime
                .app()
                .observed_update_state
                .iter()
                .find(|row| row.0 == UpdateCause::Event),
            Some(&(UpdateCause::Event, 8, 8, 8))
        );
        assert_eq!(
            runtime.app().state,
            before,
            "stale source rejects before mutation"
        );
        assert_eq!(runtime.app().parent.offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
        assert!(
            runtime.diagnostics().is_empty(),
            "{:?}",
            runtime.diagnostics()
        );

        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let point = routed_text_point(&runtime);
        assert!(runtime.app().child_key.get().is_some());
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::H)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
    }

    #[test]
    fn same_length_wrapped_source_revision_rebuilds_vertical_measurements() {
        let mut app = RoutedViewportApp::new(false, true, false);
        app.same_count = true;
        app.replace_on_event = true;
        let mut runtime = routed_runtime(app);
        let point = routed_text_point(&runtime);
        let before = runtime.app().state.clone();

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        assert_eq!(
            runtime.app().state,
            before,
            "stale source rejects before mutation"
        );

        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let point = routed_text_point(&runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        assert!(runtime.app().state.scroll().offset() > 0);
        assert_eq!(runtime.app().state.scroll().content_len(), 64);
    }

    #[test]
    fn same_length_long_to_short_pan_remeasures_without_wheel_motion() {
        let mut app = RoutedViewportApp::new(true, false, true);
        app.same_count = true;
        app.replace_on_event = true;
        app.replacement_long = false;
        let mut runtime = routed_runtime(app);
        let before = runtime.app().state.clone();
        let point = routed_text_point(&runtime);
        assert_eq!(
            runtime
                .registry()
                .hit_scroll(point, Axis::H)
                .map(|hit| hit.owner),
            Some(ROUTED_CHILD)
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );
        assert_eq!(
            runtime.app().state,
            before,
            "the old revision has no side effects"
        );
        assert!(
            runtime
                .app()
                .observed_route_attempts
                .iter()
                .any(|(cause, attempt)| {
                    *cause == UpdateCause::Event && *attempt == Some((ROUTED_CHILD, Axis::H, 7))
                })
        );

        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('x')));
        assert_eq!(runtime.app().state.horizontal.content_len(), 5);
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
        let point = routed_text_point(&runtime);
        let before_wheel = runtime.app().state.clone();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );
        assert_eq!(runtime.app().state.horizontal_offset(), 0);
        assert_eq!(
            runtime.app().state.scroll().offset(),
            before_wheel.scroll().offset()
        );
        assert_eq!(runtime.app().parent.offset(), 0);
    }

    #[test]
    fn same_length_wrapped_long_to_short_rebuilds_vertical_measurements() {
        let mut app = RoutedViewportApp::new(true, true, false);
        app.same_count = true;
        app.replace_on_event = true;
        app.replacement_long = false;
        let mut runtime = routed_runtime(app);
        assert_eq!(runtime.app().state.scroll().content_len(), 64);
        let before = runtime.app().state.clone();
        let point = routed_text_point(&runtime);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        assert_eq!(
            runtime.app().state,
            before,
            "the old revision has no side effects"
        );
        assert!(
            runtime
                .app()
                .observed_route_attempts
                .iter()
                .any(|(cause, attempt)| {
                    *cause == UpdateCause::Event && *attempt == Some((ROUTED_CHILD, Axis::V, 7))
                })
        );

        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('x')));
        assert_eq!(runtime.app().state.scroll().content_len(), 32);
        assert_eq!(runtime.app().state.scroll().offset(), 0);
        let point = routed_text_point(&runtime);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::V, 1), point.x, point.y),
        );
        assert!(runtime.app().state.scroll().offset() > 0);
        assert_eq!(runtime.app().state.scroll().content_len(), 32);
        assert_eq!(runtime.app().parent.offset(), 0);
    }

    #[test]
    fn projected_text_uses_the_same_routed_horizontal_service() {
        let mut projected = ProjectedText::default();
        for _ in 0..32 {
            projected.push_line([(
                "0123456789012345678901234567890123456789",
                None,
                ratatui_core::style::Modifier::empty(),
            )]);
        }
        let mut app = RoutedViewportApp::new(false, false, true);
        app.projected = Some(projected);
        let mut runtime = routed_runtime(app);
        let point = routed_text_point(&runtime);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Wheel(Axis::H, 1), point.x, point.y),
        );

        assert_eq!(runtime.app().state.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.scroll().offset(), 0);
        assert_eq!(runtime.app().route_callbacks, 0);
    }
}
