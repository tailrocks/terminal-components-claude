//! `TablePro`'s 21-surface × 2-size deterministic visual matrix.

use junie_tui::{ColorLevel, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, TableProApp};

const BASELINE: junie_tui_testing::Baseline = junie_tui_testing::Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/tablepro.txt"
));

#[test]
fn tablepro_visual_baseline() {
    for (width, height) in [(120, 40), (80, 24)] {
        for surface in Surface::ALL {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                    let mut app = TableProApp::default();
                    app.set_surface(surface);
                    let harness = Harness::new(app, theme.clone(), width, height).with_color(color);
                    let scene = harness.snapshot().named(surface.label());
                    assert!(
                        scene.text().contains(surface.label()),
                        "{width}x{height} {} is missing its label",
                        surface.label()
                    );
                    scene.assert_against(&BASELINE);
                }
            }
        }
    }
}
