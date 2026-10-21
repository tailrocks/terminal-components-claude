//! Component-author canvas primitives.
//!
//! A component that places content at exact cells asks the canvas for it
//! instead of building rectangles and writing cells itself. The canvas owns
//! the geometry: callers pass an origin and the content, and the canvas
//! derives the region it paints into.
//!
//! Every primitive here is a thin, deterministic wrapper over the `Ui` paint
//! surface. It exists so that application crates never construct a `Rect` or
//! a `Position` for painting, and so that the region a run occupies stays
//! derived from the content that is painted into it.

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use crate::scroll::ScrollState;
use crate::theme::PaintStyle;
use crate::ui::Ui;

/// The painted content of one cell, as the canvas sees it.
///
/// A restyle reads the cell through this view and never touches the
/// underlying buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellView {
    /// The glyph the cell holds.
    pub symbol: String,
    /// The style the cell carries.
    pub style: Style,
}

impl CellView {
    /// The foreground ink of the cell.
    pub fn fg(&self) -> Color {
        self.style.fg.unwrap_or(Color::Reset)
    }

    /// The background ink of the cell.
    pub fn bg(&self) -> Color {
        self.style.bg.unwrap_or(Color::Reset)
    }

    /// The modifiers the cell carries.
    pub fn modifier(&self) -> Modifier {
        self.style.add_modifier
    }
}

/// The number of cells `text` occupies on one line.
///
/// The canvas measures the run it is about to paint, so a following run can
/// start at `origin + text_width(text)` without the caller measuring twice.
pub fn text_width(text: &str) -> u16 {
    text.chars().count() as u16
}

/// Paint one line of text at an absolute origin.
///
/// The painted run is exactly as wide as the text.
pub fn put_text(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, style: impl Into<PaintStyle>) -> u16 {
    ui.paint_str(Rect::new(x, y, text_width(text), 1), text, style)
}

/// Paint one line of text at an absolute origin, sized to `width` cells.
pub fn put_text_w(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    width: u16,
    text: &str,
    style: impl Into<PaintStyle>,
) -> u16 {
    ui.paint_str(Rect::new(x, y, width, 1), text, style)
}

/// Paint one line of text into `area`.
///
/// The area clips the run and supplies its width.
pub fn put_text_in(ui: &mut Ui<'_>, area: Rect, text: &str, style: impl Into<PaintStyle>) -> u16 {
    ui.paint_str(area, text, style)
}

/// Paint one glyph at an absolute cell.
pub fn put_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, style: impl Into<PaintStyle>) {
    ui.paint_cell(Position::new(x, y), symbol, style);
}

/// Fill the region at an absolute origin.
pub fn put_fill(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: impl Into<PaintStyle>,
) {
    ui.fill(Rect::new(x, y, width, height), style);
}

/// Fill a region.
pub fn put_fill_in(ui: &mut Ui<'_>, area: Rect, style: impl Into<PaintStyle>) {
    ui.fill(area, style);
}

/// Draw a border around the region at an absolute origin.
///
/// The returned rectangle is the border's interior.
pub fn put_frame(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: impl Into<PaintStyle>,
) -> Rect {
    ui.frame(Rect::new(x, y, width, height), style)
}

/// Draw a border around a region.
///
/// The returned rectangle is the border's interior.
pub fn put_frame_in(ui: &mut Ui<'_>, area: Rect, style: impl Into<PaintStyle>) -> Rect {
    ui.frame(area, style)
}

/// Paint the scroll edges of a region.
pub fn put_scroll_edges(ui: &mut Ui<'_>, area: Rect, state: &ScrollState) {
    ui.scroll_edges(area, state);
}

/// Paint one glyph at a cell.
pub fn put_cell_at(ui: &mut Ui<'_>, pos: Position, symbol: &str, style: impl Into<PaintStyle>) {
    ui.paint_cell(pos, symbol, style);
}

/// Restyle the cells of a region.
///
/// `restyle` reads each cell and returns the style to apply, or `None` to
/// leave the cell as it is. The style only changes what it names: a style
/// with a foreground keeps the cell's background and modifiers, and a style
/// with only modifiers keeps the cell's inks.
pub fn restyle_cells(
    ui: &mut Ui<'_>,
    area: Rect,
    restyle: impl Fn(Position, CellView) -> Option<Style>,
) {
    let (buf, _) = ui.raw();
    for pos in area.positions() {
        let Some(cell) = buf.cell_mut(pos) else {
            continue;
        };
        let view = CellView {
            symbol: cell.symbol().to_owned(),
            style: cell.style(),
        };
        if let Some(style) = restyle(pos, view) {
            cell.set_style(style);
        }
    }
}

/// Restyle the cells of the current clip region.
///
/// This is the canvas counterpart of reading the whole clip: the component
/// never sees the buffer.
pub fn restyle_clip(ui: &mut Ui<'_>, restyle: impl Fn(Position, CellView) -> Option<Style>) {
    let clip = ui.raw().1;
    restyle_cells(ui, clip, restyle);
}

/// Repaint the glyphs of the cells a component selects.
///
/// Each selected cell keeps its glyph and is painted again with `style`. The
/// glyph is painted through the `Ui` surface, so the cell's paint role stays
/// in step with the ink it shows.
pub fn repaint_cells(
    ui: &mut Ui<'_>,
    area: Rect,
    select: impl Fn(Position, &CellView) -> bool,
    style: impl Into<PaintStyle>,
) {
    let style = style.into();
    let (buf, _) = ui.raw();
    let mut targets = Vec::new();
    for pos in area.positions() {
        let Some(cell) = buf.cell(pos) else {
            continue;
        };
        let view = CellView {
            symbol: cell.symbol().to_owned(),
            style: cell.style(),
        };
        if select(pos, &view) {
            targets.push((pos, view.symbol));
        }
    }
    for (pos, symbol) in targets {
        ui.paint_cell(pos, &symbol, style);
    }
}
