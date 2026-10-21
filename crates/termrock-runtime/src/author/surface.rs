//! An off-screen cell surface for component authors.
//!
//! A component that paints into an off-screen buffer — an effect, a dialog
//! drawn on its own layer — asks the surface for its cells instead of
//! writing the buffer itself. The surface owns the buffer and the geometry;
//! the component only says what each cell should hold.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;

use super::canvas::CellView;
use crate::ui::Ui;

/// An off-screen cell surface.
#[derive(Clone, Debug)]
pub struct PaintSurface {
    area: Rect,
    buf: Buffer,
}

impl PartialEq for PaintSurface {
    fn eq(&self, other: &Self) -> bool {
        self.buf == other.buf
    }
}

impl PaintSurface {
    /// Create a surface that covers `area`.
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            buf: Buffer::empty(area),
        }
    }

    /// The region the surface covers.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Paint one glyph at a cell.
    pub fn symbol(&mut self, pos: Position, symbol: &str, style: Style) {
        if let Some(cell) = self.buf.cell_mut(pos) {
            cell.set_symbol(symbol);
            cell.set_style(style);
        }
    }

    /// Paint a run of text, one cell per character.
    pub fn text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut col = x;
        for ch in text.chars() {
            self.symbol(Position::new(col, y), &ch.to_string(), style);
            col = col.saturating_add(1);
        }
    }

    /// Paint a run of text as a string, letting the surface measure it.
    pub fn string(&mut self, x: u16, y: u16, text: &str, style: Style) {
        self.buf.set_string(x, y, text, style);
    }

    /// Fill a region with blank cells.
    pub fn fill(&mut self, area: Rect, style: Style) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                self.symbol(Position::new(x, y), " ", style);
            }
        }
    }

    /// Set the style of every cell in a region.
    pub fn style_area(&mut self, area: Rect, style: Style) {
        self.buf.set_style(area, style);
    }

    /// Restyle the cells of a region.
    ///
    /// `restyle` reads each cell and returns the style to apply, or `None`
    /// to leave the cell as it is.
    pub fn restyle(&mut self, area: Rect, restyle: impl Fn(Position, CellView) -> Option<Style>) {
        for pos in area.positions() {
            let Some(cell) = self.buf.cell_mut(pos) else {
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

    /// Read a cell of the surface.
    pub fn cell(&self, pos: Position) -> Option<CellView> {
        self.buf.cell(pos).map(|cell| CellView {
            symbol: cell.symbol().to_owned(),
            style: cell.style(),
        })
    }

    /// Paint the surface onto the frame.
    pub fn blit(&self, ui: &mut Ui<'_>, area: Rect) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let Some(cell) = self.cell(Position::new(x, y)) else {
                    continue;
                };
                ui.paint_cell(Position::new(x, y), &cell.symbol, cell.style);
            }
        }
    }
}
