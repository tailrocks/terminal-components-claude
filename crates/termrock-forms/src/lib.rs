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

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};
pub use termrock_theme::ColorLevel;

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn};

pub mod form;
pub mod form_dialog;
pub mod form_layout;
pub mod wizard;

pub use form::{
    EnterPolicy, FieldKind, FieldMut, FieldRef, FieldSizingPolicy, FieldSpan, FieldSpec, Form,
    FormAction, FormData, FormState, GroupKey,
};
pub use form_dialog::{
    FormDialog, FormDialogAction, FormDialogLayoutPolicy, FormDialogState, FormDialogSubmitPolicy,
};
pub use form_layout::{FormLane, FormLayout, FormLayoutError, FormRow};
pub use wizard::{Wizard, WizardAction, WizardCmd, WizardState, WizardStep};
