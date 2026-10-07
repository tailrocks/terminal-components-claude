//! WI-JACKIN-FOOTER-2: menu-open footer through stock `HintBar` with zero slots.
//!
//! The menu-open arm of `draw_footer` draws
//! `HintBar::new(APP.sub("hint"), &self.hint_layers.manager_menu)` with no
//! slot overrides — stock KEY chips, stock ACTION labels, stock CONTAINER
//! fill. This test pins that dispatch:
//!
//! * the real paused journey (`F10` + ticks to the "New workspace"
//!   needle) matches the frozen menu-open frame on footer row 39, and
//!   every hint-span cell x=38–86 wears its frozen tone exactly
//!   (symbol, fg, bg, bold: 13 KEY-chip bolds, uniform muted labels,
//!   flat fill);
//! * the 80x24 journey keeps the frozen footer row with the re-centered
//!   span start (x=18, not x=38) — impossible on a stored framebuffer;
//! * the bar is live, not a stored painter: the same Harness swaps from
//!   the manager footer to the menu-open footer across `F10`;
//! * the manager arm is untouched: a fresh no-`F10` Harness still shows
//!   the frozen manager footer.
//!
//! NOTE: only the footer row is asserted (row 3 carries a pre-existing
//! body-driven dropdown-gutter diff), and no Esc/Enter close is used
//! (menu interaction is suppressed under paused motion).
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
fn paused_menu_open_footer_renders_through_components() {
    use termrock::Color;

    const WHITE: Color = Color::Rgb(255, 255, 255);
    const MUTED: Color = Color::Rgb(128, 128, 128);
    const CANVAS: Color = Color::Rgb(0, 0, 0);
    const KEYS: [(u16, &str); 4] = [(38, "← →"), (48, "↑↓"), (57, "Enter"), (71, "Esc")];
    const ACTIONS: [(u16, &str); 4] = [(42, "Menu"), (51, "Move"), (63, "Choose"), (75, "Close")];

    // Control: the stock manager footer before the journey.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let mut h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let returning =
        frozen("baselines/tuiscotti-v1/jackin/manager/scenario-returning/120x40/truecolor.txt");
    assert_eq!(returning.len(), 40, "frozen returning frame has 40 rows");
    let manager_footer = h.row(39).trim_end().to_string();
    assert_eq!(
        manager_footer, returning[39],
        "pre-F10 manager footer must match frozen"
    );

    // Journey: `F10` + ticks to the menu-open needle.
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

    // Live-ness: the same Harness swaps footer arms across `F10`.
    let menu_footer = h.row(39).trim_end().to_string();
    assert_ne!(
        menu_footer, manager_footer,
        "F10 must swap the manager footer for the menu-open footer"
    );
    let expected = frozen("baselines/tuiscotti-v1/jackin/manager/menu-open/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen menu-open frame has 40 rows");
    assert_eq!(
        menu_footer, expected[39],
        "menu-open footer row must match frozen"
    );

    // Every hint-span cell x=38–86 wears its frozen tone exactly.
    let mut span = vec![(" ".to_string(), WHITE, false); 49];
    for (x0, label) in KEYS {
        for (i, want) in label.chars().enumerate() {
            span[usize::from(x0 - 38) + i] = (want.to_string(), WHITE, true);
        }
    }
    for (x0, label) in ACTIONS {
        for (i, want) in label.chars().enumerate() {
            span[usize::from(x0 - 38) + i] = (want.to_string(), MUTED, false);
        }
    }
    for (i, (want, fg, bold)) in span.iter().enumerate() {
        let x = 38 + i as u16;
        assert_eq!(
            snap(&h, x, 39),
            (want.clone(), *fg, CANVAS, *bold),
            "menu-open span cell x={x} must match frozen"
        );
    }

    // Exactly the 13 KEY-chip cells are bold; fill is flat elsewhere.
    let mut bolds = 0;
    for x in 0..120u16 {
        if snap(&h, x, 39).3 {
            bolds += 1;
        }
    }
    assert_eq!(bolds, 13, "exactly the 13 KEY-chip cells are bold");
    for x in [0u16, 1, 37, 87, 100, 119] {
        assert_eq!(
            snap(&h, x, 39),
            (" ".to_string(), WHITE, CANVAS, false),
            "fill cell x={x} must stay flat"
        );
    }

    // The 80x24 journey keeps the frozen footer row, re-centered to x=18.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let mut h = Harness::new(app, termrock::Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::F(10));
    for _ in 0..30 {
        let _ = h.tick();
        if h.text().contains("New workspace") {
            break;
        }
    }
    assert!(
        h.text().contains("New workspace"),
        "80x24 journey must reach the menu-open needle"
    );
    let expected = frozen("baselines/tuiscotti-v1/jackin/manager/menu-open/80x24/truecolor.txt");
    assert_eq!(expected.len(), 24, "frozen 80x24 frame has 24 rows");
    assert_eq!(
        h.row(23).trim_end(),
        expected[23],
        "80x24 menu-open footer must match frozen"
    );
    assert_eq!(snap(&h, 18, 23).0, "←", "80x24 span must start at x=18");
    assert_eq!(
        snap(&h, 17, 23),
        (" ".to_string(), WHITE, CANVAS, false),
        "80x24 fill must stay flat before the span"
    );

    // Isolation: a fresh no-`F10` Harness keeps the manager footer.
    let app = App::for_scenario_at(Scenario::Returning, Motion::Paused, 0);
    let h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    assert_eq!(
        h.row(39).trim_end(),
        returning[39],
        "untouched manager arm keeps its footer"
    );
}
