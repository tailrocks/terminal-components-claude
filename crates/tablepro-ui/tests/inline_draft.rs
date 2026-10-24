#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{GridView, Surface, TableProApp, Value};
use termrock::{App, ColumnKey, Dialog, GridModel, Id, ItemKey, KeyCode, Part, Position, Theme};
use termrock_test_support::Harness;

fn draft(surface: Surface, value: &str) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    app.set_surface(surface);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(tab_navigation::tab_to(&mut h, id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str(value);
    h
}

fn view(h: &Harness<TableProApp>) -> &GridView {
    let Some((_, view)) = h.app().workbench.active_grid() else {
        unreachable!("grid")
    };
    view
}

fn assert_grid_editor_caret(
    h: &Harness<TableProApp>,
    grid_id: Id,
    cell: (ItemKey, ColumnKey),
) -> Position {
    let Some((active_id, grid)) = h.app().workbench.active_grid() else {
        unreachable!("the TablePro grid stays active while editing")
    };
    assert_eq!(active_id, grid_id);
    assert_eq!(grid.state.edit_cell(), Some(cell));

    let editor_id = grid_id.part(Part::TEXT);
    assert_eq!(h.focus(), Some(editor_id), "the Grid owns editor focus");
    let Some(grid_area) = h.area_of(grid_id) else {
        unreachable!("the active Grid is visible")
    };
    let Some(editor_area) = h.area_of(editor_id) else {
        unreachable!("the active Grid cell registers its embedded TextInput")
    };
    assert!(!editor_area.is_empty(), "the editor has visible cell area");
    assert_eq!(
        editor_area.intersection(grid_area),
        editor_area,
        "the cell editor remains inside the actual Grid bounds"
    );

    let Some(cursor) = h.cursor() else {
        unreachable!("the visible Grid editor publishes a cursor")
    };
    assert!(
        editor_area.contains(cursor),
        "Grid editor cursor {cursor:?} must stay inside {editor_area:?}"
    );
    assert!(
        h.cursor_bar(),
        "the Grid editor publishes its steady bar caret"
    );
    cursor
}

#[test]
fn inline_grid_unicode_draft_keeps_its_visible_bar_caret_without_mutating_the_result() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(grid_id) = h.app().result_id() else {
        unreachable!("the customer result Grid exists")
    };
    assert!(tab_navigation::tab_to(&mut h, grid_id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }

    let row_key = GridModel::row_key(&view(&h).model, 0);
    let cell = (row_key, ColumnKey::num(7));
    assert_eq!(
        row_key,
        ItemKey::num(1),
        "the first customer has a stable key"
    );
    assert_eq!(
        view(&h).columns.get(6).map(|(name, _)| name.as_str()),
        Some("currency"),
        "the edited key is the currency column"
    );
    assert_eq!(view(&h).state.cursor(), Some(cell));
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("USD".to_owned()))
    );

    let query_before = h.app().query().to_owned();
    let pending_before = h.app().result().pending().clone();
    assert_eq!(h.app().result().pending_total(), 0);
    let _ = h.key(KeyCode::F(2));
    assert_eq!(view(&h).state.edit_cell(), Some(cell));
    let _ = assert_grid_editor_caret(&h, grid_id, cell);

    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str("界");
    let _ = h.paste("🙂");
    assert_eq!(view(&h).state.edit_draft(), Some("界🙂"));
    assert_eq!(h.app().query(), query_before);
    assert_eq!(h.app().result().pending(), &pending_before);
    assert_eq!(h.app().result().pending_total(), 0);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("USD".to_owned())),
        "typing in Grid's draft does not change the result model"
    );
    let cursor_before_repeat_draw = assert_grid_editor_caret(&h, grid_id, cell);

    h.draw();
    let cursor_after_repeat_draw = assert_grid_editor_caret(&h, grid_id, cell);
    assert_eq!(cursor_after_repeat_draw, cursor_before_repeat_draw);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn changed_inline_draft_requires_quit_confirmation_without_committing_on_cancel() {
    let mut h = draft(Surface::TableGrid, "EUR");
    assert_eq!(h.app().result().pending_total(), 0);
    let cell = view(&h).state.edit_cell();
    let _ = h.ctrl('c');
    assert!(!h.app().should_quit());
    assert!(h.find("1 pending row change will be lost.").is_some());
    for _ in 0..3 {
        h.draw();
    }
    assert_eq!(view(&h).state.edit_cell(), cell);
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(view(&h).pending_total(), 1);
    assert_eq!(h.app().result().pending_total(), 0);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("USD".to_owned()))
    );
    let _ = h.key(KeyCode::Esc);
    assert!(!h.app().should_quit());
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(h.app().result().pending_total(), 0);
    let _ = h.ctrl('q');
    let _ = h.click_id(Dialog::new(Id::root("tablepro.quit-dialog")).action_id(1));
    assert!(h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn draft_survives_tab_and_structure_blur_then_explicit_enter_commits() {
    let mut h = draft(Surface::TableGrid, "EUR");
    let Some(key) = h.app().workbench.active_key() else {
        unreachable!("table")
    };
    let _ = h.ctrl('t');
    assert!(tab_navigation::tab_to(
        &mut h,
        Id::root("tablepro.workbench.tab-strip")
    ));
    let _ = h.key(KeyCode::Left);
    assert_eq!(h.app().workbench.active_key(), Some(key));
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    let _ = h.ctrl('d');
    let _ = h.ctrl('d');
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(h.app().result().pending_total(), 0);
    let editor = key.control("data").part(Part::TEXT);
    assert!(tab_navigation::tab_to(&mut h, editor));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(view(&h).state.edit_draft(), None);
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn existing_row_update_is_not_double_counted_and_escape_discards_only_inline_draft() {
    let mut h = draft(Surface::PendingChangeBar, "GBP");
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(view(&h).pending_total(), 1);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(view(&h).state.edit_draft(), None);
    assert_eq!(view(&h).pending_total(), 1);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn unchanged_inline_draft_does_not_invent_pending_work() {
    let mut h = draft(Surface::TableGrid, "USD");
    assert_eq!(view(&h).pending_total(), 0);
    let _ = h.ctrl('c');
    assert!(h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), 0);
}

#[test]
fn inserted_row_and_deleted_row_accounting_keep_distinct_operations() {
    let mut deleted = draft(Surface::TableGrid, "EUR");
    let Some(table) = deleted.app_mut().workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.delete_row(0));
    assert_eq!(deleted.app().result().pending_total(), 1);
    assert_eq!(
        view(&deleted).pending_total(),
        2,
        "delete plus uncommitted update are distinct operations"
    );

    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let Some(table) = app.workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.insert_row().is_some());
    let mut inserted = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = inserted.app().result_id() else {
        unreachable!("grid")
    };
    assert!(tab_navigation::tab_to(&mut inserted, id));
    let _ = inserted.key_mod(KeyCode::End, termrock::KeyModifiers::CONTROL);
    for _ in 0..6 {
        let _ = inserted.key(KeyCode::Right);
    }
    let _ = inserted.key(KeyCode::F(2));
    let _ = inserted.type_str("EUR");
    assert!(view(&inserted).state.edit_draft().is_some());
    assert_eq!(inserted.app().result().pending_total(), 1);
    assert_eq!(
        view(&inserted).pending_total(),
        1,
        "inline value belongs to the already-counted inserted row"
    );
    assert!(
        inserted.diagnostics().is_empty(),
        "{:?}",
        inserted.diagnostics()
    );
}
