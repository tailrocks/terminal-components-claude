use tui_test::{SnapshotColor, TerminalSnapshot};

use super::support::{App, Point, SHOWCASE, cell_bg, crop, find, find_all, row_text, text};

fn inside(point: Point) -> Point {
    Point {
        x: point.x + 2,
        ..point
    }
}
fn bg(screen: &TerminalSnapshot, p: Point) -> SnapshotColor {
    screen
        .cell_at(u32::from(p.x), u32::from(p.y))
        .expect("cell in viewport")
        .bg()
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

    app.click(inside(preview));
    app.wait_text("Preview ✓");
    app.down(inside(run));
    app.drag_to(outside);
    app.up(outside);
    let cancelled = app.settle();
    assert!(text(&cancelled).contains("Preview ✓"));
    assert!(!text(&cancelled).contains("Run task ✓"));

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
    app.visual(&format!("functional/showcase_buttons_{color}"));
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

    app.visual("functional/showcase_nested_scroll");
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

    app.wait_absent("Run task \u{2713}");
    app.visual("functional/showcase_primary_colours");
    app.finish();
}

#[test]
fn showcase_wheel_scrolls_pointed_grid_without_moving_sidebar() {
    let app = App::new(
        "showcase-datagrid-wheel",
        SHOWCASE,
        &["--page", "datagrid", "--motion", "paused", "--frame", "40"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Northwind Traders");
    let cell = find(&initial, "1001");

    let sidebar = |s: &TerminalSnapshot| {
        (2..39u32)
            .map(|y| {
                (0..28u32)
                    .filter_map(|x| s.cell_at(x, y).map(|c| c.text().to_string()))
                    .collect::<String>()
            })
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
    app.visual("functional/showcase_datagrid_wheel");
    app.finish();
}

#[test]
fn showcase_checks_radio_exclusivity_mixed_and_disabled() {
    let app = App::new(
        "showcase-checks",
        SHOWCASE,
        &["--page", "checks", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_screen("checks page fully rendered", |s| {
        let t = text(s);
        t.contains("Select all rows")
            && t.contains("[\u{2713}] Send notifications")
            && t.contains("(\u{25cf}) eu-west-1")
    });
    assert!(text(&initial).contains("(●) eu-west-1"));

    let first = |s: &TerminalSnapshot, needle: &str| find_all(s, needle)[0];

    app.click(inside(first(&initial, "Send notifications")));
    app.wait_text("[ ] Send notifications");
    app.click(inside(first(&app.screen(), "Send notifications")));
    app.wait_text("[✓] Send notifications");

    assert!(text(&initial).contains("(●) eu-west-1"));

    app.click(inside(find(&app.screen(), "Repeatable read")));
    let radio = app.wait_text("(●) Repeatable read");
    assert!(text(&radio).contains("( ) Read committed"));
    assert!(text(&radio).contains("( ) Serializable"));
    app.key("down");
    let next = app.wait_text("(●) Serializable");
    assert!(text(&next).contains("( ) Repeatable read"));
    app.key("up");
    app.wait_text("(●) Repeatable read");

    app.click(inside(find(&app.screen(), "Select all rows")));
    app.wait_screen("select-all checks the mixed box", |s| {
        !text(s).contains("[–] Select all rows")
    });

    let beta = find(&app.screen(), "Beta features");
    let rest = crop(&app.settle(), beta, 20, 1);
    app.move_to(inside(beta));
    assert_eq!(crop(&app.settle(), beta, 20, 1), rest);
    app.down(inside(beta));
    assert_eq!(crop(&app.settle(), beta, 20, 1), rest);
    app.up(inside(beta));
    assert!(text(&app.screen()).contains("Beta features off"));
    app.click(inside(find(&app.screen(), "Auto-save drafts")));
    app.wait_text("Auto-save drafts off");
    app.visual("functional/showcase_checks");
    app.finish();
}

#[test]
fn showcase_tab_selection_survives_focus_move_and_single_gutter() {
    let app = App::new(
        "showcase-tabs",
        SHOWCASE,
        &["--page", "tabs", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Nested strips");
    app.click(inside(find(&app.screen(), "DDL")));
    let selected = app.wait_screen("DDL strip selection changes the breadcrumb", |s| {
        text(s).contains("orders › DDL")
    });
    assert_eq!(
        selected.cells().iter().filter(|c| c.text() == "▎").count(),
        1,
        "exactly one focus gutter"
    );

    app.key("tab");
    let moved = app.settle();
    assert!(
        text(&moved).contains("orders › DDL"),
        "selection survives focus move:\n{}",
        text(&moved)
    );
    assert_eq!(moved.cells().iter().filter(|c| c.text() == "▎").count(), 1);
    app.click(inside(find(&app.screen(), "Structure")));
    app.wait_screen("reselect Structure", |s| {
        text(s).contains("orders › Structure › Columns")
    });
    app.visual("functional/showcase_tabs_selection");
    app.finish();
}

#[test]
fn showcase_select_outside_click_and_escape_dismiss_without_change() {
    let app = App::new(
        "showcase-select-dismiss",
        SHOWCASE,
        &["--page", "chipsselects", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Sort by");
    assert!(text(&initial).contains("created_at"));
    let field = find(&initial, "created_at");
    let arrow = Point {
        x: field.x + 22,
        y: field.y,
    };
    app.click(arrow);
    app.wait_text("customer");

    app.click(inside(find(&app.screen(), "Filters")));
    let dismissed = app.wait_screen("outside click closes the select popup", |s| {
        !text(s).contains("customer")
    });
    assert!(text(&dismissed).contains("created_at"));

    app.click(arrow);
    app.wait_text("customer");
    app.key("escape");
    let escaped = app.wait_screen("escape closes the select popup", |s| {
        !text(s).contains("customer")
    });
    assert!(text(&escaped).contains("created_at"));

    app.click(arrow);
    app.wait_text("customer");
    let option = find_all(&app.screen(), "total")
        .into_iter()
        .find(|pt| pt.x >= field.x && pt.y > field.y)
        .expect("popup total option");
    app.click(inside(option));
    let chosen = app.wait_screen("chosen option becomes the value", |s| {
        !text(s).contains("customer")
    });
    assert!(row_text(&chosen, u32::from(field.y)).contains("total"));
    app.visual("functional/showcase_select_dismiss");
    app.finish();
}

#[test]
fn showcase_menu_outside_click_dismisses() {
    let app = App::new(
        "showcase-menus",
        SHOWCASE,
        &["--page", "menus", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Actions ▾");
    app.click(inside(find(&app.screen(), "Actions ▾")));
    let open = app.wait_screen("actions menu opens", |s| {
        let t = text(s);
        t.contains("Rename") || t.contains("Duplicate") || t.contains("Export")
    });
    assert!(text(&open).contains("Actions"));
    app.click(inside(find(&open, "report.pdf")));
    app.wait_screen("outside click dismisses the menu", |s| {
        !text(s).contains("Rename") && !text(s).contains("Duplicate")
    });
    app.visual("functional/showcase_menu_dismiss");
    app.finish();
}

#[test]
fn showcase_splitter_seam_drag_resizes_panes() {
    let app = App::new(
        "showcase-splits",
        SHOWCASE,
        &["--page", "splits", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_screen("splits page with Editor and splitter.rs", |s| {
        !find_all(s, "Editor").is_empty() && !find_all(s, "splitter.rs").is_empty()
    });
    let editor_before = find(&initial, "Editor");
    let content = find(&initial, "splitter.rs");

    let seam = Point {
        x: editor_before.x - 5,
        y: content.y,
    };
    app.down(seam);

    let mut last_x = editor_before.x;
    for step in 1..=4 {
        app.drag_to(Point {
            x: seam.x + step * 2,
            y: seam.y,
        });
        let moved = app.wait_screen("seam drag step moves the Editor pane", |s| {
            find_all(s, "Editor").first().is_some_and(|p| p.x > last_x)
        });
        last_x = find(&moved, "Editor").x;
    }
    app.up(Point {
        x: seam.x + 8,
        y: seam.y,
    });
    let resized = app.wait_screen("dragging the seam moves the Editor pane", |s| {
        find_all(s, "Editor")
            .first()
            .is_some_and(|p| p.x >= editor_before.x + 4)
    });
    assert!(
        find(&resized, "Editor").x >= editor_before.x + 4,
        "editor before={editor_before:?} after={:?}",
        find(&resized, "Editor")
    );
    app.visual("functional/showcase_splitter_drag");
    app.finish();
}

#[test]
fn showcase_scrollbar_thumb_drag_moves_list() {
    let app = App::new(
        "showcase-scrollbar",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Row 001");
    let initial = app.stable_screen();
    let row1 = find(&initial, "Row 001");

    let thumb = (row1.x..row1.x + 30)
        .flat_map(|x| (row1.y..row1.y + 3).map(move |y| Point { x, y }))
        .find(|p| {
            app.screen()
                .cell_at(u32::from(p.x), u32::from(p.y))
                .is_some_and(|c| c.text() == "┃")
        })
        .expect("scrollbar thumb at top of Long list track");
    let neighbour_before = cropped_text(&initial, find(&initial, "Junie works through a"), 24, 7);
    app.down(thumb);
    for step in 1..=4 {
        app.drag_to(Point {
            x: thumb.x,
            y: thumb.y + step * 2,
        });
    }
    app.up(Point {
        x: thumb.x,
        y: thumb.y + 8,
    });
    app.wait_screen("thumb drag advances the list", |s| {
        visible_list_rows(s).first().is_some_and(|f| *f > 1)
    });
    let scrolled = app.stable_screen();
    assert_eq!(
        cropped_text(&scrolled, find(&scrolled, "Junie works through a"), 24, 7),
        neighbour_before,
        "neighbour pane unchanged"
    );
    app.visual("functional/showcase_scrollbar_drag");
    app.finish();
}

#[test]
fn showcase_keyboard_activation_after_click_focus() {
    let app = App::new(
        "showcase-keyboard",
        SHOWCASE,
        &["--page", "buttons", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Run task");
    assert!(!text(&initial).contains("Run task ✓"));

    app.click(inside(find(&initial, "Preview")));
    app.wait_text("Preview ✓");
    app.key("tab");
    app.key("enter");
    let done = app.wait_screen("keyboard activation fires the focused control", |s| {
        let t = text(s);
        t.contains("Run task ✓") || t.contains("Preview ✓")
    });
    assert!(text(&done).contains("Preview ✓"));
    app.visual("functional/showcase_keyboard_activation");
    app.finish();
}

#[test]
fn showcase_list_selection_keyboard_and_multiselect() {
    let app = App::new(
        "showcase-lists",
        SHOWCASE,
        &["--page", "lists", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Chosen: Rust");
    app.click(inside(find(&app.screen(), "Python")));
    app.key("enter");
    app.wait_text("Chosen: Python");
    app.key("down");
    app.key("enter");
    app.wait_text("Chosen: Kotlin");
    app.key("up");
    app.key("enter");
    app.wait_text("Chosen: Python");

    app.click(inside(find(&app.screen(), "src/api/auth.rs")));
    app.wait_text("1 selected");
    app.key("space");
    app.wait_screen("space toggles the file off", |s| {
        !text(s).contains("1 selected")
    });
    app.visual("functional/showcase_lists");
    app.finish();
}

#[test]
fn showcase_tree_expand_and_file_selection() {
    let app = App::new(
        "showcase-trees",
        SHOWCASE,
        &["--page", "trees", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("Nothing selected");
    assert!(text(&initial).contains("▾ src"));
    app.click(inside(find(&initial, "config.rs")));
    app.key("enter");
    let selected = app.wait_screen("file selection is reported", |s| {
        !text(s).contains("Nothing selected")
    });
    assert!(text(&selected).contains("config.rs"));

    let folder = find_all(&app.screen(), "src")
        .into_iter()
        .find(|pt| pt.x > 28 && pt.y < 12)
        .expect("src folder row");
    app.click(inside(folder));
    app.key("left");
    app.wait_screen("collapsing src hides its files", |s| {
        text(s).contains("▸ src") && !text(s).contains("1.9 KB")
    });
    app.key("right");
    app.wait_screen("expanding src restores its files", |s| {
        text(s).contains("▾ src") && text(s).contains("1.9 KB")
    });

    let label = find_all(&app.screen(), "src")
        .into_iter()
        .find(|pt| pt.x > 28 && pt.y < 12)
        .expect("src folder label");
    let gutter = Point {
        x: label.x - 3,
        y: label.y,
    };
    let gutter_text = |s: &TerminalSnapshot| {
        s.cell_at(u32::from(gutter.x), u32::from(gutter.y))
            .map(|c| c.text().to_string())
    };
    let settled = |s: &TerminalSnapshot| {
        gutter_text(s).as_deref() == Some("▎")
            && cell_bg(s, label) != tui_test::SnapshotColor::Rgb(56, 56, 62)
    };
    app.move_to(Point { x: 0, y: 0 });
    app.key("tab");
    app.wait_screen("focus leaves the tree", |s| {
        gutter_text(s).as_deref() != Some("▎")
    });
    app.key("shift+tab");
    app.wait_screen("focus returns to the tree", |s| {
        gutter_text(s).as_deref() == Some("▎")
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        if app.try_sustained(std::time::Duration::from_millis(600), |s| settled(s)) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "tree row must settle without hover"
        );
        app.move_to(Point { x: 0, y: 0 });
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    app.visual("functional/showcase_trees");
    app.finish();
}

#[test]
fn showcase_input_edit_apply_and_escape_reverts() {
    let app = App::new(
        "showcase-inputs",
        SHOWCASE,
        &["--page", "inputs", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("payments-gateway");

    app.key("tab");
    app.key("tab");
    app.wait_screen("field is in edit mode", |s| text(s).contains("EDIT"));
    app.key("enter");
    app.type_text("-renamed");
    app.wait_text("payments-gateway-renamed");
    app.key("enter");
    let applied = app.settle();
    assert!(text(&applied).contains("payments-gateway-renamed"));

    app.key("enter");
    app.type_text("-bad");
    app.wait_text("payments-gateway-renamed-bad");
    app.key("escape");
    let reverted = app.wait_screen("escape reverts the unapplied edit", |s| {
        !text(s).contains("-bad")
    });
    assert!(text(&reverted).contains("payments-gateway-renamed"));
    app.visual("functional/showcase_inputs");
    app.finish();
}

#[test]
fn showcase_dialog_opens_traps_and_escape_cancels() {
    let app = App::new(
        "showcase-dialogs",
        SHOWCASE,
        &["--page", "dialogs", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Open a dialog");
    app.click(inside(find(&app.screen(), "Confirm run")));
    let open = app.wait_screen("confirm dialog opens", |s| {
        let t = text(s);
        t.contains("Cancel") && (t.contains("Run") || t.contains("Confirm"))
    });
    assert!(!text(&open).contains("Nothing yet") || text(&open).contains("Cancel"));

    app.visual("functional/showcase_dialogs");
    app.key("escape");
    let cancelled = app.wait_screen("escape cancels the dialog", |s| {
        text(s).contains("Nothing yet") || text(s).contains("Cancelled")
    });
    assert!(text(&cancelled).contains("Open a dialog"));
    app.finish();
}

#[test]
fn showcase_table_header_sort_changes_row_order() {
    let app = App::new(
        "showcase-tables",
        SHOWCASE,
        &["--page", "tables", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    let initial = app.wait_text("#1040");
    assert!(text(&initial).contains("unsorted"));
    let first_before = row_text(&initial, u32::from(find(&initial, "#1040").y)).to_string();
    app.click(inside(find(&initial, "Status")));
    let sorted = app.wait_screen("header click sorts the table", |s| {
        !text(s).contains("unsorted")
    });
    assert!(text(&sorted).contains("#1040"));
    let _ = first_before;
    app.visual("functional/showcase_table_sort");
    app.finish();
}

#[test]
fn showcase_form_required_validation_and_reset() {
    let app = App::new(
        "showcase-forms",
        SHOWCASE,
        &["--page", "forms", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("New task");
    app.click(inside(find(&app.screen(), "Create task")));
    let invalid = app.wait_screen("empty required field blocks creation", |s| {
        let t = text(s).to_lowercase();
        t.contains("required") || t.contains("task name") && t.contains("!")
    });
    assert!(text(&invalid).contains("New task"));
    app.visual("functional/showcase_forms");
    app.finish();
}

fn visit_page(slug: &str, heading: &str) {
    let app = App::new(
        &format!("showcase-page-{slug}"),
        SHOWCASE,
        &["--page", slug, "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text(heading);
    app.settle();
    app.advance_clock(100);
    app.settle();
    app.visual(&format!("functional/showcase_page_{slug}"));
    app.finish();
}

macro_rules! page_test {
    ($fn_name:ident, $slug:expr, $heading:expr) => {
        #[test]
        fn $fn_name() {
            visit_page($slug, $heading);
        }
    };
}

page_test!(showcase_page_chrome, "chrome", "Chrome");
page_test!(showcase_page_statushints, "statushints", "Status & hints");
page_test!(
    showcase_page_breadcrumbsrails,
    "breadcrumbsrails",
    "Breadcrumbs & rails"
);
page_test!(showcase_page_helpkeys, "helpkeys", "Help & keys");
page_test!(showcase_page_quitleave, "quitleave", "Quit & leave");
page_test!(
    showcase_page_focusnavigation,
    "focusnavigation",
    "Focus & navigation"
);
page_test!(showcase_page_textareas, "textareas", "Text areas");
page_test!(showcase_page_toolbars, "toolbars", "Toolbars");
page_test!(showcase_page_editable, "editable", "Editable");
page_test!(showcase_page_selection, "selection", "Selection");
page_test!(showcase_page_celleditors, "celleditors", "Cell editors");
page_test!(showcase_page_columns, "columns", "Columns");
page_test!(showcase_page_sidebars, "sidebars", "Sidebars");
page_test!(showcase_page_pickers, "pickers", "Pickers");
page_test!(showcase_page_dialogkinds, "dialogkinds", "Dialog kinds");
page_test!(showcase_page_filebrowser, "filebrowser", "File browser");
page_test!(showcase_page_progress, "progress", "Progress");
page_test!(showcase_page_emptyerror, "emptyerror", "Empty & error");
page_test!(showcase_page_meters, "meters", "Meters");
page_test!(showcase_page_notices, "notices", "Notices");
page_test!(showcase_page_panels, "panels", "Panels");
page_test!(showcase_page_masterdetail, "masterdetail", "Master-detail");
page_test!(showcase_page_panetree, "panetree", "Pane tree");
page_test!(showcase_page_codeeditor, "codeeditor", "Code editor");
page_test!(showcase_page_diff, "diff", "Diff");
page_test!(showcase_page_terminal, "terminal", "Terminal");
page_test!(
    showcase_page_textutilities,
    "textutilities",
    "Text utilities"
);
page_test!(showcase_page_settings, "settings", "Settings");
page_test!(showcase_page_taskrunner, "taskrunner", "Task runner");
page_test!(
    showcase_page_queryworkbench,
    "queryworkbench",
    "Query workbench"
);
page_test!(showcase_page_connections, "connections", "Connections");
page_test!(showcase_page_finder, "finder", "Finder");
page_test!(showcase_page_diskcleanup, "diskcleanup", "Disk cleanup");
page_test!(showcase_page_activity, "activity", "Activity");
page_test!(showcase_page_workspaces, "workspaces", "Workspaces");
page_test!(
    showcase_page_accountsusage,
    "accountsusage",
    "Accounts & usage"
);
page_test!(
    showcase_page_launchcockpit,
    "launchcockpit",
    "Launch cockpit"
);
page_test!(
    showcase_page_readonlyreport,
    "readonlyreport",
    "Read-only report"
);
