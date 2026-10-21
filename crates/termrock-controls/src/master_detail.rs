use ratatui_core::layout::Rect;

use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
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

    fn reconcile_drawer(&mut self, master_focused: bool, detail_focused: bool) {
        if detail_focused {
            self.drawer_open = true;
        } else if master_focused {
            self.drawer_open = false;
            self.return_focus = None;
        }
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

    /// Route split resizing and master/detail focus transitions at every width.
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
            .get_or_insert_with(|| SplitPaneState::new(self.master_share));
        self.split_pane().update(cx, split)
    }

    /// Compute stable pane regions for a terminal area.
    pub fn layout(&self, area: Rect, state: &MasterDetailState) -> MasterDetailLayout {
        let mut layout = self.resolve_responsive_layout(area, state.drawer_open);
        if layout.mode == MasterDetailMode::Split {
            let split = self.split_pane();
            let split_state = state
                .split
                .unwrap_or_else(|| SplitPaneState::new(self.master_share));
            let (master, detail) = split.panes(split_state, area);
            layout.master = master;
            layout.detail = detail;
            layout.seam = split.seam(split_state, area);
        }
        layout
    }

    /// Resolve shared responsive mode and narrow summary geometry.
    ///
    /// Split pane geometry remains owned by the configured [`SplitPane`]. In
    /// split mode, the pane rectangles in the returned value are empty; use
    /// [`Self::layout`] when this control's split regions are needed.
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
            return split.draw(ui, area, &split_state, |ui, _, _| {
                let result = body(ui, layout);
                self.publish_navigation_bindings(ui);
                result
            });
        }
        ui.report_layout(
            self.id,
            LayoutFacts::new(0, 0, area.height, area.width).with_logical_area(area),
        );
        ui.with_area(area, |ui| {
            let result = body(ui, layout);
            self.publish_navigation_bindings(ui);
            result
        })
    }

    fn publish_navigation_bindings(&self, ui: &mut Ui<'_>) {
        let master = ui.state(self.master);
        ui.publish_additional_bindings(self.master, master, OPEN_DETAIL);
        if self.esc_returns {
            for owner in self.detail_owners.iter().copied().chain([self.detail]) {
                let live = ui.state(owner);
                ui.publish_additional_bindings(owner, live, CLOSE_DETAIL);
            }
        }
    }

    /// Build the split pane used by this block's layout, update, and draw paths.
    ///
    /// Exposed for component-author adapters that need to measure the same
    /// pane configuration without creating a second split policy.
    pub fn split_pane(&self) -> SplitPane<'static> {
        SplitPane::new(self.id, SplitAxis::Horizontal)
            .gap(self.gap)
            .min_first(self.min_master)
            .min_second(self.min_detail)
            .resizable(true)
            .seam_at_offset(1, self.gap / 2)
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
    use crate::{
        event::KeyCode,
        keymap::{Hint, HintKey, KeyMap},
        response::Response,
    };

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
        assert_eq!(l.seam, Rect::new(43, 3, 1, 40));
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

        let state = MasterDetailState {
            drawer_open: true,
            ..state
        };
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
        assert_eq!(
            l.seam,
            Rect::new(l.master.right() + 1, area.y, 1, area.height)
        );
        assert!(l.seam.right() <= area.right());
    }

    struct DrawerApp {
        md: MasterDetail,
        state: MasterDetailState,
        keymap: KeyMap,
    }

    const REAL_MASTER: Id = Id::root("master-detail.real.master");
    const REAL_DETAIL: Id = Id::root("master-detail.real.detail");
    fn real_md() -> MasterDetail {
        MD.detail_owns(&[REAL_DETAIL])
    }

    struct RealChildrenApp {
        state: MasterDetailState,
        area: Rect,
        keymap: KeyMap,
    }

    impl RealChildrenApp {
        fn master_button() -> crate::Button<'static> {
            crate::Button::new(REAL_MASTER, "Device catalog")
        }

        fn detail_button() -> crate::Button<'static> {
            crate::Button::new(REAL_DETAIL, "Device telemetry")
        }
    }

    impl App for RealChildrenApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let md = real_md();
            let response = md.update(cx, &mut self.state);
            match md.layout(self.area, &self.state).mode {
                MasterDetailMode::Split => {
                    let _ = Self::master_button().update(cx);
                    let _ = Self::detail_button().update(cx);
                }
                MasterDetailMode::Master => {
                    let _ = Self::master_button().update(cx);
                }
                MasterDetailMode::Drawer => {
                    let _ = Self::detail_button().update(cx);
                }
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            real_md().draw(ui, self.area, &self.state, |ui, layout| match layout.mode {
                MasterDetailMode::Split => {
                    let _ = Self::master_button().draw(ui, layout.master);
                    let _ = Self::detail_button().draw(ui, layout.detail);
                }
                MasterDetailMode::Master => {
                    let _ = Self::master_button().draw(ui, layout.master);
                }
                MasterDetailMode::Drawer => {
                    let _ = Self::detail_button().draw(ui, layout.detail);
                }
            });
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    impl Default for DrawerApp {
        fn default() -> Self {
            Self {
                md: MD,
                state: MasterDetailState::default(),
                keymap: KeyMap::new(),
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

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    struct ResizeApp {
        md: MasterDetail,
        state: MasterDetailState,
        area: core::cell::Cell<Rect>,
        mode: core::cell::Cell<MasterDetailMode>,
        detail_rect: core::cell::Cell<Rect>,
        modal_requested: bool,
    }

    impl Default for ResizeApp {
        fn default() -> Self {
            Self {
                md: MD.detail_owns(&[DETAIL_CHILD]),
                state: MasterDetailState::default(),
                area: core::cell::Cell::new(Rect::new(0, 0, 120, 40)),
                mode: core::cell::Cell::new(MasterDetailMode::Split),
                detail_rect: core::cell::Cell::new(Rect::ZERO),
                modal_requested: false,
            }
        }
    }

    const DETAIL_CHILD: Id = Id::root("master-detail.detail.child");
    const MODAL: Id = Id::root("master-detail.modal");
    const MODAL_FOCUS: Id = Id::root("master-detail.modal.focus");
    const MASTER_TAB_ACTION: ActionKey = ActionKey::custom("master_detail.test.master_tab");
    const MASTER_CHILD_BINDINGS: &[Binding<MasterDetailCmd>] = &[Binding {
        action: MASTER_TAB_ACTION,
        chord: Some(Chord::key(KeyCode::Tab)),
        cmd: MasterDetailCmd::OpenDetail,
        label: "Next master control",
        priority: 50,
        visible: true,
    }];
    const HIDDEN_MASTER_CHILD_BINDINGS: &[Binding<MasterDetailCmd>] = &[Binding {
        action: MASTER_TAB_ACTION,
        chord: Some(Chord::key(KeyCode::Tab)),
        cmd: MasterDetailCmd::OpenDetail,
        label: "Hidden child command",
        priority: 50,
        visible: false,
    }];

    struct WideNavigationApp {
        state: MasterDetailState,
        area: Rect,
        child_tab_seen: bool,
        child_tab_binding: bool,
        child_tab_hidden: bool,
        keymap: KeyMap,
        hints: std::cell::RefCell<Vec<Hint>>,
    }

    impl App for WideNavigationApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.child_tab_seen |= cx.intents(MD.master).any(
                |intent| matches!(intent, Intent::Binding(action) if action == MASTER_TAB_ACTION),
            );
            MD.update(cx, &mut self.state).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            MD.draw(ui, self.area, &self.state, |ui, layout| {
                ui.register_focus_only(MD.master, Focusability::Focusable);
                ui.register_focus_only(MD.detail, Focusability::Focusable);
                if layout.mode == MasterDetailMode::Split && self.child_tab_binding {
                    let table = if self.child_tab_hidden {
                        HIDDEN_MASTER_CHILD_BINDINGS
                    } else {
                        MASTER_CHILD_BINDINGS
                    };
                    ui.publish_bindings(MD.master, ui.state(MD.master), table);
                }
            });
            let _ = ui.with_focused_hints(|_, hints| {
                self.hints.replace(hints.hints.clone());
            });
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    impl App for ResizeApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if core::mem::take(&mut self.modal_requested) {
                cx.open_layer(
                    MODAL,
                    crate::runtime::LayerSpec::modal(MODAL).initial_focus(MODAL_FOCUS),
                );
            }
            self.md.update(cx, &mut self.state).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = self.area.get();
            self.md.draw(ui, area, &self.state, |ui, layout| {
                self.mode.set(layout.mode);
                self.detail_rect.set(layout.detail);
                match layout.mode {
                    MasterDetailMode::Split => {
                        ui.register_focus_only(MD.master, Focusability::Focusable);
                        ui.register_focus_only(MD.detail, Focusability::Focusable);
                        ui.register_focus_only(DETAIL_CHILD, Focusability::Focusable);
                    }
                    MasterDetailMode::Master => {
                        ui.register_focus_only(MD.master, Focusability::Focusable);
                    }
                    MasterDetailMode::Drawer => {
                        ui.register_focus_only(MD.detail, Focusability::Focusable);
                        ui.register_focus_only(DETAIL_CHILD, Focusability::Focusable);
                    }
                }
            });
            let _ = ui.layer(MODAL, |ui, _| {
                ui.register_focus_only(MODAL_FOCUS, Focusability::Focusable);
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

    #[test]
    fn split_navigation_tab_and_close_controls_work_at_wide_width() {
        let area = Rect::new(0, 0, 120, 40);
        let mut runtime = Runtime::new(
            WideNavigationApp {
                state: MasterDetailState::default(),
                area,
                child_tab_seen: false,
                child_tab_binding: false,
                child_tab_hidden: false,
                keymap: KeyMap::new(),
                hints: std::cell::RefCell::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(MD.master));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(MD.master));
        assert!(runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Show details" && hint.key == HintKey::Chord(Chord::key(KeyCode::Tab))
        }));

        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert_eq!(runtime.focus(), Some(MD.detail));
        assert!(runtime.app().state.drawer_open());
        assert!(!runtime.is_open(MODAL));

        for close in [KeyCode::BackTab, KeyCode::Left, KeyCode::Esc] {
            buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            for close_chord in [KeyCode::Esc, KeyCode::Left, KeyCode::BackTab] {
                assert!(runtime.app().hints.borrow().iter().any(|hint| {
                    hint.label == "Back" && hint.key == HintKey::Chord(Chord::key(close_chord))
                }));
            }
            let _ = deliver(&mut runtime, key(close));
            assert_eq!(runtime.focus(), Some(MD.master), "close key {close:?}");
            assert!(!runtime.app().state.drawer_open(), "close key {close:?}");
            runtime.set_focus(Some(MD.detail));
            buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
        }
    }

    #[test]
    fn child_binding_precedes_container_tab_binding_without_being_replaced() {
        let area = Rect::new(0, 0, 120, 40);
        let mut runtime = Runtime::new(
            WideNavigationApp {
                state: MasterDetailState::default(),
                area,
                child_tab_seen: false,
                child_tab_binding: true,
                child_tab_hidden: false,
                keymap: KeyMap::new(),
                hints: std::cell::RefCell::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(MD.master));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert!(runtime.app().child_tab_seen);
        assert_eq!(runtime.focus(), Some(MD.master));
        assert!(!runtime.app().state.drawer_open());
        assert!(runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Next master control"
                && hint.key == HintKey::Chord(Chord::key(KeyCode::Tab))
        }));
        assert!(!runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Show details" && hint.key == HintKey::Chord(Chord::key(KeyCode::Tab))
        }));
    }

    #[test]
    fn hidden_child_tab_binding_shadows_master_detail_hint() {
        let area = Rect::new(0, 0, 120, 40);
        let mut runtime = Runtime::new(
            WideNavigationApp {
                state: MasterDetailState::default(),
                area,
                child_tab_seen: false,
                child_tab_binding: true,
                child_tab_hidden: true,
                keymap: KeyMap::new(),
                hints: std::cell::RefCell::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(MD.master));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert!(!runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Show details" && hint.key == HintKey::Chord(Chord::key(KeyCode::Tab))
        }));
        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert!(runtime.app().child_tab_seen);
        assert_eq!(runtime.focus(), Some(MD.master));
        assert!(!runtime.app().state.drawer_open());
    }

    #[test]
    fn wide_close_action_remains_remappable() {
        let area = Rect::new(0, 0, 120, 40);
        let mut keymap = KeyMap::new();
        keymap.remap_component(
            MD.detail,
            ActionKey::custom("master_detail.close_detail"),
            Chord::key(KeyCode::Backspace),
        );
        let mut runtime = Runtime::new(
            WideNavigationApp {
                state: MasterDetailState::default(),
                area,
                child_tab_seen: false,
                child_tab_binding: false,
                child_tab_hidden: false,
                keymap,
                hints: std::cell::RefCell::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(MD.master));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert_eq!(runtime.focus(), Some(MD.detail));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert!(runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Back" && hint.key == HintKey::Chord(Chord::key(KeyCode::Backspace))
        }));
        assert!(!runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Back" && hint.key == HintKey::Chord(Chord::key(KeyCode::Esc))
        }));

        runtime
            .app_mut()
            .keymap
            .remove_component(MD.detail, ActionKey::custom("master_detail.close_detail"));
        let _ = deliver(&mut runtime, key(KeyCode::Backspace));
        buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert!(!runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Back" && hint.key == HintKey::Chord(Chord::key(KeyCode::Esc))
        }));
        assert!(!runtime.app().hints.borrow().iter().any(|hint| {
            hint.label == "Back" && hint.key == HintKey::Chord(Chord::key(KeyCode::Backspace))
        }));

        let _ = deliver(&mut runtime, key(KeyCode::Esc));
        assert_eq!(runtime.focus(), Some(MD.detail));
        let _ = deliver(&mut runtime, key(KeyCode::Backspace));
        assert_eq!(runtime.focus(), Some(MD.detail));
    }

    #[test]
    fn drawer_survives_modal_focus_excursion() {
        let mut runtime = Runtime::new(ResizeApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();
        runtime.set_focus(Some(DETAIL_CHILD));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();
        let _ = deliver(&mut runtime, crate::event::Input::Resize(80, 24));
        runtime.app().area.set(Rect::new(0, 0, 80, 24));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 80, 24));
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Drawer);
        let drawer_detail = runtime.app().detail_rect.get();

        runtime.app_mut().modal_requested = true;
        let _ = deliver(&mut runtime, key(KeyCode::Char('x')));
        assert!(runtime.is_open(MODAL));
        assert_eq!(runtime.focus(), Some(MODAL_FOCUS));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 80, 24));
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Drawer);
        assert_eq!(runtime.app().detail_rect.get(), drawer_detail);
        assert!(runtime.app().state.drawer_open());
    }

    #[test]
    fn focused_detail_child_selects_drawer_after_narrow_resize() {
        let mut runtime = Runtime::new(ResizeApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Split);

        runtime.set_focus(Some(DETAIL_CHILD));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();

        let _ = deliver(&mut runtime, crate::event::Input::Resize(80, 24));
        runtime.app().area.set(Rect::new(0, 0, 80, 24));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 80, 24));
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.focus(), Some(DETAIL_CHILD));
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Drawer);
        assert!(runtime.app().state.drawer_open());

        let _ = deliver(&mut runtime, crate::event::Input::Resize(120, 40));
        runtime.app().area.set(Rect::new(0, 0, 120, 40));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Split);

        runtime.set_focus(Some(MD.master));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 120, 40));
        runtime
            .draw_buffer(Rect::new(0, 0, 120, 40), &mut buffer)
            .commit_presented();
        assert!(runtime.app().state.drawer_open());
        let _ = deliver(&mut runtime, key(KeyCode::Char('q')));
        assert!(!runtime.app().state.drawer_open());

        let _ = deliver(&mut runtime, crate::event::Input::Resize(80, 24));
        runtime.app().area.set(Rect::new(0, 0, 80, 24));
        buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 80, 24));
        runtime
            .draw_buffer(Rect::new(0, 0, 80, 24), &mut buffer)
            .commit_presented();
        assert_eq!(runtime.focus(), Some(MD.master));
        assert_eq!(runtime.app().mode.get(), MasterDetailMode::Master);
        assert!(!runtime.app().state.drawer_open());
    }

    #[test]
    fn drawing_real_children_does_not_reconcile_or_mutate_caller_state() {
        let area = Rect::new(0, 0, 120, 30);
        let mut runtime = Runtime::new(
            RealChildrenApp {
                state: MasterDetailState::default(),
                area,
                keymap: KeyMap::new(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(REAL_DETAIL));

        let before = runtime.app().state;
        let md = real_md();
        let expected = md.layout(area, &before);
        assert_eq!(expected.mode, MasterDetailMode::Split);
        for _ in 0..2 {
            buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            assert_eq!(runtime.app().state, before);
            assert_eq!(md.layout(area, &runtime.app().state), expected);
            assert_eq!(runtime.focus(), Some(REAL_DETAIL));
        }

        let _ = deliver(&mut runtime, key(KeyCode::Char('q')));
        assert!(runtime.app().state.drawer_open());
        let narrow = Rect::new(0, 0, 80, 24);
        runtime.app_mut().area = narrow;
        buffer = ratatui_core::buffer::Buffer::empty(narrow);
        runtime.draw_buffer(narrow, &mut buffer).commit_presented();
        let resolved = md.layout(narrow, &runtime.app().state);
        assert_eq!(resolved.mode, MasterDetailMode::Drawer);
        assert_eq!(resolved.detail, narrow);
    }

    #[test]
    fn drawer_close_action_uses_the_component_keymap_remap() {
        let mut app = DrawerApp::default();
        let close = ActionKey::custom("master_detail.close_detail");
        let mut keymap = KeyMap::new();
        keymap.remap_component(MD.detail, close, Chord::key(KeyCode::Backspace));
        app.keymap = keymap;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = deliver(&mut runtime, key(KeyCode::Tab));
        assert!(runtime.app().state.drawer_open());

        let _ = deliver(&mut runtime, key(KeyCode::Esc));
        assert!(runtime.app().state.drawer_open());
        let _ = deliver(&mut runtime, key(KeyCode::Backspace));
        assert!(!runtime.app().state.drawer_open());
        assert_eq!(runtime.focus(), Some(MD.master));
    }
}
