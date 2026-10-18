//! Deterministic SQL generation for pending result edits.

use tablepro_domain::{ColType, PendingEdits, Table, Value};

/// Quote a database value for deterministic SQL generation.
pub fn sql_literal(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_owned(),
        Value::Default => "DEFAULT".to_owned(),
        Value::Text(text) => format!("'{}'", text.replace('\'', "''")),
        Value::Int(value) => value.to_string(),
        Value::Num(value) => format!("{value:.2}"),
        Value::Bool(value) => value.to_string(),
        Value::Json(value) => format!("'{}'::jsonb", value.replace('\'', "''")),
    }
}

/// Build statements `TablePro` sends for pending result edits.
pub fn preview_sql(
    table: &Table,
    columns: &[(String, ColType)],
    pending: &PendingEdits,
) -> Vec<String> {
    let pk = table.primary_key();
    let where_clause = |row: usize| {
        let mut terms = Vec::new();
        if pk.is_empty() {
            for (col, _) in columns.iter().enumerate() {
                let Some((name, _)) = columns.get(col) else {
                    continue;
                };
                let value = pending.original_value(row, col).unwrap_or(&Value::Null);
                terms.push(match value {
                    Value::Null => format!("{name} IS NULL"),
                    value => format!("{name} = {}", sql_literal(value)),
                });
            }
        } else {
            for key in &pk {
                let Some(col) = columns.iter().position(|(name, _)| name == &key.name) else {
                    continue;
                };
                let value = pending.original_value(row, col).unwrap_or(&Value::Null);
                terms.push(format!("{} = {}", key.name, sql_literal(value)));
            }
        }
        if terms.is_empty() {
            "1 = 0".to_owned()
        } else {
            terms.join(" AND ")
        }
    };

    let dirty_rows = pending.dirty_rows();
    let mut out = Vec::new();

    // Updates, then inserts, then deletes
    for &row in &dirty_rows {
        if pending.is_inserted(row) || pending.is_deleted(row) {
            continue;
        }
        let mut sets = Vec::new();
        for col in 0..pending.current_row_width(row) {
            if pending.is_dirty(row, col)
                && let Some((name, _)) = columns.get(col)
                && let Some(value) = pending.value(row, col)
            {
                sets.push(format!("{name} = {}", sql_literal(value)));
            }
        }
        if !sets.is_empty() {
            out.push(format!(
                "UPDATE {} SET {} WHERE {};",
                table.qualified(),
                sets.join(", "),
                where_clause(row)
            ));
        }
    }

    for &row in &dirty_rows {
        if pending.is_inserted(row) && !pending.is_deleted(row) {
            let mut names = Vec::new();
            let mut values = Vec::new();
            for (col, (name, _)) in columns.iter().enumerate() {
                let value = pending.value(row, col).unwrap_or(&Value::Null);
                if !matches!(value, Value::Default) {
                    names.push(name.clone());
                    values.push(sql_literal(value));
                }
            }
            if names.is_empty() {
                out.push(format!("INSERT INTO {} DEFAULT VALUES;", table.qualified()));
            } else {
                out.push(format!(
                    "INSERT INTO {} ({}) VALUES ({});",
                    table.qualified(),
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }
    }

    for &row in &dirty_rows {
        if pending.is_deleted(row) && !pending.is_inserted(row) {
            out.push(format!(
                "DELETE FROM {} WHERE {};",
                table.qualified(),
                where_clause(row)
            ));
        }
    }
    out
}
