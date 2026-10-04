//! CodeEditor and DiffView presentation, selections, completion/highlighting contracts and responsive diff modes.

pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_theme as theme;

pub(crate) mod viewport {
    pub use termrock_viewport::{
        ProjectedText, TextViewport, ViewportAction, ViewportCmd, ViewportState,
    };
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at, first_row};

pub mod code;
pub mod diff;

pub use code::{
    CodeAction, CodeCmd, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Highlighter,
    Segmenter, TabBehavior,
};
pub use diff::{DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState};
