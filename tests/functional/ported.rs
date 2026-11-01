//! Journeys ported from the companion functional-tests review (Report A)
//! onto the `tui_test` driver: literal Showcase colours and focus gutter,
//! TablePro dirty-tab close guard, Holla finder/gate journeys, and Jackin
//! launch/editor/help journeys. See `docs/functional-testing.md`.

use tui_test::{SnapshotColor, TerminalSnapshot};

use super::support::{App, HOLLA, JACKIN, Point, SHOWCASE, TABLEPRO, find, row_text, text};

fn inside(p: Point) -> Point {
    Point { x: p.x + 2, ..p }
}

fn bg(screen: &TerminalSnapshot, p: Point) -> SnapshotColor {
    screen
        .cell_at(u32::from(p.x), u32::from(p.y))
        .expect("cell in viewport")
        .bg()
}

// ---------------------------------------------------------------- Showcase

#[test]
fn showcase_primary_button_exact_rest_hover_held_colours() {
    let app = App::new(
        "showcase-primary-colours",
        SHOWCASE,
        &["--page", "buttons", "--motion", "reduced", "--frame", "40"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Run task");
    let run = inside(find(&initial, "Run task"));
    // Literal reviewed values, not values read back from the theme code.
    assert_eq!(bg(&initial, run), SnapshotColor::Rgb(72, 224, 84));
    assert!(!text(&initial).contains("Run task ✓"));

    app.move_to(run);
    let hover = app.wait_screen("primary hover colour", |s| {
        bg(s, run) == SnapshotColor::Rgb(58, 179, 67)
    });
    assert!(!text(&hover).contains("Run task ✓"));

    app.down(run);
    let held = app.wait_screen("primary held colour", |s| {
        bg(s, run) == SnapshotColor::Rgb(46, 154, 56)
    });
    assert!(
        !text(&held).contains("Run task ✓"),
        "press must not activate"
    );
    assert_eq!(
        held.cells().iter().filter(|c| c.text() == "▎").count(),
        1,
        "one keyboard-focus gutter, independent of hover/selection"
    );

    app.up(run);
    app.wait_text("Run task ✓");
    app.wait_screen("release flash returns to hover", |s| {
        bg(s, run) == SnapshotColor::Rgb(58, 179, 67)
    });
    app.finish();
}

#[test]
fn showcase_wheel_scrolls_pointed_grid_without_moving_sidebar() {
    let app = App::new(
        "showcase-datagrid-wheel",
        SHOWCASE,
        &["--page", "datagrid", "--motion", "reduced", "--frame", "40"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Northwind Traders");
    let cell = find(&initial, "1001");
    let sidebar = |s: &TerminalSnapshot| {
        (2..39u32)
            .flat_map(|y| (0..28u32).filter_map(move |x| s.cell_at(x, y).cloned()))
            .collect::<Vec<_>>()
    };
    let before = sidebar(&initial);
    assert!(row_text(&initial, u32::from(cell.y)).contains("1001"));

    app.wheel(inside(cell), true, 1);
    let down = app.wait_screen("wheel brings row 1004 to the top", |s| {
        let row = row_text(s, u32::from(cell.y));
        row.contains("1004") && row.contains("Fabrikam Robotics")
    });
    assert_eq!(sidebar(&down), before, "sidebar must not move");

    app.wheel(inside(cell), false, 1);
    let up = app.wait_screen("wheel returns to row 1001", |s| {
        let row = row_text(s, u32::from(cell.y));
        row.contains("1001") && row.contains("Northwind Traders")
    });
    assert_eq!(sidebar(&up), before);
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
    app.finish();
}

// ------------------------------------------------------------------- Holla

fn holla(name: &str, scenario: &str) -> App {
    App::new(
        name,
        HOLLA,
        &[
            "--scenario",
            scenario,
            "--motion",
            "reduced",
            "--frame",
            "40",
        ],
        120,
        40,
        "truecolor",
    )
}

#[test]
fn holla_finder_search_runs_selected_task_and_records_result() {
    let app = holla("holla-finder-history", "parity-history");
    app.wait_text("Search actions and resources");
    app.paste("cargo check");
    app.wait_text("▎ cargo check");
    app.key("enter");
    let done = app.wait_screen("successful task result", |s| {
        let t = text(s);
        t.contains("succeeded") && t.contains("exit 0") && t.contains("Finished `dev` profile")
    });
    assert!(text(&done).contains("$ cargo check"));
    app.key("ctrl+g");
    let history = app.wait_screen("completed activity in picker", |s| {
        let t = text(s);
        t.contains("0 live") && t.contains("cargo check") && t.contains("succeeded")
    });
    assert!(!text(&history).contains("Nothing has been started yet"));
    app.key("escape");
    app.finish();
}

#[test]
fn holla_cancelling_second_gate_leaves_no_started_activity() {
    let app = holla("holla-second-gate", "docker-cleanup");
    app.wait_text("Clean Docker completely");
    app.key("enter");
    app.wait_text("Here › Clean Docker completely");
    app.key("c");
    app.wait_screen("first confirmation gate", |s| {
        let t = text(s);
        t.contains("gate 1 of 2") && t.contains("Sequence · 8 commands")
    });
    app.key("right");
    app.key("enter");
    app.wait_screen("second confirmation gate", |s| {
        let t = text(s);
        t.contains("gate 2 of 2")
            && t.contains("Type I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox")
    });
    app.key("escape");
    let cancelled = app.wait_text("Cancelled · nothing was executed");
    assert!(!text(&cancelled).contains("gate 2 of 2"));
    // Independent acknowledgement: the activity picker must be empty.
    app.key("ctrl+g");
    let empty = app.wait_screen("empty activity picker", |s| {
        let t = text(s);
        t.contains("0 live") && t.contains("Nothing has been started yet")
    });
    assert!(!text(&empty).contains("gate 2 of 2"));
    app.key("escape");
    app.finish();
}

#[test]
fn holla_browser_pointer_selection_loads_preview_and_hidden_toggle() {
    let app = holla("holla-browser-pointer", "parity-browser");
    app.wait_text("Search actions and resources");
    app.paste("Browse ~/work/site");
    app.wait_text("▎ Browse ~/work/site");
    app.key("enter");
    let listing = app.wait_screen("loaded directory listing", |s| {
        let t = text(s);
        t.contains("Files · ~/work/site") && t.contains("16 entries")
    });
    assert!(!text(&listing).contains(".env"));

    let target = find(&listing, "index.html");
    app.click(target);
    let preview = app.wait_screen("selected HTML preview", |s| {
        let t = text(s);
        t.contains("<html>") && t.contains("hello")
    });
    assert!(text(&preview).contains("index.html"));

    app.key("ctrl+h");
    let shown = app.wait_text("18 entries · 2 hidden shown");
    assert!(text(&shown).contains(".env") && text(&shown).contains(".git"));
    app.key("ctrl+h");
    let hidden = app.wait_text("16 entries · 2 hidden hidden");
    assert!(!text(&hidden).contains(".env"));
    app.finish();
}

// ------------------------------------------------------------------ Jackin

fn jackin(name: &str, motion: &str) -> App {
    App::new(
        name,
        JACKIN,
        &[
            "--scenario",
            "returning",
            "--motion",
            motion,
            "--frame",
            "40",
        ],
        120,
        40,
        "truecolor",
    )
}

#[test]
fn jackin_launch_detach_reconnect_preserves_capsule() {
    let app = jackin("jackin-launch", "paused");
    let manager = app.wait_text("Current directory");
    assert!(text(&manager).contains("2 running"));
    app.key("enter");
    let picker = app.wait_text("Launch · choose Agent");
    assert!(text(&picker).contains("payments-platform › the-architect"));
    app.key("enter");
    app.wait_text("Loading the-architect into workspace payments-platform");
    app.advance_clock(30_000);
    let capsule = app.wait_screen("launch hands off to capsule", |s| {
        let t = text(s);
        t.contains("Claude Code v2.1.14")
            && t.contains("Allow edit to src/settlement/config.rs? (y/n)")
            && t.contains("Ctrl+B Pane commands")
    });
    assert!(!text(&capsule).contains("junie-tui:advance-clock:"));
    assert!(!text(&capsule).contains("Launch failed"));

    app.key("ctrl+b");
    app.wait_text("prefix…");
    app.key("d");
    let detached = app.wait_text("Detached · payments-platform keeps running");
    assert!(text(&detached).contains("3 running"));

    app.key("enter");
    app.wait_screen("reconnect restores live capsule", |s| {
        let t = text(s);
        t.contains("Claude Code v2.1.14") && t.contains("Ctrl+B Pane commands")
    });
    app.key("ctrl+b");
    app.wait_text("prefix…");
    app.key("d");
    app.wait_text("Detached · payments-platform keeps running");
    app.finish();
}

#[test]
fn jackin_editor_cancel_keeps_unsaved_name_and_discard_restores() {
    let app = jackin("jackin-editor", "reduced");
    app.wait_text("Current directory");
    app.key("down");
    app.wait_text("Workspaces › payments-platform");
    app.key("e");
    app.wait_text("Directory basename by default");
    app.key("tab");
    app.wait_text("Enter Apply");
    app.key("end");
    app.paste("-functional");
    app.wait_text("payments-platform-functional");
    app.key("enter");
    app.wait_text("Name payments-platform-functional · save to apply");

    app.click_text("Cancel");
    let dialog = app.wait_text("Unsaved changes");
    assert!(text(&dialog).contains("1 change would be"));
    app.key("enter");
    app.wait_screen("cancel returns to unchanged pending edit", |s| {
        let t = text(s);
        !t.contains("Unsaved changes")
            && t.contains("payments-platform-functional")
            && t.contains("1 change")
    });
    app.key("enter");
    app.wait_text("Unsaved changes");
    app.key("right");
    app.wait_text("Enter Discard");
    app.key("enter");
    let discarded = app.wait_text("Discarded 1 change");
    assert!(!text(&discarded).contains("payments-platform-functional"));

    app.key("e");
    let reopened = app.wait_text("Directory basename by default");
    assert!(text(&reopened).contains("payments-platform"));
    assert!(!text(&reopened).contains("payments-platform-functional"));
    assert!(!text(&reopened).contains("• 1 change"));
    app.finish();
}

#[test]
fn jackin_help_owns_keys_and_wheel_then_restores_manager_focus() {
    let app = jackin("jackin-help", "reduced");
    app.wait_text("Current directory");
    app.key("down");
    app.wait_text("Workspaces › payments-platform");
    app.key("?");
    let top = app.wait_text("1–29 of 30");
    assert!(text(&top).contains("Keyboard shortcuts"));

    app.key("e");
    app.key("pagedown");
    let bottom = app.wait_text("2–30 of 30");
    assert!(text(&bottom).contains("Keyboard shortcuts"));
    assert!(!text(&bottom).contains("Directory basename by default"));

    let point = find(&bottom, "scroll under pointer");
    app.wheel(point, false, 1);
    app.wait_screen("help wheel restores top rows", |s| {
        let t = text(s);
        t.contains("Keyboard shortcuts") && t.contains("1–29 of 30")
    });
    app.key("escape");
    let manager = app.wait_screen("help closes onto same workspace", |s| {
        let t = text(s);
        !t.contains("Keyboard shortcuts")
            && t.contains("Workspaces › payments-platform")
            && t.contains("saved workspace · 1 running")
    });
    assert!(!text(&manager).contains("› edit"));

    app.key("enter");
    let picker = app.wait_text("Launch · choose Agent");
    assert!(text(&picker).contains("payments-platform › the-architect"));
    app.key("escape");
    let cancelled = app.wait_screen("picker cancellation returns to manager", |s| {
        let t = text(s);
        !t.contains("Launch · choose Agent") && t.contains("Current directory")
    });
    assert!(text(&cancelled).contains("2 running"));
    app.finish();
}
