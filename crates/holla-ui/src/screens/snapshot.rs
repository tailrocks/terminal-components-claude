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
    ui.paint_str(band(area, 0), "Snapshot", title_style);
    if area.height < 2 {
        return;
    }
    let frame = format!("Frame {} · {}", world.now_ms().max(0), world.cwd);
    ui.paint_str(band(area, 1), &frame, muted);
}

fn band(area: Rect, row: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(row),
        area.width.saturating_sub(2),
        1,
    )
}
