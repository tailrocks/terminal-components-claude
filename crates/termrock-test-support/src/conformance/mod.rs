pub mod driver;

use bitflags::bitflags;
use ratatui_core::layout::Rect;
use termrock::{
    ActionKey, Binding, BindingState, Chord, ColorLevel, Cx, Family, Id, ItemKey, Part, PartRef,
    Response, StateFlags, Status, StylePatch, Theme, Ui,
};

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Caps: u32 {
        const ACTIVATES   = 1 << 0;
        const DISABLEABLE = 1 << 1;
        const FOCUSABLE   = 1 << 2;
        const COLLECTION  = 1 << 3;
        const EDITS       = 1 << 4;
        const SCROLLS     = 1 << 5;
        const OVERLAY     = 1 << 6;
        const CAPTURES    = 1 << 7;
        const CURSOR      = 1 << 8;
        const SECRET      = 1 << 9;
        const TYPES       = 1 << 10;
        const TRAPS_FOCUS = 1 << 11;
        const REPORTS_STATUS = 1 << 12;
        const SELECTS = 1 << 13;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerGesture {
    Click,
    DoubleClick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureRow {
    pub key: ItemKey,
    pub label: String,
    pub meta: String,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct Fixture {
    pub disabled: bool,
    pub read_only: bool,
    pub theme: Theme,
    pub color: ColorLevel,
    pub area: Rect,
    pub rows: Vec<FixtureRow>,
    pub decor_flags: StateFlags,
    pub selected: bool,
    reference_state: Option<StateFlags>,
    pub patch: Option<(Part, StylePatch)>,
    pub secret: Option<&'static str>,
    status: Status,
}

impl Default for Fixture {
    fn default() -> Self {
        Fixture {
            disabled: false,
            read_only: false,
            theme: Theme::junie(),
            color: ColorLevel::TrueColor,
            area: Rect::new(2, 2, 30, 6),
            rows: (0..5)
                .map(|i| FixtureRow {
                    key: ItemKey::num(100 + i),
                    label: format!("row {i}"),
                    meta: format!("meta {i}"),
                    disabled: false,
                })
                .collect(),
            decor_flags: StateFlags::empty(),
            selected: false,
            reference_state: None,
            patch: None,
            secret: None,
            status: Status::Ready,
        }
    }
}

impl Fixture {
    #[must_use]
    pub const fn forced(&self) -> Option<StateFlags> {
        self.reference_state
    }

    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    #[must_use]
    pub fn force(mut self, s: StateFlags) -> Self {
        self.reference_state = Some(s);
        self.disabled = s.contains(StateFlags::DISABLED);
        self.selected = s.contains(StateFlags::SELECTED);
        self.decor_flags = s & (StateFlags::ERROR | StateFlags::WARNING);
        for row in &mut self.rows {
            row.disabled = self.disabled;
        }
        self.status = if s.contains(StateFlags::BUSY) {
            Status::Busy
        } else if s.contains(StateFlags::LOADING) {
            Status::Loading
        } else if s.contains(StateFlags::ERROR) {
            Status::Error
        } else {
            Status::Ready
        };
        self
    }
}

pub const DEFAULT_MONO_STATES: &[StateFlags] = &[
    StateFlags::empty(),
    StateFlags::FOCUSED,
    StateFlags::SELECTED,
    StateFlags::PRESSED,
    StateFlags::DISABLED,
    StateFlags::ERROR,
    StateFlags::WARNING,
    StateFlags::EDITING,
    StateFlags::BUSY,
    StateFlags::ACTIVE,
];

#[must_use]
pub fn mono_states_required_by(caps: Caps) -> Vec<StateFlags> {
    let mut out = vec![StateFlags::empty()];
    if caps.contains(Caps::FOCUSABLE) {
        out.push(StateFlags::FOCUSED);
    }
    if caps.contains(Caps::ACTIVATES) {
        out.push(StateFlags::PRESSED);
    }
    if caps.contains(Caps::DISABLEABLE) {
        out.push(StateFlags::DISABLED);
    }
    if caps.contains(Caps::EDITS) {
        out.push(StateFlags::EDITING);
    }
    if caps.contains(Caps::SELECTS) {
        out.push(StateFlags::SELECTED);
    }
    if caps.contains(Caps::REPORTS_STATUS) {
        out.push(StateFlags::BUSY);
        out.push(StateFlags::ERROR);
    }
    out
}

pub trait Conformance: 'static {
    const NAME: &'static str;
    const FAMILY: Family;
    const PARTS: &'static [Part];

    type State: Default + Clone + PartialEq + core::fmt::Debug;
    type Action: PartialEq + core::fmt::Debug;
    type Cmd: Copy + 'static;

    fn caps() -> Caps;
    fn id() -> Id;
    fn control_id() -> Id {
        Self::id()
    }
    fn activation_id() -> Id {
        Self::control_id()
    }
    /// Some controls are only actionable after their owning layer is opened.
    /// The shared activation checks should exercise that real user flow even
    /// when the action control itself is hidden before opening.
    fn activation_requires_overlay() -> bool {
        false
    }
    fn scroll_id() -> Id {
        Self::id()
    }
    fn opener_id() -> Id {
        Self::control_id()
    }
    /// The action emitted while opening an owning layer, when it is distinct
    /// from activating the control inside that layer. Conformance probes use
    /// this to ensure setup responses cannot satisfy activation assertions.
    fn opener_action() -> Option<Self::Action> {
        None
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action>;
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture);
    /// Base fixture each driver check starts from.
    ///
    /// A case whose widget cannot paint or register its focus target in the
    /// default 30x6 rect widens it here instead of special-casing a check.
    fn fixture() -> Fixture {
        Fixture::default()
    }

    fn activation_chords() -> &'static [Chord] {
        &[]
    }
    fn activation_part() -> PartRef {
        PartRef::of(Part::CONTAINER)
    }
    fn activation_gesture() -> PointerGesture {
        PointerGesture::Click
    }
    fn bindings(_s: BindingState) -> &'static [Binding<Self::Cmd>] {
        &[]
    }
    fn binding_states() -> &'static [BindingState] {
        const STATES: &[BindingState] = &[
            BindingState {
                flags: StateFlags::empty(),
            },
            BindingState {
                flags: StateFlags::FOCUSED,
            },
        ];
        STATES
    }
    fn legacy_key_chords() -> &'static [Chord] {
        &[]
    }
    fn dynamic_bindings(_fixture: &Fixture) -> Vec<(ActionKey, Chord)> {
        Vec::new()
    }
    fn dynamic_binding_id(_action: ActionKey) -> Id {
        Self::control_id()
    }
    fn prepare_scroll_fixture(_fixture: &mut Fixture) {}
    fn scroll_setup_ticks() -> usize {
        0
    }
    fn item_keys(_f: &Fixture) -> Vec<ItemKey> {
        Vec::new()
    }
    fn reorder(_f: &mut Fixture, _perm: &[usize]) {}
    fn reveal_item_chords(_key: ItemKey, _f: &Fixture) -> Vec<Chord> {
        Vec::new()
    }
    fn action_key_of(_a: &Self::Action) -> Option<ItemKey> {
        None
    }
    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, k)
    }
    fn secret_bytes() -> &'static str {
        ""
    }
    fn mono_states() -> &'static [StateFlags] {
        DEFAULT_MONO_STATES
    }
    fn mono_fixture(_state: StateFlags) -> Fixture {
        Fixture::default()
    }
    fn mono_setup_chords(_state: StateFlags) -> &'static [Chord] {
        &[]
    }
    fn mono_narrowing_reason() -> &'static str {
        ""
    }
    fn open_chord() -> Option<Chord> {
        Self::activation_chords().first().copied()
    }
    fn open_overlay(_cx: &mut Cx<'_>, _state: &mut Self::State, _fixture: &Fixture) -> bool {
        false
    }
    fn layer_id() -> Option<Id> {
        None
    }
}

#[macro_export]
macro_rules! conformance_suite {
    ($($name:ident => $case:ty),+ $(,)?) => {
        $(
            mod $name {
                use super::*;
                use $crate::conformance::driver as d;
                #[test] fn name_matches_the_module() {
                    assert_eq!(
                        <$case as $crate::conformance::Conformance>::NAME,
                        stringify!($name),
                        "conformance_suite!: module `{}` registers a case named `{}`",
                        stringify!($name),
                        <$case as $crate::conformance::Conformance>::NAME,
                    );
                }
                #[test] fn disabled_cannot_activate() { d::disabled_cannot_activate::<$case>(); }
                #[test] fn keyboard_and_mouse_activation_are_equivalent() { d::keyboard_and_mouse_activation_are_equivalent::<$case>(); }
                #[test] fn traversal_order_is_registration_order() { d::traversal_order_is_registration_order::<$case>(); }
                #[test] fn hover_does_not_steal_focus() { d::hover_does_not_steal_focus::<$case>(); }
                #[test] fn draw_twice_is_byte_identical() { d::draw_twice_is_byte_identical::<$case>(); }
                #[test] fn draw_twice_leaves_state_equal() { d::draw_twice_leaves_state_equal::<$case>(); }
                #[test] fn draw_does_not_commit_or_cancel() { d::draw_does_not_commit_or_cancel::<$case>(); }
                #[test] fn draw_stays_inside_its_area() { d::draw_stays_inside_its_area::<$case>(); }
                #[test] fn mono_states_are_distinguishable() { d::mono_states_are_distinguishable::<$case>(); }
                #[test] fn local_override_does_not_mutate_the_theme() { d::local_override_does_not_mutate_the_theme::<$case>(); }
                #[test] fn id_separator_collision_free() { d::id_separator_collision_free::<$case>(); }
                #[test] fn item_identity_survives_reorder() { d::item_identity_survives_reorder::<$case>(); }
                #[test] fn focus_reconcile_follows_the_rule() { d::focus_reconcile_follows_the_rule::<$case>(); }
                #[test] fn focus_trap_and_restore() { d::focus_trap_and_restore::<$case>(); }
                #[test] fn pointer_capture_delivers_drag_and_release() { d::pointer_capture_delivers_drag_and_release::<$case>(); }
                #[test] fn wheel_at_boundary_is_consumed_without_repaint() { d::wheel_at_boundary_is_consumed_without_repaint::<$case>(); }
                #[test] fn cursor_write_is_rejected_off_top_layer() { d::cursor_write_is_rejected_off_top_layer::<$case>(); }
                #[test] fn secret_never_appears_in_debug() { d::secret_never_appears_in_debug::<$case>(); }
                #[test] fn survives_tiny_rects_0x0_to_3x3() { d::survives_tiny_rects_0x0_to_3x3::<$case>(); }
                #[test] fn bindings_match_handled_keys() { d::bindings_match_handled_keys::<$case>(); }
            }
        )+
        pub fn registered_cases() -> Vec<&'static str> {
            vec![$(<$case as $crate::Conformance>::NAME),+]
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{Caps, Conformance, Fixture, PointerGesture, StateFlags, mono_states_required_by};
    use termrock::{Cx, Family, Id, Part, Rect, Response, Status, Ui};

    const ROOT: Id = Id::root("conformance.framework.composite");
    const CONTROL: Id = Id::root("conformance.framework.composite.control");

    struct CompositeIds;

    impl Conformance for CompositeIds {
        const NAME: &'static str = "composite_ids";
        const FAMILY: Family = Family::FORM;
        const PARTS: &'static [Part] = &[Part::CONTAINER];
        type State = ();
        type Action = ();
        type Cmd = ();

        fn caps() -> Caps {
            Caps::FOCUSABLE | Caps::SCROLLS
        }

        fn id() -> Id {
            ROOT
        }

        fn control_id() -> Id {
            CONTROL
        }

        fn activation_id() -> Id {
            Id::root("conformance.composite.activation")
        }

        fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
            Response::ignored()
        }

        fn draw(_ui: &mut Ui<'_>, _area: Rect, _st: &(), _f: &Fixture) {}
    }

    #[test]
    fn fixture_distinguishes_live_from_forced_empty() {
        assert_eq!(Fixture::default().forced(), None);
        assert_eq!(
            Fixture::default().force(StateFlags::empty()).forced(),
            Some(StateFlags::empty())
        );
    }

    #[test]
    fn reports_status_requires_busy_and_error() {
        let reports = mono_states_required_by(Caps::REPORTS_STATUS);
        assert!(
            reports.contains(&StateFlags::BUSY),
            "REPORTS_STATUS must imply BUSY, got {reports:?}"
        );
        assert!(
            reports.contains(&StateFlags::ERROR),
            "REPORTS_STATUS must imply ERROR, got {reports:?}"
        );

        assert_eq!(
            mono_states_required_by(Caps::empty()),
            vec![StateFlags::empty()]
        );
    }

    #[test]
    fn selects_alone_requires_selected() {
        assert!(mono_states_required_by(Caps::SELECTS).contains(&StateFlags::SELECTED));
        assert!(
            !mono_states_required_by(Caps::COLLECTION).contains(&StateFlags::SELECTED),
            "collection identity does not imply a selection affordance"
        );
    }

    #[test]
    fn pointer_gestures_are_exactly_click_and_double_click() {
        let gestures = [PointerGesture::Click, PointerGesture::DoubleClick];
        assert_eq!(gestures.len(), 2);
        assert_ne!(gestures[0], gestures[1]);
    }

    #[test]
    fn composite_root_control_and_scroll_ids_do_not_collapse() {
        assert_eq!(CompositeIds::id(), ROOT);
        assert_eq!(CompositeIds::control_id(), CONTROL);
        assert_ne!(CompositeIds::activation_id(), CompositeIds::control_id());
        assert_eq!(CompositeIds::scroll_id(), ROOT);
        assert_eq!(CompositeIds::opener_id(), CONTROL);
    }

    #[test]
    fn force_couples_and_clears_real_disabled_state() {
        let disabled = Fixture::default().force(StateFlags::DISABLED);
        assert!(disabled.disabled);
        assert!(!disabled.selected);
        assert_eq!(disabled.forced(), Some(StateFlags::DISABLED));
        assert_eq!(disabled.status(), Status::Ready);

        let enabled = disabled.force(StateFlags::empty());
        assert!(!enabled.disabled);
        assert!(!enabled.selected);
        assert_eq!(enabled.forced(), Some(StateFlags::empty()));
        assert_eq!(enabled.status(), Status::Ready);

        assert_eq!(
            Fixture::default().force(StateFlags::BUSY).status(),
            Status::Busy
        );
        assert_eq!(
            Fixture::default().force(StateFlags::LOADING).status(),
            Status::Loading
        );
        assert_eq!(
            Fixture::default().force(StateFlags::ERROR).status(),
            Status::Error
        );

        let forced_selected = Fixture::default().force(StateFlags::SELECTED);
        assert!(
            forced_selected.selected,
            "selection is semantic fixture data"
        );
    }
}
