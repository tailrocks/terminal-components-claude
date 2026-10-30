pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as action;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_core::validate;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_theme as theme;

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at};

pub mod grid;
pub mod grid_viewport;
pub mod table;

pub use grid_viewport::{
    GridBarPolicy, GridContentChrome, GridFocusPolicy, GridViewport, GridViewportPolicy,
    paint_grid_focus_bar, paint_grid_horizontal_bar, paint_grid_vertical_bar, solve_grid_viewport,
};

pub use grid::{
    CellAction, CellRef, Column, ColumnKey, EditIntent, GRID_MAX_COLUMNS, Grid, GridAction,
    GridCell, GridCmd, GridColumnFit, GridCursorError, GridEditor, GridGutter, GridHeaderSizing,
    GridModel, GridOverflowIndicator, GridSortIndicator, GridState, NavUnit, SortDir, WidthSample,
    WidthSampleError,
};

pub use table::{Table, TableAction, TableColumn, TableSort, TableState};
