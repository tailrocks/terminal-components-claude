use ratatui_core::layout::Rect;
use termrock_runtime::Ui;
use termrock_theme::{FgStep, Modifier, Role, StylePatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportRows {
    pub heading: u16,

    pub subheading: u16,

    pub empty: u16,

    pub body: u16,
}

impl Default for ReportRows {
    fn default() -> Self {
        Self {
            heading: 0,
            subheading: 1,
            empty: 2,
            body: 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReportContent<'a> {
    pub heading: &'a str,

    pub subheading: Option<&'a str>,

    pub empty: Option<&'a str>,

    pub body: &'a [String],
}

impl<'a> ReportContent<'a> {
    pub const fn heading(heading: &'a str) -> Self {
        Self {
            heading,
            subheading: None,
            empty: None,
            body: &[],
        }
    }

    #[must_use]
    pub const fn with_subheading(mut self, subheading: &'a str) -> Self {
        self.subheading = Some(subheading);
        self
    }

    #[must_use]
    pub const fn with_empty(mut self, empty: &'a str) -> Self {
        self.empty = Some(empty);
        self
    }

    #[must_use]
    pub const fn with_body(mut self, body: &'a [String]) -> Self {
        self.body = body;
        self
    }
}

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

    #[must_use]
    pub const fn rows(mut self, rows: ReportRows) -> Self {
        self.rows = rows;
        self
    }

    #[must_use]
    pub const fn left_inset(mut self, cells: u16) -> Self {
        self.left_inset = cells;
        self
    }

    #[must_use]
    pub const fn right_inset(mut self, cells: u16) -> Self {
        self.right_inset = cells;
        self
    }

    #[must_use]
    pub const fn min_details_height(mut self, rows: u16) -> Self {
        self.min_details_height = rows;
        self
    }

    #[must_use]
    pub const fn heading_patch(mut self, patch: StylePatch) -> Self {
        self.heading_patch = patch;
        self
    }

    #[must_use]
    pub const fn body_patch(mut self, patch: StylePatch) -> Self {
        self.body_patch = patch;
        self
    }

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

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, content: &ReportContent<'_>) {
        self.draw_rows(ui, area, content, content.body.iter().map(String::as_str));
    }

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
        for line in rows
            .into_iter()
            .take(usize::from(area.height.saturating_sub(self.rows.body)))
        {
            ui.paint_str(self.band(area, row), line.as_ref(), text);
            row = row.saturating_add(1);
        }
    }
}
