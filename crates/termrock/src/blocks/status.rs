//! A single-row status bar painter shared by app chrome.
//!
//! Screens that paint their own status line (shell footers, workbench
//! chrome, preview hosts) walk the same cursor: leading segments laid
//! left to right, trailing segments right-aligned, one blank column
//! between segments, leading trailing segments dropped while they
//! collide. This helper owns that walk so the alignment rule lives in
//! one place.

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use termrock_runtime::Ui;
use termrock_text::width;
use termrock_theme::{FgStep, Theme};

/// Ink of one status segment, resolved against the passed [`Theme`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StatusTone {
    /// Primary foreground ink.
    #[default]
    Normal,
    /// Secondary foreground ink.
    Secondary,
    /// Success ink.
    Success,
    /// Danger ink.
    Danger,
}

/// One segment in [`paint_status`].
pub struct StatusSeg<'a> {
    /// Segment text.
    pub text: &'a str,
    /// Ink, resolved against the passed [`Theme`].
    pub tone: StatusTone,
}

fn tone_style(tone: StatusTone, theme: &Theme) -> Style {
    let color = match tone {
        StatusTone::Normal => theme.color.fg[FgStep::Primary.index()],
        StatusTone::Secondary => theme.color.fg[FgStep::Secondary.index()],
        StatusTone::Success => theme.color.success,
        StatusTone::Danger => theme.color.danger,
    };
    Style::default().fg(color)
}

/// Paint a status bar on the first line of `area`.
///
/// `left` segments start at `area.x`, `right` segments end at
/// `area.right()`, each painted with [`Ui::paint_str`]. Segments inside
/// a side are separated by one blank column. Leading `right` segments
/// are dropped while the two sides would overlap, so the bar never shows
/// a half segment on a narrow row. Only the first line (`area.y`) is
/// touched. Returns the painted text width in cells, excluding the
/// separator columns.
pub fn paint_status(
    ui: &mut Ui<'_>,
    area: Rect,
    left: &[StatusSeg<'_>],
    right: &[StatusSeg<'_>],
    theme: &Theme,
) -> u16 {
    if area.is_empty() {
        return 0;
    }
    let end = area.right();
    let mut painted = 0u16;
    let mut x = area.x;
    for (index, seg) in left.iter().enumerate() {
        if index > 0 {
            if x >= end {
                break;
            }
            x = x.saturating_add(1);
        }
        if x >= end {
            break;
        }
        painted = painted.saturating_add(ui.paint_str(
            Rect::new(x, area.y, end - x, 1),
            seg.text,
            tone_style(seg.tone, theme),
        ));
        x = x.saturating_add(width(seg.text));
    }
    let left_end = x.min(end);
    let (first, total) = {
        let mut first = 0usize;
        loop {
            let total = right[first..]
                .iter()
                .map(|seg| width(seg.text))
                .sum::<u16>()
                .saturating_add(right.len().saturating_sub(first).saturating_sub(1) as u16);
            if first >= right.len() || left_end.saturating_add(total) <= end {
                break (first, total);
            }
            first = first.saturating_add(1);
        }
    };
    let mut x = end.saturating_sub(total);
    for (offset, seg) in right[first..].iter().enumerate() {
        if offset > 0 {
            x = x.saturating_add(1);
        }
        painted = painted.saturating_add(ui.paint_str(
            Rect::new(x, area.y, end.saturating_sub(x), 1),
            seg.text,
            tone_style(seg.tone, theme),
        ));
        x = x.saturating_add(width(seg.text));
    }
    painted
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::layout::Rect;

    use crate::{App, Cx, Response, Runtime, Theme, Ui};

    use super::{StatusSeg, StatusTone, paint_status};

    struct StatusApp {
        area: Rect,
        left: Vec<(String, StatusTone)>,
        right: Vec<(String, StatusTone)>,
        painted: std::cell::Cell<u16>,
    }

    impl StatusApp {
        fn new(area: Rect, left: Vec<(&str, StatusTone)>, right: Vec<(&str, StatusTone)>) -> Self {
            let owned = |segs: Vec<(&str, StatusTone)>| {
                segs.into_iter()
                    .map(|(text, tone)| (text.to_string(), tone))
                    .collect()
            };
            Self {
                area,
                left: owned(left),
                right: owned(right),
                painted: std::cell::Cell::new(u16::MAX),
            }
        }
    }

    impl App for StatusApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let left: Vec<StatusSeg<'_>> = self
                .left
                .iter()
                .map(|(text, tone)| StatusSeg {
                    text: text.as_str(),
                    tone: *tone,
                })
                .collect();
            let right: Vec<StatusSeg<'_>> = self
                .right
                .iter()
                .map(|(text, tone)| StatusSeg {
                    text: text.as_str(),
                    tone: *tone,
                })
                .collect();
            self.painted
                .set(paint_status(ui, self.area, &left, &right, ui.theme_ref()));
        }
    }

    fn render(app: StatusApp, area: Rect) -> (ratatui_core::buffer::Buffer, u16) {
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
    fn left_starts_at_left_and_right_ends_at_right() {
        let area = Rect::new(0, 0, 20, 1);
        let (buffer, painted) = render(
            StatusApp::new(
                area,
                vec![("ab", StatusTone::Normal), ("CD", StatusTone::Success)],
                vec![("xy", StatusTone::Danger)],
            ),
            Rect::new(0, 0, 20, 1),
        );
        assert_eq!(painted, 6);
        assert_eq!(row_text(&buffer, 0, 20), "ab CD             xy");
        let theme = Theme::junie();
        assert_eq!(
            buffer.cell((0, 0)).expect("left cell").fg,
            theme.color.fg[termrock_theme::FgStep::Primary.index()]
        );
        assert_eq!(
            buffer.cell((3, 0)).expect("success cell").fg,
            theme.color.success
        );
        assert_eq!(
            buffer.cell((18, 0)).expect("right cell").fg,
            theme.color.danger
        );
    }

    #[test]
    fn colliding_right_segments_are_dropped() {
        let area = Rect::new(0, 0, 8, 1);
        let (buffer, painted) = render(
            StatusApp::new(
                area,
                vec![("abcdef", StatusTone::Normal)],
                vec![("XYZ", StatusTone::Secondary)],
            ),
            Rect::new(0, 0, 8, 1),
        );
        assert_eq!(painted, 6);
        assert_eq!(row_text(&buffer, 0, 8), "abcdef  ");
    }

    #[test]
    fn leading_right_segments_drop_while_they_collide() {
        let area = Rect::new(0, 0, 10, 1);
        let (buffer, painted) = render(
            StatusApp::new(
                area,
                vec![("abcdef", StatusTone::Normal)],
                vec![("AB", StatusTone::Secondary), ("CD", StatusTone::Secondary)],
            ),
            Rect::new(0, 0, 10, 1),
        );
        assert_eq!(painted, 8);
        assert_eq!(row_text(&buffer, 0, 10), "abcdef  CD");
    }

    #[test]
    fn empty_area_paints_nothing() {
        for area in [Rect::new(5, 0, 0, 1), Rect::new(0, 0, 4, 0)] {
            let (_, painted) = render(
                StatusApp::new(
                    area,
                    vec![("x", StatusTone::Normal)],
                    vec![("y", StatusTone::Normal)],
                ),
                Rect::new(0, 0, 8, 1),
            );
            assert_eq!(painted, 0);
        }
    }
}
