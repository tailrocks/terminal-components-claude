//! Termrock Runtime: context, interaction ownership, hit mapping, and layer stacking.

pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as response;
pub(crate) use termrock_core as action;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as diagnostics;
pub(crate) use termrock_text as text;
pub use termrock_text::secret;
pub(crate) use termrock_theme as theme;

pub use crate::support::{
    Acc, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket, shift,
};
pub use termrock_layout as layout;
pub use termrock_layout as measure;
pub mod author;
pub mod capture;
pub mod cursor;
pub mod decor;
pub mod empty;
pub mod field_control;
pub mod focus;
pub mod hit;
pub mod keymap;
pub mod layer;
pub mod rowui;
pub mod runtime;
pub mod support;
pub mod scroll_region;
pub mod ui;

// Re-exports
pub use author::*;
pub use capture::*;
pub use decor::*;
pub use empty::*;
pub use focus::*;
pub use hit::*;
pub use keymap::*;
pub use layer::*;
pub use rowui::*;
pub use runtime::*;
pub use scroll_region::*;
pub use termrock_core::event::Chord;
pub use termrock_core::intent::Intent;
pub use ui::cx;
pub use ui::*;

#[cfg(any(test, feature = "testing"))]
pub use runtime::stub;
