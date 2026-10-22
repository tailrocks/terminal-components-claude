#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use tablepro_ui::{FilterOp, Surface, Tab, TableProApp};
use termrock::{GridModel, KeyCode, Theme};
use termrock_test_support::Harness;

fn orders_harness() -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(4));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
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
