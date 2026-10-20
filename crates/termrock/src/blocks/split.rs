//! A two-pane block that composes existing groups through a shared SplitPane.

use termrock_controls::{SplitAction, SplitPane, SplitPaneState};
use termrock_layout::{Constraints, Maximized, Size, SplitAxis};
use termrock_runtime::{Cx, Response, Ui};

use crate::composition::{CompositionError, Group};

/// A missing pane or a composition error detected before drawing begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitBlockError {
    MissingFirst,
    MissingSecond,
    Composition(CompositionError),
}

impl From<CompositionError> for SplitBlockError {
    fn from(error: CompositionError) -> Self {
        SplitBlockError::Composition(error)
    }
}

/// Compose two measured groups inside the exact layout produced by a
/// [`SplitPane`]. The pane configuration is shared by update, measurement,
/// hit testing, and drawing; state remains owned by the consumer.
pub struct SplitBlock<'a> {
    pane: SplitPane<'a>,
    first: Option<&'a Group<'a>>,
    second: Option<&'a Group<'a>>,
}

impl<'a> SplitBlock<'a> {
    pub const fn new(pane: SplitPane<'a>) -> Self {
        Self {
            pane,
            first: None,
            second: None,
        }
    }

    #[must_use]
    pub const fn first(mut self, group: &'a Group<'a>) -> Self {
        self.first = Some(group);
        self
    }

    #[must_use]
    pub const fn second(mut self, group: &'a Group<'a>) -> Self {
        self.second = Some(group);
        self
    }

    /// Forward input to the shared pane control.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut SplitPaneState) -> Response<SplitAction> {
        self.pane.update(cx, state)
    }

    /// Measure the pane and both groups without drawing or dispatching input.
    pub fn measure(
        &self,
        ui: &Ui<'_>,
        state: &SplitPaneState,
        constraints: Constraints,
    ) -> Result<Size, SplitBlockError> {
        let first = self.first.ok_or(SplitBlockError::MissingFirst)?;
        let second = self.second.ok_or(SplitBlockError::MissingSecond)?;
        first.validate_depth()?;
        second.validate_depth()?;
        let first_natural = first.measure(ui, constraints)?;
        let second_natural = second.measure(ui, constraints)?;
        let (first_constraints, second_constraints) =
            self.pane.pane_constraints(state, constraints);
        let first_allocated = if first_constraints == constraints {
            first_natural
        } else {
            first.measure(ui, first_constraints)?
        };
        let second_allocated = if second_constraints == constraints {
            second_natural
        } else {
            second.measure(ui, second_constraints)?
        };
        let maximized = state.maximized();
        let visible_first = maximized != Maximized::Second;
        let visible_second = maximized != Maximized::First;
        let first_min = axis_extent(first_natural.min, self.pane_axis());
        let second_min = axis_extent(second_natural.min, self.pane_axis());
        let first_preferred = axis_extent(first_natural.preferred, self.pane_axis()).max(first_min);
        let second_preferred =
            axis_extent(second_natural.preferred, self.pane_axis()).max(second_min);
        let along_min = self.pane.required_extent(
            state,
            if visible_first { first_min } else { 0 },
            if visible_second { second_min } else { 0 },
        );
        let along_preferred = self.pane.required_extent(
            state,
            if visible_first { first_preferred } else { 0 },
            if visible_second { second_preferred } else { 0 },
        );
        let other_axis = match (self.pane_axis(), maximized) {
            (SplitAxis::Horizontal, Maximized::None) => {
                (first_allocated.min.1, second_allocated.min.1)
            }
            (SplitAxis::Horizontal, Maximized::First) => (first_allocated.min.1, 0),
            (SplitAxis::Horizontal, Maximized::Second) => (0, second_allocated.min.1),
            (SplitAxis::Vertical, Maximized::None) => {
                (first_allocated.min.0, second_allocated.min.0)
            }
            (SplitAxis::Vertical, Maximized::First) => (first_allocated.min.0, 0),
            (SplitAxis::Vertical, Maximized::Second) => (0, second_allocated.min.0),
        };
        let other_preferred = match (self.pane_axis(), maximized) {
            (SplitAxis::Horizontal, Maximized::None) => {
                (first_allocated.preferred.1, second_allocated.preferred.1)
            }
            (SplitAxis::Horizontal, Maximized::First) => (first_allocated.preferred.1, 0),
            (SplitAxis::Horizontal, Maximized::Second) => (0, second_allocated.preferred.1),
            (SplitAxis::Vertical, Maximized::None) => {
                (first_allocated.preferred.0, second_allocated.preferred.0)
            }
            (SplitAxis::Vertical, Maximized::First) => (first_allocated.preferred.0, 0),
            (SplitAxis::Vertical, Maximized::Second) => (0, second_allocated.preferred.0),
        };
        let other_min = other_axis.0.max(other_axis.1).max(1);
        let other_preferred = other_preferred.0.max(other_preferred.1).max(other_min);
        let (min, preferred) = match self.pane_axis() {
            SplitAxis::Horizontal => (
                (along_min, other_min),
                (along_preferred.max(along_min), other_preferred),
            ),
            SplitAxis::Vertical => (
                (other_min, along_min),
                (other_preferred, along_preferred.max(along_min)),
            ),
        };
        Ok(Size { min, preferred }.fit(constraints))
    }

    /// Draw both groups in the two rectangles returned by the shared pane
    /// control. Composition depth is validated before the seam is painted.
    pub fn show(&self, ui: &mut Ui<'_>, state: &SplitPaneState) -> Result<(), SplitBlockError> {
        let first = self.first.ok_or(SplitBlockError::MissingFirst)?;
        let second = self.second.ok_or(SplitBlockError::MissingSecond)?;
        // Preflight both groups before the SplitPane can paint or register its
        // seam. Their normal show path performs the one real shared compose.
        first.validate_depth()?;
        second.validate_depth()?;
        let area = ui.full();

        let mut result = Ok(());
        self.pane
            .draw(ui, area, state, |ui, first_area, second_area| {
                result = ui.with_area(first_area, |ui| first.show(ui).map_err(Into::into));
                if result.is_ok() {
                    result = ui.with_area(second_area, |ui| second.show(ui).map_err(Into::into));
                }
            });
        result
    }

    fn pane_axis(&self) -> SplitAxis {
        self.pane.axis()
    }
}

const fn axis_extent(size: (u16, u16), axis: SplitAxis) -> u16 {
    match axis {
        SplitAxis::Horizontal => size.0,
        SplitAxis::Vertical => size.1,
    }
}
