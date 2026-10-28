use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};

use crate::ui::Ui;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellView {
    pub symbol: String,

    pub style: Style,
}

impl CellView {
    pub fn fg(&self) -> Color {
        self.style.fg.unwrap_or(Color::Reset)
    }

    pub fn bg(&self) -> Color {
        self.style.bg.unwrap_or(Color::Reset)
    }

    pub fn modifier(&self) -> Modifier {
        self.style.add_modifier
    }
}

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
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            buffer: Buffer::empty(area),
        }
    }

    pub const fn area(&self) -> Rect {
        self.area
    }

    pub fn symbol(&mut self, pos: Position, symbol: &str, style: Style) {
        if let Some(cell) = self.buffer.cell_mut(pos) {
            cell.set_symbol(symbol).set_style(style);
        }
    }

    pub fn text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut column = x;
        let mut encoded = [0; 4];
        for ch in text.chars() {
            let symbol = ch.encode_utf8(&mut encoded);
            self.symbol(Position::new(column, y), symbol, style);
            column = column.saturating_add(1);
        }
    }

    pub fn string(&mut self, x: u16, y: u16, text: &str, style: Style) {
        if !self.area.contains(Position::new(x, y)) {
            return;
        }
        self.buffer.set_string(x, y, text, style);
    }

    pub fn fill(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        for pos in area.positions() {
            self.symbol(pos, " ", style);
        }
    }

    pub fn style_area(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        self.buffer.set_style(area, style);
    }

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

    pub fn cell(&self, pos: Position) -> Option<CellView> {
        self.buffer.cell(pos).map(|cell| CellView {
            symbol: cell.symbol().to_owned(),
            style: cell.style(),
        })
    }

    pub fn blit(&self, ui: &mut Ui<'_>, area: Rect) {
        let area = self.area.intersection(area);
        for pos in area.positions() {
            if let Some(cell) = self.cell(pos) {
                ui.paint_cell(pos, &cell.symbol, cell.style);
            }
        }
    }
}
