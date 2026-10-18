use ratatui::style::Color;
use termrock::theme::PaintStyle;

fn main() {
    let style = PaintStyle::new();
    style.bg = Some(Color::Red);
}
