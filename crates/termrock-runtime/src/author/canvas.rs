use ratatui_core::layout::{Position, Rect};

use crate::scroll::ScrollState;
use crate::theme::PaintStyle;
use crate::ui::Ui;

pub fn text_width(text: &str) -> u16 {
    text.chars().count() as u16
}

pub fn put_text(ui: &mut Ui<'_>, x: u16, y: u16, text: &str, style: impl Into<PaintStyle>) -> u16 {
    ui.paint_str(Rect::new(x, y, text_width(text), 1), text, style)
}

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

pub fn put_text_in(ui: &mut Ui<'_>, area: Rect, text: &str, style: impl Into<PaintStyle>) -> u16 {
    ui.paint_str(area, text, style)
}

pub fn put_cell(ui: &mut Ui<'_>, x: u16, y: u16, symbol: &str, style: impl Into<PaintStyle>) {
    ui.paint_cell(Position::new(x, y), symbol, style);
}

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

pub fn put_fill_in(ui: &mut Ui<'_>, area: Rect, style: impl Into<PaintStyle>) {
    ui.fill(area, style);
}

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

pub fn put_frame_in(ui: &mut Ui<'_>, area: Rect, style: impl Into<PaintStyle>) -> Rect {
    ui.frame(area, style)
}

pub fn put_scroll_edges(ui: &mut Ui<'_>, area: Rect, state: &ScrollState) {
    ui.scroll_edges(area, state);
}
