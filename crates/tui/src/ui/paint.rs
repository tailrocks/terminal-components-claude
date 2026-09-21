//! Painting (`COMPONENT_ARCHITECTURE.md` §5 R3, §17.0 A2, §22.2 items 1–2, 16, 18).
//!
//! Every method clips to the current area and marks the layer's
//! written-cell bitset. Cell, string, and span painters share Ratatui
//! grapheme/width semantics in one allocation-free writer that also records
//! provenance;
//! `paint_cell` resets the cells a wide grapheme shadows; `fill` and
//! `dim_layer` are deliberate re-implementations of `ratatui_widgets::{Fill,
//! Dimmed}` because foreign widgets cannot mark the bitset or walk roles.

use ratatui_core::buffer::{Buffer, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use super::Ui;
use crate::text::Span;
use crate::text::clusters::{ClusterFeed, ClusterScratch};
use crate::text::measure::graphemes;
use crate::theme::{FgStep, GlyphRole, PaintStyle, Role, Surface, Theme};

/// One-cluster paint cursor: the single writer shared by cell, string, span,
/// and streaming painters. Control-bearing and zero-width clusters are
/// skipped; a cluster that cannot fit ends the row; painting a lead cell
/// resets a wide glyph it overwrites — both the new glyph's continuation
/// shadow and a stale lead whose second half is overwritten.
struct ClusterSink<'a, 'f> {
    ui: &'a mut Ui<'f>,
    x: u16,
    remaining: u16,
    y: u16,
}

impl ClusterSink<'_, '_> {
    /// Paint one cluster, returning whether output can admit another.
    fn push(&mut self, symbol: &str, s: PaintStyle) -> bool {
        if symbol.contains(char::is_control) {
            return true;
        }
        let width = symbol.cell_width();
        if width == 0 {
            return true;
        }
        let Some(rest) = self.remaining.checked_sub(width) else {
            return false;
        };
        self.remaining = rest;
        let pos = Position::new(self.x, self.y);
        // A wide symbol in the previous cell covers this one: painting a
        // lead here must clear that stale lead, or the terminal keeps
        // showing the old wide glyph over the new paint (BF09).
        if let Some(prev_x) = self.x.checked_sub(1) {
            let prev = Position::new(prev_x, self.y);
            let stale = self.ui.buffer().cell(prev).is_some_and(|c| {
                let symbol = c.symbol();
                !symbol.contains(char::is_control) && symbol.cell_width() >= 2
            });
            if stale {
                if let Some(cell) = self.ui.buffer().cell_mut(prev) {
                    cell.reset();
                }
                self.ui.mark(prev, None);
            }
        }
        if let Some(cell) = self.ui.buffer().cell_mut(pos) {
            cell.set_symbol(symbol).set_style(s.into_style());
        }
        self.ui.mark(pos, Some(s));
        let end = self.x.saturating_add(width);
        self.x = self.x.saturating_add(1);
        while self.x < end {
            let pos = Position::new(self.x, self.y);
            if let Some(cell) = self.ui.buffer().cell_mut(pos) {
                cell.reset();
            }
            self.ui.mark(pos, None);
            self.x = self.x.saturating_add(1);
        }
        self.remaining != 0
    }
}

impl Ui<'_> {
    /// Paint graphemes at `pos`, refusing any grapheme wider than the clip.
    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: impl Into<PaintStyle>) {
        if self.clip.contains(pos) {
            self.paint_str(
                Rect::new(pos.x, pos.y, self.clip.right().saturating_sub(pos.x), 1),
                symbol,
                s,
            );
        }
    }

    /// Paint text with Ratatui's grapheme and width semantics. Lead cells
    /// retain the supplied origin; shadow cells reset both bytes and origin.
    /// This single walk is shared by cell, string, span, and glyph painters.
    pub fn paint_str(&mut self, area: Rect, text: &str, s: impl Into<PaintStyle>) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let s = s.into();
        self.paint_graphemes(area, graphemes(text).map(|(_, symbol)| (symbol, s)))
    }

    /// Paint text with bold emphasis at original-label grapheme ordinals.
    ///
    /// Out-of-range and repeated indices are harmless. Control graphemes keep
    /// their original ordinals but are not painted. Clipping, wide continuations,
    /// and semantic channel provenance use the same writer as `paint_str`.
    pub fn paint_matched(
        &mut self,
        area: Rect,
        text: &str,
        matched: &[usize],
        base: impl Into<PaintStyle>,
    ) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let base = base.into();
        self.paint_graphemes(
            area,
            graphemes(text).enumerate().map(|(index, (_, symbol))| {
                let style = if matched.contains(&index) {
                    base.add_modifier(Modifier::BOLD)
                } else {
                    base
                };
                (symbol, style)
            }),
        )
    }

    // Both callers supply their already-clipped row. Keeping the write walk
    // here makes continuation clearing and provenance identical for all text.
    fn paint_graphemes<'s>(
        &mut self,
        area: Rect,
        symbols: impl Iterator<Item = (&'s str, PaintStyle)>,
    ) -> u16 {
        let mut sink = ClusterSink {
            ui: self,
            x: area.x,
            remaining: area.width,
            y: area.y,
        };
        for (symbol, s) in symbols {
            if !sink.push(symbol, s) {
                break;
            }
        }
        sink.x.saturating_sub(area.x)
    }

    /// Paint middle-truncated text without allocating, preserving semantic style.
    ///
    /// The visible width after ancestor clipping is the truncation budget, as
    /// with `paint_str`. Widths below five use end truncation. Returns columns
    /// painted; wide continuations and control graphemes use the shared writer.
    pub fn paint_middle(&mut self, area: Rect, text: &str, s: impl Into<PaintStyle>) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let s = s.into();
        let mut used = 0u16;
        for part in crate::text::measure::middle_parts(text, area.width) {
            used = used.saturating_add(self.paint_str(
                Rect::new(
                    area.x.saturating_add(used),
                    area.y,
                    area.width.saturating_sub(used),
                    area.height,
                ),
                part,
                s,
            ));
        }
        used
    }

    /// Paint semantic spans with no allocation, inheriting `base` independently
    /// for each span. Width and continuation handling share the string
    /// writer, and all spans segment as one logical line: a cluster split
    /// across spans paints indivisibly in the style of the span holding its
    /// first byte (oracle `viewport.rs:19-21`; F10).
    pub fn paint_spans(
        &mut self,
        area: Rect,
        spans: &[Span<'_>],
        base: impl Into<PaintStyle>,
    ) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let base = base.into();
        let mut scratch = ClusterScratch::new();
        let mut feed = ClusterFeed::new(&mut scratch);
        let mut x = area.x;
        let mut remaining = area.width;
        for sp in spans {
            if x >= area.right() {
                break;
            }
            let mut st = base.add_modifier(sp.add);
            if let Some(role) = sp.role {
                st = st.patch(self.paint_patch(&crate::theme::StylePatch::new().set_fg(role)));
            }
            let mut sink = ClusterSink {
                ui: &mut *self,
                x,
                remaining,
                y: area.y,
            };
            feed.push(sp.text, st, &mut |cluster, style| sink.push(cluster, style));
            x = sink.x;
            remaining = sink.remaining;
        }
        if x < area.right() {
            let mut sink = ClusterSink {
                ui: &mut *self,
                x,
                remaining,
                y: area.y,
            };
            feed.finish(base, &mut |cluster, style| sink.push(cluster, style));
            x = sink.x;
        }
        x.saturating_sub(area.x)
    }

    /// Paint one [`core::fmt::Display`] value as a single logical line
    /// (BA-FMT): the value formats exactly once through a streaming
    /// logical-grapheme projection shared with the borrowed span engine, so
    /// a `Display` impl that writes a cluster in separate fragments paints
    /// the same cells as the unsplit text. Clipping, tab/control policy,
    /// width measure, original-byte provenance and wide-cell shadow reset
    /// match the shared writer; cluster style belongs to its first byte.
    ///
    /// Crate-private: the future `RowUi`/`CellUi` consumer migration
    /// (TASK-015) calls this; no public scratch argument is added. Pending
    /// storage is the frame's reusable text scratch: content is wiped on
    /// completion, `fmt::Error` and unwind while capacity is retained.
    pub(crate) fn paint_display(
        &mut self,
        area: Rect,
        value: &impl core::fmt::Display,
        style: impl Into<PaintStyle>,
    ) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let style = style.into();
        let mut guard = StreamGuard::take(self);
        guard.paint(area, value, style)
    }

    /// Restyle `area` without touching symbols (`Buffer::set_style`).
    pub fn paint_style(&mut self, area: Rect, s: impl Into<PaintStyle>) {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        self.buffer().set_style(area, s.into_style());
        self.mark_area(area, Some(s));
    }

    /// Fill `area` with spaces in `s` (per-position `set_symbol(" ")`).
    pub fn fill(&mut self, area: Rect, s: impl Into<PaintStyle>) {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        {
            let buf = self.buffer();
            for pos in area.positions() {
                if let Some(c) = buf.cell_mut(pos) {
                    c.set_symbol(" ").set_style(s.into_style());
                }
            }
        }
        self.mark_area(area, Some(s));
    }

    /// A quiet rule across `area`'s first row (`GlyphRole::RuleQuiet`).
    pub fn rule(&mut self, area: Rect) {
        let g = self.theme_ref().design.glyphs.get(GlyphRole::RuleQuiet);
        let s = self.paint_patch(&crate::theme::StylePatch::new().set_fg(Role::BorderSubtle));
        let row = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        }
        .intersection(self.clip);
        for x in row.columns() {
            self.paint_cell(Position::new(x.x, row.y), g, s);
        }
    }

    /// Draw the theme border set around `area` in `s`; returns the inner rect.
    pub fn frame(&mut self, area: Rect, s: impl Into<PaintStyle>) -> Rect {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.width < 2 || area.height < 2 {
            return Rect::ZERO;
        }
        let b = self.theme_ref().design.borders;
        let left = area.left();
        let right = area.right().saturating_sub(1);
        let top = area.top();
        let bottom = area.bottom().saturating_sub(1);
        for col in area.columns().map(|c| c.x) {
            self.paint_cell(Position::new(col, top), b.horizontal_top, s);
            self.paint_cell(Position::new(col, bottom), b.horizontal_bottom, s);
        }
        for row in area.rows().map(|r| r.y) {
            self.paint_cell(Position::new(left, row), b.vertical_left, s);
            self.paint_cell(Position::new(right, row), b.vertical_right, s);
        }
        self.paint_cell(Position::new(left, top), b.top_left, s);
        self.paint_cell(Position::new(right, top), b.top_right, s);
        self.paint_cell(Position::new(left, bottom), b.bottom_left, s);
        self.paint_cell(Position::new(right, bottom), b.bottom_right, s);
        Rect {
            x: left.saturating_add(1),
            y: top.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    }

    /// Paint a glyph role at `area`'s origin; returns the columns written.
    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: impl Into<PaintStyle>) -> u16 {
        let s = s.into();
        let sym = self.theme_ref().design.glyphs.get(g);
        self.paint_str(area, sym, s)
    }

    /// The buffer and the current clip rect. The documented escape hatch:
    /// marks the whole clip rect written.
    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let clip = self.clip;
        self.mark_area(clip, None);
        (self.buffer(), clip)
    }

    /// Dim the page under a layer by walking the role recorded per painted
    /// cell and stepping it down the foreground ladder semantically
    /// (§54, `docs/reviews/laneC-app-tick.md` Q4). `steps == 0` is identity:
    /// not a restyle to the same colours, but no write at all, so the frame
    /// is byte-identical. One step is `Fg(Muted)`, two `Fg(Faint)`, three
    /// `Fg(Ghost)` and four or more erases the glyph into the resolved
    /// backdrop background; ladder roles start from their own rung and erase
    /// once they step past `Ghost`; `Accent`/`AccentHover`/`AccentPressed`
    /// walk the accent chain and erase past its end. Backgrounds resolve
    /// from the recorded background role — never by colour identity, which
    /// is exactly the reverse-lookup defect this replaces. Only `BOLD`
    /// survives; every other modifier is cleared. Walks only `area`.
    pub fn dim_layer(&mut self, area: Rect, steps: u8) {
        if steps == 0 {
            return;
        }
        // Fail closed under timing: color-binding paint branches inside
        // `dim_layer` are outside the censused entry set. A reached
        // `dim_layer` records an uncovered-path witness rather than being
        // silently zero or timing the whole paint operation.
        #[cfg(feature = "testing")]
        self.note_uncovered_binding();
        let area = area.intersection(self.frame.screen);
        let theme = self.theme_ref();
        let surface = self.surface;
        let backdrop_text = crate::theme::resolve::bind_role(theme, Role::BackdropFg, surface);
        let backdrop_fill = crate::theme::resolve::bind_role(theme, Role::BackdropBg, surface);
        for pos in area.positions() {
            let roles = self.roles_at(pos);
            let fg = match roles.fg {
                // a ladder role steps from its own rung and erases past Ghost
                Some(Role::Fg(step)) => ladder(theme, surface, step.index(), steps),
                // the accent chain degrades through hover and pressed
                Some(Role::Accent) => accent(theme, surface, 0, steps),
                Some(Role::AccentHover) => accent(theme, surface, 1, steps),
                Some(Role::AccentPressed) => accent(theme, surface, 2, steps),
                // a background role recorded as a foreground carries no text
                Some(Role::CurrentSurface | Role::RaisedSurface | Role::Surface(_)) => {
                    FadeResult::Fg(None)
                }
                // every other semantic foreground: Muted, Faint, Ghost, erase
                Some(_) => ladder(theme, surface, 1, steps),
                None => FadeResult::Fg(backdrop_text),
            };
            let bg = match roles.bg {
                Some(Role::Surface(s)) => {
                    crate::theme::resolve::bind_role(theme, Role::Surface(s), surface)
                }
                Some(Role::CurrentSurface) => Some(theme.bg(Surface::Canvas)),
                Some(Role::RaisedSurface) => Some(theme.bg(Surface::Surface)),
                _ => backdrop_fill,
            };
            let page = self.page_mut();
            if let Some(c) = page.cell_mut(pos) {
                let bold = c.modifier.intersection(Modifier::BOLD);
                let mut st = Style::new();
                st.fg = match fg {
                    FadeResult::Fg(f) => f,
                    // erased: the glyph goes, and what is left is the
                    // resolved backdrop background
                    FadeResult::Erase => {
                        c.set_symbol(" ");
                        backdrop_fill
                    }
                };
                st.bg = bg;
                c.set_style(st);
                c.modifier = bold;
            }
        }
    }
}

/// Scoped owner of the frame's reusable text scratch for one streaming
/// paint: takes the scratch for the guarded paint, then wipes its content
/// and restores its capacity on normal completion, returned `fmt::Error`
/// and panic unwind alike. Never emits a partial cluster while unwinding.
struct StreamGuard<'a, 'f> {
    ui: &'a mut Ui<'f>,
    scratch: ClusterScratch,
}

impl<'a, 'f> StreamGuard<'a, 'f> {
    fn take(ui: &'a mut Ui<'f>) -> Self {
        let scratch = core::mem::take(&mut ui.frame.text_scratch);
        StreamGuard { ui, scratch }
    }

    /// Format `value` exactly once through the shared projection, flushing
    /// the successfully written prefix even when `Display` returns
    /// `fmt::Error`. Formatter writes past the clip still run (their effects
    /// are the caller's) but no longer segment or grow the scratch.
    fn paint(&mut self, area: Rect, value: &impl core::fmt::Display, style: PaintStyle) -> u16 {
        let mut feed = ClusterFeed::new(&mut self.scratch);
        let mut sink = ClusterSink {
            ui: &mut *self.ui,
            x: area.x,
            remaining: area.width,
            y: area.y,
        };
        let mut out = StreamWriter {
            feed: &mut feed,
            sink: &mut sink,
            style,
        };
        let _ = core::fmt::write(&mut out, format_args!("{value}"));
        feed.finish((), &mut |cluster, ()| sink.push(cluster, style));
        sink.x.saturating_sub(area.x)
    }
}

impl Drop for StreamGuard<'_, '_> {
    fn drop(&mut self) {
        self.scratch.clear();
        core::mem::swap(&mut self.ui.frame.text_scratch, &mut self.scratch);
    }
}

/// The `fmt::Write` adapter a streamed `Display` writes into: every
/// fragment feeds the shared cross-fragment projection, and completed
/// clusters paint through the one writer. Always accepts (returning `Ok`)
/// so formatting continues past the clip without segmenting further.
struct StreamWriter<'m, 's, 'a, 'f> {
    feed: &'m mut ClusterFeed<'s, ()>,
    sink: &'m mut ClusterSink<'a, 'f>,
    style: PaintStyle,
}

impl core::fmt::Write for StreamWriter<'_, '_, '_, '_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let Self { feed, sink, style } = self;
        feed.push(s, (), &mut |cluster, ()| sink.push(cluster, *style));
        Ok(())
    }
}

/// The outcome of stepping one recorded foreground role down.
enum FadeResult {
    /// The dimmed foreground (`None` leaves the cell's foreground alone).
    Fg(Option<Color>),
    /// The glyph is erased into the backdrop.
    Erase,
}

/// Step `base` (an `FgStep` index) down by `steps`, erasing past `Ghost`.
fn ladder(theme: &Theme, surface: Surface, base: usize, steps: u8) -> FadeResult {
    match base.saturating_add(usize::from(steps)) {
        i if i <= 4 => FadeResult::Fg(crate::theme::resolve::bind_role(
            theme,
            Role::Fg(index_to_step(i)),
            surface,
        )),
        _ => FadeResult::Erase,
    }
}

/// Step the accent chain (`Accent`, `AccentHover`, `AccentPressed`) down by
/// `steps` from `base`, erasing past its end.
fn accent(theme: &Theme, surface: Surface, base: usize, steps: u8) -> FadeResult {
    let role = match base.saturating_add(usize::from(steps)) {
        0 => Role::Accent,
        1 => Role::AccentHover,
        2 => Role::AccentPressed,
        _ => return FadeResult::Erase,
    };
    FadeResult::Fg(crate::theme::resolve::bind_role(theme, role, surface))
}

const fn index_to_step(i: usize) -> FgStep {
    match i {
        0 => FgStep::Primary,
        1 => FgStep::Secondary,
        2 => FgStep::Muted,
        3 => FgStep::Faint,
        _ => FgStep::Ghost,
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use core::fmt;
    use std::panic::AssertUnwindSafe;

    use junie_tui_testing::perf;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier, Style};
    use unicode_segmentation::UnicodeSegmentation;

    use super::super::cx::LastFrame;
    use super::super::{FrameState, Ui, UiCore};
    use crate::text::Span;
    use crate::text::clusters::{ClusterFeed, ClusterScratch};
    use crate::theme::{FgStep, Role, Surface, Theme};

    #[global_allocator]
    static ALLOCATOR: perf::Counting = perf::Counting;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 8,
        height: 2,
    };

    fn with_ui<R>(theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            f(&mut ui)
        };
        (out, page)
    }

    /// Paint `symbol` at `(0, 0)` carrying `fg` as its recorded foreground
    /// role over the canvas, then dim the screen by `steps`.
    fn dimmed_cell(
        theme: &Theme,
        fg: Role,
        symbol: &str,
        modifier: Modifier,
        steps: u8,
    ) -> ratatui_core::buffer::Cell {
        let ((), page) = with_ui(theme, |ui| {
            let style = ui.paint_patch(
                &crate::theme::StylePatch::new()
                    .set_fg(fg)
                    .set_bg(Role::CurrentSurface)
                    .add(modifier),
            );
            ui.paint_cell(Position::ORIGIN, symbol, style);
            ui.dim_layer(SCREEN, steps);
        });
        page.cell(Position::ORIGIN).expect("cell").clone()
    }

    fn fg_of(theme: &Theme, step: FgStep) -> Color {
        crate::theme::resolve::bind_role(theme, Role::Fg(step), Surface::Canvas).expect("fg")
    }

    /// §54: `dim_layer(area, 0)` is identity. It is not a restyle to the
    /// colours the roles already resolve to — it writes nothing at all, so
    /// every cell, including symbols, modifiers and never-painted cells,
    /// is byte-for-byte what it was.
    #[test]
    fn dim_layer_zero_steps_is_byte_identical() {
        for theme in [Theme::junie(), Theme::paper()] {
            let (before, after) = with_ui(&theme, |ui| {
                ui.paint_str(
                    SCREEN,
                    "ok",
                    Style::new()
                        .fg(fg_of(&theme, FgStep::Primary))
                        .add_modifier(Modifier::ITALIC | Modifier::BOLD),
                );
                ui.paint_cell(
                    Position::new(4, 1),
                    "x",
                    Style::new().fg(Color::Green).add_modifier(Modifier::DIM),
                );
                let before = ui.page_mut().clone();
                ui.dim_layer(SCREEN, 0);
                (before, ui.page_mut().clone())
            })
            .0;
            assert_eq!(before, after, "dim_layer(area, 0) must write nothing");
        }
    }

    /// Q4: the four non-ladder tone roles walk Muted, Faint, Ghost and then
    /// erase into the resolved backdrop background; ladder roles step from
    /// their own rung and erase past `Ghost`; the accent chain degrades
    /// through hover and pressed and then erases. Only `BOLD` survives, and
    /// nothing is decided by colour identity.
    #[test]
    fn dim_layer_semantic_roles_step_monotonically_and_erase() {
        for theme in [Theme::junie(), Theme::paper()] {
            let backdrop_bg =
                crate::theme::resolve::bind_role(&theme, Role::BackdropBg, Surface::Canvas);
            // the four non-ladder tones: Muted, Faint, Ghost, erase
            for role in [Role::Success, Role::Warning, Role::Danger, Role::Info] {
                for (steps, step) in [(1u8, FgStep::Muted), (2, FgStep::Faint), (3, FgStep::Ghost)]
                {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.fg, fg_of(&theme, step), "{role:?} at {steps}");
                    assert_eq!(c.symbol(), "x", "{role:?} at {steps} keeps its glyph");
                }
                for steps in [4u8, 5, 9] {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.symbol(), " ", "{role:?} erases at {steps}");
                    assert_eq!(c.fg, backdrop_bg.expect("backdrop"), "{role:?} at {steps}");
                }
            }
            // every other non-ladder foreground role uses the same rule —
            // `BorderSubtle` and `DisabledFg` are exactly the two the legacy
            // colour-identity lookup misclassified
            for role in [Role::BorderSubtle, Role::DisabledFg, Role::Focus] {
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 1).fg,
                    fg_of(&theme, FgStep::Muted),
                    "{role:?}"
                );
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 4).symbol(),
                    " ",
                    "{role:?}"
                );
            }
            // ladder roles step from their own rung, saturating at Ghost and
            // erasing only past it
            for (start, steps, want) in [
                (FgStep::Primary, 1u8, FgStep::Secondary),
                (FgStep::Primary, 4, FgStep::Ghost),
                (FgStep::Secondary, 2, FgStep::Faint),
                (FgStep::Muted, 2, FgStep::Ghost),
                (FgStep::Ghost, 0, FgStep::Ghost),
            ] {
                let c = dimmed_cell(&theme, Role::Fg(start), "x", Modifier::empty(), steps);
                assert_eq!(c.fg, fg_of(&theme, want), "{start:?} + {steps}");
                assert_eq!(c.symbol(), "x");
            }
            for (start, steps) in [
                (FgStep::Primary, 5u8),
                (FgStep::Muted, 3),
                (FgStep::Ghost, 1),
            ] {
                let c = dimmed_cell(&theme, Role::Fg(start), "x", Modifier::empty(), steps);
                assert_eq!(c.symbol(), " ", "{start:?} + {steps} erases past Ghost");
                assert_eq!(c.fg, backdrop_bg.expect("backdrop"));
            }
            // the accent chain
            let accent_of = |r: Role| {
                crate::theme::resolve::bind_role(&theme, r, Surface::Canvas).expect("accent")
            };
            for (start, steps, want) in [
                (Role::Accent, 1u8, Role::AccentHover),
                (Role::Accent, 2, Role::AccentPressed),
                (Role::AccentHover, 1, Role::AccentPressed),
            ] {
                let c = dimmed_cell(&theme, start, "x", Modifier::empty(), steps);
                assert_eq!(c.fg, accent_of(want), "{start:?} + {steps}");
                assert_eq!(c.symbol(), "x");
            }
            for (start, steps) in [
                (Role::Accent, 3u8),
                (Role::AccentHover, 2),
                (Role::AccentPressed, 1),
            ] {
                let c = dimmed_cell(&theme, start, "x", Modifier::empty(), steps);
                assert_eq!(c.symbol(), " ", "{start:?} + {steps} erases past the chain");
                assert_eq!(c.fg, backdrop_bg.expect("backdrop"));
            }
            // only BOLD survives
            let c = dimmed_cell(
                &theme,
                Role::Fg(FgStep::Primary),
                "x",
                Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED,
                1,
            );
            assert_eq!(c.modifier, Modifier::BOLD);
            let c = dimmed_cell(
                &theme,
                Role::Fg(FgStep::Primary),
                "x",
                Modifier::ITALIC | Modifier::REVERSED,
                1,
            );
            assert_eq!(c.modifier, Modifier::empty());
            // the background is resolved from the recorded background role,
            // never from the cell's colour
            let c = dimmed_cell(&theme, Role::Fg(FgStep::Primary), "x", Modifier::empty(), 1);
            assert_eq!(c.bg, theme.bg(Surface::Canvas));
        }
    }

    // ── TASK-013 streaming logical-grapheme projection ──

    /// A `Display` that writes `text` in two fragments around `split` with an
    /// empty write between, counting original invocations and fragment
    /// writes. Every UTF-8 split proves fragments are not cluster
    /// boundaries; the empty write proves empty fragments delimit nothing.
    struct Fragmented<'a> {
        text: &'a str,
        split: usize,
        calls: &'a Cell<usize>,
        writes: &'a Cell<usize>,
    }

    impl fmt::Display for Fragmented<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.calls.set(self.calls.get() + 1);
            f.write_str(self.text.get(..self.split).unwrap_or(""))?;
            self.writes.set(self.writes.get() + 1);
            f.write_str("")?;
            self.writes.set(self.writes.get() + 1);
            f.write_str(self.text.get(self.split..).unwrap_or(""))?;
            self.writes.set(self.writes.get() + 1);
            Ok(())
        }
    }

    /// Painter-level scratch diagnostics: live content, per-use peak,
    /// retained capacity and wipe state of the frame's text scratch.
    struct ScratchDiag {
        len: usize,
        peak: usize,
        capacity: usize,
        wiped: bool,
    }

    fn scratch_diag(ui: &Ui<'_>) -> ScratchDiag {
        let scratch = &ui.frame.text_scratch;
        ScratchDiag {
            len: scratch.len(),
            peak: scratch.peak(),
            capacity: scratch.capacity(),
            wiped: scratch.content_is_wiped(),
        }
    }

    fn with_page<R>(theme: &Theme, page: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let mut frame = FrameState::default();
        frame.reset(1, page);
        let mut buf = Buffer::empty(page);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, theme, &last);
            f(&mut ui)
        };
        (out, buf)
    }

    /// Resolve one span's paint style exactly as `paint_spans` does; the
    /// projection tests pin ownership (which span styles a cluster), while
    /// this accepted per-span formula stays fixed.
    fn resolve_span(
        ui: &Ui<'_>,
        base: crate::theme::PaintStyle,
        sp: &Span<'_>,
    ) -> crate::theme::PaintStyle {
        let mut st = base.add_modifier(sp.add);
        if let Some(role) = sp.role {
            st = st.patch(ui.paint_patch(&crate::theme::StylePatch::new().set_fg(role)));
        }
        st
    }

    /// TASK-013: the streamed `Display` seam paints exactly the unsplit
    /// cells at every UTF-8 fragment boundary (including empty fragments),
    /// invokes the original `Display` once, keeps formatter effects past the
    /// clip, and never backfills a suffix after a nonfitting wide cluster.
    #[test]
    fn completion_013_streaming_projection() {
        let long_256 = format!("a{}z", "\u{301}".repeat(256));
        let corpus: Vec<String> = [
            "",
            "plain",
            "界半x",
            "e\u{301}x",
            "👩‍💻!",
            "🇺🇸🇫🇷",
            "\r\nX",
            "क्\u{200d}ष",
            "a\u{301}\u{301}",
            "x日y",
            "日b",
            "a\tb",
        ]
        .into_iter()
        .map(str::to_string)
        .chain([long_256])
        .collect();
        for theme in [Theme::junie(), Theme::paper()] {
            let style = Style::new().fg(Color::White);
            for text in &corpus {
                let splits: Vec<usize> = if text.len() > 64 {
                    let mid = text.len() / 2;
                    [0, 1, 2, mid.saturating_sub(1), mid, text.len()]
                        .into_iter()
                        .filter(|s| text.is_char_boundary(*s))
                        .collect()
                } else {
                    text.char_indices()
                        .map(|(i, _)| i)
                        .chain([text.len()])
                        .collect()
                };
                for split in splits {
                    for width in [0u16, 1, 2, 3, 5, 8, 20] {
                        for origin_x in [0u16, 7u16] {
                            let page = Rect::new(0, 0, 40, 2);
                            let area = Rect::new(origin_x, 1, width, 1);
                            let calls = Cell::new(0);
                            let writes = Cell::new(0);
                            let item = Fragmented {
                                text,
                                split,
                                calls: &calls,
                                writes: &writes,
                            };
                            let ((used_stream, diag), streamed) = with_page(&theme, page, |ui| {
                                let used = ui.paint_display(area, &item, style);
                                (used, scratch_diag(ui))
                            });
                            let (used_plain, plain) =
                                with_page(&theme, page, |ui| ui.paint_str(area, text, style));
                            assert_eq!(
                                streamed, plain,
                                "split {split} of {text:?} at width {width} x {origin_x}"
                            );
                            assert_eq!(
                                used_stream, used_plain,
                                "used columns at split {split} of {text:?}"
                            );
                            // An empty paint never invokes the value; any
                            // live paint invokes it exactly once with all
                            // three fragment writes, even past the clip.
                            let expected = if width == 0 { (0, 0) } else { (1, 3) };
                            assert_eq!((calls.get(), writes.get()), expected);
                            assert_eq!(diag.len, 0, "logical scratch rests empty");
                            assert!(diag.wiped, "scratch content wiped after paint");
                        }
                    }
                }
            }
        }
        // A wide cluster that cannot fit ends output: no skip-and-continue,
        // no suffix backfill — the shared writer's exact clipping.
        for theme in [Theme::junie(), Theme::paper()] {
            let style = Style::new().fg(Color::White);
            for (text, width, filled) in [
                ("日b", 1u16, " "),
                ("x日y", 2u16, "x "),
                ("ab日", 2u16, "ab"),
            ] {
                let (_, buf) = with_page(&theme, Rect::new(0, 0, 8, 1), |ui| {
                    ui.paint_display(Rect::new(0, 0, width, 1), &text, style)
                });
                let row: String = (0..width)
                    .map(|x| buf.cell(Position::new(x, 0)).unwrap().symbol().to_string())
                    .collect();
                assert_eq!(row, filled, "{text:?} at width {width}");
            }
        }
    }

    /// TASK-013: cluster style belongs to the span holding its first byte.
    /// Split paints agree on cells, widths and recorded provenance with an
    /// independent per-cluster oracle — direct Unicode segmentation of the
    /// joined line, first-byte span attribution by byte arithmetic, and one
    /// complete cluster painted per `paint_str` call so no reference input
    /// is itself split — at every UTF-8 boundary of combining, ZWJ and
    /// regional-indicator clusters.
    #[test]
    fn completion_013_split_span_first_byte_style() {
        use ratatui_core::buffer::CellWidth as _;
        for theme in [Theme::junie(), Theme::paper()] {
            for text in [
                "e\u{301}x",
                "👩‍💻ab",
                "🇺🇸x",
                "a\u{301}\u{301}b",
                "界x",
                "\r\nX",
            ] {
                let splits: Vec<usize> = text
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain([text.len()])
                    .collect();
                for split in splits {
                    let (first, second) = (
                        text.get(..split).unwrap_or(""),
                        text.get(split..).unwrap_or(""),
                    );
                    for fragments in [
                        vec![Span::new(first), Span::new(second)],
                        vec![
                            Span::new(first).role(Role::Accent),
                            Span::new(second).role(Role::Danger),
                        ],
                        vec![
                            Span::new(""),
                            Span::new(first).role(Role::Success),
                            Span::new(""),
                            Span::new(second).role(Role::Info),
                            Span::new(""),
                        ],
                    ] {
                        // Byte ranges owned by each fragment over the joined
                        // line, for first-byte attribution.
                        let mut ranges: Vec<(usize, usize)> = Vec::new();
                        let mut at = 0usize;
                        for span in &fragments {
                            ranges.push((at, at + span.text.len()));
                            at += span.text.len();
                        }
                        for width in [0u16, 1, 2, 3, 5, 8, 20] {
                            let page = Rect::new(0, 0, 40, 2);
                            let area = Rect::new(7, 1, width, 1);
                            let base = crate::theme::PaintStyle::new();
                            let ((used_split, roles_split), buf_split) =
                                with_page(&theme, page, |ui| {
                                    let used = ui.paint_spans(area, &fragments, base);
                                    let roles: Vec<_> = (0..page.width)
                                        .map(|x| ui.roles_at(Position::new(x, 1)))
                                        .collect();
                                    (used, roles)
                                });
                            // Independent oracle: segment the joined line
                            // directly, attribute each cluster to its
                            // first byte's span, and paint one complete
                            // cluster per call at its exact column.
                            let ((used_oracle, roles_oracle), buf_oracle) =
                                with_page(&theme, page, |ui| {
                                    let mut col = area.x;
                                    for (index, cluster) in text.grapheme_indices(true) {
                                        if col >= area.right() {
                                            break;
                                        }
                                        let owner = ranges
                                            .iter()
                                            .position(|(lo, hi)| *lo <= index && index < *hi)
                                            .unwrap_or(0);
                                        let style = fragments
                                            .get(owner)
                                            .map_or(base, |span| resolve_span(ui, base, span));
                                        let cell = Rect::new(
                                            col,
                                            area.y,
                                            area.right().saturating_sub(col),
                                            1,
                                        );
                                        col =
                                            col.saturating_add(ui.paint_str(cell, cluster, style));
                                        // A refused wide cluster ends the
                                        // row exactly like the writer: the
                                        // next cluster starts past the end.
                                        // Controls are filtered before any
                                        // width call, as in the writer.
                                        if !cluster.contains(char::is_control)
                                            && cluster.cell_width() > 0
                                            && col == cell.x
                                            && cluster.cell_width() > cell.width
                                        {
                                            break;
                                        }
                                    }
                                    let roles: Vec<_> = (0..page.width)
                                        .map(|x| ui.roles_at(Position::new(x, 1)))
                                        .collect();
                                    (col.saturating_sub(area.x), roles)
                                });
                            assert_eq!(
                                buf_split, buf_oracle,
                                "{text:?} split {split} at width {width}"
                            );
                            assert_eq!(used_split, used_oracle, "used columns");
                            assert_eq!(roles_split, roles_oracle, "provenance");
                        }
                    }
                }
            }
        }
    }

    /// Feed-only scratch allocation scopes: cold, warm and fresh-owner
    /// event counts plus the clusters observed. The sink stores nothing,
    /// so every counted event is pending-cluster scratch.
    fn feed_allocs(
        text: &str,
        split: usize,
        scratch: &mut ClusterScratch,
    ) -> (usize, usize, usize) {
        let before = perf::allocs();
        let bytes_before = perf::bytes();
        let mut clusters = 0usize;
        {
            let mut feed = ClusterFeed::new(scratch);
            let mut sink = |_: &str, _: ()| {
                clusters += 1;
                true
            };
            feed.push(text.get(..split).unwrap_or(""), (), &mut sink);
            feed.push(text.get(split..).unwrap_or(""), (), &mut sink);
            feed.finish((), &mut sink);
        }
        (
            perf::allocs() - before,
            perf::bytes() - bytes_before,
            clusters,
        )
    }

    /// TASK-013: honest cold/warm/fresh scratch accounting with the fixed
    /// ceilings (5 events for a 256-mark cluster, 9 for 4096 marks; zero
    /// for the inline fixtures), lifetime high-water capacity retention,
    /// bounded validation work and warm paint totals that retain output
    /// storage instead of hiding it.
    #[test]
    fn completion_013_streaming_storage() {
        let _guard = perf::lock();
        let mark_256 = format!("a{}z", "\u{301}".repeat(256));
        let mark_4096 = format!("a{}z", "\u{301}".repeat(4096));
        for (name, text) in [
            ("ascii", "hello".to_string()),
            ("cjk", "中文".to_string()),
            ("combining", "e\u{301}".to_string()),
            ("zwj", "👩‍👩‍👧‍👦".to_string()),
            ("long256", mark_256),
            ("long4096", mark_4096),
        ] {
            let split = text.char_indices().nth(1).map_or(text.len(), |(i, _)| i);
            let mut scratch = ClusterScratch::default();
            let cold = feed_allocs(&text, split, &mut scratch);
            let cold_peak = scratch.peak();
            let capacity = scratch.capacity();
            let validated = scratch.validated_bytes();
            let warm = feed_allocs(&text, split, &mut scratch);
            let fresh = feed_allocs(&text, split, &mut ClusterScratch::default());
            let largest = text.graphemes(true).map(str::len).max().unwrap_or(0);
            assert!(
                cold_peak <= largest + 4,
                "{name}: peak {cold_peak} > {largest} + 4"
            );
            assert!(
                cold_peak >= largest,
                "{name}: pending never held its cluster"
            );
            assert!(
                validated <= 4 * 64 * (text.chars().count() + 1),
                "{name}: revalidated growing heap prefix"
            );
            assert_eq!(warm.0, 0, "{name}: warm scratch allocates");
            assert_eq!(cold.0, fresh.0, "{name}: fresh owner is not cold");
            assert_eq!(scratch.len(), 0, "{name}: logical scratch rests empty");
            if name.starts_with("long") {
                let ceiling = if name == "long256" { 5 } else { 9 };
                assert!(
                    (1..=ceiling).contains(&cold.0),
                    "{name}: cold {} events, ceiling {ceiling}",
                    cold.0
                );
                assert!(
                    capacity < 2 * (largest + 4),
                    "{name}: retained {capacity} beyond lifetime high-water"
                );
            } else {
                assert_eq!(cold.0, 0, "{name}: inline scratch allocates cold");
                assert_eq!(capacity, 0, "{name}: inline scratch retains heap");
            }
        }
        // Long→short→long keeps only high-water capacity; a fresh owner is
        // cold; a resize reuses the same owner's capacity with zero scratch
        // events and identical first frames.
        let long = format!("a{}z", "\u{301}".repeat(4096));
        let mut scratch = ClusterScratch::default();
        let cold = feed_allocs(&long, 1, &mut scratch);
        let high_water = scratch.capacity();
        let short = feed_allocs("abc", 1, &mut scratch);
        assert_eq!(short.0, 0);
        assert_eq!(scratch.capacity(), high_water);
        assert!(scratch.content_is_wiped());
        let warm = feed_allocs(&long, 1, &mut scratch);
        let fresh = feed_allocs(&long, 1, &mut ClusterScratch::default());
        assert_eq!((warm.0, fresh.0), (0, cold.0));
        assert!((1..=9).contains(&cold.0));
        assert!(high_water < 2 * (8193 + 4));
        assert!(
            high_water > 2 * ("abc".len() + 4),
            "retention is not bounded by the last short draw"
        );
        // Paint-level totals retain long-cell output storage on the warm
        // pass: scratch goes quiet, cells do not.
        for theme in [Theme::junie(), Theme::paper()] {
            let style = Style::new().fg(Color::White);
            let page = Rect::new(0, 0, 24, 1);
            let area = Rect::new(0, 0, 20, 1);
            let mut frame = FrameState::default();
            frame.reset(1, page);
            let mut buf = Buffer::empty(page);
            let mut core = UiCore::default();
            let last = LastFrame::default();
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, &theme, &last);
            let calls = Cell::new(0);
            let writes = Cell::new(0);
            let item = Fragmented {
                text: &long,
                split: 1,
                calls: &calls,
                writes: &writes,
            };
            let before = perf::allocs();
            ui.paint_display(area, &item, style);
            let pass0 = perf::allocs() - before;
            let before = perf::allocs();
            ui.paint_display(area, &item, style);
            let pass1 = perf::allocs() - before;
            // The warm pass allocates exactly the one long-cell output
            // symbol: scratch is silent, output storage is retained rather
            // than hidden as warm zero. The cold pass adds only the
            // ceiling-bounded scratch growth measured feed-only above.
            assert_eq!(pass1, 1, "warm paint retains its output cell");
            assert!(
                (1..=9).contains(&pass0.saturating_sub(pass1)),
                "cold paint adds ceiling-bounded scratch only"
            );
            assert_eq!((calls.get(), writes.get()), (2, 6));
        }
    }

    /// A `Display` that writes its prefix, then fails.
    struct Fails<'a>(&'a str);

    impl fmt::Display for Fails<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)?;
            Err(fmt::Error)
        }
    }

    /// A `Display` that writes its prefix, then panics.
    struct Panics<'a>(&'a str);

    impl fmt::Display for Panics<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)?;
            panic!("intentional formatter failure");
        }
    }

    /// TASK-013: a returned `fmt::Error` still flushes its written prefix;
    /// unwind clears content and restores reusable capacity without
    /// emitting a partial cluster; `Debug` never exposes scratch bytes;
    /// independent frames share nothing.
    #[test]
    fn completion_013_streaming_cleanup() {
        let _guard = perf::lock();
        // A returned error flushes the successfully written prefix.
        for theme in [Theme::junie(), Theme::paper()] {
            let style = Style::new().fg(Color::White);
            let prefix = format!("a{}", "\u{301}".repeat(256));
            let page = Rect::new(0, 0, 24, 2);
            let area = Rect::new(0, 0, 20, 1);
            let ((diag, used_err), failed) = with_page(&theme, page, |ui| {
                let used = ui.paint_display(area, &Fails(&prefix), style);
                (scratch_diag(ui), used)
            });
            let (used_ok, flushed) =
                with_page(&theme, page, |ui| ui.paint_display(area, &prefix, style));
            assert_eq!(failed, flushed, "error paint flushes its prefix");
            assert_eq!(used_err, used_ok);
            assert_eq!(diag.len, 0);
            assert!(diag.wiped);
        }
        // Unwind clears content, restores capacity, and never emits a
        // partial cluster; the same owner paints normally afterwards.
        let theme = Theme::junie();
        let style = Style::new().fg(Color::White);
        let page = Rect::new(0, 0, 24, 1);
        let area = Rect::new(0, 0, 20, 1);
        let mut frame = FrameState::default();
        frame.reset(1, page);
        let mut buf = Buffer::empty(page);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, &theme, &last);
            ui.paint_display(area, &format!("a{}z", "\u{301}".repeat(256)), style);
        }
        let grown = frame.text_scratch.capacity();
        assert!(grown > 0, "long paint grows reusable capacity");
        // A fresh buffer isolates the unwind paint from the long row above.
        buf = Buffer::empty(page);
        {
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, &theme, &last);
            let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
                ui.paint_display(area, &Panics("unwound"), style);
            }));
            assert!(outcome.is_err());
        }
        assert_eq!(frame.text_scratch.len(), 0);
        assert!(frame.text_scratch.content_is_wiped());
        assert_eq!(
            frame.text_scratch.capacity(),
            grown,
            "unwind restores capacity"
        );
        // "unwound" is 7 ASCII clusters: the 6 completed ones painted
        // before the panic, but the pending final cluster was never
        // emitted during unwinding and the scratch holds nothing.
        let row: String = (0..7)
            .map(|x| buf.cell(Position::new(x, 0)).unwrap().symbol().to_string())
            .collect();
        assert_eq!(row, "unwoun ");
        // The same owner paints normally afterwards.
        {
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, &theme, &last);
            ui.paint_display(area, &"recovered", style);
        }
        assert_eq!(frame.text_scratch.len(), 0);
        let row: String = (0..9)
            .map(|x| buf.cell(Position::new(x, 0)).unwrap().symbol().to_string())
            .collect();
        assert_eq!(row, "recovered");
        // `Debug` reports lengths only, never bytes.
        let mut scratch = ClusterScratch::default();
        {
            let mut feed = ClusterFeed::new(&mut scratch);
            let mut sink = |_: &str, _: ()| true;
            feed.push("hunter2", (), &mut sink);
        }
        let debug = format!("{scratch:?}");
        assert!(!debug.contains("hunter2"), "scratch Debug leaks content");
        assert!(debug.contains("len"), "{debug}");
        // Independent frames share nothing: a second fresh frame is cold.
        let (diag_b, _) = with_page(&theme, page, |ui| {
            ui.paint_display(area, &"abc", style);
            scratch_diag(ui)
        });
        assert_eq!(diag_b.capacity, 0, "fresh frame starts cold");
        // A `reset` (resize) keeps the same owner's warm capacity while the
        // logical scratch stays empty.
        let mut frame = FrameState::default();
        frame.reset(1, page);
        let mut buf = Buffer::empty(page);
        let mut core = UiCore::default();
        {
            let mut ui = Ui::new(&mut frame, &mut buf, &mut core, &theme, &last);
            ui.paint_display(area, &format!("a{}z", "\u{301}".repeat(256)), style);
        }
        let warm_capacity = frame.text_scratch.capacity();
        assert!(warm_capacity > 0);
        frame.reset(2, Rect::new(0, 0, 8, 1));
        assert_eq!(frame.text_scratch.capacity(), warm_capacity);
        assert_eq!(frame.text_scratch.len(), 0);
    }
}
