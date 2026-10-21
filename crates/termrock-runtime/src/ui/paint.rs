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

    /// Fade an overflowing picker list toward the card, the way option-list
    /// `fade_edges` does.
    ///
    /// A viewport of 12 rows or more fades two rows (keep 0.8, then 0.55).
    /// A shorter viewport fades only the outer row. The blend target is the
    /// majority background of `area` (the card fill). A cursor row is left whole.
    ///
    /// Call this after the rows are painted. `ScrollRegion::draw` paints the
    /// bar first, so a fade there is covered by the rows. The call that belongs
    /// in `crates/termrock-navigation/src/filter_list.rs` `FilterList::draw`,
    /// immediately after `self.draw_rows(ui, content, st, items, live);`:
    ///
    /// ```ignore
    /// let view = ScrollRegion::view(st.core.scroll(), content, visible_len);
    /// ui.fade_picker_overflow(content, &view);
    /// ```
    ///
    /// `content` is the rect `scrollbar().draw` returned (the bar column is
    /// already excluded). `visible_len` is the match count already in scope.
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
                // Preserve the allocation-free fast path for ordinary areas.
                // Once more than eight distinct backgrounds appear, count all
                // remaining colors in UiCore-owned storage and resolve ties by
                // their first-seen order.
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

    /// Space-fill `bg`, preserving the current foreground, underline, and
    /// modifiers. For a layer, those channels are inherited from the live
    /// destination when the layer is composited, so stacked overlays inherit
    /// from the nearest visible lower layer. The supplied raw background has
    /// no semantic role and replaces any existing background role.
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
                    if let Some(cell) = self.buffer().cell_mut(pos) {
                        cell.reset();
                        cell.set_symbol(" ");
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

    /// Dim a modal underlay with the active theme's shared backdrop mapping.
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

    /// Clear `dim` without changing other modifiers.
    pub fn clear_dim(&mut self, area: Rect) {
        let area = area.intersection(self.frame.screen);
        let page = &mut *self.page;
        for pos in area.positions() {
            if let Some(cell) = page.cell_mut(pos) {
                cell.modifier.remove(Modifier::DIM);
            }
        }
    }

    /// Set `dim` without clearing bold or other modifiers.
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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::{Buffer, CellDiffOption};
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::{Color, Modifier, Style};

    use super::super::cx::LastFrame;
    use super::super::{CellRoles, FrameState, Ui, UiCore};
    use crate::id::Id;
    use crate::layer::{Anchor, LayerId, LayerSpec};
    use crate::scroll::ScrollState;
    use crate::theme::{FgStep, PaintStyle, Role, StylePatch, Surface, Theme};

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
    fn underlay_fill_defers_ink_inheritance_until_composition() {
        const POPUP: Id = Id::root("paint.underlay-fill");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame.layers.begin();
        frame.layers.push(
            POPUP,
            LayerId(1),
            LayerSpec::popover(POPUP, Anchor::Point(Position::new(2, 0))),
            Rect::new(2, 0, 3, 1),
            SCREEN,
        );
        frame.top = LayerId(1);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let target = Position::new(4, 0);
        let before_roles;
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            let underlay = ui
                .paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .set_bg(Role::CurrentSurface),
                )
                .underline_color(Color::Green)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD | Modifier::ITALIC);
            ui.fill(SCREEN, underlay);
            ui.paint_str(Rect::new(1, 0, 2, 1), "界", underlay);
            if let Some(cell) = ui.page_mut().cell_mut(target) {
                cell.set_diff_option(CellDiffOption::Skip);
            }
            let reset_target = Position::new(4, 0);
            if let Some(cell) = ui.page_mut().cell_mut(reset_target) {
                cell.fg = Color::Reset;
                cell.underline_color = Color::Reset;
                cell.modifier = Modifier::empty();
            }
            before_roles = ui.roles_at(target);
            let _ = ui.layer(POPUP, |ui, area| {
                ui.fill_keep_ink(area, Color::Black);
                ui.patch_style(
                    Rect::new(3, 0, 1, 1),
                    PaintStyle::new(),
                    StylePatch {
                        underline: crate::theme::Slot::Clear,
                        remove: Modifier::BOLD,
                        ..StylePatch::new()
                    },
                );
            });
        }

        let Some(layer) = frame.layers.active().first() else {
            panic!("underlay layer was registered");
        };
        let Some(cell) = layer.buf.cell(target) else {
            panic!("filled cell exists");
        };
        assert_eq!(cell.symbol(), " ");
        assert_eq!(
            cell.fg,
            Color::Reset,
            "layer closure must not sample the page"
        );
        assert_eq!(cell.bg, Color::Black);
        assert_eq!(cell.underline_color, Color::Reset);
        assert_eq!(cell.modifier, Modifier::empty());
        assert_eq!(layer.roles_at(target), CellRoles::default());
        assert_eq!(cell.diff_option, CellDiffOption::None);
        let Some(reset_cell) = layer.buf.cell(Position::new(4, 0)) else {
            panic!("last filled cell exists");
        };
        assert_eq!(reset_cell.symbol(), " ");
        assert_eq!(reset_cell.fg, Color::Reset);
        assert_eq!(reset_cell.bg, Color::Black);
        assert_eq!(reset_cell.underline_color, Color::Reset);
        assert_eq!(reset_cell.modifier, Modifier::empty());
        assert_eq!(
            page.cell(Position::new(1, 0)).map(|cell| cell.symbol()),
            Some("界"),
            "a partial wide grapheme stays outside the clipped layer buffer"
        );
        assert!(layer.is_written(Position::new(4, 0)));
        assert!(
            !layer.is_written(Position::new(5, 0)),
            "filling must not expand beyond the layer's clipped area"
        );

        let expected_ink = page
            .cell(target)
            .map(|cell| (cell.fg, cell.underline_color, cell.modifier));
        layer.composite_onto(&mut page, &mut frame.roles);
        let composed = page.cell(target).expect("composited cell is present");
        assert_eq!(
            Some((composed.fg, composed.underline_color, composed.modifier)),
            expected_ink
        );
        assert_eq!(composed.bg, Color::Black);
        assert_eq!(
            frame.roles.get(frame.role_index(target).unwrap()),
            Some(&CellRoles {
                fg: before_roles.fg,
                bg: None,
            })
        );
        let patched = page.cell(Position::new(3, 0)).expect("patched cell exists");
        assert_eq!(patched.underline_color, Color::Reset);
        assert!(!patched.modifier.contains(Modifier::BOLD));
        assert!(patched.modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn move_cell_transfers_underlay_intent_with_the_moved_cell() {
        const POPUP: Id = Id::root("paint.underlay-move");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame.layers.begin();
        frame.layers.push(
            POPUP,
            LayerId(1),
            LayerSpec::popover(POPUP, Anchor::Point(Position::new(0, 0))),
            SCREEN,
            SCREEN,
        );
        frame.top = LayerId(1);
        let mut page = Buffer::empty(SCREEN);
        page.set_style(SCREEN, Style::new().fg(Color::Blue));
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let source = Position::new(1, 0);
        let destination = Position::new(3, 0);
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            let _ = ui.layer(POPUP, |ui, area| {
                ui.fill_keep_ink(area, Color::Black);
                ui.paint_str(
                    Rect::new(source.x, source.y, 1, 1),
                    "X",
                    Style::new().fg(Color::Yellow),
                );
                ui.move_cell(source, destination, PaintStyle::new());
            });
        }
        let layer = frame.layers.active().first().expect("popup exists");
        layer.composite_onto(&mut page, &mut frame.roles);
        let moved = page.cell(destination).expect("destination exists");
        assert_eq!(moved.symbol(), "X");
        assert_eq!(moved.fg, Color::Yellow);
        assert_eq!(moved.bg, Color::Black);
        let reset = page.cell(source).expect("source exists");
        assert_eq!(reset.symbol(), " ");
        assert_eq!(reset.fg, Color::Yellow);
        assert_eq!(reset.bg, Color::Black);
    }

    #[test]
    fn modal_backdrop_uses_theme_mapping_and_clears_only_focus_glyph() {
        use crate::theme::{ColorLevel, GlyphRole};

        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut theme = Theme::junie();
            theme.capability.color = level;
            theme.color.fg[2] = theme.color.surfaces[3];
            let source = Style::new()
                .fg(theme.color.fg[0])
                .bg(Color::Rgb(250, 250, 250))
                .add_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED);
            let blank = Style::new()
                .fg(Color::Black)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD);
            let focus_bar = theme.design.glyphs.get(GlyphRole::FocusBar);
            let unusual_bar = Style::new()
                .fg(Color::Rgb(9, 10, 11))
                .bg(theme.color.surfaces[1]);
            let (_, page) = with_ui(&theme, |ui| {
                ui.paint_cell(Position::new(0, 0), "x", source);
                ui.paint_cell(Position::new(1, 0), " ", blank);
                ui.paint_cell(Position::new(2, 0), focus_bar, unusual_bar);
                ui.modal_backdrop(SCREEN);
            });

            let original = page.cell(Position::new(0, 0)).expect("painted text");
            let (expected_fg, expected_bg) = theme.backdrop_colors(
                source.fg.unwrap_or(Color::Reset),
                source.bg.unwrap_or(Color::Reset),
            );
            assert_eq!(original.symbol(), "x", "{level:?}");
            assert_eq!(original.fg, expected_fg, "{level:?}");
            assert_eq!(original.bg, expected_bg, "{level:?}");
            assert_eq!(original.modifier, Modifier::DIM, "{level:?}");
            assert_eq!(
                original.fg, theme.color.fg[0],
                "muted==background falls back to primary"
            );

            let blank = page.cell(Position::new(1, 0)).expect("painted blank");
            assert_eq!(blank.symbol(), " ");
            assert_eq!(blank.modifier, Modifier::empty(), "blank cells do not dim");

            let bar = page.cell(Position::new(2, 0)).expect("painted focus glyph");
            assert_eq!(
                bar.symbol(),
                " ",
                "a non-ASCII focus glyph clears independent of fg"
            );
        }
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

    #[test]
    fn picker_overflow_fade_blends_the_last_two_rows_toward_the_card() {
        let theme = Theme::junie();
        let card = Color::Rgb(0x18, 0x18, 0x1b);
        let label = Color::Rgb(0xff, 0xff, 0xff);
        let meta = Color::Rgb(0x8a, 0x8a, 0x8a);
        let height = 13u16;
        let area = Rect::new(0, 0, 4, height);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            let fill = Style::new().bg(card);
            ui.fill(area, fill);
            for y in area.y..area.bottom() {
                ui.paint_str(
                    Rect::new(area.x, y, 1, 1),
                    "a",
                    Style::new().fg(label).bg(card),
                );
                ui.paint_str(
                    Rect::new(area.x + 1, y, 1, 1),
                    "V",
                    Style::new().fg(meta).bg(card),
                );
            }
            let mut state = ScrollState::new(40);
            state.set_viewport(usize::from(height));
            ui.fade_picker_overflow(area, &state);
        }
        let fg = |x, y| page.cell(Position::new(x, y)).expect("cell").fg;
        assert_eq!(
            fg(0, 0),
            label,
            "the top stays whole when nothing is hidden above"
        );
        assert_eq!(fg(0, height - 3), label, "only the last two rows fade");
        assert_eq!(fg(0, height - 2), Color::Rgb(0xd1, 0xd1, 0xd1));
        assert_eq!(fg(1, height - 2), Color::Rgb(0x73, 0x73, 0x74));
        assert_eq!(fg(0, height - 1), Color::Rgb(0x97, 0x97, 0x98));
        assert_eq!(fg(1, height - 1), Color::Rgb(0x57, 0x57, 0x58));
    }

    #[test]
    fn picker_overflow_fade_uses_area_height_when_the_viewport_is_unlaid() {
        let theme = Theme::junie();
        let card = Color::Rgb(0x18, 0x18, 0x1b);
        let label = Color::Rgb(0xff, 0xff, 0xff);
        let area = Rect::new(0, 0, 2, 13);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.fill(area, Style::new().fg(label).bg(card));
            for y in area.y..area.bottom() {
                ui.paint_str(
                    Rect::new(area.x, y, area.width, 1),
                    "ab",
                    Style::new().fg(label).bg(card),
                );
            }
            ui.fade_picker_overflow(area, &ScrollState::new(40));
        }
        let bottom = page
            .cell(Position::new(0, area.bottom() - 1))
            .expect("outer")
            .fg;
        assert_eq!(bottom, Color::Rgb(0x97, 0x97, 0x98));
    }

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

    #[test]
    fn dim_layer_semantic_roles_step_monotonically_and_erase() {
        for theme in [Theme::junie(), Theme::paper()] {
            let backdrop_bg =
                crate::theme::resolve::bind_role(&theme, Role::BackdropBg, Surface::Canvas);

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

            let c = dimmed_cell(&theme, Role::Fg(FgStep::Primary), "x", Modifier::empty(), 1);
            assert_eq!(c.bg, theme.bg(Surface::Canvas));
        }
    }
}
