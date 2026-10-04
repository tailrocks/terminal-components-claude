//! List, NavList, Tree, Tabs, ChipBar, Steps, PropsList and filtered-list composition over shared collections.

pub(crate) use termrock_collections as scroll;
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
pub(crate) use termrock_text::secret;
pub(crate) use termrock_theme as theme;

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

pub(crate) use termrock_runtime::FieldControl;
pub(crate) use termrock_runtime::field_control;

pub use termrock_core::action::ActionKey;
pub use termrock_runtime::ScrollRegion;
pub use termrock_text::{Secret, SecretPolicy};

pub(crate) use termrock_runtime::{
    Acc, PartPainter, PartStyle, SlotFn, cell_at, first_row, overlay_chrome, paint_pressed_bracket,
    shift,
};

pub mod chip;
pub mod filter_list;
pub mod item;
pub mod list;
pub mod nav_list;
pub mod steps;
pub mod tabs;
pub mod tree;

pub(crate) mod picker {
    pub use super::item::*;
}

pub(crate) mod scroll_region {
    pub use termrock_runtime::ScrollRegion;
}

pub use chip::{ChipBar, ChipBarAction, ChipBarCmd, ChipBarState, LabelChips};
pub use filter_list::{FilterList, FilterListAction, FilterListCmd, FilterListState, FilterPolicy};
pub use item::{AsItem, Item, ItemColumns, ItemRow, ItemRowLayout};
pub use list::{List, ListAction, ListCmd, ListState};
pub use nav_list::{BadgeFn, NavList, NavListAction, NavListCmd, NavListState, NavMode};
pub use steps::{StepState, Steps, StepsAction, StepsCmd, StepsState};
pub use tabs::{Tabs, TabsAction, TabsCmd, TabsState};
pub use tree::{
    NodeKind, Tree, TreeAction, TreeBranchActivation, TreeBranchClick, TreeCmd, TreeNode, TreeState,
};

// Re-export PropsList types from termrock-controls per CRATES.md
pub use termrock_controls::{PropsAction, PropsCmd, PropsList, PropsRow, PropsState, PropsValue};
