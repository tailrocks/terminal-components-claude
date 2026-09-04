//! `TextViewport` — the selectable read-only text pane
//! (`COMPONENT_ARCHITECTURE.md` §12.4, §14.1, §18.2 `viewport`, §20.9-7,
//! Appendix A 4E).
//!
//! The legacy widget owned its lines, owned a `Vec<Vec<Cell>>` of grapheme
//! `String`s, and laid the whole buffer out **twice** per frame — once at
//! `width`, once at `width − 1` for the scrollbar — which is why
//! `viewport_100k_lines_render` recorded 15.2 M allocations per frame
//! (§20.10 item 7, finding P-A). Here the caller owns the lines, cells are
//! produced by a walk instead of being stored, and the scrollbar column is
//! reserved **before** the layout rather than discovered after it, so there
//! is exactly one pass and no per-frame allocation at all.

use core::fmt;
use core::ops::ControlFlow;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Modifier, Style};

use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::text::Span;
use crate::text::measure::{grapheme_width, graphemes};
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

/// Columns a tab expands to. A tab is a control character, so
/// [`crate::text::width`] gives it zero columns; a terminal pane needs it to
/// occupy something, and four is the width the legacy viewport used.
const TAB_WIDTH: u16 = 4;

/// One source line of a [`TextViewport`].
///
/// `Plain` exists so a log of `&str` costs no span storage; `Spans` carries
/// the role-bearing runs a terminal or a highlighter produces. Both borrow;
/// neither is ever cloned by the viewport.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewportLine<'a> {
    /// A single run inheriting `Part::TEXT`.
    Plain(&'a str),
    /// Role-carrying runs.
    Spans(&'a [Span<'a>]),
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

/// A logical position in the text: a line index and a **display column**.
///
/// Columns, not byte offsets and not grapheme indices, because a display
/// column is what a pointer position maps to and what a selection compares.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct CellPos {
    /// The line index.
    pub line: usize,
    /// The display column within the line.
    pub col: usize,
}

impl CellPos {
    /// A position.
    pub const fn new(line: usize, col: usize) -> Self {
        CellPos { line, col }
    }
}

/// What a viewport reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewportAction {
    /// The offset moved.
    Scrolled,
    /// The selection changed or was extended.
    SelectionChanged,
    /// The selection was dropped.
    SelectionCleared,
    /// Tail-follow was turned on or off.
    FollowChanged(bool),
    /// The reader asked for the selection. The text is **not** carried:
    /// building it allocates and `update` must not, so the owner calls
    /// [`ViewportState::copy_into`] with a buffer it already has.
    CopyRequested,
}

/// The const-constructible commands of the viewport keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewportCmd {
    /// One row up.
    Up,
    /// One row down.
    Down,
    /// One viewport up.
    PageUp,
    /// One viewport down.
    PageDown,
    /// To the first row.
    Home,
    /// To the last row, and follow the tail.
    End,
    /// Toggle tail-follow.
    ToggleFollow,
    /// Ask the owner to copy the selection.
    Copy,
}

const fn b(
    chord: Chord,
    cmd: ViewportCmd,
    label: &'static str,
    visible: bool,
) -> Binding<ViewportCmd> {
    Binding {
        chord,
        cmd,
        label,
        priority: if visible { 50 } else { 10 },
        visible,
    }
}

const BINDINGS: &[Binding<ViewportCmd>] = &[
    b(Chord::key(KeyCode::Up), ViewportCmd::Up, "Up", true),
    b(Chord::key(KeyCode::Down), ViewportCmd::Down, "Down", true),
    b(Chord::key(KeyCode::Char('k')), ViewportCmd::Up, "Up", false),
    b(
        Chord::key(KeyCode::Char('j')),
        ViewportCmd::Down,
        "Down",
        false,
    ),
    b(
        Chord::key(KeyCode::PageUp),
        ViewportCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        ViewportCmd::PageDown,
        "Page down",
        false,
    ),
    b(Chord::key(KeyCode::Home), ViewportCmd::Home, "Top", false),
    b(
        Chord::key(KeyCode::Char('g')),
        ViewportCmd::Home,
        "Top",
        false,
    ),
    b(Chord::key(KeyCode::End), ViewportCmd::End, "Tail", true),
    b(
        Chord::key(KeyCode::Char('G')),
        ViewportCmd::End,
        "Tail",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('f')),
        ViewportCmd::ToggleFollow,
        "Follow",
        true,
    ),
    b(
        Chord::key(KeyCode::Char('y')),
        ViewportCmd::Copy,
        "Copy",
        true,
    ),
];

// ───────────────────────────── the cell walk ─────────────────────────────

/// One display cell: which run it came from, its byte range inside that
/// run's text, its column width, and whether it is a tab expansion (whose
/// bytes are a tab, not the space it paints).
#[derive(Clone, Copy, Debug)]
struct VCell {
    span: usize,
    at: usize,
    len: usize,
    w: u16,
    tab: bool,
}

/// The runs of a line; empty for `Plain`, whose single run is the line.
const fn runs(line: ViewportLine<'_>) -> &[Span<'_>] {
    match line {
        ViewportLine::Plain(_) => &[],
        ViewportLine::Spans(s) => s,
    }
}

/// The text of run `ix`.
fn run_text(line: ViewportLine<'_>, ix: usize) -> &str {
    match line {
        ViewportLine::Plain(s) => s,
        ViewportLine::Spans(sp) => sp.get(ix).map_or("", |s| s.text),
    }
}

/// The role and modifiers of run `ix`.
fn run_style(line: ViewportLine<'_>, ix: usize) -> (Option<Role>, Modifier) {
    runs(line)
        .get(ix)
        .map_or((None, Modifier::empty()), |s| (s.role, s.add))
}

/// The text one display cell paints.
fn cell_text(line: ViewportLine<'_>, c: VCell) -> &str {
    if c.tab {
        " "
    } else {
        run_text(line, c.span)
            .get(c.at..c.at.saturating_add(c.len))
            .unwrap_or("")
    }
}

/// Walk one run's display cells.
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

/// Walk every display cell of `line`, in order.
fn walk_line(line: ViewportLine<'_>, mut f: impl FnMut(VCell) -> ControlFlow<()>) {
    match line {
        ViewportLine::Plain(s) => {
            let _ = walk_run(0, s, &mut f);
        }
        ViewportLine::Spans(sp) => {
            for (i, s) in sp.iter().enumerate() {
                if walk_run(i, s.text, &mut f).is_break() {
                    return;
                }
            }
        }
    }
}

/// Total display columns of `line`.
fn line_cols(line: ViewportLine<'_>) -> usize {
    let mut n = 0usize;
    walk_line(line, |c| {
        n = n.saturating_add(usize::from(c.w));
        ControlFlow::Continue(())
    });
    n
}

/// Visual rows `line` occupies at text width `w`.
fn line_rows(line: ViewportLine<'_>, w: u16, wrap: bool) -> usize {
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

/// The `[start, end)` display columns of visual row `row` of `line`.
/// `usize::MAX` as the end means "to the end of the line".
fn row_cols(line: ViewportLine<'_>, w: u16, wrap: bool, row: usize) -> (usize, usize) {
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

// ─────────────────────── the windowed visual-row index ───────────────────────

/// The key a cached layout is valid for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct LayoutKey {
    lines: usize,
    width: u16,
    wrap: bool,
    generation: u32,
}

/// The visual-row prefix sum, held in [`Ui::cache`] because it is a function
/// of `area` and therefore knowable only in `draw` (R8, §20.9-7). It is never
/// in [`ViewportState`]: it is derived, and `draw_twice_leaves_state_equal`
/// must not see it.
#[derive(Debug, Default)]
struct ViewportLayout {
    key: LayoutKey,
    /// `prefix[i]` is the number of visual rows before line `i`; the last
    /// entry is the total. Empty when `wrap` is off, where the mapping is the
    /// identity.
    prefix: Vec<u32>,
}

impl ViewportLayout {
    /// Rebuild for `key` if it changed. Reuses the vector's capacity, so a
    /// steady stream of pushes allocates nothing after the first growth.
    fn ensure(&mut self, lines: &[ViewportLine<'_>], key: LayoutKey) {
        if self.key == key && (!key.wrap || self.prefix.len() == lines.len().saturating_add(1)) {
            return;
        }
        self.key = key;
        self.prefix.clear();
        if !key.wrap {
            return;
        }
        self.prefix.push(0);
        let mut acc = 0u32;
        for l in lines {
            let r = line_rows(*l, key.width, true);
            acc = acc.saturating_add(u32::try_from(r).unwrap_or(u32::MAX));
            self.prefix.push(acc);
        }
    }

    /// Total visual rows.
    fn total(&self, lines: &[ViewportLine<'_>]) -> usize {
        if !self.key.wrap {
            return lines.len();
        }
        self.prefix.last().map_or(0, |n| *n as usize)
    }

    /// The `(line, row-within-line)` a visual `row` falls in.
    fn row_start(&self, lines: &[ViewportLine<'_>], row: usize) -> (usize, usize) {
        if !self.key.wrap {
            return (row.min(lines.len().saturating_sub(1)), 0);
        }
        let prefix = self.prefix.as_slice();
        if prefix.len() < 2 {
            return (0, 0);
        }
        let target = u32::try_from(row).unwrap_or(u32::MAX);
        let i = match prefix.binary_search(&target) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let i = i.min(prefix.len().saturating_sub(2));
        let base = prefix.get(i).copied().unwrap_or(0);
        (i, target.saturating_sub(base) as usize)
    }
}

/// The uncached row mapping `update` uses: the identity without wrap, a
/// forward walk with it.
///
/// Recorded deliberately: with `wrap` on this is `O(offset)` in lines, and it
/// is on the **pointer** path only — never on the paint path, which reads the
/// cached prefix. It allocates nothing.
fn walk_to_row(lines: &[ViewportLine<'_>], width: u16, wrap: bool, row: usize) -> (usize, usize) {
    if !wrap {
        return (row.min(lines.len().saturating_sub(1)), 0);
    }
    let mut acc = 0usize;
    for (i, l) in lines.iter().enumerate() {
        let r = line_rows(*l, width, true);
        if row < acc.saturating_add(r) {
            return (i, row.saturating_sub(acc));
        }
        acc = acc.saturating_add(r);
    }
    (lines.len().saturating_sub(1), 0)
}

/// Total visual rows without a cache.
fn total_walk(lines: &[ViewportLine<'_>], width: u16, wrap: bool) -> usize {
    if !wrap {
        return lines.len();
    }
    lines
        .iter()
        .fold(0usize, |a, l| a.saturating_add(line_rows(*l, width, true)))
}

/// Word characters for double-click selection: [`crate::text`]'s definition
/// widened by the four separators a path or an identifier in log output
/// carries, so a double-click on `src/main.rs` selects the whole path.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.'
}

/// The `[start, end)` columns of the word run containing `col`, if any.
fn word_at(line: ViewportLine<'_>, col: usize) -> Option<(usize, usize)> {
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

// ───────────────────────────── the state ─────────────────────────────

/// Durable state of a [`TextViewport`]: offset, tail-follow, selection, drag
/// anchor and the optional caret.
///
/// It holds no text and no layout — the lines are the caller's and the
/// visual-row index is derived (§20.9-7).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewportState {
    scroll: ScrollState,
    follow: bool,
    selection: Option<(CellPos, CellPos)>,
    anchor: Option<CellPos>,
    caret: Option<CellPos>,
    generation: u32,
}

impl Default for ViewportState {
    fn default() -> Self {
        ViewportState {
            scroll: ScrollState::default(),
            follow: true,
            selection: None,
            anchor: None,
            caret: None,
            generation: 0,
        }
    }
}

impl ViewportState {
    /// The scroll state.
    pub const fn scroll(&self) -> &ScrollState {
        &self.scroll
    }

    /// Whether the view sticks to the tail.
    pub const fn follow(&self) -> bool {
        self.follow
    }

    /// Turn tail-follow on or off. Turning it on jumps to the tail on the
    /// next `update`.
    pub const fn set_follow(&mut self, on: bool) {
        self.follow = on;
    }

    /// The normalised selection, or `None` when it is empty.
    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        let (a, b) = self.selection?;
        if a == b {
            return None;
        }
        Some((a.min(b), a.max(b)))
    }

    /// Select `a..b`.
    pub const fn select(&mut self, a: CellPos, b: CellPos) {
        self.selection = Some((a, b));
        self.anchor = None;
    }

    /// Drop the selection; `true` when there was one.
    pub const fn clear_selection(&mut self) -> bool {
        let had = self.selection.is_some();
        self.selection = None;
        self.anchor = None;
        had
    }

    /// The caret, exposed as the hardware cursor while the viewport is
    /// focused and the caret's row is on screen.
    pub const fn caret(&self) -> Option<CellPos> {
        self.caret
    }

    /// Move or hide the caret.
    pub const fn set_caret(&mut self, c: Option<CellPos>) {
        self.caret = c;
    }

    /// Tell the viewport that `dropped` lines were removed from the **front**
    /// of the buffer, so every position it holds moves up by that much.
    ///
    /// This is the whole of bounded retention that belongs to the component:
    /// the buffer is the caller's and only the caller knows its cap, but the
    /// selection, the drag anchor and the caret are the viewport's and would
    /// otherwise silently name the wrong lines. The offset is deliberately
    /// **not** moved — it is clamped against the new length by the next
    /// `update`, which is what the legacy widget did.
    pub fn retained(&mut self, dropped: usize) {
        if dropped == 0 {
            return;
        }
        if self
            .selection
            .is_some_and(|(a, b)| a.line < dropped && b.line < dropped)
        {
            // both ends named lines that no longer exist: the selection is
            // gone, not "the first two columns of whatever is at the head
            // now". Saturating each end independently is what the legacy
            // widget did, and it left a live selection over unrelated text.
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

    /// The lines changed in place without changing their count. Discards the
    /// derived visual-row index on the next draw.
    pub const fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Append the selected text to `out`; `false` when there is no selection.
    ///
    /// Lines are joined with `\n` and each is right-trimmed, which is what a
    /// terminal selection puts on a clipboard.
    pub fn copy_into(&self, lines: &[ViewportLine<'_>], out: &mut String) -> bool {
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
            walk_line(*line, |c| {
                if col >= to {
                    return ControlFlow::Break(());
                }
                if col >= from {
                    out.push_str(cell_text(*line, c));
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

/// Move a position up by `dropped` lines. A position whose own line was
/// dropped lands at the head of the retained buffer — column `0`, not its old
/// column, which would name an unrelated character on a different line.
fn shift(p: &mut CellPos, dropped: usize) {
    if p.line < dropped {
        *p = CellPos::new(0, 0);
    } else {
        p.line = p.line.saturating_sub(dropped);
    }
}

// ───────────────────────────── the component ─────────────────────────────

/// A scrollable, selectable, read-only pane over borrowed styled lines.
///
/// ## Construction
/// `TextViewport::new(id)`; the lines are passed to each phase, never held
/// (§13, A3).
///
/// ## Ownership
/// The caller owns the lines (`&[ViewportLine]` per phase) and a
/// [`ViewportState`] (offset, follow, selection, anchor, caret). The runtime
/// owns focus, hover, press, wheel routing, the scrollbar capture and the
/// selection-drag capture. The visual-row index is neither's: it is derived
/// into [`Ui::cache`].
///
/// ## Configuration
/// `.wrap(bool)` (`false`), `.patch`, `.patch_part`, `.slot`,
/// `.state_override`.
///
/// ## Variants
/// `Family::VIEWPORT`, `Variant::DEFAULT` only.
///
/// ## States
/// `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` and `PRESSED` from the runtime; a
/// live selection drag keeps `PRESSED`. Nothing is props-derived: the
/// viewport takes no readiness prop, so it owes no §11.4 `Part::ICON`
/// affordance and declares none.
///
/// ## Actions
/// `Scrolled`, `SelectionChanged`, `SelectionCleared`, `FollowChanged(bool)`,
/// `CopyRequested`. None carries an `ItemKey`: a viewport has no items.
/// `CopyRequested` deliberately carries no text — see [`ViewportAction`].
///
/// ## Focus
/// One `Focusable` stop over the whole area; does not swallow typing, so an
/// application chord still reaches `KeyPhase::Capture`. No scope, no trap,
/// no `autofocus`.
///
/// ## Keyboard
/// `↑`/`k`, `↓`/`j`, `PgUp`, `PgDn`, `Home`/`g`, `End`/`G` (tail, which also
/// turns follow on), `f` toggles follow, `y` requests a copy. `Esc` is
/// **not** bound: it belongs to the §3.3 dismissal ladder, and a read-only
/// pane that swallowed it would strand a reader inside a dialog. A press
/// clears the selection.
///
/// ## Mouse
/// `PartRef::of(Part::TEXT)`: a press drops any selection and anchors a
/// drag, claiming pointer capture; drags extend the selection and
/// auto-scroll one row past either vertical edge; a double-click selects the
/// word under the pointer. `TRACK`/`THUMB` and the wheel go to the embedded
/// [`ScrollRegion`].
///
/// ## Layout
/// One focus-gutter column at the left, then the text, then the scrollbar
/// column. **The scrollbar column is reserved whether or not the bar is
/// painted**, so the wrap layout is a function of `area` alone and is
/// computed once — the legacy two-pass layout is what made
/// `viewport_100k_lines_render` unreachable (P-A). `measure` asks for the
/// gutter, the bar and twenty text columns. `draw` returns the text rect;
/// `0×0` registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the fill), `TEXT` (the runs, resolved a second time with
/// `SELECTED` for selected cells), `GUTTER` (the focus bar), `TRACK` and
/// `THUMB` (the embedded [`ScrollRegion`], composed under this component's
/// own `Id` and therefore this component's parts, §33.2).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach every part, including `Part::TRACK` and
/// `Part::THUMB`, which are forwarded into the nested [`ScrollRegion`]
/// rather than dropped (§45.1). `.slot` is honoured for `Part::GUTTER`,
/// `Part::TRACK` and `Part::THUMB`. `Part::CONTAINER` and `Part::TEXT` are
/// **not** slot-addressable: the container is the plane the text is painted
/// against, and the text is the component's whole subject.
///
/// ## Identity
/// One `Id`; no items and no `ItemKey`. Positions are [`CellPos`], which is
/// a line index and a display column and is invalidated by retention — call
/// [`ViewportState::retained`].
///
/// ## Testing
/// `TextViewportCase` with `Caps::FOCUSABLE | Caps::SCROLLS |
/// Caps::CAPTURES | Caps::CURSOR`; `render::components::text_viewport::*`.
///
/// ## Invariants
/// A frame allocates nothing: cells are walked, never stored, and the
/// visual-row prefix is rebuilt into a cache vector that keeps its capacity.
/// The layout is computed once per frame at one width. Only the visible rows
/// are walked for painting. `follow` is applied in `update`, so `draw` and
/// the next `update` agree on the offset.
pub struct TextViewport<'a> {
    id: Id,
    wrap: bool,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
    forced: Option<StateFlags>,
}

impl fmt::Debug for TextViewport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextViewport")
            .field("id", &self.id)
            .field("wrap", &self.wrap)
            .field("slot", &self.slot.map(|(p, _)| p))
            .field("forced", &self.forced)
            .finish_non_exhaustive()
    }
}

impl<'a> TextViewport<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TEXT,
        Part::GUTTER,
        Part::TRACK,
        Part::THUMB,
    ];

    /// The width `measure` prefers for the text itself.
    pub const PREFERRED_TEXT_WIDTH: u16 = 20;

    /// A viewport.
    pub const fn new(id: Id) -> Self {
        TextViewport {
            id,
            wrap: false,
            patch: None,
            parts: &[],
            slot: None,
            forced: None,
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Wrap long lines instead of clipping them.
    #[must_use]
    pub const fn wrap(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.slot = Some((p, f));
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.forced = Some(s);
        self
    }

    /// The instance overrides, assembled from the builders.
    ///
    /// They are stored as their arguments rather than as one `Overrides`
    /// because the nested [`ScrollRegion`] takes them through its **own**
    /// builders, and §45.1 found four components constructing a nested
    /// component bare and silently dropping the caller's `.patch_part` and
    /// `.slot`.
    fn overrides(&self) -> Overrides<'a> {
        let mut ov = Overrides::new().patch_part(self.parts);
        if let Some(p) = self.patch {
            ov = ov.patch(p);
        }
        if let Some((p, f)) = self.slot {
            ov = ov.slot(p, f);
        }
        ov.inherit_forced(self.forced)
    }

    /// The embedded scroll region, carrying this instance's `.patch`,
    /// `.patch_part` and `.slot`.
    ///
    /// A forced state (A11) is **not** forwarded, and that is a limitation
    /// rather than a decision: [`ScrollRegion`] exposes `state_override` and
    /// not §12.1's `inherit_forced`, and it registers its regions
    /// unconditionally, so forwarding would change the bar's colours in a
    /// reference rendering without making the rendering inert. The bar of a
    /// forced viewport therefore paints in its live state.
    fn bar(&self) -> ScrollRegion<'a> {
        let mut sr = ScrollRegion::new(self.id).patch_part(self.parts);
        if let Some(p) = self.patch {
            sr = sr.patch(p);
        }
        if let Some((p, f)) = self.slot {
            sr = sr.slot(p, f);
        }
        sr
    }

    /// The text width for a container `width` columns wide: one gutter
    /// column and one scrollbar column, both always reserved.
    const fn text_width(width: u16) -> u16 {
        width.saturating_sub(2)
    }

    /// Map a screen position to a logical position, using last frame's
    /// geometry.
    fn pos_at(
        &self,
        lines: &[ViewportLine<'_>],
        geometry: (Rect, u16, usize),
        pos: Position,
    ) -> Option<CellPos> {
        let (area, width, offset) = geometry;
        if area.is_empty() || lines.is_empty() {
            return None;
        }
        let dy = usize::from(pos.y.saturating_sub(area.y));
        let (li, row) = walk_to_row(lines, width, self.wrap, offset.saturating_add(dy));
        let line = *lines.get(li)?;
        let (start, end) = row_cols(line, width, self.wrap, row);
        let dx = usize::from(pos.x.saturating_sub(area.x.saturating_add(1)));
        let last = end.min(line_cols(line));
        Some(CellPos {
            line: li,
            col: start.saturating_add(dx).min(last),
        })
    }

    /// The update phase: follow, the scroll region, the keymap and the
    /// selection drag.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ViewportState,
        lines: &[ViewportLine<'_>],
    ) -> Response<ViewportAction> {
        let area = cx.area(self.id).unwrap_or(Rect::ZERO);
        let width = cx
            .layout(self.id)
            .map_or_else(|| Self::text_width(area.width), |l| l.cols);
        let total = total_walk(lines, width, self.wrap);
        let before = st.scroll.offset();
        let bar = self.bar().update(cx, &mut st.scroll, total);
        let mut acc = Acc::<ViewportAction>::new();
        acc.fold(&bar);
        if st.scroll.offset() != before {
            st.follow = st.scroll.at_end();
        }
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => {
                    if let Some(cmd) = Binding::lookup(BINDINGS, &k) {
                        Self::command(st, cmd, &mut acc);
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
                } => match phase {
                    Phase::Press | Phase::DragStart => {
                        let _ = cx.capture(self.id, PartRef::of(Part::TEXT));
                        let had = st.selection.take().is_some();
                        st.anchor = self.pos_at(lines, (area, width, st.scroll.offset()), pos);
                        if had {
                            acc.action(ViewportAction::SelectionCleared);
                        } else {
                            acc.consumed();
                        }
                    }
                    Phase::Drag => {
                        // one row of auto-scroll past either vertical edge,
                        // so a selection can reach off-screen text
                        if pos.y < area.y {
                            st.scroll.scroll_by(-1);
                            st.follow = false;
                        } else if pos.y >= area.bottom() {
                            st.scroll.scroll_by(1);
                            st.follow = st.scroll.at_end();
                        }
                        let clamped = Position::new(
                            pos.x
                                .clamp(area.x, area.right().saturating_sub(1).max(area.x)),
                            pos.y
                                .clamp(area.y, area.bottom().saturating_sub(1).max(area.y)),
                        );
                        let at = self.pos_at(lines, (area, width, st.scroll.offset()), clamped);
                        match (st.anchor, at) {
                            (Some(anchor), Some(head)) => {
                                st.selection = Some((anchor, head));
                                acc.action(ViewportAction::SelectionChanged);
                            }
                            _ => acc.consumed(),
                        }
                    }
                    Phase::DoubleClick => {
                        let at = self.pos_at(lines, (area, width, st.scroll.offset()), pos);
                        match at.and_then(|p| {
                            lines
                                .get(p.line)
                                .and_then(|l| word_at(*l, p.col))
                                .map(|(s, e)| (p.line, s, e))
                        }) {
                            Some((line, s, e)) => {
                                st.select(CellPos::new(line, s), CellPos::new(line, e));
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
                    Phase::Click | Phase::Secondary => acc.consumed(),
                },
                _ => {}
            }
        }
        if st.follow {
            st.scroll.jump_end();
        }
        acc.finish(self.id)
    }

    /// One keymap command. Every command consumes, so the table and the
    /// handler cannot drift (§16.2 case 20).
    fn command(st: &mut ViewportState, cmd: ViewportCmd, acc: &mut Acc<ViewportAction>) {
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
                st.follow = true;
                if was {
                    acc.changed();
                } else {
                    acc.action(ViewportAction::FollowChanged(true));
                }
                return;
            }
            ViewportCmd::ToggleFollow => {
                st.follow = !st.follow;
                if st.follow {
                    st.scroll.jump_end();
                }
                acc.action(ViewportAction::FollowChanged(st.follow));
                return;
            }
            ViewportCmd::Copy => {
                acc.action(ViewportAction::CopyRequested);
                return;
            }
        }
        if st.scroll.offset() == before {
            acc.consumed();
        } else {
            st.follow = st.scroll.at_end();
            acc.action(ViewportAction::Scrolled);
        }
    }

    /// The draw phase.
    pub fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let ov = self.overrides();
        let id = self.id;
        let forced = ov.is_forced();
        let live = ov.flags(ui.state(id), StateFlags::empty());
        let text_w = Self::text_width(area.width);
        let key = LayoutKey {
            lines: lines.len(),
            width: text_w,
            wrap: self.wrap,
            generation: st.generation,
        };
        let total = {
            let lay = ui.cache::<ViewportLayout>(id);
            lay.ensure(lines, key);
            lay.total(lines)
        };
        let mut view = st.scroll;
        view.apply_layout(usize::from(area.height), total);
        if st.follow {
            view.jump_end();
        }
        let (first_line, first_row) = {
            let lay = ui.cache::<ViewportLayout>(id);
            lay.row_start(lines, view.offset())
        };
        if !forced {
            ui.register_control(id, area, Focusability::Focusable);
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
        if !forced {
            ui.register_part(id, PartRef::of(Part::TEXT), text);
        }
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
        self.caret(ui, text, st, lines, (text_w, view.offset()), live);
        text
    }

    /// Paint the visible rows.
    fn rows(
        &self,
        ui: &mut Ui<'_>,
        text: Rect,
        lines: &[ViewportLine<'_>],
        from: (usize, usize),
        sel: Option<(CellPos, CellPos)>,
        styles: (Style, Style),
    ) {
        let (mut li, mut row) = from;
        let text_w = text.width;
        let mut rows_here = lines
            .get(li)
            .map_or(0, |l| line_rows(*l, text_w, self.wrap));
        for y in text.rows() {
            let Some(line) = lines.get(li) else { break };
            paint_row(
                ui,
                y,
                *line,
                li,
                row_cols(*line, text_w, self.wrap, row),
                sel,
                styles,
            );
            row = row.saturating_add(1);
            if row >= rows_here {
                row = 0;
                li = li.saturating_add(1);
                rows_here = lines
                    .get(li)
                    .map_or(0, |l| line_rows(*l, text_w, self.wrap));
            }
        }
    }

    /// The focus gutter column.
    fn gutter(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags, ov: &Overrides<'a>) {
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

    /// Request the hardware cursor for the caret when it is on screen.
    fn caret(
        &self,
        ui: &mut Ui<'_>,
        text: Rect,
        st: &ViewportState,
        lines: &[ViewportLine<'_>],
        view: (u16, usize),
        live: StateFlags,
    ) {
        let (width, offset) = view;
        let Some(c) = st.caret else { return };
        if !live.contains(StateFlags::FOCUSED) {
            return;
        }
        let Some(at) = lines.get(c.line) else {
            return;
        };
        let mut before = 0usize;
        if self.wrap {
            for l in lines.get(..c.line).unwrap_or(&[]) {
                before = before.saturating_add(line_rows(*l, width, true));
            }
        } else {
            before = c.line;
        }
        let mut within = 0usize;
        if self.wrap {
            for i in 0..line_rows(*at, width, true) {
                let (a, b) = row_cols(*at, width, true, i);
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
        let (start, _) = row_cols(*at, width, self.wrap, within);
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

    /// The natural size: the gutter, the bar and twenty text columns.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (3, 1),
            preferred: (Self::PREFERRED_TEXT_WIDTH.saturating_add(2), c.max.1),
        }
        .fit(c)
    }
}

/// A run being accumulated for one `paint_spans` call.
#[derive(Clone, Copy)]
struct Run {
    span: usize,
    sel: bool,
    a: usize,
    b: usize,
}

/// One visual row: the run slices of `line` between two display columns,
/// split wherever the run or the selection changes.
fn paint_row(
    ui: &mut Ui<'_>,
    rect: Rect,
    line: ViewportLine<'_>,
    line_ix: usize,
    cols: (usize, usize),
    sel: Option<(CellPos, CellPos)>,
    styles: (Style, Style),
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
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const ID: Id = Id::root("viewport.tests");

    /// §16.1's named test. Bounded retention is the caller's — it owns the
    /// buffer and its cap — but every position the viewport holds is an index
    /// into that buffer, so dropping `n` lines from the front silently
    /// re-points the selection, the drag anchor and the caret at the wrong
    /// lines. `retained(n)` is the whole of the fix-up, and it must move all
    /// three, saturating at line 0 for positions that were dropped.
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

        // only the head survived: the dropped end moves to the head of the
        // buffer at column 0, never to its old column on a different line
        let mut partial = ViewportState::default();
        partial.select(CellPos::new(1, 3), CellPos::new(4, 6));
        partial.retained(3);
        assert_eq!(
            partial.selection(),
            Some((CellPos::new(0, 0), CellPos::new(1, 6)))
        );

        // a position whose line was itself dropped saturates at the head
        st.retained(10);
        assert_eq!(
            st.selection(),
            None,
            "a fully dropped selection must collapse, not name line 0 twice"
        );
        assert_eq!(st.caret(), Some(CellPos::new(0, 0)));
        // the derived visual-row index is invalidated by the same call
        let before = st.generation;
        st.retained(1);
        assert_ne!(st.generation, before);
        // and `retained(0)` is a no-op, so an unbounded buffer costs nothing
        let g = st.generation;
        st.retained(0);
        assert_eq!(st.generation, g);
    }

    /// The wrap layout is used by three separate readers — the paint loop,
    /// the pointer mapping and the caret — so the visual rows of one line
    /// must partition its columns: contiguous, no gap, no overlap, and the
    /// last row open-ended. A row that started one column late silently
    /// dropped a character from the middle of a wrapped paragraph.
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

    /// A selection spanning lines is joined with `\n` and each line is
    /// right-trimmed, which is what a terminal puts on a clipboard. It also
    /// pins the two ends: the first line starts at the anchor column, the
    /// last stops at the head column.
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
        // no selection writes nothing and says so
        let mut st2 = ViewportState::default();
        let mut out2 = String::new();
        assert!(!st2.copy_into(&lines, &mut out2));
        assert!(out2.is_empty());
        assert!(!st2.clear_selection());
    }

    /// Double-click selects a *word*, and in log output a word includes the
    /// path separators — selecting `src` out of `src/main.rs` is useless to
    /// the reader who wanted to paste the path.
    #[test]
    fn a_double_click_selects_a_path_like_word() {
        let line = ViewportLine::Plain("see src/main.rs:42 for the rest");
        assert_eq!(word_at(line, 5), Some((4, 15)));
        assert_eq!(
            "src/main.rs",
            line_slice(line, 4, 15),
            "the word bounds do not name the path"
        );
        // the separator itself belongs to no word
        assert_eq!(word_at(line, 3), None);
        // past the end of the line there is no word
        assert_eq!(word_at(line, 999), None);
    }

    /// Cut the columns `[a, b)` out of a line, for assertions.
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

    /// Finding P-A: the legacy viewport laid the whole buffer out at `width`,
    /// discovered it overflowed, and laid it out **again** at `width − 1` for
    /// the scrollbar. Reserving the column unconditionally makes the layout a
    /// function of `area` alone, so the text rect — and therefore the wrap —
    /// is the same whether or not the bar is painted.
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
            });
            w
        };
        assert_eq!(width_of(&short), width_of(&long));
        assert_eq!(width_of(&short), TextViewport::text_width(area.width));
    }

    /// The viewport is a focus stop and paints a focus gutter, because the
    /// `VIEWPORT` recipe gives `CONTAINER` and `TEXT` no `FOCUSED` rule at
    /// all — without the gutter glyph a focused pane and an unfocused one are
    /// the same cells at `ColorLevel::Mono` and conformance case 9 could
    /// never pass for a `Caps::FOCUSABLE` component.
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
        let gutter = |forced: StateFlags| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            let st = ViewportState::default();
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                TextViewport::new(ID)
                    .state_override(forced)
                    .draw(ui, area, &st, &lines);
            });
            buf.cell(Position::new(0, 0))
                .map_or_else(String::new, |c| c.symbol().to_owned())
        };
        assert_eq!(gutter(StateFlags::FOCUSED), bar);
        assert_ne!(gutter(StateFlags::empty()), bar);
    }

    /// The derived visual-row index lives in [`Ui::cache`], so a frame that
    /// reads it warm and a frame that rebuilds it must paint the same cells —
    /// otherwise a buffer edit would move the text under the reader. The
    /// rebuild is provoked through [`ViewportState::invalidate`], which is the
    /// only public way a caller says "the lines changed in place".
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
        // the first frame is discarded: it is the one that registers the focus
        // stop, so the runtime assigns focus *between* frames 1 and 2 and the
        // gutter legitimately changes. Frames 2 and 3 differ only in that the
        // layout cache is warm.
        let mut frames = Vec::new();
        for f in 0..3 {
            // frame 2 invalidates, so the cache key changes and the prefix is
            // rebuilt rather than reused
            if f == 2 {
                st.invalidate();
            }
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                TextViewport::new(ID).wrap(true).draw(ui, area, &st, &lines);
            });
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

    /// Tabs are control characters, so [`crate::text::width`] gives them zero
    /// columns; a terminal pane would collapse an indented line onto its
    /// first token. They expand to [`TAB_WIDTH`] cells, and the expansion has
    /// to agree between the walk, the copy and the paint or a selection over
    /// an indented line copies the wrong bytes.
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

    /// §33's Invariant P: every declared part is one a drawn viewport
    /// actually resolves — including `TRACK` and `THUMB`, which the embedded
    /// [`ScrollRegion`] resolves under **this** component's `Id` and which are
    /// therefore this component's parts (§33.2). It is also the §45.1 check
    /// that the nested component is not constructed bare: if `bar()` dropped
    /// the caller's `.patch_part`, the last two rounds of this loop would not
    /// move a cell.
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
            });
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

    /// §45's Invariant R: `## Overrides` names `GUTTER`, `TRACK` and `THUMB`
    /// as slot-addressable and `CONTAINER` and `TEXT` as not, and both
    /// directions are asserted. The `TRACK`/`THUMB` half is also §45.1's
    /// nested-component finding: a bare `ScrollRegion` would swallow them.
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
            });
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
