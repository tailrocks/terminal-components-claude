//! Form and Wizard orchestration, field order/eligibility, generic validation and typed submit/navigation requests.

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
    pub use termrock_collections::*;
}

pub(crate) mod button {
    pub use termrock_controls::Button;
}

pub(crate) mod chip {
    pub use termrock_navigation::{ChipBarState, LabelChips};
}

pub(crate) mod choice {
    pub use termrock_controls::{Checkbox, LabelRadio, RadioGroupAction, RadioGroupState, Toggle};
}

pub(crate) mod input {
    pub use termrock_fields::{ErrorState, TextAction, TextInput, TextInputState, discard_error};
}

pub(crate) mod keyhint {
    pub use termrock_feedback::ChordText;
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub(crate) mod select {
    pub use termrock_overlays::{LabelSelect, SelectAction, SelectState};
}

pub(crate) mod textarea {
    pub use termrock_fields::{TextArea, TextAreaState};
}

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};
pub use termrock_theme::ColorLevel;

pub(crate) use termrock_runtime::{Acc, PartStyle, SlotFn};

pub mod form;
pub mod wizard;

pub use form::{
    EnterPolicy, FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec, Form, FormAction, FormData,
    FormState, GroupKey,
};
pub use wizard::{Wizard, WizardAction, WizardCmd, WizardState, WizardStep};
