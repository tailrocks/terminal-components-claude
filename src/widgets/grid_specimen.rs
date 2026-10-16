//! [`DataGrid`] as a [`Specimen`]: a States grid draws a short copy of a
//! grid (its header and first rows, and its pending-change bar when it has
//! changes) in a forced state.
//!
//! The sample keeps the grid's columns, values, pending changes and
//! settings and sets only what the state is about: the hovered or pressed
//! row (row 2), marked rows, an open cell edit, a cell and row error, no
//! rows, a load in progress, or a value too long for its column.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;
use crate::ui::specimen::{Specimen, SpecimenState};
use crate::widgets::grid::{CellValue, DataGrid, GridRows, RowTotal};

/// Rows a sample keeps.
const SAMPLE_ROWS: usize = 3;

impl DataGrid {
    /// The first column a sample edits: writable, not the key.
    fn sample_edit_col(&self) -> Option<usize> {
        self.columns.iter().position(|c| !c.read_only && !c.primary)
    }

    /// The first text column, for the overflow sample.
    fn sample_text_col(&self) -> Option<usize> {
        (0..self.columns.len()).find(|&c| {
            matches!(
                self.rows().first().and_then(|r| r.get(c)),
                Some(CellValue::Text(_))
            )
        })
    }
}

impl Specimen for DataGrid {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: SpecimenState) -> bool {
        use SpecimenState::*;
        match state {
            Rest | Hover | Pressed | Focused | FocusedHover | Selected | Error | Empty
            | Loading | Overflow | Narrow => true,
            Editing => self.editable && self.sample_edit_col().is_some(),
            Disabled | Busy => false,
        }
    }

    /// The header and the sample rows (a placeholder row when empty), and
    /// the pending-change bar when the grid has changes.
    fn specimen_height(&self, state: SpecimenState) -> u16 {
        let rows = if state == SpecimenState::Empty {
            // the empty state's title and its gap
            2
        } else if self.pending.is_empty() {
            SAMPLE_ROWS.min(self.rows().len().max(1)) as u16
        } else {
            self.rows().len().max(1) as u16
        };
        let bar = if self.pending.is_empty() { 0 } else { 2 };
        1 + rows + bar
    }

    fn specimen(
        &self,
        state: SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut g = self.clone();
        // a longer grid is cut to its first rows; one with pending changes
        // (keyed by source row) is drawn whole
        if g.rows().len() > SAMPLE_ROWS && g.pending.is_empty() {
            let rows = g.rows()[..SAMPLE_ROWS].to_vec();
            g.set_rows(GridRows {
                rows,
                total: RowTotal::Exact(SAMPLE_ROWS),
                more: false,
            });
        }
        g.edit = None;
        g.choice = None;
        g.cursor = (0, g.cursor.1);
        let row = 1.min(g.len().saturating_sub(1));
        let v = state.visual();
        if v.hovered {
            ctx.interaction.hover = Some(g.cell_id(row, g.cursor.1));
        }
        if v.pressed {
            ctx.interaction.pressed = Some(g.cell_id(row, g.cursor.1));
        }
        match state {
            SpecimenState::Selected => {
                g.selected_rows.insert(g.source_row(row));
            }
            SpecimenState::Editing => {
                if let Some(c) = g.sample_edit_col() {
                    g.cursor.1 = c;
                    let _ = g.begin_edit();
                }
            }
            SpecimenState::Error => {
                let c = g.sample_edit_col().unwrap_or(0);
                g.cell_errors
                    .insert((g.source_row(0), c), "Enter a valid value".into());
                g.row_errors
                    .insert(g.source_row(0), "Enter a valid value".into());
            }
            SpecimenState::Empty => g.set_rows(GridRows {
                rows: Vec::new(),
                total: RowTotal::Exact(0),
                more: false,
            }),
            SpecimenState::Loading => g.set_loading(true),
            SpecimenState::Overflow => {
                if let Some(c) = g.sample_text_col() {
                    let mut rows = g.rows().to_vec();
                    if let Some(CellValue::Text(s)) = rows.first_mut().and_then(|r| r.get_mut(c)) {
                        s.push_str(" and a value longer than its column");
                    }
                    let n = rows.len();
                    g.set_rows(GridRows {
                        rows,
                        total: RowTotal::Exact(n),
                        more: false,
                    });
                }
            }
            _ => {}
        }
        g.render(area, buf, ctx, bg);
        // samples never move the terminal cursor
        ctx.cursor = None;
    }
}
