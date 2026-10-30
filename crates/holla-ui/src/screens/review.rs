use crate::domain::action::Risk;
use crate::sim::{catalogue, world::World};
use termrock::{Rect, TextSection, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    let subject = catalogue::catalogue(world)
        .into_iter()
        .find(|action| action.risk != Risk::ReadOnly);
    let title = subject.as_ref().map_or_else(
        || "Review".to_owned(),
        |action| format!("Review · {}", action.title),
    );
    let detail = subject
        .as_ref()
        .map_or("nothing is selected for review", |action| {
            action.reason.as_str()
        });
    TextSection::new(&title).subtitle(detail).render(ui, area);
}
