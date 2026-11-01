use tui_test::TerminalSnapshot;

use super::support::{App, Point, SHOWCASE, TABLEPRO, crop, find, find_all, row_text, text};

fn inside(point: Point) -> Point {
    Point {
        x: point.x + 2,
        ..point
    }
}

fn button_lifecycle(color: &str) {
    let app = App::new(
        &format!("showcase-buttons-{color}"),
        SHOWCASE,
        &["--page", "buttons", "--motion", "paused"],
        160,
        50,
        color,
    );
    let initial = app.wait_text("Disabled primary");
    let run = find(&initial, "Run task");
    let preview = find(&initial, "Preview");
    let disabled = find(&initial, "Disabled primary");
    let toggle = find(&initial, "Auto-approve");
    let outside = find(&initial, "Actions");
    let rest = crop(&initial, run, 8, 1);
    assert!(!text(&initial).contains("Run task ✓"));

    app.move_to(inside(run));
    let hovered = app.wait_screen("Run task visibly responds to hover", |screen| {
        crop(screen, run, 8, 1) != rest
    });
    let hover = crop(&hovered, run, 8, 1);
    assert!(!text(&hovered).contains("Run task ✓"));
    app.down(inside(run));
    let pressed = app.wait_screen("held Run task differs from hover and rest", |screen| {
        let current = crop(screen, run, 8, 1);
        current != hover && current != rest
    });
    assert!(
        !text(&pressed).contains("Run task ✓"),
        "press must not activate"
    );
    app.up(inside(run));
    app.wait_text("Run task ✓");

    // Give cancellation a distinct prior result: a spurious Run activation
    // would replace Preview in the visible status line.
    app.click(inside(preview));
    app.wait_text("Preview ✓");
    app.down(inside(run));
    app.drag_to(outside);
    app.up(outside);
    let cancelled = app.settle();
    assert!(text(&cancelled).contains("Preview ✓"));
    assert!(!text(&cancelled).contains("Run task ✓"));

    // The toggle is an observable state change, so a duplicate activation on
    // press and release cannot pass by merely rewriting the same status text.
    app.click(inside(toggle));
    let on = app.wait_text("Auto-approve on");
    assert!(row_text(&on, u32::from(toggle.y)).contains("● Auto-approve"));
    app.click(inside(toggle));
    let off = app.wait_text("Auto-approve off");
    assert!(row_text(&off, u32::from(toggle.y)).contains("○ Auto-approve"));

    app.move_to(outside);
    let disabled_rest = crop(&app.settle(), disabled, 16, 1);
    app.move_to(inside(disabled));
    assert_eq!(crop(&app.settle(), disabled, 16, 1), disabled_rest);
    app.down(inside(disabled));
    assert_eq!(crop(&app.settle(), disabled, 16, 1), disabled_rest);
    app.up(inside(disabled));
    let rejected = app.settle();
    assert!(text(&rejected).contains("Auto-approve off"));
    assert!(!text(&rejected).contains("Disabled primary ✓"));
    app.click(inside(preview));
    let after_disabled = app.wait_text("Preview ✓");
    assert_eq!(crop(&after_disabled, disabled, 16, 1), disabled_rest);
    app.finish();
}

#[test]
fn showcase_button_hover_hold_release_cancel_and_disabled_truecolor() {
    button_lifecycle("truecolor");
}

#[test]
fn showcase_button_hover_hold_release_cancel_and_disabled_without_color() {
    button_lifecycle("nocolor");
}

#[test]
fn showcase_button_hover_hold_release_cancel_and_disabled_256_colors() {
    button_lifecycle("256");
}

#[test]
fn showcase_button_hover_hold_release_cancel_and_disabled_16_colors() {
    button_lifecycle("16");
}

#[test]
fn showcase_button_hover_hold_release_cancel_and_disabled_monochrome() {
    button_lifecycle("none");
}

fn visible_list_rows(screen: &TerminalSnapshot) -> Vec<u16> {
    (0..screen.rows())
        .filter_map(|row| {
            let line = row_text(screen, row);
            let (_, number) = line.split_once("Row ")?;
            number.get(..3)?.parse().ok()
        })
        .collect()
}

fn cropped_text(screen: &TerminalSnapshot, point: Point, width: u16, height: u16) -> String {
    crop(screen, point, width, height)
        .iter()
        .map(|cell| cell.text())
        .collect()
}

#[test]
fn showcase_wheel_targets_the_pointed_pane_then_keys_follow_clicked_focus() {
    let app = App::new(
        "showcase-nested-scroll",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        160,
        50,
        "truecolor",
    );
    let initial = app.wait_text("739.63s");
    let prose = find(&initial, "Junie works through a task");
    let list = find(&initial, "Row 001");
    app.click(inside(prose));
    let focused = app.wait_text("↑↓ Scroll");
    let prose_before = cropped_text(&focused, prose, 30, 10);
    let rows_before = visible_list_rows(&focused);
    assert_eq!(rows_before.first(), Some(&1));
    app.key("pagedown");
    app.wait_screen("PageDown scrolls the focused prose pane", |screen| {
        cropped_text(screen, prose, 30, 10) != prose_before
    });
    app.key("home");
    app.wait_screen("Home restores the focused prose pane", |screen| {
        cropped_text(screen, prose, 30, 10) == prose_before
    });

    app.wheel(inside(list), true, 3);
    let scrolled = app.wait_screen("wheel advances the list under the pointer", |screen| {
        visible_list_rows(screen)
            .first()
            .is_some_and(|first| *first > 1)
    });
    assert_eq!(cropped_text(&scrolled, prose, 30, 10), prose_before);
    assert!(
        text(&scrolled).contains("row 1 · vertical"),
        "wheel must not move the list cursor"
    );
    let row = (0..scrolled.rows())
        .find_map(|y| {
            let line = row_text(&scrolled, y);
            if line.contains("retired") {
                return None;
            }
            line.split_once("Row ")?.1.get(..3)?.parse::<u16>().ok()
        })
        .expect("a visible list row without the retired label");
    app.click_text(&format!("Row {row:03}"));
    app.wait_text(&format!("row {row} · vertical"));
    app.key("end");
    let end = app.wait_text("row 119 · vertical");
    assert!(text(&end).contains("Row 119"));
    assert!(!visible_list_rows(&end).contains(&1));
    app.key("home");
    let home = app.wait_text("row 1 · vertical");
    assert_eq!(visible_list_rows(&home).first(), Some(&1));
    assert_eq!(cropped_text(&home, prose, 30, 10), prose_before);
    app.finish();
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
    app.finish();
}
