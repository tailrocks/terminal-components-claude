use crate::sim::world::World;
use termrock::{FgStep, Rect, Role, StylePatch, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let title_style = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(termrock::Modifier::BOLD),
    );
    let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    ui.paint_str(band(area, 0), "Cleanup review", title_style);
    let Some(disk) = &world.disk else {
        if area.height > 1 {
            ui.paint_str(band(area, 2), "No cleanup candidates", muted);
        }
        return;
    };
    if area.height > 1 {
        let line = format!(
            "{} selected · nothing is deleted from this review",
            disk.candidates.len()
        );
        ui.paint_str(band(area, 1), &line, muted);
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
        ui.paint_str(band(area, row), &line, muted);
        row = row.saturating_add(1);
    }
}

fn band(area: Rect, row: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(row),
        area.width.saturating_sub(2),
        1,
    )
}
