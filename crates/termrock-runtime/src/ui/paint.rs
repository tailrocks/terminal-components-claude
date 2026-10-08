//! Painting (`COMPONENT_ARCHITECTURE.md` §5 R3, §17.0 A2, §22.2 items 1–2, 16, 18).
//!
//! Every method clips to the current area and marks the layer's
//! written-cell bitset. Cell, string, and span painters share Ratatui
//! grapheme/width semantics in one writer with reusable inline cluster scratch
//! and a retained heap fallback for oversized clusters;
//! `paint_cell` resets the cells a wide grapheme shadows; `fill` and
//! `dim_layer` are deliberate re-implementations of `ratatui_widgets::{Fill,
//! Dimmed}` because foreign widgets cannot mark the bitset or walk roles.

use ratatui_core::buffer::{Buffer, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use super::{Target, Ui};
use crate::scroll::ScrollState;
use crate::text::Span;
use crate::text::clusters::ClusterFeed;
use crate::text::measure::graphemes;
use crate::theme::builder::{FadeOutcome, fade_mix};
use crate::theme::{ColorLevel, FgStep, GlyphRole, PaintStyle, Role, Surface, Theme};

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

    // Both callers supply their already-clipped row and use the same writer.
    fn paint_graphemes<'s>(
        &mut self,
        area: Rect,
        symbols: impl Iterator<Item = (&'s str, PaintStyle)>,
    ) -> u16 {
        let mut x = area.x;
        let mut remaining = area.width;
        for (symbol, style) in symbols {
            if !self.paint_cluster(&mut x, &mut remaining, area.y, symbol, style) {
                break;
            }
        }
        x.saturating_sub(area.x)
    }

    fn paint_cluster(
        &mut self,
        x: &mut u16,
        remaining: &mut u16,
        y: u16,
        symbol: &str,
        style: PaintStyle,
    ) -> bool {
        if symbol.contains(char::is_control) {
            return true;
        }
        let width = symbol.cell_width();
        if width == 0 {
            return true;
        }
        let Some(rest) = remaining.checked_sub(width) else {
            return false;
        };
        *remaining = rest;
        let pos = Position::new(*x, y);
        if let Some(cell) = self.buffer().cell_mut(pos) {
            cell.set_symbol(symbol).set_style(style.into_style());
        }
        self.mark(pos, Some(style));
        let end = x.saturating_add(width);
        *x = x.saturating_add(1);
        while *x < end {
            let pos = Position::new(*x, y);
            if let Some(cell) = self.buffer().cell_mut(pos) {
                cell.reset();
            }
            self.mark(pos, None);
            *x = x.saturating_add(1);
        }
        *remaining != 0
    }

    /// Fade the viewport edge rows that conceal more scrollable content.
    /// Call after drawing the scroll region, passing its returned content rect.
    /// The scrollbar lies outside that rect. Compatible foregrounds blend on
    /// painted cells with the dominant background; outer rows receive `DIM`
    /// when blending is unavailable. Backgrounds, reversed cells, cursor rows,
    /// and explicitly kept rows retain their original values.
    pub fn scroll_edges(&mut self, area: Rect, state: &ScrollState) {
        self.scroll_edges_except(area, state, &[]);
    }

    /// As [`scroll_edges`](Self::scroll_edges), preserving explicit rows (absolute `y`).
    pub fn scroll_edges_except(&mut self, area: Rect, state: &ScrollState, keep: &[u16]) {
        let area = area.intersection(self.clip);
        if area.is_empty() || area.height < FADE_MIN_ROWS {
            return;
        }
        let up = state.offset() > 0;
        let down = state.viewport_len() > 0
            && state.offset().saturating_add(state.viewport_len()) < state.content_len();
        if !up && !down {
            return;
        }
        let depth = if area.height >= FADE_DEEP_FROM { 2 } else { 1 };
        let container = self.majority_bg(area);
        if up {
            self.fade_unless_protected(area, area.y, FADE_OUTER_KEEP, container, keep);
            if depth == 2 {
                self.fade_unless_protected(
                    area,
                    area.y.saturating_add(1),
                    FADE_INNER_KEEP,
                    container,
                    keep,
                );
            }
        }
        if down {
            self.fade_unless_protected(
                area,
                area.bottom().saturating_sub(1),
                FADE_OUTER_KEEP,
                container,
                keep,
            );
            if depth == 2 {
                self.fade_unless_protected(
                    area,
                    area.bottom().saturating_sub(2),
                    FADE_INNER_KEEP,
                    container,
                    keep,
                );
            }
        }
    }

    fn fade_unless_protected(
        &mut self,
        area: Rect,
        y: u16,
        keep_strength: f32,
        container: Color,
        keep: &[u16],
    ) {
        let protected = keep.contains(&y)
            || self
                .frame
                .cursors
                .iter()
                .any(|cursor| cursor.pos.y == y && area.contains(cursor.pos));
        if !protected {
            self.fade_edge_row(area, y, keep_strength, container);
        }
    }

    fn majority_bg(&mut self, area: Rect) -> Color {
        // Retain the color histogram across frames: steady-state drawing does
        // not allocate, and lookup cost stays bounded for richly styled views.
        let mut counts = core::mem::take(&mut self.core.scroll_bg_counts);
        let mut order = core::mem::take(&mut self.core.scroll_bg_order);
        counts.clear();
        order.clear();
        for pos in area.positions() {
            if !self.cell_written(pos) {
                continue;
            }
            let bg = self.buffer().cell(pos).map_or(Color::Reset, |cell| cell.bg);
            match counts.entry(bg) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let count = entry.get().saturating_add(1);
                    *entry.get_mut() = count;
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    order.push(bg);
                    entry.insert(1);
                }
            }
        }
        // `max_by_key` selects the last entry on ties, preserving the
        // historical rule based on first-seen background order.
        let color = order
            .iter()
            .filter_map(|bg| counts.get(bg).map(|count| (*bg, *count)))
            .max_by_key(|(_, count)| *count)
            .map_or(Color::Reset, |(bg, _)| bg);
        self.core.scroll_bg_counts = counts;
        self.core.scroll_bg_order = order;
        color
    }

    fn fade_edge_row(&mut self, area: Rect, y: u16, keep: f32, container: Color) {
        if let Target::Layer(i) = self.target
            && let Some(d) = self.frame.layers.active_mut().get_mut(i)
        {
            d.fade_rows
                .push((Rect::new(area.x, y, area.width, 1), keep, container));
        }
        let outer = keep <= FADE_OUTER_KEEP;
        // Fading changes only physical cell attributes for this frame. Keep
        // semantic provenance intact for later composition (for example, a
        // modal layer recomputes its dimmed color from roles).
        for x in area.x..area.right() {
            let pos = Position::new(x, y);
            if !self.cell_written(pos) {
                continue;
            }
            if let Some(cell) = self.buffer().cell_mut(pos)
                && cell.bg == container
                && !cell.modifier.contains(Modifier::REVERSED)
            {
                match fade_mix(cell.fg, container, keep) {
                    FadeOutcome::Blended(color) => cell.fg = color,
                    FadeOutcome::ApplyDim if outer => cell.modifier |= Modifier::DIM,
                    FadeOutcome::Unchanged | FadeOutcome::ApplyDim => {}
                }
            }
        }
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

    /// Paint semantic spans, inheriting `base` independently for each span.
    /// Spans form one logical string for grapheme segmentation: a grapheme
    /// split across fragments uses the style from its first byte.
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
        // ASCII graphemes are independent except CRLF, which is itself a
        // skipped control grapheme. Painting each fragment through the
        // shared string writer therefore preserves the logical-line result
        // while avoiding scratch/cursor work on the common label path.
        if spans.iter().all(|span| span.text.is_ascii()) {
            let mut x = area.x;
            for sp in spans {
                if x >= area.right() {
                    break;
                }
                let mut st = base.add_modifier(sp.add).remove_modifier(sp.remove);
                if let Some(role) = sp.role {
                    st = st.patch(self.paint_patch(&crate::theme::StylePatch::new().set_fg(role)));
                }
                x = x.saturating_add(self.paint_str(
                    Rect::new(x, area.y, area.right().saturating_sub(x), 1),
                    sp.text,
                    st,
                ));
            }
            return x.saturating_sub(area.x);
        }
        let mut scratch = core::mem::take(&mut self.core.cluster_scratch);
        let mut x = area.x;
        let mut remaining = area.width;
        {
            let mut feed = ClusterFeed::new(&mut scratch);
            for sp in spans {
                let mut st = base.add_modifier(sp.add).remove_modifier(sp.remove);
                if let Some(role) = sp.role {
                    st = st.patch(self.paint_patch(&crate::theme::StylePatch::new().set_fg(role)));
                }
                if x >= area.right() {
                    break;
                }
                let mut can_continue = true;
                feed.push(sp.text, st, &mut |cluster, style| {
                    can_continue &=
                        self.paint_cluster(&mut x, &mut remaining, area.y, cluster, style);
                    can_continue
                });
                if !can_continue {
                    break;
                }
            }
            if x < area.right() {
                feed.finish(base, &mut |cluster, style| {
                    self.paint_cluster(&mut x, &mut remaining, area.y, cluster, style)
                });
            }
        }
        self.core.cluster_scratch = scratch;
        x.saturating_sub(area.x)
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
        let style = s.into_style();
        {
            let buf = self.buffer();
            for pos in area.positions() {
                if let Some(c) = buf.cell_mut(pos) {
                    c.set_symbol(" ").set_style(style);
                    c.modifier = style.add_modifier;
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

    /// Read-only access to the buffer and the current clip rect without
    /// marking cells as written or erasing semantic roles.
    pub fn peek(&self) -> (&Buffer, Rect) {
        let clip = self.clip;
        let buf: &Buffer = match self.target {
            Target::Page => self.page,
            Target::Layer(i) => match self.frame.layers.active().get(i) {
                Some(d) => &d.buf,
                None => self.page,
            },
        };
        (buf, clip)
    }

    /// Dim the page under a layer by walking the role recorded per painted
    /// cell and stepping it down the foreground ladder semantically
    /// (§54, `docs/design/visual-contract.md` § Modal backdrops). `steps == 0` is identity:
    /// not a restyle to the same colours, but no write at all, so the frame
    /// is byte-identical. One step is `Fg(Secondary)`, two `Fg(Muted)`, three
    /// `Fg(Faint)` and four or more erases the glyph into the resolved
    /// backdrop background; ladder roles start from their own rung and erase
    /// once they step past `Ghost`. Backgrounds resolve from the recorded
    /// background role. All modifiers are cleared. Walks only `area`.
    pub fn dim_layer(&mut self, area: Rect, steps: u8) {
        if steps == 0 {
            return;
        }
        let area = area.intersection(self.frame.screen);
        let theme = self.theme_ref();
        let surface = self.surface;
        let backdrop_text = crate::theme::resolve::bind_role(theme, Role::BackdropFg, surface);
        let backdrop_fill = crate::theme::resolve::bind_role(theme, Role::BackdropBg, surface);
        for pos in area.positions() {
            let roles = self.roles_at(pos);
            let cell_bg = self.page().cell(pos).and_then(|c| c.style().bg);
            let bg = match cell_bg {
                Some(c)
                    if c == theme.bg(Surface::Canvas)
                        || c == theme.bg(Surface::Surface)
                        || c == theme.bg(Surface::Elevated) =>
                {
                    Some(c)
                }
                Some(c) if c == theme.bg(Surface::Field) || c == theme.bg(Surface::FieldHover) => {
                    Some(theme.bg(Surface::Elevated))
                }
                Some(_) => match roles.bg {
                    Some(Role::Surface(s)) => Some(theme.bg(s)),
                    Some(Role::CurrentSurface) => cell_bg,
                    Some(Role::RaisedSurface) => Some(theme.bg(theme.raise(surface))),
                    _ => Some(theme.bg(Surface::Overlay)),
                },
                None => backdrop_fill,
            };
            let fg = if (roles.fg.is_some() && roles.fg == roles.bg)
                || self.page().cell(pos).is_some_and(|c| {
                    (roles.fg.is_some() || roles.bg.is_some())
                        && c.style().fg.is_some()
                        && c.style().fg != Some(Color::Reset)
                        && c.style().fg == c.style().bg
                }) {
                FadeResult::Fg(bg)
            } else if theme.capability.color == ColorLevel::Mono {
                let cell_fg = self.page().cell(pos).and_then(|c| c.style().fg);
                match cell_fg {
                    Some(c) if Some(c) == cell_bg => FadeResult::Fg(bg),
                    Some(c)
                        if c == theme.bg(Surface::Canvas)
                            || c == theme.bg(Surface::Surface)
                            || c == theme.color.surfaces[0]
                            || c == theme.color.surfaces[1]
                            || matches!(c, Color::Black | Color::Indexed(0)) =>
                    {
                        FadeResult::Fg(bg)
                    }
                    Some(c)
                        if c == theme.color.fg[0]
                            || c == theme.color.accent
                            || c == theme.color.danger
                            || c == theme.color.warning
                            || matches!(
                                c,
                                Color::White | Color::Indexed(15) | Color::Gray | Color::Indexed(7)
                            ) =>
                    {
                        FadeResult::Fg(Some(theme.color.fg[2]))
                    }
                    Some(c) if c == theme.color.fg[1] || c == theme.color.on_accent => {
                        FadeResult::Fg(Some(theme.color.fg[3]))
                    }
                    _ => FadeResult::Fg(Some(theme.color.fg[4])),
                }
            } else {
                let cell_fg = self.page().cell(pos).and_then(|c| c.style().fg);
                let was_blended = match roles.fg {
                    Some(Role::Fg(FgStep::Primary)) => {
                        cell_fg.is_some_and(|c| c != theme.color.fg[0])
                    }
                    Some(Role::Fg(FgStep::Secondary)) => {
                        cell_fg.is_some_and(|c| c != theme.color.fg[1])
                    }
                    Some(Role::Success) => cell_fg.is_some_and(|c| c != theme.color.success),
                    Some(Role::Accent) => cell_fg.is_some_and(|c| c != theme.color.accent),
                    Some(Role::Focus) => cell_fg.is_some_and(|c| c != theme.color.focus),
                    Some(Role::Danger) => cell_fg.is_some_and(|c| c != theme.color.danger),
                    Some(Role::Warning) => cell_fg.is_some_and(|c| c != theme.color.warning),
                    _ => false,
                };
                if was_blended {
                    FadeResult::Fg(Some(theme.color.fg[4]))
                } else {
                    match roles.fg {
                        Some(Role::CurrentSurface | Role::RaisedSurface | Role::Surface(_)) => {
                            FadeResult::Fg(bg)
                        }
                        Some(
                            Role::Fg(FgStep::Primary)
                            | Role::Accent
                            | Role::AccentHover
                            | Role::AccentPressed
                            | Role::Focus
                            | Role::Danger
                            | Role::DangerSoft
                            | Role::Warning
                            | Role::Success
                            | Role::Info,
                        ) => ladder(theme, surface, 0, steps),
                        Some(Role::Fg(FgStep::Secondary) | Role::OnAccent | Role::OnDanger) => {
                            ladder(theme, surface, 1, steps)
                        }
                        Some(Role::Fg(FgStep::Ghost)) => ladder(theme, surface, 4, steps),
                        Some(_) => ladder(theme, surface, 2, steps),
                        None => FadeResult::Fg(backdrop_text),
                    }
                }
            };
            let page = self.page_mut();
            if let Some(c) = page.cell_mut(pos) {
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
                c.modifier = Modifier::empty();
            }
        }
    }
}

const FADE_OUTER_KEEP: f32 = 0.55;
const FADE_INNER_KEEP: f32 = 0.8;
const FADE_DEEP_FROM: u16 = 12;
const FADE_MIN_ROWS: u16 = 4;

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
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier, Style};

    use super::super::cx::LastFrame;
    use super::super::{FrameState, Ui, UiCore};
    use crate::scroll::ScrollState;
    use crate::theme::{ColorLevel, FgStep, Role, Surface, Theme};

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

    #[test]
    fn scroll_fade_changes_color_without_changing_role_provenance() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 8, 5);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            let style = ui.paint_patch(
                &crate::theme::StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::CurrentSurface),
            );
            ui.fill(area, style);
            for y in area.y..area.bottom() {
                ui.paint_str(Rect::new(area.x, y, area.width, 1), "abcdefgh", style);
            }
            let edge = Position::new(0, 0);
            let roles = ui.roles_at(edge);
            let mut state = ScrollState::new(100);
            state.set_viewport(5);
            state.scroll_by(1);
            ui.scroll_edges(area, &state);
            assert_eq!(ui.roles_at(edge), roles, "fade is a visual-only pass");
        }
        let edge = page.cell(Position::new(0, 0)).expect("edge cell");
        let middle = page.cell(Position::new(0, 1)).expect("middle cell");
        assert_ne!(edge.fg, middle.fg, "the edge foreground was faded");
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
            // the four non-ladder tones: Secondary, Muted, Faint, Ghost, erase
            for role in [Role::Success, Role::Warning, Role::Danger, Role::Info] {
                for (steps, step) in [
                    (1u8, FgStep::Secondary),
                    (2, FgStep::Muted),
                    (3, FgStep::Faint),
                    (4, FgStep::Ghost),
                ] {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.fg, fg_of(&theme, step), "{role:?} at {steps}");
                    assert_eq!(c.symbol(), "x", "{role:?} at {steps} keeps its glyph");
                }
                for steps in [5u8, 6, 9] {
                    let c = dimmed_cell(&theme, role, "x", Modifier::empty(), steps);
                    assert_eq!(c.symbol(), " ", "{role:?} erases at {steps}");
                    assert_eq!(c.fg, backdrop_bg.expect("backdrop"), "{role:?} at {steps}");
                }
            }
            // every other non-ladder foreground role uses base 2 (Faint at 1, Ghost at 2) —
            // `BorderSubtle` and `DisabledFg` are exactly the two the legacy
            // colour-identity lookup misclassified
            for role in [Role::BorderSubtle, Role::DisabledFg] {
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 1).fg,
                    fg_of(&theme, FgStep::Faint),
                    "{role:?}"
                );
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 2).fg,
                    fg_of(&theme, FgStep::Ghost),
                    "{role:?}"
                );
                assert_eq!(
                    dimmed_cell(&theme, role, "x", Modifier::empty(), 3).symbol(),
                    " ",
                    "{role:?}"
                );
            }
            // `Role::Focus` starts at ladder 0 (matching legacy accent dimming)
            assert_eq!(
                dimmed_cell(&theme, Role::Focus, "x", Modifier::empty(), 1).fg,
                fg_of(&theme, FgStep::Secondary),
                "Role::Focus at 1"
            );
            assert_eq!(
                dimmed_cell(&theme, Role::Focus, "x", Modifier::empty(), 2).fg,
                fg_of(&theme, FgStep::Muted),
                "Role::Focus at 2"
            );
            assert_eq!(
                dimmed_cell(&theme, Role::Focus, "x", Modifier::empty(), 5).symbol(),
                " ",
                "Role::Focus at 5"
            );
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
            // accent degrades through the foreground ladder
            for (start, steps, want) in [
                (Role::Accent, 1u8, FgStep::Secondary),
                (Role::Accent, 2, FgStep::Muted),
                (Role::Accent, 4, FgStep::Ghost),
            ] {
                let c = dimmed_cell(&theme, start, "x", Modifier::empty(), steps);
                assert_eq!(c.fg, fg_of(&theme, want), "{start:?} + {steps}");
                assert_eq!(c.symbol(), "x");
            }
            let c = dimmed_cell(&theme, Role::Accent, "x", Modifier::empty(), 5);
            assert_eq!(c.symbol(), " ", "Accent + 5 erases past the chain");
            assert_eq!(c.fg, backdrop_bg.expect("backdrop"));
            // all modifiers are cleared
            let c = dimmed_cell(
                &theme,
                Role::Fg(FgStep::Primary),
                "x",
                Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED,
                1,
            );
            assert_eq!(c.modifier, Modifier::empty());
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

    #[test]
    fn dim_layer_mono_preserves_baseline_backdrop_fg_resolution() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        for role in [
            Role::Fg(FgStep::Primary),
            Role::Fg(FgStep::Secondary),
            Role::Fg(FgStep::Muted),
            Role::Accent,
            Role::Success,
            Role::Warning,
            Role::Danger,
        ] {
            let c = dimmed_cell(&theme, role, "x", Modifier::empty(), 2);
            assert_eq!(c.fg, Color::Gray, "{role:?} must dim to Gray under Mono");
            assert_eq!(c.symbol(), "x");
        }
    }
}
