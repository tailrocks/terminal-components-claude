use crate::sim::world::World;
use termrock::shell::{ReportContent, TextReport};
use termrock::{Rect, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    let Some(disk) = &world.disk else {
        TextReport::new().draw(
            ui,
            area,
            &ReportContent::heading("Cleanup review").with_empty("No cleanup candidates"),
        );
        return;
    };
    let subtitle = format!(
        "{} selected · nothing is deleted from this review",
        disk.candidates.len()
    );
    let body: Vec<String> = disk
        .candidates
        .iter()
        .map(|candidate| {
            format!(
                "{} · {}",
                candidate.family.label(),
                candidate.freshness.label()
            )
        })
        .collect();
    TextReport::new().draw(
        ui,
        area,
        &ReportContent::heading("Cleanup review")
            .with_subheading(&subtitle)
            .with_body(&body),
    );
}
