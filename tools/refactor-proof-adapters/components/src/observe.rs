//! Complete-frame observation schema consumed by candidate capture.

use junie_tui::{Color, Id, LayerId, Modifier, Position};
use ratatui_core::buffer::{Buffer, Cell};

use crate::color::{ColorSpec, Origin, TerminalSize};
use crate::error::AdapterError;
use crate::expansion::Facet;
use crate::families::Family;
use crate::states::ComponentState;

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
    /// Wide-grapheme continuation cell.
    pub continuation: bool,
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
            // Paint-time buffers carry no skip flag; marked by width below.
            continuation: false,
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

/// Complete fixture-frame observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameObservation {
    /// Observed family.
    pub family: Family,
    /// Requested state.
    pub state: ComponentState,
    /// Frame size.
    pub size: TerminalSize,
    /// Widget origin.
    pub origin: Origin,
    /// Colour mode.
    pub color: ColorSpec,
    /// Proof facet.
    pub facet: Facet,
    /// Every cell in row-major order, including continuations.
    pub cells: Vec<CellObs>,
    /// Cursor position and visibility (`None` is hidden).
    pub cursor: Option<Position>,
    /// Focused control.
    pub focus: Option<Id>,
    /// Hovered control.
    pub hover: Option<Id>,
    /// Published hit rectangles.
    pub hits: Vec<HitObs>,
}

impl FrameObservation {
    /// Snapshot a presented buffer and runtime facts.
    #[expect(
        clippy::too_many_arguments,
        reason = "one capture records the complete fixture-frame schema"
    )]
    pub fn capture(
        family: Family,
        state: ComponentState,
        size: TerminalSize,
        origin: Origin,
        color: ColorSpec,
        facet: Facet,
        buffer: &Buffer,
        cursor: Option<Position>,
        focus: Option<Id>,
        hover: Option<Id>,
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
        mark_continuations(&mut cells, size);
        Ok(Self {
            family,
            state,
            size,
            origin,
            color,
            facet,
            cells,
            cursor,
            focus,
            hover,
            hits,
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

    /// Cell count must equal cols x rows.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        let expected = usize::from(self.size.cols).saturating_mul(usize::from(self.size.rows));
        self.cells.len() == expected
    }

    /// True when some cell is a wide-grapheme continuation cell.
    #[must_use]
    pub fn has_continuation(&self) -> bool {
        self.cells.iter().any(|cell| cell.continuation)
    }
}

/// Mark wide-grapheme continuations: a blank cell preceded on its row by a
/// width-2 symbol. Paint-time buffers carry no skip flag (`Skip` is set only
/// by `Buffer::diff`), so the predecessor width is the honest signal.
fn mark_continuations(cells: &mut [CellObs], size: TerminalSize) {
    let cols = usize::from(size.cols);
    for y in 0..usize::from(size.rows) {
        for x in 0..cols.saturating_sub(1) {
            let Some(current) = cells.get(y.saturating_mul(cols).saturating_add(x)) else {
                continue;
            };
            if junie_tui::width(current.symbol.as_str()) != 2 {
                continue;
            }
            let Some(next) =
                cells.get_mut(y.saturating_mul(cols).saturating_add(x).saturating_add(1))
            else {
                continue;
            };
            if next.symbol == " " || next.symbol.is_empty() {
                next.continuation = true;
            }
        }
    }
}
