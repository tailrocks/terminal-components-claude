//! Connection actions retain readable semantic pairs at every color level.
use junie_tui::{ColorLevel, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, TableProApp};

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
                assert!(harness.diagnostics().is_empty());
            }
        }
    }
}
