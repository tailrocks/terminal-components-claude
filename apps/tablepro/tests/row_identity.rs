//! Row identity, original values and undo must remain one owned record.
use junie_tui::{ColumnKey, GridEditor, GridModel, ItemKey, SortDir};
use tablepro_app::{ColType, GridView, ResultGrid, ResultSet, Tab, TableProApp, Value};

fn result() -> ResultSet {
    ResultSet {
        columns: vec![
            ("id".to_owned(), ColType::Int),
            ("label".to_owned(), ColType::Text),
        ],
        rows: vec![
            vec![Value::Int(10), Value::Text("ten".to_owned())],
            vec![Value::Int(20), Value::Text("twenty".to_owned())],
            vec![Value::Int(30), Value::Text("thirty".to_owned())],
        ],
        total: 3,
        source: None,
        duration_ms: 0,
        editable: true,
    }
}

fn assert_original_identity(grid: &ResultGrid) {
    for row in 0..grid.row_count() {
        let expected = match grid.row_key(row) {
            key if key == ItemKey::num(1) => 10,
            key if key == ItemKey::num(2) => 20,
            key if key == ItemKey::num(3) => 30,
            key => unreachable!("unexpected original key {key:?}"),
        };
        assert_eq!(
            grid.pending().value(row, 0),
            Some(&Value::Int(expected)),
            "record detached from stable key"
        );
    }
}

#[test]
fn undo_after_sort_preserves_record_keys_and_current_sort_policy() {
    let mut grid = ResultGrid::from_result(&result());
    assert!(grid.commit_cell(1, 1, "edited").is_ok());
    grid.sort(ColumnKey::num(1), SortDir::Desc);
    assert!(grid.undo());
    assert_original_identity(&grid);
    assert_eq!(grid.pending().value(0, 0), Some(&Value::Int(30)));
    assert_eq!(grid.pending().value(2, 0), Some(&Value::Int(10)));
    assert_eq!(grid.pending_total(), 0);
}

#[test]
fn sorting_an_insert_cannot_shift_existing_original_values_or_discard_keys() {
    let mut grid = ResultGrid::from_result(&result());
    let Some(inserted) = grid.insert_row() else {
        unreachable!("insert")
    };
    assert!(grid.commit_cell(inserted, 0, "5").is_ok());
    grid.sort(ColumnKey::num(1), SortDir::Asc);
    assert_eq!(
        grid.pending_total(),
        1,
        "clean rows gained false edits through original-row misalignment"
    );
    grid.discard();
    assert_eq!(grid.row_count(), 3);
    assert_original_identity(&grid);
    assert_eq!(grid.pending().value(0, 0), Some(&Value::Int(10)));
    assert_eq!(grid.pending_total(), 0);
}

#[test]
fn deleting_an_insert_removes_its_record_and_undo_restores_the_same_key() {
    let mut grid = ResultGrid::from_result(&result());
    let Some(inserted) = grid.insert_row() else {
        unreachable!("insert")
    };
    let key = grid.row_key(inserted);
    assert!(grid.commit_cell(inserted, 0, "40").is_ok());
    assert!(grid.delete_row(inserted));
    assert_eq!(grid.row_count(), 3);
    assert_eq!(grid.pending_total(), 0);
    assert!(grid.undo());
    assert_eq!(grid.row_count(), 4);
    assert_eq!(grid.row_key(3), key);
    assert_eq!(grid.pending().value(3, 0), Some(&Value::Int(40)));
}

#[test]
fn active_result_debug_never_exposes_cell_contents() {
    let secret = "active-result-secret-f83acb";
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let Some(Tab::Query(query)) = app.workbench.active_mut() else {
        unreachable!("query")
    };
    let mut data = result();
    data.rows = vec![vec![Value::Int(1), Value::Text(secret.to_owned())]];
    let mut view = GridView::from_result(&data);
    assert!(
        view.model
            .commit_cell(0, 1, "replacement-secret-f83acb")
            .is_ok()
    );
    assert!(!format!("{:?}", view.model.pending()).contains(secret));
    query.result = Some(view);
    assert!(!format!("{app:?}").contains(secret));
    assert!(!format!("{:?}", app.workbench).contains(secret));
    assert!(!format!("{:?}", app.result()).contains(secret));
    assert!(!format!("{app:?}").contains("replacement-secret-f83acb"));
    assert_eq!(
        app.result().cell(0, 1).map(|cell| cell.text),
        Some("replacement-secret-f83acb")
    );
}

#[test]
fn undo_insert_does_not_reuse_its_key_and_discard_is_undoable() {
    let mut grid = ResultGrid::from_result(&result());
    let Some(row) = grid.insert_row() else {
        unreachable!("insert")
    };
    let retired = grid.row_key(row);
    assert!(grid.undo());
    let Some(row) = grid.insert_row() else {
        unreachable!("insert")
    };
    let fresh = grid.row_key(row);
    assert_ne!(fresh, retired);
    assert!(grid.commit_cell(row, 0, "5").is_ok());
    assert!(grid.delete_row(1));
    grid.sort(ColumnKey::num(1), SortDir::Desc);
    grid.discard();
    assert_original_identity(&grid);
    assert!(grid.undo());
    assert_eq!(grid.row_key(3), fresh);
    assert_eq!(grid.pending().value(3, 0), Some(&Value::Int(5)));
    assert!(grid.pending().is_deleted(1));
    assert_eq!(grid.pending_total(), 2);
}

#[test]
fn real_grid_header_sort_then_keyboard_undo_preserves_identity_and_order() {
    use junie_tui::{KeyCode, Part, PartRef, Theme};
    use junie_tui_testing::Harness;
    use tablepro_app::Surface;
    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let Some(table) = app.workbench.active_table_mut() else {
        unreachable!("table")
    };
    table.result = GridView::from_result(&result());
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    let _ = h.type_str(" edited");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().result().pending_total(), 1);
    let header = PartRef::item(Part::HEADER, ItemKey::num(1));
    let _ = h.click_part(id, header);
    let _ = h.click_part(id, header);
    assert_eq!(
        h.app().result().pending().value(0, 0),
        Some(&Value::Int(30))
    );
    assert!(h.tab_to(id));
    let _ = h.key(KeyCode::Char('u'));
    assert_original_identity(h.app().result());
    assert_eq!(
        h.app().result().pending().value(0, 0),
        Some(&Value::Int(30))
    );
    assert_eq!(h.app().result().pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn runtime_undo_restores_sorted_insert_delete_and_discard_records() {
    use junie_tui::{KeyCode, Theme};
    use junie_tui_testing::Harness;
    use tablepro_app::Surface;
    for operation in ["insert", "delete", "discard"] {
        let mut app = TableProApp::default();
        app.set_surface(Surface::TableGrid);
        let Some(table) = app.workbench.active_table_mut() else {
            unreachable!("table")
        };
        table.result = GridView::from_result(&result());
        let model = &mut table.result.model;
        model.sort(ColumnKey::num(1), SortDir::Desc);
        match operation {
            "insert" => {
                assert!(model.insert_row().is_some());
            }
            "delete" => {
                assert!(model.delete_row(1));
            }
            _ => {
                assert!(model.commit_cell(1, 1, "changed").is_ok());
                model.discard();
            }
        }
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        let Some(id) = h.app().result_id() else {
            unreachable!("grid")
        };
        assert!(h.tab_to(id));
        let _ = h.key(KeyCode::Char('u'));
        assert_original_identity(h.app().result());
        assert_eq!(h.app().result().row_count(), 3);
        assert_eq!(
            h.app().result().pending().value(0, 0),
            Some(&Value::Int(30))
        );
        assert_eq!(
            h.app().result().pending_total(),
            usize::from(operation == "discard")
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
