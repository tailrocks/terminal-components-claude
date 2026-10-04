//! Keyed borrowed collections, reconciliation, selection, and scroll state.

pub(crate) use termrock_core::id;
pub(crate) use termrock_core::response;
pub(crate) use termrock_text as text;

pub mod collection;
pub mod scroll;

pub use collection::*;
pub use scroll::*;
