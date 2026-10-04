//! Spinner, ProgressBar, Meter, StatusBar, HintBar and KeyHint using shared motion and style contracts.

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
pub(crate) use termrock_theme as theme;

pub use termrock_core::action::ActionKey;

pub(crate) use termrock_runtime::{
    Acc, PartPainter, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket,
    shift,
};

pub(crate) mod collection {
    pub use termrock_runtime::Status;
}

pub mod hintbar;
pub mod keyhint;
pub mod meter;
pub mod progress;
pub mod status;

pub use hintbar::{DerivedHintBar, HintBar};
pub use keyhint::{ChordText, HintText, KeyHint};
pub use meter::{Meter, MeterTone, MeterVisual};
pub use progress::{ProgressBar, Spinner};
pub use status::{Emphasis, Group, MAX_ITEMS, StatusAction, StatusBar, StatusItem};
