use holla_ui::{App, Motion, Scenario};
use termrock::{ColorLevel, FeedbackClock, SimulationMoment, Theme};
use termrock_test_support::{Baseline, Harness};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/app.txt"
));

const SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];
const COLORS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn harness(
    scenario: Scenario,
    theme: Theme,
    width: u16,
    height: u16,
    color: ColorLevel,
) -> Harness<App> {
    let app = App::for_scenario_without_history(scenario, Motion::Paused, 4_000);
    let initial = SimulationMoment::from_millis(app.fixture_time_ms() as u64);
    Harness::new_with_feedback_clock(
        app,
        theme,
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
    .with_color(color)
}

#[test]
fn holla_visual_baseline() {
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    harness(scenario, theme.clone(), width, height, color)
                        .snapshot()
                        .named(scenario.name())
                        .assert_against(&BASELINE);
                }
            }
        }
    }
}
