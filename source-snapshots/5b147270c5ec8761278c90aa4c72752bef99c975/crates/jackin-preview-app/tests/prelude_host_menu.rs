//! WI-JACKIN-ACCOUNTS-LABELS (fallback): prelude host-menu titles through `MenuBar`.
//!
//! The prelude branch of `draw_host_menu` renders its File/Go/Help titles
//! through the stock `MenuBar` over `MANAGER_MENUS` with a one-part dim
//! patch — the same bar the manager arm draws. This test pins that
//! dispatch:
//!
//! * the real paused journey (`n` + ticks to the prelude needle) matches
//!   the frozen prelude frame on all 40 rows, and the 16 title cells wear
//!   the frozen dim tone exactly (symbol, fg, bg, non-bold);
//! * the 80x24 journey keeps the frozen row 0;
//! * the bar is live, not a stored painter: tabbing onto it bolds the
//!   cursor title and raises the stock focus gutter, and tabbing away
//!   restores the dim titles;
//! * the manager arm is untouched: leaving the prelude returns to the
//!   stock bar, whose focused cursor title wears unpatched bold Primary
//!   — the dim patch is scoped to the prelude arm.
use jackin_preview_app::{APP, App, Motion, Scenario, screens::prelude::FILE_LIST};
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

/// A row-0 cell snapshot: symbol, fg, bg, bold.
fn snap(h: &Harness<App>, x: u16) -> (String, termrock::Color, termrock::Color, bool) {
    let c = h.cell(x, 0);
    (
        c.symbol().to_string(),
        c.fg,
        c.bg,
        c.modifier.contains(termrock::Modifier::BOLD),
    )
}

#[test]
fn paused_prelude_menu_renders_through_components() {
    use termrock::Color;

    const DIM: Color = Color::Rgb(77, 77, 77);
    const CANVAS: Color = Color::Rgb(0, 0, 0);
    const PRIMARY: Color = Color::Rgb(255, 255, 255);
    const SECONDARY: Color = Color::Rgb(179, 179, 179);
    const FOCUS: Color = Color::Rgb(72, 224, 84);
    const TITLES: [(u16, &str); 3] = [(12, " File "), (19, " Go "), (24, " Help ")];

    // Control: the stock manager titles before the journey.
    let app = App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400);
    let mut h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    for (x0, label) in TITLES {
        for (i, want) in label.chars().enumerate() {
            let x = x0 + i as u16;
            assert_eq!(snap(&h, x), (want.to_string(), SECONDARY, CANVAS, false));
        }
    }

    // Journey: `n` + ticks to the prelude needle; focus stays pinned off the bar.
    let _ = h.key(KeyCode::Char('n'));
    for _ in 0..60 {
        let _ = h.tick();
        if h.text().contains("step 1 of 5") {
            break;
        }
    }
    assert!(
        h.text().contains("step 1 of 5"),
        "journey must reach the prelude needle"
    );
    assert_eq!(h.focus(), Some(FILE_LIST));

    // Full-frame parity: all 40 rows match the frozen prelude frame.
    let expected =
        frozen("baselines/tuiscotti-v1/jackin/manager/new-workspace-prelude/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen frame has 40 rows");
    for y in 0..40u16 {
        assert_eq!(
            h.row(y).trim_end(),
            expected[usize::from(y)],
            "paused prelude row {y} must match frozen"
        );
    }

    // The 16 title cells wear the frozen dim tone exactly.
    for (x0, label) in TITLES {
        for (i, want) in label.chars().enumerate() {
            let x = x0 + i as u16;
            assert_eq!(snap(&h, x), (want.to_string(), DIM, CANVAS, false));
        }
    }

    // Ownership mutation: focusing the bar bolds the cursor title and
    // raises the stock focus gutter — impossible on a stored painter.
    assert!(h.tab_to(APP.sub("manager-menu-bar")));
    assert_eq!(snap(&h, 11), ("▎".to_string(), FOCUS, CANVAS, false));
    for (i, want) in " File ".chars().enumerate() {
        let x = 12 + i as u16;
        assert_eq!(snap(&h, x), (want.to_string(), DIM, CANVAS, true));
    }
    for (x0, label) in [(19u16, " Go "), (24u16, " Help ")] {
        for (i, want) in label.chars().enumerate() {
            let x = x0 + i as u16;
            assert_eq!(snap(&h, x), (want.to_string(), DIM, CANVAS, false));
        }
    }

    // Tab away: the dim titles are restored exactly.
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(FILE_LIST));
    assert_eq!(snap(&h, 11).0, " ");
    for (x0, label) in TITLES {
        for (i, want) in label.chars().enumerate() {
            let x = x0 + i as u16;
            assert_eq!(snap(&h, x), (want.to_string(), DIM, CANVAS, false));
        }
    }

    // Isolation: leaving the prelude returns to the stock manager bar.
    // Reconcile focus lands on the bar (Tab is the detail-drawer chord
    // in the manager, so focus cannot Tab away); the focused bar wears
    // the unpatched stock tones — bold Primary cursor title — proving
    // the dim patch is scoped to the prelude arm.
    let _ = h.key(KeyCode::Esc);
    assert!(
        !h.text().contains("step 1 of 5"),
        "Esc must leave the prelude"
    );
    assert_eq!(h.focus(), Some(APP.sub("manager-menu-bar")));
    assert_eq!(snap(&h, 11), ("▎".to_string(), FOCUS, CANVAS, false));
    for (i, want) in " File ".chars().enumerate() {
        let x = 12 + i as u16;
        assert_eq!(snap(&h, x), (want.to_string(), PRIMARY, CANVAS, true));
    }
    for (x0, label) in [(19u16, " Go "), (24u16, " Help ")] {
        for (i, want) in label.chars().enumerate() {
            let x = x0 + i as u16;
            assert_eq!(snap(&h, x), (want.to_string(), SECONDARY, CANVAS, false));
        }
    }

    // The 80x24 journey keeps the frozen row 0.
    let app = App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400);
    let mut h = Harness::new(app, termrock::Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Char('n'));
    for _ in 0..60 {
        let _ = h.tick();
        if h.text().contains("step 1 of 5") {
            break;
        }
    }
    assert!(
        h.text().contains("step 1 of 5"),
        "80x24 journey must reach the prelude needle"
    );
    let expected =
        frozen("baselines/tuiscotti-v1/jackin/manager/new-workspace-prelude/80x24/truecolor.txt");
    assert_eq!(
        h.row(0).trim_end(),
        expected[0],
        "80x24 prelude row 0 must match frozen"
    );
}
