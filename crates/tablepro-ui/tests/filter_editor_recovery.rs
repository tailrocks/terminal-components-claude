#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use tablepro_ui::{Filter, FilterOp, SafeMode, Surface, Tab, TableProApp, Value};
use termrock::{Dialog, GridEditor, GridModel, KeyCode, Theme};
use termrock_test_support::Harness;

#[path = "common/tab_navigation.rs"]
mod tab_navigation;

fn orders_harness() -> Harness<TableProApp> {
    orders_harness_at(120, 40)
}

fn orders_harness_at(width: u16, height: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4));
    let mut harness = Harness::new(app, Theme::junie(), width, height);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    harness
}

#[test]
fn inline_eq_filter_still_applies_through_the_value_input() {
    let mut harness = orders_harness();
    let _ = harness.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Char('f'));
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.apply"))
    );
    let _ = harness.key(KeyCode::BackTab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.step"))
    );
    let _ = harness.key(KeyCode::BackTab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.value"))
    );
    let _ = harness.type_str("pending");
    assert!(harness.app().filter_value_focused());
    let _ = harness.key(KeyCode::Enter);

    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table")
    };
    assert_eq!(
        table.filters.len(),
        1,
        "surface={:?}; status={}; text={}",
        harness.app().surface(),
        harness.app().status(),
        harness.text()
    );
    assert_eq!(table.filters[0].column, "status");
    assert_eq!(table.filters[0].op, FilterOp::Eq);
    assert_eq!(table.filters[0].value, "pending");
    assert_eq!(table.result.pending_total(), 0);
}

#[test]
fn modal_selects_edit_both_between_values_and_keep_unsupported_apply_atomic() {
    let mut harness = orders_harness();
    let _ = harness.key(KeyCode::Home);
    let Some(Tab::Table(before)) = harness.app().workbench.active() else {
        panic!("orders is active before opening the editor")
    };
    let before_filters = before.filters.clone();
    let before_rows = before.result.model.row_count();
    let before_total = before.result.model.total();
    let before_cursor = before.result.state.cursor();
    let before_selection = format!("{:?}", before.result.state.selected_rows());
    let before_cell = before
        .result
        .model
        .cell(0, 0)
        .map(|cell| cell.text.to_owned());
    let _ = harness.ctrl('f');
    assert!(harness.text().contains("Add filter"), "{}", harness.text());
    assert!(harness.is_open(termrock::Id::root("tablepro.filter.layer")));
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.value"))
    );
    let ring = harness
        .ring()
        .entries()
        .iter()
        .map(|entry| (entry.id, entry.scope, entry.layer))
        .collect::<Vec<_>>();
    let trap = harness.ring().active_trap();
    assert!(
        ring.iter()
            .any(|(id, _, _)| *id == termrock::Id::root("tablepro.filter.operator")),
        "operator Select is not reachable: {ring:?}; trap={:?}",
        harness.ring().active_trap()
    );

    // The component ring is Column, Operator, Value, Value2, Cancel, Apply.
    let _ = harness.key(KeyCode::BackTab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.operator")),
        "ring={ring:?}; initial_trap={trap:?}; current_trap={:?}; surface={:?}; open={}",
        harness.ring().active_trap(),
        harness.app().surface(),
        harness.is_open(termrock::Id::root("tablepro.filter.layer"))
    );
    let _ = harness.key(KeyCode::Enter);
    for _ in 0..16 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.text().contains("BETWEEN"), "{}", harness.text());
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.type_str("4");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.type_str("19");
    let _ = harness.key(KeyCode::Enter);

    let text = harness.text();
    assert!(text.contains("not supported by the query engine"), "{text}");
    assert!(text.contains("BETWEEN '4' AND '19'"), "{text}");
    assert_eq!(harness.app().surface(), Surface::FilterEditor);
    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table")
    };
    assert_eq!(table.filters, before_filters);
    assert_eq!(table.result.pending_total(), 0);
    assert_eq!(table.result.model.row_count(), before_rows);
    assert_eq!(table.result.model.total(), before_total);
    assert_eq!(table.result.state.cursor(), before_cursor);
    assert_eq!(
        format!("{:?}", table.result.state.selected_rows()),
        before_selection
    );
    assert_eq!(
        table
            .result
            .model
            .cell(0, 0)
            .map(|cell| cell.text.to_owned()),
        before_cell
    );
}

#[test]
fn modal_column_select_and_cancel_button_preserve_the_table() {
    let mut harness = orders_harness();
    let _ = harness.key(KeyCode::Home);
    let _ = harness.ctrl('f');
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.text().contains("customer_id"), "{}", harness.text());

    // Tab from the column Select lands on the operator, then the value field.
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.type_str("7");
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Enter);
    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table")
    };
    assert_eq!(table.filters.len(), 1);
    assert_eq!(table.filters[0].column, "customer_id");
    assert_eq!(table.filters[0].op, FilterOp::Eq);
    assert_eq!(table.filters[0].value, "7");

    let before_rows = table.result.model.row_count();
    let before_surface = harness.app().surface();
    let _ = harness.ctrl('f');
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().surface(), before_surface);
    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table")
    };
    assert_eq!(table.filters[0].column, "customer_id");
    assert_eq!(table.result.model.row_count(), before_rows);

    let filters_before_escape = table.filters.clone();
    let rows_before_escape = table.result.model.row_count();
    let _ = harness.ctrl('f');
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(termrock::Id::root("tablepro.filter.layer")));
    assert_eq!(harness.app().surface(), before_surface);
    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table")
    };
    assert_eq!(table.filters, filters_before_escape);
    assert_eq!(table.result.model.row_count(), rows_before_escape);
}

#[test]
fn disabled_between_can_be_edited_and_applied_without_filtering_rows() {
    let mut harness = orders_harness();
    let Some(Tab::Table(table)) = harness.app_mut().workbench.active_mut() else {
        panic!("orders is active before seeding the disabled filter")
    };
    table.set_filter(Filter {
        column: "id".to_owned(),
        op: FilterOp::Between,
        value: String::new(),
        value2: String::new(),
        enabled: false,
    });
    let Some(Tab::Table(before)) = harness.app().workbench.active() else {
        panic!("orders is active before opening the editor")
    };
    let before_rows = before.result.model.row_count();
    let before_total = before.result.model.total();

    let _ = harness.key(KeyCode::Home);
    let _ = harness.ctrl('f');
    assert!(harness.text().contains("Edit filter"), "{}", harness.text());
    assert!(harness.text().contains("BETWEEN"), "{}", harness.text());
    let _ = harness.type_str("4");
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.value2"))
    );
    let _ = harness.type_str("19");
    let _ = harness.key(KeyCode::Enter);

    assert!(!harness.is_open(termrock::Id::root("tablepro.filter.layer")));
    let Some(Tab::Table(after)) = harness.app().workbench.active() else {
        panic!("orders remains active after applying the disabled filter")
    };
    assert_eq!(
        after.filters,
        [Filter {
            column: "id".to_owned(),
            op: FilterOp::Between,
            value: "4".to_owned(),
            value2: "19".to_owned(),
            enabled: false,
        }]
    );
    assert_eq!(after.result.model.row_count(), before_rows);
    assert_eq!(after.result.model.total(), before_total);
    assert_eq!(after.result.pending_total(), 0);
}

#[test]
fn operator_popup_reorders_for_selected_column_and_keeps_stable_operator_key() {
    let mut harness = orders_harness();
    let _ = harness.key(KeyCode::Home);
    let _ = harness.ctrl('f');
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::BackTab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.column"))
    );
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.text().contains("order_number"),
        "{}",
        harness.text()
    );

    let _ = harness.key(KeyCode::Tab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.operator"))
    );
    let _ = harness.key(KeyCode::Enter);
    let menu = harness.text();
    let between = menu
        .find("between")
        .expect("numeric operator menu contains Between");
    let contains = menu
        .find("contains")
        .expect("numeric operator menu keeps text matching available");
    assert!(between < contains, "numeric menu order is wrong:\n{menu}");
    for _ in 0..6 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.text().contains("BETWEEN"), "{}", harness.text());
    assert!(
        harness.text().contains("order_number"),
        "{}",
        harness.text()
    );
}

#[test]
fn applying_a_filter_preserves_unique_selection_and_cursor_through_save_review() {
    let mut harness = orders_harness();
    harness.app_mut().set_safe_mode(SafeMode::Silent);
    let result_id = harness
        .app()
        .result_id()
        .expect("the orders data grid has a focus target");
    assert!(tab_navigation::tab_to(&mut harness, result_id));
    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::Right); // order_number is the active filter column.
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Char(' '));
    for _ in 0..2 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Char(' '));
    for _ in 0..2 {
        let _ = harness.key(KeyCode::Down);
    }

    let (selected_keys, selected_order_numbers, cursor_key, cursor_column, before_currencies) = {
        let Some(Tab::Table(table)) = harness.app().workbench.active() else {
            panic!("orders is the active table before filtering")
        };
        assert!(table.result.model.row_count() > 8);
        let selected = table.result.state.selected_rows().clone();
        assert_eq!(selected.keys().len(), 2);
        let selected_orders = (0..table.result.model.row_count())
            .filter(|row| selected.contains(GridModel::row_key(&table.result.model, *row)))
            .filter_map(|row| match table.result.model.pending().value(row, 1) {
                Some(Value::Int(order)) => Some(*order),
                _ => None,
            })
            .collect::<Vec<_>>();
        let cursor = table.result.state.cursor().expect("the grid has a cursor");
        assert_eq!(
            usize::from(cursor.1.raw()).checked_sub(1),
            Some(1),
            "the cursor starts in the order_number column"
        );
        let currencies = (0..table.result.model.row_count())
            .filter_map(|row| {
                let id = match table.result.model.pending().value(row, 0)? {
                    Value::Text(id) => id.clone(),
                    _ => return None,
                };
                let currency = match table.result.model.pending().value(row, 6)? {
                    Value::Text(currency) => currency.clone(),
                    _ => return None,
                };
                Some((id, currency))
            })
            .collect::<std::collections::HashMap<_, _>>();
        (selected, selected_orders, cursor.0, cursor.1, currencies)
    };
    assert_eq!(selected_order_numbers, [10_005, 10_007]);

    let _ = harness.ctrl('f');
    assert!(
        harness.text().contains("order_number"),
        "{}",
        harness.text()
    );
    let _ = harness.key(KeyCode::BackTab);
    assert_eq!(
        harness.focus(),
        Some(termrock::Id::root("tablepro.filter.operator"))
    );
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.type_str("10004");
    let _ = harness.key(KeyCode::Enter);

    let (surviving_order, surviving_id) = {
        let Some(Tab::Table(table)) = harness.app().workbench.active() else {
            panic!("orders remains the active table after filter Apply")
        };
        assert!(table.result.model.row_count() > 8);
        assert_eq!(table.result.state.selected_rows(), &selected_keys);
        let selected_after = (0..table.result.model.row_count())
            .filter(|row| {
                table
                    .result
                    .state
                    .selected_rows()
                    .contains(GridModel::row_key(&table.result.model, *row))
            })
            .filter_map(|row| match table.result.model.pending().value(row, 1) {
                Some(Value::Int(order)) => Some(*order),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(selected_after, selected_order_numbers);
        let cursor = table
            .result
            .state
            .cursor()
            .expect("cursor survives the filter");
        assert_eq!(cursor.0, cursor_key);
        assert_eq!(cursor.1, cursor_column);
        let row = (0..table.result.model.row_count())
            .find(|row| GridModel::row_key(&table.result.model, *row) == cursor_key)
            .expect("cursor key resolves in the refreshed result");
        let order = match table.result.model.pending().value(row, 1) {
            Some(Value::Int(order)) => *order,
            _ => panic!("cursor row has an integer order number"),
        };
        let id = match table.result.model.pending().value(row, 0) {
            Some(Value::Text(id)) => id.clone(),
            _ => panic!("cursor row has a text primary key"),
        };
        (order, id)
    };
    assert_eq!(surviving_order, 10_009);

    let Some((_, grid)) = harness.app_mut().workbench.active_grid_mut() else {
        panic!("the filtered table has an active editable grid")
    };
    let row = (0..grid.model.row_count())
        .find(|row| GridModel::row_key(&grid.model, *row) == cursor_key)
        .expect("the same row key remains after filtering");
    assert!(grid.model.commit_cell(row, 6, "FILTERED-SAVE").is_ok());

    assert!(tab_navigation::tab_to(&mut harness, result_id));
    let _ = harness.ctrl('s');
    assert!(harness.find("Review pending changes").is_some());
    let _ = harness.click_id(Dialog::new(termrock::Id::root("tablepro.quit-dialog")).action_id(1));

    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains the active table after Ctrl+S review")
    };
    assert_eq!(table.result.pending_total(), 0);
    let mut changed_ids = Vec::new();
    for row in 0..table.result.model.row_count() {
        let id = match table.result.model.pending().value(row, 0) {
            Some(Value::Text(id)) => id,
            _ => panic!("every filtered order retains its primary key"),
        };
        let currency = match table.result.model.pending().value(row, 6) {
            Some(Value::Text(currency)) => currency,
            _ => panic!("every filtered order retains its currency"),
        };
        if id == &surviving_id {
            assert_eq!(currency, "FILTERED-SAVE");
            changed_ids.push(id.clone());
        } else if let Some(before_currency) = before_currencies.get(id) {
            assert_eq!(
                currency, before_currency,
                "saving the cursor row must leave every other record untouched"
            );
        }
    }
    assert_eq!(changed_ids, [surviving_id]);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn applying_a_filter_drops_removed_selection_and_moves_a_removed_cursor_forward() {
    let mut harness = orders_harness();
    let result_id = harness
        .app()
        .result_id()
        .expect("the orders data grid has a focus target");
    assert!(tab_navigation::tab_to(&mut harness, result_id));
    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::Right);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Char(' '));
    for _ in 0..2 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Char(' '));
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Up);
    }

    let old_cursor_key = {
        let Some(Tab::Table(table)) = harness.app().workbench.active() else {
            panic!("orders is active before filtering")
        };
        let selected = table.result.state.selected_rows();
        let selected_orders = (0..table.result.model.row_count())
            .filter(|row| selected.contains(GridModel::row_key(&table.result.model, *row)))
            .filter_map(|row| match table.result.model.pending().value(row, 1) {
                Some(Value::Int(order)) => Some(*order),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(selected_orders, [10_004, 10_006]);
        let (key, column) = table.result.state.cursor().expect("the grid has a cursor");
        assert_eq!(
            usize::from(column.raw()).checked_sub(1),
            Some(1),
            "the cursor starts in the order_number column"
        );
        assert_eq!(
            table.result.model.pending().value(2, 1),
            Some(&Value::Int(10_002))
        );
        key
    };

    let _ = harness.ctrl('f');
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.type_str("10004");
    let _ = harness.key(KeyCode::Enter);

    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains active after filter Apply")
    };
    let selected_orders = (0..table.result.model.row_count())
        .filter(|row| {
            table
                .result
                .state
                .selected_rows()
                .contains(GridModel::row_key(&table.result.model, *row))
        })
        .filter_map(|row| match table.result.model.pending().value(row, 1) {
            Some(Value::Int(order)) => Some(*order),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(selected_orders, [10_006]);
    let (cursor_key, cursor_column) = table
        .result
        .state
        .cursor()
        .expect("the removed cursor moves to a nearby result row");
    assert_ne!(cursor_key, old_cursor_key);
    assert_eq!(
        usize::from(cursor_column.raw()).checked_sub(1),
        Some(1),
        "cursor column remains attached to order_number"
    );
    let cursor_row = (0..table.result.model.row_count())
        .find(|row| GridModel::row_key(&table.result.model, *row) == cursor_key)
        .expect("the fallback cursor key resolves");
    assert_eq!(
        table.result.model.pending().value(cursor_row, 1),
        Some(&Value::Int(10_007)),
        "the old cursor index chooses the nearest forward filtered row"
    );
}

#[test]
fn narrow_filter_apply_keeps_a_nonfirst_cursor_column_visible() {
    let mut harness = orders_harness_at(72, 24);
    let result_id = harness
        .app()
        .result_id()
        .expect("the orders data grid has a focus target");
    assert!(tab_navigation::tab_to(&mut harness, result_id));
    let _ = harness.key(KeyCode::Home);
    for _ in 0..2 {
        let _ = harness.key(KeyCode::Down);
    }
    for _ in 0..6 {
        let _ = harness.key(KeyCode::Right);
    }
    let (row_key, column_key, currency) = {
        let Some(Tab::Table(table)) = harness.app().workbench.active() else {
            panic!("orders is active before narrow filtering")
        };
        let (row_key, column_key) = table.result.state.cursor().expect("grid cursor exists");
        assert_eq!(
            usize::from(column_key.raw()).checked_sub(1),
            Some(6),
            "the cursor is in the nonfirst currency column"
        );
        let row = (0..table.result.model.row_count())
            .find(|row| GridModel::row_key(&table.result.model, *row) == row_key)
            .expect("cursor row resolves before filtering");
        let currency = match table.result.model.pending().value(row, 6) {
            Some(Value::Text(currency)) => currency.clone(),
            _ => panic!("currency is text in the active row"),
        };
        (row_key, column_key, currency)
    };
    let before_header = harness
        .text()
        .lines()
        .find(|line| line.contains("‹") && line.contains("currency"))
        .expect("the active currency column is visible before filtering")
        .to_owned();
    assert!(before_header.contains("‹"), "{before_header}");

    let _ = harness.ctrl('f');
    assert!(harness.text().contains("currency"), "{}", harness.text());
    let _ = harness.type_str(&currency);
    let _ = harness.key(KeyCode::Enter);

    let Some(Tab::Table(table)) = harness.app().workbench.active() else {
        panic!("orders remains active after narrow filter Apply")
    };
    assert!(table.result.model.row_count() > 0);
    assert_eq!(
        table.result.state.cursor(),
        Some((row_key, column_key)),
        "filter Apply keeps the row and nonfirst column that define the user's horizontal context"
    );
    let text = harness.text();
    assert!(
        text.contains("‹") && text.contains("currency ∇"),
        "the narrow viewport keeps the cursor's horizontal window after filtering: {text}"
    );
    let after_header = text
        .lines()
        .find(|line| line.contains("‹") && line.contains("currency"))
        .expect("the cursor column remains visible after filtering");
    assert!(
        after_header.contains("‹"),
        "the filtered result remains horizontally positioned around the surviving cursor: {after_header}"
    );
}
