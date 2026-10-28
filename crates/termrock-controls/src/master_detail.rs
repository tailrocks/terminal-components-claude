use core::fmt;

use ratatui_core::layout::Rect;

use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
use crate::id::Id;
use crate::intent::Intent;
use crate::keymap::Binding;
use crate::layout::SplitAxis;
use crate::response::Response;
use crate::split::{ResetPolicy, SplitAction, SplitCmd, SplitPane, SplitPaneState};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MasterDetailCmd {
    OpenDetail,
    CloseDetail,
}

const OPEN_DETAIL: &[Binding<MasterDetailCmd>] = &[Binding {
    action: ActionKey::custom("master_detail.open_detail"),
    chord: Some(Chord::key(KeyCode::Tab)),
    cmd: MasterDetailCmd::OpenDetail,
    label: "Show details",
    priority: 40,
    visible: true,
}];

const CLOSE_DETAIL: &[Binding<MasterDetailCmd>] = &[
    Binding {
        action: ActionKey::custom("master_detail.close_detail"),
        chord: Some(Chord::key(KeyCode::Esc)),
        cmd: MasterDetailCmd::CloseDetail,
        label: "Back",
        priority: 40,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("master_detail.close_detail_left"),
        chord: Some(Chord::key(KeyCode::Left)),
        cmd: MasterDetailCmd::CloseDetail,
        label: "Back",
        priority: 40,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("master_detail.close_detail_backtab"),
        chord: Some(Chord::key(KeyCode::BackTab)),
        cmd: MasterDetailCmd::CloseDetail,
        label: "Back",
        priority: 40,
        visible: true,
    },
];

fn is_open_detail(action: ActionKey) -> bool {
    Binding::command(OPEN_DETAIL, action) == Some(MasterDetailCmd::OpenDetail)
}

fn is_close_detail(action: ActionKey) -> bool {
    Binding::command(CLOSE_DETAIL, action) == Some(MasterDetailCmd::CloseDetail)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MasterDetailMode {
    #[default]
    Split,

    Master,

    Drawer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MasterDetailLayout {
    pub mode: MasterDetailMode,
    pub master: Rect,
    pub detail: Rect,
    pub summary: Rect,
    pub seam: Rect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MasterDetailState {
    split: Option<SplitPaneState>,
    drawer_open: bool,
    return_focus: Option<Id>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SplitSizing {
    Resizable,
    FixedDetail { percent: u8, min: u16, max: u16 },
    FixedMaster { percent: u8, min: u16, max: u16 },
}

impl MasterDetailState {
    pub const fn split(&self) -> Option<SplitPaneState> {
        self.split
    }

    pub const fn drawer_open(&self) -> bool {
        self.drawer_open
    }

    fn open_drawer(&mut self, cx: &mut Cx<'_>, detail: Id, return_focus: Id) {
        self.drawer_open = true;
        self.return_focus = Some(return_focus);
        cx.focus(detail);
    }

    fn close_drawer(&mut self, cx: &mut Cx<'_>, master: Id) {
        self.drawer_open = false;
        cx.focus(self.return_focus.take().unwrap_or(master));
    }

    fn reconcile_drawer(&mut self, master_focused: bool, detail_focused: bool) {
        if detail_focused {
            self.drawer_open = true;
        } else if master_focused {
            self.drawer_open = false;
            self.return_focus = None;
        }
    }
}

#[derive(Clone)]
pub struct MasterDetail {
    id: Id,
    master: Id,
    detail: Id,
    master_share: u8,
    initial_master_cells: Option<u16>,
    min_master: u16,
    min_detail: u16,
    gap: u16,
    collapse_below: u16,
    summary_rows: u16,
    esc_returns: bool,
    detail_owners: Vec<Id>,
    reset_policy: ResetPolicy,
    split_sizing: SplitSizing,
    preserve_drag_offset: bool,
    resize_step_cells: u16,
    resize_bindings: Option<&'static [Binding<SplitCmd>]>,
}

impl fmt::Debug for MasterDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("MasterDetail");
        debug
            .field("id", &self.id)
            .field("master", &self.master)
            .field("detail", &self.detail)
            .field("master_share", &self.master_share);
        if self.initial_master_cells.is_some() {
            debug.field("initial_master_cells", &self.initial_master_cells);
        }
        debug
            .field("min_master", &self.min_master)
            .field("min_detail", &self.min_detail)
            .field("gap", &self.gap)
            .field("collapse_below", &self.collapse_below)
            .field("summary_rows", &self.summary_rows)
            .field("esc_returns", &self.esc_returns)
            .field("detail_owners", &self.detail_owners)
            .field("reset_policy", &self.reset_policy)
            .field("split_sizing", &self.split_sizing)
            .field("preserve_drag_offset", &self.preserve_drag_offset)
            .field("resize_step_cells", &self.resize_step_cells)
            .field("resize_bindings", &self.resize_bindings)
            .finish()
    }
}

impl MasterDetail {
    pub const fn new(id: Id, master: Id, detail: Id) -> Self {
        Self {
            id,
            master,
            detail,
            master_share: 32,
            initial_master_cells: None,
            min_master: 28,
            min_detail: 40,
            gap: 2,
            collapse_below: 100,
            summary_rows: 6,
            esc_returns: true,
            detail_owners: Vec::new(),
            reset_policy: ResetPolicy::Balanced,
            split_sizing: SplitSizing::Resizable,
            preserve_drag_offset: false,
            resize_step_cells: 1,
            resize_bindings: None,
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn master_id(&self) -> Id {
        self.master
    }

    pub const fn detail_id(&self) -> Id {
        self.detail
    }

    pub fn open_drawer(&self, state: &mut MasterDetailState, cx: &mut Cx<'_>) {
        state.open_drawer(cx, self.detail, self.master);
    }

    pub fn open_drawer_from(
        &self,
        state: &mut MasterDetailState,
        cx: &mut Cx<'_>,
        return_focus: Id,
    ) {
        state.open_drawer(cx, self.detail, return_focus);
    }

    pub fn close_drawer(&self, state: &mut MasterDetailState, cx: &mut Cx<'_>) {
        state.close_drawer(cx, self.master);
    }

    pub fn toggle_drawer(&self, state: &mut MasterDetailState, cx: &mut Cx<'_>) {
        if state.drawer_open {
            self.close_drawer(state, cx);
        } else {
            self.open_drawer(state, cx);
        }
    }

    #[must_use]
    pub const fn master_share(mut self, percent: u8) -> Self {
        self.master_share = percent;
        self.initial_master_cells = None;
        self
    }

    #[must_use]
    pub const fn initial_master_cells(mut self, cells: u16) -> Self {
        self.initial_master_cells = Some(cells);
        self.split_sizing = SplitSizing::Resizable;
        self
    }

    #[must_use]
    pub const fn fixed_detail_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.split_sizing = SplitSizing::FixedDetail {
            percent,
            min: low,
            max: high,
        };
        self
    }

    #[must_use]
    pub const fn fixed_master_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.split_sizing = SplitSizing::FixedMaster {
            percent,
            min: low,
            max: high,
        };
        self
    }

    #[must_use]
    pub const fn resizable(mut self) -> Self {
        self.split_sizing = SplitSizing::Resizable;
        self
    }

    #[must_use]
    pub const fn resize_step_cells(mut self, cells: u16) -> Self {
        self.resize_step_cells = if cells == 0 { 1 } else { cells };
        self
    }

    #[must_use]
    pub const fn preserve_drag_offset(mut self, preserve: bool) -> Self {
        self.preserve_drag_offset = preserve;
        self
    }

    #[must_use]
    pub const fn resize_bindings(mut self, table: &'static [Binding<SplitCmd>]) -> Self {
        self.resize_bindings = Some(table);
        self
    }

    #[must_use]
    pub const fn reset_policy(mut self, policy: ResetPolicy) -> Self {
        self.reset_policy = policy;
        self
    }

    #[must_use]
    pub const fn min_master(mut self, width: u16) -> Self {
        self.min_master = width;
        self
    }

    #[must_use]
    pub const fn min_detail(mut self, width: u16) -> Self {
        self.min_detail = width;
        self
    }

    #[must_use]
    pub const fn gap(mut self, columns: u16) -> Self {
        self.gap = columns;
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

    #[must_use]
    pub const fn esc_returns(mut self, yes: bool) -> Self {
        self.esc_returns = yes;
        self
    }

    #[must_use]
    pub fn detail_owns(mut self, ids: &[Id]) -> Self {
        self.detail_owners = ids.to_vec();
        self
    }

    pub const fn esc_returns_enabled(&self) -> bool {
        self.esc_returns
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut MasterDetailState) -> Response<SplitAction> {
        let master_focused = cx
            .state(self.master)
            .contains(crate::response::StateFlags::FOCUSED);
        let detail_focused = self
            .detail_owners
            .iter()
            .copied()
            .chain([self.detail])
            .any(|id| cx.state(id).contains(crate::response::StateFlags::FOCUSED));
        state.reconcile_drawer(master_focused, detail_focused);
        for intent in cx.intents(self.master) {
            if matches!(intent, Intent::Binding(action) if is_open_detail(action)) {
                self.open_drawer(state, cx);
                return Response::changed().for_id(self.id);
            }
        }
        if detail_focused && self.esc_returns {
            for owner in self.detail_owners.iter().copied().chain([self.detail]) {
                for intent in cx.intents(owner) {
                    if matches!(intent, Intent::Binding(action) if is_close_detail(action)) {
                        self.close_drawer(state, cx);
                        return Response::changed().for_id(self.id);
                    }
                }
            }
        }
        let split = state
            .split
            .get_or_insert_with(|| self.initial_split_state());
        let pane = self.split_pane();
        let mut response = pane.update(cx, split);
        if self.split_sizing == SplitSizing::Resizable && self.resize_bindings.is_some() {
            let split_active = cx
                .layout(self.id)
                .and_then(|facts| facts.logical_area)
                .is_some_and(|area| area.width >= self.collapse_below);
            if split_active {
                let owner = if master_focused {
                    Some(self.master)
                } else {
                    self.detail_owners
                        .iter()
                        .copied()
                        .chain([self.detail])
                        .find(|id| cx.state(*id).contains(crate::response::StateFlags::FOCUSED))
                };
                if let Some(owner) = owner.filter(|owner| *owner != self.id) {
                    let child_response = pane.update_from_owner(cx, split, owner);
                    if child_response.is_consumed() || child_response.is_changed() {
                        response = child_response;
                    }
                }
            }
        }
        response
    }

    pub fn layout(&self, area: Rect, state: &MasterDetailState) -> MasterDetailLayout {
        let mut layout = self.resolve_responsive_layout(area, state.drawer_open);
        if layout.mode == MasterDetailMode::Split {
            let split = self.split_pane();
            let split_state = state.split.unwrap_or_else(|| self.initial_split_state());
            let (master, detail) = split.panes(split_state, area);
            layout.master = master;
            layout.detail = detail;
            layout.seam = split.seam(split_state, area);
        }
        layout
    }

    pub fn resolve_responsive_layout(&self, area: Rect, drawer_open: bool) -> MasterDetailLayout {
        if area.width >= self.collapse_below {
            return MasterDetailLayout {
                mode: MasterDetailMode::Split,
                ..MasterDetailLayout::default()
            };
        }

        if drawer_open {
            return MasterDetailLayout {
                mode: MasterDetailMode::Drawer,
                detail: area,
                ..MasterDetailLayout::default()
            };
        }

        let rows = self.summary_rows.min(area.height / 3);
        if rows == 0 {
            return MasterDetailLayout {
                mode: MasterDetailMode::Master,
                master: area,
                ..MasterDetailLayout::default()
            };
        }
        let master_height = area.height.saturating_sub(rows.saturating_add(1));
        let master = Rect::new(area.x, area.y, area.width, master_height);
        MasterDetailLayout {
            mode: MasterDetailMode::Master,
            master,
            summary: Rect::new(area.x, master.bottom().saturating_add(1), area.width, rows),
            ..MasterDetailLayout::default()
        }
    }

    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &MasterDetailState,
        body: impl FnOnce(&mut Ui<'_>, MasterDetailLayout) -> R,
    ) -> R {
        let layout = self.layout(area, state);
        if layout.mode == MasterDetailMode::Split {
            let split = self.split_pane();
            let split_state = state.split.unwrap_or_else(|| self.initial_split_state());
            return split.draw(ui, area, &split_state, |ui, _, _| {
                let result = body(ui, layout);
                self.publish_navigation_bindings(ui, true);
                result
            });
        }
        ui.report_layout(
            self.id,
            LayoutFacts::new(0, 0, area.height, area.width).with_logical_area(area),
        );
        ui.with_area(area, |ui| {
            let result = body(ui, layout);
            self.publish_navigation_bindings(ui, false);
            result
        })
    }

    fn publish_navigation_bindings(&self, ui: &mut Ui<'_>, split_active: bool) {
        let master = ui.state(self.master);
        ui.publish_additional_bindings(self.master, master, OPEN_DETAIL);
        if self.esc_returns {
            for owner in self.detail_owners.iter().copied().chain([self.detail]) {
                let live = ui.state(owner);
                ui.publish_additional_bindings(owner, live, CLOSE_DETAIL);
            }
        }
        if split_active
            && self.split_sizing == SplitSizing::Resizable
            && let Some(table) = self.resize_bindings
        {
            ui.publish_additional_bindings(self.master, master, table);
            for owner in self.detail_owners.iter().copied().chain([self.detail]) {
                if owner != self.master {
                    let live = ui.state(owner);
                    ui.publish_additional_bindings(owner, live, table);
                }
            }
        }
    }

    fn initial_split_state(&self) -> SplitPaneState {
        match self.initial_master_cells {
            Some(_) => SplitPaneState::new_with_initial_cells(self.master_share),
            None => SplitPaneState::new(self.master_share),
        }
    }

    pub fn split_pane(&self) -> SplitPane<'static> {
        let mut split = SplitPane::new(self.id, SplitAxis::Horizontal)
            .gap(self.gap)
            .min_first(self.min_master)
            .min_second(self.min_detail)
            .resizable(true)
            .resize_step_cells(self.resize_step_cells)
            .reset_policy(self.reset_policy)
            .seam_at_offset(1, self.gap / 2)
            .preserve_drag_offset(self.preserve_drag_offset);
        if let Some(cells) = self.initial_master_cells {
            split = split.initial_first_cells(cells);
        }
        if let Some(table) = self.resize_bindings {
            split = split.resize_bindings(table);
        }
        match self.split_sizing {
            SplitSizing::Resizable => split,
            SplitSizing::FixedDetail { percent, min, max } => {
                split.fixed_second_share(percent, min, max).fixed_sizing()
            }
            SplitSizing::FixedMaster { percent, min, max } => {
                split.fixed_first_share(percent, min, max).fixed_sizing()
            }
        }
    }
}
