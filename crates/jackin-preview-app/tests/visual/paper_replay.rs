use super::*;

const CURRENT_PAPER_SECONDARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Rgb(222, 217, 208), bg: Rgb(222, 217, 208), underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const HISTORICAL_PAPER_SECONDARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Rgb(27, 26, 23), bg: Rgb(222, 217, 208), underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const CURRENT_PAPER_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Rgb(59, 91, 219), bg: Rgb(59, 91, 219), underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const HISTORICAL_PAPER_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Rgb(251, 250, 248), bg: Rgb(59, 91, 219), underline_color: Reset, modifier: BOLD, diff_option: None, skip: false }"#;

pub(super) fn is_historical_returning_paper_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    scenario == Scenario::Returning
        && (width, height) == (100, 30)
        && *theme == Theme::paper()
        && color == ColorLevel::TrueColor
}

fn apply_historical_paper_gutter_recipe(harness: &mut Harness<App>) {
    let mut theme = harness.runtime().theme().clone();
    theme.design.glyphs.set(GlyphRole::SwitchOff, " ");

    let button = theme.recipes.get_mut(Family::BUTTON);
    assert_eq!(
        button.default_variant,
        Variant::SECONDARY,
        "Paper's ordinary button default remains SECONDARY"
    );

    let secondary_parts = button.variant_mut(Variant::SECONDARY);
    assert!(
        secondary_parts.get(Part::GUTTER).is_some(),
        "Paper secondary button keeps its authored gutter state rules"
    );
    let secondary_gutter = secondary_parts.entry(Part::GUTTER);
    secondary_gutter.glyph = Slot::Set(GlyphRole::SwitchOff);
    secondary_gutter.base = StylePatch::new()
        .set_fg(Role::Fg(FgStep::Primary))
        .set_bg(Role::Surface(Surface::Overlay));

    let primary_parts = button.variant_mut(Variant::PRIMARY);
    assert!(
        primary_parts.get(Part::GUTTER).is_some(),
        "Paper primary button keeps its authored gutter state rules"
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

fn assert_current_returning_paper_tuples(buffer: &Buffer) {
    for &(x, y) in &EXPECTED_POSITIONS[..4] {
        assert_complete_cell_tuple(
            buffer,
            (x, y),
            Color::Rgb(222, 217, 208),
            Color::Rgb(222, 217, 208),
            Modifier::empty(),
            CURRENT_PAPER_SECONDARY_DEBUG,
            "current Paper secondary header gutter",
        );
    }
    for &(x, y) in &EXPECTED_POSITIONS[4..] {
        assert_complete_cell_tuple(
            buffer,
            (x, y),
            Color::Rgb(59, 91, 219),
            Color::Rgb(59, 91, 219),
            Modifier::empty(),
            CURRENT_PAPER_PRIMARY_DEBUG,
            "current Paper primary action gutter",
        );
    }
}

fn assert_returning_paper_pair(current: &Buffer, historical: &Buffer) {
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(historical.area, current.area);
    assert_eq!(
        u32::from(current.area.width).checked_mul(u32::from(current.area.height)),
        Some(3_000)
    );

    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = cell_at(current, x, y);
            let historical_cell = cell_at(historical, x, y);
            let position = (x, y);
            if EXPECTED_POSITIONS.contains(&position) {
                assert_ne!(
                    current_cell, historical_cell,
                    "Paper gutter must be a measured difference at ({x}, {y})"
                );
            } else {
                assert_eq!(
                    current_cell, historical_cell,
                    "logical/native Paper Cell changed outside the six measured gutters at ({x}, {y})"
                );
                assert_eq!(
                    format!("{current_cell:?}"),
                    format!("{historical_cell:?}"),
                    "native Paper Cell Debug changed outside the six measured gutters at ({x}, {y})"
                );
            }
        }
    }

    for &(x, y) in &EXPECTED_POSITIONS[..4] {
        assert_complete_cell_tuple(
            current,
            (x, y),
            Color::Rgb(222, 217, 208),
            Color::Rgb(222, 217, 208),
            Modifier::empty(),
            CURRENT_PAPER_SECONDARY_DEBUG,
            "current Paper secondary header gutter",
        );
        assert_complete_cell_tuple(
            historical,
            (x, y),
            Color::Rgb(27, 26, 23),
            Color::Rgb(222, 217, 208),
            Modifier::empty(),
            HISTORICAL_PAPER_SECONDARY_DEBUG,
            "historical Paper secondary header gutter",
        );
    }
    for &(x, y) in &EXPECTED_POSITIONS[4..] {
        assert_complete_cell_tuple(
            current,
            (x, y),
            Color::Rgb(59, 91, 219),
            Color::Rgb(59, 91, 219),
            Modifier::empty(),
            CURRENT_PAPER_PRIMARY_DEBUG,
            "current Paper primary action gutter",
        );
        assert_complete_cell_tuple(
            historical,
            (x, y),
            Color::Rgb(251, 250, 248),
            Color::Rgb(59, 91, 219),
            Modifier::BOLD,
            HISTORICAL_PAPER_PRIMARY_DEBUG,
            "historical Paper primary action gutter",
        );
    }
}

fn assert_returning_paper_context(harness: &Harness<App>, phase: &str) {
    assert_eq!(
        harness.app().route(),
        jackin_preview_app::Route::Manager,
        "{phase}: actual route remains Manager"
    );
    assert_eq!(
        harness.focus(),
        Some(jackin_preview_app::MANAGER),
        "{phase}: actual focus owner remains MANAGER"
    );
    assert!(
        !harness.focus_visible(),
        "{phase}: Manager frame does not expose keyboard focus"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{phase}: rendering reports no diagnostics"
    );
}

pub(super) fn assert_historical_returning_paper_replay() {
    let app = App::for_scenario(Scenario::Returning, Motion::Paused);
    let mut harness = Harness::new(app, Theme::paper(), 100, 30).with_color(ColorLevel::TrueColor);
    harness.draw();
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::TrueColor,
        "actual Runtime capability is TrueColor before historical adaptation"
    );
    assert_returning_paper_context(&harness, "ordinary current frame");

    let current_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(current_scene.key(), "returning 100 30 paper truecolor");
    assert_eq!(
        format!("{:016x}", current_scene.digest()),
        "f7243df264b9c7d0",
        "unadapted current Paper frame is asserted before historical recipe application"
    );

    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();
    let current_cursor_bar = harness.cursor_bar();
    let current_focus = harness.runtime().focus();
    assert!(
        current_focus.is_some(),
        "current Paper frame retains its actual focused owner"
    );
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(current_cursor, None);
    assert!(!current_cursor_bar);
    assert_current_returning_paper_tuples(&current);

    apply_historical_paper_gutter_recipe(&mut harness);
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::TrueColor,
        "historical recipe redraw keeps the actual Runtime capability TrueColor"
    );
    assert_returning_paper_context(&harness, "historical recipe frame");
    let historical = harness.buffer().clone();
    assert_eq!(harness.cursor(), current_cursor);
    assert_eq!(harness.cursor_bar(), current_cursor_bar);
    assert_eq!(harness.runtime().focus(), current_focus);
    assert_returning_paper_pair(&current, &historical);

    let historical_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(historical_scene.key(), "returning 100 30 paper truecolor");
    assert_eq!(
        format!("{:016x}", historical_scene.digest()),
        "bce8c97e3c84ddfa",
        "held digest is checked only after current tuples and full-cell pair assertions"
    );
    historical_scene.assert_against(&BASELINE);
}

#[test]
fn jackin_returning_100x30_paper_truecolor_gutter_replay_matches_held_frame() {
    assert_historical_returning_paper_replay();
}

pub(super) fn is_historical_returning_paper_mono_case(
    scenario: Scenario,
    width: u16,
    height: u16,
    theme: &Theme,
    color: ColorLevel,
) -> bool {
    scenario == Scenario::Returning
        && (width, height) == (100, 30)
        && *theme == Theme::paper()
        && color == ColorLevel::Mono
}

const PAPER_MONO_PRIMARY_POSITIONS: [(u16, u16); 2] = [(4, 24), (4, 25)];
const CURRENT_PAPER_MONO_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: Black, bg: Black, underline_color: Reset, modifier: NONE, diff_option: None, skip: false }"#;
const HISTORICAL_PAPER_MONO_PRIMARY_DEBUG: &str = r#"Cell { symbol: Some(" "), fg: White, bg: Black, underline_color: Reset, modifier: BOLD, diff_option: None, skip: false }"#;

fn apply_historical_paper_mono_primary_gutter_recipe(harness: &mut Harness<App>) {
    let mut theme = harness.runtime().theme().clone();
    theme.design.glyphs.set(GlyphRole::SwitchOff, " ");

    let button = theme.recipes.get_mut(Family::BUTTON);
    assert_eq!(
        button.default_variant,
        Variant::SECONDARY,
        "Paper's ordinary button default remains SECONDARY"
    );
    let primary_parts = button.variant_mut(Variant::PRIMARY);
    assert!(
        primary_parts.get(Part::GUTTER).is_some(),
        "Paper primary button keeps its authored gutter state rules"
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

fn assert_current_paper_mono_primary_gutter(buffer: &Buffer) {
    for &position in &PAPER_MONO_PRIMARY_POSITIONS {
        assert_complete_cell_tuple(
            buffer,
            position,
            Color::Black,
            Color::Black,
            Modifier::empty(),
            CURRENT_PAPER_MONO_PRIMARY_DEBUG,
            "current Paper Mono primary gutter",
        );
    }
}

fn assert_paper_mono_primary_pair(current: &Buffer, historical: &Buffer) {
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(historical.area, current.area);
    assert_eq!(
        u32::from(current.area.width).checked_mul(u32::from(current.area.height)),
        Some(3_000)
    );

    for y in 0..current.area.height {
        for x in 0..current.area.width {
            let current_cell = cell_at(current, x, y);
            let historical_cell = cell_at(historical, x, y);
            let position = (x, y);
            if PAPER_MONO_PRIMARY_POSITIONS.contains(&position) {
                assert_ne!(
                    current_cell, historical_cell,
                    "Paper Mono primary gutter must be a measured change at ({x}, {y})"
                );
                assert_complete_cell_tuple(
                    current,
                    position,
                    Color::Black,
                    Color::Black,
                    Modifier::empty(),
                    CURRENT_PAPER_MONO_PRIMARY_DEBUG,
                    "current Paper Mono primary gutter",
                );
                assert_complete_cell_tuple(
                    historical,
                    position,
                    Color::White,
                    Color::Black,
                    Modifier::BOLD,
                    HISTORICAL_PAPER_MONO_PRIMARY_DEBUG,
                    "historical Paper Mono primary gutter",
                );
            } else {
                assert_eq!(
                    current_cell, historical_cell,
                    "logical/native Paper Mono Cell changed outside the two measured primary gutters at ({x}, {y})"
                );
                assert_eq!(
                    format!("{current_cell:?}"),
                    format!("{historical_cell:?}"),
                    "native Paper Mono Cell Debug changed outside the two measured primary gutters at ({x}, {y})"
                );
            }
        }
    }
}

pub(super) fn assert_historical_returning_paper_mono_replay() {
    let app = App::for_scenario(Scenario::Returning, Motion::Paused);
    let mut harness = Harness::new(app, Theme::paper(), 100, 30).with_color(ColorLevel::Mono);
    harness.draw();
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::Mono,
        "actual Runtime capability is Mono before historical adaptation"
    );
    assert_returning_paper_context(&harness, "ordinary current frame");

    let current_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(current_scene.key(), "returning 100 30 paper mono");
    assert_eq!(
        format!("{:016x}", current_scene.digest()),
        "f6dd49a7500b69b0",
        "unadapted current Paper Mono frame is asserted before historical recipe application"
    );

    let current = harness.buffer().clone();
    let current_cursor = harness.cursor();
    let current_cursor_bar = harness.cursor_bar();
    let current_focus = harness.runtime().focus();
    assert_eq!(current.area, Rect::new(0, 0, 100, 30));
    assert_eq!(current_cursor, None);
    assert!(!current_cursor_bar);
    assert!(
        current_focus.is_some(),
        "current Paper Mono frame retains its actual focused owner"
    );
    assert_current_paper_mono_primary_gutter(&current);

    apply_historical_paper_mono_primary_gutter_recipe(&mut harness);
    assert_eq!(
        harness.runtime().theme().capability.color,
        ColorLevel::Mono,
        "historical recipe redraw keeps the actual Runtime capability Mono"
    );
    assert_returning_paper_context(&harness, "historical recipe frame");
    let historical = harness.buffer().clone();
    assert_eq!(harness.cursor(), current_cursor);
    assert_eq!(harness.cursor_bar(), current_cursor_bar);
    assert_eq!(harness.runtime().focus(), current_focus);
    assert_paper_mono_primary_pair(&current, &historical);

    let historical_scene = harness.snapshot().named(Scenario::Returning.name());
    assert_eq!(historical_scene.key(), "returning 100 30 paper mono");
    assert_eq!(
        format!("{:016x}", historical_scene.digest()),
        "ff946d9c5f14b514",
        "held digest is checked only after current tuples and the full-cell pair proof"
    );
    historical_scene.assert_against(&BASELINE);
}

#[test]
fn jackin_returning_100x30_paper_mono_primary_gutter_replay_matches_held_key() {
    assert_historical_returning_paper_mono_replay();
}
