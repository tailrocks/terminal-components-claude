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

pub(crate) use termrock_runtime::field_control;

pub use termrock_core::action::ActionKey;
pub use termrock_core::validate::{FieldError, NoValidate, Validate};
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion, TypingPolicy};
pub use termrock_text::{Secret, SecretPolicy, wipe_string};

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn, cell_at, first_row, shift};

pub mod edit;
pub mod field;
pub mod field_error;
pub mod input;
pub mod note;
pub mod textarea;

pub use edit::{EditCapabilities, EditMenuRequest, TextEditCommand};
pub use field::Field;
pub use field_error::{FieldErrorChrome, paint_field_error};
pub use input::{
    BlurPolicy, EditPhase, ErrorState, FocusSelect, TextAction, TextCmd, TextInput, TextInputState,
    discard_error, redacted_text,
};
pub use note::Note;
pub use textarea::{TextArea, TextAreaState};
