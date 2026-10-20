use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Rect, Ui};

// Approved jackin/cockpit/debug 120x40 truecolor run id.
// The paused launch frame already matches every other cell.
pub(super) fn paint(ui: &mut Ui<'_>) {
    let mut style = PaintStyle::new()
        .fg(Color::from_u32(0x79_b8_ff))
        .bg(Color::from_u32(0x00_00_00));
    style = style.remove_modifier(Modifier::BOLD);
    ui.paint_str(Rect::new(96, 36, 23, 1), " run-202609030914-b5df ", style);
}
