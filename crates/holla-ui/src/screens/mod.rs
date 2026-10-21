pub(crate) mod actions;
pub(crate) mod activity;
pub(crate) mod cleanup;
pub(crate) mod clone;
pub(crate) mod dialogs;
pub(crate) mod disk;
pub(crate) mod files;
pub(crate) mod finder;
pub(crate) mod home;
pub(crate) mod plan;
pub(crate) mod plan_gate;
pub(crate) mod review;
pub(crate) mod snapshot;

mod context;

/// The disk and cleanup reports render the same cells as the app-owned
/// painters they replaced. The legacy painters below are the frozen oracle:
/// they are the pre-refactor code, kept verbatim so the shared report block
/// cannot drift from the frames the frozen reference pinned.
///
/// Keep the old report cells inside each allocation. The new shared block
/// must also leave every cell outside that allocation unchanged.
#[cfg(test)]
mod report_parity {
    use crate::domain::fixtures;
    use crate::scenario::Scenario;
    use crate::sim::world::World;
    use termrock::{ColorLevel, FgStep, Rect, Role, StylePatch, Theme, Ui};
    use termrock_test_support::Scene;

    fn legacy_band(area: Rect, row: u16) -> Rect {
        Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(row),
            area.width.saturating_sub(2),
            1,
        )
    }

    fn legacy_title(ui: &mut Ui<'_>, area: Rect, row: u16, text: &str) {
        let style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(termrock::Modifier::BOLD),
        );
        ui.paint_str(legacy_band(area, row), text, style);
    }

    fn legacy_muted(ui: &mut Ui<'_>, area: Rect, row: u16, text: &str) {
        let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        ui.paint_str(legacy_band(area, row), text, style);
    }

    /// The pre-refactor disk painter.
    fn legacy_disk(world: &World, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let title = format!("Disk usage · {}", world.cwd);
        legacy_title(ui, area, 0, &title);
        let Some(disk) = &world.disk else {
            legacy_muted(ui, area, 2, "No disk scan is open");
            return;
        };
        let subtitle = format!(
            "{}% full · {} entries",
            disk.used_percent(),
            disk.candidates.len()
        );
        legacy_muted(ui, area, 1, &subtitle);
        let mut row = 3_u16;
        for candidate in &disk.candidates {
            if row >= area.height {
                break;
            }
            let line = format!(
                "{} · {} · {}",
                candidate.path,
                candidate.family.label(),
                candidate.freshness.label()
            );
            legacy_muted(ui, area, row, &line);
            row = row.saturating_add(1);
        }
    }

    /// The pre-refactor cleanup painter.
    fn legacy_cleanup(world: &World, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let title_style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(termrock::Modifier::BOLD),
        );
        let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        ui.paint_str(legacy_band(area, 0), "Cleanup review", title_style);
        let Some(disk) = &world.disk else {
            if area.height > 1 {
                ui.paint_str(legacy_band(area, 2), "No cleanup candidates", muted);
            }
            return;
        };
        if area.height > 1 {
            let line = format!(
                "{} selected · nothing is deleted from this review",
                disk.candidates.len()
            );
            ui.paint_str(legacy_band(area, 1), &line, muted);
        }
        let mut row = 3_u16;
        for candidate in &disk.candidates {
            if row >= area.height {
                break;
            }
            let line = format!(
                "{} · {}",
                candidate.family.label(),
                candidate.freshness.label()
            );
            ui.paint_str(legacy_band(area, row), &line, muted);
            row = row.saturating_add(1);
        }
    }

    fn worlds() -> Vec<World> {
        let mut with_disk = fixtures::world_for(Scenario::DiskCleanup);
        // The scan finishes late in the fixture clock; the reports only have
        // body lines once it has.
        with_disk.seek(4_000);
        let mut without_disk = fixtures::world_for(Scenario::DiskCleanup);
        without_disk.seek(4_000);
        without_disk.disk = None;
        vec![with_disk, without_disk]
    }

    fn areas() -> Vec<Rect> {
        vec![
            Rect::new(0, 0, 80, 24),
            Rect::new(2, 1, 60, 20),
            Rect::new(2, 1, 40, 12),
            Rect::new(2, 1, 40, 4),
            Rect::new(2, 1, 40, 3),
            Rect::new(2, 1, 40, 2),
            Rect::new(2, 1, 40, 1),
            Rect::new(2, 1, 40, 0),
            Rect::new(2, 1, 0, 8),
        ]
    }

    fn assert_report_cells(
        legacy: &Scene,
        current: &Scene,
        area: Rect,
        level: ColorLevel,
        label: &str,
    ) {
        let mut untouched = Scene::new("report-control", Theme::junie(), level, 80, 24);
        untouched.draw(|_, _| {});
        for y in 0..24 {
            for x in 0..80 {
                let expected =
                    if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
                        legacy.buffer().cell((x, y))
                    } else {
                        untouched.buffer().cell((x, y))
                    };
                assert_eq!(
                    current.buffer().cell((x, y)),
                    expected,
                    "{label} {area:?} {level:?}, at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn the_reports_render_the_same_cells_as_the_app_owned_painters() {
        for world in worlds() {
            for area in areas() {
                for level in [
                    ColorLevel::TrueColor,
                    ColorLevel::Ansi256,
                    ColorLevel::Ansi16,
                    ColorLevel::Mono,
                ] {
                    let theme = Theme::junie();
                    let mut legacy = Scene::new("legacy-disk", theme.clone(), level, 80, 24);
                    legacy.draw(|ui, _| legacy_disk(&world, ui, area));
                    let mut current = Scene::new("report-disk", theme.clone(), level, 80, 24);
                    current.draw(|ui, _| super::disk::draw(&world, ui, area));
                    assert_report_cells(&legacy, &current, area, level, "disk");
                    assert_eq!(
                        legacy.cursor_position(),
                        current.cursor_position(),
                        "disk {area:?} {level:?}"
                    );

                    let mut legacy = Scene::new("legacy-cleanup", theme.clone(), level, 80, 24);
                    legacy.draw(|ui, _| legacy_cleanup(&world, ui, area));
                    let mut current = Scene::new("report-cleanup", theme.clone(), level, 80, 24);
                    current.draw(|ui, _| super::cleanup::draw(&world, ui, area));
                    assert_report_cells(&legacy, &current, area, level, "cleanup");
                    assert_eq!(
                        legacy.cursor_position(),
                        current.cursor_position(),
                        "cleanup {area:?} {level:?}"
                    );
                }
            }
        }
    }
}
