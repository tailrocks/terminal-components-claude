//! A plain text report: a heading, an optional subheading, an optional
//! empty-state line and a list of body lines.
//!
//! The block owns the row grid the report paints on and the columns it
//! insets its lines by. The app supplies the words and the policy; nothing
//! here knows about the domain the report describes.
//!
//! It is the multi-line sibling of `termrock_feedback::TextSection`: a
//! section paints a single body line, a report paints a list and an empty
//! state.

use ratatui_core::layout::Rect;
use termrock_runtime::Ui;
use termrock_theme::{FgStep, Modifier, Role, StylePatch};

/// The rows a report paints inside its area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportRows {
    /// The row the heading paints on.
    pub heading: u16,
    /// The row the subheading paints on.
    pub subheading: u16,
    /// The row the empty-state line paints on.
    pub empty: u16,
    /// The first row a body line paints on.
    pub body: u16,
}

impl Default for ReportRows {
    /// A heading on the first row, a subheading on the second, the empty
    /// state on the third and the body from the fourth.
    fn default() -> Self {
        Self {
            heading: 0,
            subheading: 1,
            empty: 2,
            body: 3,
        }
    }
}

/// The words a report paints.
#[derive(Debug, Clone, Copy)]
pub struct ReportContent<'a> {
    /// The heading line.
    pub heading: &'a str,
    /// The subheading under the heading.
    pub subheading: Option<&'a str>,
    /// The empty-state line, painted on the empty row.
    pub empty: Option<&'a str>,
    /// The body lines, top to bottom.
    pub body: &'a [String],
}

impl<'a> ReportContent<'a> {
    /// A report with only a heading.
    pub const fn heading(heading: &'a str) -> Self {
        Self {
            heading,
            subheading: None,
            empty: None,
            body: &[],
        }
    }

    /// The report with a subheading.
    #[must_use]
    pub const fn with_subheading(mut self, subheading: &'a str) -> Self {
        self.subheading = Some(subheading);
        self
    }

    /// The report with an empty-state line.
    #[must_use]
    pub const fn with_empty(mut self, empty: &'a str) -> Self {
        self.empty = Some(empty);
        self
    }

    /// The report with body lines.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [String]) -> Self {
        self.body = body;
        self
    }
}

/// A plain text report block.
///
/// The default grid puts the heading on the first row, the subheading on the
/// second, the empty state on the third and the body from the fourth, with
/// one column of inset on each side. Every part of that is policy and can be
/// overridden.
#[derive(Debug, Clone, Copy)]
pub struct TextReport {
    rows: ReportRows,
    left_inset: u16,
    right_inset: u16,
    min_details_height: u16,
    heading_patch: StylePatch,
    body_patch: StylePatch,
}

impl Default for TextReport {
    fn default() -> Self {
        Self::new()
    }
}

impl TextReport {
    /// A report with the default grid, insets and roles.
    pub const fn new() -> Self {
        Self {
            rows: ReportRows {
                heading: 0,
                subheading: 1,
                empty: 2,
                body: 3,
            },
            left_inset: 1,
            right_inset: 1,
            min_details_height: 2,
            heading_patch: StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
            body_patch: StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
        }
    }

    /// Set the rows the report paints on.
    #[must_use]
    pub const fn rows(mut self, rows: ReportRows) -> Self {
        self.rows = rows;
        self
    }

    /// Set the columns inset from the left edge of the area.
    #[must_use]
    pub const fn left_inset(mut self, cells: u16) -> Self {
        self.left_inset = cells;
        self
    }

    /// Set the columns inset from the right edge of the area.
    #[must_use]
    pub const fn right_inset(mut self, cells: u16) -> Self {
        self.right_inset = cells;
        self
    }

    /// The smallest area height that still paints the subheading and the
    /// empty-state line. Zero paints them whatever the area height is.
    #[must_use]
    pub const fn min_details_height(mut self, rows: u16) -> Self {
        self.min_details_height = rows;
        self
    }

    /// Set the patch the heading paints with.
    #[must_use]
    pub const fn heading_patch(mut self, patch: StylePatch) -> Self {
        self.heading_patch = patch;
        self
    }

    /// Set the patch the subheading, the empty state and the body paint with.
    #[must_use]
    pub const fn body_patch(mut self, patch: StylePatch) -> Self {
        self.body_patch = patch;
        self
    }

    /// The band one row of the report paints in.
    fn band(&self, area: Rect, row: u16) -> Rect {
        Rect::new(
            area.x.saturating_add(self.left_inset),
            area.y.saturating_add(row),
            area.width
                .saturating_sub(self.left_inset.saturating_add(self.right_inset)),
            1,
        )
        .intersection(area)
    }

    /// Paint the report into `area`.
    ///
    /// Every line is clipped to `area`. The subheading and the empty-state
    /// line also follow the minimum details height. Body rows stop at the
    /// allocated area's bottom edge.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, content: &ReportContent<'_>) {
        self.draw_rows(ui, area, content, content.body.iter().map(String::as_str));
    }

    /// Paint lazy body rows without collecting a new vector for each frame.
    /// Only rows that fit in the allocation are requested from the iterator.
    pub fn draw_rows<I, S>(&self, ui: &mut Ui<'_>, area: Rect, content: &ReportContent<'_>, rows: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if area.is_empty() {
            return;
        }
        let heading = ui.paint_patch(&self.heading_patch);
        ui.paint_str(self.band(area, self.rows.heading), content.heading, heading);
        let text = ui.paint_patch(&self.body_patch);
        if area.height >= self.min_details_height {
            if let Some(subheading) = content.subheading {
                ui.paint_str(self.band(area, self.rows.subheading), subheading, text);
            }
            if let Some(empty) = content.empty {
                ui.paint_str(self.band(area, self.rows.empty), empty, text);
            }
        }
        let mut row = self.rows.body;
        for line in rows.into_iter().take(usize::from(area.height.saturating_sub(self.rows.body))) {
            ui.paint_str(self.band(area, row), line.as_ref(), text);
            row = row.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock_test_support::Scene;
    use termrock_theme::{ColorLevel, Theme};

    fn symbol(scene: &Scene, x: u16, y: u16) -> Option<&str> {
        scene.buffer().cell((x, y)).map(|cell| cell.symbol())
    }

    #[test]
    fn paints_the_heading_the_subheading_and_the_body_on_the_default_grid() {
        let mut scene = Scene::new("text-report", Theme::junie(), ColorLevel::TrueColor, 20, 6);
        let body = vec!["first".to_string(), "second".to_string()];
        scene.draw(|ui, _area| {
            ui.fill(ui.full(), ui.surface_style());
            TextReport::new().draw(
                ui,
                Rect::new(0, 0, 20, 6),
                &ReportContent::heading("Heading")
                    .with_subheading("detail")
                    .with_body(&body),
            );
        });
        assert_eq!(symbol(&scene, 1, 0), Some("H"));
        assert_eq!(symbol(&scene, 1, 1), Some("d"));
        assert_eq!(symbol(&scene, 1, 3), Some("f"));
        assert_eq!(symbol(&scene, 1, 4), Some("s"));
        assert_eq!(symbol(&scene, 1, 2), Some(" "));
    }

    #[test]
    fn a_short_area_drops_the_subheading_and_the_body() {
        let mut scene = Scene::new("text-report", Theme::junie(), ColorLevel::TrueColor, 20, 6);
        let body = vec!["first".to_string()];
        scene.draw(|ui, _area| {
            ui.fill(ui.full(), ui.surface_style());
            TextReport::new().draw(
                ui,
                Rect::new(0, 0, 20, 1),
                &ReportContent::heading("Heading")
                    .with_subheading("detail")
                    .with_body(&body),
            );
        });
        assert_eq!(symbol(&scene, 1, 0), Some("H"));
        assert_eq!(symbol(&scene, 1, 1), Some(" "));
    }

    #[test]
    fn the_empty_state_paints_where_the_body_would_start() {
        let mut scene = Scene::new("text-report", Theme::junie(), ColorLevel::TrueColor, 20, 6);
        scene.draw(|ui, _area| {
            ui.fill(ui.full(), ui.surface_style());
            TextReport::new().draw(
                ui,
                Rect::new(0, 0, 20, 6),
                &ReportContent::heading("Heading").with_empty("nothing here"),
            );
        });
        assert_eq!(symbol(&scene, 1, 2), Some("n"));
        assert_eq!(symbol(&scene, 1, 3), Some(" "));
    }

    #[test]
    fn the_grid_the_insets_and_the_roles_are_policy() {
        let mut scene = Scene::new("text-report", Theme::junie(), ColorLevel::TrueColor, 20, 6);
        scene.draw(|ui, _area| {
            ui.fill(ui.full(), ui.surface_style());
            TextReport::new()
                .rows(ReportRows {
                    heading: 0,
                    subheading: 1,
                    empty: 1,
                    body: 2,
                })
                .left_inset(0)
                .right_inset(0)
                .draw(
                    ui,
                    Rect::new(0, 0, 20, 6),
                    &ReportContent::heading("Heading").with_empty("nothing here"),
                );
        });
        assert_eq!(symbol(&scene, 0, 0), Some("H"));
        assert_eq!(symbol(&scene, 0, 1), Some("n"));
    }

    #[test]
    fn custom_rows_and_short_areas_never_paint_outside_the_allocation() {
        for height in 0..=3 {
            for heading in [0, 7] {
                let area = Rect::new(3, 2, 8, height);
                let mut untouched = Scene::new("report-control", Theme::junie(), ColorLevel::TrueColor, 20, 10);
                untouched.draw(|_, _| {});
                let mut scene = Scene::new("report-bounds", Theme::junie(), ColorLevel::TrueColor, 20, 10);
                let body = vec!["body".to_owned(); 8];
                scene.draw(|ui, _| {
                    TextReport::new()
                        .rows(ReportRows { heading, subheading: 1, empty: 2, body: 3 })
                        .min_details_height(0)
                        .draw(ui, area, &ReportContent::heading("Heading")
                            .with_subheading("detail")
                            .with_empty("empty")
                            .with_body(&body));
                });
                for y in 0..10 {
                    for x in 0..20 {
                        if x < area.x || x >= area.right() || y < area.y || y >= area.bottom() {
                            assert_eq!(scene.buffer().cell((x, y)), untouched.buffer().cell((x, y)),
                                "outside {area:?}, heading={heading}, at {x},{y}");
                        }
                    }
                }
            }
        }
    }


    #[test]
    fn lazy_rows_do_not_format_hidden_content() {
        for (height, expected) in [(1, 0), (5, 2)] {
            let formatted = std::cell::Cell::new(0_usize);
            let rows = (0..1_000).map(|index| {
                formatted.set(formatted.get() + 1);
                format!("row {index}")
            });
            let mut scene =
                Scene::new("report-lazy", Theme::junie(), ColorLevel::TrueColor, 20, 10);
            scene.draw(|ui, _| {
                TextReport::new().draw_rows(
                    ui,
                    Rect::new(2, 1, 16, height),
                    &ReportContent::heading("Heading"),
                    rows,
                );
            });
            assert_eq!(formatted.get(), expected, "height={height}");
        }
    }

}
