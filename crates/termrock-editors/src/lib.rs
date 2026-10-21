pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_theme as theme;

pub use termrock_core::action::ActionKey;
pub use termrock_core::edit_entry::EditEntry;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{CursorPos, Secret, SecretPolicy};

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at, first_row};

pub mod code;
pub mod completion;
pub mod diff;

pub use code::{
    CodeAction, CodeCmd, CodeDiagnostic, CodeEditor, CodeEditorState, CodeFindInfo, CodeSeverity,
    Highlighter, Segmenter, TabBehavior,
};
pub use completion::{
    COMPLETION_MANUAL_CHORD, COMPLETION_NEXT_FOCUS_CHORD, COMPLETION_PREV_FOCUS_CHORD,
    CompletingCodeEditor, CompletingCodeEditorState, CompletionBatch, CompletionProvider,
    CompletionTrigger, CompletionValue, FOCUS_NEXT_ACTION, FOCUS_PREVIOUS_ACTION,
    MANUAL_COMPLETION_ACTION,
};
pub use diff::{DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState};
