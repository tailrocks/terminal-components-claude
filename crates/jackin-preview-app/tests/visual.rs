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

#[path = "visual/paper_replay.rs"]
mod paper_replay;

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

fn is_historical_returning_mono_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    scenario == Scenario::Returning
        && (width, height) == (100, 30)
        && *theme == Theme::junie()
        && color == ColorLevel::Mono
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

const CURRENT_MONO_NAV_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Black, bg: Black, underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const HISTORICAL_MONO_NAV_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: White, bg: Black, underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const CURRENT_MONO_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Gray, bg: Gray, underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const HISTORICAL_MONO_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Black, bg: Gray, underline_color: Reset, modifier: BOLD, diff_option: None, skip: false }"#;

fn assert_complete_cell_tuple(
    buffer: &Buffer,
    position: (u16, u16),
    fg: Color,
    bg: Color,
    modifier: Modifier,
    native_debug: &str,
    label: &str,
) {
    let (x, y) = position;
    let actual = cell_at(buffer, x, y);
    let expected = expected_cell(" ", fg, bg, modifier);
    assert_eq!(actual, &expected, "{label} complete cell at ({x}, {y})");
    assert_eq!(
        format!("{actual:?}"),
        native_debug,
        "{label} native Cell Debug at ({x}, {y})"
    );
}

fn assert_current_returning_mono_tuples(buffer: &Buffer) {
    for &(x, y) in &EXPECTED_POSITIONS[..4] {
        assert_complete_cell_tuple(
            buffer,
            (x, y),
            Color::Black,
            Color::Black,
            Modifier::empty(),
            CURRENT_MONO_NAV_DEBUG,
            "current Mono navigation gutter",
        );
    }
    for &(x, y) in &EXPECTED_POSITIONS[4..] {
        assert_complete_cell_tuple(
            buffer,
            (x, y),
            Color::Gray,
            Color::Gray,
            Modifier::empty(),
            CURRENT_MONO_PRIMARY_DEBUG,
            "current Mono primary gutter",
        );
    }
}

fn assert_returning_mono_pair(current: &Buffer, historical: &Buffer) {
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(historical.area, current.area);

    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = cell_at(current, x, y);
            let historical_cell = cell_at(historical, x, y);
            let position = (x, y);

            if EXPECTED_POSITIONS[..4].contains(&position) {
                assert_ne!(
                    current_cell, historical_cell,
                    "Mono navigation gutter must be a measured change at ({x}, {y})"
                );
                assert_complete_cell_tuple(
                    current,
                    position,
                    Color::Black,
                    Color::Black,
                    Modifier::empty(),
                    CURRENT_MONO_NAV_DEBUG,
                    "current Mono navigation gutter",
                );
                assert_complete_cell_tuple(
                    historical,
                    position,
                    Color::White,
                    Color::Black,
                    Modifier::empty(),
                    HISTORICAL_MONO_NAV_DEBUG,
                    "historical Mono navigation gutter",
                );
            } else if EXPECTED_POSITIONS[4..].contains(&position) {
                assert_ne!(
                    current_cell, historical_cell,
                    "Mono primary gutter must be a measured change at ({x}, {y})"
                );
                assert_complete_cell_tuple(
                    current,
                    position,
                    Color::Gray,
                    Color::Gray,
                    Modifier::empty(),
                    CURRENT_MONO_PRIMARY_DEBUG,
                    "current Mono primary gutter",
                );
                assert_complete_cell_tuple(
                    historical,
                    position,
                    Color::Black,
                    Color::Gray,
                    Modifier::BOLD,
                    HISTORICAL_MONO_PRIMARY_DEBUG,
                    "historical Mono primary gutter",
                );
            } else {
                assert_eq!(
                    current_cell, historical_cell,
                    "logical/native cell changed outside the six measured gutters at ({x}, {y})"
                );
                assert_eq!(
                    format!("{current_cell:?}"),
                    format!("{historical_cell:?}"),
                    "native Cell Debug changed outside the six measured gutters at ({x}, {y})"
                );
            }
        }
    }
}

fn assert_historical_returning_mono_replay() {
    let app = App::for_scenario(Scenario::Returning, Motion::Paused);
    let mut harness = Harness::new(app, Theme::junie(), 100, 30).with_color(ColorLevel::Mono);
    harness.draw();
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::Mono,
        "actual Runtime capability is Mono before historical adaptation"
    );

    let current_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(current_scene.key(), "returning 100 30 junie mono");
    assert_eq!(
        format!("{:016x}", current_scene.digest()),
        "872e1c645db902a8",
        "unadapted current Mono frame is asserted before historical recipe application"
    );

    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();
    let current_cursor_bar = harness.cursor_bar();
    let current_focus = harness.runtime().focus();
    assert!(
        current_focus.is_some(),
        "current Mono frame retains its actual focused owner"
    );
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(current_cursor, None);
    assert!(!current_cursor_bar);

    // Pin the complete current tuples before applying any historical style recipe.
    assert_current_returning_mono_tuples(&current);

    // This uses the existing public Theme recipe path; the rendered product is unchanged.
    apply_historical_gutter_recipe(&mut harness);
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::Mono,
        "historical recipe redraw keeps the actual Runtime capability Mono"
    );
    let historical = harness.buffer().clone();
    assert_eq!(harness.cursor(), current_cursor);
    assert_eq!(harness.cursor_bar(), current_cursor_bar);
    assert_eq!(harness.runtime().focus(), current_focus);

    assert_returning_mono_pair(&current, &historical);

    let historical_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(
        format!("{:016x}", historical_scene.digest()),
        "bd5ecca964a2c2be",
        "historical digest is checked only after the unadapted current Mono assertions"
    );
    historical_scene.assert_against(&BASELINE);
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
                    if paper_replay::is_historical_returning_paper_case(
                        scenario, width, height, &theme, color,
                    ) {
                        paper_replay::assert_historical_returning_paper_replay();
                        continue;
                    }
                    if is_historical_returning_case(scenario, width, height, &theme, color) {
                        assert_historical_returning_replay();
                        continue;
                    }
                    if is_historical_returning_mono_case(scenario, width, height, &theme, color) {
                        assert_historical_returning_mono_replay();
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

#[test]
fn jackin_returning_100x30_junie_mono_gutter_replay_matches_held_frame() {
    assert_historical_returning_mono_replay();
}
