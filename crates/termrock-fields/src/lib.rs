//! Field chrome, TextInput and TextArea behavior, drafts, editing policy, generic validation and protected input adapters.

pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as action;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
pub(crate) use termrock_core::validate;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_text as text;
pub(crate) use termrock_text::secret;
pub(crate) use termrock_theme as theme;

pub(crate) mod collection {
    pub use termrock_runtime::{CellUi, Status};
}

pub(crate) use termrock_runtime::field_control;

pub use termrock_core::action::ActionKey;
pub use termrock_core::validate::{FieldError, NoValidate, Validate};
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion, TypingPolicy};
pub use termrock_text::{Secret, SecretPolicy, wipe_string};

pub mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at, first_row, shift};

pub(crate) mod edit_keys;
pub mod field;
pub mod input;
pub mod note;
pub mod textarea;

pub use field::Field;
pub use input::{
    BlurPolicy, EditPhase, ErrorState, TextAction, TextCmd, TextInput, TextInputState,
    discard_error, redacted_text,
};
pub use note::Note;
pub use textarea::{TextArea, TextAreaState};
