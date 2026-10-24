#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![expect(
    clippy::too_many_lines,
    reason = "application rendering functions are organized by screen and component"
)]

pub mod app;
pub mod cli;
pub mod footer;
pub mod help_keys;
pub mod quick_switcher;
pub mod safe_mode;
pub mod tab_list;
pub mod workbench;

pub use tablepro_demo::db;
pub use tablepro_domain::{connections, domain, filter_editor, grid_model, model, tabs};
pub use tablepro_sql::sql;

pub use app::{MIN_HEIGHT, MIN_WIDTH, QueryOutcome, Screen, Surface, TableProApp, run_with};
pub use cli::run;
pub const CONNECTION_NAME: termrock::Id = connections::field::NAME;
pub use connections::{ConnectionDraft, ConnectionsScreen, form_fields};
pub use db::{Catalog, ColType, Connection, SafeMode, Table, Value};
pub use domain::{PendingEdits, ResultGrid};
pub use filter_editor::{Filter, FilterOp};
pub use grid_model::preview_for;
pub use model::{
    Completion, CompletionBatch, CompletionKind, History, SwitchItem, SwitchTarget, SwitcherIndex,
    auto_trigger, complete, completion_batch,
};
pub use sql::{Decision, ParseError, ResultSet, gate, parse};
pub use tabs::{ExplorerItem, GridView, HistoryTab, QueryTab, Tab, TabKey, TabRecord, TableTab};
pub use workbench::Workbench;
