use crate::sim::world::World;
use termrock::shell::{ReportContent, TextReport};
use termrock::{Rect, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let Some(disk) = &world.disk else {
        TextReport::new().draw(
            ui,
            area,
            &ReportContent::heading("Cleanup review").with_empty("No cleanup candidates"),
        );
        return;
    };
    let subtitle = if area.height > 1 {
        format!(
            "{} selected · nothing is deleted from this review",
            disk.candidates.len()
        )
    } else {
        String::new()
    };
    let body = disk.candidates.iter().map(|candidate| {
        format!(
            "{} · {}",
            candidate.family.label(),
            candidate.freshness.label()
        )
    });
    TextReport::new().draw_rows(
        ui,
        area,
        &ReportContent::heading("Cleanup review").with_subheading(&subtitle),
        body,
    );
}
