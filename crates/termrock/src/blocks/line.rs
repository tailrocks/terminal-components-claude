//! A single-line row painter shared by list-like screens.
//!
//! Screens that paint their own rows (pickers, home lists, table chrome)
//! walk the same cursor: fill once, then lay text segments left to right,
//! clipping at the row end. This helper owns that walk so the clipping rule
//! lives in one place.

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use termrock_runtime::Ui;

/// Paint one row of text segments left to right starting at `area.x`.
///
/// Each segment is painted with [`Ui::paint_str`], which clips at the
/// remaining width, so a segment longer than the budget is truncated and
/// later segments are skipped once the row is full. Only the first line
/// (`area.y`) is touched. Returns the total painted width in cells.
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

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::{Color, Style};

    use crate::{App, Cx, Response, Runtime, Theme, Ui};

    use super::paint_row;

    struct RowApp {
        area: Rect,
        segments: Vec<(String, Style)>,
        painted: std::cell::Cell<u16>,
    }

    impl RowApp {
        fn new(area: Rect, segments: Vec<(&str, Style)>) -> Self {
            Self {
                area,
                segments: segments
                    .into_iter()
                    .map(|(text, style)| (text.to_string(), style))
                    .collect(),
                painted: std::cell::Cell::new(u16::MAX),
            }
        }
    }

    impl App for RowApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let segments: Vec<(&str, Style)> = self
                .segments
                .iter()
                .map(|(text, style)| (text.as_str(), *style))
                .collect();
            self.painted.set(paint_row(ui, self.area, &segments));
        }
    }

    fn render(app: RowApp, area: Rect) -> (ratatui_core::buffer::Buffer, u16) {
        let mut runtime = Runtime::new(app, Theme::junie());
        let prepared = runtime.prepare(area).expect("runtime prepares a frame");
        let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
        terminal
            .draw(|frame| {
                runtime
                    .draw_prepared(prepared, frame)
                    .expect("prepared frame matches terminal bounds");
            })
            .expect("test backend draws successfully");
        runtime
            .acknowledge_presented(prepared)
            .expect("test frame is acknowledged");
        let painted = runtime.app().painted.get();
        (terminal.backend().buffer().clone(), painted)
    }

    fn row_text(buffer: &ratatui_core::buffer::Buffer, y: u16, width: u16) -> String {
        let mut rendered = String::new();
        for x in 0..width {
            if let Some(cell) = buffer.cell((x, y)) {
                rendered.push_str(cell.symbol());
            }
        }
        rendered
    }

    #[test]
    fn paints_segments_left_to_right_in_order() {
        let red = Style::default().fg(Color::Red);
        let blue = Style::default().fg(Color::Blue);
        let (buffer, painted) = render(
            RowApp::new(
                Rect::new(2, 1, 10, 1),
                vec![("ab", red), ("CDE", blue), ("fg", red)],
            ),
            Rect::new(0, 0, 20, 3),
        );
        assert_eq!(painted, 7);
        assert_eq!(&row_text(&buffer, 1, 20)[2..9], "abCDEfg");
        assert_eq!(
            buffer.cell((2, 1)).expect("first cell").fg,
            Color::Red,
            "first segment keeps its style"
        );
        assert_eq!(
            buffer.cell((4, 1)).expect("boundary cell").fg,
            Color::Blue,
            "second segment starts where the first ends"
        );
        assert_eq!(row_text(&buffer, 0, 20).trim(), "");
        assert_eq!(row_text(&buffer, 2, 20).trim(), "");
    }

    #[test]
    fn clips_at_the_row_end_and_reports_painted_width() {
        let style = Style::default().fg(Color::Green);
        let (buffer, painted) = render(
            RowApp::new(Rect::new(0, 0, 4, 1), vec![("abcdef", style), ("X", style)]),
            Rect::new(0, 0, 8, 1),
        );
        assert_eq!(painted, 4);
        assert_eq!(row_text(&buffer, 0, 8), "abcd    ");
    }

    #[test]
    fn empty_area_paints_nothing() {
        for area in [Rect::new(5, 0, 0, 1), Rect::new(0, 0, 4, 0)] {
            let (buffer, painted) = render(
                RowApp::new(area, vec![("x", Style::default())]),
                Rect::new(0, 0, 8, 1),
            );
            assert_eq!(painted, 0);
            assert_eq!(row_text(&buffer, 0, 8), "        ");
        }
    }
}
