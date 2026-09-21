//! Complete-frame observation schema consumed by candidate capture.

use junie_tui::{Color, Id, LayerId, Modifier, Position};
use ratatui_core::buffer::{Buffer, Cell};

use crate::color::{ColorSpec, TerminalSize};
use crate::error::AdapterError;
use crate::pages::OraclePageId;

/// One published cell: symbol, colours, modifiers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellObs {
    /// Column.
    pub x: u16,
    /// Row.
    pub y: u16,
    /// Grapheme / continuation symbol.
    pub symbol: String,
    /// Foreground.
    pub fg: Color,
    /// Background.
    pub bg: Color,
    /// Style modifiers.
    pub modifier: Modifier,
}

impl CellObs {
    fn from_cell(x: u16, y: u16, cell: &Cell) -> Self {
        Self {
            x,
            y,
            symbol: cell.symbol().to_owned(),
            fg: cell.fg,
            bg: cell.bg,
            modifier: cell.modifier,
        }
    }
}

/// One registered hit region from the last presented frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitObs {
    /// Owner.
    pub owner: Id,
    /// Region rectangle.
    pub x: u16,
    /// Region rectangle.
    pub y: u16,
    /// Region rectangle.
    pub width: u16,
    /// Region rectangle.
    pub height: u16,
    /// Layer.
    pub layer: LayerId,
}

/// Complete parent-frame observation. Transient regions are not masked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameObservation {
    /// Oracle page identity requested by the scenario.
    pub oracle_page: OraclePageId,
    /// Production page after the last update/draw, if the route exists.
    pub production_page: Option<OraclePageId>,
    /// Terminal size.
    pub size: TerminalSize,
    /// Colour mode.
    pub color: ColorSpec,
    /// Every cell in row-major order, including continuations.
    pub cells: Vec<CellObs>,
    /// Cursor position and visibility (`None` is hidden).
    pub cursor: Option<Position>,
    /// Focused control.
    pub focus: Option<Id>,
    /// Hovered control.
    pub hover: Option<Id>,
    /// Whether the app requested quit.
    pub quit: bool,
    /// Logical Instant elapsed milliseconds.
    pub elapsed_ms: u128,
    /// Runtime moment milliseconds.
    pub moment_ms: u128,
    /// Published hit rectangles.
    pub hits: Vec<HitObs>,
    /// True when production has no route for `oracle_page`.
    pub production_route_absent: bool,
}

impl FrameObservation {
    /// Snapshot a presented buffer and runtime facts.
    #[expect(
        clippy::too_many_arguments,
        reason = "one capture records the complete parent-frame schema"
    )]
    pub fn capture(
        oracle_page: OraclePageId,
        production_page: Option<OraclePageId>,
        size: TerminalSize,
        color: ColorSpec,
        buffer: &Buffer,
        cursor: Option<Position>,
        focus: Option<Id>,
        hover: Option<Id>,
        quit: bool,
        elapsed_ms: u128,
        moment_ms: u128,
        hits: Vec<HitObs>,
    ) -> Result<Self, AdapterError> {
        let mut cells = Vec::new();
        for y in 0..size.rows {
            for x in 0..size.cols {
                let Some(cell) = buffer.cell((x, y)) else {
                    return Err(AdapterError::MissingCell { x, y });
                };
                cells.push(CellObs::from_cell(x, y, cell));
            }
        }
        Ok(Self {
            oracle_page,
            production_page,
            size,
            color,
            cells,
            cursor,
            focus,
            hover,
            quit,
            elapsed_ms,
            moment_ms,
            hits,
            production_route_absent: production_page.is_none(),
        })
    }

    /// Plain-text rows joined with newlines (observation, never a selector).
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        for y in 0..self.size.rows {
            if y > 0 {
                out.push('\n');
            }
            for cell in self.cells.iter().filter(|cell| cell.y == y) {
                out.push_str(&cell.symbol);
            }
        }
        out
    }

    /// Cell count must equal cols × rows.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        let expected = usize::from(self.size.cols).saturating_mul(usize::from(self.size.rows));
        self.cells.len() == expected
    }
}
