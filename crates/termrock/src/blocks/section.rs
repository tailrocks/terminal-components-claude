use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};
use termrock_runtime::Ui;
use termrock_theme::{FgStep, Theme};

pub fn paint_section_header(ui: &mut Ui<'_>, area: Rect, title: &str, theme: &Theme) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let end = area.right();
    let title_style = Style::default()
        .fg(theme.color.fg[FgStep::Primary.index()])
        .add_modifier(Modifier::BOLD);
    let rule_style = Style::default().fg(theme.color.border_subtle);
    let mut x = area.x.saturating_add(ui.paint_str(
        Rect::new(area.x, area.y, area.width, 1),
        title,
        title_style,
    ));
    if x < end {
        x = x.saturating_add(1);
    }
    while x < end {
        let rule = "─".repeat(usize::from(end.saturating_sub(x)));
        let painted = ui.paint_str(Rect::new(x, area.y, end - x, 1), &rule, rule_style);
        if painted == 0 {
            break;
        }
        x = x.saturating_add(painted);
    }
    area.width
}
