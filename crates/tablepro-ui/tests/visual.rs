use tablepro_ui::{Surface, TableProApp};
use termrock::{ColorLevel, Theme};
use termrock_test_support::Harness;

const QUERY_DURATION: std::time::Duration = std::time::Duration::from_millis(42);

const BASELINE: termrock_test_support::Baseline = termrock_test_support::Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/tablepro.txt"
));

#[test]
fn tablepro_visual_baseline() {
    for (width, height) in [(120, 40), (80, 24)] {
        for surface in Surface::ALL {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                    let mut app = TableProApp::new_with_query_duration(QUERY_DURATION);
                    app.set_surface(surface);
                    let harness = Harness::new(app, theme.clone(), width, height).with_color(color);

                    harness
                        .snapshot()
                        .named(surface.label())
                        .assert_against(&BASELINE);
                }
            }
        }
    }
}
