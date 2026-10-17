//! One painter for the rows of every list-like widget (plan S10,
//! `RowUi`).
//!
//! A row's look follows one recipe everywhere: its state (rest, hover,
//! pressed, focused, selected, disabled) resolves through the theme into a
//! body style and a gutter cell, the body fills the row, and the gutter
//! carries the single `▎` focus indicator on the focused row. Lists,
//! trees, property lists, step lists, tables, grids and the popup option
//! lists used to repeat that recipe (eleven copies); they now ask
//! [`RowUi::paint`] and draw only their own content on the [`RowPaint`]
//! it returns.
//!
//! Two looks ([`RowLook`]):
//!
//! - [`RowLook::Bar`]: a row of a widget that is itself the focus stop (a
//!   list, a tree, a table): the theme's row ladder, and the `▎` gutter on
//!   the focused cursor row.
//! - [`RowLook::Fill`]: a row of a list whose keyboard focus belongs to an
//!   owner (a popup list under a search field, an editor or a select): the
//!   cursor row is a highlight fill and draws no second focus bar
//!   ([`CursorStyle::Fill`], AGENTS.md "Visual states"), or the in-page
//!   look with [`CursorStyle::Bar`].
//!
//! Content helpers keep the shared rules in one place: [`RowPaint::label`]
//! draws a cut-to-fit label and wires its reveal (pointer, and keyboard
//! when the owner keeps a [`RevealState`]), [`RowPaint::ink`] keeps a
//! secondary ink readable on the row's fill.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::id::WidgetId;
use crate::theme::{CursorRow, CursorStyle, Kind};
use crate::ui::ctx::{RenderCtx, VisualState, fill};
use crate::ui::reveal::RevealState;
use crate::ui::text::truncate;

/// How a row shows its state. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowLook {
    /// A row of a widget that is itself the focus stop: the row ladder and
    /// the `▎` gutter on the focused row (the default).
    #[default]
    Bar,
    /// A row of a list whose focus belongs to an owner, or a list that
    /// picks its cursor look ([`CursorStyle`]).
    Fill(CursorStyle),
}

impl From<CursorStyle> for RowLook {
    fn from(c: CursorStyle) -> Self {
        RowLook::Fill(c)
    }
}

/// What a chosen row's mark shows ([`RowPaint::mark`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// The current item of a single choice: [`crate::theme::Glyphs::current`].
    Current,
    /// A ticked row of a multiple choice: [`crate::theme::Glyphs::tick`].
    Tick,
    /// A checked box: [`crate::theme::Glyphs::check_on`].
    Check,
    /// The owner's own glyph (a tree's `selected_marker`).
    Glyph(&'static str),
}

/// Per-row marks an owner sets beside the selection (plan S16): the
/// pointer `›` of a row the app tracks as current (a page in a settings
/// list), the `•` of a row with unsaved changes, and the mixed `–` of a
/// multi-select row only part of whose contents are checked (a category
/// with some of its files checked, the macOS mixed checkbox).
///
/// [`crate::widgets::list::ListItem::mark`] carries one per list row and
/// [`crate::widgets::table::DataTable::set_mark`] one per table row (drawn
/// in a [`crate::widgets::table::Column::marker`] column). Every field is
/// off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RowMark {
    /// The row is the owner's current one: `›` in the mark column
    /// ([`crate::theme::Glyphs::current`], [`crate::theme::Theme::chosen`]).
    pub pointer: bool,
    /// The row has unsaved changes: `•` in warning
    /// ([`crate::theme::Glyphs::dirty`]).
    pub dirty: bool,
    /// Multi-select: the row is partly checked. An unchecked row shows the
    /// mixed `–` ([`crate::theme::Glyphs::dash`]) where a checked one shows
    /// `✓`; checking it (Space, a click on the mark) checks it fully.
    pub partial: bool,
}

impl RowMark {
    /// No marks.
    pub const NONE: RowMark = RowMark {
        pointer: false,
        dirty: false,
        partial: false,
    };

    /// See [`Self::pointer`](RowMark::pointer).
    pub const fn pointer(mut self, on: bool) -> Self {
        self.pointer = on;
        self
    }

    /// See [`Self::dirty`](RowMark::dirty).
    pub const fn dirty(mut self, on: bool) -> Self {
        self.dirty = on;
        self
    }

    /// See [`Self::partial`](RowMark::partial).
    pub const fn partial(mut self, on: bool) -> Self {
        self.partial = on;
        self
    }
}

/// The row painter. See the module docs.
#[derive(Debug, Clone, Copy, Default)]
pub struct RowUi;

impl RowUi {
    /// Resolve the row's look for `state` on the plane `bg`, fill `area`
    /// with its body and draw its gutter cell on every line of `area`.
    /// The owner draws its content on the returned [`RowPaint`]; content
    /// starts at `area.x + 1`.
    pub fn paint(
        ctx: &RenderCtx,
        buf: &mut Buffer,
        area: Rect,
        state: VisualState,
        look: RowLook,
        bg: Color,
    ) -> RowPaint {
        let p = Self::resolve(ctx, area, state, look, bg);
        p.draw(buf);
        p
    }

    /// [`Self::paint`] without drawing: an owner that restyles the body
    /// (a deleted row's strike-through) changes [`RowPaint::look`] and
    /// then calls [`RowPaint::draw`].
    pub fn resolve(
        ctx: &RenderCtx,
        area: Rect,
        state: VisualState,
        look: RowLook,
        bg: Color,
    ) -> RowPaint {
        let t = ctx.theme;
        let resolved = match look {
            RowLook::Bar => t.interactive(Kind::Row, state, t.plane_of(bg)),
            RowLook::Fill(c) => t.cursor_row(state, bg, c),
        };
        RowPaint {
            area,
            state,
            look: resolved,
            plane: bg,
        }
    }
}

/// A painted row: its cells, its state and the resolved look its content
/// draws on ([`RowUi::paint`]).
#[derive(Debug, Clone, Copy)]
pub struct RowPaint {
    /// The row's cells (gutter included; content starts at `area.x + 1`).
    pub area: Rect,
    /// The state it was painted in.
    pub state: VisualState,
    /// The resolved look: body style, gutter cell and whether the body is
    /// a fill ([`crate::theme::Theme::interactive`]).
    pub look: CursorRow,
    /// The plane the row sits on.
    pub plane: Color,
}

impl RowPaint {
    /// Fill the row with its body and draw its gutter cell on every line.
    pub fn draw(&self, buf: &mut Buffer) {
        fill(buf, self.area, self.look.style);
        for y in self.area.top()..self.area.bottom() {
            buf.set_string(self.area.x, y, self.look.gutter, self.look.gutter_style);
        }
    }

    /// The row's body style: content draws with it, changing only its ink.
    pub fn style(&self) -> Style {
        self.look.style
    }

    /// The row's background (its body fill, else the plane).
    pub fn bg(&self) -> Color {
        self.look.style.bg.unwrap_or(self.plane)
    }

    /// A secondary ink (a detail, a group, a glyph) as it reads on this
    /// row: on a highlight fill it becomes primary text
    /// ([`crate::theme::Theme::row_ink`]).
    pub fn ink(&self, ctx: &RenderCtx, ink: Color) -> Color {
        ctx.theme.row_ink(&self.look, ink)
    }

    /// Draw a chosen-value mark at `(x, y)`: the current item (`●`), a
    /// ticked row (`✓`), a checked box, or the owner's own glyph. Its ink
    /// is [`crate::theme::Theme::chosen`]: the accent only while the
    /// widget (`focused`) has keyboard focus, primary text otherwise, made
    /// readable on the row; a disabled row's mark keeps the row's ink.
    pub fn mark(
        &self,
        ctx: &RenderCtx,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        kind: MarkKind,
        focused: bool,
    ) {
        let t = ctx.theme;
        let glyph = match kind {
            MarkKind::Current => t.glyphs.current,
            MarkKind::Tick => t.glyphs.tick,
            MarkKind::Check => t.glyphs.check_on,
            MarkKind::Glyph(g) => g,
        };
        let style = if self.state.disabled {
            self.style()
        } else {
            self.style().fg(t.legible(t.chosen(focused), self.bg()))
        };
        buf.set_string(x, y, glyph, style);
    }

    /// Draw the unsaved-changes dot ([`RowMark::dirty`]) at `(x, y)`: `•`
    /// in warning, readable on the row; a disabled row keeps its ink.
    pub fn dirty(&self, ctx: &RenderCtx, buf: &mut Buffer, x: u16, y: u16) {
        let t = ctx.theme;
        let style = if self.state.disabled {
            self.style()
        } else {
            self.style().fg(t.legible_mark(t.warning, self.bg()))
        };
        buf.set_string(x, y, t.glyphs.dirty, style);
    }

    /// Draw a status glyph at `(x, y)` in `tone`, readable on the row
    /// (an error is bold too, so colour is never the only cue beside the
    /// glyph itself). A disabled row keeps its ink.
    pub fn status(
        &self,
        ctx: &RenderCtx,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        glyph: &str,
        tone: crate::theme::Tone,
    ) {
        use crate::theme::Tone;
        let t = ctx.theme;
        let mut style = if self.state.disabled {
            self.style()
        } else {
            self.style().fg(t.tone_on(tone, self.bg()))
        };
        if matches!(tone, Tone::Error | Tone::Danger) {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        buf.set_string(x, y, glyph, style);
    }

    /// Draw `text` cut to `cell` (with `…` when it does not fit) in
    /// `style`, and register the cut cell for reveal: the pointer's
    /// expansion tooltip always, and with `reveal` the keyboard reveal of
    /// the focused cursor row. Returns whether the text was cut.
    #[allow(clippy::too_many_arguments)]
    pub fn label(
        &self,
        ctx: &mut RenderCtx,
        buf: &mut Buffer,
        reveal: &mut RevealState,
        id: WidgetId,
        cell: Rect,
        text: &str,
        style: Style,
    ) -> bool {
        buf.set_string(cell.x, cell.y, truncate(text, cell.width as usize), style);
        if self.state.disabled {
            return false;
        }
        reveal.cell(ctx, id, cell, text, self.state.focused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;

    fn paint(s: VisualState, look: RowLook) -> (Buffer, RowPaint) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let ctx = RenderCtx::new(&theme, Default::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 2));
        let p = RowUi::paint(&ctx, &mut buf, Rect::new(0, 0, 6, 2), s, look, theme.canvas);
        (buf, p)
    }

    #[test]
    fn the_focused_bar_row_draws_the_gutter_on_every_line() {
        let s = VisualState {
            focused: true,
            ..Default::default()
        };
        let (buf, p) = paint(s, RowLook::Bar);
        let t = Theme::junie();
        for y in 0..2 {
            assert_eq!(buf[(0, y)].symbol(), t.glyphs.focus_bar);
            assert_eq!(buf[(0, y)].fg, t.focus);
            assert_eq!(buf[(1, y)].bg, p.bg());
        }
        // the same look a widget got from the theme by hand
        assert_eq!(p.style(), t.row(s, t.canvas));
    }

    #[test]
    fn a_fill_cursor_row_draws_no_second_focus_bar() {
        let s = VisualState {
            focused: true,
            ..Default::default()
        };
        let (buf, p) = paint(s, RowLook::Fill(CursorStyle::Fill));
        assert_eq!(buf[(0, 0)].symbol(), " ");
        assert!(p.look.on_fill);
        assert_eq!(p.bg(), Theme::junie().highlight);
    }

    #[test]
    fn a_disabled_row_shows_no_focus_bar() {
        let s = VisualState {
            focused: true,
            disabled: true,
            ..Default::default()
        };
        let (buf, _) = paint(s, RowLook::Bar);
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }
}
