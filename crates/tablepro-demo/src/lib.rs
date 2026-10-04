//! `tablepro-demo`
//!
//! In-memory demo database/catalog, deterministic query outcomes and scenario data.

#![forbid(unsafe_code)]

mod db;
mod query;

pub use db::{connections, rows};
pub use query::{ExecError, PlanNode, ROW_CAP, cmp_values, explain, plan_text, run_select};

// Re-export domain types commonly used with demo database
pub use tablepro_domain::{Catalog, ColType, Connection, SafeMode, Table, Value};
