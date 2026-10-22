use holla_ui::{App, Motion, Scenario};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Modifier},
};
use termrock::{
    ColorLevel, Family, FeedbackClock, FgStep, Part, Role, SimulationMoment, StylePatch, Theme,
};
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
    let app = App::for_scenario_without_history(scenario, Motion::Paused, 4_000)
        .with_source_theme(theme.clone());
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

fn is_historical_ansi16_faint_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    matches!(
        scenario,
        Scenario::ActivitiesMulti | Scenario::LaunchFailure
    ) && SIZES.contains(&(width, height))
        && *theme == Theme::junie()
        && color == ColorLevel::Ansi16
}

fn is_historical_overflow_faint_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    scenario == Scenario::ParityDiscovery
        && (width, height) == (100, 30)
        && (*theme == Theme::junie() || *theme == Theme::paper())
        && COLORS.contains(&color)
}

fn historical_ansi16_faint_positions(scenario: Scenario, height: u16) -> Vec<(u16, u16)> {
    match scenario {
        Scenario::ActivitiesMulti => (34..=50).chain(66..=71).map(|x| (x, 1)).collect(),
        Scenario::LaunchFailure => (14..=19)
            .map(|x| (x, 1))
            .chain((8..=height.saturating_sub(3)).map(|y| (1, y)))
            .collect(),
        _ => Vec::new(),
    }
}

fn foreground_replay_mismatch(
    current: &Buffer,
    historical: &Buffer,
    expected_positions: &[(u16, u16)],
    current_fg: Color,
    historical_fg: Color,
) -> Option<String> {
    if current.area != historical.area {
        return Some(format!(
            "area changed from {:?} to {:?}",
            current.area, historical.area
        ));
    }
    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = current.cell((x, y)).expect("position in current buffer");
            let historical_cell = historical
                .cell((x, y))
                .expect("position in historical buffer");
            if expected_positions.contains(&(x, y)) {
                if current_cell.fg != current_fg {
                    return Some(format!(
                        "current foreground at ({x}, {y}) was {:?}, expected {:?}",
                        current_cell.fg, current_fg
                    ));
                }
                if historical_cell.fg != historical_fg {
                    return Some(format!(
                        "historical foreground at ({x}, {y}) was {:?}, expected {:?}",
                        historical_cell.fg, historical_fg
                    ));
                }
                let mut expected = current_cell.clone();
                expected.fg = historical_fg;
                if historical_cell != &expected {
                    return Some(format!(
                        "non-foreground field changed at ({x}, {y}): current {:?}, historical {:?}",
                        current_cell, historical_cell
                    ));
                }
            } else if current_cell != historical_cell {
                return Some(format!(
                    "unexpected frame difference at ({x}, {y}): current {:?}, historical {:?}",
                    current_cell, historical_cell
                ));
            }
        }
    }
    None
}

fn assert_foreground_only_replay(
    current: &Buffer,
    historical: &Buffer,
    expected_positions: &[(u16, u16)],
    current_fg: Color,
    historical_fg: Color,
) {
    let mismatch = foreground_replay_mismatch(
        current,
        historical,
        expected_positions,
        current_fg,
        historical_fg,
    );
    assert!(
        mismatch.is_none(),
        "historical replay escaped its foreground-only allowlist: {}",
        mismatch.as_deref().unwrap_or("unknown mismatch")
    );
}

fn assert_historical_ansi16_faint_replay(scenario: Scenario, width: u16, height: u16) {
    let mut harness = harness(scenario, Theme::junie(), width, height, ColorLevel::Ansi16);
    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();
    let expected_cursor = match scenario {
        Scenario::ActivitiesMulti => Some(Position { x: 3, y: 4 }),
        Scenario::LaunchFailure => None,
        _ => None,
    };
    assert_eq!(current.area, Rect::new(0, 0, width, height));
    assert_eq!(current_cursor, expected_cursor);

    let positions = historical_ansi16_faint_positions(scenario, height);
    let expected_count = match scenario {
        Scenario::ActivitiesMulti => 23,
        Scenario::LaunchFailure => 6 + usize::from(height.saturating_sub(3).saturating_sub(8)) + 1,
        _ => 0,
    };
    assert_eq!(positions.len(), expected_count);
    for &(x, y) in &positions {
        let cell = current.cell((x, y)).expect("measured faint cell");
        assert_eq!(
            cell.fg,
            Color::Gray,
            "current Junie ANSI16 cell at ({x}, {y})"
        );
        assert_eq!(cell.bg, Color::Black, "background at ({x}, {y})");
    }

    let mut historical_theme = harness.runtime().theme().clone();
    historical_theme.color.fg[3] = Color::DarkGray;
    harness.runtime_mut().set_theme(historical_theme);
    harness.draw();
    let historical = harness.buffer().clone();
    assert_eq!(harness.cursor(), current_cursor);
    assert_foreground_only_replay(
        &current,
        &historical,
        &positions,
        Color::Gray,
        Color::DarkGray,
    );
    harness
        .snapshot()
        .named(scenario.name())
        .assert_against(&BASELINE);
}

fn expected_overflow_faint(theme: &Theme, color: ColorLevel) -> Option<Color> {
    if *theme == Theme::junie() {
        Some(match color {
            ColorLevel::TrueColor => Color::Rgb(115, 115, 115),
            ColorLevel::Ansi256 => Color::Indexed(242),
            ColorLevel::Ansi16 | ColorLevel::Mono => Color::Gray,
            _ => return None,
        })
    } else if *theme == Theme::paper() {
        Some(match color {
            ColorLevel::TrueColor => Color::Rgb(156, 148, 138),
            ColorLevel::Ansi256 => Color::Indexed(245),
            // Paper Faint RGB(156,148,138) projects to DarkGray at ANSI16.
            ColorLevel::Ansi16 => Color::DarkGray,
            // The readable source Faint collapses to Black, so the source-qualified footer
            // fallback uses Paper's projected canvas ink to keep the clipped marker visible.
            ColorLevel::Mono => Color::White,
            _ => return None,
        })
    } else {
        None
    }
}

fn historical_projected_white(color: ColorLevel) -> Color {
    match color {
        ColorLevel::TrueColor => Color::Rgb(255, 255, 255),
        ColorLevel::Ansi256 => Color::Indexed(231),
        ColorLevel::Ansi16 | ColorLevel::Mono => Color::White,
        _ => Color::Reset,
    }
}

fn black_for_level(color: ColorLevel) -> Color {
    match color {
        ColorLevel::TrueColor => Color::Rgb(0, 0, 0),
        ColorLevel::Ansi256 => Color::Indexed(16),
        ColorLevel::Ansi16 | ColorLevel::Mono => Color::Black,
        _ => Color::Reset,
    }
}

const OVERFLOW_POSITION: (u16, u16) = (92, 29);
const CLIPPED_HINT_PREFIX: &str =
    " Type Search  ↑↓ Move  Enter Confirm…  Alt+Enter Alternatives…  Tab Preview  Ctrl+↑↓ Scope  …";

fn assert_clipped_hint_row(row: &str) {
    assert!(
        row.starts_with(CLIPPED_HINT_PREFIX),
        "unexpected hint row: {row:?}"
    );
    assert!(
        !row.contains("Ctrl+G Activities"),
        "seventh hint was not clipped: {row:?}"
    );
    assert!(
        row.trim_end().ends_with('…'),
        "overflow marker is absent: {row:?}"
    );
}

fn assert_historical_overflow_faint_replay(theme: Theme, color: ColorLevel) {
    let current_fg = expected_overflow_faint(&theme, color).expect("reviewed theme and color tier");
    let mut harness = harness(Scenario::ParityDiscovery, theme, 100, 30, color);
    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(current_cursor, Some(Position { x: 3, y: 4 }));
    let current_cell = current
        .cell(OVERFLOW_POSITION)
        .expect("overflow marker inside captured frame");
    assert_eq!(
        current_cell,
        &expected_cell("…", current_fg, black_for_level(color), Modifier::empty()),
        "current semantic overflow at {OVERFLOW_POSITION:?}"
    );
    assert_ne!(
        current_cell.fg, current_cell.bg,
        "overflow must remain visible"
    );
    assert_clipped_hint_row(&harness.row(29));

    let historical_fg = historical_projected_white(color);
    let mut historical_theme = harness.runtime().theme().clone();
    historical_theme.color.fg[3] = historical_fg;
    harness.runtime_mut().set_theme(historical_theme);
    harness.draw();
    let historical = harness.buffer().clone();
    assert_eq!(harness.cursor(), current_cursor);
    assert_foreground_only_replay(
        &current,
        &historical,
        &[OVERFLOW_POSITION],
        current_fg,
        historical_fg,
    );
    let historical_cell = historical
        .cell(OVERFLOW_POSITION)
        .expect("historical overflow marker inside captured frame");
    assert_eq!(
        historical_cell,
        &expected_cell(
            "…",
            historical_fg,
            black_for_level(color),
            Modifier::empty()
        )
    );
    assert_clipped_hint_row(&harness.row(29));
    harness
        .snapshot()
        .named(Scenario::ParityDiscovery.name())
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
                    if is_historical_ansi16_faint_case(scenario, width, height, &theme, color) {
                        assert_historical_ansi16_faint_replay(scenario, width, height);
                        continue;
                    }
                    if is_historical_overflow_faint_case(scenario, width, height, &theme, color) {
                        assert_historical_overflow_faint_replay(theme.clone(), color);
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

#[test]
fn holla_historical_faint_replays_match_the_sixteen_reviewed_frames() {
    for scenario in [Scenario::ActivitiesMulti, Scenario::LaunchFailure] {
        for (width, height) in SIZES {
            assert_historical_ansi16_faint_replay(scenario, width, height);
        }
    }
    for theme in [Theme::junie(), Theme::paper()] {
        for color in COLORS {
            assert_historical_overflow_faint_replay(theme.clone(), color);
        }
    }
}

#[test]
fn holla_historical_faint_replay_selectors_are_exact() {
    let junie = Theme::junie();
    let paper = Theme::paper();
    for (scenario, width, height) in [
        (Scenario::ActivitiesMulti, 80, 24),
        (Scenario::ActivitiesMulti, 100, 30),
        (Scenario::ActivitiesMulti, 120, 40),
        (Scenario::ActivitiesMulti, 160, 50),
        (Scenario::LaunchFailure, 80, 24),
        (Scenario::LaunchFailure, 100, 30),
        (Scenario::LaunchFailure, 120, 40),
        (Scenario::LaunchFailure, 160, 50),
    ] {
        assert!(is_historical_ansi16_faint_case(
            scenario,
            width,
            height,
            &junie,
            ColorLevel::Ansi16
        ));
    }
    assert!(!is_historical_ansi16_faint_case(
        Scenario::ActivitiesMulti,
        101,
        30,
        &junie,
        ColorLevel::Ansi16
    ));
    assert!(!is_historical_ansi16_faint_case(
        Scenario::ActivitiesMulti,
        100,
        30,
        &paper,
        ColorLevel::Ansi16
    ));
    assert!(!is_historical_ansi16_faint_case(
        Scenario::ActivitiesMulti,
        100,
        30,
        &junie,
        ColorLevel::Ansi256
    ));
    assert!(!is_historical_ansi16_faint_case(
        Scenario::ParityDiscovery,
        100,
        30,
        &junie,
        ColorLevel::Ansi16
    ));

    for theme in [&junie, &paper] {
        for color in COLORS {
            assert!(is_historical_overflow_faint_case(
                Scenario::ParityDiscovery,
                100,
                30,
                theme,
                color
            ));
        }
    }
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        101,
        30,
        &junie,
        ColorLevel::TrueColor
    ));
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        80,
        24,
        &junie,
        ColorLevel::Mono
    ));
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityHistory,
        100,
        30,
        &junie,
        ColorLevel::TrueColor
    ));
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        100,
        30,
        &Theme::from_tokens(junie.color),
        ColorLevel::TrueColor
    ));
}

#[test]
fn holla_clipped_hint_overflow_tracks_custom_faint_only() {
    let baseline = harness(
        Scenario::ParityDiscovery,
        Theme::junie(),
        100,
        30,
        ColorLevel::TrueColor,
    );
    let faint = Color::Rgb(18, 224, 136);
    let primary = Color::Rgb(66, 132, 198);
    let muted = Color::Rgb(224, 74, 138);
    let mut custom_theme = Theme::junie();
    custom_theme.color.fg[0] = primary;
    custom_theme.color.fg[2] = muted;
    custom_theme.color.fg[3] = faint;
    let custom = harness(
        Scenario::ParityDiscovery,
        custom_theme,
        100,
        30,
        ColorLevel::TrueColor,
    );

    assert_eq!(baseline.buffer().area, custom.buffer().area);
    assert_eq!(baseline.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(custom.cursor(), baseline.cursor());
    assert_clipped_hint_row(&baseline.row(29));
    assert_clipped_hint_row(&custom.row(29));
    let expected = expected_cell("…", faint, Color::Rgb(0, 0, 0), Modifier::empty());
    assert_eq!(
        custom.cell(OVERFLOW_POSITION.0, OVERFLOW_POSITION.1),
        &expected
    );
    assert_ne!(expected.fg, expected.bg);
    assert_ne!(expected.fg, primary);
    assert_ne!(expected.fg, muted);
    for x in 0..100 {
        if (x, 29) == OVERFLOW_POSITION {
            continue;
        }
        assert_eq!(
            baseline.cell(x, 29),
            custom.cell(x, 29),
            "hint cell at x={x}"
        );
    }
}

fn paper_theme_with_faint(faint: Color) -> Theme {
    let mut tokens = Theme::paper().color;
    tokens.fg[3] = faint;
    Theme::from_tokens(tokens)
}

fn paper_theme_with_faint_base_and_mono_foreground(foreground: Color) -> Theme {
    Theme::paper()
        .define_family(Family::HINTBAR, |recipe| {
            recipe
                .part(Part::OVERFLOW)
                .base(StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
        })
        .define_family_for_level(ColorLevel::Mono, Family::HINTBAR, |recipe| {
            recipe
                .part(Part::OVERFLOW)
                .base(StylePatch::new().set_fg(Role::Custom(foreground)));
        })
}

fn unconfigured_parity_harness(theme: Theme, width: u16, height: u16) -> Harness<App> {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let initial = SimulationMoment::from_millis(app.fixture_time_ms() as u64);
    Harness::new_with_feedback_clock(
        app,
        theme,
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
}

fn assert_only_overflow_diff(current: &Buffer, projected: &Buffer, position: (u16, u16)) {
    assert_eq!(current.area, projected.area);
    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = current.cell((x, y)).expect("inside current frame");
            let projected_cell = projected.cell((x, y)).expect("inside projected frame");
            if (x, y) == position {
                assert_ne!(current_cell, projected_cell, "overflow at ({x}, {y})");
            } else {
                assert_eq!(current_cell, projected_cell, "cell at ({x}, {y})");
            }
        }
    }
}

#[test]
fn holla_paper_mono_overflow_uses_only_configured_readable_source_provenance() {
    for (width, height, position) in [(80, 24, (77, 23)), (100, 30, OVERFLOW_POSITION)] {
        let mut configured = harness(
            Scenario::ParityDiscovery,
            Theme::paper(),
            width,
            height,
            ColorLevel::Mono,
        );
        let projected_theme = Theme::paper().for_level(ColorLevel::Mono);
        let unconfigured = unconfigured_parity_harness(projected_theme, width, height);
        assert_eq!(configured.cursor(), Some(Position { x: 3, y: 4 }));
        assert_eq!(unconfigured.cursor(), configured.cursor());
        assert_only_overflow_diff(configured.buffer(), unconfigured.buffer(), position);
        assert_eq!(
            unconfigured
                .buffer()
                .cell(position)
                .expect("clipped marker in projected frame")
                .fg,
            Color::Black,
            "an embedding without source provenance retains its projected collision"
        );

        let cell = configured
            .buffer()
            .cell(position)
            .expect("clipped marker in frame");
        assert_eq!(
            cell,
            &expected_cell("…", Color::White, Color::Black, Modifier::empty())
        );
        let projected_surface = Theme::paper().for_level(ColorLevel::Mono).color.surfaces[0];
        assert_eq!(cell.fg, projected_surface);
        let contrast = termrock::theme::contrast_ratio(cell.fg, cell.bg)
            .expect("Paper Mono overflow colors have known contrast");
        assert!(contrast >= 3.0, "configured marker contrast was {contrast}");
        assert_ne!(cell.fg, cell.bg, "the real clipped marker remains visible");
        if width == 100 {
            assert_clipped_hint_row(&configured.row(height - 1));
        } else {
            assert!(
                configured.row(height - 1).trim_end().ends_with('…'),
                "the narrow row retains its clipped marker"
            );
        }

        let mut changed_runtime_theme = configured.runtime().theme().clone();
        changed_runtime_theme.color.fg[0] = Color::White;
        configured.runtime_mut().set_theme(changed_runtime_theme);
        configured.draw();
        assert_eq!(
            configured
                .buffer()
                .cell(position)
                .expect("clipped marker in frame")
                .fg,
            Color::Black,
            "a runtime edit invalidates stale source-theme provenance"
        );
    }
}

#[test]
fn holla_source_projection_cache_tracks_level_changes_narrowed_sources_and_reconfiguration() {
    let scene = harness(
        Scenario::ParityDiscovery,
        Theme::paper(),
        80,
        24,
        ColorLevel::TrueColor,
    );
    let mut scene = scene;
    scene
        .runtime_mut()
        .set_theme(Theme::paper().for_level(ColorLevel::Ansi16));
    scene.draw();
    assert_eq!(
        scene
            .buffer()
            .cell((77, 23))
            .expect("clipped marker after ANSI16 projection")
            .fg,
        Color::DarkGray,
        "the same scene selects the cached ANSI16 source projection"
    );
    scene
        .runtime_mut()
        .set_theme(Theme::paper().for_level(ColorLevel::Mono));
    scene.draw();
    assert_eq!(
        scene
            .buffer()
            .cell((77, 23))
            .expect("clipped marker after Mono projection"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "the same scene selects the cached Mono source projection"
    );

    let narrowed_source = Theme::paper().for_level(ColorLevel::Ansi256);
    let narrowed = harness(
        Scenario::ParityDiscovery,
        narrowed_source,
        80,
        24,
        ColorLevel::Mono,
    );
    assert_eq!(
        narrowed
            .buffer()
            .cell((77, 23))
            .expect("clipped marker from a narrowed source"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "an already-projected source remains narrow while its exact Mono projection validates"
    );

    let source_theme = Theme::paper();
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
        .with_source_theme(Theme::junie())
        .with_source_theme(source_theme.clone());
    let initial = SimulationMoment::from_millis(app.fixture_time_ms() as u64);
    let reconfigured = Harness::new_with_feedback_clock(
        app,
        source_theme,
        80,
        24,
        FeedbackClock::Simulation { initial },
    )
    .with_color(ColorLevel::Mono);
    assert_eq!(
        reconfigured
            .buffer()
            .cell((77, 23))
            .expect("clipped marker after source reconfiguration"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "reconfiguration replaces source metadata and every cached projection together"
    );
}

#[test]
fn holla_overflow_preserves_authored_part_and_level_foregrounds() {
    let paper = paper_theme_with_faint(Color::Rgb(156, 148, 138));
    let direct_ink = paper.clone().define_family(Family::HINTBAR, |recipe| {
        recipe
            .part(Part::OVERFLOW)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Rgb(255, 255, 255))));
    });
    let direct_control =
        unconfigured_parity_harness(direct_ink.clone().for_level(ColorLevel::Mono), 100, 30);
    let mut direct = harness(
        Scenario::ParityDiscovery,
        direct_ink,
        100,
        30,
        ColorLevel::Mono,
    );
    assert_eq!(
        direct
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "the source recipe's foreground remains above Holla's local default"
    );
    assert_eq!(
        direct_control
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in unconfigured frame")
            .fg,
        Color::Black,
        "the control confirms Holla's local Faint patch would otherwise win"
    );

    let mut stale_direct = direct.runtime().theme().clone();
    stale_direct.color.fg[0] = Color::Green;
    direct.runtime_mut().set_theme(stale_direct);
    direct.draw();
    assert_eq!(
        direct
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker after a runtime theme edit")
            .fg,
        Color::Black,
        "a runtime edit invalidates direct source-recipe provenance"
    );

    let reset_ink = paper.clone().define_family(Family::HINTBAR, |recipe| {
        recipe
            .part(Part::OVERFLOW)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Reset)));
    });
    let reset_control =
        unconfigured_parity_harness(reset_ink.clone().for_level(ColorLevel::Mono), 100, 30);
    let reset = harness(
        Scenario::ParityDiscovery,
        reset_ink,
        100,
        30,
        ColorLevel::Mono,
    );
    assert_eq!(
        reset
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "an authored Reset leaves the previously painted cell ink in place"
    );
    assert_eq!(
        reset_control
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in unconfigured frame")
            .fg,
        Color::Black,
        "the control confirms the local Faint patch would mask the authored Reset"
    );

    let level_reset = paper.define_family_for_level(ColorLevel::Mono, Family::HINTBAR, |recipe| {
        recipe
            .part(Part::OVERFLOW)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Reset)));
    });
    let level = harness(
        Scenario::ParityDiscovery,
        level_reset,
        100,
        30,
        ColorLevel::Mono,
    );
    assert_eq!(
        level
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame"),
        &expected_cell("…", Color::Reset, Color::Black, Modifier::empty()),
        "an opaque level recipe keeps its authored Reset when no resolved ink is available"
    );
}

#[test]
fn holla_unknown_faint_base_preserves_mono_level_white_foreground() {
    let source = paper_theme_with_faint_base_and_mono_foreground(Color::White);
    let source_foreground = source
        .resolve(
            Family::HINTBAR,
            termrock::Variant::DEFAULT,
            Part::OVERFLOW,
            termrock::StateFlags::empty(),
            termrock::Surface::Canvas,
        )
        .style
        .fg_role;
    assert!(matches!(
        source_foreground,
        Some((Role::Fg(FgStep::Faint), _))
    ));

    let mono = source.for_level(ColorLevel::Mono);
    assert!(matches!(
        mono.resolve(
            Family::HINTBAR,
            termrock::Variant::DEFAULT,
            Part::OVERFLOW,
            termrock::StateFlags::empty(),
            termrock::Surface::Canvas,
        )
        .style
        .fg_role,
        Some((Role::Custom(Color::White), _))
    ));

    let unconfigured = unconfigured_parity_harness(mono, 100, 30);
    let scene = harness(Scenario::ParityDiscovery, source, 100, 30, ColorLevel::Mono);
    assert_eq!(scene.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(unconfigured.cursor(), scene.cursor());
    assert_only_overflow_diff(scene.buffer(), unconfigured.buffer(), OVERFLOW_POSITION);
    assert_eq!(
        scene
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame"),
        &expected_cell("…", Color::White, Color::Black, Modifier::empty()),
        "an unknown source recipe must preserve its explicit Mono foreground"
    );
    assert_eq!(
        unconfigured
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in unconfigured frame")
            .fg,
        Color::Black,
        "the unconfigured control confirms the local Faint patch would mask White"
    );
}

#[test]
fn holla_unknown_faint_base_preserves_mono_level_reset_foreground() {
    let source = paper_theme_with_faint_base_and_mono_foreground(Color::Reset);
    let source_foreground = source
        .resolve(
            Family::HINTBAR,
            termrock::Variant::DEFAULT,
            Part::OVERFLOW,
            termrock::StateFlags::empty(),
            termrock::Surface::Canvas,
        )
        .style
        .fg_role;
    assert!(matches!(
        source_foreground,
        Some((Role::Fg(FgStep::Faint), _))
    ));

    let mono = source.for_level(ColorLevel::Mono);
    let unconfigured = unconfigured_parity_harness(mono, 100, 30);
    let scene = harness(Scenario::ParityDiscovery, source, 100, 30, ColorLevel::Mono);
    assert_eq!(scene.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(unconfigured.cursor(), scene.cursor());
    assert_only_overflow_diff(scene.buffer(), unconfigured.buffer(), OVERFLOW_POSITION);
    assert_eq!(
        scene
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame"),
        &expected_cell("…", Color::Reset, Color::Black, Modifier::empty()),
        "an unknown source recipe must preserve its explicit Mono Reset"
    );
    assert_eq!(
        unconfigured
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in unconfigured frame")
            .fg,
        Color::Black,
        "the unconfigured control confirms the local Faint patch would mask Reset"
    );
}

#[test]
fn holla_overflow_does_not_guess_for_equal_near_equal_reset_or_explicit_ink() {
    for faint in [Color::Black, Color::Rgb(20, 20, 20), Color::Reset] {
        let source = paper_theme_with_faint(faint);
        let unconfigured =
            unconfigured_parity_harness(source.clone().for_level(ColorLevel::Mono), 100, 30);
        let scene = harness(Scenario::ParityDiscovery, source, 100, 30, ColorLevel::Mono);
        assert_eq!(
            scene
                .buffer()
                .cell(OVERFLOW_POSITION)
                .expect("clipped marker in frame"),
            unconfigured
                .buffer()
                .cell(OVERFLOW_POSITION)
                .expect("clipped marker in unconfigured frame"),
            "authored faint {faint:?} does not receive a contrast guess"
        );
    }

    let source = paper_theme_with_faint(Color::Rgb(156, 148, 138));
    let mut explicit_mono = source.for_level(ColorLevel::Mono).color;
    explicit_mono.fg[3] = Color::Black;
    let explicit_theme = source
        .clone()
        .builder()
        .capability_palettes(
            termrock::CapabilityPalettes::new(source.color).with(ColorLevel::Mono, explicit_mono),
        )
        .build();
    let scene = harness(
        Scenario::ParityDiscovery,
        explicit_theme,
        100,
        30,
        ColorLevel::Mono,
    );
    assert_eq!(
        scene
            .buffer()
            .cell(OVERFLOW_POSITION)
            .expect("clipped marker in frame")
            .fg,
        Color::Black,
        "an explicit per-tier palette retains its authored collision"
    );
}

#[test]
fn holla_hintbar_uses_adjacent_widths_to_clip_or_fit() {
    let mut rendered = Vec::new();
    for width in 80..=160 {
        let harness = harness(
            Scenario::ParityDiscovery,
            Theme::junie(),
            width,
            40,
            ColorLevel::TrueColor,
        );
        rendered.push((width, harness.row(39)));
    }
    let fit_index = rendered
        .iter()
        .position(|(_, row)| row.contains("Ctrl+G Activities") && !row.trim_end().ends_with('…'))
        .expect("wide real HintBar eventually fits all seven hints");
    assert!(
        fit_index > 0,
        "the discovery hint row must clip at narrower widths"
    );
    let (narrow_width, narrow_row) = &rendered[fit_index - 1];
    let (wide_width, wide_row) = &rendered[fit_index];
    assert_eq!(
        *wide_width,
        narrow_width + 1,
        "fit boundary uses adjacent widths"
    );
    assert!(narrow_row.contains("Type Search"), "{narrow_row:?}");
    assert!(!narrow_row.contains("Ctrl+G Activities"), "{narrow_row:?}");
    assert!(narrow_row.trim_end().ends_with('…'), "{narrow_row:?}");
    assert!(wide_row.contains("Ctrl+G Activities"), "{wide_row:?}");
    assert!(!wide_row.trim_end().ends_with('…'), "{wide_row:?}");
}

#[test]
fn holla_faint_replay_guards_reject_role_and_unrelated_cell_mutations() {
    let current_harness = harness(
        Scenario::ParityDiscovery,
        Theme::junie(),
        100,
        30,
        ColorLevel::TrueColor,
    );
    let current = current_harness.buffer().clone();
    let mut historical_harness = harness(
        Scenario::ParityDiscovery,
        Theme::junie(),
        100,
        30,
        ColorLevel::TrueColor,
    );
    let mut historical_theme = historical_harness.runtime().theme().clone();
    historical_theme.color.fg[3] = Color::Rgb(255, 255, 255);
    historical_harness.runtime_mut().set_theme(historical_theme);
    historical_harness.draw();
    let historical = historical_harness.buffer().clone();
    let positions = [OVERFLOW_POSITION];
    let current_fg = Color::Rgb(115, 115, 115);
    let historical_fg = Color::Rgb(255, 255, 255);
    assert_eq!(
        foreground_replay_mismatch(&current, &historical, &positions, current_fg, historical_fg),
        None
    );

    let mut wrong_faint_theme = Theme::junie();
    wrong_faint_theme.color.fg[3] = Color::Gray;
    let wrong_faint = harness(
        Scenario::ParityDiscovery,
        wrong_faint_theme,
        100,
        30,
        ColorLevel::TrueColor,
    )
    .buffer()
    .clone();
    assert!(
        foreground_replay_mismatch(
            &wrong_faint,
            &historical,
            &positions,
            current_fg,
            historical_fg
        )
        .is_some()
    );

    let mut unrelated_mutation = historical.clone();
    unrelated_mutation
        .cell_mut((0, 0))
        .expect("unrelated cell in frame")
        .fg = Color::Red;
    assert!(
        foreground_replay_mismatch(
            &current,
            &unrelated_mutation,
            &positions,
            current_fg,
            historical_fg
        )
        .is_some()
    );
}
