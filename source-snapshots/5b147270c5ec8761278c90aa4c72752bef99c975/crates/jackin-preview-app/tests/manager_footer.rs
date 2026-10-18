//! WI-JACKIN-FOOTER-3: manager footer through stock `HintBar` with zero slots.
//!
//! The Manager arm of `draw_footer` draws
//! `HintBar::new(APP.sub("hint"), &hints).status_text(...)` with no slot
//! overrides — stock KEY chips, stock ACTION labels, stock CONTAINER fill.
//! This test pins that dispatch:
//!
//! * the real paused journey (`App::for_scenario_at(Returning, Paused,
//!   0)`, no keys) matches the frozen returning frame on footer row 39,
//!   and every hint-span cell x=14–102 wears its frozen tone exactly
//!   (symbol, fg, bg, bold: 15 KEY-chip bolds, uniform muted labels,
//!   flat fill, no status leak at the right edge);
//! * the bar is live, not a stored painter: the same Harness swaps from
//!   the directory footer to the workspace-variant footer across `down`,
//!   matching the frozen tree-expanded footer row;
//! * the 100x30 journey keeps the frozen footer row with the re-centered
//!   span start (x=4, not x=14) — impossible on a stored framebuffer;
//! * the menu-open arm is untouched: a fresh `F10` Harness still shows
//!   the frozen menu-open footer.
//!
//! NOTE: only the footer row is asserted (the body is other slices'
//! scope), no 80x24/72x20-vs-frozen asserts (pre-existing 1-col stock
//! centering shift, documented in the slice plan, must not be frozen as
//! correct), and no status-setting keys (none keep the Manager footer).
use jackin_preview_app::{App, Motion, Scenario};
use termrock::KeyCode;
use termrock_test_support::Harness;

fn frozen(path: &str) -> Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(path);
    std::fs::read_to_string(&root)
        .unwrap()
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect()
}

/// A footer-row cell snapshot: symbol, fg, bg, bold.
fn snap(h: &Harness<App>, x: u16, y: u16) -> (String, termrock::Color, termrock::Color, bool) {
    let c = h.cell(x, y);
    (
        c.symbol().to_string(),
        c.fg,
        c.bg,
        c.modifier.contains(termrock::Modifier::BOLD),
    )
}

#[test]
fn paused_manager_footer_renders_through_components() {
    use termrock::Color;

    const WHITE: Color = Color::Rgb(255, 255, 255);
    const MUTED: Color = Color::Rgb(128, 128, 128);
    const CANVAS: Color = Color::Rgb(0, 0, 0);
    const KEYS: [(u16, &str); 9] = [
        (14, "Enter"),
        (28, "n"),
        (35, "e"),
        (43, "Tab"),
        (56, "c"),
        (68, "u"),
        (77, "s"),
        (89, "?"),
        (97, "q"),
    ];
    const ACTIONS: [(u16, &str); 9] = [
        (20, "Launch"),
        (30, "New"),
        (37, "Edit"),
        (47, "Details"),
        (58, "Accounts"),
        (70, "Usage"),
        (79, "Settings"),
        (91, "Help"),
        (99, "Quit"),
    ];

    // Control: the real paused journey, no keys.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let mut h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let returning =
        frozen("baselines/tuiscotti-v1/jackin/manager/scenario-returning/120x40/truecolor.txt");
    assert_eq!(returning.len(), 40, "frozen returning frame has 40 rows");
    let manager_footer = h.row(39).trim_end().to_string();
    assert_eq!(
        manager_footer, returning[39],
        "manager footer row must match frozen"
    );

    // Every hint-span cell x=14–102 wears its frozen tone exactly.
    let mut span = vec![(" ".to_string(), WHITE, false); 89];
    for (x0, label) in KEYS {
        for (i, want) in label.chars().enumerate() {
            span[usize::from(x0 - 14) + i] = (want.to_string(), WHITE, true);
        }
    }
    for (x0, label) in ACTIONS {
        for (i, want) in label.chars().enumerate() {
            span[usize::from(x0 - 14) + i] = (want.to_string(), MUTED, false);
        }
    }
    for (i, (want, fg, bold)) in span.iter().enumerate() {
        let x = 14 + i as u16;
        assert_eq!(
            snap(&h, x, 39),
            (want.clone(), *fg, CANVAS, *bold),
            "manager span cell x={x} must match frozen"
        );
    }

    // Exactly the 15 KEY-chip cells are bold; fill is flat elsewhere.
    let mut bolds = 0;
    for x in 0..120u16 {
        if snap(&h, x, 39).3 {
            bolds += 1;
        }
    }
    assert_eq!(bolds, 15, "exactly the 15 KEY-chip cells are bold");
    for x in [0u16, 1, 13, 103, 110, 119] {
        assert_eq!(
            snap(&h, x, 39),
            (" ".to_string(), WHITE, CANVAS, false),
            "fill cell x={x} must stay flat"
        );
    }

    // Live-ness: the same Harness swaps to the workspace-variant footer
    // across `down` (fails on a stored framebuffer).
    let _ = h.key(KeyCode::Down);
    for _ in 0..30 {
        let _ = h.tick();
        if h.row(39).trim_end() != manager_footer {
            break;
        }
    }
    let workspace_footer = h.row(39).trim_end().to_string();
    assert_ne!(
        workspace_footer, manager_footer,
        "down must swap the directory footer for the workspace footer"
    );
    let expanded =
        frozen("baselines/tuiscotti-v1/jackin/manager/tree-expanded/120x40/truecolor.txt");
    assert_eq!(expanded.len(), 40, "frozen tree-expanded frame has 40 rows");
    assert_eq!(
        workspace_footer, expanded[39],
        "workspace footer row must match frozen"
    );

    // The 100x30 journey keeps the frozen footer row, re-centered to x=4.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let h = Harness::new(app, termrock::Theme::junie(), 100, 30);
    let returning =
        frozen("baselines/tuiscotti-v1/jackin/manager/scenario-returning/100x30/truecolor.txt");
    assert_eq!(returning.len(), 30, "frozen 100x30 frame has 30 rows");
    assert_eq!(
        h.row(29).trim_end(),
        returning[29],
        "100x30 manager footer must match frozen"
    );
    assert_eq!(snap(&h, 4, 29).0, "E", "100x30 span must start at x=4");
    assert_eq!(
        snap(&h, 3, 29),
        (" ".to_string(), WHITE, CANVAS, false),
        "100x30 fill must stay flat before the span"
    );

    // Isolation: a fresh `F10` Harness keeps the menu-open footer.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let mut h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::F(10));
    for _ in 0..30 {
        let _ = h.tick();
        if h.text().contains("New workspace") {
            break;
        }
    }
    assert!(
        h.text().contains("New workspace"),
        "journey must reach the menu-open needle"
    );
    let expected = frozen("baselines/tuiscotti-v1/jackin/manager/menu-open/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen menu-open frame has 40 rows");
    assert_eq!(
        h.row(39).trim_end(),
        expected[39],
        "untouched menu-open arm keeps its footer"
    );
}
