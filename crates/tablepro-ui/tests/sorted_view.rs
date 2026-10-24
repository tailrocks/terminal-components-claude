#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::TableProApp;
use tablepro_ui::{Value, db, sql};
use termrock::{KeyCode, StateFlags, Theme};
use termrock_test_support::Harness;

fn nulls_last_page(
    table: &tablepro_ui::Table,
    column: usize,
    ascending: bool,
    keep: usize,
) -> Vec<Vec<Value>> {
    let source = db::rows(table, 0, table.row_count.min(2_000));
    let mut non_null = Vec::new();
    let mut null_rows = Vec::new();
    for row in source {
        if matches!(row.get(column), Some(Value::Null)) {
            null_rows.push(row);
        } else {
            non_null.push(row);
        }
    }
    non_null.sort_by(|left, right| {
        let null = Value::Null;
        let order = sql::cmp_values(
            left.get(column).unwrap_or(&null),
            right.get(column).unwrap_or(&null),
        );
        if ascending { order } else { order.reverse() }
    });
    non_null.extend(null_rows);
    non_null.truncate(keep);
    non_null
}

fn active_grid_rows(harness: &Harness<TableProApp>) -> Result<Vec<Vec<Value>>, String> {
    let (_, grid) = harness
        .app()
        .workbench
        .active_grid()
        .ok_or_else(|| "active table did not expose its real result grid".to_owned())?;
    (0..grid.model.row_count())
        .map(|row| {
            (0..grid.columns.len())
                .map(|column| grid.model.pending().value(row, column).cloned())
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| format!("visible result row {row} was incomplete"))
        })
        .collect()
}

fn assert_rows_match(
    actual: &[Vec<Value>],
    expected: &[Vec<Value>],
    sort_column: usize,
    direction: &str,
) {
    assert_eq!(actual.len(), expected.len(), "{direction} row count");
    for (index, (actual_row, expected_row)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual_row,
            expected_row,
            "{direction} row {index} differs: actual key/value {:?}/{:?}, expected {:?}/{:?}",
            actual_row.first(),
            actual_row.get(sort_column),
            expected_row.first(),
            expected_row.get(sort_column),
        );
    }
}

#[test]
fn keyboard_sort_scrolls_to_created_at_ascending() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.text().contains("public › orders"),
        "{}",
        harness.text()
    );
    for _ in 0..12 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Char('s'));
    let text = harness.text();
    assert!(
        text.contains("sort created_at ▴"),
        "missing sort status: {text}"
    );
    assert!(
        text.contains("Sorted by created_at ascending"),
        "missing footer: {text}"
    );
    assert!(text.contains("‹10"), "missing left overflow: {text}");
    assert!(
        text.contains("cols 11–13 of 14"),
        "missing column window: {text}"
    );
    assert!(
        text.contains("2024-01-01 02:13:21+00"),
        "rows are not ascending: {text}"
    );
    assert!(
        text.contains("2025-06-25 12:30:14"),
        "first updated_at drifted: {text}"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the complete two-direction keyboard flow shares one real Harness state"
)]
fn clean_payment_grid_keeps_nulls_last_for_both_keyboard_sort_directions() -> Result<(), String> {
    let mut app = TableProApp::default();
    if !app.connect(0) {
        return Err("the default connection did not open".to_owned());
    }
    if !app.workbench.open_table("payments") {
        return Err("the real public.payments table did not open".to_owned());
    }
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let (grid_id, table, sort_column, keep, total, source_rows, history_before, query_before) = {
        let table = harness
            .app()
            .workbench
            .active_table()
            .ok_or_else(|| "the active tab was not a table".to_owned())?;
        if table.table.name != "payments" || !table.filters.is_empty() {
            return Err("the test must begin on the clean, unfiltered payments table".to_owned());
        }
        if table.result.pending_total() != 0 || table.result.model.pending_total() != 0 {
            return Err("the test must begin with no pending edits".to_owned());
        }
        if table.result.model.source() != Some("public.payments") {
            return Err("the table result lost its source identity".to_owned());
        }
        let sort_column = table
            .table
            .columns
            .iter()
            .position(|column| column.name == "failure_reason")
            .ok_or_else(|| "payments.failure_reason was missing".to_owned())?;
        let (_, grid) = harness
            .app()
            .workbench
            .active_grid()
            .ok_or_else(|| "the table had no mounted result grid".to_owned())?;
        let source_rows = db::rows(&table.table, 0, table.table.row_count.min(2_000));
        if !source_rows
            .iter()
            .any(|row| matches!(row.get(sort_column), Some(Value::Null)))
            || !source_rows.iter().any(|row| {
                row.get(sort_column)
                    .is_some_and(|value| !matches!(value, Value::Null))
            })
        {
            return Err("the real scan must contain both NULL and non-NULL reasons".to_owned());
        }
        (
            harness
                .app()
                .workbench
                .active_grid()
                .map(|(id, _)| id)
                .ok_or_else(|| "the table had no grid identity".to_owned())?,
            table.table.clone(),
            sort_column,
            grid.model.row_count(),
            grid.model.total(),
            source_rows,
            harness.app().workbench.history.entries.clone(),
            harness.app().query().to_owned(),
        )
    };
    if !tab_navigation::tab_to(&mut harness, grid_id) {
        return Err("keyboard traversal did not focus the result grid".to_owned());
    }
    if !harness.state_of(grid_id).contains(StateFlags::FOCUSED) {
        return Err("the focused sort target was not the mounted grid".to_owned());
    }

    let target_column = u16::try_from(sort_column.saturating_add(1))
        .map_err(|_| "sort column does not fit the grid key".to_owned())?;
    let current_column = harness
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor().map(|(_, column)| column.raw()))
        .ok_or_else(|| "focused grid did not expose a cursor column".to_owned())?;
    if current_column > target_column {
        let _ = harness.key(KeyCode::Home);
    }
    let current_column = harness
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor().map(|(_, column)| column.raw()))
        .ok_or_else(|| "focused grid lost its cursor column".to_owned())?;
    if current_column > target_column {
        return Err("Home did not return the grid cursor to a reachable column".to_owned());
    }
    for _ in current_column..target_column {
        let _ = harness.key(KeyCode::Right);
    }
    let selected_column = harness
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor().map(|(_, column)| column.raw()))
        .ok_or_else(|| "grid sort column was not selected".to_owned())?;
    assert_eq!(selected_column, target_column);

    let expected_ascending = nulls_last_page(&table, sort_column, true, keep);
    assert_eq!(expected_ascending.len(), keep);
    assert!(source_rows.iter().any(|row| {
        row.get(sort_column)
            .is_some_and(|value| !matches!(value, Value::Null))
    }));
    assert!(
        source_rows
            .iter()
            .any(|row| matches!(row.get(sort_column), Some(Value::Null)))
    );
    let _ = harness.key(KeyCode::Char('s'));
    let ascending_text = harness.text();
    assert!(
        ascending_text.contains("sort failure_reason ▴"),
        "missing ascending sort status: {ascending_text}"
    );
    assert!(
        ascending_text.contains("Sorted by failure_reason ascending"),
        "missing ascending footer: {ascending_text}"
    );
    assert_rows_match(
        &active_grid_rows(&harness)?,
        &expected_ascending,
        sort_column,
        "ascending",
    );

    let expected_descending = nulls_last_page(&table, sort_column, false, keep);
    let _ = harness.key(KeyCode::Char('s'));
    let text = harness.text();
    assert!(
        text.contains("sort failure_reason ▾"),
        "missing descending sort status: {text}"
    );
    assert!(
        text.contains("Sorted by failure_reason descending"),
        "missing descending footer: {text}"
    );
    let actual = active_grid_rows(&harness)?;
    assert_rows_match(&actual, &expected_descending, sort_column, "descending");
    let first_null = actual
        .iter()
        .position(|row| matches!(row.get(sort_column), Some(Value::Null)))
        .ok_or_else(|| "the sorted page did not retain visible NULL rows".to_owned())?;
    assert!(first_null > 0, "the sorted page must retain non-NULL rows");
    assert!(
        first_null < actual.len(),
        "the sorted page must retain NULL rows"
    );
    assert!(actual.get(first_null..).is_some_and(|tail| {
        tail.iter()
            .all(|row| matches!(row.get(sort_column), Some(Value::Null)))
    }));

    let active = harness
        .app()
        .workbench
        .active_table()
        .ok_or_else(|| "sorting changed the active table".to_owned())?;
    assert_eq!(active.filters.as_slice(), &[]);
    assert_eq!(active.result.pending_total(), 0);
    assert_eq!(active.result.model.pending_total(), 0);
    assert_eq!(active.result.model.total(), total);
    assert_eq!(active.result.model.source(), Some("public.payments"));
    assert_eq!(harness.app().query(), query_before);
    assert_eq!(harness.app().workbench.history.entries, history_before);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}
