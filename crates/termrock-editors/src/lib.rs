//! CodeEditor and DiffView presentation, selections, completion/highlighting contracts and responsive diff modes.

pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as action;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as keys;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as layout;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_render as render;
pub(crate) use termrock_runtime as ui;
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as hit;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_runtime as capture;
pub(crate) use termrock_runtime as layer;
pub(crate) use termrock_text as text;
pub(crate) use termrock_text::secret;
pub(crate) use termrock_theme as theme;

pub(crate) mod collection {
    pub use termrock_collections::*;
    pub use termrock_runtime::empty;
    pub use termrock_runtime::{
        CellDecor, CellUi, ColumnsUi, EmptyState, MAX_COLUMNS, RowDecor, RowFn, RowTotal, RowUi,
        Status,
    };
}

pub(crate) mod viewport {
    pub use termrock_viewport::{
        CellPos, ProjectedText, TextViewport, ViewportAction, ViewportCmd, ViewportLine,
        ViewportState,
    };
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub(crate) use termrock_runtime::FieldControl;
pub(crate) use termrock_runtime::field_control;

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::ScrollRegion;
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{
    Acc, PartPainter, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket,
    shift,
};

pub mod code;
pub mod diff;

pub use code::{
    CodeAction, CodeCmd, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Highlighter,
    Segmenter, TabBehavior,
};
pub use diff::{DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState};
