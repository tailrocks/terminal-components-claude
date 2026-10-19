use crate::sim::world::World;
use termrock::{FgStep, Rect, Role, StylePatch, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let title = format!("Disk usage · {}", world.cwd);
    paint_title(ui, area, 0, &title);
    let Some(disk) = &world.disk else {
        paint_muted(ui, area, 2, "No disk scan is open");
        return;
    };
    let subtitle = format!(
        "{}% full · {} entries",
        disk.used_percent(),
        disk.candidates.len()
    );
    paint_muted(ui, area, 1, &subtitle);
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
        paint_muted(ui, area, row, &line);
        row = row.saturating_add(1);
    }
}

fn paint_title(ui: &mut Ui<'_>, area: Rect, row: u16, text: &str) {
    let style = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(termrock::Modifier::BOLD),
    );
    ui.paint_str(band(area, row), text, style);
}

fn paint_muted(ui: &mut Ui<'_>, area: Rect, row: u16, text: &str) {
    let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    ui.paint_str(band(area, row), text, style);
}

fn band(area: Rect, row: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(row),
        area.width.saturating_sub(2),
        1,
    )
}
