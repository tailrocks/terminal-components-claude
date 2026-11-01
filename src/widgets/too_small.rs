use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::theme::Theme;
use crate::ui::text::center_x;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooSmall<'a> {
    brand: &'a str,
    brand_style: Option<Style>,
    min: (u16, u16),
    quit_hint: &'a str,
}

impl<'a> TooSmall<'a> {
    pub const MIN_WIDTH: u16 = 72;
    pub const MIN_HEIGHT: u16 = 20;

    pub const fn new(brand: &'a str) -> Self {
        Self {
            brand,
            brand_style: None,
            min: (Self::MIN_WIDTH, Self::MIN_HEIGHT),
            quit_hint: "q Quit",
        }
    }

    pub const fn brand_style(mut self, style: Style) -> Self {
        self.brand_style = Some(style);
        self
    }

    pub const fn min(mut self, width: u16, height: u16) -> Self {
        self.min = (width, height);
        self
    }

    pub const fn quit_hint(mut self, hint: &'a str) -> Self {
        self.quit_hint = hint;
        self
    }

    pub fn applies(&self, area: Rect) -> bool {
        area.width < self.min.0 || area.height < self.min.1
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme) {
        let canvas = |s: Style| s.bg(t.canvas);
        let need = format!(
            "Need {}×{}, have {}×{}",
            self.min.0, self.min.1, area.width, area.height
        );
        let lines = [
            (
                self.brand,
                self.brand_style.unwrap_or_else(|| canvas(t.title())),
            ),
            ("Terminal too small", canvas(t.secondary())),
            (need.as_str(), canvas(t.muted())),
            (self.quit_hint, canvas(t.faint())),
        ];
        let y0 = area.y + area.height.saturating_sub(5) / 2;
        for (i, (text, style)) in lines.into_iter().enumerate() {
            let y = y0 + i as u16 + u16::from(i == 3);
            if y < area.bottom() {
                buf.set_string(center_x(area, text), y, text, style);
            }
        }
    }
}
