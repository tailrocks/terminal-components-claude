use ratatui_core::layout::Rect;
use termrock_controls::{
    Binding, MasterDetail, MasterDetailLayout, MasterDetailMode, MasterDetailState, SplitCmd,
    SplitPaneState,
};
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, MouseKind};
use termrock_core::id::{Id, Part, PartRef};
use termrock_core::keys::{KeyCode, KeyModifiers};
use termrock_layout::Constraints;
use termrock_runtime::{App, BindingState, Bindings, Cx, Focusability, KeyMap, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const CONTAINER: Id = Id::root("master-detail.policy.container");
const MASTER: Id = Id::root("master-detail.policy.master");
const DETAIL: Id = Id::root("master-detail.policy.detail");
const DETAIL_CHILD: Id = Id::root("master-detail.policy.detail-child");
const OTHER: Id = Id::root("master-detail.policy.other");
const AREA: Rect = Rect::new(3, 2, 120, 30);

const GROW: ActionKey = ActionKey::custom("master_detail.policy.grow");
const SHRINK: ActionKey = ActionKey::custom("master_detail.policy.shrink");
const RESET: ActionKey = ActionKey::custom("master_detail.policy.reset");

const RESIZE_BINDINGS: &[Binding<SplitCmd>] = &[
    Binding {
        action: SHRINK,
        chord: Some(Chord::with(KeyCode::Char('h'), KeyModifiers::CONTROL)),
        cmd: SplitCmd::Shrink,
        label: "Narrower",
        priority: 40,
        visible: true,
    },
    Binding {
        action: GROW,
        chord: Some(Chord::with(KeyCode::Char('l'), KeyModifiers::CONTROL)),
        cmd: SplitCmd::Grow,
        label: "Wider",
        priority: 40,
        visible: true,
    },
    Binding {
        action: RESET,
        chord: Some(Chord::with(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        cmd: SplitCmd::Reset,
        label: "Reset split",
        priority: 40,
        visible: true,
    },
];

const NO_RESIZE_BINDINGS: &[Binding<SplitCmd>] = &[];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sizing {
    Resizable,
    FixedDetail,
    FixedMaster,
}

struct PolicyPage {
    state: MasterDetailState,
    area: Rect,
    master_share: u8,
    initial_master_cells: Option<u16>,
    reset_policy: termrock_controls::ResetPolicy,
    sizing: Sizing,
    resize_bindings: Option<&'static [Binding<SplitCmd>]>,
    preserve_drag_offset: bool,
    step: u16,
    keymap: KeyMap,
}

impl Default for PolicyPage {
    fn default() -> Self {
        Self {
            state: MasterDetailState::default(),
            area: AREA,
            master_share: 50,
            initial_master_cells: None,
            reset_policy: termrock_controls::ResetPolicy::Balanced,
            sizing: Sizing::Resizable,
            resize_bindings: None,
            preserve_drag_offset: false,
            step: 1,
            keymap: KeyMap::new(),
        }
    }
}

impl PolicyPage {
    fn control(&self) -> MasterDetail {
        let control = MasterDetail::new(CONTAINER, MASTER, DETAIL)
            .master_share(self.master_share)
            .reset_policy(self.reset_policy)
            .min_master(26)
            .min_detail(34)
            .gap(2)
            .collapse_below(100)
            .detail_owns(&[DETAIL_CHILD])
            .preserve_drag_offset(self.preserve_drag_offset)
            .resize_step_cells(self.step);
        let control = match self.initial_master_cells {
            Some(cells) => control.initial_master_cells(cells),
            None => control,
        };
        let control = match self.resize_bindings {
            Some(bindings) => control.resize_bindings(bindings),
            None => control,
        };
        match self.sizing {
            Sizing::Resizable => control.resizable(),
            Sizing::FixedDetail => control.fixed_detail_share(40, 34, 56),
            Sizing::FixedMaster => control.fixed_master_share(33, 26, 40),
        }
    }
}

impl App for PolicyPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.control().update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.control()
            .draw(ui, self.area, &self.state, |ui, layout| match layout.mode {
                MasterDetailMode::Split => {
                    ui.register_focus_only(MASTER, Focusability::Focusable);
                    ui.register_focus_only(DETAIL, Focusability::Focusable);
                    ui.register_focus_only(DETAIL_CHILD, Focusability::Focusable);
                    ui.register_focus_only(OTHER, Focusability::Focusable);
                }
                MasterDetailMode::Master => {
                    ui.register_focus_only(MASTER, Focusability::Focusable);
                }
                MasterDetailMode::Drawer => {
                    ui.register_focus_only(DETAIL, Focusability::Focusable);
                    ui.register_focus_only(DETAIL_CHILD, Focusability::Focusable);
                }
            });
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: PolicyPage) -> Harness<PolicyPage> {
    harness_sized(page, AREA.width, AREA.height)
}

fn harness_sized(page: PolicyPage, width: u16, height: u16) -> Harness<PolicyPage> {
    Harness::new(page, Theme::junie(), width, height)
}

fn layout(h: &Harness<PolicyPage>) -> MasterDetailLayout {
    h.app().control().layout(h.app().area, &h.app().state)
}

#[test]
fn legacy_default_and_master_share_keep_their_initial_resizable_geometry() {
    let default = MasterDetail::new(CONTAINER, MASTER, DETAIL);
    let typed: termrock_controls::SplitPane<'static> = default.split_pane();
    assert_eq!(
        default
            .layout(AREA, &MasterDetailState::default())
            .master
            .width,
        37
    );
    assert_eq!(typed.id(), CONTAINER);

    let configured = default.clone().min_detail(20).master_share(70);
    assert_eq!(
        configured
            .layout(AREA, &MasterDetailState::default())
            .master
            .width,
        82
    );
    assert!(
        configured
            .split_pane()
            .bindings(BindingState::default())
            .iter()
            .any(|binding| binding.cmd == SplitCmd::Grow)
    );

    let mode = default
        .master_share(70)
        .fixed_master_share(33, 26, 40)
        .master_share(60);
    assert_eq!(
        mode.layout(AREA, &MasterDetailState::default())
            .master
            .width,
        40
    );
    assert_eq!(
        mode.resizable()
            .layout(AREA, &MasterDetailState::default())
            .master
            .width,
        70,
        "explicit resizable mode restores the stored initial share"
    );
}

#[test]
fn initial_master_cells_stays_exact_across_supported_widths_and_respects_limits() {
    let control = MasterDetail::new(CONTAINER, MASTER, DETAIL)
        .initial_master_cells(39)
        .min_master(28)
        .min_detail(40)
        .gap(2)
        .collapse_below(100);
    let state = MasterDetailState::default();
    for width in [100, 102, 120, 160] {
        let area = Rect::new(5, 3, width, 24);
        let split = control.layout(area, &state);
        assert_eq!(split.mode, MasterDetailMode::Split);
        assert_eq!(split.master.width, 39, "width={width}");
        assert_eq!(split.detail.x, split.master.right() + 2);
        assert_eq!(split.master.width + 2 + split.detail.width, width);
    }

    let limited = control.clone().min_detail(80);
    assert_eq!(
        limited
            .layout(Rect::new(0, 0, 120, 24), &state)
            .master
            .width,
        38,
        "the detail minimum limits the requested initial width"
    );

    let too_narrow = control
        .clone()
        .collapse_below(0)
        .layout(Rect::new(0, 0, 69, 24), &state);
    assert_eq!(too_narrow.master.width, 69);
    assert!(too_narrow.detail.is_empty());

    assert_eq!(
        control
            .clone()
            .master_share(70)
            .layout(Rect::new(0, 0, 120, 24), &state)
            .master
            .width,
        78,
        "the retained detail minimum clamps the later percentage layout"
    );
    assert_eq!(
        MasterDetail::new(CONTAINER, MASTER, DETAIL)
            .layout(AREA, &state)
            .master
            .width,
        37,
        "the legacy default is unchanged"
    );

    assert_eq!(
        MasterDetail::new(CONTAINER, MASTER, DETAIL)
            .fixed_master_share(33, 26, 40)
            .initial_master_cells(39)
            .layout(Rect::new(0, 0, 120, 24), &state)
            .master
            .width,
        39,
        "the cell policy selects resizable sizing when called after a fixed mode"
    );
    let fixed_after_cells = MasterDetail::new(CONTAINER, MASTER, DETAIL)
        .initial_master_cells(39)
        .fixed_master_share(33, 26, 40);
    assert_eq!(
        fixed_after_cells
            .layout(Rect::new(0, 0, 120, 24), &state)
            .master
            .width,
        40,
        "a later fixed-share builder still selects fixed sizing"
    );
    assert_eq!(
        fixed_after_cells
            .resizable()
            .layout(Rect::new(0, 0, 120, 24), &state)
            .master
            .width,
        39,
        "explicit resizable mode restores the configured cell policy"
    );
}

#[test]
fn initial_cell_policy_supports_keyboard_resize_pointer_drag_resize_and_reset() {
    let mut h = harness_sized(
        PolicyPage {
            area: Rect::new(0, 0, 120, 30),
            initial_master_cells: Some(39),
            reset_policy: termrock_controls::ResetPolicy::Initial,
            resize_bindings: Some(RESIZE_BINDINGS),
            preserve_drag_offset: true,
            ..PolicyPage::default()
        },
        120,
        30,
    );
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    assert_eq!(layout(&h).master.width, 39);

    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(
        layout(&h).master.width,
        40,
        "the first key resize starts at 39"
    );
    let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert_eq!(
        layout(&h).master.width,
        39,
        "Initial reset restores the cell width"
    );

    let seam = h
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("the configured split keeps its real seam");
    let _ = h.mouse(MouseKind::Down, seam.x, seam.y + 5);
    let _ = h.mouse(MouseKind::Drag, seam.x + 5, seam.y + 5);
    assert_eq!(layout(&h).master.width, 44, "the first drag starts at 39");
    let _ = h.mouse(MouseKind::Up, seam.x + 5, seam.y + 5);
    assert_eq!(h.runtime().capture_owner(), None);

    let adjusted = h.app().state.split().expect("drag initializes split state");
    let adjusted_basis_points = adjusted.basis_points();
    h.app_mut().area = Rect::new(0, 0, 140, 30);
    let _ = h.resize(140, 30);
    h.draw();
    assert_eq!(
        h.app().state.split().unwrap().basis_points(),
        adjusted_basis_points,
        "a manual ratio survives terminal resize"
    );
    assert_eq!(
        layout(&h).master.width,
        (138u32 * u32::from(adjusted_basis_points) / 10_000) as u16
    );

    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert_eq!(
        layout(&h).master.width,
        39,
        "Initial reset returns to cells"
    );
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn configured_child_table_is_published_remappable_and_uses_cell_steps() {
    let mut page = PolicyPage {
        resize_bindings: Some(RESIZE_BINDINGS),
        step: 5,
        ..PolicyPage::default()
    };
    page.keymap.remap_component(
        MASTER,
        GROW,
        Chord::with(KeyCode::Char('m'), KeyModifiers::ALT),
    );
    page.keymap.remap_component(
        DETAIL_CHILD,
        GROW,
        Chord::with(KeyCode::Char('d'), KeyModifiers::ALT),
    );
    page.keymap.remap_component(
        OTHER,
        GROW,
        Chord::with(KeyCode::Char('o'), KeyModifiers::ALT),
    );
    let mut h = harness(page);
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();

    let before = layout(&h).master.width;
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(
        layout(&h).master.width,
        before,
        "the remapped chord replaces its default"
    );
    let _ = h.key_mod(KeyCode::Char('m'), KeyModifiers::ALT);
    assert_eq!(layout(&h).master.width, before + 5);

    h.runtime_mut().set_focus(Some(DETAIL_CHILD));
    h.draw();
    let before = layout(&h).master.width;
    let _ = h.key_mod(KeyCode::Char('d'), KeyModifiers::ALT);
    assert_eq!(layout(&h).master.width, before + 5);

    h.runtime_mut().set_focus(Some(OTHER));
    h.draw();
    let before = layout(&h).master.width;
    let _ = h.key_mod(KeyCode::Char('o'), KeyModifiers::ALT);
    assert_eq!(
        layout(&h).master.width,
        before,
        "unowned controls do not resize"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn configured_two_cell_step_moves_the_actual_pane_by_two_cells() {
    let mut h = harness(PolicyPage {
        resize_bindings: Some(RESIZE_BINDINGS),
        step: 2,
        ..PolicyPage::default()
    });
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    let before = layout(&h).master.width;
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(layout(&h).master.width, before + 2);
}

#[test]
fn aligned_seam_drag_keeps_raw_default_and_can_opt_into_the_grab_offset() {
    let area = Rect::new(0, 6, 120, 34);
    let mut raw = harness_sized(
        PolicyPage {
            area,
            master_share: 40,
            resize_bindings: Some(RESIZE_BINDINGS),
            step: 2,
            ..PolicyPage::default()
        },
        120,
        40,
    );
    raw.runtime_mut().set_focus(Some(MASTER));
    raw.draw();
    let raw_initial = raw
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("the aligned split is published before nudging");
    let _ = raw.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let raw_start = raw
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("default MasterDetail has an aligned seam");
    assert_eq!(raw_start.x, raw_initial.x + 2);
    let _ = raw.mouse(MouseKind::Down, raw_start.x, raw_start.y + 5);
    let _ = raw.mouse(MouseKind::Drag, raw_start.x + 8, raw_start.y + 5);
    let raw_after = raw
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("raw drag keeps the live seam");
    assert_eq!(raw_after.x, raw_start.x + 9);
    let _ = raw.mouse(MouseKind::Up, raw_start.x + 8, raw_start.y + 5);
    assert_eq!(raw.runtime().capture_owner(), None);

    let mut exact = harness_sized(
        PolicyPage {
            area,
            master_share: 40,
            preserve_drag_offset: true,
            resize_bindings: Some(RESIZE_BINDINGS),
            step: 2,
            ..PolicyPage::default()
        },
        120,
        40,
    );
    exact.runtime_mut().set_focus(Some(MASTER));
    exact.draw();
    let exact_initial = exact
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("the opt-in split is published before nudging");
    let _ = exact.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let exact_start = exact
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("opt-in MasterDetail has the same aligned seam");
    assert_eq!(exact_start.x, exact_initial.x + 2);
    let _ = exact.mouse(MouseKind::Down, exact_start.x, exact_start.y + 5);
    let focused_seam = exact.focus();
    assert_eq!(focused_seam, Some(CONTAINER));
    let _ = exact.mouse(MouseKind::Drag, exact_start.x, exact_start.y + 5);
    assert_eq!(
        exact
            .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
            .expect("zero movement leaves the seam present"),
        exact_start,
        "pressing on the painted seam and holding still does not jump"
    );
    let _ = exact.mouse(MouseKind::Drag, exact_start.x + 8, exact_start.y + 5);
    let moved = exact
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("opt-in drag keeps the live seam");
    assert_eq!(moved.x, exact_start.x + 8);
    let _ = exact.mouse(MouseKind::Drag, exact_start.x, exact_start.y + 5);
    assert_eq!(
        exact
            .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
            .expect("reverse drag keeps the live seam"),
        exact_start,
        "returning to the original pointer coordinate restores the original split"
    );
    let _ = exact.mouse(MouseKind::Up, exact_start.x, exact_start.y + 5);
    assert_eq!(exact.runtime().capture_owner(), None);
    assert_eq!(exact.focus(), focused_seam);
    assert!(raw.diagnostics().is_empty(), "{:?}", raw.diagnostics());
    assert!(exact.diagnostics().is_empty(), "{:?}", exact.diagnostics());
}

#[test]
fn legacy_container_focused_resize_remains_one_exact_cell() {
    let mut h = harness(PolicyPage::default());
    h.runtime_mut().set_focus(Some(CONTAINER));
    h.draw();
    let before = layout(&h).master.width;
    let _ = h.key(KeyCode::Right);
    assert_eq!(layout(&h).master.width, before + 1);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn empty_resize_table_removes_keyboard_commands_but_keeps_pointer_drag() {
    let page = PolicyPage {
        resize_bindings: Some(NO_RESIZE_BINDINGS),
        ..PolicyPage::default()
    };
    assert!(
        page.control()
            .split_pane()
            .bindings(BindingState::default())
            .is_empty()
    );

    let mut h = Harness::new(page, Theme::junie(), AREA.width, AREA.height);
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    let before = layout(&h).master.width;
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Home);
    assert_eq!(layout(&h).master.width, before);

    let seam = h
        .area_of_part(CONTAINER, PartRef::of(Part::SEAM))
        .expect("a resizable split retains its pointer seam");
    let _ = h.drag((seam.x, seam.y + 3), (seam.x + 5, seam.y + 3));
    assert_eq!(layout(&h).master.width, before + 6);
    assert_eq!(h.runtime().capture_owner(), None);
}

#[test]
fn fixed_modes_round_clamp_and_keep_layout_within_offset_and_tiny_areas() {
    let base = MasterDetail::new(CONTAINER, MASTER, DETAIL)
        .min_master(26)
        .min_detail(34)
        .gap(2)
        .collapse_below(0);
    let state = MasterDetailState::default();
    let fixed_master = base.clone().fixed_master_share(33, 26, 40);
    let expected = [(80, 26), (100, 33), (102, 34), (120, 40), (160, 40)];
    for (width, master_width) in expected {
        let area = Rect::new(7, 4, width, 18);
        let pane = fixed_master.layout(area, &state);
        assert_eq!(pane.master.width, master_width, "width={width}");
        assert_eq!(pane.master.x, area.x);
        assert_eq!(pane.detail.x, pane.master.right() + 2);
        assert_eq!(pane.master.right() + 2 + pane.detail.width, area.right());
    }

    let fixed_detail = base.fixed_detail_share(40, 34, 56);
    let expected = [(107, 42), (120, 48), (160, 56)];
    for (width, detail_width) in expected {
        let pane = fixed_detail.layout(Rect::new(7, 4, width, 18), &state);
        assert_eq!(pane.detail.width, detail_width, "width={width}");
        assert_eq!(pane.detail.x, pane.master.right() + 2);
        assert_eq!(pane.detail.right(), 7 + width);
    }

    let first_pane = fixed_master.split_pane();
    let (master_constraints, detail_constraints) =
        first_pane.pane_constraints(&SplitPaneState::new(50), Constraints::loose(120, 18));
    assert_eq!(
        (master_constraints.max.0, detail_constraints.max.0),
        (40, 78)
    );
    assert_eq!(
        first_pane.required_extent(&SplitPaneState::new(50), 35, 34),
        105,
        "required extent shares the resolved fixed-master geometry"
    );

    let detail_pane = fixed_detail.split_pane();
    let (master_constraints, detail_constraints) =
        detail_pane.pane_constraints(&SplitPaneState::new(50), Constraints::loose(120, 18));
    assert_eq!(
        (master_constraints.max.0, detail_constraints.max.0),
        (70, 48)
    );
    assert_eq!(
        detail_pane.required_extent(&SplitPaneState::new(50), 70, 36),
        119,
        "required extent shares the resolved fixed-detail geometry"
    );

    let tiny_area = Rect::new(7, 4, 40, 18);
    let tiny = fixed_master.layout(tiny_area, &state);
    assert_eq!(tiny.master, tiny_area);
    assert!(tiny.detail.is_empty());
    assert!(tiny.master.right() <= tiny_area.right());

    let responsive = MasterDetail::new(CONTAINER, MASTER, DETAIL)
        .fixed_master_share(33, 26, 40)
        .layout(Rect::new(7, 4, 80, 12), &state);
    assert_eq!(responsive.mode, MasterDetailMode::Master);
    let mut drawer = harness(PolicyPage {
        sizing: Sizing::FixedMaster,
        area: Rect::new(0, 0, 80, 12),
        ..PolicyPage::default()
    });
    drawer.runtime_mut().set_focus(Some(MASTER));
    drawer.draw();
    let _ = drawer.key(KeyCode::Tab);
    assert_eq!(layout(&drawer).mode, MasterDetailMode::Drawer);
}

#[test]
fn fixed_modes_have_no_seam_bindings_pointer_capture_or_reset_and_keep_ratio_on_return() {
    let mut h = harness(PolicyPage {
        resize_bindings: Some(RESIZE_BINDINGS),
        preserve_drag_offset: true,
        step: 5,
        ..PolicyPage::default()
    });
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let before_fixed = h.app().state.split().expect("the split was initialized");
    h.app_mut().sizing = Sizing::FixedDetail;
    h.draw();

    assert_eq!(
        h.area_of_part(CONTAINER, PartRef::of(Part::SEAM)),
        None,
        "fixed mode does not register an interactive seam"
    );
    let part = h.app().control().split_pane();
    assert!(part.bindings(BindingState::default()).is_empty());
    let fixed_layout = layout(&h);
    assert_eq!(fixed_layout.detail.width, 48);
    let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
    let _ = h.mouse(MouseKind::Down, fixed_layout.master.right(), 8);
    let _ = h.mouse(MouseKind::Drag, fixed_layout.master.right() + 8, 8);
    let _ = h.mouse(MouseKind::Up, fixed_layout.master.right() + 8, 8);
    let _ = h.double_click(fixed_layout.master.right(), 8);
    assert_eq!(h.runtime().capture_owner(), None);
    assert_eq!(
        h.app()
            .state
            .split()
            .expect("the split remains initialized"),
        before_fixed,
        "fixed mode ignores resize/reset input without changing any saved split state"
    );

    h.app_mut().sizing = Sizing::Resizable;
    h.draw();
    assert_eq!(
        h.app()
            .state
            .split()
            .expect("the split remains initialized"),
        before_fixed,
        "fixed layout does not overwrite the saved interactive ratio"
    );
    h.runtime_mut().set_focus(Some(MASTER));
    h.draw();
    let before_resize = layout(&h).master.width;
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(layout(&h).master.width, before_resize + 5);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
