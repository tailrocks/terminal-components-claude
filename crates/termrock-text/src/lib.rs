//! Text measurement, editing, matching, spans, and secrets (`COMPONENT_ARCHITECTURE.md` §15, §18.1).

pub(crate) use termrock_theme as theme;

pub mod buffer;
pub mod clusters;
pub mod editor;
pub mod fuzzy;
pub mod measure;
pub mod secret;
pub mod span;

pub use buffer::{CursorPos, TextBuffer};
pub use editor::{EditAction, EditOutcome, Extend, Motion, TextEditorCore};
pub use fuzzy::{FuzzyBoundary, fuzzy, fuzzy_with_boundary};
pub use measure::{truncate, truncate_middle, width, wrap, wrapped_rows};
pub use secret::{CellWriter, Secret, SecretPolicy, wipe_string};
pub use span::Span;
