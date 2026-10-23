use ratatui_core::style::Color;
use termrock_theme::PaintStyle;

fn main() {
    let mut style = PaintStyle::new();
    style = style.bg(Color::Red);
    assert_eq!(style.as_style().bg, Some(Color::Red));
}
