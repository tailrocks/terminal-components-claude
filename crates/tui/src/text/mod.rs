//! Text measurement, editing, matching and spans (`COMPONENT_ARCHITECTURE.md` §15, §18.1).

pub(crate) mod buffer;
pub(crate) mod editor;
pub(crate) mod fuzzy;
pub(crate) mod measure;
pub(crate) mod span;

pub use buffer::CursorPos;
pub(crate) use buffer::TextBuffer;
pub(crate) use editor::TextEditorCore;
pub use editor::{EditAction, EditOutcome, Extend, Motion};
pub use fuzzy::fuzzy;
pub use measure::{truncate, truncate_middle, width, wrap, wrapped_rows};
pub use span::Span;
