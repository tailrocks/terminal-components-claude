//! Actual focused-grid row operations preserve keyed ownership and typed values.
use junie_tui::{ColumnKey, GlyphRole, GridModel, ItemKey, KeyCode, Part, PartRef, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Tab, TableProApp, Value};

fn query_harness(query: &str) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query(query);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    h
}
#[test]
fn plus_inserts_typed_defaults_selects_new_key_and_minus_toggles_rows() {
    let mut h = query_harness("SELECT id, currency FROM orders LIMIT 1");
    let original = h.app().result().row_key(0);
    let _ = h.key(KeyCode::Char('+'));
    assert_eq!(h.app().result().row_count(), 2);
    let inserted = h.app().result().row_key(1);
    assert_ne!(inserted, original);
    assert_eq!(
        h.app().result().pending().value(1, 0),
        Some(&Value::Default)
    );
    assert_eq!(h.app().result().pending().value(1, 1), Some(&Value::Null));
    assert_eq!(
        h.app().result().row_decor(1).marker,
        Some(GlyphRole::Inserted)
    );
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    assert_eq!(view.state.cursor(), Some((inserted, ColumnKey::num(2))));
    assert!(!view.state.is_editing());
    let _ = h.key(KeyCode::Char('-'));
    assert_eq!(h.app().result().row_count(), 1);
    assert_eq!(h.app().result().pending_total(), 0);
    let _ = h.key(KeyCode::Char('-'));
    assert!(h.app().result().pending().is_deleted(0));
    assert!(h.app().result().row_decor(0).strike);
    assert_eq!(
        h.app().result().row_decor(0).marker,
        Some(GlyphRole::Deleted)
    );
    let Some((x, y)) = h.find("USD") else {
        unreachable!("deleted result cell")
    };
    assert!(
        h.cell(x, y)
            .modifier
            .contains(junie_tui::Modifier::CROSSED_OUT)
    );
    let _ = h.key(KeyCode::Char('-'));
    assert!(!h.app().result().pending().is_deleted(0));
    assert_eq!(h.app().result().row_key(0), original);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn uppercase_u_requires_confirmation_and_preserves_sql_and_other_tab() {
    let mut h = query_harness("SELECT id, currency FROM orders LIMIT 1");
    let Some(key) = h.app().workbench.active_key() else {
        unreachable!("key")
    };
    let sql = h.app().query().to_owned();
    let _ = h.key(KeyCode::Char('+'));
    let _ = h.key(KeyCode::Char('U'));
    assert!(h.find("Discard unsaved changes?").is_some());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.key(KeyCode::Char('U'));
    let Some(other) = h.app_mut().workbench.new_query("neighbor draft") else {
        unreachable!("other")
    };
    h.draw();
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), Some(other));
    let Some(Tab::Query(tab)) = h.app().workbench.tab(key) else {
        unreachable!("query")
    };
    assert_eq!(tab.query, sql);
    assert!(tab.dirty());
    let Some(result) = tab.result.as_ref() else {
        unreachable!("result")
    };
    assert_eq!(result.pending_total(), 0);
    assert_eq!(result.model.row_count(), 1);
    assert!(
        matches!(h.app().workbench.tab(other), Some(Tab::Query(tab)) if tab.query == "neighbor draft")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn sorted_insert_delete_and_undo_keep_original_row_identity() {
    let mut h = query_harness("SELECT id, currency FROM orders LIMIT 4");
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    let original: Vec<_> = (0..4)
        .map(|row| {
            (
                h.app().result().row_key(row),
                h.app().result().pending().value(row, 0).cloned(),
            )
        })
        .collect();
    let _ = h.click_part(id, PartRef::item(Part::HEADER, ItemKey::num(1)));
    assert!(h.tab_to(id));
    let _ = h.key(KeyCode::Char('+'));
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    let Some((inserted, _)) = view.state.cursor() else {
        unreachable!("cursor")
    };
    let _ = h.key(KeyCode::Char('-'));
    let _ = h.key(KeyCode::Char('u'));
    assert_eq!(h.app().result().row_count(), 5);
    assert!((0..5).any(|row| h.app().result().row_key(row) == inserted));
    for (key, value) in original {
        let Some(row) = (0..5).find(|row| h.app().result().row_key(*row) == key) else {
            unreachable!("original")
        };
        assert_eq!(h.app().result().pending().value(row, 0).cloned(), value);
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn row_keys_type_normally_during_inline_edit_and_readonly_results_refuse_actions() {
    let mut h = query_harness("SELECT id, currency FROM orders LIMIT 1");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    let _ = h.type_str("+-U");
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    assert_eq!(view.state.edit_draft(), Some("USD+-U"));
    assert_eq!(view.model.row_count(), 1);
    assert!(h.find("Discard unsaved changes?").is_none());
    let mut readonly = query_harness("SELECT currency FROM orders LIMIT 1");
    for key in ['+', '-', 'U'] {
        let _ = readonly.key(KeyCode::Char(key));
    }
    assert_eq!(readonly.app().result().row_count(), 1);
    assert_eq!(readonly.app().result().pending_total(), 0);
    assert!(readonly.find("Discard unsaved changes?").is_none());
    assert!(h.diagnostics().is_empty());
    assert!(readonly.diagnostics().is_empty());
}

#[test]
fn empty_result_insert_and_stale_discard_preserve_generation_bound_work() {
    let mut h = query_harness("SELECT id, currency FROM orders WHERE id = 'missing' LIMIT 1");
    assert_eq!(h.app().result().row_count(), 0);
    let _ = h.key(KeyCode::Char('+'));
    assert_eq!(h.app().result().row_count(), 1);
    let _ = h.key(KeyCode::Char('U'));
    assert!(h.find("Discard unsaved changes?").is_some());
    let Some((_, view)) = h.app_mut().workbench.active_grid_mut() else {
        unreachable!("grid")
    };
    assert!(junie_tui::GridEditor::commit_cell(&mut view.model, 0, 1, "EUR").is_ok());
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(
        h.app().result().pending().value(0, 1),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert!(h.find("Work changed").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inserting_after_five_hundred_rows_reveals_the_new_owned_record() {
    let mut h = query_harness("SELECT * FROM orders");
    assert_eq!(h.app().result().row_count(), 500);
    let _ = h.key(KeyCode::Char('+'));
    assert_eq!(h.app().result().row_count(), 501);
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    assert_eq!(
        view.state.cursor(),
        Some((view.model.row_key(500), ColumnKey::num(3)))
    );
    assert!(h.find("DEFAULT").is_some());
    assert!(h.find("NULL").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
