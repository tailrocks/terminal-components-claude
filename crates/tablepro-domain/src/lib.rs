//! Domain model for the `TablePro` application: connections, switcher
//! index, result grids, and pending-edit bookkeeping.

#![forbid(unsafe_code)]

pub mod connections;
pub mod domain;
pub mod filter_editor;
pub mod grid_model;
pub mod model;
pub mod tabs;

pub use connections::{ConnectionDraft, ConnectionsScreen, form_fields};
pub use domain::{PendingEdits, ResultGrid};
pub use filter_editor::{Filter, FilterOp};
pub use grid_model::{ResultGridModel, StructureModel, TableGridModel, preview_for};
pub use model::{
    Completion, CompletionBatch, CompletionKind, History, SwitchItem, SwitchTarget, SwitcherIndex,
    auto_trigger, complete, completion_batch,
};
