use core::ops::Range;

use crate::ReferenceTarget;
use crate::author::{Constraints, Measure, Rect, Size, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferenceCase {
    Rest,

    Hover,

    Pressed,

    Focused,

    FocusedHover,

    Selected,

    Disabled,

    Busy,

    Error,

    Editing,

    Empty,

    Loading,

    Overflow,

    Narrow,
}

impl ReferenceCase {
    pub const ALL: [Self; 14] = [
        Self::Rest,
        Self::Hover,
        Self::Pressed,
        Self::Focused,
        Self::FocusedHover,
        Self::Selected,
        Self::Disabled,
        Self::Busy,
        Self::Error,
        Self::Editing,
        Self::Empty,
        Self::Loading,
        Self::Overflow,
        Self::Narrow,
    ];

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Rest => "rest",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Focused => "focused",
            Self::FocusedHover => "focused_hover",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::Busy => "busy",
            Self::Error => "error",
            Self::Editing => "editing",
            Self::Empty => "empty",
            Self::Loading => "loading",
            Self::Overflow => "overflow",
            Self::Narrow => "narrow",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::FocusedHover => "focus + hover",
            other => other.slug(),
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|case| case.slug() == slug)
    }
}

pub trait ReferenceSpecimen {
    fn supports(&self, case: ReferenceCase) -> bool;

    fn specimen_height(&self, _case: ReferenceCase) -> u16 {
        1
    }

    fn target(&self, _case: ReferenceCase) -> Option<ReferenceTarget> {
        None
    }

    fn draw(&self, case: ReferenceCase, ui: &mut Ui<'_>, area: Rect);
}

struct ReferenceColumn<'a> {
    label: String,
    specimen: Box<dyn ReferenceSpecimen + 'a>,
}

pub struct ReferenceGrid<'a> {
    rows: Vec<ReferenceCase>,
    columns: Vec<ReferenceColumn<'a>>,
    label_width: u16,
    column_width: u16,
}

impl<'a> ReferenceGrid<'a> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            label_width: 15,
            column_width: 15,
        }
    }

    #[must_use]
    pub fn row(mut self, case: ReferenceCase) -> Self {
        self.rows.push(case);
        self
    }

    #[must_use]
    pub fn rows(mut self, cases: &[ReferenceCase]) -> Self {
        self.rows.extend_from_slice(cases);
        self
    }

    #[must_use]
    pub fn column(
        mut self,
        label: impl Into<String>,
        specimen: impl ReferenceSpecimen + 'a,
    ) -> Self {
        self.columns.push(ReferenceColumn {
            label: label.into(),
            specimen: Box::new(specimen),
        });
        self
    }

    #[must_use]
    pub const fn label_width(mut self, width: u16) -> Self {
        self.label_width = width;
        self
    }

    #[must_use]
    pub const fn column_width(mut self, width: u16) -> Self {
        self.column_width = width;
        self
    }

    pub fn row_heights(&self) -> Vec<u16> {
        self.rows
            .iter()
            .map(|case| self.row_height(*case))
            .collect()
    }

    pub fn size(&self) -> Size {
        let column_count = u16::try_from(self.columns.len()).unwrap_or(u16::MAX);
        let width = self
            .label_width
            .saturating_add(self.column_width.saturating_mul(column_count));
        let height = self.rows.iter().fold(1u16, |total, case| {
            total.saturating_add(self.row_height(*case))
        });
        Size::exact(width, height)
    }

    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.size().fit(constraints)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Vec<usize> {
        self.draw_rows(ui, 0..self.rows.len(), true, area)
    }

    pub fn draw_rows(
        &self,
        ui: &mut Ui<'_>,
        rows: Range<usize>,
        header: bool,
        area: Rect,
    ) -> Vec<usize> {
        let area = area.intersection(ui.full());
        if area.is_empty() {
            return Vec::new();
        }

        if header {
            self.draw_header(ui, area);
        }

        let start = rows.start.min(self.rows.len());
        let end = rows.end.min(self.rows.len());
        if start >= end {
            return Vec::new();
        }

        let header_height = if header { 1 } else { 0 };
        let mut y = area.y.saturating_add(header_height);
        let mut remaining = area.height.saturating_sub(header_height);
        let mut drawn = Vec::new();

        for index in start..end {
            let case = self.rows[index];
            let height = self.row_height(case);
            if height > remaining {
                break;
            }

            let label = Rect::new(
                area.x,
                y,
                self.label_width.saturating_sub(1).min(area.width),
                1,
            )
            .intersection(area);
            let _ = ui.paint_str(label, case.label(), ui.surface_style());

            for (column_index, column) in self.columns.iter().enumerate() {
                let Some(cell) = self.cell_area(area, y, height, column_index) else {
                    break;
                };
                if cell.is_empty() {
                    continue;
                }
                if !column.specimen.supports(case) {
                    let marker = Rect::new(
                        cell.x.saturating_add(1),
                        cell.y,
                        cell.width.saturating_sub(1),
                        1,
                    )
                    .intersection(cell);
                    let _ = ui.paint_str(marker, "–", ui.surface_style());
                    continue;
                }
                let target = column.specimen.target(case);
                ui.reference(target, |ui| {
                    ui.with_area(cell, |ui| column.specimen.draw(case, ui, cell));
                });
            }

            drawn.push(index);
            y = y.saturating_add(height);
            remaining = remaining.saturating_sub(height);
        }

        drawn
    }

    fn row_height(&self, case: ReferenceCase) -> u16 {
        self.columns
            .iter()
            .filter(|column| column.specimen.supports(case))
            .map(|column| column.specimen.specimen_height(case).max(1))
            .max()
            .unwrap_or(1)
    }

    fn cell_area(&self, area: Rect, y: u16, height: u16, index: usize) -> Option<Rect> {
        let index = u16::try_from(index).unwrap_or(u16::MAX);
        let offset = self.column_width.saturating_mul(index);
        let left = self.label_width.saturating_add(offset);
        if left >= area.width {
            return None;
        }
        Some(
            Rect::new(
                area.x.saturating_add(left),
                y,
                self.column_width.saturating_sub(1),
                height,
            )
            .intersection(area),
        )
    }

    fn draw_header(&self, ui: &mut Ui<'_>, area: Rect) {
        for (index, column) in self.columns.iter().enumerate() {
            let Some(cell) = self.cell_area(area, area.y, 1, index) else {
                break;
            };
            let header = Rect::new(
                cell.x,
                cell.y,
                cell.width.min(self.column_width.saturating_sub(1)),
                1,
            )
            .intersection(area);
            let _ = ui.paint_str(header, &column.label, ui.surface_style());
        }
    }
}

impl Default for ReferenceGrid<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Measure for ReferenceGrid<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        ReferenceGrid::measure(self, ui, constraints)
    }
}
