use termrock::{FgStep, Rect, Role, StylePatch, Ui};

pub(crate) const RECENT: &str = "Recent here";
pub(crate) const RECENT_EMPTY: &str = "nothing used here yet";
pub(crate) const EXPLORE: &str = "Tasks   Git   Files   Disk   Services   System";

pub(crate) fn paint_recent_lead(ui: &mut Ui<'_>, area: Rect) -> u16 {
    if area.height < 3 {
        return 0;
    }
    let title = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(termrock::Modifier::BOLD),
    );
    let muted = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    ui.paint_str(line(area, 0), RECENT, title);
    ui.paint_str(line(area, 1), RECENT_EMPTY, muted);
    ui.paint_str(line(area, 2), EXPLORE, muted);
    3
}

fn line(area: Rect, row: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(2),
        area.y.saturating_add(row),
        area.width.saturating_sub(2),
        1,
    )
}
