use holla_ui::{App, Motion, Scenario, domain::action::Scope, domain::ranking::Pin};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Position, Rect},
    style::{Color, Modifier},
};
use std::collections::BTreeSet;
use termrock::{
    ColorLevel, Family, FeedbackClock, FgStep, Id, Part, PartRef, Role, SimulationMoment,
    StateFlags, StylePatch, Theme,
};
use termrock_test_support::{Baseline, Harness, Scene};

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

const BORDER_PALETTE_CASES: [Scenario; 4] = [
    Scenario::RustDirty,
    Scenario::MonorepoRoot,
    Scenario::MonorepoChild,
    Scenario::ActivitiesMulti,
];
const HOME_ROWS: Id = Id::root("home.rows");
const HOME_QUERY: Id = Id::root("home.query");

const TRACKED_PORT_OWNER_KEYS: [&str; 11] = [
    "monorepo-root 80 24 junie mono",
    "monorepo-root 80 24 paper truecolor",
    "monorepo-root 80 24 paper 256",
    "monorepo-root 80 24 paper 16",
    "monorepo-root 80 24 paper mono",
    "activities-multi 80 24 junie truecolor",
    "activities-multi 80 24 junie mono",
    "activities-multi 80 24 paper truecolor",
    "activities-multi 80 24 paper 256",
    "activities-multi 80 24 paper 16",
    "activities-multi 80 24 paper mono",
];

const EXISTING_CURRENT_A_OWNER_KEYS: [&str; 6] = [
    "rust-dirty 80 24 junie 256",
    "monorepo-root 80 24 junie 256",
    "monorepo-child 80 24 junie 256",
    "activities-multi 80 24 junie 256",
    "monorepo-root 80 24 junie truecolor",
    "monorepo-root 80 24 junie 16",
];

const ROOT100_NORMAL_A_OWNER_KEY: &str = "monorepo-root 100 30 junie truecolor";
const ROOT_NORMAL_A_OWNER_VIEWPORTS: [(u16, u16); 3] = [(100, 30), (120, 40), (160, 50)];
const REMAINING_ROOT_NORMAL_A_OWNER_KEYS: [&str; 23] = [
    "monorepo-root 100 30 junie 256",
    "monorepo-root 100 30 junie 16",
    "monorepo-root 100 30 junie mono",
    "monorepo-root 100 30 paper truecolor",
    "monorepo-root 100 30 paper 256",
    "monorepo-root 100 30 paper 16",
    "monorepo-root 100 30 paper mono",
    "monorepo-root 120 40 junie truecolor",
    "monorepo-root 120 40 junie 256",
    "monorepo-root 120 40 junie 16",
    "monorepo-root 120 40 junie mono",
    "monorepo-root 120 40 paper truecolor",
    "monorepo-root 120 40 paper 256",
    "monorepo-root 120 40 paper 16",
    "monorepo-root 120 40 paper mono",
    "monorepo-root 160 50 junie truecolor",
    "monorepo-root 160 50 junie 256",
    "monorepo-root 160 50 junie 16",
    "monorepo-root 160 50 junie mono",
    "monorepo-root 160 50 paper truecolor",
    "monorepo-root 160 50 paper 256",
    "monorepo-root 160 50 paper 16",
    "monorepo-root 160 50 paper mono",
];

fn tracked_port_owner_key(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> Option<&'static str> {
    if (width, height) != (80, 24) {
        return None;
    }

    let junie = *theme == Theme::junie();
    let paper = *theme == Theme::paper();
    match scenario {
        Scenario::MonorepoRoot if junie && color == ColorLevel::Mono => {
            Some("monorepo-root 80 24 junie mono")
        }
        Scenario::MonorepoRoot if paper => match color {
            ColorLevel::TrueColor => Some("monorepo-root 80 24 paper truecolor"),
            ColorLevel::Ansi256 => Some("monorepo-root 80 24 paper 256"),
            ColorLevel::Ansi16 => Some("monorepo-root 80 24 paper 16"),
            ColorLevel::Mono => Some("monorepo-root 80 24 paper mono"),
            _ => None,
        },
        Scenario::ActivitiesMulti if junie => match color {
            ColorLevel::TrueColor => Some("activities-multi 80 24 junie truecolor"),
            ColorLevel::Mono => Some("activities-multi 80 24 junie mono"),
            _ => None,
        },
        Scenario::ActivitiesMulti if paper => match color {
            ColorLevel::TrueColor => Some("activities-multi 80 24 paper truecolor"),
            ColorLevel::Ansi256 => Some("activities-multi 80 24 paper 256"),
            ColorLevel::Ansi16 => Some("activities-multi 80 24 paper 16"),
            ColorLevel::Mono => Some("activities-multi 80 24 paper mono"),
            _ => None,
        },
        _ => None,
    }
}

fn assert_tracked_port_owner_current_a(
    scenario: Scenario,
    theme: Theme,
    color: ColorLevel,
    expected_key: &str,
) -> String {
    let mut world = holla_ui::sim::fixtures::world_for(scenario);
    world.seek(4_000);
    assert_eq!(world.scenario, scenario, "{expected_key} model scenario");
    assert_eq!(world.clock.now_ms, 4_000, "{expected_key} model time");

    let raw = holla_ui::sim::catalogue::catalogue(&world);
    assert_eq!(
        raw.iter()
            .filter(|action| action.id == "system.port")
            .count(),
        1,
        "{expected_key} unique raw Port action"
    );
    let port = raw
        .iter()
        .find(|action| action.id == "system.port")
        .expect("unique raw Port action is present");
    assert_eq!(
        port.scope,
        Scope::Host,
        "{expected_key} canonical Port scope"
    );
    assert_eq!(
        port.workdir.as_str(),
        world.cwd.as_str(),
        "{expected_key} Port command uses the fixture cwd"
    );
    let local_port_pin = Pin {
        path: world.cwd.clone(),
        command: port.command.clone(),
    };
    assert_eq!(
        local_port_pin.path, port.workdir,
        "{expected_key} local Port pin path"
    );
    assert!(
        !world
            .memory
            .hidden_at(&local_port_pin.path, &local_port_pin.command),
        "{expected_key} actual local Port command is unhidden"
    );

    let rows = home_display_rows(&world);
    assert_eq!(
        rows.iter()
            .filter(|(_, action_id)| action_id == "system.port")
            .count(),
        1,
        "{expected_key} unique visible Port action"
    );
    let extent = home_visual_extent(&rows);
    assert!(
        extent > 15,
        "{expected_key} model rows overflow the viewport"
    );
    let (first_section, first_action_id) = rows
        .first()
        .expect("nonempty Home model has a first action");
    assert!(
        !first_section.is_empty(),
        "{expected_key} first section exists"
    );

    let ordinary = harness(scenario, theme.clone(), 80, 24, color);
    assert_eq!(
        ordinary.runtime().theme(),
        &theme.for_level(color),
        "{expected_key} runtime uses the source theme projection"
    );
    assert_eq!(
        ordinary.app().fixture_time_ms(),
        4_000,
        "{expected_key} App fixture time"
    );
    assert_eq!(
        ordinary.focus(),
        Some(HOME_QUERY),
        "{expected_key} query focus"
    );
    assert_eq!(
        ordinary.cursor(),
        Some(Position { x: 3, y: 4 }),
        "{expected_key} cursor"
    );
    assert!(
        ordinary.diagnostics().is_empty(),
        "{expected_key} runtime diagnostics"
    );

    let frame = ordinary.buffer();
    assert_eq!(*frame.area(), Rect::new(0, 0, 80, 24));
    let mut complete_cells = 0_usize;
    for y in 0..24 {
        for x in 0..80 {
            assert!(
                frame.cell((x, y)).is_some(),
                "{expected_key} cell ({x},{y})"
            );
            complete_cells = complete_cells.saturating_add(1);
        }
    }
    assert_eq!(complete_cells, 1_920, "{expected_key} full frame cells");

    let scene = ordinary.snapshot().named(scenario.name());
    let actual_key = scene.key().to_owned();
    assert_eq!(actual_key, expected_key, "named normal-A frame key");

    let rows_area = ordinary
        .area_of(HOME_ROWS)
        .expect("native Home rows are registered");
    let track = ordinary
        .area_of_part(HOME_ROWS, PartRef::of(Part::TRACK))
        .expect("native Home track is registered");
    let thumb = ordinary
        .area_of_part(HOME_ROWS, PartRef::of(Part::THUMB))
        .expect("native Home thumb is registered");
    assert_eq!(
        rows_area,
        Rect::new(1, 6, 78, 15),
        "{expected_key} rows area"
    );
    assert_eq!(track, Rect::new(78, 6, 1, 15), "{expected_key} track area");
    assert_eq!(thumb.x, track.x, "{expected_key} thumb column");
    assert_eq!(thumb.y, track.y, "{expected_key} zero initial offset");
    assert_eq!(thumb.width, track.width, "{expected_key} thumb width");
    let expected_thumb_height =
        (usize::from(rows_area.height).saturating_mul(usize::from(track.height)) / extent)
            .max(1)
            .min(usize::from(track.height));
    assert_eq!(
        usize::from(thumb.height),
        expected_thumb_height,
        "{expected_key} model-derived thumb height"
    );

    let first_row = ordinary
        .area_of_part(
            HOME_ROWS,
            PartRef::item(Part::ROW, termrock::ItemKey::text(first_action_id)),
        )
        .expect("native first action row is registered");
    assert_eq!(first_row.x, rows_area.x, "{expected_key} first row x");
    assert_eq!(first_row.height, 1, "{expected_key} first row height");
    assert_eq!(
        first_row.y,
        rows_area.y + 1,
        "{expected_key} header precedes first row; offset is zero"
    );
    actual_key
}

fn root100_normal_a_owner_key(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> Option<&'static str> {
    if scenario == Scenario::MonorepoRoot
        && (width, height) == (100, 30)
        && *theme == Theme::junie()
        && color == ColorLevel::TrueColor
    {
        Some(ROOT100_NORMAL_A_OWNER_KEY)
    } else {
        None
    }
}

fn root_normal_a_owner_key(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> Option<String> {
    if scenario != Scenario::MonorepoRoot
        || !ROOT_NORMAL_A_OWNER_VIEWPORTS.contains(&(width, height))
    {
        return None;
    }

    let theme_name = if *theme == Theme::junie() {
        "junie"
    } else if *theme == Theme::paper() {
        "paper"
    } else {
        return None;
    };
    let color_name = match color {
        ColorLevel::TrueColor => "truecolor",
        ColorLevel::Ansi256 => "256",
        ColorLevel::Ansi16 => "16",
        ColorLevel::Mono => "mono",
        _ => return None,
    };

    Some(format!(
        "{} {width} {height} {theme_name} {color_name}",
        scenario.name()
    ))
}

fn remaining_root_normal_a_owner_key(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> Option<String> {
    let key = root_normal_a_owner_key(scenario, width, height, theme, color)?;
    (key != ROOT100_NORMAL_A_OWNER_KEY).then_some(key)
}

fn root100_normal_a_action_rects(
    rows: &[(String, String)],
    rows_area: Rect,
) -> Vec<(String, Rect)> {
    let mut action_rects = Vec::with_capacity(rows.len());
    let mut y = rows_area.y;
    let mut previous_section = None;

    for (section, action_id) in rows {
        if previous_section != Some(section.as_str()) {
            if previous_section.is_some() {
                y = y.saturating_add(1);
            }
            y = y.saturating_add(1);
            previous_section = Some(section.as_str());
        }

        action_rects.push((
            action_id.clone(),
            Rect::new(rows_area.x, y, rows_area.width, 1),
        ));
        y = y.saturating_add(1);
    }

    action_rects
}

fn assert_root100_normal_a_owner(
    scenario: Scenario,
    theme: Theme,
    color: ColorLevel,
    expected_key: &str,
) -> String {
    assert_eq!(expected_key, ROOT100_NORMAL_A_OWNER_KEY);
    assert_root_normal_a_owner(scenario, theme, color, 100, 30, expected_key)
}

fn assert_remaining_root_normal_a_owner(
    scenario: Scenario,
    theme: Theme,
    color: ColorLevel,
    width: u16,
    height: u16,
    expected_key: &str,
) -> String {
    assert_ne!(expected_key, ROOT100_NORMAL_A_OWNER_KEY);
    assert_root_normal_a_owner(scenario, theme, color, width, height, expected_key)
}

fn assert_root_normal_a_owner(
    scenario: Scenario,
    theme: Theme,
    color: ColorLevel,
    width: u16,
    height: u16,
    expected_key: &str,
) -> String {
    assert_eq!(scenario, Scenario::MonorepoRoot);
    assert_eq!(
        root_normal_a_owner_key(scenario, width, height, &theme, color).as_deref(),
        Some(expected_key),
        "{expected_key} exact normal-A configuration key"
    );
    let frame_area = Rect::new(0, 0, width, height);
    let rows_area_expected = Rect::new(1, 6, width.saturating_sub(2), height.saturating_sub(9));
    let expected_port_row = Rect::new(1, 24, width.saturating_sub(2), 1);
    let expected_frame_cells = usize::from(width).saturating_mul(usize::from(height));

    let mut world = holla_ui::sim::fixtures::world_for(scenario);
    world.seek(4_000);
    assert_eq!(
        world.scenario, scenario,
        "{expected_key} pre-App fixture scenario"
    );
    assert_eq!(
        world.clock.now_ms, 4_000,
        "{expected_key} pre-App fixture time"
    );
    assert!(
        world.clock.running,
        "{expected_key} pre-App fixture clock is running"
    );

    let raw = holla_ui::sim::catalogue::catalogue(&world);
    assert_eq!(
        raw.iter()
            .filter(|action| action.id == "system.port")
            .count(),
        1,
        "{expected_key} unique raw Port action"
    );
    let port = raw
        .iter()
        .find(|action| action.id == "system.port")
        .expect("unique raw Port action is present");
    assert_eq!(
        port.scope,
        Scope::Host,
        "{expected_key} canonical Port host scope"
    );
    assert_eq!(
        port.workdir.as_str(),
        world.cwd.as_str(),
        "{expected_key} Port workdir is the fixture cwd"
    );
    let local_port_pin = Pin {
        path: world.cwd.clone(),
        command: port.command.clone(),
    };
    assert_eq!(
        local_port_pin.path, port.workdir,
        "{expected_key} local Port pin path"
    );
    assert!(
        !world
            .memory
            .hidden_at(&local_port_pin.path, &local_port_pin.command),
        "{expected_key} actual local Port command is unhidden"
    );

    let rows = home_display_rows(&world);
    assert_eq!(rows.len(), 16, "{expected_key} visible action count");
    assert_eq!(
        rows.iter()
            .filter(|(_, action_id)| action_id == "system.port")
            .count(),
        1,
        "{expected_key} unique visible Port action"
    );
    let visual_extent = home_visual_extent(&rows);
    assert_eq!(
        visual_extent, 19,
        "{expected_key} section and action token count"
    );
    let first_section = rows
        .first()
        .map(|(section, _)| section.as_str())
        .expect("nonempty Home model has a first action");
    assert!(
        !first_section.is_empty(),
        "{expected_key} first model section exists"
    );

    let action_rects = root100_normal_a_action_rects(&rows, rows_area_expected);
    assert_eq!(
        action_rects.len(),
        16,
        "{expected_key} every modeled action has a row"
    );
    let unique_action_ids: BTreeSet<_> = action_rects
        .iter()
        .map(|(action_id, _)| action_id.as_str())
        .collect();
    assert_eq!(
        unique_action_ids.len(),
        rows.len(),
        "{expected_key} modeled action IDs are unique"
    );
    let modeled_port_row = action_rects
        .iter()
        .find(|(action_id, _)| action_id == "system.port")
        .map(|(_, row)| *row);
    assert_eq!(
        modeled_port_row,
        Some(expected_port_row),
        "{expected_key} Port row follows its modeled section position"
    );

    let mut ordinary = harness(scenario, theme.clone(), width, height, color);
    ordinary.draw();
    assert_eq!(
        ordinary.runtime().theme(),
        &theme.for_level(color),
        "{expected_key} runtime theme projection"
    );
    assert_eq!(
        ordinary.runtime().clock_ms(),
        0,
        "{expected_key} elapsed runtime clock remains unadvanced"
    );
    assert_eq!(
        ordinary.app().fixture_time_ms(),
        4_000,
        "{expected_key} App time"
    );
    assert_eq!(
        ordinary.focus(),
        Some(HOME_QUERY),
        "{expected_key} query focus"
    );
    assert_eq!(
        ordinary.cursor(),
        Some(Position { x: 3, y: 4 }),
        "{expected_key} query cursor"
    );
    assert!(
        ordinary.diagnostics().is_empty(),
        "{expected_key} diagnostics"
    );
    assert!(ordinary.hover().is_none(), "{expected_key} hover");
    let rows_state = ordinary.state_of(HOME_ROWS);
    assert!(
        !rows_state.contains(StateFlags::HOVERED),
        "{expected_key} Rows are not hovered"
    );
    assert!(
        !rows_state.contains(StateFlags::PRESSED),
        "{expected_key} Rows are not pressed"
    );
    assert!(
        !rows_state.contains(StateFlags::FOCUSED),
        "{expected_key} Rows are not focused"
    );

    let frame = ordinary.buffer();
    assert_eq!(*frame.area(), frame_area, "{expected_key} frame area");
    let mut complete_cells = 0_usize;
    for y in 0..height {
        for x in 0..width {
            assert!(
                frame.cell((x, y)).is_some(),
                "{expected_key} cell ({x},{y})"
            );
            complete_cells = complete_cells.saturating_add(1);
        }
    }
    assert_eq!(
        complete_cells, expected_frame_cells,
        "{expected_key} all frame cells exist"
    );
    let ordinary_text = ordinary.text();
    assert!(
        ordinary_text.contains("holla❯"),
        "{expected_key} brand is visible"
    );
    assert!(
        ordinary_text.contains(scenario.boot_title()),
        "{expected_key} fixture title is visible"
    );

    let actual_key = ordinary.snapshot().named(scenario.name()).key().to_owned();
    assert_eq!(actual_key, expected_key, "named normal-A frame key");
    let rows_area = ordinary
        .area_of(HOME_ROWS)
        .expect("native Home Rows are registered");
    assert_eq!(
        rows_area, rows_area_expected,
        "{expected_key} exact Rows area"
    );
    assert!(
        ordinary
            .area_of_part(HOME_ROWS, PartRef::of(Part::TRACK))
            .is_none(),
        "{expected_key} has no track"
    );
    assert!(
        ordinary
            .area_of_part(HOME_ROWS, PartRef::of(Part::THUMB))
            .is_none(),
        "{expected_key} has no thumb"
    );
    for (action_id, expected_row) in &action_rects {
        assert_eq!(
            ordinary.area_of_part(
                HOME_ROWS,
                PartRef::item(Part::ROW, termrock::ItemKey::text(action_id)),
            ),
            Some(*expected_row),
            "{expected_key} native action row follows section-derived position: {action_id}"
        );
    }
    let native_port_row = ordinary
        .area_of_part(
            HOME_ROWS,
            PartRef::item(Part::ROW, termrock::ItemKey::text("system.port")),
        )
        .expect("native Port row is registered");
    assert_eq!(
        Some(native_port_row),
        modeled_port_row,
        "{expected_key} native Port row follows its model-derived position"
    );
    assert!(
        ordinary
            .row(native_port_row.y)
            .contains(port.title.as_str()),
        "{expected_key} native Port row renders the model title"
    );

    actual_key
}

fn is_border_palette_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    BORDER_PALETTE_CASES.contains(&scenario)
        && (width, height) == (80, 24)
        && *theme == Theme::junie()
        && color == ColorLevel::Ansi256
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

fn home_display_rows(world: &holla_ui::sim::world::World) -> Vec<(String, String)> {
    let recent_first = world.scenario.boot_title() == "Recent here";
    holla_ui::sim::catalogue::sections(world)
        .into_iter()
        .flat_map(|section| {
            let name = if recent_first && section.name == "Suggested here" {
                "Explore"
            } else {
                section.name
            };
            holla_ui::sim::catalogue::visible(&section.actions, None, "", &world.memory, &world.cwd)
                .into_iter()
                .map(move |action| (name.to_owned(), action.id))
        })
        .collect()
}

fn home_visual_extent(rows: &[(String, String)]) -> usize {
    let mut groups = 0_usize;
    let mut previous = None;
    for (section, _) in rows {
        if previous != Some(section.as_str()) {
            groups = groups.saturating_add(1);
            previous = Some(section.as_str());
        }
    }
    rows.len()
        .saturating_add(groups)
        .saturating_add(groups.saturating_sub(1))
}

fn exposed_track_cells(track: Rect, thumb: Rect) -> Vec<(u16, u16)> {
    let mut cells = Vec::new();
    for y in track.y..track.y.saturating_add(track.height) {
        for x in track.x..track.x.saturating_add(track.width) {
            let in_thumb = x >= thumb.x
                && x < thumb.x.saturating_add(thumb.width)
                && y >= thumb.y
                && y < thumb.y.saturating_add(thumb.height);
            if !in_thumb {
                cells.push((x, y));
            }
        }
    }
    cells
}

fn assert_current_model_border_palette_locality(scenario: Scenario) {
    let mut world = holla_ui::sim::fixtures::world_for(scenario);
    world.seek(4_000);

    let raw = holla_ui::sim::catalogue::catalogue(&world);
    assert_eq!(
        raw.iter()
            .filter(|action| action.id == "system.port")
            .count(),
        1,
        "canonical Port remains in the raw catalogue for {scenario:?}"
    );
    let rows = home_display_rows(&world);
    assert_eq!(
        rows.iter().filter(|(_, id)| id == "system.port").count(),
        1,
        "canonical Port remains in the normal Home projection for {scenario:?}"
    );

    let content_extent = home_visual_extent(&rows);
    assert!(content_extent > 0, "nonempty Home model for {scenario:?}");
    if scenario == Scenario::RustDirty {
        assert_eq!(rows.len(), 16, "normal RustDirty action count");
        assert_eq!(content_extent, 21, "normal RustDirty visual row count");
    }

    let mut h = harness(scenario, Theme::junie(), 80, 24, ColorLevel::Ansi256);
    let expected_rows = Rect::new(1, 6, 78, 15);
    let expected_track = Rect::new(78, 6, 1, 15);
    let rows_area = h.area_of(HOME_ROWS);
    let track = h.area_of_part(HOME_ROWS, PartRef::of(Part::TRACK));
    let thumb = h.area_of_part(HOME_ROWS, PartRef::of(Part::THUMB));
    assert!(rows_area.is_some(), "native Home rows for {scenario:?}");
    assert!(track.is_some(), "native Home track for {scenario:?}");
    assert!(thumb.is_some(), "native Home thumb for {scenario:?}");
    let (Some(rows_area), Some(track), Some(thumb)) = (rows_area, track, thumb) else {
        return;
    };

    assert_eq!(
        rows_area, expected_rows,
        "native Home viewport for {scenario:?}"
    );
    assert_eq!(track, expected_track, "native Home track for {scenario:?}");
    let viewport = usize::from(expected_rows.height);
    let track_len = usize::from(expected_track.height);
    let thumb_len = viewport
        .saturating_mul(track_len)
        .checked_div(content_extent)
        .unwrap_or_default()
        .max(1)
        .min(track_len);
    if scenario == Scenario::RustDirty {
        assert_eq!(viewport, 15, "RustDirty native viewport height");
        assert_eq!(thumb_len, 10, "RustDirty model-derived thumb height");
    }
    let expected_thumb = Rect::new(
        expected_track.x,
        expected_track.y,
        expected_track.width,
        u16::try_from(thumb_len).unwrap_or_default(),
    );
    assert_eq!(
        thumb, expected_thumb,
        "model-derived Home thumb for {scenario:?}"
    );
    assert_eq!(
        thumb.y, expected_track.y,
        "initial offset is zero for {scenario:?}"
    );

    let exposed = exposed_track_cells(expected_track, expected_thumb);
    if scenario == Scenario::RustDirty {
        assert_eq!(
            exposed,
            (16..=20).map(|y| (78, y)).collect::<Vec<_>>(),
            "RustDirty model-derived exposed track rows"
        );
    }

    let current = h.buffer().clone();
    let current_cursor = h.cursor();
    assert_eq!(current.area, Rect::new(0, 0, 80, 24));
    assert_eq!(current_cursor, Some(Position { x: 3, y: 4 }));
    let current_track_cell = expected_cell(
        "┆",
        Color::Indexed(238),
        Color::Indexed(16),
        Modifier::empty(),
    );
    let changed_track_cell = expected_cell(
        "┆",
        Color::Indexed(235),
        Color::Indexed(16),
        Modifier::empty(),
    );
    for &(x, y) in &exposed {
        assert_eq!(
            current.cell((x, y)),
            Some(&current_track_cell),
            "current track cell at ({x}, {y})"
        );
    }
    let thumb_cell = expected_cell(
        "┃",
        Color::Indexed(245),
        Color::Indexed(16),
        Modifier::empty(),
    );
    for y in thumb.y..thumb.y.saturating_add(thumb.height) {
        assert_eq!(
            current.cell((thumb.x, y)),
            Some(&thumb_cell),
            "thumb cell at ({}, {y})",
            thumb.x
        );
    }

    let mut changed_theme = h.runtime().theme().clone();
    assert_eq!(
        changed_theme.color.border_subtle,
        Color::Indexed(238),
        "only the authored Junie BorderSubtle source token is replayed"
    );
    changed_theme.color.border_subtle = Color::Indexed(235);
    h.runtime_mut().set_theme(changed_theme);
    h.draw();
    let changed = h.buffer();

    assert_eq!(changed.area, current.area);
    assert_eq!(h.area_of(HOME_ROWS), Some(expected_rows));
    assert_eq!(
        h.area_of_part(HOME_ROWS, PartRef::of(Part::TRACK)),
        Some(expected_track)
    );
    assert_eq!(
        h.area_of_part(HOME_ROWS, PartRef::of(Part::THUMB)),
        Some(expected_thumb)
    );
    assert_eq!(h.cursor(), current_cursor, "cursor after palette replay");
    assert_only_expected_cells_differ(&current, changed, &exposed);
    for &(x, y) in &exposed {
        assert_eq!(
            changed.cell((x, y)),
            Some(&changed_track_cell),
            "changed track cell at ({x}, {y})"
        );
    }
    for y in thumb.y..thumb.y.saturating_add(thumb.height) {
        assert_eq!(
            changed.cell((thumb.x, y)),
            Some(&thumb_cell),
            "unchanged thumb cell at ({}, {y})",
            thumb.x
        );
    }
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
        && (((width, height) == (100, 30)
            && (*theme == Theme::junie() || *theme == Theme::paper())
            && COLORS.contains(&color))
            || ((width, height) == (80, 24)
                && *theme == Theme::paper()
                && color == ColorLevel::Mono))
}

fn historical_80_overflow_faint_color(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> Option<Color> {
    if scenario != Scenario::ParityDiscovery || (width, height) != (80, 24) {
        return None;
    }

    // Source 830ca083 authored overflow as Custom(RGB 115,115,115). These are its exact
    // projected Faint colors for the four independently measured built-in/theme-tier pairs.
    if *theme == Theme::junie() && color == ColorLevel::Mono {
        Some(Color::Black)
    } else if *theme == Theme::paper() {
        match color {
            ColorLevel::TrueColor => Some(Color::Rgb(115, 115, 115)),
            ColorLevel::Ansi256 => Some(Color::Indexed(242)),
            ColorLevel::Ansi16 => Some(Color::Gray),
            _ => None,
        }
    } else {
        None
    }
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

fn projected_historical_80_faint_harness(
    theme: Theme,
    color: ColorLevel,
    historical_faint: Color,
) -> Harness<App> {
    let mut projected_theme = if color == ColorLevel::TrueColor {
        theme
    } else {
        theme.downgrade(color)
    };
    projected_theme.color.fg[FgStep::Faint.index()] = historical_faint;

    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
    let initial = SimulationMoment::from_millis(app.fixture_time_ms() as u64);
    Harness::new_with_feedback_clock(
        app,
        projected_theme,
        80,
        24,
        FeedbackClock::Simulation { initial },
    )
}

fn assert_historical_80_overflow_faint_replay(theme: Theme, color: ColorLevel) {
    const POSITION: (u16, u16) = (77, 23);
    let historical_faint =
        historical_80_overflow_faint_color(Scenario::ParityDiscovery, 80, 24, &theme, color)
            .expect("reviewed exact built-in theme and 80x24 tier");
    let current_faint = expected_overflow_faint(&theme, color).expect("reviewed current tier");
    let historical_theme_name = if theme == Theme::junie() {
        "junie"
    } else {
        "paper"
    };

    let current = harness(Scenario::ParityDiscovery, theme.clone(), 80, 24, color);
    let historical = projected_historical_80_faint_harness(theme, color, historical_faint);

    assert_eq!(current.buffer().area, Rect::new(0, 0, 80, 24));
    assert_eq!(historical.buffer().area, current.buffer().area);
    assert_eq!(current.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(historical.cursor(), current.cursor());
    assert_foreground_only_replay(
        current.buffer(),
        historical.buffer(),
        &[POSITION],
        current_faint,
        historical_faint,
    );

    let current_cell = current
        .buffer()
        .cell(POSITION)
        .expect("current clipped marker inside frame");
    assert_eq!(
        current_cell,
        &expected_cell(
            "…",
            current_faint,
            black_for_level(color),
            Modifier::empty(),
        )
    );
    let historical_cell = historical
        .buffer()
        .cell(POSITION)
        .expect("historical clipped marker inside frame");
    assert_eq!(
        historical_cell,
        &expected_cell(
            "…",
            historical_faint,
            black_for_level(color),
            Modifier::empty(),
        )
    );
    assert!(current.row(23).trim_end().ends_with('…'));
    assert!(historical.row(23).trim_end().ends_with('…'));
    let color_label = match color {
        ColorLevel::TrueColor => "truecolor",
        ColorLevel::Ansi256 => "256",
        ColorLevel::Ansi16 => "16",
        ColorLevel::Mono => "mono",
        _ => "other",
    };
    let historical_scene = Scene::from_buffer(
        Scenario::ParityDiscovery.name(),
        historical_theme_name,
        color,
        historical.buffer().clone(),
    );
    assert_eq!(
        historical_scene.key(),
        format!("parity-discovery 80 24 {historical_theme_name} {color_label}"),
        "the projected test profile retains the original built-in baseline identity"
    );
    historical_scene.assert_against(&BASELINE);
}

#[test]
fn holla_visual_baseline() {
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    if is_border_palette_case(scenario, width, height, &theme, color) {
                        assert_current_model_border_palette_locality(scenario);
                        continue;
                    }
                    if is_historical_ansi16_faint_case(scenario, width, height, &theme, color) {
                        assert_historical_ansi16_faint_replay(scenario, width, height);
                        continue;
                    }
                    if historical_80_overflow_faint_color(scenario, width, height, &theme, color)
                        .is_some()
                    {
                        assert_historical_80_overflow_faint_replay(theme.clone(), color);
                        continue;
                    }
                    if is_historical_overflow_faint_case(scenario, width, height, &theme, color) {
                        if (width, height) == (80, 24) {
                            assert_original_paper_mono_80_historical_replay();
                        } else {
                            assert_historical_overflow_faint_replay(theme.clone(), color);
                        }
                        continue;
                    }
                    if let Some(expected_key) =
                        tracked_port_owner_key(scenario, width, height, &theme, color)
                    {
                        // The private tracked_port_owner unit test owns the actual B hide and
                        // full-cell A/B comparison. This shared normal-A helper verifies the
                        // named matrix input, public model projection and native geometry.
                        assert_eq!(
                            assert_tracked_port_owner_current_a(
                                scenario,
                                theme.clone(),
                                color,
                                expected_key,
                            ),
                            expected_key,
                            "normal-A owner routing retains the exact key"
                        );
                        continue;
                    }
                    if let Some(expected_key) =
                        root100_normal_a_owner_key(scenario, width, height, &theme, color)
                    {
                        // The private `root_100_nonoverflow_port_owner` module owns
                        // `monorepo_root_100_junie_truecolor_port_hide_preserves_held_nonoverflow_frame` and its B,
                        // full-cell locality, and painter checks.
                        assert_eq!(
                            assert_root100_normal_a_owner(
                                scenario,
                                theme.clone(),
                                color,
                                expected_key,
                            ),
                            expected_key,
                            "normal-A Root100 route retains the exact key"
                        );
                        continue;
                    }
                    if let Some(expected_key) =
                        remaining_root_normal_a_owner_key(scenario, width, height, &theme, color)
                    {
                        // The private `root_100_nonoverflow_port_owner` RootCase family owns B,
                        // full-cell locality, and painter checks for these remaining Root configs.
                        let actual_key = assert_remaining_root_normal_a_owner(
                            scenario,
                            theme.clone(),
                            color,
                            width,
                            height,
                            &expected_key,
                        );
                        assert_eq!(
                            actual_key, expected_key,
                            "normal-A remaining Root route retains the exact key"
                        );
                        continue;
                    }
                    let ordinary = harness(scenario, theme.clone(), width, height, color);
                    let scene = ordinary.snapshot().named(scenario.name());
                    if scenario == Scenario::MonorepoRoot
                        && width == 80
                        && height == 24
                        && theme == Theme::junie()
                        && color == ColorLevel::TrueColor
                    {
                        // `monorepo_root_truecolor_port_visibility_ab_characterization` owns
                        // the held digest and checks this normal model's full-cell locality.
                        assert_eq!(
                            scene.key(),
                            "monorepo-root 80 24 junie truecolor",
                            "the relocated key is the exact named historical entry"
                        );
                        assert_eq!(
                            ordinary.cursor(),
                            Some(Position { x: 3, y: 4 }),
                            "ordinary MonorepoRoot TrueColor cursor"
                        );
                        assert_eq!(
                            ordinary.area_of(HOME_ROWS),
                            Some(Rect::new(1, 6, 78, 15)),
                            "ordinary Home rows geometry"
                        );
                        assert_eq!(
                            ordinary.area_of_part(HOME_ROWS, PartRef::of(Part::TRACK)),
                            Some(Rect::new(78, 6, 1, 15)),
                            "ordinary Home track geometry"
                        );
                        assert!(
                            ordinary
                                .area_of_part(HOME_ROWS, PartRef::of(Part::THUMB))
                                .is_some(),
                            "ordinary Home thumb is registered"
                        );
                    } else if scenario == Scenario::MonorepoRoot
                        && width == 80
                        && height == 24
                        && theme == Theme::junie()
                        && color == ColorLevel::Ansi16
                    {
                        // `monorepo_root_ansi16_port_hide_ab_diagnostic` owns the held digest
                        // and checks this normal model's full-cell locality.
                        assert_eq!(
                            scene.key(),
                            "monorepo-root 80 24 junie 16",
                            "the relocated key is the exact named historical entry"
                        );
                        assert_eq!(
                            ordinary.cursor(),
                            Some(Position { x: 3, y: 4 }),
                            "ordinary MonorepoRoot ANSI16 cursor"
                        );
                        assert_eq!(
                            ordinary.area_of(HOME_ROWS),
                            Some(Rect::new(1, 6, 78, 15)),
                            "ordinary Home rows geometry"
                        );
                        assert_eq!(
                            ordinary.area_of_part(HOME_ROWS, PartRef::of(Part::TRACK)),
                            Some(Rect::new(78, 6, 1, 15)),
                            "ordinary Home track geometry"
                        );
                        assert!(
                            ordinary
                                .area_of_part(HOME_ROWS, PartRef::of(Part::THUMB))
                                .is_some(),
                            "ordinary Home thumb is registered"
                        );
                    } else {
                        scene.assert_against(&BASELINE);
                    }
                }
            }
        }
    }
}

#[test]
fn holla_tracked_port_normal_a_owner_matrix_is_exact() {
    let mut matrix_inputs = 0_usize;
    let mut selected_keys = BTreeSet::new();
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    matrix_inputs = matrix_inputs.saturating_add(1);
                    let key = tracked_port_owner_key(scenario, width, height, &theme, color);

                    let has_specialized_owner =
                        is_border_palette_case(scenario, width, height, &theme, color)
                            || is_historical_ansi16_faint_case(
                                scenario, width, height, &theme, color,
                            )
                            || historical_80_overflow_faint_color(
                                scenario, width, height, &theme, color,
                            )
                            .is_some()
                            || is_historical_overflow_faint_case(
                                scenario, width, height, &theme, color,
                            );
                    assert!(
                        key.is_none() || !has_specialized_owner,
                        "tracked owner does not overlap a specialized or faint protocol: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );

                    let is_root_truecolor_relocation = scenario == Scenario::MonorepoRoot
                        && (width, height) == (80, 24)
                        && theme == Theme::junie()
                        && color == ColorLevel::TrueColor;
                    let is_root_ansi16_relocation = scenario == Scenario::MonorepoRoot
                        && (width, height) == (80, 24)
                        && theme == Theme::junie()
                        && color == ColorLevel::Ansi16;
                    assert!(
                        key.is_none()
                            || !(is_root_truecolor_relocation || is_root_ansi16_relocation),
                        "tracked owner remains disjoint from the two relocated Root keys"
                    );

                    if let Some(expected_key) = key {
                        assert!(
                            !EXISTING_CURRENT_A_OWNER_KEYS.contains(&expected_key),
                            "tracked key has a distinct existing owner: {expected_key}"
                        );
                        let actual_key = assert_tracked_port_owner_current_a(
                            scenario,
                            theme.clone(),
                            color,
                            expected_key,
                        );
                        assert!(
                            selected_keys.insert(actual_key.clone()),
                            "matrix selects each tracked key once: {actual_key}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        matrix_inputs, 1_120,
        "complete Scenario × size × theme × tier matrix"
    );
    assert_eq!(
        selected_keys.len(),
        TRACKED_PORT_OWNER_KEYS.len(),
        "all eleven tracked owner keys are unique"
    );
    let expected_keys: BTreeSet<_> = TRACKED_PORT_OWNER_KEYS
        .iter()
        .map(|key| (*key).to_owned())
        .collect();
    assert_eq!(expected_keys.len(), TRACKED_PORT_OWNER_KEYS.len());
    assert_eq!(selected_keys, expected_keys, "exact tracked owner key set");
    let existing_owners: BTreeSet<_> = EXISTING_CURRENT_A_OWNER_KEYS
        .iter()
        .map(|key| (*key).to_owned())
        .collect();
    assert!(
        selected_keys.is_disjoint(&existing_owners),
        "tracked owner key set is disjoint from all six existing current-A owners"
    );
}

#[test]
fn holla_root100_normal_a_owner_selector_is_exact() {
    let junie = Theme::junie();
    let paper = Theme::paper();
    assert_eq!(
        root100_normal_a_owner_key(
            Scenario::MonorepoRoot,
            100,
            30,
            &junie,
            ColorLevel::TrueColor,
        ),
        Some(ROOT100_NORMAL_A_OWNER_KEY)
    );
    assert!(
        root100_normal_a_owner_key(
            Scenario::MonorepoRoot,
            99,
            30,
            &junie,
            ColorLevel::TrueColor,
        )
        .is_none()
    );
    assert!(
        root100_normal_a_owner_key(
            Scenario::MonorepoRoot,
            100,
            29,
            &junie,
            ColorLevel::TrueColor,
        )
        .is_none()
    );
    assert!(
        root100_normal_a_owner_key(
            Scenario::MonorepoRoot,
            100,
            30,
            &paper,
            ColorLevel::TrueColor,
        )
        .is_none()
    );
    assert!(
        root100_normal_a_owner_key(Scenario::MonorepoRoot, 100, 30, &junie, ColorLevel::Ansi256,)
            .is_none()
    );
    assert!(
        root100_normal_a_owner_key(
            Scenario::MonorepoChild,
            100,
            30,
            &junie,
            ColorLevel::TrueColor,
        )
        .is_none()
    );

    let mut matrix_inputs = 0_usize;
    let mut selected_keys = BTreeSet::new();
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    matrix_inputs = matrix_inputs.saturating_add(1);
                    let key = root100_normal_a_owner_key(scenario, width, height, &theme, color);
                    let tracked_key =
                        tracked_port_owner_key(scenario, width, height, &theme, color);
                    let has_specialized_owner =
                        is_border_palette_case(scenario, width, height, &theme, color)
                            || is_historical_ansi16_faint_case(
                                scenario, width, height, &theme, color,
                            )
                            || historical_80_overflow_faint_color(
                                scenario, width, height, &theme, color,
                            )
                            .is_some()
                            || is_historical_overflow_faint_case(
                                scenario, width, height, &theme, color,
                            );
                    assert!(
                        key.is_none() || tracked_key.is_none(),
                        "Root100 owner remains disjoint from the eleven tracked owners: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );
                    assert!(
                        key.is_none() || !has_specialized_owner,
                        "Root100 owner remains disjoint from specialized and faint owners: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );
                    if let Some(owner_key) = key {
                        assert_eq!(owner_key, ROOT100_NORMAL_A_OWNER_KEY);
                        assert!(
                            !TRACKED_PORT_OWNER_KEYS.contains(&owner_key),
                            "Root100 key is distinct from all eleven tracked keys"
                        );
                        assert!(
                            !EXISTING_CURRENT_A_OWNER_KEYS.contains(&owner_key),
                            "Root100 key is distinct from all six existing current-A keys"
                        );
                        let actual_key = assert_root100_normal_a_owner(
                            scenario,
                            theme.clone(),
                            color,
                            owner_key,
                        );
                        assert_eq!(
                            actual_key, owner_key,
                            "normal-A helper retains the exact Root100 key"
                        );
                        assert!(
                            selected_keys.insert(actual_key),
                            "matrix selects Root100 key once"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        matrix_inputs, 1_120,
        "complete Scenario × size × theme × tier matrix"
    );
    assert_eq!(
        selected_keys.len(),
        1,
        "exactly one Root100 owner selection"
    );
    assert_eq!(
        selected_keys,
        BTreeSet::from([ROOT100_NORMAL_A_OWNER_KEY.to_owned()]),
        "exact Root100 normal-A owner key set"
    );
}

#[test]
fn holla_remaining_root_normal_a_owner_matrix_is_exact() {
    let junie = Theme::junie();
    let paper = Theme::paper();
    for (scenario, width, height, theme, color) in [
        (
            Scenario::MonorepoRoot,
            99,
            30,
            &junie,
            ColorLevel::TrueColor,
        ),
        (
            Scenario::MonorepoRoot,
            100,
            29,
            &junie,
            ColorLevel::TrueColor,
        ),
        (
            Scenario::MonorepoRoot,
            101,
            30,
            &junie,
            ColorLevel::TrueColor,
        ),
        (
            Scenario::MonorepoRoot,
            100,
            31,
            &junie,
            ColorLevel::TrueColor,
        ),
        (Scenario::MonorepoRoot, 120, 39, &paper, ColorLevel::Ansi256),
        (
            Scenario::MonorepoChild,
            120,
            40,
            &junie,
            ColorLevel::TrueColor,
        ),
    ] {
        assert!(
            remaining_root_normal_a_owner_key(scenario, width, height, theme, color).is_none(),
            "neighboring config is not a remaining Root owner: {scenario:?} {width}x{height} {theme:?} {color:?}"
        );
    }
    assert!(
        remaining_root_normal_a_owner_key(
            Scenario::MonorepoRoot,
            100,
            30,
            &junie,
            ColorLevel::TrueColor,
        )
        .is_none(),
        "the exact Root100 owner remains on its existing selector"
    );

    let expected_keys: BTreeSet<_> = REMAINING_ROOT_NORMAL_A_OWNER_KEYS
        .iter()
        .map(|key| (*key).to_owned())
        .collect();
    assert_eq!(
        expected_keys.len(),
        REMAINING_ROOT_NORMAL_A_OWNER_KEYS.len()
    );

    let mut matrix_inputs = 0_usize;
    let mut selected_keys = BTreeSet::new();
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    matrix_inputs = matrix_inputs.saturating_add(1);
                    let key =
                        remaining_root_normal_a_owner_key(scenario, width, height, &theme, color);
                    let root100_key =
                        root100_normal_a_owner_key(scenario, width, height, &theme, color);
                    let tracked_key =
                        tracked_port_owner_key(scenario, width, height, &theme, color);
                    let has_specialized_owner =
                        is_border_palette_case(scenario, width, height, &theme, color)
                            || is_historical_ansi16_faint_case(
                                scenario, width, height, &theme, color,
                            )
                            || historical_80_overflow_faint_color(
                                scenario, width, height, &theme, color,
                            )
                            .is_some()
                            || is_historical_overflow_faint_case(
                                scenario, width, height, &theme, color,
                            );
                    assert!(
                        key.is_none() || root100_key.is_none(),
                        "remaining Root owner excludes Root100: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );
                    assert!(
                        key.is_none() || tracked_key.is_none(),
                        "remaining Root owner excludes all eleven tracked routes: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );
                    assert!(
                        key.as_ref().is_none_or(|key| {
                            !EXISTING_CURRENT_A_OWNER_KEYS.contains(&key.as_str())
                        }),
                        "remaining Root owner excludes all six older routes: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );
                    assert!(
                        key.is_none() || !has_specialized_owner,
                        "remaining Root owner excludes specialized and faint protocols: {scenario:?} {width}x{height} {theme:?} {color:?}"
                    );

                    if let Some(owner_key) = key {
                        assert!(
                            expected_keys.contains(&owner_key),
                            "selected key belongs to independent finite key set: {owner_key}"
                        );
                        assert!(
                            !TRACKED_PORT_OWNER_KEYS.contains(&owner_key.as_str()),
                            "remaining Root key is distinct from all eleven tracked keys"
                        );
                        assert!(
                            !EXISTING_CURRENT_A_OWNER_KEYS.contains(&owner_key.as_str()),
                            "remaining Root key is distinct from all six older keys"
                        );
                        let actual_key = assert_remaining_root_normal_a_owner(
                            scenario,
                            theme.clone(),
                            color,
                            width,
                            height,
                            &owner_key,
                        );
                        assert_eq!(
                            actual_key, owner_key,
                            "normal-A helper retains the exact remaining Root key"
                        );
                        assert!(
                            selected_keys.insert(actual_key.clone()),
                            "matrix selects each remaining Root key once: {actual_key}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        matrix_inputs, 1_120,
        "complete Scenario × size × theme × tier matrix"
    );
    assert_eq!(
        selected_keys.len(),
        REMAINING_ROOT_NORMAL_A_OWNER_KEYS.len(),
        "all twenty-three remaining Root owner keys are unique"
    );
    assert_eq!(
        selected_keys, expected_keys,
        "exact remaining Root normal-A owner key set"
    );
}

#[test]
fn holla_current_model_border_subtle_palette_is_local_for_four_scenarios() {
    // The four held digests are preserved by the matching historical fixtures:
    // rust_dirty_hidden_port_retains_historical_border_frame,
    // monorepo_root_hidden_port_retains_historical_border_frame,
    // monorepo_child_hidden_port_retains_historical_border_frame, and
    // activities_multi_hidden_port_retains_historical_border_frame. These normal-model
    // cases keep Port visible and independently check current native geometry/locality.
    for scenario in BORDER_PALETTE_CASES {
        assert_current_model_border_palette_locality(scenario);
    }
}

#[test]
fn holla_historical_faint_replays_match_the_seventeen_reviewed_frames() {
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
    assert_original_paper_mono_80_historical_replay();
}

#[test]
fn holla_historical_projected_faint_replays_match_the_four_reviewed_80x24_frames() {
    let cases = [
        (Theme::junie(), ColorLevel::Mono),
        (Theme::paper(), ColorLevel::TrueColor),
        (Theme::paper(), ColorLevel::Ansi256),
        (Theme::paper(), ColorLevel::Ansi16),
    ];
    assert_eq!(cases.len(), 4);
    for (theme, color) in cases {
        assert_historical_80_overflow_faint_replay(theme, color);
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
    assert!(is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        80,
        24,
        &paper,
        ColorLevel::Mono
    ));
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
    let historical_80_cases = [
        (Theme::junie(), ColorLevel::Mono, Color::Black),
        (
            Theme::paper(),
            ColorLevel::TrueColor,
            Color::Rgb(115, 115, 115),
        ),
        (Theme::paper(), ColorLevel::Ansi256, Color::Indexed(242)),
        (Theme::paper(), ColorLevel::Ansi16, Color::Gray),
    ];
    assert_eq!(historical_80_cases.len(), 4);
    for (theme, color, expected) in &historical_80_cases {
        assert_eq!(
            historical_80_overflow_faint_color(Scenario::ParityDiscovery, 80, 24, theme, *color,),
            Some(*expected)
        );
    }
    for (theme, color, _) in &historical_80_cases {
        for (scenario, width, height) in [
            (Scenario::ParityHistory, 80, 24),
            (Scenario::ParityDiscovery, 81, 24),
            (Scenario::ParityDiscovery, 80, 25),
        ] {
            assert_eq!(
                historical_80_overflow_faint_color(scenario, width, height, theme, *color),
                None
            );
        }
    }
    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        assert_eq!(
            historical_80_overflow_faint_color(Scenario::ParityDiscovery, 80, 24, &junie, color,),
            (color == ColorLevel::Mono).then_some(Color::Black)
        );
        assert_eq!(
            historical_80_overflow_faint_color(Scenario::ParityDiscovery, 80, 24, &paper, color,),
            match color {
                ColorLevel::TrueColor => Some(Color::Rgb(115, 115, 115)),
                ColorLevel::Ansi256 => Some(Color::Indexed(242)),
                ColorLevel::Ansi16 => Some(Color::Gray),
                ColorLevel::Mono => None,
                _ => None,
            }
        );
    }
    let mut changed_paper = paper.clone();
    changed_paper.color.fg[0] = Color::Green;
    assert_eq!(
        historical_80_overflow_faint_color(
            Scenario::ParityDiscovery,
            80,
            24,
            &changed_paper,
            ColorLevel::TrueColor,
        ),
        None
    );
    assert_eq!(
        historical_80_overflow_faint_color(
            Scenario::ParityDiscovery,
            80,
            24,
            &Theme::from_tokens(paper.color),
            ColorLevel::TrueColor,
        ),
        None,
        "token-only themes have no historical recipe provenance"
    );
    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
    ] {
        assert!(!is_historical_overflow_faint_case(
            Scenario::ParityDiscovery,
            80,
            24,
            &paper,
            color
        ));
    }
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        81,
        24,
        &paper,
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
        Scenario::ParityHistory,
        80,
        24,
        &paper,
        ColorLevel::Mono
    ));
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        100,
        30,
        &Theme::from_tokens(junie.color),
        ColorLevel::TrueColor
    ));
    assert!(!is_historical_overflow_faint_case(
        Scenario::ParityDiscovery,
        80,
        24,
        &Theme::from_tokens(paper.color),
        ColorLevel::Mono
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

fn original_unconfigured_parity_harness(
    theme: Theme,
    width: u16,
    height: u16,
    color: ColorLevel,
) -> Harness<App> {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000);
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

fn assert_original_paper_mono_80_historical_replay() {
    const POSITION: (u16, u16) = (77, 23);
    let current = harness(
        Scenario::ParityDiscovery,
        Theme::paper(),
        80,
        24,
        ColorLevel::Mono,
    );
    let historical = original_unconfigured_parity_harness(Theme::paper(), 80, 24, ColorLevel::Mono);

    assert_eq!(current.buffer().area, Rect::new(0, 0, 80, 24));
    assert_eq!(historical.buffer().area, current.buffer().area);
    assert_eq!(current.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(historical.cursor(), current.cursor());
    assert_only_overflow_diff(current.buffer(), historical.buffer(), POSITION);

    let visible = current
        .buffer()
        .cell(POSITION)
        .expect("current clipped marker inside frame");
    assert_eq!(
        visible,
        &expected_cell("…", Color::White, Color::Black, Modifier::empty())
    );
    assert!(
        termrock::theme::contrast_ratio(visible.fg, visible.bg).is_some_and(|ratio| ratio >= 3.0),
        "the configured current marker remains readable"
    );

    let old = historical
        .buffer()
        .cell(POSITION)
        .expect("historical clipped marker inside frame");
    assert_eq!(
        old,
        &expected_cell("…", Color::Black, Color::Black, Modifier::empty()),
        "the original unconfigured Paper Mono renderer leaves the marker invisible"
    );
    assert!(current.row(23).trim_end().ends_with('…'));
    assert!(historical.row(23).trim_end().ends_with('…'));

    historical
        .snapshot()
        .named(Scenario::ParityDiscovery.name())
        .assert_against(&BASELINE);
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
