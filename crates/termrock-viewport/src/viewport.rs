use core::fmt;
use core::ops::{ControlFlow, Range};

#[cfg(feature = "testing")]
use std::cell::Cell;

use crate::theme::PaintStyle;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::{Acc, PartStyle, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::{FollowCursor, ScrollState};
use crate::text::Span;
use crate::text::measure::{grapheme_width, graphemes};
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
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

#[derive(Clone, Copy, Debug)]
struct VCell {
    span: usize,
    at: usize,
    len: usize,
    w: u16,
    tab: bool,
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

fn cell_text<'a>(line: impl Into<LineRef<'a>>, c: VCell) -> &'a str {
    let line = line.into();
    if c.tab {
        " "
    } else {
        run_text(line, c.span)
            .get(c.at..c.at.saturating_add(c.len))
            .unwrap_or("")
    }
}

fn walk_run(
    ix: usize,
    text: &str,
    f: &mut impl FnMut(VCell) -> ControlFlow<()>,
) -> ControlFlow<()> {
    for (at, g) in graphemes(text) {
        if g == "\t" {
            for _ in 0..TAB_WIDTH {
                f(VCell {
                    span: ix,
                    at,
                    len: g.len(),
                    w: 1,
                    tab: true,
                })?;
            }
            continue;
        }
        let w = grapheme_width(g);
        if w == 0 {
            continue;
        }
        f(VCell {
            span: ix,
            at,
            len: g.len(),
            w,
            tab: false,
        })?;
    }
    ControlFlow::Continue(())
}

fn walk_line<'a>(line: impl Into<LineRef<'a>>, mut f: impl FnMut(VCell) -> ControlFlow<()>) {
    let line = line.into();
    for index in 0..run_count(line) {
        if walk_run(index, run_text(line, index), &mut f).is_break() {
            return;
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
}

#[derive(Debug, Default)]
struct ViewportLayout {
    key: LayoutKey,
    prefix: Vec<usize>,
    lines: usize,
}

impl ViewportLayout {
    fn ensure<'a>(&mut self, lines: impl Into<LineSet<'a>>, key: LayoutKey) -> usize {
        let lines = lines.into();
        let reusable = key.generation != u32::MAX;
        if reusable && self.key == key && self.lines == lines.len() {
            return 0;
        }
        let append = reusable && self.key == key && self.lines <= lines.len();
        if !append {
            self.key = key;
            self.prefix.clear();
            self.lines = 0;
        }
        if !key.wrap {
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
        let text = cell_text(line, c);
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewportState {
    scroll: ScrollState,
    follow: FollowCursor,
    selection: Option<(CellPos, CellPos)>,
    anchor: Option<CellPos>,
    caret: Option<CellPos>,
    generation: u32,
}

impl Default for ViewportState {
    fn default() -> Self {
        ViewportState {
            scroll: ScrollState::default(),
            follow: FollowCursor::Always,
            selection: None,
            anchor: None,
            caret: None,
            generation: 0,
        }
    }
}

impl ViewportState {
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
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
    }

    pub const fn clear_selection(&mut self) -> bool {
        let had = self.selection.is_some();
        self.selection = None;
        self.anchor = None;
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
        if self
            .selection
            .is_some_and(|(a, b)| a.line < dropped && b.line < dropped)
        {
            self.selection = None;
        } else if let Some((a, b)) = self.selection.as_mut() {
            shift(a, dropped);
            shift(b, dropped);
        }
        if let Some(a) = self.anchor.as_mut() {
            shift(a, dropped);
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
                    out.push_str(cell_text(line, c));
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
    caret_when_unfocused: bool,
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
            .field("caret_when_unfocused", &self.caret_when_unfocused)
            .field("click_selects_word", &self.click_selects_word)
            .field("slot", &self.slot.map(|(p, _)| p))
            .finish_non_exhaustive()
    }
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
            caret_when_unfocused: false,
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

    /// Allows this viewport to publish its state caret while it lacks
    /// keyboard focus. The caret remains subject to follow, selection,
    /// viewport and clipping rules; this setting does not make the viewport
    /// editable or change its focus styles.
    #[must_use]
    pub const fn caret_when_unfocused(mut self, enabled: bool) -> Self {
        self.caret_when_unfocused = enabled;
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

    const fn text_width(width: u16) -> u16 {
        width.saturating_sub(2)
    }

    const fn layout_key(&self, st: &ViewportState, width: u16) -> LayoutKey {
        LayoutKey {
            width,
            wrap: self.wrap,
            generation: st.generation,
        }
    }

    fn pos_at(
        &self,
        cx: &mut Cx<'_>,
        lines: LineSet<'_>,
        geometry: (Rect, LayoutKey, usize),
        pos: Position,
    ) -> Option<CellPos> {
        let (area, key, offset) = geometry;
        if area.is_empty() || lines.is_empty() {
            return None;
        }
        let layout = cx.cache::<ViewportLayout>(self.id);
        let indexed = layout.ensure(lines, key);
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let dy = usize::from(pos.y.saturating_sub(area.y));
        let visual = offset.saturating_add(dy);
        let (li, row) = layout.row_start(lines, visual);
        let line = lines.get(li)?;
        let (start, end) = row_cols(line, key.width, self.wrap, row);
        let dx = usize::from(pos.x.saturating_sub(area.x.saturating_add(1)));
        let last = end.min(line_cols(line));
        Some(CellPos {
            line: li,
            col: start.saturating_add(dx).min(last),
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
        let area = cx.area(self.id).unwrap_or(Rect::ZERO);
        let width = cx
            .layout(self.id)
            .map_or_else(|| Self::text_width(area.width), |l| l.cols);
        let key = self.layout_key(st, width);
        let (total, indexed) = {
            let layout = cx.cache::<ViewportLayout>(self.id);
            let indexed = layout.ensure(lines, key);
            (layout.total(), indexed)
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let before = st.scroll.offset();
        let bar = self.bar().update(cx, &mut st.scroll, total);
        let mut acc = Acc::<ViewportAction>::new();
        acc.fold(&bar);
        if st.scroll.offset() != before {
            st.follow = if st.scroll.at_end() {
                FollowCursor::Always
            } else {
                FollowCursor::OnMove
            };
        }
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    if let Some(cmd) = Binding::command(BINDINGS, action) {
                        Self::command(st, lines, cmd, &mut acc);
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::TEXT, ..
                        },
                    pos,
                    ..
                } => self.handle_text_pointer(cx, st, lines, (area, key), (phase, pos), &mut acc),
                _ => {}
            }
        }
        if st.follow == FollowCursor::Always {
            st.scroll.jump_end();
        }
        acc.finish(self.id)
    }

    fn handle_text_pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: LineSet<'_>,
        geometry: (Rect, LayoutKey),
        pointer: (Phase, Position),
        acc: &mut Acc<ViewportAction>,
    ) {
        let (area, key) = geometry;
        let (phase, pos) = pointer;
        match phase {
            Phase::Press | Phase::DragStart => {
                cx.focus(self.id);
                let _ = cx.capture(self.id, PartRef::of(Part::TEXT));
                let had = st.selection.take().is_some();
                st.anchor = self.pos_at(cx, lines, (area, key, st.scroll.offset()), pos);
                if had {
                    acc.action(ViewportAction::SelectionChanged);
                } else {
                    acc.consumed();
                }
            }
            Phase::Drag => {
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
                let clamped = Position::new(
                    pos.x
                        .clamp(area.x, area.right().saturating_sub(1).max(area.x)),
                    pos.y
                        .clamp(area.y, area.bottom().saturating_sub(1).max(area.y)),
                );
                let at = self.pos_at(cx, lines, (area, key, st.scroll.offset()), clamped);
                match (st.anchor, at) {
                    (Some(anchor), Some(head)) => {
                        st.selection = Some((anchor, head));
                        acc.action(ViewportAction::SelectionChanged);
                    }
                    _ => acc.consumed(),
                }
            }
            Phase::Click if !self.click_selects_word => {
                cx.focus(self.id);
                acc.consumed();
            }
            Phase::Click | Phase::DoubleClick => {
                cx.focus(self.id);
                let at = self.pos_at(cx, lines, (area, key, st.scroll.offset()), pos);
                match at.and_then(|p| {
                    lines
                        .get(p.line)
                        .and_then(|line| word_at(line, p.col))
                        .map(|(start, end)| (p.line, start, end))
                }) {
                    Some((line, start, end)) => {
                        st.select(CellPos::new(line, start), CellPos::new(line, end));
                        acc.action(ViewportAction::SelectionChanged);
                    }
                    None => acc.consumed(),
                }
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
        let text_w = Self::text_width(area.width);
        let key = self.layout_key(st, text_w);
        let (total, indexed) = {
            let lay = ui.cache::<ViewportLayout>(id);
            let indexed = lay.ensure(lines, key);
            (lay.total(), indexed)
        };
        #[cfg(feature = "testing")]
        self.note_indexed(indexed);
        #[cfg(not(feature = "testing"))]
        Self::note_indexed(indexed);
        let mut view = st.scroll;
        view.apply_layout(usize::from(area.height), total);
        if st.follow == FollowCursor::Always {
            view.jump_end();
        }
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
        let container = ov.style(
            ui,
            id,
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        self.gutter(ui, area, live, &ov);
        let body = Rect {
            x: area.x.saturating_add(1),
            y: area.y,
            width: area.width.saturating_sub(1),
            height: area.height,
        };
        let content = self.bar().draw(ui, body, &view, total);
        let text = Rect {
            width: text_w.min(content.width),
            ..content
        };
        ui.report_layout(
            id,
            LayoutFacts::new(usize::from(area.height), total, area.height, text_w),
        );
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
            (first_line, first_row),
            st.selection(),
            (base.style, selected.style),
        );
        self.caret(
            ui,
            text,
            st,
            lines,
            (text_w, view.offset(), caret_line_start),
            live,
        );
        text
    }

    fn rows(
        &self,
        ui: &mut Ui<'_>,
        text: Rect,
        lines: LineSet<'_>,
        from: (usize, usize),
        sel: Option<(CellPos, CellPos)>,
        styles: (PaintStyle, PaintStyle),
    ) {
        let (mut li, mut row) = from;
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
            paint_row(
                ui,
                y,
                line,
                li,
                row_cols(line, text_w, self.wrap, row),
                sel,
                styles,
            );
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
        view: (u16, usize, Option<usize>),
        live: StateFlags,
    ) {
        let (width, offset, line_start) = view;
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
        let dx = c.col.saturating_sub(start);
        if dy >= usize::from(text.height) || dx >= usize::from(text.width) {
            return;
        }
        ui.set_cursor(
            self.id,
            Position::new(
                text.x.saturating_add(dx as u16),
                text.y.saturating_add(dy as u16),
            ),
        );
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (3, 1),
            preferred: (Self::PREFERRED_TEXT_WIDTH.saturating_add(2), c.max.1),
        }
        .fit(c)
    }
}

#[derive(Clone, Copy)]
struct Run {
    span: usize,
    sel: bool,
    a: usize,
    b: usize,
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
        let text = run_text(line, r.span).get(r.a..r.b).unwrap_or("");
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
        } else {
            let end = c.at.saturating_add(c.len);
            match run {
                Some(r) if r.span == c.span && r.sel == is_sel && r.b == c.at => {
                    run = Some(Run { b: end, ..r });
                }
                other => {
                    flush(ui, other, &mut x);
                    run = Some(Run {
                        span: c.span,
                        sel: is_sel,
                        a: c.at,
                        b: end,
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
    use crate::event::MouseKind;
    use crate::runtime::stub::{SCREEN, Stub, key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::{ColorLevel, Surface, Theme};
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("viewport.tests");
    const POINTER_AREA: Rect = Rect::new(0, 0, 12, 8);
    const MANY_LINES: [ViewportLine<'static>; 100] = [ViewportLine::Plain("row"); 100];

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
        let render = |caret: CellPos| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
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
        assert_eq!(render(CellPos::new(0, 0)), None, "caret above viewport");
        assert_eq!(render(CellPos::new(20, 0)), None, "caret below viewport");
        assert_eq!(render(CellPos::new(6, 1)), Some(Position::new(2, 1)));
    }

    #[test]
    fn append_extends_the_wrapped_index_without_rebuilding_its_prefix() {
        let key = LayoutKey {
            width: 4,
            wrap: true,
            generation: 0,
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
                out.push_str(cell_text(line, c));
            }
            col = col.saturating_add(usize::from(c.w));
            ControlFlow::Continue(())
        });
        out
    }

    #[test]
    fn the_text_width_does_not_depend_on_whether_the_bar_is_painted() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 24,
            height: 4,
        };
        let short = [ViewportLine::Plain("one"), ViewportLine::Plain("two")];
        let long: Vec<ViewportLine<'_>> = (0..50).map(|_| ViewportLine::Plain("row")).collect();
        let width_of = |lines: &[ViewportLine<'_>]| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ViewportState::default();
            let mut w = 0;
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                w = TextViewport::new(ID).draw(ui, area, &st, lines).width;
            })
            .commit_presented();
            w
        };
        assert_eq!(width_of(&short), width_of(&long));
        assert_eq!(width_of(&short), TextViewport::text_width(area.width));
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
