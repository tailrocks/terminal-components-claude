//! A single-row section header painter shared by grouped screens.
//!
//! Screens that group rows under a titled rule (pickers, option pages,
//! demo sections) walk the same cursor: the bold title on the left, one
//! blank column, then a horizontal rule to the row end. This helper owns
//! that walk so the header rule lives in one place.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};
use termrock_runtime::Ui;
use termrock_theme::{FgStep, Theme};

/// Paint a section header on the first line of `area`.
///
/// The title paints bold in the primary ink with [`Ui::paint_str`], one
/// blank column follows, then `─` fills the rest of the row in the
/// subtle border ink. A title that fills the row leaves no room for the
/// rule; only the first line (`area.y`) is touched. Returns the row
/// width in cells, or zero for an empty area.
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

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Modifier;

    use crate::{App, Cx, Response, Runtime, Theme, Ui};

    use super::paint_section_header;

    struct HeaderApp {
        area: Rect,
        title: String,
        painted: std::cell::Cell<u16>,
    }

    impl HeaderApp {
        fn new(area: Rect, title: &str) -> Self {
            Self {
                area,
                title: title.to_string(),
                painted: std::cell::Cell::new(u16::MAX),
            }
        }
    }

    impl App for HeaderApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.painted.set(paint_section_header(
                ui,
                self.area,
                self.title.as_str(),
                ui.theme_ref(),
            ));
        }
    }

    fn render(app: HeaderApp, area: Rect) -> (ratatui_core::buffer::Buffer, u16) {
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
    fn title_paints_bold_primary_with_rule_to_row_end() {
        let area = Rect::new(0, 0, 12, 1);
        let (buffer, painted) = render(HeaderApp::new(area, "Opts"), Rect::new(0, 0, 12, 1));
        assert_eq!(painted, 12);
        assert_eq!(row_text(&buffer, 0, 12), "Opts ───────");
        let theme = Theme::junie();
        let title_cell = buffer.cell((0, 0)).expect("title cell");
        assert_eq!(
            title_cell.fg,
            theme.color.fg[termrock_theme::FgStep::Primary.index()]
        );
        assert!(title_cell.modifier.contains(Modifier::BOLD));
        assert_eq!(
            buffer.cell((6, 0)).expect("rule cell").fg,
            theme.color.border_subtle
        );
    }

    #[test]
    fn full_row_title_leaves_no_room_for_rule() {
        let area = Rect::new(0, 0, 4, 1);
        let (buffer, painted) = render(HeaderApp::new(area, "abcd"), Rect::new(0, 0, 4, 1));
        assert_eq!(painted, 4);
        assert_eq!(row_text(&buffer, 0, 4), "abcd");
    }

    #[test]
    fn empty_area_paints_nothing() {
        for area in [Rect::new(5, 0, 0, 1), Rect::new(0, 0, 4, 0)] {
            let (_, painted) = render(HeaderApp::new(area, "Opts"), Rect::new(0, 0, 8, 1));
            assert_eq!(painted, 0);
        }
    }
}
