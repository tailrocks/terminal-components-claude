#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use std::time::Duration;

use tablepro_ui::{QueryOutcome, SafeMode, Surface, Tab, TabKey, TableProApp};
use termrock::{Dialog, GridEditor, Id, KeyCode, MouseKind, Theme};
use termrock_test_support::Harness;

const QUERY_SAFETY: Id = Id::root("tablepro.query-safety");
const QUERY_PREVIEW: Id = QUERY_SAFETY.sub("sql-preview");
const REPLACE_RESULTS: Id = Id::root("tablepro.quit-dialog");
const HELP: Id = Id::root("tablepro.help");
const FIXED_DURATION: Duration = Duration::from_millis(42);
const SELECT_ORDERS: &str = "SELECT * FROM public.orders LIMIT 3";

fn query_harness(
    safe_mode: SafeMode,
    connection_index: usize,
    query: &str,
) -> Result<(Harness<TableProApp>, TabKey), &'static str> {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    if !app.connect(connection_index) {
        return Err("the fixture connection is available");
    }
    app.set_safe_mode(safe_mode);
    app.surface = Surface::QueryEditing;
    let Some(key) = app.workbench.active_key() else {
        return Err("connected workbench has a query tab");
    };
    let Some(Tab::Query(tab)) = app.workbench.tab_mut(key) else {
        return Err("connected active tab is a query");
    };
    tab.replace_query(query);

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    if !tab_navigation::tab_to(&mut harness, key.control("query")) {
        return Err("the active query editor receives focus");
    }
    Ok((harness, key))
}

fn result_rows(harness: &Harness<TableProApp>, key: TabKey) -> Option<usize> {
    let Some(Tab::Query(tab)) = harness.app().workbench.tab(key) else {
        return None;
    };
    tab.result.as_ref().map(|grid| grid.model.row_count())
}

fn query_status(harness: &Harness<TableProApp>, key: TabKey) -> Option<(bool, u32)> {
    let Some(Tab::Query(tab)) = harness.app().workbench.tab(key) else {
        return None;
    };
    let status = tab.last_status.as_deref()?;
    Some((status.is_error, status.duration_ms))
}

fn confirm_query(harness: &mut Harness<TableProApp>) {
    let _ = harness.click_id(Dialog::new(QUERY_SAFETY).action_id(1));
}

fn pending_result_harness() -> Result<(Harness<TableProApp>, TabKey), &'static str> {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    if !app.connect(0) {
        return Err("local demo connection is available");
    }
    app.set_safe_mode(SafeMode::Silent);
    let Some(key) = app.workbench.active_key() else {
        return Err("connected workbench has an active query");
    };
    let Some(Tab::Query(tab)) = app.workbench.tab_mut(key) else {
        return Err("active tab is a query");
    };
    tab.replace_query("SELECT id, currency FROM public.orders LIMIT 1");
    if !matches!(
        app.run_query("SELECT id, currency FROM public.orders LIMIT 1"),
        QueryOutcome::Executed { rows: 1, .. }
    ) {
        return Err("fixture SELECT produces one editable result row");
    }
    let Some((_, grid)) = app.workbench.active_grid_mut() else {
        return Err("query result grid is available");
    };
    if grid.model.commit_cell(0, 1, "CHF").is_err() {
        return Err("fixture cell can be staged without changing data");
    }
    app.set_safe_mode(SafeMode::SafeFull);
    app.surface = Surface::QueryEditing;
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    if !tab_navigation::tab_to(&mut harness, key.control("query")) {
        return Err("the active query editor receives focus");
    }
    Ok((harness, key))
}

#[test]
fn safe_full_select_requires_exact_target_and_executes_once_with_fixed_timing()
-> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
    let history_before = harness.app().workbench.history.entries.len();

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(harness.text().contains(SELECT_ORDERS));
    assert!(harness.text().contains("Production"));
    assert!(harness.text().contains("acme_prod"));
    assert!(harness.text().contains("public.orders"));
    assert_eq!(
        harness.focus(),
        Some(Dialog::new(QUERY_SAFETY).input_id()),
        "deliberate confirmation starts in the acknowledgement field"
    );
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), None);

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    let diagnostic_count = harness.diagnostics().len();
    harness.draw();
    assert_eq!(harness.diagnostics().len(), diagnostic_count);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn empty_and_case_mismatched_acknowledgements_cannot_execute() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
    let history_before = harness.app().workbench.history.entries.len();

    let _ = harness.key(KeyCode::F(5));
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert_eq!(result_rows(&harness, key), None);

    let _ = harness.type_str("Orders");
    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), None);
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(harness.focus(), Some(key.control("query")));
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn alert_full_select_requires_confirmation_but_no_typed_token() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::AlertFull, 0, SELECT_ORDERS)?;

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(
        harness
            .area_of(Dialog::new(QUERY_SAFETY).input_id())
            .is_none()
    );
    assert_eq!(result_rows(&harness, key), None);

    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn alert_write_uses_plain_confirmation_and_reports_the_unsupported_executor()
-> Result<(), &'static str> {
    let sql = "UPDATE public.orders SET status = 'paid' WHERE id = 'missing'";
    let (mut harness, key) = query_harness(SafeMode::Alert, 0, sql)?;

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(harness.text().contains(sql));
    assert!(harness.text().contains("unsupported demo executor"));
    assert!(
        harness
            .area_of(Dialog::new(QUERY_SAFETY).input_id())
            .is_none()
    );
    assert_eq!(result_rows(&harness, key), None);

    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert!(harness.app().status().contains("only runs SELECT"));
    assert_eq!(query_status(&harness, key), Some((true, 42)));
    assert_eq!(result_rows(&harness, key), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn safe_mode_write_requires_target_acknowledgement_before_unsupported_result()
-> Result<(), &'static str> {
    let sql = "UPDATE public.orders SET status = 'paid' WHERE id = 'missing'";
    let (mut harness, key) = query_harness(SafeMode::Safe, 0, sql)?;

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(
        harness
            .text()
            .contains("Acknowledgement: type exactly orders")
    );
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), None);

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert!(harness.app().status().contains("only runs SELECT"));
    assert_eq!(query_status(&harness, key), Some((true, 42)));
    assert_eq!(result_rows(&harness, key), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn public_headless_execution_keeps_confirmation_required_contract() -> Result<(), &'static str> {
    let mut app = TableProApp::new_with_query_duration(FIXED_DURATION);
    if !app.connect(4) {
        return Err("production demo connection is available");
    }
    app.set_safe_mode(SafeMode::SafeFull);
    assert!(matches!(
        app.run_query(SELECT_ORDERS),
        QueryOutcome::ConfirmationRequired {
            deliberate: true,
            ..
        }
    ));
    assert_eq!(app.result().row_count(), 0);
    Ok(())
}

#[test]
fn silent_mode_still_requires_target_ack_for_dangerous_production_write() -> Result<(), &'static str>
{
    let sql = "DELETE FROM public.orders";
    let (mut harness, key) = query_harness(SafeMode::Silent, 4, sql)?;

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(
        harness
            .text()
            .contains("Acknowledgement: type exactly orders")
    );
    assert_eq!(
        harness.focus(),
        Some(Dialog::new(QUERY_SAFETY).input_id()),
        "production destructive review starts in the target field"
    );
    assert_eq!(result_rows(&harness, key), None);

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert!(harness.app().status().contains("only runs SELECT"));
    assert_eq!(query_status(&harness, key), Some((true, 42)));
    assert_eq!(result_rows(&harness, key), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn read_only_write_is_denied_without_a_review_or_result_change() -> Result<(), &'static str> {
    let sql = "UPDATE orders SET status = 'paid' WHERE id = 'missing'";
    let (mut harness, key) = query_harness(SafeMode::ReadOnly, 0, sql)?;
    let history_before = harness.app().workbench.history.entries.len();
    let status_before = harness.app().status().to_owned();

    let _ = harness.ctrl('r');

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert!(harness.app().status().contains("denied in Read-Only mode"));
    assert_ne!(harness.app().status(), status_before);
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), None);
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn confirmed_unsupported_write_is_reported_as_an_error_not_success() -> Result<(), &'static str> {
    let sql = "UPDATE public.orders SET status = 'paid' WHERE id = 'missing'";
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, sql)?;
    let history_before = harness.app().workbench.history.entries.len();

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(harness.text().contains("unsupported"));
    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), Some((true, 42)));
    assert!(harness.app().status().contains("only runs SELECT"));
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

fn assert_qualified_unsupported_write_stays_unverified(sql: &str) -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, sql)?;
    let history_before = harness.app().workbench.history.entries.len();

    assert_eq!(harness.app().workbench.current_schema(), "public");
    let _ = harness.ctrl('r');

    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let review = harness.text();
    assert!(review.contains(sql));
    assert!(review.contains("schema context: public"));
    assert!(review.contains("Target: orders (unverified)"));
    assert!(!review.contains("Target: public.orders"));
    assert!(review.contains("Acknowledgement: type exactly orders"));

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert!(harness.app().status().contains("only runs SELECT"));
    assert_eq!(query_status(&harness, key), Some((true, 42)));
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn missing_qualified_schema_is_not_rebound_to_current_schema_target() -> Result<(), &'static str> {
    assert_qualified_unsupported_write_stays_unverified(
        "UPDATE missing_schema.orders SET status = 'paid' WHERE id = 'missing'",
    )
}

#[test]
fn same_named_target_in_other_schema_is_not_rebound_to_current_schema_table()
-> Result<(), &'static str> {
    // The fixture has public.orders. SQL naming analytics.orders must not be
    // presented as verified public.orders when the parser drops the qualifier.
    assert_qualified_unsupported_write_stays_unverified(
        "UPDATE analytics.orders SET status = 'paid' WHERE id = 'missing'",
    )
}

#[test]
fn query_safety_cancel_and_outside_dismissal_preserve_query_state() -> Result<(), &'static str> {
    for outside in [false, true] {
        let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
        let history_before = harness.app().workbench.history.entries.len();

        let _ = harness.ctrl('r');
        let Some(layer) = harness.layer_area(QUERY_SAFETY) else {
            return Err("run opens query safety review");
        };
        if outside {
            assert!(
                layer.x > 0 || layer.y > 0,
                "dialog is centered away from (0, 0)"
            );
            let _ = harness.mouse(MouseKind::Down, 0, 0);
            let _ = harness.mouse(MouseKind::Up, 0, 0);
        } else {
            let _ = harness.key(KeyCode::Esc);
        }

        assert!(
            harness.layer_area(QUERY_SAFETY).is_none(),
            "outside={outside}, status={}",
            harness.app().status()
        );
        assert_eq!(harness.focus(), Some(key.control("query")));
        assert_eq!(result_rows(&harness, key), None);
        assert_eq!(query_status(&harness, key), None);
        assert_eq!(
            harness.app().workbench.history.entries.len(),
            history_before
        );
        let diagnostic_count = harness.diagnostics().len();
        harness.draw();
        assert_eq!(harness.diagnostics().len(), diagnostic_count);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn changed_query_connection_schema_or_safe_mode_fails_closed() -> Result<(), &'static str> {
    #[derive(Clone, Copy)]
    enum Change {
        Query,
        Connection,
        Schema,
        SafeMode,
    }

    for change in [
        Change::Query,
        Change::Connection,
        Change::Schema,
        Change::SafeMode,
    ] {
        let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
        let _ = harness.ctrl('r');
        assert!(harness.layer_area(QUERY_SAFETY).is_some());

        match change {
            Change::Query => {
                let Some(Tab::Query(tab)) = harness.app_mut().workbench.tab_mut(key) else {
                    return Err("the captured query tab remains available for mutation");
                };
                tab.replace_query("SELECT * FROM public.customers LIMIT 1");
            }
            Change::Connection => {
                harness
                    .app_mut()
                    .workbench
                    .connection
                    .host
                    .push_str("-changed");
            }
            Change::Schema => {
                assert!(harness.app_mut().workbench.select_schema("audit"));
            }
            Change::SafeMode => harness.app_mut().set_safe_mode(SafeMode::Silent),
        }

        let _ = harness.type_str("orders");
        confirm_query(&mut harness);

        assert!(harness.layer_area(QUERY_SAFETY).is_none());
        assert_eq!(harness.app().status(), "Work changed; request again");
        assert_eq!(result_rows(&harness, key), None);
        assert_eq!(query_status(&harness, key), None);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn changed_result_projection_fails_closed_and_preserves_new_pending_edits()
-> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::Silent, 0, SELECT_ORDERS)?;
    assert!(matches!(
        harness.app_mut().run_query(SELECT_ORDERS),
        QueryOutcome::Executed { rows: 3, .. }
    ));
    harness.app_mut().set_safe_mode(SafeMode::SafeFull);
    harness.app_mut().surface = Surface::QueryEditing;
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());

    let Some((_, grid)) = harness.app_mut().workbench.active_grid_mut() else {
        return Err("the captured query result is still available");
    };
    if grid.model.commit_cell(0, 6, "EUR").is_err() {
        return Err("a projected currency cell can be staged while review is open");
    }
    grid.columns.swap(5, 6);
    let pending_after_change = harness.app().result().pending().clone();

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(harness.app().status(), "Work changed; request again");
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(harness.app().result().pending(), &pending_after_change);
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn stable_tab_identity_survives_reorder_and_inactive_owner() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());

    let Some(other) = harness
        .app_mut()
        .workbench
        .new_query("SELECT * FROM public.customers LIMIT 1")
    else {
        return Err("another stable query tab can be added");
    };
    assert!(harness.app_mut().workbench.reorder_tabs(&[other, key]));
    assert_eq!(harness.app().workbench.active_key(), Some(other));

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(result_rows(&harness, other), None);
    assert_eq!(harness.app().workbench.active_key(), Some(key));
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn a_closed_tab_is_never_replaced_by_its_active_neighbor() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());

    assert!(harness.app_mut().workbench.close_tab_from_picker(key));
    let Some(neighbor) = harness
        .app_mut()
        .workbench
        .new_query("SELECT * FROM public.customers LIMIT 1")
    else {
        return Err("a neighboring query tab can be created");
    };
    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(harness.app().status(), "Work changed; request again");
    assert_eq!(result_rows(&harness, neighbor), None);
    assert_eq!(query_status(&harness, neighbor), None);
    assert_eq!(harness.app().workbench.active_key(), Some(neighbor));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn resize_preserves_acknowledgement_and_a_closed_review_can_reopen() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let _ = harness.type_str("orders");
    let _ = harness.resize(96, 28);
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert_eq!(
        harness.focus(),
        Some(Dialog::new(QUERY_SAFETY).input_id()),
        "resize retains the target acknowledgement field"
    );
    confirm_query(&mut harness);
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(query_status(&harness, key), Some((false, 42)));

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(3));
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn closing_and_reopening_review_does_not_capture_escape_from_help() -> Result<(), &'static str> {
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, SELECT_ORDERS)?;

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), None);

    let _ = harness.key(KeyCode::F(1));
    assert!(harness.layer_area(HELP).is_some());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(HELP).is_none());

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    let _ = harness.type_str("orders");
    confirm_query(&mut harness);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(3));

    let _ = harness.key(KeyCode::F(1));
    assert!(harness.layer_area(HELP).is_some());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(HELP).is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn full_sql_preview_scrolls_at_narrow_size_and_survives_resize() -> Result<(), &'static str> {
    let comments = (0..24)
        .map(|line| format!("-- review-tail-{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let sql = format!("{SELECT_ORDERS}\n{comments}");
    let (mut harness, key) = query_harness(SafeMode::SafeFull, 4, &sql)?;
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert!(harness.text().contains(SELECT_ORDERS));
    assert!(!harness.text().contains("review-tail-23"));

    let _ = harness.resize(72, 20);
    assert!(harness.area_of(QUERY_PREVIEW).is_some());
    let _ = harness.click_id(QUERY_PREVIEW);
    for _ in 0..16 {
        let _ = harness.key(KeyCode::PageDown);
    }
    assert!(harness.text().contains("review-tail-23"));
    assert!(harness.layer_area(QUERY_SAFETY).is_some());

    let _ = harness.resize(120, 40);
    assert!(harness.text().contains("review-tail-23"));
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), None);
    assert_eq!(query_status(&harness, key), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn pending_result_replacement_opens_a_separate_query_safety_review() -> Result<(), &'static str> {
    let (mut harness, key) = pending_result_harness()?;
    let pending_before = harness.app().result().pending().clone();
    let history_before = harness.app().workbench.history.entries.len();

    let _ = harness.ctrl('r');
    assert!(harness.layer_area(REPLACE_RESULTS).is_some());
    let _ = harness.click_id(Dialog::new(REPLACE_RESULTS).action_id(1));

    assert!(harness.layer_area(REPLACE_RESULTS).is_none());
    assert!(harness.layer_area(QUERY_SAFETY).is_some());
    assert_eq!(harness.app().result().pending(), &pending_before);

    let _ = harness.type_str("orders");
    confirm_query(&mut harness);

    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(result_rows(&harness, key), Some(1));
    assert_eq!(harness.app().result().pending_total(), 0);
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn cancelling_either_pending_result_review_preserves_pending_rows() -> Result<(), &'static str> {
    for cancel_replacement in [true, false] {
        let (mut harness, key) = pending_result_harness()?;
        let pending_before = harness.app().result().pending().clone();
        let history_before = harness.app().workbench.history.entries.len();

        let _ = harness.ctrl('r');
        assert!(harness.layer_area(REPLACE_RESULTS).is_some());
        if cancel_replacement {
            let _ = harness.key(KeyCode::Esc);
        } else {
            let _ = harness.click_id(Dialog::new(REPLACE_RESULTS).action_id(1));
            assert!(harness.layer_area(QUERY_SAFETY).is_some());
            let _ = harness.key(KeyCode::Esc);
        }

        assert!(harness.layer_area(REPLACE_RESULTS).is_none());
        assert!(harness.layer_area(QUERY_SAFETY).is_none());
        assert_eq!(harness.app().result().pending(), &pending_before);
        assert_eq!(result_rows(&harness, key), Some(1));
        assert_eq!(query_status(&harness, key), Some((false, 42)));
        assert_eq!(
            harness.app().workbench.history.entries.len(),
            history_before
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn changes_while_pending_replacement_is_open_fail_closed() -> Result<(), &'static str> {
    let (mut harness, key) = pending_result_harness()?;
    let pending_before = harness.app().result().pending().clone();
    let _ = harness.ctrl('r');
    assert!(harness.layer_area(REPLACE_RESULTS).is_some());

    let Some((_, grid)) = harness.app_mut().workbench.active_grid_mut() else {
        return Err("pending result remains available");
    };
    if grid.model.commit_cell(0, 1, "USD").is_err() {
        return Err("a second pending edit can be staged");
    }
    let pending_after_change = harness.app().result().pending().clone();
    assert_ne!(pending_after_change, pending_before);

    let _ = harness.click_id(Dialog::new(REPLACE_RESULTS).action_id(1));
    assert!(harness.layer_area(REPLACE_RESULTS).is_none());
    assert!(harness.layer_area(QUERY_SAFETY).is_none());
    assert_eq!(harness.app().status(), "Work changed; request again");
    assert_eq!(harness.app().result().pending(), &pending_after_change);
    assert_eq!(result_rows(&harness, key), Some(1));
    assert_eq!(query_status(&harness, key), Some((false, 42)));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}
