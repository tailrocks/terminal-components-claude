use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use termrock_runtime::Ui;

pub fn paint_row(ui: &mut Ui<'_>, area: Rect, segments: &[(&str, Style)]) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let mut x = area.x;
    let end = area.right();
    for (text, style) in segments {
        if x >= end {
            break;
        }
        let painted = ui.paint_str(Rect::new(x, area.y, end - x, 1), text, *style);
        x = x.saturating_add(painted);
    }
    x.saturating_sub(area.x)
}
