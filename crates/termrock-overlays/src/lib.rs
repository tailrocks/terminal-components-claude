//! Dialog, Menu/ContextMenu/MenuBar, Select, Picker/CommandPalette/PickerChain, Completion and HelpOverlay.

pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_core as action;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as response;
#[cfg(test)]
pub(crate) use termrock_core::diagnostics;
pub(crate) use termrock_layout as layout;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_runtime as ui;
#[cfg(test)]
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
#[cfg(test)]
pub(crate) use termrock_runtime as hit;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_runtime as layer;
pub(crate) use termrock_text as text;
pub(crate) use termrock_text::secret;
pub(crate) use termrock_theme as theme;

pub(crate) mod collection {
    pub use termrock_collections::*;
    pub use termrock_runtime::empty;
    pub use termrock_runtime::{EmptyState, RowFn, RowUi, Status};
}

pub(crate) mod form {
    pub use termrock_runtime::InheritedFormState;
}

pub(crate) mod button {
    pub use termrock_controls::Button;
}

pub(crate) mod field {
    pub use termrock_fields::Field;
}

pub(crate) mod input {
    pub use termrock_fields::{TextAction, TextInput, TextInputState, redacted_text};
}

pub(crate) mod keyhint {
    pub use termrock_feedback::{ChordText, HintText};
}

pub(crate) mod filter_list {
    pub use termrock_navigation::{FilterList, FilterListAction, FilterListState, FilterPolicy};
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::{ReferenceState, ReferenceTarget, ScrollRegion};
pub use termrock_text::{Secret, SecretPolicy};
pub use termrock_theme::ColorLevel;

pub(crate) use termrock_runtime::{
    Acc, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket, shift,
};

pub mod completion;
pub mod dialog;
pub mod help;
pub mod menu;
pub mod picker;
pub mod picker_chain;
pub mod select;

pub use completion::{
    Completion, CompletionAction, CompletionCmd, CompletionController, CompletionState,
};
pub use dialog::{Dialog, DialogAction, DialogCmd, DialogState};
pub use help::{HelpAction, HelpCmd, HelpOverlay, HelpOverlayState, HelpSection};
pub use menu::{ContextMenu, Menu, MenuAction, MenuBar, MenuCmd, MenuItem, MenuState};
pub use picker::{
    AsItem, CommandPalette, Item, ItemRow, ItemRowLayout, Picker, PickerAction, PickerState,
    ScopeKey,
};
pub use picker_chain::{
    PickerChain, PickerChainAction, PickerChainCmd, PickerChainState, PickerStage,
};
pub use select::{LabelSelect, Select, SelectAction, SelectCmd, SelectState};
