use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};
use termrock_runtime::Ui;
use termrock_text::width;
use termrock_theme::{FgStep, Theme};

pub struct Tab<'a> {
    pub label: &'a str,

    pub active: bool,
}

pub fn paint_tabs(ui: &mut Ui<'_>, area: Rect, tabs: &[Tab<'_>], theme: &Theme) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let active = Style::default()
        .fg(theme.color.accent)
        .add_modifier(Modifier::BOLD);
    let idle = Style::default().fg(theme.color.fg[FgStep::Secondary.index()]);
    let mut x = area.x;
    let end = area.right();
    for (index, tab) in tabs.iter().enumerate() {
        if index > 0 {
            if x >= end {
                break;
            }
            x = x.saturating_add(1);
        }
        let tab_width = width(tab.label).saturating_add(2);
        if x.saturating_add(tab_width) > end {
            break;
        }
        let style = if tab.active { active } else { idle };
        let text = format!(" {} ", tab.label);
        let painted = ui.paint_str(Rect::new(x, area.y, end - x, 1), &text, style);
        x = x.saturating_add(painted);
    }
    x.saturating_sub(area.x)
}
