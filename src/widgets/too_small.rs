//! The "terminal too small" screen an application shows instead of its
//! layout when the window is below its minimum size: the brand, the reason,
//! the size needed against the size there is, and how to quit, centred.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::theme::Theme;
use crate::ui::text::center_x;

/// The too-small screen. `TooSmall::new(brand).render(area, buf, &theme)`
/// after [`TooSmall::applies`] says the area is below the minimum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooSmall<'a> {
    brand: &'a str,
    brand_style: Option<Style>,
    min: (u16, u16),
    quit_hint: &'a str,
}

impl<'a> TooSmall<'a> {
    /// The smallest layout every application supports: 72×20.
    pub const MIN_WIDTH: u16 = 72;
    pub const MIN_HEIGHT: u16 = 20;

    /// The screen with `brand` on its first line, the default minimum
    /// ([`MIN_WIDTH`](Self::MIN_WIDTH)×[`MIN_HEIGHT`](Self::MIN_HEIGHT)) and
    /// the `q Quit` hint.
    pub const fn new(brand: &'a str) -> Self {
        Self {
            brand,
            brand_style: None,
            min: (Self::MIN_WIDTH, Self::MIN_HEIGHT),
            quit_hint: "q Quit",
        }
    }

    /// Draw the brand in `style`, as is (a lockup brings its own fill).
    /// Default: [`Theme::title`] on the canvas.
    pub const fn brand_style(mut self, style: Style) -> Self {
        self.brand_style = Some(style);
        self
    }

    /// The minimum size, `width`×`height`.
    pub const fn min(mut self, width: u16, height: u16) -> Self {
        self.min = (width, height);
        self
    }

    /// The last line (default `q Quit`): the keys the too-small screen
    /// accepts.
    pub const fn quit_hint(mut self, hint: &'a str) -> Self {
        self.quit_hint = hint;
        self
    }

    /// Whether `area` is below the minimum, so this screen replaces the
    /// layout.
    pub fn applies(&self, area: Rect) -> bool {
        area.width < self.min.0 || area.height < self.min.1
    }

    /// Draw the screen centred in `area`. The caller fills the background.
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
            // a blank line before the quit hint
            let y = y0 + i as u16 + u16::from(i == 3);
            if y < area.bottom() {
                buf.set_string(center_x(area, text), y, text, style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ColorLevel;

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    #[test]
    fn the_screen_says_what_is_needed_centred() {
        let t = Theme::for_level(ColorLevel::TrueColor);
        let area = Rect::new(0, 0, 40, 10);
        let screen = TooSmall::new("Brand");
        assert!(screen.applies(area));
        assert!(!screen.applies(Rect::new(0, 0, 72, 20)));
        let mut buf = Buffer::empty(area);
        screen.render(area, &mut buf, &t);
        assert_eq!(row(&buf, 2), format!("{}Brand", " ".repeat(17)));
        assert_eq!(row(&buf, 3).trim(), "Terminal too small");
        assert_eq!(row(&buf, 4).trim(), "Need 72×20, have 40×10");
        assert_eq!(row(&buf, 5), "");
        assert_eq!(row(&buf, 6).trim(), "q Quit");
        assert_eq!(buf[(17, 2)].style().bg, Some(t.canvas));
    }

    #[test]
    fn a_custom_brand_style_is_used_as_is_and_rows_clip() {
        let t = Theme::for_level(ColorLevel::TrueColor);
        let style = Style::new().fg(t.accent);
        let area = Rect::new(0, 0, 30, 2);
        let mut buf = Buffer::empty(area);
        TooSmall::new("X")
            .brand_style(style)
            .min(80, 24)
            .quit_hint("Ctrl+Q Quit")
            .render(area, &mut buf, &t);
        assert_eq!(buf[(14, 0)].style().fg, Some(t.accent));
        assert_ne!(buf[(14, 0)].style().bg, Some(t.canvas));
        assert_eq!(row(&buf, 1).trim(), "Terminal too small");
    }
}
