use tablepro_ui::{Surface, TableProApp};
use termrock::{Color, ColorLevel, Theme};
use termrock_test_support::Harness;

#[test]
fn connections_header_rule_and_focus_bar_match_the_tag() {
    let harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let shows = |x, y, symbol: &str| {
        harness
            .buffer()
            .cell((x, y))
            .is_some_and(|cell| cell.symbol() == symbol)
    };
    let ink = |x, y, color: Color| {
        harness
            .buffer()
            .cell((x, y))
            .is_some_and(|cell| cell.fg == color)
    };
    assert!(shows(1, 0, " "), "brand keeps the chrome edge");
    assert!(shows(2, 0, "▪"));
    assert!(shows(28, 0, "6"));
    assert!(ink(28, 0, Color::from_u32(0x008a_8a8a)));
    assert!(shows(92, 0, "t"));
    assert!(ink(92, 0, Color::from_u32(0x008a_8a8a)));
    assert!(shows(110, 0, " "));
    assert!(shows(112, 0, "?"));
    assert!(shows(44, 3, "─"));
    assert!(shows(109, 3, "─"));
    assert!(shows(110, 3, " "));
    assert!(ink(44, 3, Color::from_u32(0x003a_3a3a)));
    assert!(
        harness
            .buffer()
            .cell((44, 3))
            .is_some_and(|cell| cell.bg == Color::from_u32(0x0011_1111))
    );
    assert!(shows(3, 4, " "), "filter row has no focus bar");
    assert!(shows(3, 5, "▎"), "focus bar stays on the tree row");
}

#[test]
fn all_connection_action_labels_keep_distinct_foreground_and_background() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for surface in [Surface::Connections, Surface::ConnectionsFailed] {
                let mut app = TableProApp::default();
                app.set_surface(surface);
                let harness = Harness::new(app, theme.clone(), 120, 40).with_color(level);
                for (start, label) in [
                    (45, "Connect"),
                    (56, "Edit"),
                    (64, "Duplicate"),
                    (77, "Delete…"),
                ] {
                    for (x, glyph) in (start..).zip(label.chars()) {
                        let cell = harness.buffer().cell((x, 17));
                        assert!(
                            cell.is_some_and(
                                |cell| cell.symbol() == glyph.to_string() && cell.fg != cell.bg
                            ),
                            "{surface:?}/{level:?}: {label} cell {x} must remain readable"
                        );
                    }
                }
                assert_eq!(harness.diagnostics(), []);
            }
        }
    }
}
