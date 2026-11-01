use super::support::{App, Point, SHOWCASE, find, text};

#[test]
fn showcase_help_survives_shrink_and_growth_and_returns_focus() {
    let app = App::new(
        "showcase-lifecycle",
        SHOWCASE,
        &["--page", "overview", "--motion", "paused", "--frame", "40"],
        120,
        40,
        "truecolor",
    );
    let screen = app.wait_text("120×40");
    assert_eq!((screen.cols(), screen.rows()), (120, 40));
    app.key("f1");
    let open = app.wait_text("Keyboard & mouse");
    let original_title = find(&open, "Keyboard & mouse");
    app.resize(80, 24);
    let narrow = app.wait_screen("help reflows at 80×24", |s| {
        s.cols() == 80
            && s.rows() == 24
            && text(s).contains("80×24")
            && text(s).contains("Keyboard & mouse")
    });
    assert_ne!(find(&narrow, "Keyboard & mouse"), original_title);
    app.resize(160, 50);
    app.wait_screen("help reflows at 160×50", |s| {
        s.cols() == 160
            && s.rows() == 50
            && text(s).contains("160×50")
            && text(s).contains("Keyboard & mouse")
    });
    app.key("escape");
    app.wait_absent("Keyboard & mouse");
    app.key("ctrl+pagedown");
    app.wait_text("App shell › Chrome");
    app.key("ctrl+pageup");
    app.wait_text("Tokens and principles behind every component");
    assert!(!text(&app.screen()).contains("App shell › Chrome"));
    app.visual("functional/showcase_help_resize");
    app.finish();
}

#[test]
fn showcase_popover_survives_resize_and_escape_returns_focus() {
    let app = App::new(
        "showcase-popover-resize",
        SHOWCASE,
        &["--page", "popups", "--motion", "paused"],
        120,
        40,
        "truecolor",
    );
    app.wait_text("Open popover ▾");
    let trigger = Point {
        x: find(&app.screen(), "Open popover ▾").x + 2,
        y: find(&app.screen(), "Open popover ▾").y,
    };
    app.click(trigger);
    // Wait for the popover's own content: the page prose also mentions
    // "Dismiss", and resizing mid-open leaves the trigger in its owner
    // state instead of the settled one.
    let open = app.wait_text("report.pdf");
    assert!(text(&open).contains("Open popover"));
    app.resize(80, 24);
    let narrow = app.wait_screen("popover still owned at 80x24", |s| {
        s.cols() == 80 && s.rows() == 24 && text(s).contains("Open popover")
    });
    assert_eq!((narrow.cols(), narrow.rows()), (80, 24));
    // Freeze with the popover still open: the page beneath a popup shows
    // no focus or hover, so this frame has no timing-sensitive state.
    // (Freezing after dismissal raced the trigger's hover background,
    // which depends on when the terminal delivers the parked motion.)
    // The page-neutral-under-popup restyle (selected tab loses its
    // accent) lands on a later frame than the popup itself; wait for the
    // tab underline to actually go neutral before freezing.
    let tab = find(&app.screen(), "Playground");
    app.wait_screen("selected tab neutral under popover", |s| {
        s.cell_at(u32::from(tab.x), u32::from(tab.y) + 1)
            .map(|c| c.fg())
            == Some(tui_test::SnapshotColor::Rgb(77, 77, 77))
    });
    // Let the neutral state age, then park the pointer off the trigger
    // with a single motion: repeated motions to the same cell are
    // deduped by the terminal, and a motion sent before the popover
    // finished re-anchoring is consumed before the hover lock engages.
    assert!(
        app.try_sustained(std::time::Duration::from_millis(600), |s| {
            s.cell_at(u32::from(tab.x), u32::from(tab.y) + 1)
                .map(|c| c.fg())
                == Some(tui_test::SnapshotColor::Rgb(77, 77, 77))
        }),
        "selected tab must go neutral under the popover"
    );
    app.move_to(super::support::Point { x: 79, y: 0 });
    let trigger = find(&app.screen(), "Open popover \u{25be}");
    app.wait_screen("trigger unhovered under popover", |s| {
        super::support::cell_bg(s, trigger) == tui_test::SnapshotColor::Rgb(39, 39, 42)
    });
    app.visual("functional/showcase_popover_resize");
    app.key("escape");
    app.wait_screen("escape dismisses the popover after resize", |s| {
        !text(s).contains("report.pdf") && text(s).contains("Open popover ▾")
    });
    app.finish();
}
