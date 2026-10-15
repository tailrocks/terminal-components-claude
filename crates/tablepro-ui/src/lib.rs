//! `TablePro` application UI shell: workbench, tabs, connection screen,
//! quick switcher, and CLI entry point, built only on the public `termrock`
//! facade.

#![forbid(unsafe_code)]

pub mod app;
pub mod cli;
pub mod quick_switcher;
pub mod workbench;

pub use tablepro_demo::db;
pub use tablepro_domain::{
    connections, domain, filter_editor, grid_model, model, tabs,
};
pub use tablepro_sql::sql;

pub use app::{MIN_HEIGHT, MIN_WIDTH, QueryOutcome, Screen, Surface, TableProApp, run_with};
pub use cli::run;
/// Stable id for the connection name field.
pub const CONNECTION_NAME: termrock::Id = tablepro_domain::connections::field::NAME;
pub use connections::{ConnectionDraft, ConnectionsScreen, form_fields};
pub use db::{Catalog, ColType, Connection, SafeMode, Table, Value};
pub use domain::ResultGrid;
pub use model::{
    Completion, CompletionBatch, CompletionKind, History, SwitchItem, SwitchTarget, SwitcherIndex,
    auto_trigger, complete, completion_batch,
};
pub use sql::{Decision, ParseError, ResultSet, gate, parse};
pub use tabs::{ExplorerItem, GridView, HistoryTab, QueryTab, Tab, TabKey, TabRecord, TableTab};
pub use workbench::Workbench;
