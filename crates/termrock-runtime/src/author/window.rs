use core::ops::Range;

use ratatui_core::layout::Rect;

/// A visible physical window into a component's logical vertical body.
///
/// The first source row is the first row of the component's logical body that
/// is present in the visible physical body. The descriptor maps rows only; it
/// does not alter the Ui coordinate system or allocate an off-screen buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerticalWindow {
    visible_body: Rect,
    logical_rows: usize,
    first_source_row: usize,
}

/// Why a VerticalWindow could not be constructed from a clipped body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalWindowError {
    /// A clipped window must contain at least one cell.
    EmptyVisibleBody,
    /// The visible rows extend past the component's logical body.
    OutsideLogicalBody,
    /// The visible physical rows exceed the terminal coordinate range.
    PhysicalRowsOverflow,
}

impl VerticalWindow {
    /// Construct a validated non-empty physical window into a logical body.
    ///
    /// Empty intersections and source ranges outside logical_rows return a
    /// typed error.
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

    /// Describe an unclipped body whose physical rows begin at source row zero.
    ///
    /// Empty areas are permitted here to preserve zero-sized layout behavior;
    /// row mapping then returns no rows.
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

    /// Return the source rows visible in this window.
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

    /// Map an absolute physical terminal row to the component's source row.
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

    /// Map a visible source row to an absolute physical row.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_only_rows_inside_the_visible_source_window() {
        let window = VerticalWindow::new(Rect::new(4, 9, 12, 3), 8, 2).unwrap();

        assert_eq!(window.visible_source_range(), 2..5);
        assert_eq!(window.source_row_for_physical(8), None);
        assert_eq!(window.source_row_for_physical(9), Some(2));
        assert_eq!(window.source_row_for_physical(11), Some(4));
        assert_eq!(window.source_row_for_physical(12), None);
        assert_eq!(window.physical_row_for_source(1), None);
        assert_eq!(window.physical_row_for_source(2), Some(9));
        assert_eq!(window.physical_row_for_source(4), Some(11));
        assert_eq!(window.physical_row_for_source(5), None);
    }

    #[test]
    fn rejects_empty_or_out_of_range_windows_without_panicking() {
        assert_eq!(
            VerticalWindow::new(Rect::new(0, 0, 4, 0), 3, 0),
            Err(VerticalWindowError::EmptyVisibleBody)
        );
        assert_eq!(
            VerticalWindow::new(Rect::new(0, 0, 0, 1), 3, 0),
            Err(VerticalWindowError::EmptyVisibleBody)
        );
        assert_eq!(
            VerticalWindow::new(Rect::new(0, 0, 2, 2), 3, 2),
            Err(VerticalWindowError::OutsideLogicalBody)
        );
        assert_eq!(
            VerticalWindow::new(Rect::new(0, 0, 2, 1), usize::MAX, usize::MAX),
            Err(VerticalWindowError::OutsideLogicalBody)
        );
        assert_eq!(
            VerticalWindow::new(
                Rect {
                    x: 0,
                    y: u16::MAX,
                    width: 1,
                    height: 2,
                },
                2,
                0,
            ),
            Err(VerticalWindowError::PhysicalRowsOverflow)
        );
    }

    #[test]
    fn full_window_preserves_zero_size_and_checked_physical_coordinates() {
        let empty = VerticalWindow::full(Rect::new(0, 7, 0, 0));
        assert_eq!(empty.visible_source_range(), 0..0);
        assert_eq!(empty.source_row_for_physical(7), None);
        assert_eq!(empty.physical_row_for_source(0), None);

        for area in [Rect::new(0, 7, 0, 3), Rect::new(0, 7, 4, 0)] {
            let empty = VerticalWindow::full(area);
            assert_eq!(empty.visible_source_range(), 0..0);
            assert_eq!(empty.source_row_for_physical(7), None);
            assert_eq!(empty.physical_row_for_source(0), None);
        }

        let edge = VerticalWindow::new(
            Rect {
                x: 0,
                y: u16::MAX,
                width: 1,
                height: 1,
            },
            1,
            0,
        )
        .unwrap();
        assert_eq!(edge.physical_row_for_source(0), Some(u16::MAX));
        assert_eq!(edge.physical_row_for_source(1), None);
        let malformed_full = VerticalWindow::full(Rect {
            x: 0,
            y: u16::MAX,
            width: 1,
            height: 2,
        });
        assert_eq!(malformed_full.physical_row_for_source(1), None);
    }

    #[test]
    fn prepared_layout_facts_keep_the_optional_window_additive() {
        let full_area = Rect::new(2, 3, 5, 4);
        let facts = crate::ui::LayoutFacts::new(4, 9, 4, 5);
        assert_eq!(facts.vertical_window, None);

        let window = VerticalWindow::new(Rect::new(2, 4, 5, 2), 4, 1).unwrap();
        let facts = facts.with_vertical_window(window);
        assert_eq!(facts.vertical_window, Some(window));
        assert_eq!(facts.logical_area, None);
        assert_eq!(VerticalWindow::full(full_area).visible_body(), full_area);
    }
}
