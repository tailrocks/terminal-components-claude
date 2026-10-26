use ratatui_core::buffer::{Buffer, CellDiffOption, CellWidth};
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use super::{Target, Ui};
use crate::scroll::ScrollState;
use crate::text::Span;
use crate::text::clusters::ClusterFeed;
use crate::text::measure::graphemes;
use crate::theme::builder::{FadeOutcome, fade_mix};
use crate::theme::{FgStep, GlyphRole, PaintStyle, Role, Slot, StylePatch, Surface, Theme};

impl Ui<'_> {
    pub(crate) fn patch_style(&mut self, area: Rect, style: PaintStyle, explicit: StylePatch) {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        let paint = style;
        let style = paint.into_style();
        for pos in area.positions() {
            let Some(cell) = self.buffer().cell_mut(pos) else {
                continue;
            };
            match explicit.fg {
                Slot::Inherit => {}
                Slot::Set(_) => cell.fg = style.fg.unwrap_or(Color::Reset),
                Slot::Clear => cell.fg = Color::Reset,
            }
            match explicit.bg {
                Slot::Inherit => {}
                Slot::Set(_) => cell.bg = style.bg.unwrap_or(Color::Reset),
                Slot::Clear => cell.bg = Color::Reset,
            }
            match explicit.underline {
                Slot::Inherit => {}
                Slot::Set(_) => {
                    cell.underline_color = style.underline_color.unwrap_or(Color::Reset)
                }
                Slot::Clear => cell.underline_color = Color::Reset,
            }
            cell.modifier = cell
                .modifier
                .difference(explicit.remove)
                .union(explicit.add);
            if let Target::Layer(index) = self.target
                && let Some(layer) = self.frame.layers.active_mut().get_mut(index)
            {
                layer.patch_underlay(pos, paint, explicit);
            }
        }
    }

    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: impl Into<PaintStyle>) {
        if self.clip.contains(pos) {
            self.paint_str(
                Rect::new(pos.x, pos.y, self.clip.right().saturating_sub(pos.x), 1),
                symbol,
                s,
            );
        }
    }

    pub fn paint_str(&mut self, area: Rect, text: &str, s: impl Into<PaintStyle>) -> u16 {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return 0;
        }
        let s = s.into();
        self.paint_graphemes(area, graphemes(text).map(|(_, symbol)| (symbol, s)))
    }

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

    pub fn scroll_edges(&mut self, area: Rect, state: &ScrollState) {
        self.scroll_edges_except(area, state, &[]);
    }

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

    pub fn fade_picker_overflow(&mut self, area: Rect, state: &ScrollState) {
        let mut laid_out = *state;
        if laid_out.viewport_len() == 0 {
            laid_out.set_viewport(usize::from(area.height));
        }
        self.scroll_edges(area, &laid_out);
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
        let mut entries: [(Color, usize); 8] = [(Color::Reset, 0); 8];
        let mut count_entries = 0;
        let mut overflowed = false;
        for pos in area.positions() {
            if !self.cell_written(pos) {
                continue;
            }
            let bg = self.buffer().cell(pos).map_or(Color::Reset, |cell| cell.bg);

            if overflowed {
                let order = self.core.majority_bg_counts.len();
                self.core
                    .majority_bg_counts
                    .entry(bg)
                    .and_modify(|(count, _)| *count = count.saturating_add(1))
                    .or_insert((1, order));
                continue;
            }

            if let Some(entry) = entries[..count_entries]
                .iter_mut()
                .find(|entry| entry.0 == bg)
            {
                entry.1 = entry.1.saturating_add(1);
            } else if count_entries < entries.len() {
                entries[count_entries] = (bg, 1);
                count_entries += 1;
            } else {
                let counts = &mut self.core.majority_bg_counts;
                counts.clear();
                for (order, (color, count)) in entries[..count_entries].iter().enumerate() {
                    counts.insert(*color, (*count, order));
                }
                counts.insert(bg, (1, count_entries));
                overflowed = true;
            }
        }

        if overflowed {
            self.core
                .majority_bg_counts
                .iter()
                .max_by_key(|(_, (count, order))| (*count, *order))
                .map_or(Color::Reset, |(bg, _)| *bg)
        } else {
            entries[..count_entries]
                .iter()
                .max_by_key(|(_, count)| *count)
                .map_or(Color::Reset, |(bg, _)| *bg)
        }
    }

    fn fade_edge_row(&mut self, area: Rect, y: u16, keep: f32, container: Color) {
        let outer = keep <= FADE_OUTER_KEEP;

        for x in area.x..area.right() {
            let pos = Position::new(x, y);
            if !self.cell_written(pos) {
                continue;
            }
            if let Target::Layer(index) = self.target
                && let Some(layer) = self.frame.layers.active_mut().get_mut(index)
                && layer.has_underlay(pos)
            {
                layer.defer_fade(pos, container, keep, outer);
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

        if spans.iter().all(|span| span.text.is_ascii()) {
            let mut x = area.x;
            for sp in spans {
                if x >= area.right() {
                    break;
                }
                let mut st = base.add_modifier(sp.add);
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
                let mut st = base.add_modifier(sp.add);
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

    pub fn paint_style(&mut self, area: Rect, s: impl Into<PaintStyle>) {
        let s = s.into();
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        self.buffer().set_style(area, s.into_style());
        self.mark_area(area, Some(s));
    }

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

    pub fn fill_keep_ink(&mut self, area: Rect, bg: Color) {
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        for pos in area.positions() {
            match self.target {
                Target::Page => {
                    let Some((fg, underline_color, modifier)) = self
                        .page
                        .cell(pos)
                        .map(|cell| (cell.fg, cell.underline_color, cell.modifier))
                    else {
                        continue;
                    };
                    if let Some(cell) = self.page.cell_mut(pos) {
                        cell.reset();
                        cell.set_symbol(" ");
                        cell.fg = fg;
                        cell.bg = bg;
                        cell.underline_color = underline_color;
                        cell.modifier = modifier;
                        cell.set_diff_option(CellDiffOption::None);
                    }
                    self.mark(pos, Some(PaintStyle::new().bg(bg)));
                }
                Target::Layer(index) => {
                    let ink = self
                        .page
                        .cell(pos)
                        .map(|cell| (cell.fg, cell.underline_color, cell.modifier));
                    if let Some(cell) = self.buffer().cell_mut(pos) {
                        cell.reset();
                        cell.set_symbol(" ");
                        if let Some((fg, underline_color, modifier)) = ink {
                            cell.fg = fg;
                            cell.underline_color = underline_color;
                            cell.modifier = modifier;
                        }
                        cell.bg = bg;
                        cell.set_diff_option(CellDiffOption::None);
                    }
                    if let Some(layer) = self.frame.layers.active_mut().get_mut(index) {
                        layer.begin_underlay(pos, bg);
                    }
                }
            }
        }
    }

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

    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: impl Into<PaintStyle>) -> u16 {
        let s = s.into();
        let sym = self.theme_ref().design.glyphs.get(g);
        self.paint_str(area, sym, s)
    }

    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let clip = self.clip;
        self.mark_area(clip, None);
        (self.buffer(), clip)
    }

    pub fn modal_backdrop(&mut self, area: Rect) {
        let area = area.intersection(self.frame.screen);
        let theme = self.theme;
        let focus_bar = theme.design.glyphs.get(crate::theme::GlyphRole::FocusBar);
        let page = &mut *self.page;
        for pos in area.positions() {
            let Some(cell) = page.cell_mut(pos) else {
                continue;
            };
            let clear_bar = cell.symbol() == focus_bar
                && (!focus_bar.is_ascii()
                    || cell.fg == theme.color.focus
                    || cell.fg == theme.color.on_accent);
            let backdrop = theme.backdrop_style(cell.style());
            cell.set_style(backdrop);
            cell.modifier = backdrop.add_modifier;
            if clear_bar {
                cell.set_symbol(" ");
            }
        }
    }

    pub fn clear_dim(&mut self, area: Rect) {
        let area = area.intersection(self.frame.screen);
        let page = &mut *self.page;
        for pos in area.positions() {
            if let Some(cell) = page.cell_mut(pos) {
                cell.modifier.remove(Modifier::DIM);
            }
        }
    }

    pub fn add_dim(&mut self, area: Rect) {
        let area = area.intersection(self.frame.screen);
        let page = &mut *self.page;
        for pos in area.positions() {
            if let Some(cell) = page.cell_mut(pos) {
                cell.modifier.insert(Modifier::DIM);
            }
        }
    }

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
            let fg = match roles.fg {
                Some(Role::Fg(step)) => ladder(theme, surface, step.index(), steps),

                Some(Role::Accent) => accent(theme, surface, 0, steps),
                Some(Role::AccentHover) => accent(theme, surface, 1, steps),
                Some(Role::AccentPressed) => accent(theme, surface, 2, steps),

                Some(Role::CurrentSurface | Role::RaisedSurface | Role::Surface(_)) => {
                    FadeResult::Fg(None)
                }

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

const FADE_OUTER_KEEP: f32 = 0.55;
const FADE_INNER_KEEP: f32 = 0.8;
const FADE_DEEP_FROM: u16 = 12;
const FADE_MIN_ROWS: u16 = 4;

enum FadeResult {
    Fg(Option<Color>),
    Erase,
}

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
