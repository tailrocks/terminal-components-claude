//! Basic actionable/decorative controls: Button, Brand, choices, Panel, SplitPane, Props, Empty and TooSmall.

pub(crate) use termrock_core as action;
pub(crate) use termrock_core as event;
pub(crate) use termrock_core as id;
pub(crate) use termrock_core as intent;
pub(crate) use termrock_core as keys;
pub(crate) use termrock_core as response;
pub(crate) use termrock_layout as layout;
pub(crate) use termrock_layout as measure;
pub(crate) use termrock_theme as theme;
pub(crate) use termrock_text as text;
pub(crate) use termrock_text::secret;
pub(crate) use termrock_render as render;
pub(crate) use termrock_collections as scroll;
pub(crate) use termrock_runtime as ui;
pub(crate) use termrock_runtime as runtime;
pub(crate) use termrock_runtime as focus;
pub(crate) use termrock_runtime as hit;
pub(crate) use termrock_runtime as keymap;
pub(crate) use termrock_runtime as capture;
pub(crate) use termrock_runtime as layer;

pub(crate) mod collection {
    pub use termrock_collections::*;
    pub use termrock_runtime::{
        CellDecor, CellUi, ColumnsUi, EmptyState, MAX_COLUMNS, RowDecor, RowFn, RowTotal, RowUi,
        Status,
    };
}

pub(crate) mod form {
    pub use termrock_runtime::InheritedFormState;
}

pub(crate) use termrock_runtime::field_control;
pub(crate) use termrock_runtime::FieldControl;

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::ScrollRegion;
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{
    Acc, PartPainter, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket,
    shift,
};

pub mod brand;
pub mod button;
pub mod choice;
pub mod empty;
pub mod panel;
pub mod props;
pub mod split;
pub mod too_small;

pub use brand::Brand;
pub use button::{Button, ButtonCmd};
pub use choice::{
    Checkbox, ChoiceCmd, LabelRadio, RadioGroup, RadioGroupAction, RadioGroupState, Toggle,
};
pub use empty::Empty;
pub use panel::{Panel, PanelKind};
pub use props::{Props, PropsAction, PropsCmd, PropsList, PropsRow, PropsState, PropsValue};
pub use split::{SplitAction, SplitCmd, SplitPane, SplitPaneState};
pub use too_small::TooSmall;
