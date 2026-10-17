//! One tabular engine for row-table and editable cell-grid presentation with separate read and edit contracts.

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

pub(crate) mod collection {
    pub use termrock_collections::*;
    pub use termrock_runtime::{CellDecor, CellUi, EmptyState, RowDecor, RowTotal};
}

pub(crate) mod input {
    pub use termrock_fields::{BlurPolicy, TextAction, TextInput, TextInputState};
}

pub(crate) mod controls {
    pub use termrock_controls::Button;
}

pub(crate) mod progress {
    pub use termrock_feedback::Spinner;
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at};

pub mod grid;

pub use grid::{
    CellAction, CellRef, Column, ColumnKey, EditIntent, GRID_MAX_COLUMNS, Grid, GridAction,
    GridCell, GridCmd, GridColumnFit, GridCursorError, GridEditor, GridGutter, GridHeaderSizing,
    GridModel, GridOverflowIndicator, GridSortIndicator, GridState, NavUnit, SortDir, WidthSample,
    WidthSampleError,
};
