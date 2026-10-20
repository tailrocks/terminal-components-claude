//! Responsive master/detail composition built from existing groups and split layout.

use ratatui_core::layout::Rect;
use termrock_controls::{SplitPane, SplitPaneState};
use termrock_layout::{ChildPolicy, Composition, Constraints, Size, SplitAxis};
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::{CompositionSlots, ContentSlot, Cx, Measure, Response, Ui};

use crate::composition::{CompositionError, Group};

/// Responsive layout policy for a reusable master/detail block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasterDetailPolicy {
    detail_share: u8,
    detail_min: u16,
    detail_max: u16,
    gap: u16,
    collapse_below: u16,
    summary_rows: u16,
}

impl Default for MasterDetailPolicy {
    fn default() -> Self {
        Self {
            detail_share: 34,
            detail_min: 34,
            detail_max: 48,
            gap: 2,
            collapse_below: 108,
            summary_rows: 1,
        }
    }
}

impl MasterDetailPolicy {
    /// Set the desired percentage and inclusive bounds for the detail pane.
    #[must_use]
    pub const fn detail_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        self.detail_share = if percent > 100 { 100 } else { percent };
        self.detail_min = if min < max { min } else { max };
        self.detail_max = if min > max { min } else { max };
        self
    }

    #[must_use]
    pub const fn gap(mut self, cells: u16) -> Self {
        self.gap = cells;
        self
    }

    #[must_use]
    pub const fn collapse_below(mut self, width: u16) -> Self {
        self.collapse_below = width;
        self
    }

    #[must_use]
    pub const fn summary_rows(mut self, rows: u16) -> Self {
        self.summary_rows = rows;
        self
    }

    pub const fn detail_share_percent(self) -> u8 {
        self.detail_share
    }

    pub const fn is_collapsed(self, width: u16) -> bool {
        width < self.collapse_below
    }
}

/// State for the narrow detail drawer and its shared focus return target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MasterDetailState {
    drawer_open: bool,
    focus_return: Option<crate::Id>,
}

impl MasterDetailState {
    pub const fn drawer_open(&self) -> bool {
        self.drawer_open
    }
}

/// Typed result emitted when the narrow detail drawer opens or closes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MasterDetailAction {
    DrawerOpened,
    DrawerClosed,
}

/// Missing content or a composition failure detected before rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MasterDetailError {
    MissingMaster,
    MissingDetail,
    Composition(CompositionError),
}

impl From<CompositionError> for MasterDetailError {
    fn from(error: CompositionError) -> Self {
        Self::Composition(error)
    }
}

/// A responsive block that composes named master, detail, and optional
/// summary groups through the shared layout resolver.
pub struct MasterDetail<'a> {
    id: crate::Id,
    master_owner: crate::Id,
    detail_owner: crate::Id,
    policy: MasterDetailPolicy,
    master: Option<&'a Group<'a>>,
    detail: Option<&'a Group<'a>>,
    summary: Option<&'a Group<'a>>,
}

impl<'a> MasterDetail<'a> {
    pub const fn new(id: crate::Id, master_owner: crate::Id, detail_owner: crate::Id) -> Self {
        Self {
            id,
            master_owner,
            detail_owner,
            policy: MasterDetailPolicy {
                detail_share: 34,
                detail_min: 34,
                detail_max: 48,
                gap: 2,
                collapse_below: 108,
                summary_rows: 1,
            },
            master: None,
            detail: None,
            summary: None,
        }
    }

    #[must_use]
    pub const fn policy(mut self, policy: MasterDetailPolicy) -> Self {
        self.policy = policy;
        self
    }

    #[must_use]
    pub fn master(mut self, group: &'a Group<'a>) -> Self {
        self.master = Some(group);
        self
    }

    #[must_use]
    pub fn detail(mut self, group: &'a Group<'a>) -> Self {
        self.detail = Some(group);
        self
    }

    #[must_use]
    pub fn summary(mut self, group: &'a Group<'a>) -> Self {
        self.summary = Some(group);
        self
    }

    /// Open the narrow detail drawer and move focus to its configured owner.
    pub fn open_detail(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if state.drawer_open {
            return Response::consumed();
        }
        state.drawer_open = true;
        state.focus_return = Some(self.master_owner);
        cx.focus(self.detail_owner);
        Response::action(MasterDetailAction::DrawerOpened).for_id(self.id)
    }

    /// Close the narrow detail drawer and restore focus to the master owner.
    pub fn close_detail(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if !state.drawer_open {
            return Response::ignored();
        }
        state.drawer_open = false;
        cx.focus(state.focus_return.take().unwrap_or(self.master_owner));
        Response::action(MasterDetailAction::DrawerClosed).for_id(self.id)
    }

    /// Dismiss an open drawer on Escape before the configured detail owner
    /// handles the same input. Call before updating that detail control, and
    /// return the consumed response immediately. Other intents remain available
    /// to the detail control through the normal update pass.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if state.drawer_open
            && self.policy.is_collapsed(cx.viewport().width)
            && cx.intents(self.detail_owner).any(
                |intent| matches!(intent, crate::Intent::Key(key) if key.is(crate::KeyCode::Esc)),
            )
        {
            self.close_detail(cx, state)
        } else {
            Response::ignored()
        }
    }

    /// Measure the active responsive layout without input, drawing, or domain effects.
    pub fn measure(
        &self,
        ui: &Ui<'_>,
        state: &MasterDetailState,
        constraints: Constraints,
    ) -> Result<Size, MasterDetailError> {
        let master = self.master.ok_or(MasterDetailError::MissingMaster)?;
        let detail = self.detail.ok_or(MasterDetailError::MissingDetail)?;
        master.validate_depth()?;
        detail.validate_depth()?;
        if let Some(summary) = self.summary {
            summary.validate_depth()?;
        }

        if constraints.max.0 >= self.policy.collapse_below {
            self.wide_split(master, detail)
                .measure(ui, &SplitPaneState::default(), constraints)
                .map_err(|error| match error {
                    crate::blocks::SplitBlockError::Composition(error) => {
                        MasterDetailError::Composition(error)
                    }
                    crate::blocks::SplitBlockError::MissingFirst => {
                        MasterDetailError::MissingMaster
                    }
                    crate::blocks::SplitBlockError::MissingSecond => {
                        MasterDetailError::MissingDetail
                    }
                })
        } else if state.drawer_open {
            detail.measure(ui, constraints).map_err(Into::into)
        } else {
            self.narrow_slots(master, constraints.max.1)
                .measure(ui, constraints)
                .map_err(Into::into)
        }
    }

    /// Draw into the framework-owned root area using the resolved shared layout.
    pub fn show(
        &self,
        ui: &mut Ui<'_>,
        state: &MasterDetailState,
    ) -> Result<(), MasterDetailError> {
        let master = self.master.ok_or(MasterDetailError::MissingMaster)?;
        let detail = self.detail.ok_or(MasterDetailError::MissingDetail)?;
        master.validate_depth()?;
        detail.validate_depth()?;
        if let Some(summary) = self.summary {
            summary.validate_depth()?;
        }

        if ui.full().width >= self.policy.collapse_below {
            self.wide_split(master, detail)
                .show(ui, &SplitPaneState::default())
                .map_err(|error| match error {
                    crate::blocks::SplitBlockError::Composition(error) => {
                        MasterDetailError::Composition(error)
                    }
                    crate::blocks::SplitBlockError::MissingFirst => {
                        MasterDetailError::MissingMaster
                    }
                    crate::blocks::SplitBlockError::MissingSecond => {
                        MasterDetailError::MissingDetail
                    }
                })
        } else if state.drawer_open {
            detail.show(ui).map_err(Into::into)
        } else {
            ui.compose(&self.narrow_slots(master, ui.full().height))
                .map_err(Into::into)
        }
    }

    fn wide_split(
        &self,
        master: &'a Group<'a>,
        detail: &'a Group<'a>,
    ) -> crate::blocks::SplitBlock<'a> {
        crate::blocks::SplitBlock::new(
            SplitPane::new(self.id, SplitAxis::Horizontal)
                .gap(self.policy.gap)
                .min_first(1)
                .min_second(self.policy.detail_min)
                .resizable(false)
                .fixed_second_share(
                    self.policy.detail_share,
                    self.policy.detail_min,
                    self.policy.detail_max,
                )
                .seam_start(0),
        )
        .first(master)
        .second(detail)
    }

    fn narrow_slots(&self, master: &'a Group<'a>, height: u16) -> CompositionSlots<'a> {
        let summary_rows = self.policy.summary_rows.min(height / 3);
        if let Some(summary) = self.summary.filter(|_| summary_rows > 0) {
            CompositionSlots::column([
                ContentSlot::new(master, ChildPolicy::default().flex(1)),
                ContentSlot::new(
                    summary,
                    ChildPolicy::default()
                        .min(0, summary_rows)
                        .max(None, Some(summary_rows)),
                ),
            ])
            .spec(Composition::new().gap(1))
        } else {
            CompositionSlots::column([ContentSlot::new(master, ChildPolicy::default().flex(1))])
        }
    }
}

impl Measure for Group<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.measure(ui, constraints).unwrap_or(Size {
            min: (0, 0),
            preferred: (0, 0),
        })
    }
}

impl CompositionLeaf for Group<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        let _ = ui.with_area(logical, |ui| self.show(ui));
    }
}
