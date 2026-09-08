//! Text measurement, editing, matching and spans (`COMPONENT_ARCHITECTURE.md` §15, §18.1).

pub(crate) mod buffer;
pub(crate) mod editor;
pub(crate) mod fuzzy;
pub(crate) mod measure;
pub(crate) mod span;

pub use buffer::{CursorPos, TextBuffer};
pub use editor::{EditAction, EditOutcome, Extend, Motion, TextEditorCore};
pub use fuzzy::{FuzzyBoundary, fuzzy, fuzzy_with_boundary};
pub use measure::{truncate, truncate_middle, width, wrap, wrapped_rows};
pub use span::Span;
