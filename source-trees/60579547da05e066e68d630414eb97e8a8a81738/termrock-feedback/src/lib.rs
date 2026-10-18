//! Spinner, ProgressBar, Meter, StatusBar, HintBar and KeyHint using shared motion and style contracts.

pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_theme as theme;

pub use termrock_core::action::ActionKey;

pub(crate) use termrock_runtime::{PartStyle, SlotFn, first_row, shift};

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
