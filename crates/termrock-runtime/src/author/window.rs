use core::ops::Range;

use ratatui_core::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerticalWindow {
    visible_body: Rect,
    logical_rows: usize,
    first_source_row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalWindowError {
    EmptyVisibleBody,

    OutsideLogicalBody,

    PhysicalRowsOverflow,
}

impl VerticalWindow {
    pub const fn new(
        visible_body: Rect,
        logical_rows: usize,
        first_source_row: usize,
    ) -> Result<Self, VerticalWindowError> {
        if visible_body.width == 0 || visible_body.height == 0 {
            return Err(VerticalWindowError::EmptyVisibleBody);
        }
        if visible_body
            .y
            .checked_add(visible_body.height.saturating_sub(1))
            .is_none()
        {
            return Err(VerticalWindowError::PhysicalRowsOverflow);
        }
        let Some(source_end) = first_source_row.checked_add(visible_body.height as usize) else {
            return Err(VerticalWindowError::OutsideLogicalBody);
        };
        if source_end > logical_rows {
            return Err(VerticalWindowError::OutsideLogicalBody);
        }
        Ok(Self {
            visible_body,
            logical_rows,
            first_source_row,
        })
    }

    pub const fn full(area: Rect) -> Self {
        Self {
            visible_body: area,
            logical_rows: area.height as usize,
            first_source_row: 0,
        }
    }

    pub const fn visible_body(self) -> Rect {
        self.visible_body
    }

    pub const fn logical_rows(self) -> usize {
        self.logical_rows
    }

    pub const fn first_source_row(self) -> usize {
        self.first_source_row
    }

    pub fn visible_source_range(self) -> Range<usize> {
        if self.visible_body.width == 0 || self.visible_body.height == 0 {
            return self.first_source_row..self.first_source_row;
        }
        let end = self
            .first_source_row
            .saturating_add(usize::from(self.visible_body.height))
            .min(self.logical_rows);
        self.first_source_row.min(end)..end
    }

    pub fn source_row_for_physical(self, physical_row: u16) -> Option<usize> {
        if self.visible_body.width == 0 || self.visible_body.height == 0 {
            return None;
        }
        let local = physical_row.checked_sub(self.visible_body.y)?;
        if local >= self.visible_body.height {
            return None;
        }
        let source = self.first_source_row.checked_add(usize::from(local))?;
        (source < self.logical_rows).then_some(source)
    }

    pub fn physical_row_for_source(self, source_row: usize) -> Option<u16> {
        if self.visible_body.width == 0 || self.visible_body.height == 0 {
            return None;
        }
        let local = source_row.checked_sub(self.first_source_row)?;
        if local >= usize::from(self.visible_body.height) || source_row >= self.logical_rows {
            return None;
        }
        self.visible_body.y.checked_add(u16::try_from(local).ok()?)
    }
}
