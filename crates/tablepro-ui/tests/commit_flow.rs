#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_domain::model::HistorySource;
use tablepro_ui::{QueryOutcome, ResultGrid, SafeMode, Surface, TableProApp, Value};
use termrock::{Dialog, GridEditor, GridModel, Id, ItemKey, KeyCode, Theme};
use termrock_test_support::Harness;

const COMMIT_DIALOG: Id = Id::root("tablepro.quit-dialog");
const COMMIT_PREVIEW: Id = Id::root("tablepro.commit-preview");

fn staged_app(mode: SafeMode) -> (TableProApp, ItemKey, ItemKey, ItemKey, String) {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT id, currency FROM orders LIMIT 4"),
        QueryOutcome::Executed { rows: 4, .. }
    ));
    app.set_safe_mode(mode);
    app.surface = Surface::QueryEditing;

    let Some((_, grid)) = app.workbench.active_grid_mut() else {
        unreachable!("editable result grid")
    };
    let existing = grid.model.row_key(0);
    let deleted = grid.model.row_key(3);
    assert!(grid.model.commit_cell(0, 1, "EUR").is_ok());
    let Some(inserted_row) = grid.model.insert_row() else {
        unreachable!("inserted row")
    };
    assert!(
        grid.model
            .commit_cell(inserted_row, 0, "new-order-id")
            .is_ok()
    );
    assert!(grid.model.commit_cell(inserted_row, 1, "CAD").is_ok());
    let inserted = grid.model.row_key(inserted_row);
    assert!(grid.model.delete_row(3));
    let Some(source) = grid.model.source() else {
        unreachable!("table source")
    };
    let target = source.to_owned();
    (app, existing, deleted, inserted, target)
}

fn save(h: &mut Harness<TableProApp>) {
    focus_grid(h);
    let _ = h.ctrl('s');
}

fn focus_grid(h: &mut Harness<TableProApp>) {
    let Some(result) = h.app().result_id() else {
        unreachable!("result grid")
    };
    assert!(tab_navigation::tab_to(h, result));
}

fn confirm(h: &mut Harness<TableProApp>) {
    let _ = h.click_id(Dialog::new(COMMIT_DIALOG).action_id(1));
}

fn row_for_key(grid: &ResultGrid, key: ItemKey) -> Option<usize> {
    (0..grid.row_count()).find(|row| grid.row_key(*row) == key)
}

fn text_cell(grid: &ResultGrid, row: usize, column: usize) -> String {
    let Some(Value::Text(value)) = grid.pending().value(row, column) else {
        unreachable!("expected text at ({row}, {column})")
    };
    value.clone()
}

#[test]
fn save_review_cancel_keeps_edits_insert_and_delete_staged() {
    let (app, existing, deleted, inserted, _) = staged_app(SafeMode::Silent);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let before = h.app().result().pending().clone();
    let history_before = h.app().workbench.history.entries.len();

    save(&mut h);
    assert!(h.find("Review pending changes").is_some());
    assert!(h.find("SQL: 3 statement(s)").is_some());
    assert_eq!(h.app().result().pending_total(), 3);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);

    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending(), &before);
    assert_eq!(h.app().result().row_key(0), existing);
    assert!(row_for_key(h.app().result(), deleted).is_some());
    assert!(row_for_key(h.app().result(), inserted).is_some());
    assert_eq!(h.app().result().pending_total(), 3);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn confirmed_save_applies_edit_insert_delete_and_clears_precommit_undo() {
    let (app, existing, deleted, inserted, _) = staged_app(SafeMode::Silent);
    let row_id = text_cell(app.result(), 0, 0);
    let deleted_id = text_cell(app.result(), 3, 0);
    let expected_sql = [
        format!("UPDATE public.orders SET currency = 'EUR' WHERE id = '{row_id}';"),
        "INSERT INTO public.orders (id, currency) VALUES ('new-order-id', 'CAD');".to_owned(),
        format!("DELETE FROM public.orders WHERE id = '{deleted_id}';"),
    ];
    let history_before = app.workbench.history.entries.len();
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    focus_grid(&mut h);
    let _ = h.key(KeyCode::Char(' '));
    let cursor_before = h
        .app()
        .workbench
        .active_grid()
        .and_then(|(_, grid)| grid.state.cursor());
    let selected_before = h
        .app()
        .workbench
        .active_grid()
        .map(|(_, grid)| grid.state.selected_rows().clone());
    assert!(
        selected_before
            .as_ref()
            .is_some_and(|rows| !rows.is_empty())
    );
    let _ = h.ctrl('s');
    assert!(h.find("Review pending changes").is_some());

    confirm(&mut h);

    let grid = h.app().result();
    assert_eq!(grid.row_count(), 4);
    assert_eq!(grid.pending_total(), 0);
    assert!(row_for_key(grid, deleted).is_none());
    let Some(existing_row) = row_for_key(grid, existing) else {
        unreachable!("surviving row key")
    };
    assert_eq!(
        grid.pending().value(existing_row, 1),
        Some(&Value::Text("EUR".to_owned()))
    );
    let Some(inserted_row) = row_for_key(grid, inserted) else {
        unreachable!("committed insert key")
    };
    assert_eq!(
        grid.pending().value(inserted_row, 0),
        Some(&Value::Text("new-order-id".to_owned()))
    );
    assert_eq!(
        grid.pending().value(inserted_row, 1),
        Some(&Value::Text("CAD".to_owned()))
    );
    for row in 0..grid.row_count() {
        assert!(!grid.pending().is_dirty_row(row));
        assert!(!grid.pending().is_inserted(row));
        assert!(!grid.pending().is_deleted(row));
    }
    let Some((_, active)) = h.app().workbench.active_grid() else {
        unreachable!("active grid")
    };
    assert_eq!(active.state.cursor(), cursor_before);
    assert_eq!(Some(active.state.selected_rows().clone()), selected_before);
    assert_eq!(
        h.app().status(),
        "Committed 3 row change(s) to public.orders"
    );
    assert_eq!(
        h.app().workbench.history.entries.len(),
        history_before + expected_sql.len()
    );
    for (entry, expected) in h
        .app()
        .workbench
        .history
        .entries
        .iter()
        .take(expected_sql.len())
        .zip(expected_sql.iter().rev())
    {
        assert_eq!(&entry.sql, expected);
        assert_eq!(entry.source, HistorySource::RowEdits);
        assert_eq!(entry.connection, "Local PostgreSQL");
        assert_eq!(entry.database, "acme_dev");
        assert_eq!(entry.schema, "public");
        assert_eq!(entry.duration_ms, Some(1));
        assert_eq!(entry.rows, Some(1));
        assert_eq!(entry.error, None);
    }
    let Some((_, active)) = h.app_mut().workbench.active_grid_mut() else {
        unreachable!("active grid")
    };
    assert!(!active.model.undo(), "commit must clear older undo history");
    assert_eq!(active.model.pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn safe_mode_requires_exact_target_and_escape_preserves_pending_work() {
    let (app, _, _, _, _) = staged_app(SafeMode::Safe);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let before = h.app().result().pending().clone();
    let history_before = h.app().workbench.history.entries.len();
    save(&mut h);
    assert!(h.find("Review pending changes").is_some());

    let _ = h.type_str("wrong.target");
    confirm(&mut h);
    assert_eq!(h.app().result().pending(), &before);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);
    assert!(h.find("Review pending changes").is_some());

    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending(), &before);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn safe_mode_exact_target_acknowledgement_commits() {
    let (app, _, _, _, target) = staged_app(SafeMode::Safe);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    save(&mut h);
    let _ = h.type_str(&target);
    confirm(&mut h);
    assert_eq!(h.app().result().pending_total(), 0);
    assert!(h.app().status().starts_with("Committed 3 row change(s)"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn read_only_and_clean_grids_refuse_save_without_opening_review() {
    let (app, _, _, _, _) = staged_app(SafeMode::ReadOnly);
    let mut readonly = Harness::new(app, Theme::junie(), 120, 40);
    let readonly_history = readonly.app().workbench.history.entries.len();
    save(&mut readonly);
    assert_eq!(readonly.app().result().pending_total(), 3);
    assert_eq!(
        readonly.app().workbench.history.entries.len(),
        readonly_history
    );
    assert_eq!(
        readonly.app().status(),
        "Read-only mode refuses row changes"
    );
    assert!(readonly.find("Review pending changes").is_none());

    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT id, currency FROM orders LIMIT 1");
    app.surface = Surface::QueryEditing;
    let mut clean = Harness::new(app, Theme::junie(), 120, 40);
    let clean_history = clean.app().workbench.history.entries.len();
    save(&mut clean);
    assert_eq!(clean.app().status(), "No pending changes");
    assert_eq!(clean.app().workbench.history.entries.len(), clean_history);
    assert!(clean.find("Review pending changes").is_none());
    assert!(clean.diagnostics().is_empty(), "{:?}", clean.diagnostics());
}

#[test]
fn changed_safe_mode_authorization_invalidates_typed_review() {
    let (app, _, _, _, target) = staged_app(SafeMode::Safe);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let history_before = h.app().workbench.history.entries.len();
    save(&mut h);
    let _ = h.type_str(&target);
    h.app_mut().set_safe_mode(SafeMode::Silent);
    h.app_mut().surface = Surface::QueryEditing;
    h.draw();
    confirm(&mut h);

    assert_eq!(h.app().result().pending_total(), 3);
    assert_eq!(h.app().safe_mode(), SafeMode::Silent);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);
    assert!(h.find("Work changed; request again").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn changed_snapshot_and_active_tab_cannot_reuse_a_commit_review() {
    let (app, _, _, _, _) = staged_app(SafeMode::Silent);
    let mut snapshot = Harness::new(app, Theme::junie(), 120, 40);
    let snapshot_history = snapshot.app().workbench.history.entries.len();
    save(&mut snapshot);
    let Some((_, grid)) = snapshot.app_mut().workbench.active_grid_mut() else {
        unreachable!("active result")
    };
    assert!(grid.model.commit_cell(0, 1, "GBP").is_ok());
    snapshot.draw();
    confirm(&mut snapshot);
    assert_eq!(snapshot.app().result().pending_total(), 3);
    assert_eq!(
        snapshot.app().result().pending().value(0, 1),
        Some(&Value::Text("GBP".to_owned()))
    );
    assert!(snapshot.find("Work changed; request again").is_some());
    assert_eq!(
        snapshot.app().workbench.history.entries.len(),
        snapshot_history
    );

    let (app, _, _, _, _) = staged_app(SafeMode::Silent);
    let mut switched = Harness::new(app, Theme::junie(), 120, 40);
    let Some(original_tab) = switched.app().workbench.active_key() else {
        unreachable!("active tab")
    };
    save(&mut switched);
    let Some(new_tab) = switched.app_mut().workbench.new_query("neighbor draft") else {
        unreachable!("new tab")
    };
    switched.draw();
    confirm(&mut switched);
    assert_eq!(switched.app().workbench.active_key(), Some(new_tab));
    let Some(tablepro_ui::Tab::Query(tab)) = switched.app().workbench.tab(original_tab) else {
        unreachable!("original tab")
    };
    let Some(result) = tab.result.as_ref() else {
        unreachable!("original result")
    };
    assert_eq!(result.pending_total(), 3);
    assert!(switched.find("Work changed; request again").is_some());
    assert!(
        switched.diagnostics().is_empty(),
        "{:?}",
        switched.diagnostics()
    );
}

#[test]
fn projected_preview_uses_subset_and_result_column_order() {
    let (app, _, _, _, _) = staged_app(SafeMode::Silent);
    let mut subset = Harness::new(app, Theme::junie(), 120, 40);
    let subset_history = subset.app().workbench.history.entries.len();
    save(&mut subset);
    assert!(
        subset
            .find("UPDATE public.orders SET currency = 'EUR'")
            .is_some()
    );
    assert!(
        subset
            .find("INSERT INTO public.orders (id, currency)")
            .is_some()
    );
    assert!(
        subset
            .find("DELETE FROM public.orders WHERE id =")
            .is_some()
    );
    assert!(subset.find("order_number = 'EUR'").is_none());
    assert!(subset.find("SET id = 'EUR'").is_none());
    let _ = subset.key(KeyCode::Esc);
    assert_eq!(subset.app().workbench.history.entries.len(), subset_history);
    assert_eq!(subset.app().result().pending_total(), 3);

    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT currency, id FROM orders LIMIT 4"),
        QueryOutcome::Executed { rows: 4, .. }
    ));
    app.set_safe_mode(SafeMode::Silent);
    app.surface = Surface::QueryEditing;
    let (first_id, deleted_id) = {
        let Some((_, grid)) = app.workbench.active_grid_mut() else {
            unreachable!("editable projected result")
        };
        let first_id = text_cell(&grid.model, 0, 1);
        let deleted_id = text_cell(&grid.model, 3, 1);
        assert!(grid.model.commit_cell(0, 0, "EUR").is_ok());
        let Some(inserted_row) = grid.model.insert_row() else {
            unreachable!("projected insert row")
        };
        assert!(grid.model.commit_cell(inserted_row, 0, "CAD").is_ok());
        assert!(
            grid.model
                .commit_cell(inserted_row, 1, "new-order-id")
                .is_ok()
        );
        assert!(grid.model.delete_row(3));
        (first_id, deleted_id)
    };
    let expected_sql = [
        format!("UPDATE public.orders SET currency = 'EUR' WHERE id = '{first_id}';"),
        "INSERT INTO public.orders (currency, id) VALUES ('CAD', 'new-order-id');".to_owned(),
        format!("DELETE FROM public.orders WHERE id = '{deleted_id}';"),
    ];
    let history_before = app.workbench.history.entries.len();
    let mut reordered = Harness::new(app, Theme::junie(), 120, 40);
    save(&mut reordered);
    assert!(
        reordered
            .find("UPDATE public.orders SET currency = 'EUR'")
            .is_some()
    );
    assert!(
        reordered
            .find("INSERT INTO public.orders (currency, id)")
            .is_some()
    );
    assert!(
        reordered
            .find("DELETE FROM public.orders WHERE id =")
            .is_some()
    );
    assert!(reordered.find("SET id = 'EUR'").is_none());
    assert!(reordered.find("order_number").is_none());

    confirm(&mut reordered);
    let history = &reordered.app().workbench.history.entries;
    assert_eq!(history.len(), history_before + expected_sql.len());
    for (entry, expected) in history
        .iter()
        .take(expected_sql.len())
        .zip(expected_sql.iter().rev())
    {
        assert_eq!(&entry.sql, expected);
        assert_eq!(entry.source, HistorySource::RowEdits);
    }
    assert!(
        reordered.diagnostics().is_empty(),
        "{:?}",
        reordered.diagnostics()
    );
}

#[test]
fn review_binds_connection_environment_database_and_target() {
    let (app, _, _, _, _) = staged_app(SafeMode::Silent);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let history_before = h.app().workbench.history.entries.len();
    save(&mut h);
    assert!(h.find("Connection: Local PostgreSQL · local").is_some());
    assert!(
        h.find("Database: acme_dev · Target: public.orders")
            .is_some()
    );

    let connection = &mut h.app_mut().workbench.connection;
    connection.name = "Changed connection".to_owned();
    connection.environment = tablepro_demo::db::Environment::Production;
    connection.database = "changed_database".to_owned();
    h.draw();
    confirm(&mut h);

    assert_eq!(h.app().result().pending_total(), 3);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);
    assert!(h.find("Work changed; request again").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn changed_projection_cannot_reuse_commit_review() {
    let (app, _, _, _, _) = staged_app(SafeMode::Silent);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let pending_before = h.app().result().pending().clone();
    let history_before = h.app().workbench.history.entries.len();

    save(&mut h);
    assert!(
        h.find("UPDATE public.orders SET currency = 'EUR'")
            .is_some()
    );

    let Some((_, grid)) = h.app_mut().workbench.active_grid_mut() else {
        unreachable!("editable result grid")
    };
    grid.columns.swap(0, 1);
    h.draw();
    confirm(&mut h);

    assert_eq!(h.app().result().pending(), &pending_before);
    assert_eq!(h.app().result().pending_total(), 3);
    assert_eq!(h.app().workbench.history.entries.len(), history_before);
    assert!(h.find("Work changed; request again").is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn every_preview_statement_scrolls_at_narrow_size_and_after_resize() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT id, currency FROM orders LIMIT 8"),
        QueryOutcome::Executed { rows: 8, .. }
    ));
    app.surface = Surface::QueryEditing;
    let Some((_, grid)) = app.workbench.active_grid_mut() else {
        unreachable!("editable query result")
    };
    for row in 0..8 {
        assert!(
            grid.model
                .commit_cell(row, 1, &format!("review_currency_{row}"))
                .is_ok()
        );
    }

    let mut h = Harness::new(app, Theme::junie(), 100, 32);
    save(&mut h);
    assert!(h.find("review_currency_0").is_some());
    let _ = h.resize(72, 20);
    assert!(h.find("Connection: Local PostgreSQL · local").is_some());
    assert!(
        h.find("Database: acme_dev · Target: public.orders")
            .is_some()
    );
    assert!(h.find("8 update").is_some());
    assert!(h.find("Silent mode").is_some());
    assert!(h.find("SQL: 8 statement(s)").is_some());
    assert!(h.find("Cancel").is_some());
    assert!(h.find("Commit").is_some());
    assert!(h.find("review_currency_7").is_none());
    assert!(h.area_of(COMMIT_PREVIEW).is_some());
    let _ = h.click_id(COMMIT_PREVIEW);
    assert_eq!(h.focus(), Some(COMMIT_PREVIEW));
    for _ in 0..12 {
        let _ = h.key(KeyCode::PageDown);
    }
    assert!(h.find("review_currency_7").is_some());
    assert!(h.find("review_currency_0").is_none());

    let _ = h.resize(100, 32);
    assert!(h.find("SQL: 8 statement(s)").is_some());
    assert!(h.find("review_currency_7").is_some());
    let _ = h.key(KeyCode::Home);
    assert!(h.find("review_currency_0").is_some());
    assert_eq!(h.app().result().pending_total(), 8);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
