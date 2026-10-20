//! A two-pane block that composes existing groups through a shared SplitPane.

use termrock_controls::{SplitAction, SplitPane, SplitPaneState};
use termrock_layout::{Constraints, Size, SplitAxis};
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
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Result<Size, SplitBlockError> {
        let first = self.first.ok_or(SplitBlockError::MissingFirst)?;
        let second = self.second.ok_or(SplitBlockError::MissingSecond)?;
        let first_size = first.measure(ui, constraints)?;
        let second_size = second.measure(ui, constraints)?;
        let pane_size = self.pane.measure(ui, constraints);
        let gap = self.pane_gap();
        let combined = match self.pane_axis() {
            SplitAxis::Horizontal => Size {
                min: (
                    first_size
                        .min
                        .0
                        .saturating_add(gap)
                        .saturating_add(second_size.min.0)
                        .max(pane_size.min.0),
                    first_size.min.1.max(second_size.min.1).max(pane_size.min.1),
                ),
                preferred: (
                    first_size
                        .preferred
                        .0
                        .saturating_add(gap)
                        .saturating_add(second_size.preferred.0),
                    first_size.preferred.1.max(second_size.preferred.1),
                ),
            },
            SplitAxis::Vertical => Size {
                min: (
                    first_size.min.0.max(second_size.min.0).max(pane_size.min.0),
                    first_size
                        .min
                        .1
                        .saturating_add(gap)
                        .saturating_add(second_size.min.1)
                        .max(pane_size.min.1),
                ),
                preferred: (
                    first_size.preferred.0.max(second_size.preferred.0),
                    first_size
                        .preferred
                        .1
                        .saturating_add(gap)
                        .saturating_add(second_size.preferred.1),
                ),
            },
        };
        Ok(Size {
            min: combined.min,
            preferred: (
                combined.preferred.0.max(combined.min.0),
                combined.preferred.1.max(combined.min.1),
            ),
        }
        .fit(constraints))
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

    fn pane_gap(&self) -> u16 {
        self.pane.gap_size()
    }
}
