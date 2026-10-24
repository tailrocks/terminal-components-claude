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

use crate::scroll::ScrollState;
use crate::theme::PaintStyle;
use crate::ui::Ui;

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
