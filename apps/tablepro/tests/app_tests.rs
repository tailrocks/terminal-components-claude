//! Deterministic `TablePro` application-boundary tests.

use tablepro_app::{
    TableProApp,
    db::{self, ColType, Value},
    domain::{PendingEdits, preview_sql},
    sql::{self, Decision, Statement},
};

fn orders_table() -> Option<db::Table> {
    db::Catalog::acme_prod()
        .find(Some("public"), "orders")
        .cloned()
}

#[test]
fn query_safety_gate_is_conservative() {
    let parse = |query: &str| match sql::parse(query) {
        Ok(statement) => Some(statement),
        Err(error) => {
            assert!(error.message.is_empty(), "query must parse: {}", error.message);
            None
        }
    };
    let Some(select) = parse("SELECT * FROM orders") else {
        return;
    };
    let Some(destructive) = parse("DELETE FROM orders") else {
        return;
    };
    let Some(write) = parse("UPDATE orders SET status = 'paid' WHERE id = 7") else {
        return;
    };

    assert_eq!(sql::gate(db::SafeMode::Silent, &select), Decision::Run);
    assert_eq!(
        sql::gate(db::SafeMode::Silent, &destructive),
        Decision::Confirm { deliberate: false }
    );
    assert_eq!(sql::gate(db::SafeMode::ReadOnly, &write), Decision::Deny);
}

#[test]
fn pending_edits_preview_preserves_original_keys_and_order() {
    let table = orders_table();
    assert!(table.is_some(), "demo catalog must contain public.orders");
    let Some(table) = table else {
        return;
    };
    let columns = vec![
        ("id".to_owned(), ColType::Int),
        ("name".to_owned(), ColType::Text),
    ];
    let mut pending = PendingEdits::new(vec![
        vec![Value::Int(7), Value::Text("Ada".to_owned())],
        vec![Value::Int(8), Value::Text("remove".to_owned())],
    ]);
    assert!(pending.set(0, 0, Value::Int(99)));
    assert!(pending.set(0, 1, Value::Text("O'Reilly".to_owned())));
    assert!(pending.delete_row(1));
    let inserted = pending.insert_row(columns.len());
    assert!(pending.set(inserted, 0, Value::Int(9)));
    assert!(pending.set(inserted, 1, Value::Text("new".to_owned())));

    assert_eq!(
        preview_sql(&table, &columns, &pending),
        vec![
            "UPDATE public.orders SET id = 99, name = 'O''Reilly' WHERE id = 7;".to_owned(),
            "INSERT INTO public.orders (id, name) VALUES (9, 'new');".to_owned(),
            "DELETE FROM public.orders WHERE id = 8;".to_owned(),
        ]
    );
}

#[test]
fn tablepro_app_starts_with_deterministic_result() {
    let app = TableProApp::default();

    assert_eq!(
        app.query(),
        "SELECT * FROM orders WHERE status = 'pending' ORDER BY total_amount DESC LIMIT 20"
    );
    assert_eq!(app.result().row_count(), 20);
    assert!(app.result().is_editable());
    assert!(app.status().contains("Loaded 20 rows"));
}

#[test]
fn projected_result_is_read_only_when_no_key_is_selected() {
    let catalog = db::Catalog::acme_prod();
    let statement = match sql::parse("SELECT status FROM orders LIMIT 3") {
        Ok(Statement::Select(statement)) => statement,
        Ok(_) => {
            assert!(matches!(sql::parse("SELECT status FROM orders LIMIT 3"), Ok(Statement::Select(_))));
            return;
        }
        Err(error) => {
            assert!(error.message.is_empty(), "query must parse: {}", error.message);
            return;
        }
    };
    let result = match sql::run_select(&catalog, &statement) {
        Ok(result) => result,
        Err(error) => {
            assert!(error.message.is_empty(), "query must execute: {}", error.message);
            return;
        }
    };

    assert!(!result.editable);
    assert_eq!(result.rows.len(), 3);
}
