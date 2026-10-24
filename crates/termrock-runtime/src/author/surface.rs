//! Owned off-screen paint storage for component authors.
//!
//! The surface exposes bounded symbol/style operations without exposing its
//! backing [`Buffer`]. Publishing uses the ordinary [`Ui`] paint path, so the
//! destination Ui clip and layer behavior still apply.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use crate::ui::Ui;

/// An owned read-only snapshot of the symbol and style visible at one cell.
///
/// This intentionally omits buffer diff metadata such as skip/forced-width
/// flags. Reading a view owns a copy of its symbol; it never exposes the
/// surface's backing cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellView {
    /// The cell's visible glyph string.
    pub symbol: String,
    /// The cell's represented style channels and modifier patch.
    pub style: Style,
}

impl CellView {
    /// Return the foreground, treating an unspecified foreground as reset.
    pub fn fg(&self) -> Color {
        self.style.fg.unwrap_or(Color::Reset)
    }

    /// Return the background, treating an unspecified background as reset.
    pub fn bg(&self) -> Color {
        self.style.bg.unwrap_or(Color::Reset)
    }

    /// Return the style's added modifiers.
    pub fn modifier(&self) -> Modifier {
        self.style.add_modifier
    }
}

/// A bounded, owned off-screen surface for component painting.
///
/// Coordinates are absolute terminal coordinates. The backing buffer is never
/// exposed mutably. `text` advances one cell for each Rust scalar value;
/// `string` delegates to Ratatui's measured string writer. `blit` republishes
/// each cell's symbol and style through [`Ui::paint_cell`]; it does not copy
/// private Cell diff metadata or promise byte-for-byte Cell transfer.
#[derive(Clone, Debug)]
pub struct PaintSurface {
    area: Rect,
    buffer: Buffer,
}

impl PartialEq for PaintSurface {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer
    }
}

impl PaintSurface {
    /// Create an empty surface covering `area`.
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            buffer: Buffer::empty(area),
        }
    }

    /// Return the absolute region owned by this surface.
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// Set one cell's symbol and the style channels named by `style`.
    ///
    /// Positions outside the owned area are ignored. Ratatui's cell style
    /// merge semantics apply, including explicit reset colors and removed
    /// modifiers.
    pub fn symbol(&mut self, pos: Position, symbol: &str, style: Style) {
        if let Some(cell) = self.buffer.cell_mut(pos) {
            cell.set_symbol(symbol).set_style(style);
        }
    }

    /// Paint one Rust scalar per successive column, saturating at `u16::MAX`.
    ///
    /// This deliberately differs from [`Self::string`], which measures
    /// grapheme widths.
    pub fn text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut column = x;
        let mut encoded = [0; 4];
        for ch in text.chars() {
            let symbol = ch.encode_utf8(&mut encoded);
            self.symbol(Position::new(column, y), symbol, style);
            column = column.saturating_add(1);
        }
    }

    /// Paint text using Ratatui's grapheme and cell-width measurement.
    ///
    /// A starting position outside the owned region ignores the entire write.
    /// Valid starts retain Ratatui's measured grapheme widths and right-edge
    /// truncation.
    pub fn string(&mut self, x: u16, y: u16, text: &str, style: Style) {
        if !self.area.contains(Position::new(x, y)) {
            return;
        }
        self.buffer.set_string(x, y, text, style);
    }

    /// Fill the intersection of `area` and the owned region with blank symbols.
    pub fn fill(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        for pos in area.positions() {
            self.symbol(pos, " ", style);
        }
    }

    /// Apply Ratatui's style patch semantics to cells in the owned intersection.
    ///
    /// Unspecified channels are inherited; `Color::Reset` explicitly resets an
    /// ink channel, and removed modifiers are applied by the cell's style
    /// merge.
    pub fn style_area(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        self.buffer.set_style(area, style);
    }

    /// Read and optionally restyle cells in the intersection of the requested
    /// and owned regions.
    ///
    /// The callback receives an owned [`CellView`], never a reference to the
    /// backing buffer. Returning `None` leaves the cell unchanged; a returned
    /// style is merged using Ratatui's ordinary cell semantics.
    pub fn restyle(&mut self, area: Rect, restyle: impl Fn(Position, CellView) -> Option<Style>) {
        let area = self.area.intersection(area);
        for pos in area.positions() {
            let Some(cell) = self.buffer.cell_mut(pos) else {
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

    /// Return an owned read-only view when `pos` is inside the surface.
    pub fn cell(&self, pos: Position) -> Option<CellView> {
        self.buffer.cell(pos).map(|cell| CellView {
            symbol: cell.symbol().to_owned(),
            style: cell.style(),
        })
    }

    /// Publish the requested surface cells through the current Ui paint path.
    ///
    /// The source and requested regions are intersected before visiting cells;
    /// [`Ui::paint_cell`] supplies the destination clip and layer semantics.
    /// Wide-cell continuations and diff flags are not copied as raw Cell data.
    pub fn blit(&self, ui: &mut Ui<'_>, area: Rect) {
        let area = self.area.intersection(area);
        for pos in area.positions() {
            if let Some(cell) = self.cell(pos) {
                ui.paint_cell(pos, &cell.symbol, cell.style);
            }
        }
    }
}
