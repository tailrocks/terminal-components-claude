use super::support::{App, Point, TABLEPRO, crop, find, find_all, row_text, text};

fn inside(point: Point) -> Point {
    Point {
        x: point.x + 2,
        ..point
    }
}

fn dialog_action(app: &App, label: &str) -> Point {
    let screen = app.screen();
    let actions: Vec<_> = find_all(&screen, label)
        .into_iter()
        .filter(|point| row_text(&screen, u32::from(point.y)).contains("Cancel"))
        .collect();
    assert_eq!(
        actions.len(),
        1,
        "one dialog action {label}:\n{}",
        text(&screen)
    );
    inside(actions[0])
}

#[test]
fn tablepro_write_confirmation_cancels_retries_and_executes_only_when_armed() {
    let app = App::new(
        "tablepro-write-gate",
        TABLEPRO,
        &["--connect", "Production", "--motion", "reduced"],
        160,
        50,
        "truecolor",
    );
    app.wait_text("Query 1");
    app.key("tab");
    app.paste("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    app.wait_text("WHERE id = 'x'");
    app.key("escape");
    app.key("ctrl+r");
    let gate = app.wait_text("Type orders to confirm");
    assert!(!text(&gate).contains("rows affected"));
    let outside = find(&gate, "Explorer");
    app.click(outside);
    let refused = app.settle();
    assert!(text(&refused).contains("Type orders to confirm"));
    assert!(!text(&refused).contains("rows affected"));
    app.key("escape");
    let cancelled = app.wait_text("Cancelled · nothing was executed");
    assert!(!text(&cancelled).contains("Type orders to confirm"));
    assert!(!text(&cancelled).contains("rows affected"));

    app.key("ctrl+r");
    app.wait_text("Type orders to confirm");
    app.type_text("wrong");
    app.wait_text("wrong");
    let execute = dialog_action(&app, "Execute");
    let disabled = crop(&app.screen(), execute, 5, 1);
    app.move_to(execute);
    assert_eq!(crop(&app.settle(), execute, 5, 1), disabled);
    app.down(execute);
    let held = app.settle();
    assert_eq!(crop(&held, execute, 5, 1), disabled);
    assert!(!text(&held).contains("rows affected"));
    app.up(execute);
    app.move_to(outside);
    // Let the preceding click feedback finish while Execute is still disabled,
    // before its enabled appearance becomes the next comparison baseline.
    app.settle();

    // Typing the required token proves the rejected click did not close the
    // modal or transfer typing to the underlying SQL editor.
    app.key("ctrl+l");
    app.type_text("orders");
    let enabled = app.wait_screen("confirmation token enables Execute", |screen| {
        crop(screen, execute, 5, 1) != disabled && text(screen).contains("Type orders to confirm")
    });
    let rest = crop(&enabled, execute, 5, 1);
    app.move_to(execute);
    let hover = app.wait_screen("enabled Execute responds to hover", |screen| {
        crop(screen, execute, 5, 1) != rest
    });
    let hover = crop(&hover, execute, 5, 1);
    app.down(execute);
    let armed = app.wait_screen("armed Execute visibly responds to held press", |screen| {
        let cells = crop(screen, execute, 5, 1);
        cells != hover && cells != rest
    });
    assert!(!text(&armed).contains("rows affected"));
    app.up(execute);
    let executed = app.wait_text("rows affected");
    assert!(!text(&executed).contains("Type orders to confirm"));

    // Close the dirty query through its public leave guard, then quit cleanly.
    // Returning from the modal resumes text editing; leave it before Ctrl+W,
    // which is an editor word-deletion shortcut while the EDIT footer is shown.
    app.key("escape");
    let navigation = app.wait_text("Ctrl+X Explain");
    assert!(text(&navigation).contains("UPDATE orders SET status = 'paid' WHERE id = 'x'"));
    app.key("ctrl+w");
    app.wait_text("Close tab with unsaved work?");
    app.click_text("Close anyway");
    app.wait_text("No open tabs");
    app.visual("functional/tablepro_write_gate");
    app.finish();
}

#[test]
fn tablepro_connection_failure_retries_then_another_connection_succeeds() {
    let app = App::new(
        "tablepro-connection-recovery",
        TABLEPRO,
        &["--motion", "reduced"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Filter connections");
    app.click_text("Staging");
    app.wait_text("staging-db.acme.io");
    let connect = app.point("Connect    Edit");
    let rest = crop(&app.screen(), connect, 7, 1);
    app.move_to(inside(connect));
    let hover = app.wait_screen("Connect hover changes its own cells", |screen| {
        crop(screen, connect, 7, 1) != rest
    });
    let hover = crop(&hover, connect, 7, 1);
    app.down(inside(connect));
    let held = app.wait_screen("Connect held state differs from hover", |screen| {
        let cells = crop(screen, connect, 7, 1);
        cells != hover && cells != rest
    });
    assert!(!text(&held).contains("Opening SSH tunnel"));
    app.up(inside(connect));
    app.wait_text("Authentication failed");
    app.click_text("Reconnect");
    app.wait_text("Opening SSH tunnel");
    app.wait_text("Authentication failed");
    app.click_text("Local PostgreSQL");
    app.wait_text("localhost:5432");
    app.click_text("Connect    Edit");
    let connected = app.wait_text("Connected to Local PostgreSQL");
    assert!(text(&connected).contains("Query 1"));
    assert!(!text(&connected).contains("Filter connections"));
    assert!(!text(&connected).contains("Authentication failed"));
    app.visual("functional/tablepro_connection_recovery");
    app.finish();
}

// ---------------------------------------------------------------- TablePro

const CLOSE_PROMPT: &str = "Close tab with unsaved work?";
const TARGET_SQL: &str = "SELECT 'unsaved_picker_regression';";
const OTHER_SQL: &str = "SELECT 'other_query_survives';";

fn tablepro_connected(name: &str) -> App {
    let app = App::new(
        name,
        TABLEPRO,
        &["--connect", "Local PostgreSQL", "--motion", "reduced"],
        120,
        40,
        "none",
    );
    app.wait_text("Query 1");
    app
}

fn new_query(app: &App, name: &str, sql: Option<&str>) {
    app.key("ctrl+t");
    app.wait_text(name);
    if let Some(sql) = sql {
        app.paste(sql);
        app.wait_text(sql);
    }
    app.key("f6");
}

fn open_tabs(app: &App, filter: Option<&str>) {
    app.key("ctrl+g");
    app.wait_screen("Open tabs picker", |s| {
        let t = text(s);
        t.contains("Open tabs") && t.contains("Delete Close tab")
    });
    if let Some(filter) = filter {
        app.paste(filter);
        app.wait_text(filter);
    }
}

fn delete_requires_confirmation(app: &App) {
    app.key("delete");
    app.wait_text(CLOSE_PROMPT);
}

fn cancel_close(app: &App, active_sql: &str) {
    app.key("escape");
    app.wait_screen("Cancel preserves the active query", |s| {
        let t = text(s);
        !t.contains(CLOSE_PROMPT) && !t.contains("Open tabs") && t.contains(active_sql)
    });
}

fn confirm_close(app: &App, removed: &str, retained: &str) {
    app.click_text("Close anyway");
    app.wait_screen("only the confirmed tab is closed", |s| {
        let t = text(s);
        !t.contains(CLOSE_PROMPT)
            && !t.contains("Open tabs")
            && !t.contains(removed)
            && t.contains(retained)
    });
}

#[test]
fn tablepro_dirty_picker_close_matches_ctrl_w_and_cancel_preserves_sql() {
    let app = tablepro_connected("tablepro-dirty-picker");
    new_query(&app, "Query 2", Some(TARGET_SQL));

    app.key("ctrl+w");
    app.wait_text(CLOSE_PROMPT);
    cancel_close(&app, TARGET_SQL);

    open_tabs(&app, None);
    delete_requires_confirmation(&app);
    cancel_close(&app, TARGET_SQL);

    open_tabs(&app, None);
    delete_requires_confirmation(&app);
    confirm_close(&app, "Query 2", "Query 1");
    app.visual("functional/tablepro_dirty_picker");
    app.finish();
}

#[test]
fn tablepro_filtered_picker_closes_requested_dirty_tab_and_preserves_active() {
    let app = tablepro_connected("tablepro-filtered-picker");
    new_query(&app, "Query 2", Some(TARGET_SQL));
    new_query(&app, "Query 3", Some(OTHER_SQL));

    open_tabs(&app, Some("query 2"));
    delete_requires_confirmation(&app);
    cancel_close(&app, OTHER_SQL);

    open_tabs(&app, Some("query 2"));
    app.key("enter");
    app.wait_screen("cancelled target retains its SQL", |s| {
        let t = text(s);
        !t.contains("Open tabs") && t.contains(TARGET_SQL)
    });
    open_tabs(&app, Some("query 3"));
    app.key("enter");
    app.wait_screen("returning to the other dirty query", |s| {
        let t = text(s);
        !t.contains("Open tabs") && t.contains(OTHER_SQL)
    });
    app.key("f6");

    open_tabs(&app, Some("query 2"));
    delete_requires_confirmation(&app);
    confirm_close(&app, "Query 2", OTHER_SQL);
    let screen = app.screen();
    let t = text(&screen);
    assert!(
        t.contains("Query 1") && t.contains("Query 3"),
        "filtered close removed the wrong tab:\n{t}"
    );

    open_tabs(&app, None);
    delete_requires_confirmation(&app);
    confirm_close(&app, "Query 3", "Query 1");
    app.visual("functional/tablepro_filtered_picker");
    app.finish();
}

#[test]
fn tablepro_clean_picker_close_is_immediate_and_picker_stays_usable() {
    let app = tablepro_connected("tablepro-clean-picker");
    new_query(&app, "Query 2", None);
    open_tabs(&app, None);
    app.key("delete");
    let closed = app.wait_screen("clean tab closed in the picker", |s| {
        let t = text(s);
        t.contains("Open tabs") && !t.contains("Query 2") && t.contains("Query 1")
    });
    assert!(
        !text(&closed).contains(CLOSE_PROMPT),
        "clean tab must close without an unsaved-work prompt"
    );
    app.key("enter");
    app.wait_screen("remaining tab can still be activated", |s| {
        let t = text(s);
        !t.contains("Open tabs") && t.contains("Query 1")
    });
    app.visual("functional/tablepro_clean_picker");
    app.finish();
}
