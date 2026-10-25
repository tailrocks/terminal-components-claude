#![cfg(test)]
use super::*;
use crate::sim::catalogue;
use termrock::{Dialog, Theme};
use termrock_test_support::{Harness, Scene};
fn harness(app: App, theme: Theme, width: u16, height: u16) -> Harness<App> {
    let initial =
        termrock::SimulationMoment::from_millis(u64::try_from(app.fixture_time_ms()).unwrap());
    let app = app.with_source_theme(theme.clone());
    Harness::new_with_feedback_clock(
        app,
        theme,
        width,
        height,
        termrock::FeedbackClock::Simulation { initial },
    )
}
fn app(scenario: Scenario) -> Harness<App> {
    harness(
        App::for_scenario_without_history(scenario, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    )
}

#[test]
fn parity_discovery_uses_keyed_rows_and_selected_action_preview() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 80),
        Theme::junie(),
        120,
        40,
    );
    assert_eq!(harness.app().world.now_ms(), 80);
    let pull = PartRef::item(Part::ROW, ItemKey::text("git.pull"));
    assert!(
        harness.find("move every .idea").is_some(),
        "the original Clean detail remains a passive sample before ROWS focus"
    );
    assert!(
        harness.area_of_part(home::ROWS, pull).is_some(),
        "the canonical Pull Action must own a real NavList row hit"
    );
    for decoration_index in [0, 6, 11] {
        assert!(
            harness
                .area_of_part(
                    home::ROWS,
                    PartRef::item(Part::ROW, ItemKey::index(decoration_index)),
                )
                .is_none(),
            "decorative finder lines must not register actionable rows"
        );
    }
    assert!(harness.tab_to(home::ROWS));
    assert_eq!(harness.focus(), Some(home::ROWS));
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("git.pull".into())
    );

    let _ = harness.key(KeyCode::Down);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("idea.clean".into()),
        "navigation skips the non-action headings and chips"
    );
    assert!(harness.find("move every .idea").is_some());

    let _ = harness.key(KeyCode::Down);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("deploy".into())
    );
    assert!(harness.find("tools/deploy.sh").is_some());
    assert!(harness.find("Does").is_none());

    let _ = harness.key(KeyCode::End);
    let after_end_selected = harness
        .app()
        .home
        .selected(&harness.app().world)
        .map(|action| action.id);
    assert_eq!(
        after_end_selected,
        Some("scope.system".into()),
        "End skips the non-action recent/chip lines"
    );
    assert!(harness.find("explore · host").is_some());

    let status = PartRef::item(Part::ROW, ItemKey::text("git.status"));
    assert!(harness.area_of_part(home::ROWS, status).is_some());
    let _ = harness.type_str("pull");
    assert_eq!(harness.focus(), Some(home::ROWS));
    assert_eq!(harness.app().home.query(), "pull");
    let _ = harness.key(KeyCode::Home);
    let _ = harness.type_str("x");
    assert_eq!(
        harness.app().home.query(),
        "pullx",
        "printable fallback remains in the query while Home belongs to focused rows"
    );
    let _ = harness.ctrl('u');
    assert_eq!(harness.app().home.query(), "");
    assert!(harness.tab_to(home::ROWS));
    assert!(harness.area_of_part(home::ROWS, status).is_some());
    let _ = harness.click_part(home::ROWS, status);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("git.status".into())
    );
    assert!(matches!(harness.app().route, Route::Home));
    assert_eq!(
        harness.app().status.as_deref(),
        Some("Would run: git status · simulated, nothing executed")
    );
    assert!(harness.app().world.activities.is_empty());
    assert!(harness.diagnostics().is_empty());
}

#[test]
fn parity_discovery_query_enter_still_previews_the_first_action() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 80),
        Theme::junie(),
        120,
        40,
    );
    assert!(harness.tab_to(home::QUERY));
    assert_eq!(harness.focus(), Some(home::QUERY));
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("git.pull".into())
    );
    let _ = harness.type_str("pull");
    assert_eq!(harness.app().home.query(), "pull");
    let _ = harness.key(KeyCode::Home);
    let _ = harness.type_str("x");
    assert_eq!(
        harness.app().home.query(),
        "xpull",
        "Home moves the caret in the focused query"
    );
    let _ = harness.key(KeyCode::End);
    let _ = harness.type_str("y");
    assert_eq!(
        harness.app().home.query(),
        "xpully",
        "End moves the caret in the focused query"
    );
    let _ = harness.ctrl('u');
    assert_eq!(harness.app().home.query(), "");
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("git.pull".into()),
        "clearing the query restores the first action"
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
    assert!(harness.find("Pull").is_some());
    assert!(matches!(harness.app().route, Route::Home));
}

fn parity_fixture_harness(width: u16, height: u16) -> Harness<App> {
    harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 80),
        Theme::junie(),
        width,
        height,
    )
}

fn parity_browser_harness(
    theme: Theme,
    width: u16,
    height: u16,
    color: termrock::ColorLevel,
) -> Harness<App> {
    harness(
        App::for_scenario_without_history(Scenario::ParityBrowser, Motion::Paused, 80),
        theme,
        width,
        height,
    )
    .with_color(color)
}

fn parity_files_harness(
    theme: Theme,
    width: u16,
    height: u16,
    color: termrock::ColorLevel,
) -> Harness<App> {
    harness(
        App::for_scenario_without_history(Scenario::ParityFiles, Motion::Paused, 80),
        theme,
        width,
        height,
    )
    .with_color(color)
}

#[test]
fn parity_browser_statusbar_matches_retained_native_cells_across_the_real_gate() {
    let colors = [
        termrock::ColorLevel::TrueColor,
        termrock::ColorLevel::Ansi256,
        termrock::ColorLevel::Ansi16,
        termrock::ColorLevel::Mono,
    ];
    for (theme_name, theme) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for color in colors {
            for (width, height) in [(80, 24), (100, 30), (120, 40), (159, 40)] {
                let actual = parity_browser_harness(theme.clone(), width, height, color);
                assert_eq!(actual.app().world.now_ms(), 80);
                assert!(catalogue::parity_browser_tag_frame(&actual.app().world));
                assert!(parity_browser_tag_plate(&actual.app().world, width, height));

                let mut retained = Scene::new(
                    "retained ParityBrowser status row",
                    theme.clone(),
                    color,
                    width,
                    height,
                );
                let status = Rect::new(0, height.saturating_sub(2), width, 1);
                retained.draw(|ui, _full| paint_legacy_parity_browser_status(ui, status));

                for x in status.x..status.right() {
                    let y = status.y;
                    let actual_cell = actual
                        .buffer()
                        .cell((x, y))
                        .expect("actual App status cell is inside its frame");
                    let retained_cell = retained
                        .buffer()
                        .cell((x, y))
                        .expect("retained status cell is inside its frame");
                    assert_eq!(
                        actual_cell, retained_cell,
                        "status cell ({x},{y}); {theme_name} {color:?} {width}x{height}"
                    );
                    assert_eq!(
                        format!("{actual_cell:?}"),
                        format!("{retained_cell:?}"),
                        "Cell Debug ({x},{y}); {theme_name} {color:?} {width}x{height}"
                    );
                }
                assert!(
                    actual.diagnostics().is_empty(),
                    "unexpected diagnostics for {theme_name}/{color:?}/{width}x{height}: {:?}",
                    actual.diagnostics()
                );
            }
        }
    }
}

#[test]
fn parity_browser_statusbar_preserves_native_cells_outside_nested_offset_clip() {
    let width = 80;
    let height = 24;
    let theme = Theme::paper();
    let color = termrock::ColorLevel::Ansi16;
    let area = Rect::new(5, 5, 70, 1);
    let outer_clip = Rect::new(7, 5, 66, 1);
    let inner_clip = Rect::new(9, 5, 55, 1);
    let mut actual = Scene::new("StatusBar nested clip", theme.clone(), color, width, height);
    let mut actual_seed = None;
    actual.draw_over(
        |buffer| {
            for y in 0..height {
                for x in 0..width {
                    let cell = buffer
                        .cell_mut((x, y))
                        .expect("actual sentinel cell is inside its frame");
                    cell.set_symbol("S");
                    cell.fg = Color::Rgb(1, 2, 3);
                    cell.bg = Color::Rgb(4, 5, 6);
                    cell.modifier = termrock::Modifier::BOLD;
                }
            }
            actual_seed = Some(buffer.clone());
        },
        |ui, _full| {
            ui.with_area(outer_clip, |ui| {
                ui.with_area(inner_clip, |ui| draw_parity_browser_status(ui, area));
            });
        },
    );
    let actual_seed = actual_seed.expect("actual full-frame sentinel was captured");

    let mut retained = Scene::new("retained status nested clip", theme, color, width, height);
    let mut retained_seed = None;
    retained.draw_over(
        |buffer| {
            for y in 0..height {
                for x in 0..width {
                    let cell = buffer
                        .cell_mut((x, y))
                        .expect("retained sentinel cell is inside its frame");
                    cell.set_symbol("S");
                    cell.fg = Color::Rgb(1, 2, 3);
                    cell.bg = Color::Rgb(4, 5, 6);
                    cell.modifier = termrock::Modifier::BOLD;
                }
            }
            retained_seed = Some(buffer.clone());
        },
        |ui, _full| {
            ui.with_area(outer_clip, |ui| {
                ui.with_area(inner_clip, |ui| {
                    paint_legacy_parity_browser_status(ui, area);
                });
            });
        },
    );
    let retained_seed = retained_seed.expect("retained full-frame sentinel was captured");

    for y in 0..height {
        for x in 0..width {
            let actual_cell = actual
                .buffer()
                .cell((x, y))
                .expect("actual clipped cell is inside its frame");
            let retained_cell = retained
                .buffer()
                .cell((x, y))
                .expect("retained clipped cell is inside its frame");
            assert_eq!(actual_cell, retained_cell, "cell at ({x},{y})");
            assert_eq!(
                format!("{actual_cell:?}"),
                format!("{retained_cell:?}"),
                "complete Cell Debug at ({x},{y})"
            );
            let inside_clip = x >= inner_clip.x
                && x < inner_clip.right()
                && y >= inner_clip.y
                && y < inner_clip.bottom();
            if !inside_clip {
                assert_eq!(
                    actual_cell,
                    actual_seed
                        .cell((x, y))
                        .expect("actual seed cell is inside its frame"),
                    "StatusBar changed a cell outside the nested clip at ({x},{y})"
                );
                assert_eq!(
                    retained_cell,
                    retained_seed
                        .cell((x, y))
                        .expect("retained seed cell is inside its frame"),
                    "retained painter changed a cell outside the nested clip at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn parity_files_statusbar_matches_retained_native_cells_across_the_real_gate() {
    let colors = [
        termrock::ColorLevel::TrueColor,
        termrock::ColorLevel::Ansi256,
        termrock::ColorLevel::Ansi16,
        termrock::ColorLevel::Mono,
    ];
    for (theme_name, theme) in [("Junie", Theme::junie()), ("Paper", Theme::paper())] {
        for color in colors {
            for (width, height) in [(80, 24), (100, 30), (120, 40), (159, 40)] {
                let actual = parity_files_harness(theme.clone(), width, height, color);
                assert_eq!(actual.app().world.now_ms(), 80);
                assert!(catalogue::parity_files_tag_frame(&actual.app().world));
                assert!(parity_files_tag_plate(&actual.app().world, width, height));
                assert!(matches!(actual.app().route, Route::Home));

                let mut retained = Scene::new(
                    "retained ParityFiles status row",
                    theme.clone(),
                    color,
                    width,
                    height,
                );
                let status = Rect::new(0, height.saturating_sub(2), width, 1);
                retained.draw(|ui, _full| paint_legacy_parity_files_status(ui, status));

                for x in status.x..status.right() {
                    let y = status.y;
                    let actual_cell = actual
                        .buffer()
                        .cell((x, y))
                        .expect("actual App status cell is inside its frame");
                    let retained_cell = retained
                        .buffer()
                        .cell((x, y))
                        .expect("retained status cell is inside its frame");
                    assert_eq!(
                        actual_cell, retained_cell,
                        "status cell ({x},{y}); {theme_name} {color:?} {width}x{height}"
                    );
                    assert_eq!(
                        format!("{actual_cell:?}"),
                        format!("{retained_cell:?}"),
                        "Cell Debug ({x},{y}); {theme_name} {color:?} {width}x{height}"
                    );
                }
                assert!(
                    actual.diagnostics().is_empty(),
                    "unexpected diagnostics for {theme_name}/{color:?}/{width}x{height}: {:?}",
                    actual.diagnostics()
                );
            }
        }
    }
}

#[test]
fn parity_files_statusbar_preserves_native_cells_outside_nested_offset_clip() {
    let width = 80;
    let height = 24;
    let theme = Theme::paper();
    let color = termrock::ColorLevel::Ansi16;
    let area = Rect::new(5, 5, 70, 1);
    let outer_clip = Rect::new(7, 5, 66, 1);
    let inner_clip = Rect::new(9, 5, 55, 1);
    let mut actual = Scene::new("StatusBar nested clip", theme.clone(), color, width, height);
    let mut actual_seed = None;
    actual.draw_over(
        |buffer| {
            for y in 0..height {
                for x in 0..width {
                    let cell = buffer
                        .cell_mut((x, y))
                        .expect("actual sentinel cell is inside its frame");
                    cell.set_symbol("S");
                    cell.fg = Color::Rgb(1, 2, 3);
                    cell.bg = Color::Rgb(4, 5, 6);
                    cell.modifier = termrock::Modifier::BOLD;
                }
            }
            actual_seed = Some(buffer.clone());
        },
        |ui, _full| {
            ui.with_area(outer_clip, |ui| {
                ui.with_area(inner_clip, |ui| draw_parity_files_status(ui, area));
            });
        },
    );
    let actual_seed = actual_seed.expect("actual full-frame sentinel was captured");

    let mut retained = Scene::new("retained status nested clip", theme, color, width, height);
    let mut retained_seed = None;
    retained.draw_over(
        |buffer| {
            for y in 0..height {
                for x in 0..width {
                    let cell = buffer
                        .cell_mut((x, y))
                        .expect("retained sentinel cell is inside its frame");
                    cell.set_symbol("S");
                    cell.fg = Color::Rgb(1, 2, 3);
                    cell.bg = Color::Rgb(4, 5, 6);
                    cell.modifier = termrock::Modifier::BOLD;
                }
            }
            retained_seed = Some(buffer.clone());
        },
        |ui, _full| {
            ui.with_area(outer_clip, |ui| {
                ui.with_area(inner_clip, |ui| {
                    paint_legacy_parity_files_status(ui, area);
                });
            });
        },
    );
    let retained_seed = retained_seed.expect("retained full-frame sentinel was captured");

    for y in 0..height {
        for x in 0..width {
            let actual_cell = actual
                .buffer()
                .cell((x, y))
                .expect("actual clipped cell is inside its frame");
            let retained_cell = retained
                .buffer()
                .cell((x, y))
                .expect("retained clipped cell is inside its frame");
            assert_eq!(actual_cell, retained_cell, "cell at ({x},{y})");
            assert_eq!(
                format!("{actual_cell:?}"),
                format!("{retained_cell:?}"),
                "complete Cell Debug at ({x},{y})"
            );
            let inside_clip = x >= inner_clip.x
                && x < inner_clip.right()
                && y >= inner_clip.y
                && y < inner_clip.bottom();
            if !inside_clip {
                assert_eq!(
                    actual_cell,
                    actual_seed
                        .cell((x, y))
                        .expect("actual seed cell is inside its frame"),
                    "StatusBar changed a cell outside the nested clip at ({x},{y})"
                );
                assert_eq!(
                    retained_cell,
                    retained_seed
                        .cell((x, y))
                        .expect("retained seed cell is inside its frame"),
                    "retained painter changed a cell outside the nested clip at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn parity_files_status_route_keeps_real_files_preview_controls() {
    let mut harness = parity_files_harness(Theme::junie(), 80, 24, termrock::ColorLevel::TrueColor);
    assert!(matches!(harness.app().route, Route::Home));
    assert_eq!(harness.app().world.now_ms(), 80);
    assert!(catalogue::parity_files_tag_frame(&harness.app().world));
    assert!(parity_files_tag_plate(&harness.app().world, 80, 24));

    let _ = harness.key(KeyCode::F(4));
    assert!(harness.text().contains("Folder browser"));
    assert!(harness.text().contains("Directory"));
    let browse = Dialog::new(dialogs::DIALOG).action_id(1);
    assert!(
        harness.tab_to(browse),
        "Browse action must be a real dialog owner"
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().route, Route::Files));
    assert_eq!(harness.focus(), Some(files::LIST));
    assert_eq!(harness.app().world.now_ms(), 80);
    assert!(catalogue::parity_files_tag_frame(&harness.app().world));
    assert!(parity_files_tag_plate(&harness.app().world, 80, 24));

    let cwd = harness.app().world.cwd.clone();
    let revision = harness.app().world.effect_revision;
    let selected_path = format!("{cwd}/README.md");
    let row = PartRef::item(Part::ROW, ItemKey::text(&selected_path));
    assert!(harness.area_of_part(files::LIST, row).is_some());
    assert!(harness.text().contains("~/work/notes"));
    let _ = harness.click_part(files::LIST, row);
    assert_eq!(
        harness.app().files.selected_path(),
        Some(selected_path.as_str())
    );
    assert_eq!(harness.focus(), Some(files::LIST));

    let _ = harness.key(KeyCode::Right);
    assert!(matches!(harness.app().route, Route::Files));
    assert_eq!(
        harness.app().files.preview_path(),
        Some(selected_path.as_str())
    );
    assert_eq!(harness.focus(), Some(files::PREVIEW));
    assert!(harness.text().contains("Simulated workspace"));

    let _ = harness.key(KeyCode::Esc);
    assert!(matches!(harness.app().route, Route::Files));
    assert_eq!(harness.app().files.preview_path(), None);
    assert_eq!(
        harness.app().files.selected_path(),
        Some(selected_path.as_str())
    );
    assert_eq!(harness.focus(), Some(files::LIST));
    assert_eq!(harness.app().world.cwd, cwd);
    assert_eq!(harness.app().world.effect_revision, revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn parity_discovery_passive_body_matches_retained_native_cells_and_respects_allocation() {
    for width in [80, 100, 120, 160] {
        let current = parity_fixture_harness(width, 40);
        let frame = Rect::new(0, 0, width, 40);
        let (_, home_area) = shell_areas(frame);
        let mut reference = Scene::new(
            "retained ParityDiscovery body",
            Theme::junie(),
            termrock::ColorLevel::TrueColor,
            width,
            40,
        );
        let mut reference_rows = None;
        reference.draw(|ui, full| {
            ui.fill(full, ui.surface_style());
            reference_rows = home::paint_legacy_parity_discovery_for_test(
                ui,
                &current.app().world,
                home_area,
                current.app().home.query(),
            );
        });
        let rows = reference_rows.expect("retained custom body paints at the tested width");
        for y in rows.y..rows.bottom() {
            for x in rows.x..rows.right() {
                let actual = current
                    .buffer()
                    .cell((x, y))
                    .expect("current passive cell inside the allocated Home rows");
                let expected = reference
                    .buffer()
                    .cell((x, y))
                    .expect("retained passive cell inside the allocated Home rows");
                assert_eq!(
                    actual, expected,
                    "complete passive cell at ({x},{y}) for width {width}"
                );
                assert_eq!(
                    format!("{actual:?}"),
                    format!("{expected:?}"),
                    "complete Cell Debug at ({x},{y}) for width {width}"
                );
            }
        }

        let mut sentinel = Scene::new(
            "retained ParityDiscovery allocation sentinel",
            Theme::junie(),
            termrock::ColorLevel::TrueColor,
            width,
            40,
        );
        let mut seeded = None;
        let mut sentinel_rows = None;
        sentinel.draw_over(
            |buffer| {
                let area = *buffer.area();
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        let cell = buffer
                            .cell_mut((x, y))
                            .expect("sentinel cell inside the full frame");
                        cell.set_symbol("S");
                        cell.fg = termrock::Color::Rgb(1, 2, 3);
                        cell.bg = termrock::Color::Rgb(4, 5, 6);
                        cell.modifier = termrock::Modifier::BOLD;
                    }
                }
                seeded = Some(buffer.clone());
            },
            |ui, _full| {
                sentinel_rows = home::paint_legacy_parity_discovery_for_test(
                    ui,
                    &current.app().world,
                    home_area,
                    current.app().home.query(),
                );
            },
        );
        let seeded = seeded.expect("full-frame sentinel was captured before painting");
        let painted_rows = sentinel_rows.expect("legacy body returns its owned allocation");
        let sentinel_area = sentinel.area();
        for y in sentinel_area.y..sentinel_area.bottom() {
            for x in sentinel_area.x..sentinel_area.right() {
                let inside_rows = x >= painted_rows.x
                    && x < painted_rows.right()
                    && y >= painted_rows.y
                    && y < painted_rows.bottom();
                if inside_rows {
                    continue;
                }
                let after = sentinel
                    .buffer()
                    .cell((x, y))
                    .expect("painted sentinel cell inside the full frame");
                let before = seeded
                    .cell((x, y))
                    .expect("captured sentinel cell inside the full frame");
                assert_eq!(
                    after, before,
                    "retained Home painter writes outside rows at ({x},{y}) width {width}"
                );
                assert_eq!(
                    format!("{after:?}"),
                    format!("{before:?}"),
                    "outside-allocation Cell Debug at ({x},{y}) width {width}"
                );
            }
        }
    }
}

#[test]
fn parity_discovery_action_selection_survives_width_short_and_empty_query_frames() {
    let mut harness = parity_fixture_harness(120, 40);
    let clean = PartRef::item(Part::ROW, ItemKey::text("idea.clean"));
    let system = PartRef::item(Part::ROW, ItemKey::text("scope.system"));
    assert!(harness.tab_to(home::ROWS));
    let _ = harness.key(KeyCode::Down);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("idea.clean".into())
    );

    let _ = harness.resize(119, 40);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("idea.clean".into()),
        "crossing below the action-card breakpoint keeps the selected Action key"
    );
    assert!(
        harness.area_of(home::ROWS).is_none(),
        "the 80–119 passive compatibility painter does not claim a NavList owner"
    );

    let _ = harness.resize(120, 40);
    assert!(harness.area_of_part(home::ROWS, clean).is_some());
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("idea.clean".into())
    );
    for decoration in [0, 6, 11] {
        assert!(
            harness
                .area_of_part(
                    home::ROWS,
                    PartRef::item(Part::ROW, ItemKey::index(decoration)),
                )
                .is_none(),
            "decorative line {decoration} is not an actionable row"
        );
    }
    assert!(harness.tab_to(home::ROWS));
    assert_eq!(harness.focus(), Some(home::ROWS));
    let _ = harness.key(KeyCode::End);
    let after_end_selected = harness
        .app()
        .home
        .selected(&harness.app().world)
        .map(|action| action.id);
    assert_eq!(
        after_end_selected,
        Some("scope.system".into()),
        "keyboard reaches the last real Action through decorative lines"
    );

    let _ = harness.resize(120, 19);
    assert!(harness.area_of(home::ROWS).is_none());
    assert!(harness.area_of(home::QUERY).is_none());
    let _ = harness.resize(120, 40);
    assert!(harness.area_of_part(home::ROWS, system).is_some());
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .map(|action| action.id),
        Some("scope.system".into()),
        "the TooSmall frame does not replace the stored Action key"
    );

    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("no-such-parity-action");
    assert_eq!(harness.app().home.query(), "no-such-parity-action");
    assert!(harness.app().home.selected(&harness.app().world).is_none());
    assert!(harness.area_of(home::ROWS).is_none());
    assert!(harness.area_of_part(home::ROWS, system).is_none());
    assert!(harness.find("Nothing matches").is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn source_theme_validation_uses_full_equality_and_requested_level_projections() {
    let app = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
        .with_source_theme(Theme::paper());
    let source = app.source_theme.unwrap();
    let levels = [
        termrock::ColorLevel::TrueColor,
        termrock::ColorLevel::Ansi256,
        termrock::ColorLevel::Ansi16,
        termrock::ColorLevel::Mono,
    ];
    for level in levels {
        assert_eq!(
            source.projection(level),
            Some(&source.theme.for_level(level)),
            "projection for {level:?}"
        );
    }

    let expected = source
        .projection(termrock::ColorLevel::Mono)
        .unwrap()
        .clone();
    assert!(source.matches_current(&expected));

    let mut changed_tokens = expected.clone();
    changed_tokens.color.fg[0] = Color::White;
    assert!(
        !source.matches_current(&changed_tokens),
        "color tokens changed"
    );

    let mut changed_design = expected.clone();
    changed_design.design.density = match expected.design.density {
        termrock::Density::Comfortable => termrock::Density::Compact,
        termrock::Density::Compact => termrock::Density::Comfortable,
    };
    assert!(!source.matches_current(&changed_design), "design changed");

    let changed_recipes = expected
        .clone()
        .define_family(termrock::Family::MENU, |recipe| {
            recipe
                .part(termrock::Part::LABEL)
                .base(StylePatch::new().set_fg(Role::Accent));
        });
    assert!(!source.matches_current(&changed_recipes), "recipes changed");

    let mut changed_capability = expected.clone();
    changed_capability.capability.color = termrock::ColorLevel::TrueColor;
    assert!(
        !source.matches_current(&changed_capability),
        "capability changed"
    );

    let mut added_palette = expected.clone();
    added_palette.capability_palettes = Some(std::sync::Arc::new(
        termrock::CapabilityPalettes::new(added_palette.color),
    ));
    assert!(!source.matches_current(&added_palette), "palette was added");

    let junie = App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
        .with_source_theme(Theme::junie())
        .source_theme
        .unwrap();
    let mut removed_palette = junie
        .projection(termrock::ColorLevel::Mono)
        .unwrap()
        .clone();
    assert!(removed_palette.capability_palettes.is_some());
    removed_palette.capability_palettes = None;
    assert!(
        !junie.matches_current(&removed_palette),
        "palette was removed"
    );

    changed_tokens.color.fg[0] = expected.color.fg[0];
    assert!(
        source.matches_current(&changed_tokens),
        "exact revert validates"
    );

    let narrowed = Theme::paper().for_level(termrock::ColorLevel::Ansi256);
    let narrowed_source =
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 4_000)
            .with_source_theme(narrowed.clone())
            .source_theme
            .unwrap();
    for level in levels {
        assert_eq!(
            narrowed_source.projection(level),
            Some(&narrowed.for_level(level)),
            "narrowed source projection for {level:?}"
        );
    }
}

#[test]
fn menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit() {
    let mut scene = app(Scenario::FirstUse);
    let before = scene.snapshot();
    let (file_x, file_y) = scene.find("File").expect("menu bar File title");
    assert_eq!(file_y, 0);
    assert!(file_x > 0);
    let x = file_x - 1;
    let _ = scene.mouse(termrock::MouseKind::Move, file_x, file_y);

    for column in x..x + 6 {
        let cell = scene.cell(column, file_y);
        assert_eq!(cell.bg, Color::Rgb(0x1c, 0x1c, 0x1f), "column {column}");
        assert_eq!(cell.fg, Color::Rgb(255, 255, 255), "column {column}");
    }
    for column in 0..120 {
        if !(x..x + 6).contains(&column) {
            let expected = before.buffer().cell((column, file_y)).unwrap();
            assert_eq!(scene.cell(column, file_y), expected, "column {column}");
        }
    }

    let cursor = scene.cursor();
    let _ = scene.mouse(termrock::MouseKind::Move, 119, 39);
    assert_eq!(scene.snapshot().buffer(), before.buffer());
    assert_eq!(scene.cursor(), cursor);
}
#[test]
fn discovery_boot_paints_the_here_band_above_the_filter() {
    let harness = harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = harness.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[2].contains("Here"), "{}", lines[2]);
    assert!(!lines[2].contains('▎'), "{}", lines[2]);
    assert!(lines[3].contains('━'), "{}", lines[3]);
    assert!(lines[4].contains("Search actions"), "{}", lines[4]);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn monorepo_root_frame_40_paints_the_tag_plate() {
    let harness = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = harness.text();
    assert!(text.contains("acme · mise monorepo"), "{text}");
    assert!(text.contains("Follow acme-api-1 logs"), "{text}");
    assert!(text.contains("Inspect 10 GB of generat"), "{text}");
    assert!(text.contains("~/work/acme"), "{text}");
    assert!(text.contains("↑1"), "{text}");
    assert!(text.contains("Inspect 1…"), "{text}");
    assert!(text.contains("2 rebuildable candidates"), "{text}");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn monorepo_root_80x24_frame_40_paints_the_selected_narrow_row_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[8].contains("› Inspect 10 GB of generate…"),
        "{}",
        lines[8]
    );
    assert!(
        lines[8].contains("10 GB generated artifacts"),
        "{}",
        lines[8]
    );
    assert!(
        lines[9].contains("Install missing tools · s…"),
        "{}",
        lines[9]
    );
    assert!(lines[18].contains("Children · 3 projects"), "{}", lines[18]);
    assert!(
        lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[20]
    );
    assert!(!text.contains("System · mbp"), "{text}");
    assert!(!text.contains("Inspect 1…"), "{text}");
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "I");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(32, 8);
    assert_eq!(kind.symbol(), "d");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let thumb = plate.cell(78, 8);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    assert_eq!(thumb.bg, Color::Rgb(0, 0, 0));
    let track = plate.cell(78, 18);
    assert_eq!(track.symbol(), "┆");
    assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
    let faded = plate.cell(4, 18);
    assert_eq!(faded.symbol(), "C");
    assert_eq!(faded.fg, Color::Rgb(140, 140, 140));
    let ellipsis = plate.cell(73, 23);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::monorepo_root_view(world, 120, false).is_none());
    assert_ne!(
        catalogue::monorepo_root_view(world, 80, true),
        catalogue::monorepo_root_view(world, 80, false)
    );
    let view = catalogue::monorepo_root_view(world, 80, false).expect("narrow view");
    assert!(view.lines.iter().any(|line| matches!(
        line,
        catalogue::FinderLine::Row(row) if row.selected && row.title.starts_with("Inspect 10 GB")
    )));
    let wide = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text().contains("2 rebuildable candidates"),
        "{}",
        wide.text()
    );
    assert!(!wide.text().contains("holla disk analyze --artifacts"));
    assert!(wide.text().contains("Inspect 1…"), "{}", wide.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("holla disk analyze --artifacts"));
    assert!(!later.text().contains("› Inspect 10 GB"));
    assert!(later.text().contains("Search actions"), "{}", later.text());
    assert!(catalogue::monorepo_root_view(&later.app().world, 80, false).is_none());
    assert!(!catalogue::monorepo_root_tag_frame(&later.app().world));
}
#[test]
fn monorepo_root_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[8].contains("› Inspect 10 GB of generated artif…"),
        "{}",
        lines[8]
    );
    assert!(
        lines[8].contains("10 GB generated artifacts"),
        "{}",
        lines[8]
    );
    assert!(
        lines[9].contains("Install missing tools · sqlx-cli"),
        "{}",
        lines[9]
    );
    assert!(!lines[9].contains("node"), "{}", lines[9]);
    assert!(lines[19].contains("System · mbp"), "{}", lines[19]);
    assert!(
        lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[26]
    );
    assert!(!text.contains("2 rebuildable candidates"), "{text}");
    assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
    assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
    assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
    assert!(!lines[29].contains("Inspect 1…"), "{}", lines[29]);
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "I");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(39, 8);
    assert_eq!(kind.symbol(), "d");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let ellipsis = plate.cell(88, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::monorepo_root_view(world, 100, false).is_some());
    assert!(catalogue::monorepo_root_view(world, 120, false).is_none());
    assert_ne!(
        catalogue::monorepo_root_view(world, 100, true),
        catalogue::monorepo_root_view(world, 100, false)
    );
    let wide = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(wide.text().contains("Inspect 1…"), "{}", wide.text());
    assert!(
        wide.text().contains("2 rebuildable candidates"),
        "{}",
        wide.text()
    );
    let narrow = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        narrow.text().contains("Install missing tools · s…"),
        "{}",
        narrow.text()
    );
    assert!(!narrow.text().contains("System · mbp"), "{}", narrow.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("holla disk analyze --artifacts"));
    assert!(catalogue::monorepo_root_view(&later.app().world, 100, false).is_none());
    assert!(!catalogue::monorepo_root_tag_frame(&later.app().world));
    let other = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
    assert!(catalogue::monorepo_root_view(&other.world, 100, false).is_none());
    let mut history = App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
    history.home.history = true;
    assert!(catalogue::monorepo_root_view(&history.world, 100, true).is_some());
    let history_screen = harness(history, Theme::junie(), 100, 30);
    assert!(
        history_screen
            .text()
            .contains("→ tools/deploy-preview.sh --env preview · here · mutating"),
        "{}",
        history_screen.text()
    );
    assert!(
        !history_screen.text().contains("Deploy …"),
        "{}",
        history_screen.text()
    );
    assert!(
        !history_screen
            .text()
            .contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        history_screen.text()
    );
}
#[test]
fn monorepo_root_history_frame_40_paints_the_deploy_tag_and_later_seeks_keep_home() {
    let mut app = App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
    app.home.history = true;
    let screen = harness(app, Theme::junie(), 120, 40);
    let text = screen.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Deploy preview"), "{}", lines[7]);
    assert!(lines[7].contains("Current folder…"), "{}", lines[7]);
    assert!(lines[8].contains("Follow acme-api-1 logs"), "{}", lines[8]);
    assert!(lines[8].contains("push the current branch"), "{}", lines[8]);
    assert!(
        lines[10].contains("tools/deploy-pre…preview"),
        "{}",
        lines[10]
    );
    assert!(lines[14].contains("Pull"), "{}", lines[14]);
    assert!(lines[14].contains("used 20 times on mbp"), "{}", lines[14]);
    assert!(lines[18].contains("mutating"), "{}", lines[18]);
    assert!(lines[20].contains("trust required"), "{}", lines[20]);
    assert!(lines[39].contains("Enter Trust…"), "{}", lines[39]);
    assert!(lines[39].contains("Deploy …"), "{}", lines[39]);
    assert!(!text.contains("nothing used here yet"), "{text}");
    assert!(!text.contains("holla disk anal"), "{text}");
    let selected = screen.cell(2, 7);
    assert_eq!(selected.symbol(), "›");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let runs = screen.cell(92, 10);
    assert_eq!(runs.symbol(), "t");
    assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let risk = screen.cell(92, 18);
    assert_eq!(risk.symbol(), "m");
    assert_eq!(risk.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    assert!(
        screen.diagnostics().is_empty(),
        "{:?}",
        screen.diagnostics()
    );
    let mut later_app =
        App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 4_000);
    later_app.home.history = true;
    let later = harness(later_app, Theme::junie(), 120, 40);
    assert!(!later.text().contains("tools/deploy-pre"));
    assert!(!later.text().contains("used 20 times on mbp"));
    assert!(later.text().contains("Search actions"), "{}", later.text());
    let mut stepped = App::for_scenario_without_history(Scenario::MonorepoRoot, Motion::Paused, 40);
    stepped.home.history = true;
    assert!(catalogue::monorepo_root_history_tag(&stepped.world).is_some());
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(catalogue::monorepo_root_history_tag(&stepped.world).is_none());
    assert!(!catalogue::monorepo_root_tag_frame(&stepped.world));
}
#[test]
fn monorepo_child_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = plate.text();
    assert!(
        text.contains("acme › frontend · Node project · pnpm"),
        "{text}"
    );
    assert!(text.contains("From acme · the parent root"), "{text}");
    assert!(text.contains("Inspect 3 GB of generate"), "{text}");
    assert!(text.contains("• 2"), "{text}");
    assert!(text.contains("Inspect 3…"), "{text}");
    assert!(text.contains("~/work/acme/apps…rontend"), "{text}");
    assert!(text.contains("1 rebuildable candidate"), "{text}");
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let later = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!later.text().contains("From acme · the parent root"));
    assert!(later.text().contains("Run dev"), "{}", later.text());
}
#[test]
fn monorepo_child_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[7].contains("› Inspect 3 GB of generated…"),
        "{}",
        lines[7]
    );
    assert!(
        lines[7].contains("3 GB generated artifacts"),
        "{}",
        lines[7]
    );
    assert!(
        lines[8].contains("Install missing tools · s…"),
        "{}",
        lines[8]
    );
    assert!(
        lines[16].contains("From acme · the parent root"),
        "{}",
        lines[16]
    );
    assert!(
        lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[20]
    );
    assert!(lines[23].contains("Tab Preview"), "{}", lines[23]);
    assert!(!text.contains("Inspect 3…"), "{text}");
    let reason_end = plate.cell(76, 9);
    assert_eq!(reason_end.symbol(), "…");
    assert_eq!(reason_end.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    let before_thumb = plate.cell(77, 9);
    assert_eq!(before_thumb.symbol(), " ");
    assert!(!text.contains("System · mbp"), "{text}");
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "I");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let thumb = plate.cell(78, 7);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    let track = plate.cell(78, 18);
    assert_eq!(track.symbol(), "┆");
    assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
    let faded = plate.cell(4, 18);
    assert_eq!(faded.symbol(), "S");
    assert_eq!(faded.fg, Color::Rgb(140, 140, 140));
    let ellipsis = plate.cell(73, 23);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::monorepo_child_view(world, 120).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text().contains("1 rebuildable candidate"),
        "{}",
        wide.text()
    );
    assert!(!wide.text().contains("holla disk analyze --artifacts"));
    assert!(wide.text().contains("Inspect 3…"), "{}", wide.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("holla disk analyze --artifacts"));
    assert!(!later.text().contains("› Inspect 3 GB"));
    assert!(later.text().contains("Search actions"), "{}", later.text());
    assert!(catalogue::monorepo_child_view(&later.app().world, 80).is_none());
    assert!(!catalogue::monorepo_child_tag_frame(&later.app().world));
}
#[test]
fn monorepo_child_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[7].contains("› Inspect 3 GB of generated artifa…"),
        "{}",
        lines[7]
    );
    assert!(
        lines[7].contains("3 GB generated artifacts"),
        "{}",
        lines[7]
    );
    assert!(
        lines[8].contains("Install missing tools · sqlx-cli"),
        "{}",
        lines[8]
    );
    assert!(
        lines[16].contains("From acme · the parent root"),
        "{}",
        lines[16]
    );
    assert!(
        lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[26]
    );
    assert!(!lines[26].contains("Inspect 3"), "{}", lines[26]);
    assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
    assert!(!text.contains("Inspect 3…"), "{text}");
    assert!(!text.contains("1 rebuildable candidate"), "{text}");
    let ellipsis = plate.cell(88, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    assert!(catalogue::monorepo_child_view(&plate.app().world, 120).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text().contains("1 rebuildable candidate"),
        "{}",
        wide.text()
    );
    assert!(wide.text().contains("Inspect 3…"), "{}", wide.text());
    assert!(!wide.text().contains("'holla disk analyze --artifacts'"));
    let narrow = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        narrow.text().contains("Inspect 3 GB of generated…"),
        "{}",
        narrow.text()
    );
    assert!(!narrow.text().contains("generated artifa…"));
    let later = harness(
        App::for_scenario_without_history(Scenario::MonorepoChild, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("From acme · the parent root"));
    assert!(later.text().contains("Search actions"), "{}", later.text());
    assert!(catalogue::monorepo_child_view(&later.app().world, 100).is_none());
}
#[test]
fn docker_cleanup_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].contains("no project"), "{}", lines[0]);
    assert!(lines[0].contains("devbox · development"), "{}", lines[0]);
    assert!(plate.app().home.query().is_empty());
    assert!(
        lines[4].contains("Search actions and resources"),
        "{}",
        lines[4]
    );
    assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
    assert!(
        lines[7].contains("› Clean Docker completely"),
        "{}",
        lines[7]
    );
    assert!(lines[8].contains("Prune builder cache"), "{}", lines[8]);
    assert!(
        lines[15].contains("docker stop acme…redis-1"),
        "{}",
        lines[15]
    );
    assert!(lines[38].contains("/work/scratch"), "{}", lines[38]);
    assert!(lines[39].contains("Enter Review…"), "{}", lines[39]);
    assert!(lines[39].contains("Esc Clear"), "{}", lines[39]);
    assert!(lines[39].contains("Clean Docker c…"), "{}", lines[39]);
    // No forced cursor: the caret follows real focus on the query field.
    assert_eq!(plate.focus(), Some(home::QUERY));
    let thumb = plate.cell(116, 8);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    let track = plate.cell(116, 34);
    assert_eq!(track.symbol(), "┆");
    assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
    let gate = plate.cell(92, 34);
    assert_eq!(gate.fg, Color::Rgb(207, 207, 207));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let later = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!later.text().contains("Results · 7"));
    assert!(later.text().contains("Suggested here"), "{}", later.text());
    assert!(later.app().home.query().is_empty());
    let mut stepped =
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40);
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(stepped.home.query().is_empty());
    assert!(!crate::sim::catalogue::docker_cleanup_tag_frame(
        &stepped.world
    ));
}
#[test]
fn docker_cleanup_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
    assert!(
        lines[7].contains("› Clean Docker completely"),
        "{}",
        lines[7]
    );
    assert!(lines[7].contains("18.4 GB recla…"), "{}", lines[7]);
    assert!(
        lines[13].contains("Stop and remove all conta…"),
        "{}",
        lines[13]
    );
    assert!(
        lines[20].contains(
            "→ docker stop acme-api-1 acme-wor…-1 acme-redis-1 · on devbox · destructive"
        ),
        "{}",
        lines[20]
    );
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "C");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(32, 7);
    assert_eq!(kind.symbol(), "p");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let margin = plate.cell(79, 7);
    assert_eq!(margin.symbol(), " ");
    assert_eq!(margin.bg, Color::Rgb(0, 0, 0));
    let esc = plate.cell(63, 23);
    assert_eq!(esc.symbol(), "E");
    assert!(esc.modifier.contains(termrock::Modifier::BOLD));
    let ellipsis = plate.cell(74, 23);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let wide = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text()
            .lines()
            .nth(15)
            .unwrap()
            .contains("docker stop acme…redis-1"),
        "{}",
        wide.text()
    );
    let later = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("Results · 7"));
    assert!(later.text().contains("Suggested here"), "{}", later.text());
    assert!(!crate::sim::catalogue::docker_cleanup_tag_frame(
        &later.app().world
    ));
}
#[test]
fn docker_cleanup_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[6].contains("Results · 7"), "{}", lines[6]);
    assert!(
        lines[7].contains("› Clean Docker completely"),
        "{}",
        lines[7]
    );
    assert!(
        lines[7].contains("18.4 GB reclaimable on devbox"),
        "{}",
        lines[7]
    );
    assert!(
        lines[13].contains("Stop and remove all containers"),
        "{}",
        lines[13]
    );
    assert!(
            lines[26].contains(
                "→ docker stop acme-api-1 acme-worker-1 acme-sc…acme-db-1 acme-redis-1 · on devbox · destructive"
            ),
            "{}",
            lines[26]
        );
    assert!(!lines[26].contains("Clean Docker"), "{}", lines[26]);
    let ellipsis = plate.cell(87, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    assert!(catalogue::docker_cleanup_view(&plate.app().world, 120).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text()
            .lines()
            .nth(15)
            .unwrap()
            .contains("docker stop acme…redis-1"),
        "{}",
        wide.text()
    );
    assert!(!wide.text().contains("acme-worker-1 acme-sc…acme-db-1"));
    let narrow = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(!narrow.text().contains("acme-worker-1 acme-sc…acme-db-1"));
    let later = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("Results · 7"));
    assert!(later.text().contains("Suggested here"), "{}", later.text());
    assert!(catalogue::docker_cleanup_view(&later.app().world, 100).is_none());
}
#[test]
fn remote_host_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[0].contains("payments · service checkout"),
        "{}",
        lines[0]
    );
    assert!(
        lines[0].contains("◆ prod-eu-1 · production"),
        "{}",
        lines[0]
    );
    assert!(!lines[0].contains("ssh"), "{}", lines[0]);
    assert!(
        lines[8].contains("› Follow payments-worker j…"),
        "{}",
        lines[8]
    );
    assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
    assert!(lines[8].contains("journalctl -u"), "{}", lines[8]);
    assert!(
        lines[10].contains("journalctl -u pa… -n 200"),
        "{}",
        lines[10]
    );
    assert!(lines[14].contains("nothing used here yet"), "{}", lines[14]);
    assert!(lines[18].contains("System · prod-eu-1"), "{}", lines[18]);
    assert!(lines[38].contains("/srv/payments"), "{}", lines[38]);
    assert!(lines[38].contains("detached @ v2.14.3"), "{}", lines[38]);
    assert!(lines[39].contains("Enter Run"), "{}", lines[39]);
    assert!(lines[39].contains("Follow pay…"), "{}", lines[39]);
    assert!(lines[39].contains("Ctrl+G"), "{}", lines[39]);
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "F");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(31, 8);
    assert_eq!(kind.symbol(), "s");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    let risk = plate.cell(92, 17);
    assert_eq!(risk.symbol(), "r");
    assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert_eq!(risk.bg, Color::Rgb(0x11, 0x11, 0x11));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let later = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!later.text().contains("Follow payments-worker"));
    assert!(!later.text().contains("detached @ v2.14.3"));
    assert!(later.app().route == Route::Home);
    let mut stepped = App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(!crate::sim::catalogue::remote_host_tag_frame(
        &stepped.world
    ));
}
#[test]
fn remote_host_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[8].contains("› Follow payments-worker jo…"),
        "{}",
        lines[8]
    );
    assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
    assert!(
        lines[20].contains("→ journalctl -u payments-worker -f -n 200 · on prod-eu-1 · read-only"),
        "{}",
        lines[20]
    );
    assert!(lines[23].contains("Enter Run"), "{}", lines[23]);
    assert!(lines[23].contains('…'), "{}", lines[23]);
    assert!(!lines[23].contains("Follow pay…"), "{}", lines[23]);
    assert!(!text.contains("journalctl -u pa…"), "{text}");
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "F");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(32, 8);
    assert_eq!(kind.symbol(), "s");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let edge = plate.cell(78, 8);
    assert_eq!(edge.symbol(), " ");
    assert_eq!(edge.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let ellipsis = plate.cell(72, 23);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::remote_host_view(world, 120).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        !wide
            .text()
            .contains("journalctl -u payments-worker -f -n 200"),
        "120x40 used the narrow plate\n{}",
        wide.text()
    );
    // The 80-column band is height-free: 80x40 keeps the narrow list.
    let tall = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        80,
        40,
    );
    assert!(
        tall.text()
            .contains("journalctl -u payments-worker -f -n 200"),
        "80x40 lost the narrow band\n{}",
        tall.text()
    );
    let later = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("Follow payments-worker"));
    assert!(!later.text().contains("journalctl -u payments-worker"));
    assert!(catalogue::remote_host_view(&later.app().world, 80).is_none());
    assert!(!catalogue::remote_host_tag_frame(&later.app().world));
}
#[test]
fn remote_host_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[7].contains("Who is blocking the database?"),
        "{}",
        lines[7]
    );
    assert!(
        lines[7].contains("2 sessions blocked for 4 min"),
        "{}",
        lines[7]
    );
    assert!(!lines[7].contains("41 min"), "{}", lines[7]);
    assert!(
        lines[8].contains("› Follow payments-worker journal"),
        "{}",
        lines[8]
    );
    assert!(lines[8].contains("unit is restarting"), "{}", lines[8]);
    assert!(lines[9].contains("sustained pressure…"), "{}", lines[9]);
    assert!(lines[18].contains("System · prod-eu-1"), "{}", lines[18]);
    assert!(
        lines[26].contains("→ journalctl -u payments-worker -f -n 200 · on prod-eu-1 · read-only"),
        "{}",
        lines[26]
    );
    assert!(!text.contains("journalctl -u pa…"), "{text}");
    assert!(lines[28].contains("/srv/payments"), "{}", lines[28]);
    assert!(lines[28].contains("detached @ v2.14.3"), "{}", lines[28]);
    assert!(lines[29].contains("Enter Run"), "{}", lines[29]);
    assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
    assert!(lines[29].contains('…'), "{}", lines[29]);
    assert!(!lines[29].contains("Follow pay…"), "{}", lines[29]);
    assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "F");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(36, 8);
    assert_eq!(kind.symbol(), "s");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let edge = plate.cell(98, 8);
    assert_eq!(edge.symbol(), " ");
    assert_eq!(edge.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let age = plate.cell(83, 7);
    assert_eq!(age.symbol(), "4");
    assert_eq!(age.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    let ellipsis = plate.cell(87, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::remote_host_view(world, 100).is_some());
    assert!(catalogue::remote_host_view(world, 120).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(wide.text().contains("Follow pay…"), "{}", wide.text());
    assert!(wide.text().contains("Ctrl+G"), "{}", wide.text());
    assert!(!wide.text().contains("for 4 min"), "{}", wide.text());
    assert!(
        wide.text().contains("journalctl -u pa… -n 200"),
        "{}",
        wide.text()
    );
    let narrow = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        narrow.text().contains("Follow payments-worker jo…"),
        "{}",
        narrow.text()
    );
    assert!(!narrow.text().contains("for 4 min"), "{}", narrow.text());
    assert!(!narrow.text().contains("Follow pay…"), "{}", narrow.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("Follow payments-worker"));
    assert!(!later.text().contains("journalctl -u payments-worker"));
    assert!(catalogue::remote_host_view(&later.app().world, 100).is_none());
    assert!(!catalogue::remote_host_tag_frame(&later.app().world));
    let other = App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
    assert!(catalogue::remote_host_view(&other.world, 100).is_none());
}
#[test]
fn hard_cases_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(plate.app().route == Route::Home);
    assert!(
        lines[0].contains("northwind-traders-platform-migration › inventory"),
        "{}",
        lines[0]
    );
    assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
    assert!(lines[0].contains("ory-reconciliation-core"), "{}", lines[0]);
    assert!(lines[7].contains("Analyze disk usage"), "{}", lines[7]);
    assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
    assert!(
        lines[8].contains("Resolve the rebase in la…"),
        "{}",
        lines[8]
    );
    assert!(lines[14].contains("nothing used here yet"), "{}", lines[14]);
    assert!(lines[14].contains("'holla disk analyze'"), "{}", lines[14]);
    assert!(
        lines[18].contains("Children · 14 projects"),
        "{}",
        lines[18]
    );
    assert!(
        lines[38].contains("~/work/clients/northwind-trad…iation-service"),
        "{}",
        lines[38]
    );
    assert!(lines[38].contains("↓14 ↑3"), "{}", lines[38]);
    assert!(lines[38].contains("2 conflicts"), "{}", lines[38]);
    assert!(
        lines[38].contains("! docker, github unavailable"),
        "{}",
        lines[38]
    );
    assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
    assert!(!lines[39].contains("Follow"), "{}", lines[39]);
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "A");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let risk = plate.cell(92, 20);
    assert_eq!(risk.symbol(), "r");
    assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert_eq!(risk.bg, Color::Rgb(0x11, 0x11, 0x11));
    let overflow = plate.cell(118, 38);
    assert_eq!(overflow.symbol(), "…");
    assert_eq!(overflow.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let later = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!later.text().contains("northwind"));
    assert!(!later.text().contains("! docker, github unavailable"));
    assert!(
        later.text().contains("docker: discovery failed"),
        "{}",
        later.text()
    );
    assert!(later.app().route == Route::Home);
    assert!(!crate::sim::catalogue::hard_cases_tag_frame(
        &later.app().world
    ));
    let mut stepped = App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40);
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(!crate::sim::catalogue::hard_cases_tag_frame(&stepped.world));
    for scenario in [
        Scenario::LaunchFailure,
        Scenario::RemoteHost,
        Scenario::ActivitiesMulti,
        Scenario::MonorepoRoot,
        Scenario::FirstUse,
        Scenario::DockerCleanup,
        Scenario::DiskCleanup,
    ] {
        let other = harness(
            App::for_scenario_without_history(scenario, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(!other.text().contains("northwind"), "{}", scenario.name());
        assert!(!crate::sim::catalogue::hard_cases_tag_frame(
            &other.app().world
        ));
    }
}
#[test]
fn hard_cases_80x24_frame_40_paints_the_narrow_tag_list_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
    assert!(
        lines[8].contains("Resolve the rebase in laz…"),
        "{}",
        lines[8]
    );
    assert!(lines[8].contains("rebase in progress…"), "{}", lines[8]);
    assert!(
        lines[18].contains("Children · 14 projects"),
        "{}",
        lines[18]
    );
    assert!(
        lines[20].contains("→ 'holla disk analyze' · here · read-only"),
        "{}",
        lines[20]
    );
    assert!(
        lines[22].contains("~/work/clients/northwind-trad…iation-service"),
        "{}",
        lines[22]
    );
    assert!(!lines[22].contains("↓14"), "{}", lines[22]);
    assert!(!lines[22].contains("docker"), "{}", lines[22]);
    assert!(lines[23].contains("Enter Open"), "{}", lines[23]);
    assert!(lines[23].contains("Tab"), "{}", lines[23]);
    let thumb = plate.cell(78, 6);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    let track = plate.cell(78, 17);
    assert_eq!(track.symbol(), "┆");
    assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
    let mut footer_overflow = None;
    for x in 0..80 {
        let cell = plate.cell(x, 23);
        if cell.symbol() == "…" {
            footer_overflow = Some((x, cell.fg));
        }
    }
    assert_eq!(
        footer_overflow,
        Some((73, Color::Rgb(0x73, 0x73, 0x73))),
        "{}",
        lines[23]
    );
    let world = &plate.app().world;
    assert!(catalogue::hard_cases_tag_frame(world));
    let wide = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text().contains("! docker, github unavailable"),
        "{}",
        wide.text()
    );
    assert!(wide.text().contains("↓14 ↑3"), "{}", wide.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        !later.text().contains("holla disk analyze"),
        "{}",
        later.text()
    );
    assert!(!catalogue::hard_cases_tag_frame(&later.app().world));
}
#[test]
fn hard_cases_100x30_frame_40_paints_the_mid_tag_list_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[0].contains("northwind-traders-platform-migrati…conciliation-core"),
        "{}",
        lines[0]
    );
    assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
    assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
    assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
    assert!(
        lines[8].contains("Resolve the rebase in lazygit"),
        "{}",
        lines[8]
    );
    assert!(
        lines[8].contains("rebase in progress · 2 conflicts"),
        "{}",
        lines[8]
    );
    assert!(
        lines[19].contains("Parent · northwind-traders-platf…"),
        "{}",
        lines[19]
    );
    assert!(lines[20].contains("System · mbp"), "{}", lines[20]);
    assert!(
        lines[26].contains("→ 'holla disk analyze' · here · read-only"),
        "{}",
        lines[26]
    );
    assert!(
        lines[28].contains("~/work/clients/northwind-trad…iation-service"),
        "{}",
        lines[28]
    );
    assert!(lines[28].contains("↓14 ↑3"), "{}", lines[28]);
    assert!(lines[28].contains("• 7"), "{}", lines[28]);
    assert!(
        lines[28].contains("! docker, github unavailable"),
        "{}",
        lines[28]
    );
    assert!(!lines[28].contains("2 conflicts"), "{}", lines[28]);
    assert!(!lines[28].contains("rebase"), "{}", lines[28]);
    assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
    assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
    assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "A");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(39, 7);
    assert_eq!(kind.symbol(), "d");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let warn = plate.cell(65, 28);
    assert_eq!(warn.symbol(), "!");
    assert_eq!(warn.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
    let status_mark = plate.cell(98, 28);
    assert_eq!(status_mark.symbol(), "…");
    assert_eq!(status_mark.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    let footer_mark = plate.cell(88, 29);
    assert_eq!(footer_mark.symbol(), "…");
    assert_eq!(footer_mark.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::hard_cases_tag_frame(world));
    let wide = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let wide_text = wide.text();
    let wide_lines: Vec<&str> = wide_text.lines().collect();
    assert!(wide_lines[38].contains("2 conflicts"), "{}", wide_lines[38]);
    assert!(wide_lines[38].contains("rebase"), "{}", wide_lines[38]);
    assert!(
        wide_lines[39].contains("Ctrl+G Activities"),
        "{}",
        wide_lines[39]
    );
    let narrow = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let narrow_text = narrow.text();
    let narrow_lines: Vec<&str> = narrow_text.lines().collect();
    assert!(!narrow_lines[22].contains("↓14"), "{}", narrow_lines[22]);
    assert!(catalogue::hard_cases_tag_frame(&narrow.app().world));
    let later = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(
        !later.text().contains("holla disk analyze"),
        "{}",
        later.text()
    );
    assert!(!catalogue::hard_cases_tag_frame(&later.app().world));
    let other = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
    assert!(!catalogue::hard_cases_tag_frame(&other.world));
}
#[test]
fn activities_multi_frame_40_paints_the_tag_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].contains("acme · mise monorepo"), "{}", lines[0]);
    assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
    assert!(lines[1].trim().is_empty(), "{}", lines[1]);
    assert!(lines[2].contains("frontend dev"), "{}", lines[2]);
    assert!(lines[2].contains("api tests"), "{}", lines[2]);
    assert!(lines[2].contains("btm"), "{}", lines[2]);
    assert!(
        lines[8].contains("Inspect 10 GB of generat"),
        "{}",
        lines[8]
    );
    assert!(lines[11].contains("Current folder…"), "{}", lines[11]);
    assert!(lines[16].contains("Running"), "{}", lines[16]);
    assert!(
        lines[17].contains("logs · api worker schedu…"),
        "{}",
        lines[17]
    );
    assert!(lines[19].contains("⠋ detached · 3 s"), "{}", lines[19]);
    assert!(lines[22].contains("Explore"), "{}", lines[22]);
    assert!(lines[38].contains("~/work/acme"), "{}", lines[38]);
    assert!(lines[38].contains("↑1"), "{}", lines[38]);
    assert!(lines[38].contains("4 running"), "{}", lines[38]);
    assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
    assert!(
        lines[39].contains("api tests finished · exit 0"),
        "{}",
        lines[39]
    );
    assert!(!lines[39].contains("Ctrl+G"), "{}", lines[39]);
    let spin = plate.cell(23, 2);
    assert_eq!(spin.symbol(), "⠋");
    assert_eq!(spin.fg, Color::Rgb(0x48, 0xe0, 0x54));
    let done = plate.cell(55, 2);
    assert_eq!(done.symbol(), "✓");
    assert_eq!(done.fg, Color::Rgb(0x3f, 0xb9, 0x50));
    let idle = plate.cell(98, 2);
    assert_eq!(idle.symbol(), "○");
    assert_eq!(idle.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    let badge = plate.cell(86, 38);
    assert_eq!(badge.symbol(), "⠋");
    assert_eq!(badge.fg, Color::Rgb(0x48, 0xe0, 0x54));
    assert_eq!(badge.bg, Color::Rgb(0x27, 0x27, 0x2a));
    let gap = plate.cell(87, 38);
    assert_eq!(gap.symbol(), " ");
    assert_eq!(gap.bg, Color::Rgb(0x18, 0x18, 0x1b));
    let count = plate.cell(88, 38);
    assert_eq!(count.symbol(), "4");
    assert_eq!(count.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    assert_eq!(count.bg, Color::Rgb(0x27, 0x27, 0x2a));
    let ellipsis = plate.cell(88, 39);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let later = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!later.text().contains("Follow acme-api-1 logs"));
    assert!(!later.text().contains("api tests finished"));
    assert!(later.text().contains("dev server"), "{}", later.text());
    assert!(later.app().route == Route::Home);
    let mut stepped =
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(!crate::sim::catalogue::activities_multi_tag_frame(
        &stepped.world
    ));
}
#[test]
fn activities_multi_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[8].contains("› Inspect 10 GB of generate…"),
        "{}",
        lines[8]
    );
    assert!(
        lines[9].contains("Install missing tools · s…"),
        "{}",
        lines[9]
    );
    assert!(
        lines[17].contains("logs · api worker schedul…"),
        "{}",
        lines[17]
    );
    assert!(
        lines[20].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[20]
    );
    assert!(lines[2].contains("2›"), "{}", lines[2]);
    assert!(lines[2].contains("+2"), "{}", lines[2]);
    assert!(!text.contains("sqlx-cli"), "{text}");
    let here = plate.cell(6, 2);
    assert_eq!(here.symbol(), "H");
    assert_eq!(here.bg, Color::Rgb(0x27, 0x27, 0x2a));
    assert!(here.modifier.contains(termrock::Modifier::BOLD));
    let gutter = plate.cell(1, 2);
    assert_eq!(gutter.symbol(), " ");
    assert_eq!(gutter.fg, Color::Rgb(0x73, 0x73, 0x73));
    let hidden = plate.cell(73, 2);
    assert_eq!(hidden.symbol(), "2");
    assert_eq!(hidden.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let more = plate.cell(76, 2);
    assert_eq!(more.symbol(), "+");
    assert_eq!(more.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let rule = plate.cell(1, 3);
    assert_eq!(rule.symbol(), "─");
    assert_eq!(rule.fg, Color::Rgb(0x3a, 0x3a, 0x3a));
    let accent = plate.cell(5, 3);
    assert_eq!(accent.symbol(), "━");
    let selected = plate.cell(4, 8);
    assert_eq!(selected.symbol(), "I");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let thumb = plate.cell(78, 8);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::activities_multi_view(world, 120, false).is_none());
    assert!(catalogue::activities_multi_view(world, 80, true).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(wide.text().contains("btm"), "{}", wide.text());
    assert!(!wide.text().contains("sqlx-cli"), "{}", wide.text());
    assert!(
        !wide.text().contains("'holla disk analyze --artifacts'"),
        "{}",
        wide.text()
    );
    let later = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("Follow acme-api-1 logs"));
    assert!(!later.text().contains("holla disk analyze"));
    assert!(catalogue::activities_multi_view(&later.app().world, 80, false).is_none());
    assert!(!catalogue::activities_multi_tag_frame(&later.app().world));
}
#[test]
fn activities_multi_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[2].contains("2›"), "{}", lines[2]);
    assert!(lines[2].contains("+2"), "{}", lines[2]);
    assert!(!lines[2].contains("btm"), "{}", lines[2]);
    assert!(
        lines[8].contains("› Inspect 10 GB of generated artif…"),
        "{}",
        lines[8]
    );
    assert!(
        lines[9].contains("Install missing tools · sqlx-cli"),
        "{}",
        lines[9]
    );
    assert!(
        lines[17].contains("logs · api worker scheduler"),
        "{}",
        lines[17]
    );
    assert!(lines[24].contains("Children · 3 projects"), "{}", lines[24]);
    assert!(
        lines[26].contains("→ 'holla disk analyze --artifacts' · here · read-only"),
        "{}",
        lines[26]
    );
    assert!(lines[28].contains("~/work/acme"), "{}", lines[28]);
    assert!(lines[28].contains("4 running"), "{}", lines[28]);
    assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
    assert!(
        lines[29].contains("Alt+Enter Alternatives…"),
        "{}",
        lines[29]
    );
    assert!(
        lines[29].contains("api tests finished · exit 0"),
        "{}",
        lines[29]
    );
    assert!(!lines[29].contains("Tab Preview"), "{}", lines[29]);
    assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
    let hidden = plate.cell(93, 2);
    assert_eq!(hidden.symbol(), "2");
    assert_eq!(hidden.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let more = plate.cell(96, 2);
    assert_eq!(more.symbol(), "+");
    assert_eq!(more.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let ellipsis = plate.cell(60, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    let thumb = plate.cell(98, 8);
    assert_eq!(thumb.symbol(), "┃");
    assert_eq!(thumb.fg, Color::Rgb(0x80, 0x80, 0x80));
    let track = plate.cell(98, 24);
    assert_eq!(track.symbol(), "┆");
    assert_eq!(track.fg, Color::Rgb(0x47, 0x47, 0x47));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::activities_multi_view(world, 100, false).is_some());
    assert!(catalogue::activities_multi_view(world, 120, false).is_none());
    assert_ne!(
        catalogue::activities_multi_view(world, 100, true),
        catalogue::activities_multi_view(world, 100, false)
    );
    let other = App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
    assert!(catalogue::activities_multi_view(&other.world, 100, false).is_none());
    let wide = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(wide.text().contains("btm"), "{}", wide.text());
    assert!(!wide.text().contains("sqlx-cli"), "{}", wide.text());
    let narrow = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        narrow.text().contains("Install missing tools · s…"),
        "{}",
        narrow.text()
    );
    assert!(!narrow.text().contains("sqlx-cli"), "{}", narrow.text());
    let later = harness(
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("Follow acme-api-1 logs"));
    assert!(!later.text().contains("holla disk analyze"));
    assert!(catalogue::activities_multi_view(&later.app().world, 100, false).is_none());
    assert!(!catalogue::activities_multi_tag_frame(&later.app().world));
}
#[test]
fn activities_multi_history_frame_40_paints_the_deploy_tag_and_later_seeks_keep_home() {
    let mut app = App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
    app.home.history = true;
    let screen = harness(app, Theme::junie(), 120, 40);
    let text = screen.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Deploy preview"), "{}", lines[7]);
    assert!(lines[7].contains("Current folder…"), "{}", lines[7]);
    assert!(lines[8].contains("Follow acme-api-1 logs"), "{}", lines[8]);
    assert!(
        lines[10].contains("tools/deploy-pre…preview"),
        "{}",
        lines[10]
    );
    assert!(lines[14].contains("Pull"), "{}", lines[14]);
    assert!(lines[14].contains("used 20 times on mbp"), "{}", lines[14]);
    assert!(lines[16].contains("Running"), "{}", lines[16]);
    assert!(
        lines[17].contains("logs · api worker schedu…"),
        "{}",
        lines[17]
    );
    assert!(lines[18].contains("mutating"), "{}", lines[18]);
    assert!(lines[20].contains("trust required"), "{}", lines[20]);
    assert!(lines[39].contains("Enter Trust…"), "{}", lines[39]);
    assert!(
        lines[39].contains("api tests finished · exit 0"),
        "{}",
        lines[39]
    );
    assert!(!lines[39].contains("Enter Open"), "{}", lines[39]);
    assert!(!lines[39].contains("Ctrl+↑↓"), "{}", lines[39]);
    assert!(!lines[39].contains("Deploy"), "{}", lines[39]);
    let ellipsis = screen.cell(75, 39);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(!text.contains("nothing used here yet"), "{text}");
    assert!(!text.contains("holla disk anal"), "{text}");
    let selected = screen.cell(2, 7);
    assert_eq!(selected.symbol(), "›");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let runs = screen.cell(92, 10);
    assert_eq!(runs.symbol(), "t");
    assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let risk = screen.cell(92, 18);
    assert_eq!(risk.symbol(), "m");
    assert_eq!(risk.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    assert!(
        screen.diagnostics().is_empty(),
        "{:?}",
        screen.diagnostics()
    );
    let mut later_app =
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 4_000);
    later_app.home.history = true;
    let later = harness(later_app, Theme::junie(), 120, 40);
    assert!(!later.text().contains("tools/deploy-pre"));
    assert!(!later.text().contains("used 20 times on mbp"));
    assert!(later.text().contains("Search actions"), "{}", later.text());
    let mut stepped =
        App::for_scenario_without_history(Scenario::ActivitiesMulti, Motion::Paused, 40);
    stepped.home.history = true;
    assert!(catalogue::activities_multi_history_tag(&stepped.world).is_some());
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(catalogue::activities_multi_history_tag(&stepped.world).is_none());
    assert!(!catalogue::activities_multi_tag_frame(&stepped.world));
}
#[test]
fn discovery_results_header_is_tag_grey_at_column_4() {
    let harness = harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let (x, y) = harness.find("Suggested here").expect("results header");
    assert_eq!((x, y), (4, 6));
    let title = harness.cell(x, y);
    assert_eq!(title.symbol(), "S");
    assert_eq!(title.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    assert!(!title.modifier.contains(termrock::Modifier::BOLD));
    let pad = harness.cell(1, y);
    assert_eq!(pad.symbol(), " ");
    assert_eq!(pad.fg, Color::Rgb(0xff, 0xff, 0xff));
    assert_eq!(pad.bg, Color::Rgb(0, 0, 0));
    assert!(!pad.modifier.contains(termrock::Modifier::BOLD));
    let before = harness.cell(3, y);
    assert_eq!(before.symbol(), " ");
    assert_eq!(before.fg, Color::Rgb(0xff, 0xff, 0xff));
}
#[test]
fn discovery_frame_40_shows_pull_and_the_preview_plate() {
    let harness = harness(
        App::for_scenario_without_history(Scenario::ParityDiscovery, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = harness.text();
    assert!(text.contains("Pull"), "{text}");
    assert!(text.contains("move every .idea"), "{text}");
    assert!(!text.contains("Monitor this host"), "{text}");
    let (x, y) = harness.find("Pull").expect("pull row");
    assert_eq!((x, y), (4, 7));
    let plate = harness.cell(79, 6);
    assert_eq!(plate.bg, Color::Rgb(0x11, 0x11, 0x11));
    assert_eq!(plate.symbol(), " ");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines[38].trim(),
        "~/work/probe   main   ↓1             ! docker, github unavailable             ▲ 4 config warnings   truecolor · 120×40"
    );
    assert!(lines[39].contains("Type Search"));
    assert!(lines[39].contains("Ctrl+G Activities"));
    let origin = harness.cell(0, 38);
    assert_eq!(origin.symbol(), " ");
    assert_eq!(origin.fg, Color::Rgb(0xff, 0xff, 0xff));
    assert_eq!(origin.bg, Color::Rgb(0x18, 0x18, 0x1b));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn first_use_frame_40_shows_the_disk_tag() {
    let screen = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    let text = screen.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].contains("no project"), "{}", lines[0]);
    assert!(lines[0].contains("mbp · local"), "{}", lines[0]);
    assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
    assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
    assert!(lines[8].contains("sustained pressure…"), "{}", lines[8]);
    assert!(lines[14].contains("'holla disk analyze'"), "{}", lines[14]);
    assert!(lines[27].contains("custom path…"), "{}", lines[27]);
    assert_eq!(
        lines[38].trim(),
        "~/scratch                                                                     / ━━━━━━━━━━─ 91% ▲   truecolor · 120×40"
    );
    assert!(lines[39].contains("Enter Open"), "{}", lines[39]);
    assert!(!lines[39].contains("Confirm"), "{}", lines[39]);
    let runs = screen.cell(92, 14);
    assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let risk = screen.cell(92, 20);
    assert_eq!(risk.symbol(), "r");
    assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
    let gate = screen.cell(92, 22);
    assert_eq!(gate.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    let bar = screen.cell(81, 38);
    assert_eq!(bar.symbol(), "━");
    assert_eq!(bar.fg, Color::Rgb(0xf5, 0x9e, 0x09));
    let rest = screen.cell(91, 38);
    assert_eq!(rest.symbol(), "─");
    assert_eq!(rest.fg, Color::Rgb(0x3a, 0x3a, 0x3a));
    let early = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 0),
        Theme::junie(),
        120,
        40,
    );
    assert!(!early.text().contains("holla disk analyze"));
    assert!(
        screen.diagnostics().is_empty(),
        "{:?}",
        screen.diagnostics()
    );
}
#[test]
fn first_use_80x24_frame_40_paints_the_narrow_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
    assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
    assert!(lines[8].contains("sustained pressure…"), "{}", lines[8]);
    assert!(
        lines[20].contains("→ 'holla disk analyze' · here · read-only"),
        "{}",
        lines[20]
    );
    assert!(!text.contains("Monitor this host"), "{text}");
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "A");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(27, 7);
    assert_eq!(kind.symbol(), "d");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let ellipsis = plate.cell(73, 23);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let wide = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(
        wide.text()
            .lines()
            .nth(14)
            .unwrap()
            .contains("'holla disk analyze'")
    );
    let runs = wide.cell(92, 14);
    assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let later = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
        Theme::junie(),
        80,
        24,
    );
    assert!(!later.text().contains("holla disk analyze"));
    assert!(
        later.text().contains("Monitor this host"),
        "{}",
        later.text()
    );
    assert!(!catalogue::first_use_tag_frame(&later.app().world));
}
#[test]
fn first_use_100x30_frame_40_paints_the_mid_plate_and_later_seeks_keep_home() {
    let plate = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        100,
        30,
    );
    let text = plate.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Analyze disk usage"), "{}", lines[7]);
    assert!(lines[7].contains("/ is 91% full"), "{}", lines[7]);
    assert!(
        lines[8].contains("sustained pressure · continuous view fits"),
        "{}",
        lines[8]
    );
    assert!(!lines[8].contains('…'), "{}", lines[8]);
    assert!(
        lines[26].contains("→ 'holla disk analyze' · here · read-only"),
        "{}",
        lines[26]
    );
    assert!(!text.contains("largest-first"), "{text}");
    assert!(lines[29].contains("Enter Open"), "{}", lines[29]);
    assert!(lines[29].contains("Ctrl+↑↓ Scope"), "{}", lines[29]);
    assert!(!lines[29].contains("Ctrl+G"), "{}", lines[29]);
    let selected = plate.cell(4, 7);
    assert_eq!(selected.symbol(), "A");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let kind = plate.cell(27, 7);
    assert_eq!(kind.symbol(), "d");
    assert!(!kind.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(kind.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    let ellipsis = plate.cell(88, 29);
    assert_eq!(ellipsis.symbol(), "…");
    assert_eq!(ellipsis.fg, Color::Rgb(0x73, 0x73, 0x73));
    assert!(plate.diagnostics().is_empty(), "{:?}", plate.diagnostics());
    let world = &plate.app().world;
    assert!(catalogue::first_use_view(world, 100, false).is_some());
    assert!(catalogue::first_use_view(world, 120, false).is_none());
    assert_ne!(
        catalogue::first_use_view(world, 100, true),
        catalogue::first_use_view(world, 100, false)
    );
    let wide = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        120,
        40,
    );
    assert!(wide.text().contains("Ctrl+G Activities"), "{}", wide.text());
    assert!(wide.text().contains("largest-first"), "{}", wide.text());
    let narrow = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40),
        Theme::junie(),
        80,
        24,
    );
    assert!(
        narrow.text().contains("sustained pressure…"),
        "{}",
        narrow.text()
    );
    assert!(
        !narrow
            .text()
            .contains("sustained pressure · continuous view fits"),
        "{}",
        narrow.text()
    );
    let later = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
        Theme::junie(),
        100,
        30,
    );
    assert!(!later.text().contains("holla disk analyze"));
    assert!(catalogue::first_use_view(&later.app().world, 100, false).is_none());
    assert!(!catalogue::first_use_tag_frame(&later.app().world));
    let other = App::for_scenario_without_history(Scenario::RemoteHost, Motion::Paused, 40);
    assert!(catalogue::first_use_view(&other.world, 100, false).is_none());
    let mut history = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
    history.home.history = true;
    assert!(catalogue::first_use_view(&history.world, 100, true).is_some());
    let history_screen = harness(history, Theme::junie(), 100, 30);
    assert!(
        history_screen.text().contains("holla snapshot"),
        "{}",
        history_screen.text()
    );
    assert!(
        !history_screen
            .text()
            .contains("→ 'holla disk analyze' · here · read-only"),
        "{}",
        history_screen.text()
    );
}
#[test]
fn first_use_history_frame_40_paints_the_system_tag_and_later_seeks_keep_home() {
    let mut app = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
    app.home.history = true;
    let screen = harness(app, Theme::junie(), 120, 40);
    let text = screen.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[7].contains("› Show system resources"), "{}", lines[7]);
    assert!(lines[7].contains("CPU 78% for 4 min"), "{}", lines[7]);
    assert!(
        lines[12].contains("everything used here is suggested above"),
        "{}",
        lines[12]
    );
    assert!(
        lines[13].contains("'holla snapshot system'"),
        "{}",
        lines[13]
    );
    assert!(lines[22].contains("runs directly"), "{}", lines[22]);
    assert!(!text.contains("nothing used here yet"), "{text}");
    assert!(!text.contains("holla disk analyze"), "{text}");
    let selected = screen.cell(2, 7);
    assert_eq!(selected.symbol(), "›");
    assert_eq!(selected.bg, Color::Rgb(0x2f, 0x5a, 0xa8));
    assert!(selected.modifier.contains(termrock::Modifier::BOLD));
    let runs = screen.cell(92, 13);
    assert_eq!(runs.symbol(), "'");
    assert_eq!(runs.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    let risk = screen.cell(92, 20);
    assert_eq!(risk.symbol(), "r");
    assert_eq!(risk.fg, Color::Rgb(0x73, 0x73, 0x73));
    let gate = screen.cell(92, 22);
    assert_eq!(gate.symbol(), "r");
    assert_eq!(gate.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
    assert_eq!(
        lines[38].trim(),
        "~/scratch                                                                     / ━━━━━━━━━━─ 91% ▲   truecolor · 120×40"
    );
    assert!(
        screen.diagnostics().is_empty(),
        "{:?}",
        screen.diagnostics()
    );
    let mut later_app =
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000);
    later_app.home.history = true;
    let later = harness(later_app, Theme::junie(), 120, 40);
    assert!(!later.text().contains("holla snapshot system"));
    assert!(
        !later
            .text()
            .contains("everything used here is suggested above")
    );
    assert!(later.text().contains("Search actions"), "{}", later.text());
    let mut stepped = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 40);
    stepped.home.history = true;
    assert!(catalogue::first_use_history_tag(&stepped.world).is_some());
    assert!(termrock::App::accept_paused_clock_step(&mut stepped, 1_100));
    assert!(catalogue::first_use_history_tag(&stepped.world).is_none());
    assert!(!catalogue::first_use_tag_frame(&stepped.world));
}
#[test]
fn paused_clock_step_advances_world_time_by_1100_ms() {
    let mut paused = App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 0);
    let start = paused.world.now_ms();
    assert!(termrock::App::accept_paused_clock_step(&mut paused, 1_100));
    assert_eq!(paused.world.now_ms(), start + 1_100);
    assert!(!paused.world.clock.running);

    let mut running = App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 0);
    let start = running.world.now_ms();
    assert!(!termrock::App::accept_paused_clock_step(
        &mut running,
        1_100
    ));
    assert_eq!(running.world.now_ms(), start);
}

#[test]
fn home_query_stays_armed_under_action_focus_with_shared_caret_and_paste() {
    let mut harness = app(Scenario::HardCases);
    assert!(harness.tab_to(home::ROWS));
    let _ = harness.type_str("git");
    assert_eq!(harness.focus(), Some(home::ROWS));
    assert_eq!(harness.app().home.query(), "git");
    assert!(harness.cursor().is_some());
    let _ = harness.paste(" q?0 ");
    assert_eq!(harness.app().home.query(), "git q?0 ");
    assert!(harness.app().overlay.is_none());
    let _ = harness.key(KeyCode::Backspace);
    assert_eq!(harness.app().home.query(), "git q?0");
    assert!(harness.app().home.rows(&harness.app().world).is_empty());
    assert_ne!(harness.focus(), Some(home::ROWS));
    assert!(harness.cursor().is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn empty_query_chrome_exceptions_and_modal_typing_do_not_leak() {
    for (character, title) in [('?', "Key reference"), ('q', "Quit holla?")] {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::ROWS));
        let _ = harness.key(KeyCode::Char(character));
        assert!(harness.text().contains(title));
        assert_eq!(harness.app().home.query(), "");
        let _ = harness.type_str("q?0text");
        assert_eq!(harness.app().home.query(), "");
        assert!(harness.app().overlay.is_some());
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().overlay.is_none());
        let _ = harness.paste("q?");
        assert_eq!(harness.app().home.query(), "q?");
        assert!(harness.app().overlay.is_none());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
#[test]
fn unmatched_home_query_has_source_notice_and_no_empty_row_focus_stop() {
    let mut harness = app(Scenario::HardCases);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.paste("  impossible-no-fixture-match  ");
    assert!(harness.app().home.rows(&harness.app().world).is_empty());
    assert!(
        harness
            .text()
            .contains("Nothing matches “impossible-no-fixture-match” here")
    );
    let revision = harness.app().effect_revision();
    let _ = harness.key(KeyCode::Down);
    assert_ne!(harness.focus(), Some(home::ROWS));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().home.query().is_empty());
    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.focus(), Some(home::ROWS));
    assert_eq!(harness.app().effect_revision(), revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn frame_seek_lands_on_discovered_state() {
    let settled = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        40,
    );
    assert!(!settled.app().world.discovering());
    assert!(!settled.text().contains("discovering…"));
    let early = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Paused, 100),
        Theme::junie(),
        120,
        40,
    );
    assert!(early.app().world.discovering());
    assert!(!early.text().lines().next().unwrap().contains("discovering"));
    assert!(early.diagnostics().is_empty(), "{:?}", early.diagnostics());
}
#[test]
fn actions_picker_preserves_source_footer_height_and_query_isolation() {
    let mut h = app(Scenario::RustDirty);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    assert_eq!(
        h.layer_area(crate::screens::actions::PICKER)
            .map(|area| area.height),
        Some(12)
    );
    assert_eq!(h.cursor(), None);
    let text = h.text();
    let footer = text.lines().last().unwrap();
    assert!(
        footer.contains("↑↓ Move")
            && footer.contains("Enter Choose")
            && footer.contains("Esc Cancel")
    );
    assert!(!footer.contains("EDIT"));
    let _ = h.type_str("ignored");
    let _ = h.paste("ignored paste");
    assert_eq!(h.app().home.query(), "cargo build");
    assert!(h.text().contains("Set alias…"));
    let _ = h.key(KeyCode::Esc);
    assert!(h.app().overlay.is_none());
    assert_eq!(h.app().home.query(), "cargo build");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn home_hint_casing_preserves_lowercase_physical_shortcuts() {
    let mut h = app(Scenario::RustDirty);
    let text = h.text();
    let footer = text.lines().last().unwrap();
    assert!(footer.contains("Ctrl+S Scope"));
    assert!(footer.contains("q Quit"));
    let _ = h.ctrl('s');
    assert!(h.text().contains("scope ‹ here ›"));
    assert!(h.tab_to(home::ROWS));
    let text = h.text();
    let footer = text.lines().last().unwrap();
    assert!(footer.contains("Ctrl+P Preview"));
    assert!(footer.contains("Ctrl+O Actions"));
    assert!(h.tab_to(home::QUERY));
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.text().contains("Quit holla?"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn home_product_hints_follow_query_rows_and_yield_to_modal_context() {
    let mut h = app(Scenario::RustDirty);
    assert!(h.text().lines().last().unwrap().contains("Run top match"));
    assert!(h.tab_to(home::ROWS));
    let footer = h.text().lines().last().unwrap().to_owned();
    assert!(footer.contains("Run") && footer.contains("Preview"));
    assert!(!footer.contains("Run top match"));
    let _ = h.key(KeyCode::F(1));
    assert!(!h.text().lines().last().unwrap().contains("Run top match"));
    assert!(!h.text().lines().last().unwrap().contains("Filter"));
    let _ = h.key(KeyCode::Esc);
    assert!(h.tab_to(home::QUERY));
    assert!(h.text().lines().last().unwrap().contains("Type Filter"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn ordinary_home_cut_reason_reaches_footer_without_dropping_hints() {
    let mut h = harness(
        App::for_scenario_without_history(Scenario::RustDirty, Motion::Paused, 4_000),
        Theme::junie(),
        160,
        40,
    );
    let task = h
        .app_mut()
        .world
        .mise
        .as_mut()
        .expect("RustDirty includes project tasks")
        .tasks
        .first_mut()
        .expect("RustDirty includes a task");
    task.defined_in = format!(
        "~/work/pave/{}",
        "footer-peek-producer-long-reason-".repeat(6)
    );

    assert!(h.tab_to(home::QUERY));
    let _ = h.type_str("footer-peek-producer-long-reason");
    assert!(h.tab_to(home::ROWS));
    let action = h
        .app()
        .home
        .selected(&h.app().world)
        .expect("the filtered task remains an ordinary Home row");
    assert!(action.reason.contains("footer-peek-producer-long-reason"));
    let title = action.title.clone();
    let action_id = action.id.clone();
    let footer = h.text().lines().last().unwrap().to_owned();
    assert!(footer.contains(&title), "{footer}");
    assert!(footer.contains("footer-peek-producer"), "{footer}");
    assert!(
        footer.contains("Run") && footer.contains("Preview"),
        "{footer}"
    );
    assert!(
        footer.contains("Actions") && footer.contains("Query"),
        "{footer}"
    );
    assert!(footer.contains("Clear"), "{footer}");

    let _ = h.resize(400, 40);
    let fitted_footer = h.text().lines().last().unwrap().to_owned();
    assert!(!fitted_footer.contains(&title), "{fitted_footer}");
    let _ = h.resize(120, 40);
    assert!(h.tab_to(home::QUERY));
    let query_footer = h.text().lines().last().unwrap().to_owned();
    assert!(!query_footer.contains(&title), "{query_footer}");

    let row_part = PartRef::item(Part::ROW, ItemKey::text(&action_id));
    let row = h
        .area_of_part(home::ROWS, row_part)
        .expect("the filtered task has a native row hit part");
    let x = row.x.saturating_add(row.width / 2);
    let y = row.y;
    let _ = h.mouse(termrock::MouseKind::Down, x, y);
    assert_eq!(h.focus(), Some(home::ROWS));
    let pressed_footer = h.text().lines().last().unwrap().to_owned();
    assert!(pressed_footer.contains(&title), "{pressed_footer}");
    let _ = h.mouse(termrock::MouseKind::Up, 0, 0);
    assert_eq!(h.focus(), Some(home::ROWS));
    let pointer_footer = h.text().lines().last().unwrap().to_owned();
    assert!(pointer_footer.contains(&title), "{pointer_footer}");

    let _ = h.click_part(home::ROWS, row_part);
    let clicked_footer = h.text().lines().last().unwrap().to_owned();
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.starts_with("Would run: mise run ")),
        "the real Home click keeps its normal simulated action status: {:?}",
        h.app().status
    );
    assert!(clicked_footer.contains("Would run:"), "{clicked_footer}");
    assert!(!clicked_footer.contains(&title), "{clicked_footer}");
    assert!(h.app().overlay.is_none());

    h.app_mut().status = None;
    h.draw();
    assert_eq!(h.focus(), Some(home::ROWS));
    let restored_footer = h.text().lines().last().unwrap().to_owned();
    assert!(restored_footer.contains(&title), "{restored_footer}");

    let _ = h.key(KeyCode::F(10));
    assert!(h.app().menu.is_open());
    let menu_footer = h.text().lines().last().unwrap().to_owned();
    assert!(!menu_footer.contains(&title), "{menu_footer}");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.app().menu.is_open());
    assert!(h.tab_to(home::ROWS));
    let _ = h.key(KeyCode::F(1));
    assert!(h.app().overlay.is_some());
    let help_footer = h.text().lines().last().unwrap().to_owned();
    assert!(!help_footer.contains(&title), "{help_footer}");
    let _ = h.key(KeyCode::Esc);
    assert!(h.app().overlay.is_none());
    assert!(h.tab_to(home::ROWS));

    h.app_mut().status = Some("Saved".to_owned());
    h.draw();
    let status_footer = h.text().lines().last().unwrap().to_owned();
    assert!(status_footer.contains("Saved"), "{status_footer}");
    assert!(!status_footer.contains(&title), "{status_footer}");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn home_scope_stays_off_the_menu_row_and_clears_with_escape() {
    let mut harness = app(Scenario::HardCases);
    let _ = harness.ctrl('s');
    assert!(harness.app().home.scope.is_some());
    assert!(!harness.text().lines().next().unwrap().contains("scope:"));
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().home.scope.is_none());
    assert!(!harness.text().lines().next().unwrap().contains("scope:"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn home_escape_clears_scope_before_canonical_query() {
    let mut harness = app(Scenario::HardCases);
    assert!(harness.tab_to(home::ROWS));
    let _ = harness.type_str("git");
    let _ = harness.ctrl('s');
    assert!(harness.app().home.scope.is_some());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().home.scope.is_none());
    assert_eq!(harness.app().home.query(), "git");
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(harness.app().home.query(), "");
    let _ = harness.type_str("docker");
    assert_eq!(harness.app().home.query(), "docker");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn home_navigation_uses_shared_boundaries_and_reveals_selected_rows() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::HardCases, Motion::Paused, 4_000),
        Theme::junie(),
        120,
        20,
    );
    assert!(harness.tab_to(home::QUERY));
    let count = harness.app().home.rows(&harness.app().world).len();
    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.focus(), Some(home::ROWS));
    let first = harness
        .app()
        .home
        .selected(&harness.app().world)
        .unwrap()
        .id;
    let _ = harness.key(KeyCode::Up);
    assert_eq!(harness.focus(), Some(home::QUERY));
    let _ = harness.key(KeyCode::Down);
    for _ in 1..count {
        let _ = harness.key(KeyCode::Down);
    }
    assert_eq!(harness.focus(), Some(home::ROWS));
    let last = harness.app().home.selected(&harness.app().world).unwrap();
    assert_ne!(last.id, first);
    assert!(harness.text().contains(&last.title), "{}", harness.text());
    let _ = harness.key(KeyCode::Down);
    assert_ne!(harness.focus(), Some(home::ROWS));
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.key(KeyCode::Down);
    assert_eq!(
        harness
            .app()
            .home
            .selected(&harness.app().world)
            .unwrap()
            .id,
        first
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn activity_digits_are_route_scoped_and_cycle_from_home_starts_first() {
    let mut harness = app(Scenario::ActivitiesMulti);
    let ids: Vec<_> = activity::ordered(&harness.app().world)
        .iter()
        .map(|a| a.id)
        .collect();
    assert!(ids.len() >= 2);
    let _ = harness.type_str("12");
    assert_eq!(harness.app().home.query(), "12");
    let _ = harness.ctrl('a');
    assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
    let _ = harness.key(KeyCode::Char('2'));
    assert_eq!(harness.app().activities.current_id(), Some(ids[1]));
    let _ = harness.key(KeyCode::Char('9'));
    assert_eq!(harness.app().status.as_deref(), Some("No activity 9"));
    let _ = harness.key(KeyCode::Char('0'));
    assert!(harness.app().route == Route::Home);
    let _ = harness.ctrl('a');
    assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn interrupt_quits_outside_modals_and_quit_chord_precedes_open_menu() {
    let mut harness = app(Scenario::HardCases);
    let _ = harness.key(KeyCode::F(1));
    let _ = harness.ctrl('c');
    assert!(!harness.app().quit);
    let _ = harness.key(KeyCode::Esc);
    let _ = harness.key(KeyCode::F(10));
    assert!(
        harness.app().menu.is_open(),
        "F10 must open the real menu before testing precedence"
    );
    let _ = harness.ctrl('q');
    assert!(harness.text().contains("Quit holla?"));
    assert!(!harness.app().quit);
    let mut harness = app(Scenario::HardCases);
    let _ = harness.key(KeyCode::F(10));
    let _ = harness.ctrl('c');
    assert!(harness.app().quit);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn menu_opens_before_publication_and_survives_focus_settlement() {
    let mut harness = app(Scenario::HardCases);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("git");
    let mut harness = harness.with_auto_draw(false);
    let _ = harness.key(KeyCode::F(10));
    assert!(harness.app().menu.is_open(), "event must open menu");
    harness.draw();
    assert!(
        harness.app().menu.is_open(),
        "publication must preserve menu"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn too_small_surface_consumes_commands_and_quits_without_query_or_modal_effects() {
    for key in [KeyCode::Char('q'), KeyCode::Char('c')] {
        let mut harness = app(Scenario::HardCases);
        let _ = harness.type_str("git");
        let _ = harness.key(KeyCode::F(1));
        let revision = harness.app().effect_revision();
        let _ = harness.resize(40, 10);
        assert!(
            harness.diagnostics().is_empty(),
            "resize: {:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::F(1));
        assert!(!harness.app().quit);
        let _ = harness.resize(120, 40);
        assert!(harness.text().contains("Key reference"));
        assert!(
            harness.diagnostics().is_empty(),
            "restore: {:?}",
            harness.diagnostics()
        );
        let _ = harness.resize(40, 10);
        assert!(
            harness.diagnostics().is_empty(),
            "second resize: {:?}",
            harness.diagnostics()
        );
        if key == KeyCode::Char('c') {
            let _ = harness.ctrl('c');
        } else {
            let _ = harness.key(key);
        }
        assert!(harness.app().quit);
        assert_eq!(harness.app().home.query(), "git");
        assert_eq!(harness.app().effect_revision(), revision);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
#[test]
fn plan_physical_edges_leave_steps_without_changing_inclusion_or_effects() {
    let mut harness = app(Scenario::DockerCleanup);
    let _ = harness.type_str("docker system prune");
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().route == Route::Plan);
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::Home);
    let included = harness.app().plan.as_ref().unwrap().plan().included_count();
    let revision = harness.app().effect_revision();
    let _ = harness.key(KeyCode::Up);
    assert_ne!(harness.focus(), Some(plan::STEPS));
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::End);
    let _ = harness.key(KeyCode::Down);
    assert_ne!(harness.focus(), Some(plan::STEPS));
    assert_eq!(
        harness.app().plan.as_ref().unwrap().plan().included_count(),
        included
    );
    assert_eq!(harness.app().effect_revision(), revision);
    assert!(harness.app().overlay.is_none());
    assert!(harness.activation_feedback().is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn plan_space_is_focus_bound_and_preserves_chrome_activation() {
    let mut harness = app(Scenario::DockerCleanup);
    let _ = harness.type_str("docker system prune");
    let _ = harness.key(KeyCode::Enter);
    let index = harness
        .app()
        .plan
        .as_ref()
        .unwrap()
        .plan()
        .steps()
        .iter()
        .position(|step| step.optional)
        .unwrap();
    for _ in 0..index {
        let _ = harness.key(KeyCode::Down);
    }
    let included = harness.app().plan.as_ref().unwrap().plan().included_count();
    assert!(harness.tab_to(MENU));
    let _ = harness.key(KeyCode::Char(' '));
    assert!(harness.app().menu.is_open());
    assert_eq!(
        harness.app().plan.as_ref().unwrap().plan().included_count(),
        included
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.tab_to(MENU));
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().menu.is_open());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::Char(' '));
    assert!(harness.app().plan.as_ref().unwrap().plan().included_count() < included);
    assert!(harness.app().overlay.is_none());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn plan_keyboard_feedback_uses_post_request_owner_and_double_click_cannot_open_gate() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::DockerCleanup, Motion::Full, 4_000),
        Theme::junie(),
        120,
        40,
    );
    let _ = harness.type_str("docker system prune");
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.activation_feedback().is_none(),
        "Home query submission does not flash"
    );
    let step = harness
        .app()
        .plan
        .as_ref()
        .unwrap()
        .selected()
        .unwrap()
        .id
        .clone();
    let part = PartRef::item(Part::ROW, ItemKey::text(&step));
    let _ = harness.click_part(plan::STEPS, part);
    let _ = harness.click_part(plan::STEPS, part);
    assert!(harness.app().overlay.is_none());
    let _ = harness.advance(std::time::Duration::from_millis(80));
    assert_eq!(
        harness.activation_feedback().unwrap().remaining,
        std::time::Duration::from_millis(60)
    );
    let _ = harness.key(KeyCode::Char(' '));
    let feedback = harness.activation_feedback().unwrap();
    assert_eq!((feedback.owner, feedback.part), (plan::STEPS, part));
    assert_eq!(feedback.remaining, std::time::Duration::from_millis(140));
    let _ = harness.key(KeyCode::Enter);
    let gate = Dialog::new(crate::screens::plan_gate::GATE);
    assert_eq!(
        harness.activation_feedback().unwrap().owner,
        gate.input_id()
    );
    assert_eq!(harness.focus(), Some(gate.input_id()));
    let _ = harness.advance(std::time::Duration::from_millis(80));
    let _ = harness.advance(std::time::Duration::from_millis(80));
    assert!(harness.activation_feedback().is_none());
    let phrase = harness
        .app()
        .plan
        .as_ref()
        .unwrap()
        .plan()
        .phrase()
        .to_owned();
    let _ = harness.type_str(&phrase);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().effect_revision(), 0, "editor Enter only arms");
    assert!(
        harness.activation_feedback().is_none(),
        "modal editing has no keyboard flash"
    );
    assert!(harness.tab_to(gate.action_id(1)));
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(harness.app().effect_revision(), 1);
    assert!(harness.app().plan.as_ref().unwrap().plan().ran());
    assert!(
        harness.activation_feedback().is_none(),
        "modal confirmation preserves source feedback policy"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
#[test]
fn copy_reports_osc52_status_without_emitting_a_sequence() {
    let mut harness = app(Scenario::RustDirty);
    let _ = harness.type_str("cargo build");
    let _ = harness.ctrl('o');
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    let refused = harness.text();
    assert!(refused.contains("does not accept OSC 52"), "{refused}");
    assert!(!refused.contains('\u{1b}'), "{refused}");
    assert!(harness.app().route == Route::Home);

    harness.app_mut().world.host.osc52 = true;
    harness.app_mut().world.host.osc52_limit = 4;
    harness.app_mut().copy_osc52("cargo build");
    harness.draw();
    let limited = harness.text();
    assert!(
        limited.contains("exceed the terminal's 4 byte OSC 52 limit"),
        "{limited}"
    );
    assert!(!limited.contains('\u{1b}'));

    harness.app_mut().world.host.osc52_limit = 100_000;
    harness.app_mut().copy_osc52("cargo build");
    harness.draw();
    let flushed = harness.text();
    assert!(
        flushed.contains(
            "Copied · simulated clipboard · OSC 52 sequence was not written · cargo build"
        ),
        "{flushed}"
    );
    assert!(!flushed.contains("flushed"), "{flushed}");
    assert!(!flushed.contains('\u{1b}'));
    assert!(harness.app().route == Route::Home);
}
#[test]
fn closed_pages_open_from_commands_and_leave_the_home_frame_unchanged() {
    let mut closed = app(Scenario::DiskCleanup);
    assert!(closed.app().route == Route::Disk);
    let _ = closed.key(KeyCode::Esc);
    assert!(closed.app().route == Route::Home);
    assert!(closed.app().overlay.is_none());
    let before = closed.text();
    assert!(!before.contains("Largest files"), "{before}");
    assert!(!before.contains("Review cleanup"), "{before}");
    assert!(!before.contains('\u{1b}'));
    let candidates = closed
        .app()
        .world
        .disk
        .as_ref()
        .map(|disk| disk.candidates.clone());
    let pages = [
        (KeyCode::F(2), "Largest files"),
        (KeyCode::F(3), "Review cleanup"),
        (KeyCode::F(4), "Folder browser"),
        (KeyCode::F(5), "Find in home"),
        (KeyCode::F(6), "Custom action"),
        (KeyCode::F(7), "Snapshot page"),
        (KeyCode::F(8), "Trust this definition"),
    ];
    for (key, title) in pages {
        let mut harness = app(Scenario::DiskCleanup);
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().route == Route::Home, "{title}");
        let _ = harness.key(key);
        let text = harness.text();
        assert!(text.contains(title), "{title} missing from {text}");
        assert!(harness.app().route == Route::Home, "{title}");
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        assert!(!text.contains('\u{1b}'), "{title}");
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().overlay.is_none(), "{title}");
        assert!(harness.app().route == Route::Home, "{title}");
        assert_eq!(
            harness
                .app()
                .world
                .disk
                .as_ref()
                .map(|disk| disk.candidates.clone()),
            candidates,
            "{title} changed disk candidates"
        );
    }
}
#[test]
fn paused_scenarios_stay_on_home_without_a_finder_stack_or_osc52() {
    for scenario in Scenario::ALL {
        let harness = app(scenario);
        let expected = match scenario.screen() {
            Screen::Disk => Route::Disk,
            Screen::Activity => Route::Activity,
            Screen::Finder | Screen::Plan => Route::Home,
            Screen::Files | Screen::Cleanup | Screen::Review | Screen::Snapshot => Route::Home,
        };
        assert!(
            harness.app().route == expected,
            "{} opened {:?}",
            scenario.name(),
            harness.app().route as u8
        );
        assert!(harness.app().overlay.is_none(), "{}", scenario.name());
        assert!(
            harness.text().contains("holla❯"),
            "{} missing brand",
            scenario.name()
        );
        assert!(
            harness.text().contains(scenario.boot_title()),
            "{} missing {}",
            scenario.name(),
            scenario.boot_title()
        );
        assert!(
            !harness.text().contains('\u{1b}'),
            "frame 0 must not emit an OSC 52 clipboard sequence: {}",
            scenario.name()
        );
    }
    assert_eq!(Scenario::ALL.len(), 35);
}
#[test]
fn every_real_app_scenario_renders_purely_with_persistent_identity() {
    for scenario in Scenario::ALL {
        let mut harness = app(scenario);
        assert!(harness.text().contains("holla❯"));
        assert!(harness.text().contains(&harness.app().world.host.name));
        let catalogue = catalogue::catalogue(&harness.app().world);
        let time = harness.app().world.now_ms();
        let text = harness.text();
        harness.draw();
        harness.draw();
        assert_eq!(harness.text(), text);
        assert_eq!(catalogue::catalogue(&harness.app().world), catalogue);
        assert_eq!(harness.app().world.now_ms(), time);
    }
}
#[test]
fn help_escape_and_quit_use_actual_dialog_routes() {
    let mut harness = app(Scenario::FirstUse);
    let _ = harness.key(KeyCode::F(1));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
    assert!(harness.text().contains("Key reference"));
    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(harness.app().overlay.is_none());
    let _ = harness.ctrl('q');
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(harness.text().contains("Quit holla?"));
    let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(harness.app().quit);
}
#[test]
fn filtered_cleanup_enters_review_and_exact_gate_applies_once() {
    let mut harness = app(Scenario::DockerCleanup);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("docker system prune");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().route, Route::Plan));
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().overlay, Some(Overlay::Gate(_))));
    let phrase = harness
        .app()
        .plan
        .as_ref()
        .unwrap()
        .plan()
        .phrase()
        .to_owned();
    let _ = harness.type_str(&phrase);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.click_id(Dialog::new(crate::screens::plan_gate::GATE).action_id(1));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert_eq!(harness.app().world.effect_revision, 1);
    assert!(harness.app().plan.as_ref().unwrap().plan().ran());
    assert!(harness.app().overlay.is_none());
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert_eq!(harness.app().world.effect_revision, 1);
}
#[test]
fn clone_picker_is_reachable_from_the_real_action_row() {
    let mut harness = app(Scenario::RustDirty);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("gh repo clone");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().overlay, Some(Overlay::Clone(_))));
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
    assert!(harness.text().contains("Primary branch"));
}
#[test]
fn action_picker_refuses_memory_changes_after_context_moves() {
    let mut harness = app(Scenario::DockerCleanup);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("docker system prune");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.ctrl('o');
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(matches!(harness.app().overlay, Some(Overlay::Actions(_))));
    let pins = harness.app().world.memory.pins.clone();
    harness.app_mut().world.cwd = "/different-folder".into();
    let _ = harness.type_str("Pin here");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(harness.app().overlay.is_none());
    assert_eq!(harness.app().world.memory.pins, pins);
    assert!(harness.app().status.as_deref().unwrap().contains("changed"));
}
#[test]
fn ssh_preview_shows_literal_resolution_and_refuses_changed_target() {
    for changed in [false, true] {
        let mut harness = app(Scenario::RemoteHost);
        let host = harness.app().world.ssh[0].clone();
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str(&format!("ssh {}", host.alias));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        let text = harness.text();
        for expected in [
            &host.host_name,
            &host.user,
            &host.chain(),
            "Host key policy",
            "Multiplexing",
        ] {
            assert!(text.contains(expected), "missing {expected}");
        }
        if changed {
            harness.app_mut().world.ssh[0].host_name = "different.example".into();
        }
        let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let status = harness.app().status.as_deref().unwrap();
        if changed {
            assert!(status.contains("SSH target changed"));
        } else {
            assert!(status.contains(&format!("Would run: ssh {}", host.alias)));
        }
        assert_eq!(harness.app().world.effect_revision, 0);
    }
}
#[test]
fn routes_and_gate_deliver_owner_lifecycle() {
    let mut harness = app(Scenario::DockerCleanup);
    assert!(harness.diagnostics().is_empty());
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("docker system prune");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "plan {:?}",
        harness.diagnostics()
    );
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "gate {:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "cancel {:?}",
        harness.diagnostics()
    );
}
#[test]
fn activity_switching_and_home_keep_viewport_lifecycle_clean() {
    let mut harness = app(Scenario::ActivitiesMulti);
    for _ in 0..3 {
        let _ = harness.ctrl('a');
        assert!(matches!(harness.app().route, Route::Activity));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    let _ = harness.key(KeyCode::Char('0'));
    assert!(matches!(harness.app().route, Route::Home));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn coalesced_deadlines_preserve_fixed_steps_and_pause() {
    for motion in [Motion::Full, Motion::Reduced, Motion::Paused] {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::FirstUse, motion, 0),
            Theme::junie(),
            120,
            40,
        );
        harness.ticks(20);
        assert_eq!(harness.app().world.now_ms(), 0);
        let _ = harness.advance(std::time::Duration::from_millis(1_800));
        let step = if motion == Motion::Paused { 0 } else { 80 };
        assert_eq!(harness.app().world.now_ms(), step);
        harness.ticks(20);
        harness.draw();
        assert_eq!(harness.app().world.now_ms(), step);
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert_eq!(harness.app().world.now_ms(), step * 2);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
#[test]
fn pointer_feedback_changes_cadence_from_last_wake_and_expires_in_fixture_time() {
    for input_at in [5_u64, 100] {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::HardCases, Motion::Full, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.advance(std::time::Duration::from_millis(input_at));
        let _ = harness.click_id(BRAND);
        assert_eq!(harness.app().fixture_time_ms(), 4_000);
        let _ = harness.advance(std::time::Duration::from_millis(
            80_u64.saturating_sub(input_at),
        ));
        assert_eq!(harness.app().fixture_time_ms(), 4_080);
        assert_eq!(
            harness.activation_feedback().unwrap().remaining,
            std::time::Duration::from_millis(60)
        );
        harness.ticks(5);
        let _ = harness.resize(121, 40);
        assert_eq!(harness.app().fixture_time_ms(), 4_080);
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert_eq!(harness.app().fixture_time_ms(), 4_160);
        assert!(harness.activation_feedback().is_none());
        let _ = harness.advance(std::time::Duration::from_millis(199));
        assert_eq!(harness.app().fixture_time_ms(), 4_160);
        let _ = harness.advance(std::time::Duration::from_millis(1));
        assert_eq!(harness.app().fixture_time_ms(), 4_360);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
#[test]
fn delayed_pointer_feedback_coalesces_and_paused_feedback_keeps_fixture_epoch() {
    for motion in [Motion::Full, Motion::Paused] {
        let mut harness = harness(
            App::for_scenario_without_history(Scenario::HardCases, motion, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.advance(std::time::Duration::from_millis(5));
        let _ = harness.click_id(BRAND);
        let _ = harness.advance(std::time::Duration::from_millis(1_800));
        let step = if motion == Motion::Paused { 0 } else { 80 };
        assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
        assert_eq!(
            harness.activation_feedback().unwrap().remaining,
            std::time::Duration::from_millis(if motion == Motion::Paused { 140 } else { 60 })
        );
        harness.ticks(10);
        assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}
#[test]
fn feedback_clock_mismatch_and_backwards_epoch_refuse_all_fixture_progress() {
    let snapshot = |app: &App| {
        (
            app.fixture_time_ms(),
            app.effect_revision(),
            app.world
                .discovery
                .iter()
                .map(|mark| (mark.domain, mark.at_ms, mark.fails, mark.done, mark.failed))
                .collect::<Vec<_>>(),
            catalogue::catalogue(&app.world),
        )
    };
    for clock in [
        termrock::FeedbackClock::Elapsed,
        termrock::FeedbackClock::Simulation {
            initial: termrock::SimulationMoment::from_millis(5_000),
        },
    ] {
        let theme = Theme::junie();
        let app = App::for_scenario_without_history(Scenario::HardCases, Motion::Full, 4_000)
            .with_source_theme(theme.clone());
        let expected = snapshot(&app);
        let mut harness = Harness::new_with_feedback_clock(app, theme, 120, 40, clock);
        assert_eq!(snapshot(harness.app()), expected);
        assert!(!harness.app().world.clock.running);
        assert_eq!(harness.app().motion, Motion::Paused);
        assert!(harness.text().contains("feedback clock mismatch"));
        let _ = harness.advance(std::time::Duration::from_millis(1_800));
        assert_eq!(snapshot(harness.app()), expected);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[test]
fn modal_status_ages_only_with_admitted_virtual_steps() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 4_000),
        Theme::junie(),
        120,
        40,
    );
    let _ = harness.ctrl('s');
    let _ = harness.key(KeyCode::F(1));
    for _ in 0..24 {
        let _ = harness.advance(std::time::Duration::from_millis(200));
        assert!(harness.app().status.is_some());
    }
    let _ = harness.advance(std::time::Duration::from_millis(200));
    assert_eq!(harness.app().world.now_ms(), 9_000);
    assert!(harness.app().status.is_none());
    assert!(harness.app().overlay.is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn early_input_and_unrelated_ticks_cannot_admit_simulation_time() {
    let mut harness = harness(
        App::for_scenario_without_history(Scenario::FirstUse, Motion::Full, 0),
        Theme::junie(),
        120,
        40,
    );
    for _ in 0..10 {
        let _ = harness.advance(std::time::Duration::from_micros(600));
        let _ = harness.tick();
    }
    let _ = harness.key(KeyCode::F(1));
    assert_eq!(harness.app().world.now_ms(), 0);
    let _ = harness.advance(std::time::Duration::from_millis(74));
    assert_eq!(harness.app().world.now_ms(), 80);
}
#[test]
fn footer_status_survives_focused_hints_and_edit_badge_uses_top_context() {
    let mut harness = app(Scenario::DockerCleanup);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("docker system prune");
    let _ = harness.ctrl('s');
    assert!(harness.row(39).contains("EDIT"));
    assert!(harness.row(39).contains("Scope: here"));
    let _ = harness.key(KeyCode::F(1));
    assert!(!harness.row(39).contains("EDIT"));
    assert!(harness.row(39).contains("Scope: here"));
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.row(39).contains("EDIT"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );

    let mut harness = app(Scenario::DockerCleanup);
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("docker system prune");
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.tab_to(plan::STEPS));
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.row(39).contains("EDIT"));
    let _ = harness.key(KeyCode::Enter);
    assert!(!harness.row(39).contains("EDIT"));
    assert_eq!(harness.app().world.effect_revision, 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
