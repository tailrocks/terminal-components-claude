//! The shared conformance suite (`COMPONENT_ARCHITECTURE.md` §16.2).
//!
//! One trait implemented once per public component; one macro generates the
//! whole 20-case matrix. Cases marked with a capability run only when the
//! component declares it; the driver never lets a declared capability skip
//! its case.

pub mod driver;

use bitflags::bitflags;
use ratatui_core::layout::Rect;
use tui_next::{
    ActionKey, Binding, BindingState, Chord, ColorLevel, Cx, Family, Id, ItemKey, Part, PartRef,
    Response, StateFlags, Status, StylePatch, Theme, Ui,
};

bitflags! {
    /// What a component can do, selecting the capability-gated cases.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Caps: u32 {
        /// Has a keyboard and a mouse activation path.
        const ACTIVATES   = 1 << 0;
        /// Can be disabled.
        const DISABLEABLE = 1 << 1;
        /// Registers a focus stop.
        const FOCUSABLE   = 1 << 2;
        /// Takes items + a key fn.
        const COLLECTION  = 1 << 3;
        /// Has an edit lifecycle.
        const EDITS       = 1 << 4;
        /// Scrolls.
        const SCROLLS     = 1 << 5;
        /// Opens a layer.
        const OVERLAY     = 1 << 6;
        /// Claims pointer capture.
        const CAPTURES    = 1 << 7;
        /// Writes the hardware cursor.
        const CURSOR      = 1 << 8;
        /// May hold secret bytes.
        const SECRET      = 1 << 9;
        /// The focus entry sets `swallows_typing`; bare `Char` chords are
        /// exempt from case 20's reverse direction.
        const TYPES       = 1 << 10;
        /// Opens a layer that traps focus; implies [`Caps::OVERLAY`].
        const TRAPS_FOCUS = 1 << 11;
        /// Accepts a readiness prop (`.status(Status)` or an `EmptyState`)
        /// and therefore owes §11.4's `BUSY`/`ERROR` affordance.
        const REPORTS_STATUS = 1 << 12;
        /// Owns a selected/checked value and renders its selected affordance.
        /// This is independent of [`Caps::COLLECTION`]: some collections only
        /// navigate, while some scalar choices render selection.
        const SELECTS = 1 << 13;
    }
}

/// Pointer gesture equivalent to an activation chord in cases 2 and 12.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerGesture {
    /// One primary-button press and release.
    Click,
    /// Two clicks inside the runtime's double-click window.
    DoubleClick,
}

/// One fixture row for collection cases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureRow {
    /// The stable key.
    pub key: ItemKey,
    /// The label.
    pub label: String,
    /// Secondary text.
    pub meta: String,
    /// Disabled.
    pub disabled: bool,
}

/// The knobs the driver varies.
#[derive(Clone, Debug)]
pub struct Fixture {
    /// Render disabled.
    pub disabled: bool,
    /// Render read-only.
    pub read_only: bool,
    /// The theme.
    pub theme: Theme,
    /// The colour level.
    pub color: ColorLevel,
    /// The area the component draws into.
    pub area: Rect,
    /// Real rows; `update`/`draw` borrow from here and `reorder` permutes it.
    pub rows: Vec<FixtureRow>,
    /// Owner-supplied data decoration for row/cell-local semantic status.
    pub decor_flags: StateFlags,
    /// Caller-owned committed selection used by controlled-value fixtures.
    pub selected: bool,
    /// Requested state for the mono case, if this is a reference rendering.
    ///
    /// Private so [`Fixture::force`] remains the only way to set a forced
    /// state and its coupled readiness status.
    reference_state: Option<StateFlags>,
    /// An instance patch for the override case.
    pub patch: Option<(Part, StylePatch)>,
    /// Secret bytes to type for the secret case.
    pub secret: Option<&'static str>,
    /// Data readiness, derived from [`Fixture::force`] so that a state whose
    /// affordance comes from *props* (the spinner) is actually reachable.
    /// Private for the same reason as [`Fixture::forced`].
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
    /// The forced state flags, if this is a reference rendering.
    #[must_use]
    pub const fn forced(&self) -> Option<StateFlags> {
        self.reference_state
    }

    /// The readiness coupled to the forced state.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Request reference state `s` **and make semantic state real**. Even an
    /// empty `s` marks this as an inert reference rendering rather than a live
    /// baseline instance. Runtime-owned focus and press are injected by the
    /// driver's outer [`Ui::reference`] scope; disabled, selection, readiness
    /// and row decoration remain caller-owned data.
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

/// The states case 9 compares by default: **all ten** of §16.2's list
/// (MA-8). A five-state default silently gave every component a weaker check
/// than the contract asks for.
///
/// [`Conformance::mono_states`] may only **narrow** this list — the driver
/// asserts that, and asserts that every state the component's [`Caps`] imply
/// is still present.
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

/// The states a capability implies, which `mono_states()` may not drop.
///
/// A **union**, not a first-match: an `if / else if` chain let a component
/// declaring `EDITS | DISABLEABLE` keep only `EDITING`, which is exactly the
/// escape MA-8 exists to close.
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
        // §11.4: a component that takes a readiness prop owes the spinner and
        // the error affordance, so neither state may be narrowed away.
        out.push(StateFlags::BUSY);
        out.push(StateFlags::ERROR);
    }
    out
}

/// One registration per public component. `State = ()` for stateless components.
pub trait Conformance: 'static {
    /// The component name (`"button"`, `"list"`, …).
    const NAME: &'static str;
    /// The theme family.
    const FAMILY: Family;
    /// The declared parts.
    const PARTS: &'static [Part];

    /// The durable state type.
    type State: Default + Clone + PartialEq + core::fmt::Debug;
    /// The action type; compared structurally by cases 2 and 12.
    type Action: PartialEq + core::fmt::Debug;
    /// The binding command type.
    type Cmd: Copy + 'static;

    /// The capabilities.
    fn caps() -> Caps;
    /// The component's id.
    fn id() -> Id;
    /// Focusable/editable child id for a composite; defaults to its root id.
    fn control_id() -> Id {
        Self::id()
    }
    /// Owner whose activation part is exercised (`ACTIVATES`).
    fn activation_id() -> Id {
        Self::control_id()
    }
    /// Scroll owner id for a composite; defaults to its root id.
    fn scroll_id() -> Id {
        Self::id()
    }
    /// Focusable opener for an overlay composite; defaults to its control.
    fn opener_id() -> Id {
        Self::control_id()
    }
    /// Run `update` against the fixture.
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action>;
    /// Run `draw` against the fixture.
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture);

    /// Chords that activate (`ACTIVATES`).
    fn activation_chords() -> &'static [Chord] {
        &[]
    }
    /// The part a press/release activates.
    fn activation_part() -> PartRef {
        PartRef::of(Part::CONTAINER)
    }
    /// The pointer gesture equivalent to [`Self::activation_chords`].
    fn activation_gesture() -> PointerGesture {
        PointerGesture::Click
    }
    /// The binding table for a state.
    fn bindings(_s: BindingState) -> &'static [Binding<Self::Cmd>] {
        &[]
    }
    /// Raw legacy/dynamic chords intentionally handled outside a static table.
    fn legacy_key_chords() -> &'static [Chord] {
        &[]
    }
    /// Caller-declared action bindings that depend on the fixture.
    fn dynamic_bindings(_fixture: &Fixture) -> Vec<(ActionKey, Chord)> {
        Vec::new()
    }
    /// Owner that publishes one caller-declared action binding.
    fn dynamic_binding_id(_action: ActionKey) -> Id {
        Self::control_id()
    }
    /// Make the scroll case overflow when the default fixture fits exactly.
    fn prepare_scroll_fixture(_fixture: &mut Fixture) {}
    /// Lifecycle updates required before the scroll target exists.
    fn scroll_setup_ticks() -> usize {
        0
    }
    /// The keys of the fixture rows (`COLLECTION`).
    fn item_keys(_f: &Fixture) -> Vec<ItemKey> {
        Vec::new()
    }
    /// Permute the fixture rows (`COLLECTION`).
    fn reorder(_f: &mut Fixture, _perm: &[usize]) {}
    /// Chords that reveal `key` after a reorder moved it outside the viewport.
    fn reveal_item_chords(_key: ItemKey, _f: &Fixture) -> Vec<Chord> {
        Vec::new()
    }
    /// The key an action carries (`COLLECTION`).
    fn action_key_of(_a: &Self::Action) -> Option<ItemKey> {
        None
    }
    /// The part that addresses row `k` (`COLLECTION`).
    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, k)
    }
    /// Secret bytes (`SECRET`).
    fn secret_bytes() -> &'static str {
        ""
    }
    /// The states the component can wear, for the mono case.
    fn mono_states() -> &'static [StateFlags] {
        DEFAULT_MONO_STATES
    }
    /// Fixture used for one mono reference state.
    fn mono_fixture(_state: StateFlags) -> Fixture {
        Fixture::default()
    }
    /// Chords used to put durable state into the requested mono state.
    ///
    /// The driver applies them to a focused component, then copies the
    /// resulting durable state into the isolated reference rendering.
    fn mono_setup_chords(_state: StateFlags) -> &'static [Chord] {
        &[]
    }
    /// Explain every state omitted by [`Self::mono_states`]. Case 9 checks
    /// that this is empty exactly when the default state set is unchanged and
    /// that every dropped flag name appears in the explanation.
    ///
    /// Its scope is deliberately small: it may excuse **only** states that no
    /// declared capability implies. A capability-implied state cannot be
    /// dropped at all — [`mono_states_required_by`] fails case 9 before this
    /// string is ever read — so a narrowing reason can never be the place a
    /// §11.4 affordance goes missing. The sentence therefore answers one
    /// question, "why is this state out of scope for a component that cannot
    /// enter it", and never stands in as evidence that an affordance exists.
    fn mono_narrowing_reason() -> &'static str {
        ""
    }
    /// The chord that opens the component's layer (`OVERLAY`); defaults to
    /// the first activation chord.
    fn open_chord() -> Option<Chord> {
        Self::activation_chords().first().copied()
    }
    /// Open the component's layer directly during the overlay case.
    ///
    /// Return `true` when the component uses an owner/controller request
    /// instead of an opener chord. The driver invokes this hook only for the
    /// explicit overlay-open step.
    fn open_overlay(_cx: &mut Cx<'_>, _state: &mut Self::State, _fixture: &Fixture) -> bool {
        false
    }
    /// The layer id the component opens (`OVERLAY`).
    fn layer_id() -> Option<Id> {
        None
    }
}

/// Generate the conformance matrix: one module per component, named by the
/// identifier before `=>`, so the fully-qualified test names read
/// `conformance::<component>::<case>`.
#[macro_export]
macro_rules! conformance_suite {
    ($($name:ident => $case:ty),+ $(,)?) => {
        $(
            mod $name {
                use super::*;
                use $crate::conformance::driver as d;
                /// D-8: the macro cannot derive the module identifier from
                /// `Conformance::NAME`, so the ident is written explicitly.
                /// This guard is what keeps the two from drifting: a renamed
                /// `NAME` with an unrenamed module fails here.
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
        /// Every registered case's name, for the registry checks.
        pub fn registered_cases() -> Vec<&'static str> {
            vec![$(<$case as $crate::Conformance>::NAME),+]
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{Caps, Conformance, Fixture, PointerGesture, StateFlags, mono_states_required_by};
    use tui_next::{Cx, Family, Id, Part, Rect, Response, Ui};

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

        // Neither bit is implied by the empty capability set, so the gate stays
        // off for components that never take a readiness prop.
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

        let enabled = disabled.force(StateFlags::empty());
        assert!(!enabled.disabled);
        assert!(!enabled.selected);
        assert_eq!(enabled.forced(), Some(StateFlags::empty()));

        let forced_selected = Fixture::default().force(StateFlags::SELECTED);
        assert!(
            forced_selected.selected,
            "selection is semantic fixture data"
        );
    }
}
