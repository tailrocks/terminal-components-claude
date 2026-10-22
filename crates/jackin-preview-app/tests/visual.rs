use jackin_preview_app::{App, Motion, Scenario};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    style::{Color, Modifier},
};
use termrock::{
    ColorLevel, Family, FgStep, GlyphRole, Part, Role, Slot, StylePatch, Surface, Theme, Variant,
};
use termrock_test_support::{Baseline, Harness};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/jackin.txt"
));

const SIZES: [(u16, u16); 2] = [(100, 30), (120, 40)];
const COLORS: [ColorLevel; 2] = [ColorLevel::TrueColor, ColorLevel::Mono];
const EXPECTED_POSITIONS: [(u16, u16); 6] = [(14, 3), (26, 3), (38, 3), (50, 3), (4, 24), (4, 25)];

fn is_historical_returning_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    scenario == Scenario::Returning
        && (width, height) == (100, 30)
        && *theme == Theme::junie()
        && color == ColorLevel::TrueColor
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

fn cell_at(buffer: &Buffer, x: u16, y: u16) -> &Cell {
    let cell = buffer.cell((x, y));
    assert!(
        cell.is_some(),
        "cell at ({x}, {y}) must be inside the frame"
    );
    cell.unwrap_or(&Cell::EMPTY)
}

fn assert_only_expected_cells_differ(
    current: &Buffer,
    historical: &Buffer,
    expected_positions: &[(u16, u16)],
) {
    assert_eq!(current.area, historical.area);
    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = cell_at(current, x, y);
            let historical_cell = cell_at(historical, x, y);
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

fn apply_historical_gutter_recipe(harness: &mut Harness<App>) {
    let mut theme = harness.runtime().theme().clone();
    theme.design.glyphs.set(GlyphRole::SwitchOff, " ");

    let button = theme.recipes.get_mut(Family::BUTTON);
    assert!(
        button.parts.get(Part::GUTTER).is_some(),
        "Junie default button gutter recipe"
    );
    let default_gutter = button.parts.entry(Part::GUTTER);
    default_gutter.glyph = Slot::Set(GlyphRole::SwitchOff);
    default_gutter.base = StylePatch::new()
        .set_fg(Role::Fg(FgStep::Primary))
        .set_bg(Role::Surface(Surface::Overlay));

    let primary_parts = button.variant_mut(Variant::PRIMARY);
    assert!(
        primary_parts.get(Part::GUTTER).is_some(),
        "Junie primary button gutter recipe"
    );
    let primary_gutter = primary_parts.entry(Part::GUTTER);
    primary_gutter.glyph = Slot::Set(GlyphRole::SwitchOff);
    primary_gutter.base = StylePatch::new()
        .set_fg(Role::OnAccent)
        .set_bg(Role::Accent)
        .add(Modifier::BOLD);

    harness.runtime_mut().set_theme(theme);
    harness.draw();
}

fn assert_historical_returning_replay() {
    let app = App::for_scenario(Scenario::Returning, Motion::Paused);
    let mut harness = Harness::new(app, Theme::junie(), 100, 30).with_color(ColorLevel::TrueColor);
    harness.draw();
    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();

    apply_historical_gutter_recipe(&mut harness);
    let historical = harness.buffer();
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(historical.area, current.area);
    assert_eq!(current_cursor, None);
    assert_eq!(harness.cursor(), current_cursor);
    assert_only_expected_cells_differ(&current, historical, &EXPECTED_POSITIONS);

    let current_nav = expected_cell(
        " ",
        Color::Rgb(39, 39, 42),
        Color::Rgb(39, 39, 42),
        Modifier::empty(),
    );
    let historical_nav = expected_cell(
        " ",
        Color::Rgb(255, 255, 255),
        Color::Rgb(39, 39, 42),
        Modifier::empty(),
    );
    for &(x, y) in &EXPECTED_POSITIONS[..4] {
        assert_eq!(
            cell_at(&current, x, y),
            &current_nav,
            "current Jackin nav cell at ({x}, {y})"
        );
        assert_eq!(
            cell_at(historical, x, y),
            &historical_nav,
            "historical Jackin nav cell at ({x}, {y})"
        );
    }

    let current_primary = expected_cell(
        " ",
        Color::Rgb(72, 224, 84),
        Color::Rgb(72, 224, 84),
        Modifier::empty(),
    );
    let historical_primary = expected_cell(
        " ",
        Color::Rgb(25, 25, 28),
        Color::Rgb(72, 224, 84),
        Modifier::BOLD,
    );
    for &(x, y) in &EXPECTED_POSITIONS[4..] {
        assert_eq!(
            cell_at(&current, x, y),
            &current_primary,
            "current Jackin primary cell at ({x}, {y})"
        );
        assert_eq!(
            cell_at(historical, x, y),
            &historical_primary,
            "historical Jackin primary cell at ({x}, {y})"
        );
    }

    harness
        .snapshot()
        .named(Scenario::Returning.name())
        .assert_against(&BASELINE);
}

#[test]
fn jackin_visual_baseline() {
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    if is_historical_returning_case(scenario, width, height, &theme, color) {
                        assert_historical_returning_replay();
                        continue;
                    }
                    let app = App::for_scenario(scenario, Motion::Paused);
                    let mut harness =
                        Harness::new(app, theme.clone(), width, height).with_color(color);
                    harness.draw();
                    harness
                        .snapshot()
                        .named(scenario.name())
                        .assert_against(&BASELINE);
                }
            }
        }
    }
}

#[test]
fn jackin_historical_returning_gutter_replay_matches_held_frame() {
    assert_historical_returning_replay();
}
