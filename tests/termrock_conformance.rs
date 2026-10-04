//! Termrock Conformance Suite and Protected Path Invariants.

#![allow(unused_imports, unused_variables, dead_code)]

#[path = "conformance/drivers.rs"]
mod drivers;
#[path = "conformance/registry.rs"]
mod registry;

#[path = "../src/bin/tablepro/db.rs"]
pub mod db;
#[path = "../src/bin/tablepro/sql.rs"]
pub mod sql;
pub mod tabs {
    use crate::db::{ColType, Value};
    use junie_tui::widgets::grid::{CellKind, CellValue};

    pub fn from_cell(v: &CellValue) -> Value {
        match v {
            CellValue::Null | CellValue::Default => Value::Null,
            CellValue::Text(s) => Value::Text(s.clone()),
            CellValue::Int(i) => Value::Int(*i),
            CellValue::Num(n) => Value::Num(*n),
            CellValue::Bool(b) => Value::Bool(*b),
            CellValue::Json(j) => Value::Json(j.clone()),
        }
    }

    pub fn cell_kind(ty: ColType) -> CellKind {
        match ty {
            ColType::Int | ColType::Numeric => CellKind::Number,
            ColType::Bool => CellKind::Bool,
            ColType::Json => CellKind::Json,
            _ => CellKind::Text,
        }
    }

    pub fn to_cell(v: &Value) -> CellValue {
        match v {
            Value::Null => CellValue::Null,
            Value::Int(i) => CellValue::Int(*i),
            Value::Num(n) => CellValue::Num(*n),
            Value::Bool(b) => CellValue::Bool(*b),
            Value::Text(s) => CellValue::Text(s.clone()),
            Value::Json(j) => CellValue::Json(j.clone()),
        }
    }
}
#[path = "../src/bin/tablepro/model.rs"]
pub mod model;

use std::path::Path;

fn make_col(name: &str, ty: db::ColType, primary: bool, nullable: bool) -> db::Column {
    db::Column {
        name: name.into(),
        ty,
        nullable,
        default: None,
        primary,
        references: None,
        enum_values: vec![],
        generated: false,
    }
}

fn make_table(schema: &str, name: &str, columns: Vec<db::Column>) -> db::Table {
    db::Table {
        schema: schema.into(),
        name: name.into(),
        kind: db::ObjectKind::Table,
        columns,
        indexes: vec![],
        constraints: vec![],
        triggers: vec![],
        row_count: 0,
        comment: None,
    }
}

#[test]
fn protected_paths() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // snapshots/ must NOT exist (cut over to baselines/tuiscotti-v1)
    assert!(
        !manifest_dir.join("snapshots").exists(),
        "legacy snapshots/ must remain deleted"
    );

    // Protected paths must remain intact
    assert!(manifest_dir.join("src/bin").exists(), "src/bin must exist");
    assert!(
        manifest_dir.join("tests/visual_baseline").exists(),
        "tests/visual_baseline must exist"
    );
    assert!(
        manifest_dir.join("Cargo.toml").exists(),
        "Cargo.toml must exist"
    );
    assert!(
        manifest_dir.join("Cargo.lock").exists(),
        "Cargo.lock must exist"
    );
}

#[test]
fn test_termrock_conformance_pins_and_manifest() {
    assert_eq!(registry::HISTORICAL_BASELINE_TAG, "visual-baseline");
    assert_eq!(
        registry::HISTORICAL_BASELINE_COMMIT,
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    );

    let manifest = registry::RequiredCasesManifest::load();
    assert_eq!(manifest.components_count, 45);
    assert_eq!(manifest.foundations_count, 12);
    assert_eq!(manifest.family_dispositions_count, 54);
    assert_eq!(manifest.total_cases_count, 524);
}

#[test]
fn tablepro_preview_sql_oracle() {
    use db::{Catalog, ColType};
    use junie_tui::widgets::grid::CellValue;
    use model::{TermrockGridPendingState, preview_sql};

    let cat = Catalog::acme_prod();
    let orders_table = cat.find(None, "orders").expect("orders table");
    let cols: Vec<(String, ColType)> = orders_table
        .columns
        .iter()
        .map(|c| (c.name.clone(), c.ty))
        .collect();

    let base_rows: Vec<Vec<CellValue>> = db::rows(orders_table, 0, 5)
        .iter()
        .map(|r| r.iter().map(tabs::to_cell).collect())
        .collect();

    // 1. Update/Insert/Delete Ordering Oracle:
    // Updates must precede Inserts, Inserts must precede Deletes.
    let status_idx = cols.iter().position(|c| c.0 == "status").unwrap();
    let curr_idx = cols.iter().position(|c| c.0 == "currency").unwrap();

    let mut state = TermrockGridPendingState::new(base_rows.clone());
    // Edit row 1
    state.record_edit(1, status_idx, CellValue::Text("shipped".into()));
    // Insert row 5
    state.insert_row(
        5,
        vec![
            (0, CellValue::Text("ord-new-99".into())),
            (1, CellValue::Text("cust-01".into())),
            (status_idx, CellValue::Text("pending".into())),
            (curr_idx, CellValue::Text("USD".into())),
        ],
    );
    // Delete row 3
    state.mark_deleted(3);

    let sqls = preview_sql(orders_table, &cols, &state);
    assert_eq!(
        sqls.len(),
        3,
        "must emit exactly 3 statements: UPDATE, INSERT, DELETE"
    );
    assert!(
        sqls[0].starts_with("UPDATE public.orders SET "),
        "first statement must be UPDATE: {}",
        sqls[0]
    );
    assert!(
        sqls[1].starts_with("INSERT INTO public.orders "),
        "second statement must be INSERT: {}",
        sqls[1]
    );
    assert!(
        sqls[2].starts_with("DELETE FROM public.orders WHERE "),
        "third statement must be DELETE: {}",
        sqls[2]
    );

    // 2. Changed-Column Ordering Oracle:
    // Multiple column edits on the same row must be ordered by column index.
    let mut multi_col_state = TermrockGridPendingState::new(base_rows.clone());
    // Edit in reverse index order: currency (larger index) then status (smaller index)
    multi_col_state.record_edit(0, curr_idx, CellValue::Text("EUR".into()));
    multi_col_state.record_edit(0, status_idx, CellValue::Text("completed".into()));
    let multi_col_sql = preview_sql(orders_table, &cols, &multi_col_state);
    assert_eq!(multi_col_sql.len(), 1);
    let expected_pattern = format!(
        "UPDATE public.orders SET status = 'completed', currency = 'EUR' WHERE id = '{}';",
        base_rows[0][0].text()
    );
    assert_eq!(multi_col_sql[0], expected_pattern);

    // 3. Original-Key Predicates Oracle:
    // WHERE clause must retain original PK values even if edits target other cells.
    assert!(sqls[0].contains(&format!("WHERE id = '{}';", base_rows[1][0].text())));
    assert!(sqls[2].contains(&format!("WHERE id = '{}';", base_rows[3][0].text())));

    // 4. NULL and DEFAULT Handling & Literal Escaping Oracle:
    let mut literal_state = TermrockGridPendingState::new(base_rows.clone());
    // Null value
    literal_state.record_edit(0, status_idx, CellValue::Null);
    // Escaped single quotes
    literal_state.record_edit(
        1,
        status_idx,
        CellValue::Text("O'Reilly's 'Special'".into()),
    );
    // JSON literal with quotes
    let json_val = r#"{"flag": true, "note": "it's done"}"#;
    literal_state.record_edit(2, status_idx, CellValue::Json(json_val.into()));
    let lit_sqls = preview_sql(orders_table, &cols, &literal_state);
    assert_eq!(lit_sqls.len(), 3);
    assert!(
        lit_sqls[0].contains("SET status = NULL WHERE"),
        "Null value must format as NULL: {}",
        lit_sqls[0]
    );
    assert!(
        lit_sqls[1].contains("SET status = 'O''Reilly''s ''Special''' WHERE"),
        "Single quotes must be doubled: {}",
        lit_sqls[1]
    );
    assert!(
        lit_sqls[2]
            .contains("SET status = '{\"flag\": true, \"note\": \"it''s done\"}'::jsonb WHERE"),
        "JSON quotes must be doubled and cast to jsonb: {}",
        lit_sqls[2]
    );

    // 5. DEFAULT VALUES Oracle:
    let mut default_state = TermrockGridPendingState::new(base_rows.clone());
    // Insert with only Default values
    default_state.insert_row(6, vec![(0, CellValue::Default), (1, CellValue::Default)]);
    let default_sqls = preview_sql(orders_table, &cols, &default_state);
    assert_eq!(default_sqls.len(), 1);
    assert_eq!(default_sqls[0], "INSERT INTO public.orders DEFAULT VALUES;");

    // 6. No-Primary-Key Fallback Oracle:
    let no_pk_table = make_table(
        "public",
        "events_log",
        vec![
            make_col("event_name", ColType::Text, false, false),
            make_col("payload", ColType::Text, false, true),
        ],
    );
    let no_pk_cols: Vec<(String, ColType)> = no_pk_table
        .columns
        .iter()
        .map(|c| (c.name.clone(), c.ty))
        .collect();
    let no_pk_rows = vec![
        vec![CellValue::Text("click".into()), CellValue::Null],
        vec![
            CellValue::Text("submit".into()),
            CellValue::Text("ok".into()),
        ],
    ];
    let mut no_pk_state = TermrockGridPendingState::new(no_pk_rows);
    no_pk_state.record_edit(0, 0, CellValue::Text("double_click".into()));
    let no_pk_sqls = preview_sql(&no_pk_table, &no_pk_cols, &no_pk_state);
    assert_eq!(no_pk_sqls.len(), 1);
    assert_eq!(
        no_pk_sqls[0],
        "UPDATE public.events_log SET event_name = 'double_click' WHERE event_name = 'click' AND payload IS NULL;"
    );

    // 7. No-Op Revert Oracle:
    let mut revert_state = TermrockGridPendingState::new(base_rows.clone());
    revert_state.record_edit(0, status_idx, CellValue::Text("shipped".into()));
    assert_eq!(preview_sql(orders_table, &cols, &revert_state).len(), 1);
    // Revert back to original value
    let orig = base_rows[0][status_idx].clone();
    revert_state.record_edit(0, status_idx, orig);
    assert_eq!(
        preview_sql(orders_table, &cols, &revert_state).len(),
        0,
        "Reverting to original value must produce 0 SQL statements"
    );

    // 8. Composite Primary Key Oracle:
    let composite_table = make_table(
        "public",
        "order_items",
        vec![
            make_col("order_id", ColType::Text, true, false),
            make_col("item_id", ColType::Int, true, false),
            make_col("qty", ColType::Int, false, false),
        ],
    );
    let comp_cols: Vec<(String, ColType)> = composite_table
        .columns
        .iter()
        .map(|c| (c.name.clone(), c.ty))
        .collect();
    let comp_rows = vec![vec![
        CellValue::Text("ord-100".into()),
        CellValue::Int(1),
        CellValue::Int(5),
    ]];
    let mut comp_state = TermrockGridPendingState::new(comp_rows);
    comp_state.record_edit(0, 2, CellValue::Int(10));
    comp_state.mark_deleted(0);
    let comp_sqls = preview_sql(&composite_table, &comp_cols, &comp_state);
    assert_eq!(comp_sqls.len(), 2);
    assert_eq!(
        comp_sqls[0],
        "UPDATE public.order_items SET qty = 10 WHERE order_id = 'ord-100' AND item_id = 1;"
    );
    assert_eq!(
        comp_sqls[1],
        "DELETE FROM public.order_items WHERE order_id = 'ord-100' AND item_id = 1;"
    );
}
