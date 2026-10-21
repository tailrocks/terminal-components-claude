use crate::sim::world::World;
use termrock::{Rect, TextSection, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    let frame = format!("Frame {} · {}", world.now_ms().max(0), world.cwd);
    TextSection::new("Snapshot")
        .subtitle(&frame)
        .render(ui, area);
}
