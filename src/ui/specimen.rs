//! Widgets drawn in a forced state, and the States grid that lays them out.
//!
//! A [`Specimen`] is a widget that can draw a copy of itself in any
//! [`SpecimenState`] through its own `render`: focus, hover and press come
//! from a forced [`Interaction`], the widget's own flags (disabled, busy,
//! error, editing, ...) from the specimen implementation. A [`StateGrid`]
//! lays specimens out as rows = states and columns = variants, so every
//! state of a widget shows at once, exactly as the widget draws it.
//!
//! # One focus indicator
//!
//! A sample in the `focused` row would draw the live focus bar
//! (`Theme::glyphs.focus_bar`, `▎`), and the screen would then show a second
//! "you are here" next to the real one. The grid therefore depicts focus
//! instead of showing it: every focus-bar cell a sample draws is replaced
//! with [`StateGrid::focus_mark`] (default `▏`, `:` under ASCII glyphs) in
//! the muted ink (a filled control keeps its own ink so the mark stays
//! legible on the fill). The mark has a different shape from the live bar,
//! so it reads as a depiction at every colour level, monochrome included.
//! The live bar stays the only `▎` on screen.
//!
//! The same holds for the editing caret: samples are inert, so they never
//! move the terminal cursor; an editing sample draws a static caret cell.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};

use crate::core::focus::FocusRing;
use crate::core::hit::HitRegistry;
use crate::core::id::WidgetId;
use crate::theme::{ColorLevel, Theme};
use crate::ui::ctx::{Interaction, RenderCtx, VisualState};

/// A state a [`Specimen`] can be drawn in: one row of a [`StateGrid`].
///
/// Interaction states (hover, press, focus) are forced through the
/// render context; the rest are the widget's own flags or content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecimenState {
    /// No interaction.
    Rest,
    /// The pointer is over it.
    Hover,
    /// The primary button is held on it.
    Pressed,
    /// It holds keyboard focus (depicted, see the module docs).
    Focused,
    /// Focused with the pointer over it.
    FocusedHover,
    /// Selected or on (a toggle that is on).
    Selected,
    /// Drawn but not reachable or activatable.
    Disabled,
    /// Working: a spinner instead of the action.
    Busy,
    /// Carries a validation error.
    Error,
    /// Editing its value (the caret is shown).
    Editing,
    /// No content (a field's placeholder, an empty list).
    Empty,
    /// Content is still arriving.
    Loading,
    /// Content longer than the room it has.
    Overflow,
    /// Drawn in the grid's narrow slot ([`StateGrid::narrow_width`]).
    Narrow,
}

impl SpecimenState {
    /// Every state, in the order a grid lists them.
    pub const ALL: [Self; 14] = [
        Self::Rest,
        Self::Hover,
        Self::Pressed,
        Self::Focused,
        Self::FocusedHover,
        Self::Selected,
        Self::Disabled,
        Self::Busy,
        Self::Error,
        Self::Editing,
        Self::Empty,
        Self::Loading,
        Self::Overflow,
        Self::Narrow,
    ];

    /// Stable name: the catalog's `state::<slug>` and the showcase's
    /// States-row declarations use it.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Rest => "rest",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Focused => "focused",
            Self::FocusedHover => "focused_hover",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::Busy => "busy",
            Self::Error => "error",
            Self::Editing => "editing",
            Self::Empty => "empty",
            Self::Loading => "loading",
            Self::Overflow => "overflow",
            Self::Narrow => "narrow",
        }
    }

    /// The row label a grid draws.
    pub fn label(self) -> &'static str {
        match self {
            Self::FocusedHover => "focus + hover",
            other => other.slug(),
        }
    }

    /// The state from its [`Self::slug`].
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.slug() == slug)
    }

    /// The visual flags this state forces. Content states (empty, loading,
    /// overflow, narrow) force none: the specimen changes its content.
    pub fn visual(self) -> VisualState {
        let mut v = VisualState::default();
        match self {
            Self::Hover => v.hovered = true,
            // a pointer press is always over the control
            Self::Pressed => {
                v.hovered = true;
                v.pressed = true;
            }
            Self::Focused => v.focused = true,
            Self::FocusedHover => {
                v.focused = true;
                v.hovered = true;
            }
            Self::Selected => v.selected = true,
            Self::Disabled => v.disabled = true,
            Self::Busy => v.busy = true,
            Self::Error => v.error = true,
            Self::Editing => {
                v.focused = true;
                v.editing = true;
            }
            Self::Rest | Self::Empty | Self::Loading | Self::Overflow | Self::Narrow => {}
        }
        v
    }

    /// The forced interaction that makes `id` render in this state.
    fn interaction(self, id: WidgetId, tick: u64) -> Interaction {
        let v = self.visual();
        Interaction {
            focus: v.focused.then_some(id),
            hover: v.hovered.then_some(id),
            pressed: v.pressed.then_some(id),
            tick,
            ..Default::default()
        }
    }
}

/// A widget that can draw itself in a forced [`SpecimenState`].
pub trait Specimen {
    /// The id the forced interaction points at (the widget's own id).
    fn specimen_id(&self) -> WidgetId;
    /// Whether the widget has a look for `state`. A grid cell for an
    /// unsupported state shows a faint `–`.
    fn supports(&self, state: SpecimenState) -> bool;
    /// Rows the sample takes in `state`.
    fn specimen_height(&self, _state: SpecimenState) -> u16 {
        1
    }
    /// Draw a copy of the widget in `state` into `area`. `ctx` is inert
    /// and already forces focus, hover and press on [`Self::specimen_id`];
    /// the implementation sets the widget's own flags and content.
    fn specimen(
        &self,
        state: SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    );
}

/// The four colour levels "Compare levels" shows.
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

/// Rows = states, columns = variants: a static reference of every state of
/// a widget, drawn by the widget itself ([`Specimen`]).
///
/// It is not interactive: samples register no hit region, focus stop or
/// cursor, and the focus look is depicted, never live (module docs).
pub struct StateGrid {
    pub id: WidgetId,
    /// The rows, top to bottom.
    pub rows: Vec<SpecimenState>,
    columns: Vec<(String, Box<dyn Specimen>)>,
    /// Width of the row-label column. Default 15.
    pub label_width: u16,
    /// Width of each variant column, its gap included. Default 15.
    pub column_width: u16,
    /// Width of the slot the [`SpecimenState::Narrow`] row draws in
    /// (capped by [`Self::column_width`]). Default 24.
    pub narrow_width: u16,
    /// The glyph that depicts a focus bar. `None` (default): `▏`, or `:`
    /// when the theme's focus bar is ASCII.
    pub focus_mark: Option<&'static str>,
    /// Rows drawn whole by the last render.
    rendered: Vec<SpecimenState>,
}

impl std::fmt::Debug for StateGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateGrid")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .field(
                "columns",
                &self.columns.iter().map(|(l, _)| l).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl StateGrid {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            rows: Vec::new(),
            columns: Vec::new(),
            label_width: 15,
            column_width: 15,
            narrow_width: 24,
            focus_mark: None,
            rendered: Vec::new(),
        }
    }

    /// Append one row.
    pub fn row(mut self, state: SpecimenState) -> Self {
        self.rows.push(state);
        self
    }

    /// Append rows.
    pub fn rows(mut self, states: &[SpecimenState]) -> Self {
        self.rows.extend_from_slice(states);
        self
    }

    /// Append a variant column headed `label`.
    pub fn column(mut self, label: &str, specimen: impl Specimen + 'static) -> Self {
        self.columns.push((label.to_owned(), Box::new(specimen)));
        self
    }

    pub fn label_width(mut self, w: u16) -> Self {
        self.label_width = w;
        self
    }

    pub fn column_width(mut self, w: u16) -> Self {
        self.column_width = w;
        self
    }

    pub fn narrow_width(mut self, w: u16) -> Self {
        self.narrow_width = w;
        self
    }

    pub fn focus_mark(mut self, glyph: &'static str) -> Self {
        self.focus_mark = Some(glyph);
        self
    }

    /// The column labels, left to right.
    pub fn column_labels(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().map(|(l, _)| l.as_str())
    }

    /// Rows the last render drew whole (both renders record them).
    pub fn rendered_rows(&self) -> &[SpecimenState] {
        &self.rendered
    }

    fn row_height(&self, state: SpecimenState) -> u16 {
        self.columns
            .iter()
            .filter(|(_, s)| s.supports(state))
            .map(|(_, s)| s.specimen_height(state))
            .max()
            .unwrap_or(1)
            .max(1)
    }

    /// Natural size: the label column, every variant column, a header row
    /// and every row.
    pub fn size(&self) -> (u16, u16) {
        let w = self.label_width + self.column_width * self.columns.len() as u16;
        let h = 1 + self.rows.iter().map(|&r| self.row_height(r)).sum::<u16>();
        (w, h)
    }

    /// The glyph that depicts a focus bar under `t`.
    fn mark(&self, t: &Theme) -> &'static str {
        self.focus_mark.unwrap_or(if t.glyphs.focus_bar.is_ascii() {
            ":"
        } else {
            "▏"
        })
    }

    /// Draw the grid at the context's colour level.
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.rendered.clear();
        let tick = ctx.interaction.tick;
        let theme = ctx.theme;
        self.draw(theme, tick, area, buf, bg);
    }

    /// Height of [`Self::render_levels`] in `width` columns.
    pub fn levels_height(&self, width: u16) -> u16 {
        let (w, h) = self.size();
        let per_row = Self::per_row(w, width);
        let bands = LEVELS.len().div_ceil(per_row) as u16;
        // a title row per pane, one blank row between bands
        bands * (h + 1) + bands.saturating_sub(1)
    }

    /// Panes side by side in `width`: all four, two, or one.
    fn per_row(grid_w: u16, width: u16) -> usize {
        [4usize, 2]
            .into_iter()
            .find(|&n| grid_w * n as u16 + 2 * (n as u16 - 1) <= width)
            .unwrap_or(1)
    }

    /// "Compare levels": the grid at every colour level (truecolor, 256,
    /// 16, none), each pane on its own level's surface under its level
    /// name. Panes sit side by side when they fit (four, then two) and
    /// stack otherwise; whatever does not fit `area` is cut.
    pub fn render_levels(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.rendered.clear();
        let tick = ctx.interaction.tick;
        let (w, h) = self.size();
        let per_row = Self::per_row(w, area.width);
        let area = area.intersection(buf.area);
        for (i, level) in LEVELS.into_iter().enumerate() {
            let col = (i % per_row) as u16;
            let band = (i / per_row) as u16;
            let x = area.x + col * (w + 2);
            let y = area.y + band * (h + 2);
            let pane =
                Rect::new(x, y, w.min(area.right().saturating_sub(x)), h + 1).intersection(area);
            if pane.is_empty() {
                continue;
            }
            let own;
            let t: &Theme = if ctx.theme.level == level {
                ctx.theme
            } else {
                own = Theme::for_level(level);
                &own
            };
            let bg = t.surface;
            crate::ui::ctx::fill(buf, pane, Style::new().bg(bg));
            buf.set_string(pane.x, pane.y, level.label(), t.muted().bg(bg));
            let body = Rect::new(
                pane.x,
                pane.y + 1,
                pane.width,
                pane.height.saturating_sub(1),
            );
            let before = self.rendered.clone();
            self.rendered.clear();
            self.draw(t, tick, body, buf, bg);
            // a row counts when every pane drew it
            if i > 0 {
                self.rendered.retain(|r| before.contains(r));
            }
        }
    }

    fn draw(&mut self, t: &Theme, tick: u64, area: Rect, buf: &mut Buffer, bg: Color) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let mark = self.mark(t);
        let col_x = |k: usize| area.x + self.label_width + k as u16 * self.column_width;
        let fits = |k: usize| col_x(k) + self.column_width <= area.right() + 1;
        for (k, (label, _)) in self.columns.iter().enumerate() {
            if !fits(k) {
                break;
            }
            let room = self.column_width.saturating_sub(1) as usize;
            buf.set_string(
                col_x(k),
                area.y,
                crate::ui::text::fit(label, room),
                t.muted().bg(bg),
            );
        }
        let mut y = area.y + 1;
        for &state in &self.rows {
            let h = self.row_height(state);
            if y + h > area.bottom() {
                break;
            }
            buf.set_string(
                area.x,
                y,
                crate::ui::text::fit(state.label(), self.label_width.saturating_sub(1) as usize),
                t.secondary().bg(bg),
            );
            for (k, (_, specimen)) in self.columns.iter().enumerate() {
                if !fits(k) {
                    break;
                }
                let mut w = self.column_width.saturating_sub(1);
                if state == SpecimenState::Narrow {
                    w = w.min(self.narrow_width);
                }
                let cell = Rect::new(col_x(k), y, w, h).intersection(area);
                if !specimen.supports(state) {
                    buf.set_string(cell.x + 1, cell.y, "–", t.faint().bg(bg));
                    continue;
                }
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let interaction = state.interaction(specimen.specimen_id(), tick);
                let mut sctx = RenderCtx::detached(t, interaction, &mut hits, &mut ring);
                specimen.specimen(state, cell, buf, &mut sctx, bg);
                depict_focus(buf, cell, t, mark, bg);
            }
            self.rendered.push(state);
            y += h;
        }
    }
}

/// Replace every live focus-bar cell in `area` with `mark` (module docs).
fn depict_focus(buf: &mut Buffer, area: Rect, t: &Theme, mark: &str, bg: Color) {
    let bar = t.glyphs.focus_bar;
    for pos in area.positions() {
        let Some(c) = buf.cell_mut(pos) else {
            continue;
        };
        // an ASCII bar (`|`) is ordinary text too: only the focus inks mark it
        let live =
            c.symbol() == bar && (!bar.is_ascii() || c.fg == t.focus || c.fg == t.text_on_accent);
        if !live {
            continue;
        }
        c.set_symbol(mark);
        if c.bg == bg {
            c.fg = t.text_muted;
        }
    }
}

impl Specimen for crate::widgets::button::Button {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: SpecimenState) -> bool {
        use SpecimenState::*;
        match state {
            Rest | Hover | Pressed | Focused | FocusedHover | Disabled | Busy | Overflow => true,
            Selected => self.on.is_some(),
            Error | Editing | Empty | Loading | Narrow => false,
        }
    }

    fn specimen(
        &self,
        state: SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut b = self.clone();
        b.tip = None;
        let v = state.visual();
        b.disabled |= v.disabled;
        b.busy |= v.busy;
        if v.selected {
            b.on = Some(true);
        }
        if state == SpecimenState::Overflow {
            b.label = format!("{} with a long label", b.label);
        }
        b.render(area, buf, ctx, bg);
    }
}

impl Specimen for crate::widgets::input::TextInput {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: SpecimenState) -> bool {
        use SpecimenState::*;
        match state {
            Rest | Hover | Focused | FocusedHover | Editing | Error | Disabled | Empty
            | Overflow | Narrow => true,
            Pressed | Selected | Busy | Loading => false,
        }
    }

    /// The label and the field; an error adds its message row.
    fn specimen_height(&self, state: SpecimenState) -> u16 {
        if state == SpecimenState::Error || self.error.is_some() {
            3
        } else {
            2
        }
    }

    fn specimen(
        &self,
        state: SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut f = self.clone();
        f.help.clear();
        let v = state.visual();
        f.disabled |= v.disabled;
        f.editing = v.editing;
        if v.error && f.error.is_none() {
            f.error = Some("Enter a valid value".into());
        }
        match state {
            SpecimenState::Empty => {
                if f.placeholder.is_empty() {
                    f.placeholder = "Type a value…".into();
                }
                f = f.value("");
            }
            SpecimenState::Overflow => {
                let long = format!("{} and a value longer than the field", f.buffer.text());
                f = f.value(&long);
            }
            _ => {}
        }
        f.render(area, buf, ctx, bg);
        // samples never move the terminal cursor: draw the caret as a cell
        if v.editing && f.area.width > 3 {
            let field = f.area;
            let text_w = crate::ui::text::width(f.buffer.text()) as u16;
            let x = (field.x + 2 + text_w).min(field.right().saturating_sub(2));
            if let Some(c) = buf.cell_mut(Position::new(x, field.y)) {
                c.bg = ctx.theme.text_primary;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::button::Button;
    use crate::widgets::input::TextInput;

    fn grid() -> StateGrid {
        let id = WidgetId::of("grid");
        StateGrid::new(id)
            .rows(&[
                SpecimenState::Rest,
                SpecimenState::Focused,
                SpecimenState::Pressed,
                SpecimenState::Disabled,
                SpecimenState::Selected,
            ])
            .column("Primary", Button::primary(id.child(0), "Label"))
            .column("Toggle", Button::toggle(id.child(1), "Label", false))
    }

    fn draw(grid: &mut StateGrid, t: &Theme, area: Rect, levels: bool) -> (Buffer, HitRegistry) {
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(t, Interaction::default(), &mut hits, &mut ring);
        if levels {
            grid.render_levels(area, &mut buf, &mut ctx);
        } else {
            grid.render(area, &mut buf, &mut ctx, t.surface);
        }
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        assert!(ring.reachable().is_empty(), "samples are never focus stops");
        (buf, hits)
    }

    fn row_text(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    #[test]
    fn rows_are_labelled_and_drawn_by_the_widget() {
        let t = Theme::junie();
        let mut g = grid();
        let (buf, hits) = draw(&mut g, &t, Rect::new(0, 0, 60, 8), false);
        assert!(row_text(&buf, 0).contains("Primary"));
        assert!(row_text(&buf, 0).contains("Toggle"));
        for (i, s) in g.rows.iter().enumerate() {
            assert!(row_text(&buf, 1 + i as u16).starts_with(s.label()), "{s:?}");
        }
        assert_eq!(g.rendered_rows(), g.rows.as_slice());
        // a disabled primary is drawn in the disabled ink, as the widget does
        let x = (15..30)
            .find(|&x| buf[(x, 4)].symbol() == "L")
            .expect("disabled label");
        assert_eq!(buf[(x, 4)].fg, t.disabled);
        // a toggle that is on shows its on dot; a primary has no selected look
        assert!(row_text(&buf, 5).contains(t.glyphs.dot_on));
        assert!(row_text(&buf, 5)[..30].contains('–'));
        assert!(hits.hit(Position::new(16, 1)).is_none(), "no hit regions");
    }

    #[test]
    fn a_pressed_sample_differs_from_rest() {
        let t = Theme::junie();
        let mut g = grid();
        let (buf, _) = draw(&mut g, &t, Rect::new(0, 0, 60, 8), false);
        let style = |y: u16| buf[(17, y)].style();
        assert_ne!(style(1), style(3), "pressed is not rest");
    }

    #[test]
    fn a_focused_sample_depicts_focus_without_a_live_bar() {
        for level in LEVELS {
            let t = Theme::for_level(level);
            let mut g = grid();
            let (buf, _) = draw(&mut g, &t, Rect::new(0, 0, 60, 8), false);
            assert!(
                !buf.content()
                    .iter()
                    .any(|c| c.symbol() == t.glyphs.focus_bar),
                "{level:?}: no second live focus bar"
            );
            // the focused row carries the depiction, at the sample's gutter
            let focused = row_text(&buf, 2);
            assert!(focused.contains('▏'), "{level:?}: {focused}");
            assert!(!row_text(&buf, 1).contains('▏'), "rest has no mark");
        }
        let mut ascii = Theme::junie();
        crate::theme::ascii_glyphs(&mut ascii);
        let mut g = grid();
        let (buf, _) = draw(&mut g, &ascii, Rect::new(0, 0, 60, 8), false);
        assert!(row_text(&buf, 2).contains(':'));
        assert!(!row_text(&buf, 2).contains('|'));
    }

    #[test]
    fn the_depiction_is_overridable() {
        let t = Theme::junie();
        let mut g = grid().focus_mark("┊");
        let (buf, _) = draw(&mut g, &t, Rect::new(0, 0, 60, 8), false);
        assert!(row_text(&buf, 2).contains('┊'));
    }

    #[test]
    fn compare_levels_draws_every_level() {
        let t = Theme::junie();
        let mut g = grid();
        let (w, h) = g.size();
        // all four side by side when they fit
        let wide = Rect::new(0, 0, w * 4 + 6, h + 1);
        let (buf, _) = draw(&mut g, &t, wide, true);
        let title = row_text(&buf, 0);
        for level in LEVELS {
            assert!(title.contains(level.label()), "{title}");
        }
        assert_eq!(g.levels_height(wide.width), h + 1);
        assert_eq!(g.rendered_rows(), g.rows.as_slice());
        // stacked one per band when narrow
        let narrow = Rect::new(0, 0, w, g.levels_height(w));
        let (buf, _) = draw(&mut g, &t, narrow, true);
        for (i, level) in LEVELS.into_iter().enumerate() {
            assert!(row_text(&buf, i as u16 * (h + 2)).starts_with(level.label()));
        }
        // the mono pane carries no colour
        let mono_y = 3 * (h + 2) + 1;
        assert!((0..w).all(|x| buf[(x, mono_y)].fg == Color::Reset
            || buf[(x, mono_y)].fg == Theme::for_level(ColorLevel::Mono).text_muted));
    }

    #[test]
    fn inputs_draw_editing_error_and_narrow() {
        let t = Theme::junie();
        let id = WidgetId::of("inputs");
        let mut g = StateGrid::new(id)
            .column_width(34)
            .rows(&[
                SpecimenState::Editing,
                SpecimenState::Error,
                SpecimenState::Empty,
                SpecimenState::Narrow,
                SpecimenState::Pressed,
            ])
            .column(
                "Text",
                TextInput::new(id.child(0), "Project name").value("payments-gateway"),
            );
        let (w, h) = g.size();
        assert_eq!(h, 1 + 2 + 3 + 2 + 2 + 1);
        let (buf, _) = draw(&mut g, &t, Rect::new(0, 0, w, h), false);
        // editing: a static caret cell after the text, no terminal cursor
        let text = row_text(&buf, 2);
        let end = text.find("gateway").unwrap() + "gateway".len();
        let caret = text[..end].chars().count() as u16;
        assert_eq!(buf[(caret, 2)].bg, t.text_primary);
        // error: the message row
        assert!((0..buf.area.height).any(|y| row_text(&buf, y).contains("Enter a valid value")));
        // empty: the placeholder
        assert!((0..buf.area.height).any(|y| row_text(&buf, y).contains("Type a value")));
        // narrow: the field ends inside the 24-column slot
        let narrow_field = 1 + 2 + 3 + 2 + 1;
        let x0 = g.label_width;
        assert!(buf[(x0 + 24, narrow_field)].bg != buf[(x0 + 1, narrow_field)].bg);
    }
}
