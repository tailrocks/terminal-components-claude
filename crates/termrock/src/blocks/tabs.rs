//! A single-row tab strip painter shared by tabbed screens.
//!
//! Screens that paint their own tab bar (table workbenches, preview
//! managers, demo pages) walk the same cursor: one padded label per tab,
//! a single blank column between tabs, tabs that no longer fit skipped
//! rather than clipped. This helper owns that walk so the spacing rule
//! lives in one place.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Modifier, Style};
use termrock_runtime::Ui;
use termrock_text::width;
use termrock_theme::{FgStep, Theme};

/// One tab in [`paint_tabs`].
pub struct Tab<'a> {
    /// Tab caption, painted with one cell of padding on each side.
    pub label: &'a str,
    /// Active tabs paint bold in the accent ink; the rest paint secondary.
    pub active: bool,
}

/// Paint a tab strip on the first line of `area`.
///
/// Each tab paints as `␣label␣` with [`Ui::paint_str`]. Tabs are separated
/// by one blank column. A tab that does not fully fit in the remaining
/// width is skipped, so a strip never shows a half label. Only the first
/// line (`area.y`) is touched. Returns the total strip width in cells,
/// including the separator columns.
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

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Modifier;

    use crate::{App, Cx, Response, Runtime, Theme, Ui};

    use super::{Tab, paint_tabs};

    struct TabsApp {
        area: Rect,
        labels: Vec<(String, bool)>,
        painted: std::cell::Cell<u16>,
    }

    impl TabsApp {
        fn new(area: Rect, tabs: Vec<(&str, bool)>) -> Self {
            Self {
                area,
                labels: tabs
                    .into_iter()
                    .map(|(label, active)| (label.to_string(), active))
                    .collect(),
                painted: std::cell::Cell::new(u16::MAX),
            }
        }
    }

    impl App for TabsApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let tabs: Vec<Tab<'_>> = self
                .labels
                .iter()
                .map(|(label, active)| Tab {
                    label: label.as_str(),
                    active: *active,
                })
                .collect();
            self.painted
                .set(paint_tabs(ui, self.area, &tabs, ui.theme_ref()));
        }
    }

    fn render(app: TabsApp, area: Rect) -> (ratatui_core::buffer::Buffer, u16) {
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
    fn active_tab_paints_accent_bold_and_idle_paints_secondary() {
        let area = Rect::new(0, 0, 20, 1);
        let (buffer, painted) = render(
            TabsApp::new(area, vec![("One", true), ("Two", false)]),
            Rect::new(0, 0, 20, 1),
        );
        assert_eq!(painted, 11);
        assert_eq!(row_text(&buffer, 0, 20), " One   Two          ");
        let theme = Theme::junie();
        let active_cell = buffer.cell((1, 0)).expect("active label cell");
        assert_eq!(active_cell.fg, theme.color.accent);
        assert!(active_cell.modifier.contains(Modifier::BOLD));
        let idle_cell = buffer.cell((7, 0)).expect("idle label cell");
        assert_eq!(
            idle_cell.fg,
            theme.color.fg[termrock_theme::FgStep::Secondary.index()]
        );
        assert!(!idle_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn tab_that_does_not_fit_is_skipped_not_clipped() {
        let area = Rect::new(0, 0, 8, 1);
        let (buffer, painted) = render(
            TabsApp::new(area, vec![("One", false), ("Two", false)]),
            Rect::new(0, 0, 8, 1),
        );
        assert_eq!(painted, 6);
        assert_eq!(row_text(&buffer, 0, 8), " One    ");
    }

    #[test]
    fn empty_area_paints_nothing() {
        for area in [Rect::new(5, 0, 0, 1), Rect::new(0, 0, 4, 0)] {
            let (_, painted) = render(
                TabsApp::new(area, vec![("One", true)]),
                Rect::new(0, 0, 8, 1),
            );
            assert_eq!(painted, 0);
        }
    }
}
