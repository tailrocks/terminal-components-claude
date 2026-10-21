//! Responsive master/detail composition built from existing groups and split layout.

use ratatui_core::layout::Rect;
pub use termrock_controls::MasterDetailState;
use termrock_controls::{
    MasterDetail as ResponsiveMasterDetail, ResetPolicy, SplitPane, SplitPaneState,
};
use termrock_layout::{ChildPolicy, Composition, Constraints, Size, SplitAxis};
use termrock_runtime::author::{self, CompositionLeaf};
use termrock_runtime::focus::{ScopeId, ScopeMode};
use termrock_runtime::keymap::Binding;
use termrock_runtime::{
    CompositionSlots, ContentSlot, Cx, FrameRead, LayoutFacts, Measure, Response, Ui,
};

use crate::composition::{CompositionError, Group};

const CLOSE_DETAIL: crate::ActionKey =
    crate::ActionKey::custom("termrock.master_detail.close_detail");
const DETAIL_BINDINGS: &[Binding<MasterDetailAction>] = &[Binding {
    action: CLOSE_DETAIL,
    chord: Some(crate::Chord::key(crate::KeyCode::Esc)),
    cmd: MasterDetailAction::DrawerClosed,
    label: "Close detail",
    priority: 0,
    visible: false,
}];

fn discard_action<A, B>(response: Response<A>) -> Response<B> {
    let flow = response.flow();
    let invalidate = response.invalidate();
    let id = response.id();
    let state = response.state();
    let mut response = match flow {
        crate::Flow::Ignored => Response::ignored(),
        crate::Flow::Consumed => Response::consumed(),
    }
    .with_state(state);
    if let Some(id) = id {
        response = response.for_id(id);
    }
    match invalidate {
        crate::Invalidate::None => response,
        crate::Invalidate::Paint => response.repaint(),
        crate::Invalidate::Layout => response.relayout(),
    }
}

/// Responsive layout policy for a reusable master/detail block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasterDetailPolicy {
    sizing: MasterDetailSizing,
    detail_share: u8,
    detail_min: u16,
    detail_max: u16,
    gap: u16,
    collapse_below: u16,
    summary_rows: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MasterDetailSizing {
    FixedDetail,
    ResizableMaster {
        initial_percent: u8,
        min_master: u16,
        min_detail: u16,
    },
}

impl Default for MasterDetailPolicy {
    fn default() -> Self {
        Self {
            sizing: MasterDetailSizing::FixedDetail,
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
        self.sizing = MasterDetailSizing::FixedDetail;
        self.detail_share = if percent > 100 { 100 } else { percent };
        self.detail_min = if min < max { min } else { max };
        self.detail_max = if min > max { min } else { max };
        self
    }

    /// Opt into a user-resizable master pane with a persistent initial share.
    ///
    /// The existing fixed-detail policy remains the default. The initial share
    /// seeds state only before its split state has been initialized; resizing
    /// and redraws preserve the consumer-owned [`MasterDetailState`].
    #[must_use]
    pub const fn resizable_master(
        mut self,
        initial_percent: u8,
        min_master: u16,
        min_detail: u16,
    ) -> Self {
        self.sizing = MasterDetailSizing::ResizableMaster {
            initial_percent: if initial_percent > 100 {
                100
            } else {
                initial_percent
            },
            min_master,
            min_detail,
        };
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

    /// Return the configured percentage for fixed-detail sizing.
    ///
    /// This setting is retained when [`Self::resizable_master`] is selected;
    /// it does not report that mode's initial master percentage.
    pub const fn detail_share_percent(self) -> u8 {
        self.detail_share
    }

    const fn resizable_master_config(self) -> Option<(u8, u16, u16)> {
        match self.sizing {
            MasterDetailSizing::FixedDetail => None,
            MasterDetailSizing::ResizableMaster {
                initial_percent,
                min_master,
                min_detail,
            } => Some((initial_percent, min_master, min_detail)),
        }
    }

    pub const fn is_collapsed(self, width: u16) -> bool {
        width < self.collapse_below
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
    detail_owners: Vec<crate::Id>,
}

impl<'a> MasterDetail<'a> {
    pub const fn new(id: crate::Id, master_owner: crate::Id, detail_owner: crate::Id) -> Self {
        Self {
            id,
            master_owner,
            detail_owner,
            policy: MasterDetailPolicy {
                sizing: MasterDetailSizing::FixedDetail,
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
            detail_owners: Vec::new(),
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

    /// Identify focusable descendants owned by the detail group.
    #[must_use]
    pub fn detail_owns(mut self, ids: &[crate::Id]) -> Self {
        self.detail_owners = ids.to_vec();
        self
    }

    /// Open the narrow detail drawer and move focus to its configured owner.
    pub fn open_detail(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if state.drawer_open() {
            return Response::consumed();
        }
        self.responsive_engine().open_drawer(state, cx);
        Response::action(MasterDetailAction::DrawerOpened).for_id(self.id)
    }

    /// Close the narrow detail drawer and restore focus to the master owner.
    pub fn close_detail(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if !state.drawer_open() {
            return Response::ignored();
        }
        self.responsive_engine().close_drawer(state, cx);
        Response::action(MasterDetailAction::DrawerClosed).for_id(self.id)
    }

    /// Close an open narrow drawer only for the scoped bubble binding that the
    /// runtime routes after the focused detail control declines Escape.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut MasterDetailState,
    ) -> Response<MasterDetailAction> {
        if self.policy.resizable_master_config().is_some() {
            let was_open = state.drawer_open();
            let response = self.responsive_engine().update(cx, state);
            if state.drawer_open() != was_open {
                return Response::action(if state.drawer_open() {
                    MasterDetailAction::DrawerOpened
                } else {
                    MasterDetailAction::DrawerClosed
                })
                .for_id(self.id);
            }
            // Keep the established public action enum exhaustive/source-compatible.
            // Split changes remain observable as a consumed, changed response;
            // consumers inspect their persistent `MasterDetailState` if needed.
            return discard_action(response);
        }

        let scope = self.detail_scope();
        let responsive = self.responsive_engine();
        let narrow_drawer_is_active = state.drawer_open()
            && cx
                .layout(self.id)
                .and_then(|facts| facts.logical_area)
                .filter(|area| !area.is_empty())
                .is_some_and(|area| {
                    responsive
                        .resolve_responsive_layout(area, state.drawer_open())
                        .mode
                        == termrock_controls::MasterDetailMode::Drawer
                })
            && cx.focused_in_scope(scope).is_some();
        if narrow_drawer_is_active
            && cx.intents(self.id).any(
                |intent| matches!(intent, crate::Intent::Binding(action) if action == CLOSE_DETAIL),
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

        let responsive = self.responsive_engine();
        let resolved = responsive.resolve_responsive_layout(
            Rect::new(0, 0, constraints.max.0, constraints.max.1),
            state.drawer_open(),
        );
        if resolved.mode == termrock_controls::MasterDetailMode::Split {
            let split_state = self.measure_split_state(state);
            self.wide_split(master, detail)
                .measure(ui, &split_state, constraints)
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
        } else if resolved.mode == termrock_controls::MasterDetailMode::Drawer {
            detail.measure(ui, constraints).map_err(Into::into)
        } else {
            self.narrow_slots(master, resolved.summary.height)
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

        let area = ui.full();
        if self.policy.resizable_master_config().is_some() {
            return self
                .responsive_engine()
                .draw(ui, area, state, |ui, layout| {
                    self.draw_resizable_regions(ui, layout, master, detail)
                })
                .map_err(Into::into);
        }
        let resolved = self
            .responsive_engine()
            .resolve_responsive_layout(area, state.drawer_open());
        ui.report_layout(self.id, LayoutFacts::default().with_logical_area(area));
        if resolved.mode == termrock_controls::MasterDetailMode::Split {
            let mut result: Result<(), CompositionError> = Ok(());
            self.wide_pane()
                .draw(ui, area, &SplitPaneState::default(), |ui, first, second| {
                    result = ui.with_area(first, |ui| master.show(ui));
                    if result.is_ok() {
                        result = ui.with_area(second, |ui| {
                            ui.focus_scope(self.detail_scope().id(), ScopeMode::Normal, |ui| {
                                detail.show(ui)
                            })
                        });
                    }
                });
            result.map_err(Into::into)
        } else if resolved.mode == termrock_controls::MasterDetailMode::Drawer {
            ui.focus_scope(self.detail_scope().id(), ScopeMode::Normal, |ui| {
                let result = detail.show(ui);
                if result.is_ok() {
                    author::publish_scope_bindings(
                        ui,
                        self.detail_scope(),
                        self.id,
                        crate::StateFlags::empty(),
                        DETAIL_BINDINGS,
                    );
                }
                result
            })
            .map_err(Into::into)
        } else {
            ui.compose(&self.narrow_slots(master, resolved.summary.height))
                .map_err(Into::into)
        }
    }

    fn responsive_engine(&self) -> ResponsiveMasterDetail {
        let summary_rows = match self.policy.resizable_master_config() {
            Some(_) if self.summary.is_none() => 0,
            _ => self.policy.summary_rows,
        };
        let engine = ResponsiveMasterDetail::new(self.id, self.master_owner, self.detail_owner)
            .collapse_below(self.policy.collapse_below)
            .summary_rows(summary_rows)
            .detail_owns(&self.detail_owners);
        match self.policy.resizable_master_config() {
            Some((initial, min_master, min_detail)) => engine
                .master_share(initial)
                .min_master(min_master)
                .min_detail(min_detail)
                .gap(self.policy.gap)
                .reset_policy(ResetPolicy::Initial),
            None => engine,
        }
    }

    fn measure_split_state(&self, state: &MasterDetailState) -> SplitPaneState {
        self.policy.resizable_master_config().map_or_else(
            SplitPaneState::default,
            |(initial, _, _)| {
                state
                    .split()
                    .unwrap_or_else(|| SplitPaneState::new(initial))
            },
        )
    }

    fn wide_split(
        &self,
        master: &'a Group<'a>,
        detail: &'a Group<'a>,
    ) -> crate::blocks::SplitBlock<'a> {
        crate::blocks::SplitBlock::new(self.wide_pane())
            .first(master)
            .second(detail)
    }

    fn wide_pane(&self) -> SplitPane<'a> {
        if self.policy.resizable_master_config().is_some() {
            return self.responsive_engine().split_pane();
        }
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
            .seam_start(0)
    }

    fn draw_resizable_regions(
        &self,
        ui: &mut Ui<'_>,
        layout: termrock_controls::MasterDetailLayout,
        master: &'a Group<'a>,
        detail: &'a Group<'a>,
    ) -> Result<(), CompositionError> {
        match layout.mode {
            termrock_controls::MasterDetailMode::Split => {
                ui.with_area(layout.master, |ui| master.show(ui))?;
                ui.with_area(layout.detail, |ui| {
                    ui.focus_scope(self.detail_scope().id(), ScopeMode::Normal, |ui| {
                        detail.show(ui)
                    })
                })
            }
            termrock_controls::MasterDetailMode::Drawer => {
                ui.focus_scope(self.detail_scope().id(), ScopeMode::Normal, |ui| {
                    detail.show(ui)
                })
            }
            termrock_controls::MasterDetailMode::Master => {
                ui.with_area(layout.master, |ui| master.show(ui))?;
                if let (Some(summary), false) = (self.summary, layout.summary.is_empty()) {
                    ui.with_area(layout.summary, |ui| summary.show(ui))?;
                }
                Ok(())
            }
        }
    }

    const fn detail_scope(&self) -> ScopeId {
        ScopeId::new(self.id.sub("detail.scope"))
    }

    fn narrow_slots(&self, master: &'a Group<'a>, summary_rows: u16) -> CompositionSlots<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const ID: crate::Id = crate::Id::root("master-detail.shared-layout.test");
    const MASTER: crate::Id = crate::Id::root("master-detail.shared-layout.master");
    const DETAIL: crate::Id = crate::Id::root("master-detail.shared-layout.detail");

    #[test]
    fn facade_and_control_share_resolver_but_keep_default_breakpoints_and_rows() {
        let state = MasterDetailState::default();
        let facade = MasterDetail::new(ID, MASTER, DETAIL);
        let controls = ResponsiveMasterDetail::new(ID, MASTER, DETAIL);

        let between_defaults = Rect::new(0, 0, 104, 30);
        let facade_layout = facade
            .responsive_engine()
            .resolve_responsive_layout(between_defaults, state.drawer_open());
        let controls_layout = controls.layout(between_defaults, &state);
        assert_eq!(
            facade_layout.mode,
            termrock_controls::MasterDetailMode::Master
        );
        assert_eq!(facade_layout.summary.height, 1);
        assert_eq!(
            controls_layout.mode,
            termrock_controls::MasterDetailMode::Split
        );

        let narrow = Rect::new(0, 0, 99, 24);
        let facade_layout = facade
            .responsive_engine()
            .resolve_responsive_layout(narrow, state.drawer_open());
        let controls_layout = controls.layout(narrow, &state);
        assert_eq!(facade_layout.mode, controls_layout.mode);
        assert_eq!(facade_layout.summary.height, 1);
        assert_eq!(controls_layout.summary.height, 6);
    }
}
