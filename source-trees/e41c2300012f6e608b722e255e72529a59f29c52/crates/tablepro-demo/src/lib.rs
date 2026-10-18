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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_connections() {
        let conns = connections();
        assert!(!conns.is_empty());
        assert_eq!(conns[0].name, "Local PostgreSQL");
    }
}
