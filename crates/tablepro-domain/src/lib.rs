//! `tablepro-domain`
//!
//! Connection, schema, query, result, history, safety and pending-change domain values.
//! Pure data structures without dependency on TUI or terminal widgets.

#![forbid(unsafe_code)]

mod connection;
mod history;
mod pending;
mod schema;
mod value;

pub use connection::{ConnectOutcome, Connection, Engine, Environment, SafeMode};
pub use history::{History, HistoryEntry, HistorySource};
pub use pending::{PendingEdits, PendingRow};
pub use schema::{Catalog, ColType, Column, Constraint, Index, ObjectKind, Table};
pub use value::{ResultSet, Value, cmp_values};
