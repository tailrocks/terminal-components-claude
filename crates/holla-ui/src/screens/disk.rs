use crate::sim::world::World;
use termrock::shell::{ReportContent, TextReport};
use termrock::{Rect, Ui};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    let title = format!("Disk usage · {}", world.cwd);
    let Some(disk) = &world.disk else {
        TextReport::new().min_details_height(0).draw(
            ui,
            area,
            &ReportContent::heading(&title).with_empty("No disk scan is open"),
        );
        return;
    };
    let subtitle = format!(
        "{}% full · {} entries",
        disk.used_percent(),
        disk.candidates.len()
    );
    let body: Vec<String> = disk
        .candidates
        .iter()
        .map(|candidate| {
            format!(
                "{} · {} · {}",
                candidate.path,
                candidate.family.label(),
                candidate.freshness.label()
            )
        })
        .collect();
    TextReport::new().min_details_height(0).draw(
        ui,
        area,
        &ReportContent::heading(&title)
            .with_subheading(&subtitle)
            .with_body(&body),
    );
}
