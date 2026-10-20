use ratatui_core::layout::Rect;

use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::id::Id;
use crate::intent::Intent;
use crate::keymap::Binding;
use crate::layout::SplitAxis;
use crate::response::Response;
use crate::split::{SplitAction, SplitPane, SplitPaneState};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MasterDetailCmd {
    OpenDetail,
}

const OPEN_DETAIL: &[Binding<MasterDetailCmd>] = &[Binding {
    action: ActionKey::custom("master_detail.open_detail"),
    chord: Some(Chord::key(KeyCode::Tab)),
    cmd: MasterDetailCmd::OpenDetail,
    label: "Show details",
    priority: 40,
    visible: true,
}];

/// Responsive presentation chosen for a master/detail container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MasterDetailMode {
    /// Master and detail appear side by side.
    #[default]
    Split,
    /// Detail is hidden while the master and its summary remain visible.
    Master,
    /// Detail fills the available area on a narrow terminal.
    Drawer,
}

/// Regions assigned to the master, detail, summary, and resize seam.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MasterDetailLayout {
    pub mode: MasterDetailMode,
    pub master: Rect,
    pub detail: Rect,
    pub summary: Rect,
    pub seam: Rect,
}

/// Persistent split and narrow-drawer state for [`MasterDetail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MasterDetailState {
    split: Option<SplitPaneState>,
    drawer_open: bool,
    return_focus: Option<Id>,
}

impl MasterDetailState {
    pub const fn split(&self) -> Option<SplitPaneState> {
        self.split
    }

    pub const fn drawer_open(&self) -> bool {
        self.drawer_open
    }

    /// Open the narrow detail view and remember the control to restore on close.
    fn open_drawer(&mut self, cx: &mut Cx<'_>, detail: Id, return_focus: Id) {
        self.drawer_open = true;
        self.return_focus = Some(return_focus);
        cx.focus(detail);
    }

    /// Close the detail view and return focus to its remembered owner.
    fn close_drawer(&mut self, cx: &mut Cx<'_>, master: Id) {
        self.drawer_open = false;
        cx.focus(self.return_focus.take().unwrap_or(master));
    }
}

/// Responsive master/detail layout that delegates split geometry and seam input
/// to [`SplitPane`].
#[derive(Clone, Debug)]
pub struct MasterDetail {
    id: Id,
    master: Id,
    detail: Id,
    master_share: u8,
    min_master: u16,
    min_detail: u16,
    gap: u16,
    collapse_below: u16,
    summary_rows: u16,
    esc_returns: bool,
    detail_owners: Vec<Id>,
}

impl MasterDetail {
    pub const fn new(id: Id, master: Id, detail: Id) -> Self {
        Self {
            id,
            master,
            detail,
            master_share: 32,
            min_master: 28,
            min_detail: 40,
            gap: 2,
            collapse_below: 100,
            summary_rows: 6,
            esc_returns: true,
            detail_owners: Vec::new(),
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

    /// Open the narrow detail view, returning focus to the master on close.
    pub fn open_drawer(&self, state: &mut MasterDetailState, cx: &mut Cx<'_>) {
        state.open_drawer(cx, self.detail, self.master);
    }

    /// Open the narrow detail view, remembering the current child as focus owner.
    pub fn open_drawer_from(
        &self,
        state: &mut MasterDetailState,
        cx: &mut Cx<'_>,
        return_focus: Id,
    ) {
        state.open_drawer(cx, self.detail, return_focus);
    }

    /// Close the narrow detail view and restore its remembered focus owner.
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

    /// Route split resizing and the narrow drawer's Escape focus return.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut MasterDetailState) -> Response<SplitAction> {
        let narrow = cx
            .layout(self.id)
            .and_then(|facts| facts.logical_area)
            .is_some_and(|area| area.width < self.collapse_below);
        if narrow {
            for intent in cx.intents(self.master) {
                if matches!(intent, Intent::Binding(action) if Binding::command(OPEN_DETAIL, action) == Some(MasterDetailCmd::OpenDetail))
                {
                    self.open_drawer(state, cx);
                    return Response::changed().for_id(self.id);
                }
            }
        }
        if state.drawer_open && self.esc_returns {
            for owner in self.detail_owners.iter().copied().chain([self.detail]) {
                for intent in cx.intents(owner) {
                    if matches!(intent, Intent::Key(key) if key.code == KeyCode::Esc && key.mods == KeyModifiers::empty())
                    {
                        self.close_drawer(state, cx);
                        return Response::changed().for_id(self.id);
                    }
                }
            }
        }
        let split = state
            .split
            .get_or_insert_with(|| SplitPaneState::new(self.master_share));
        self.split_pane().update(cx, split)
    }

    /// Compute stable pane regions for a terminal area.
    pub fn layout(&self, area: Rect, state: &MasterDetailState) -> MasterDetailLayout {
        if area.width >= self.collapse_below {
            let split = self.split_pane();
            let split_state = state
                .split
                .unwrap_or_else(|| SplitPaneState::new(self.master_share));
            let (master, detail) = split.panes(split_state, area);
            return MasterDetailLayout {
                mode: MasterDetailMode::Split,
                master,
                detail,
                summary: Rect::ZERO,
                seam: split.seam(split_state, area),
            };
        }

        if state.drawer_open {
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

    /// Draw the resize seam in split mode, then expose the three child regions.
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
            let split_state = state
                .split
                .unwrap_or_else(|| SplitPaneState::new(self.master_share));
            return split.draw(ui, area, &split_state, |ui, _, _| body(ui, layout));
        }
        ui.report_layout(
            self.id,
            LayoutFacts::new(0, 0, area.height, area.width).with_logical_area(area),
        );
        ui.with_area(area, |ui| {
            let result = body(ui, layout);
            if layout.mode == MasterDetailMode::Master {
                let live = ui.state(self.master);
                ui.publish_bindings(self.master, live, OPEN_DETAIL);
            }
            result
        })
    }

    fn split_pane(&self) -> SplitPane<'static> {
        SplitPane::new(self.id, SplitAxis::Horizontal)
            .gap(self.gap)
            .min_first(self.min_master)
            .min_second(self.min_detail)
            .resizable(true)
            .seam_center(1)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;

    use super::*;
    use crate::focus::Focusability;
    use crate::runtime::stub::{SCREEN, deliver, key};
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use crate::ui::Ui;
    use crate::{event::KeyCode, response::Response};

    const MD: MasterDetail = MasterDetail::new(
        Id::root("master-detail.test"),
        Id::root("master-detail.master"),
        Id::root("master-detail.detail"),
    );

    #[test]
    fn default_split_matches_master_share_gap_and_minima() {
        let l = MD.layout(Rect::new(5, 3, 120, 40), &MasterDetailState::default());
        assert_eq!(l.mode, MasterDetailMode::Split);
        assert_eq!(l.master, Rect::new(5, 3, 37, 40));
        assert_eq!(l.detail, Rect::new(44, 3, 81, 40));
        assert_eq!(l.seam, Rect::new(42, 3, 1, 40));
        assert_eq!(l.summary, Rect::ZERO);
    }

    #[test]
    fn collapse_threshold_switches_to_master_summary_or_detail_drawer() {
        let state = MasterDetailState::default();
        let at_threshold = MD.layout(Rect::new(0, 0, 100, 40), &state);
        assert_eq!(at_threshold.mode, MasterDetailMode::Split);
        let below = MD.layout(Rect::new(0, 0, 99, 40), &state);
        assert_eq!(below.mode, MasterDetailMode::Master);
        assert_eq!(below.master, Rect::new(0, 0, 99, 33));
        assert_eq!(below.summary, Rect::new(0, 34, 99, 6));

        let mut state = state;
        state.drawer_open = true;
        let drawer = MD.layout(Rect::new(4, 2, 80, 20), &state);
        assert_eq!(drawer.mode, MasterDetailMode::Drawer);
        assert_eq!(drawer.detail, Rect::new(4, 2, 80, 20));
        assert_eq!(drawer.master, Rect::ZERO);
        assert_eq!(drawer.summary, Rect::ZERO);
    }

    #[test]
    fn narrow_summary_respects_tiny_heights_and_area_origin() {
        let tiny = MD.layout(Rect::new(7, 9, 60, 2), &MasterDetailState::default());
        assert_eq!(tiny.mode, MasterDetailMode::Master);
        assert_eq!(tiny.master, Rect::new(7, 9, 60, 2));
        assert!(tiny.summary.is_empty());
    }

    #[test]
    fn split_minima_and_overrides_are_applied_without_escaping_area() {
        let area = Rect::new(9, 2, 100, 12);
        let l = MD
            .min_master(45)
            .min_detail(50)
            .gap(5)
            .master_share(70)
            .collapse_below(80)
            .layout(area, &MasterDetailState::default());
        assert_eq!(l.mode, MasterDetailMode::Split);
        assert_eq!(l.master.width, 45);
        assert_eq!(l.detail.width, 50);
        assert_eq!(l.detail.x, l.master.right() + 5);
        assert!(l.master.right() <= area.right());
        assert!(l.detail.right() <= area.right());

        let share = MD
            .master_share(70)
            .min_detail(20)
            .layout(Rect::new(0, 0, 120, 30), &MasterDetailState::default());
        assert_eq!(share.master.width, 82);
    }

    #[test]
    fn split_percentage_tracks_state_and_seam_stays_inside_the_area() {
        let state = MasterDetailState {
            split: Some(SplitPaneState::new(60)),
            ..MasterDetailState::default()
        };
        let area = Rect::new(4, 6, 120, 30);
        let l = MD.layout(area, &state);
        assert_eq!(l.master.width, 70);
        assert_eq!(l.detail.x, l.master.right() + 2);
        assert_eq!(l.seam, Rect::new(l.master.right(), area.y, 1, area.height));
        assert!(l.seam.right() <= area.right());
    }

    struct DrawerApp {
        md: MasterDetail,
        state: MasterDetailState,
    }

    impl Default for DrawerApp {
        fn default() -> Self {
            Self {
                md: MD,
                state: MasterDetailState::default(),
            }
        }
    }

    impl App for DrawerApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.md.update(cx, &mut self.state).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.md
                .draw(ui, SCREEN, &self.state, |ui, layout| match layout.mode {
                    MasterDetailMode::Split => {
                        ui.register_focus_only(MD.master, Focusability::Focusable);
                        ui.register_focus_only(MD.detail, Focusability::Focusable);
                    }
                    MasterDetailMode::Master => {
                        ui.register_focus_only(MD.master, Focusability::Focusable);
                    }
                    MasterDetailMode::Drawer => {
                        ui.register_focus_only(MD.detail, Focusability::Focusable);
                    }
                });
        }
    }

    #[test]
    fn narrow_tab_opens_detail_and_escape_restores_master_focus() {
        let mut runtime = Runtime::new(DrawerApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(MD.master));

        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert!(runtime.app().state.drawer_open());
        assert_eq!(runtime.focus(), Some(MD.detail));

        let _ = deliver(&mut runtime, key(KeyCode::Esc));
        assert!(!runtime.app().state.drawer_open());
        assert_eq!(runtime.focus(), Some(MD.master));
    }
}
