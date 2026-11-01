use super::support::{App, SHOWCASE, find, text};

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
    app.click(super::support::Point {
        x: find(&app.screen(), "Open popover ▾").x + 2,
        y: find(&app.screen(), "Open popover ▾").y,
    });
    let open = app.wait_screen("popover opens", |s| {
        text(s).contains("Esc on") || text(s).contains("Dismiss")
    });
    assert!(text(&open).contains("Open popover"));
    app.resize(80, 24);
    let narrow = app.wait_screen("popover still owned at 80x24", |s| {
        s.cols() == 80 && s.rows() == 24 && text(s).contains("Open popover")
    });
    assert_eq!((narrow.cols(), narrow.rows()), (80, 24));
    app.key("escape");
    app.wait_screen("escape dismisses the popover after resize", |s| {
        text(s).contains("Open popover ▾")
    });
    app.visual("functional/showcase_popover_resize");
    app.finish();
}
