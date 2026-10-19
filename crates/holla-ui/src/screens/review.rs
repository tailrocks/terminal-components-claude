use crate::domain::action::Risk;
use crate::sim::{catalogue, world::World};
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
    let subject = catalogue::catalogue(world)
        .into_iter()
        .find(|action| action.risk != Risk::ReadOnly);
    let title = subject.as_ref().map_or_else(
        || "Review".to_owned(),
        |action| format!("Review · {}", action.title),
    );
    ui.paint_str(band(area, 0), &title, title_style);
    if area.height < 2 {
        return;
    }
    let detail = subject
        .as_ref()
        .map_or("nothing is selected for review", |action| {
            action.reason.as_str()
        });
    ui.paint_str(band(area, 1), detail, muted);
}

fn band(area: Rect, row: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(row),
        area.width.saturating_sub(2),
        1,
    )
}
