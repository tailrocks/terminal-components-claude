pub(crate) use termrock_theme as theme;

pub mod buffer;
pub mod clusters;
pub mod editor;
pub mod fuzzy;
pub mod measure;
pub mod secret;
pub mod span;
pub mod undo;

pub use buffer::{CursorPos, TextBuffer};
pub use editor::{EditAction, EditOutcome, Extend, Motion, TextEditorCore};
pub use fuzzy::{FuzzyBoundary, fuzzy, fuzzy_with_boundary};
pub use measure::{
    middle_parts, truncate, truncate_middle, truncate_parts, width, wrap, wrapped_rows,
};
pub use secret::{CellWriter, Secret, SecretPolicy};
pub use span::Span;
pub use termrock_core::secret::wipe_string;
pub use undo::{EditKind, UndoStack};
