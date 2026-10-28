use termrock::{FgStep, Rect, Role, StylePatch, TextSection, Ui};

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
    let muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
    let content_area = Rect::new(
        area.x.saturating_add(2),
        area.y,
        area.width.saturating_sub(2),
        3,
    );
    TextSection::new(RECENT)
        .subtitle(RECENT_EMPTY)
        .body(EXPLORE)
        .body_row(2)
        .inset(0)
        .title_style(title)
        .text_patch(muted)
        .render(ui, content_area);
    3
}
