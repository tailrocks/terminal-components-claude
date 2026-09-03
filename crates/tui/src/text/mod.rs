//! Text measurement, editing, matching and spans (`COMPONENT_ARCHITECTURE.md` §15, §18.1).

pub(crate) mod buffer;
pub(crate) mod editor;
pub(crate) mod fuzzy;
pub(crate) mod measure;
pub(crate) mod span;

pub use buffer::{CursorPos, TextBuffer};
pub use editor::{EditAction, EditOutcome, Extend, Motion, TextEditorCore};
pub use fuzzy::fuzzy;
pub use measure::{
    grapheme_width, is_word_char, thousands, truncate, truncate_middle, width, wrap,
};
pub use span::Span;
