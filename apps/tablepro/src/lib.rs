//! `TablePro` application package.
//!
//! Database semantics stay in application-owned adapters; terminal behavior
//! is reached only through the public `tui-next` facade.
#![forbid(unsafe_code)]
#![allow(missing_docs, unused_qualifications)]
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::arithmetic_side_effects,
    unused_imports
)]
#![cfg_attr(
    test,
    allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines,
        clippy::module_name_repetitions,
        reason = "deterministic application tests use direct assertions"
    )
)]

pub mod connections;
pub mod db;
pub mod domain;
pub mod filter_editor;
pub mod grid_model;
pub mod model;
pub mod sql;
pub mod tabs;
pub mod workbench;

mod app;

pub use app::{MIN_HEIGHT, MIN_WIDTH, QueryOutcome, Screen, Surface, TableProApp, run};

#[cfg(test)]
mod tablepro {
    use super::{
        db::{self, ColType, SafeMode, Value},
        domain::{PendingEdits, ResultGrid},
        sql::{self, Statement},
    };
    use tui_next::{ColumnKey, EditIntent, GridEditor, GridModel, ItemKey, SortDir};

    #[test]
    fn grid_adapter_keeps_every_pending_change_capability() {
        let mut pending = PendingEdits::new(vec![
            vec![Value::Int(7), Value::Text("Ada".to_owned())],
            vec![Value::Int(8), Value::Text("Grace".to_owned())],
        ]);
        assert!(pending.set(0, 1, Value::Text("Ada Lovelace".to_owned())));
        let inserted = pending.insert_row(2);
        assert!(pending.set(inserted, 0, Value::Int(9)));
        assert!(pending.set(inserted, 1, Value::Text("Lin".to_owned())));
        assert!(pending.delete_row(1));
        assert_eq!(pending.dirty_rows(), vec![0, 1, 2]);
        assert!(pending.is_inserted(inserted));
        assert!(pending.is_deleted(1));

        pending.reorder(&[2, 0, 1]);
        assert_eq!(pending.value(0, 0), Some(&Value::Int(9)));
        assert!(pending.is_inserted(0));

        pending.clear();
        assert!(pending.dirty_rows().is_empty());
    }

    #[test]
    fn view_grid_is_read_only_with_a_reason() {
        let catalog = db::Catalog::acme_prod();
        let statement = match sql::parse("SELECT status FROM orders LIMIT 3") {
            Ok(Statement::Select(select)) => select,
            Ok(_) => {
                panic!("projection must parse as SELECT");
            }
            Err(error) => {
                panic!("query must parse: {}", error.message);
            }
        };
        let result = match sql::run_select(&catalog, &statement) {
            Ok(result) => result,
            Err(error) => {
                panic!("query must execute: {}", error.message);
            }
        };
        let grid = ResultGrid::from_result(&result);

        assert!(!grid.is_editable());
        assert!(grid.read_only_reason().is_some());
        assert!(matches!(grid.edit_intent(0, 0), EditIntent::Refuse { .. }));
    }

    #[test]
    fn result_grid_sorts_locally_and_refuses_edits() {
        let result = sql::ResultSet {
            columns: vec![("id".to_owned(), ColType::Int)],
            rows: vec![vec![Value::Int(20)], vec![Value::Int(10)]],
            total: 2,
            source: Some("public.orders".to_owned()),
            duration_ms: 1,
            editable: false,
        };
        let mut grid = ResultGrid::from_result(&result);
        let ten_key = grid.row_key(1);

        grid.sort(ColumnKey::num(1), SortDir::Asc);

        assert_eq!(grid.row_key(0), ten_key);
        assert_eq!(grid.cell(0, 0).map(|cell| cell.text), Some("10"));
        assert!(matches!(grid.edit_intent(0, 0), EditIntent::Refuse { .. }));
        assert_eq!(ten_key, ItemKey::num(2));
    }

    #[test]
    fn query_safety_gate_preserves_safe_mode_policy() {
        let select = match sql::parse("SELECT * FROM orders") {
            Ok(statement) => statement,
            Err(error) => {
                panic!("query must parse: {}", error.message);
            }
        };
        let destructive = match sql::parse("DELETE FROM orders") {
            Ok(statement) => statement,
            Err(error) => {
                panic!("query must parse: {}", error.message);
            }
        };
        let scoped_write = match sql::parse("UPDATE orders SET status = 'paid' WHERE id = 7") {
            Ok(statement) => statement,
            Err(error) => {
                panic!("query must parse: {}", error.message);
            }
        };

        assert_eq!(sql::gate(SafeMode::Silent, &select), sql::Decision::Run);
        assert_eq!(
            sql::gate(SafeMode::Silent, &destructive),
            sql::Decision::Confirm { deliberate: false }
        );
        assert_eq!(
            sql::gate(SafeMode::ReadOnly, &scoped_write),
            sql::Decision::Deny
        );
    }
}
