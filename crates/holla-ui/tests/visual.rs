use holla_ui::{App, Motion, Scenario};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Modifier},
};
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

const LEGACY_BORDER_CASES: [Scenario; 4] = [
    Scenario::RustDirty,
    Scenario::MonorepoRoot,
    Scenario::MonorepoChild,
    Scenario::ActivitiesMulti,
];

fn is_legacy_border_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    LEGACY_BORDER_CASES.contains(&scenario)
        && (width, height) == (80, 24)
        && *theme == Theme::junie()
        && color == ColorLevel::Ansi256
}

fn holla_border_positions(scenario: Scenario) -> &'static [(u16, u16)] {
    match scenario {
        Scenario::RustDirty | Scenario::MonorepoChild => &[(78, 17), (78, 18), (78, 19), (78, 20)],
        Scenario::MonorepoRoot | Scenario::ActivitiesMulti => &[(78, 18), (78, 19), (78, 20)],
        _ => panic!("scenario is outside the historical Holla border replay"),
    }
}

fn expected_cell(symbol: &str, fg: Color, bg: Color, modifier: Modifier) -> Cell {
    let mut cell = Cell::default();
    cell.set_symbol(symbol);
    cell.fg = fg;
    cell.bg = bg;
    cell.underline_color = Color::Reset;
    cell.modifier = modifier;
    cell
}

fn assert_only_expected_cells_differ(
    current: &Buffer,
    historical: &Buffer,
    expected_positions: &[(u16, u16)],
) {
    assert_eq!(current.area, historical.area);
    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = current.cell((x, y)).expect("position inside current frame");
            let historical_cell = historical
                .cell((x, y))
                .expect("position inside historical frame");
            if expected_positions.contains(&(x, y)) {
                assert_ne!(
                    current_cell, historical_cell,
                    "unexpected equality at ({x}, {y})"
                );
            } else {
                assert_eq!(
                    current_cell, historical_cell,
                    "unexpected difference at ({x}, {y})"
                );
            }
        }
    }
}

fn assert_historical_border_replay(scenario: Scenario) {
    let mut harness = harness(scenario, Theme::junie(), 80, 24, ColorLevel::Ansi256);
    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();

    let mut historical_theme = harness.runtime().theme().clone();
    historical_theme.color.border_subtle = Color::Indexed(235);
    harness.runtime_mut().set_theme(historical_theme);
    harness.draw();

    let historical = harness.buffer();
    let expected_positions = holla_border_positions(scenario);
    assert_eq!(current.area, Rect::new(0, 0, 80, 24));
    assert_eq!(historical.area, current.area);
    assert_eq!(current_cursor, Some(Position { x: 3, y: 4 }));
    assert_eq!(harness.cursor(), current_cursor);
    assert_only_expected_cells_differ(&current, historical, expected_positions);

    let current_cell = expected_cell(
        "┆",
        Color::Indexed(238),
        Color::Indexed(16),
        Modifier::empty(),
    );
    let historical_cell = expected_cell(
        "┆",
        Color::Indexed(235),
        Color::Indexed(16),
        Modifier::empty(),
    );
    for &(x, y) in expected_positions {
        assert_eq!(
            current.cell((x, y)).expect("historical cell inside frame"),
            &current_cell,
            "current Holla cell at ({x}, {y})"
        );
        assert_eq!(
            historical
                .cell((x, y))
                .expect("historical cell inside frame"),
            &historical_cell,
            "historical Holla cell at ({x}, {y})"
        );
    }

    harness
        .snapshot()
        .named(scenario.name())
        .assert_against(&BASELINE);
}

#[test]
fn holla_visual_baseline() {
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    if is_legacy_border_case(scenario, width, height, &theme, color) {
                        assert_historical_border_replay(scenario);
                        continue;
                    }
                    harness(scenario, theme.clone(), width, height, color)
                        .snapshot()
                        .named(scenario.name())
                        .assert_against(&BASELINE);
                }
            }
        }
    }
}

#[test]
fn holla_historical_border_subtle_replay_matches_four_held_frames() {
    for scenario in LEGACY_BORDER_CASES {
        assert_historical_border_replay(scenario);
    }
}
