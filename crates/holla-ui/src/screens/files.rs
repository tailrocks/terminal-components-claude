use crate::sim::world::World;
use termrock::{Rect, TextSection, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    let title = format!("Files · {}", world.cwd);
    TextSection::new(&title)
        .subtitle("1 entry · directories first")
        .body(world.cwd.as_str())
        .render(ui, area);
}
