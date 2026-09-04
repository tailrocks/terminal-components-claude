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
    Binding, BindingState, Chord, ColorLevel, Cx, Family, Id, ItemKey, Part, PartRef, Response,
    StateFlags, Status, StylePatch, Theme, Ui,
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
    }
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
    /// Forced state flags (`state_override`) for the mono case.
    pub state_override: StateFlags,
    /// An instance patch for the override case.
    pub patch: Option<(Part, StylePatch)>,
    /// Secret bytes to type for the secret case.
    pub secret: Option<&'static str>,
    /// Data readiness, derived from [`Fixture::force`] so that a state whose
    /// affordance comes from *props* (the spinner) is actually reachable.
    pub status: Status,
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
            state_override: StateFlags::empty(),
            patch: None,
            secret: None,
            status: Status::Ready,
        }
    }
}

impl Fixture {
    /// Force `s` **and make it real**: a state whose affordance comes from
    /// props rather than from a theme rule (`BUSY`/`LOADING`/`ERROR` drive
    /// `Status`, and `Status` drives the spinner) is unreachable through
    /// `state_override` alone, so case 9 would prove nothing about it.
    #[must_use]
    pub fn force(mut self, s: StateFlags) -> Self {
        self.state_override = s;
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
    if caps.contains(Caps::COLLECTION) {
        out.push(StateFlags::SELECTED);
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
    /// The binding table for a state.
    fn bindings(_s: BindingState) -> &'static [Binding<Self::Cmd>] {
        &[]
    }
    /// The keys of the fixture rows (`COLLECTION`).
    fn item_keys(_f: &Fixture) -> Vec<ItemKey> {
        Vec::new()
    }
    /// Permute the fixture rows (`COLLECTION`).
    fn reorder(_f: &mut Fixture, _perm: &[usize]) {}
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
    /// The chord that opens the component's layer (`OVERLAY`); defaults to
    /// the first activation chord.
    fn open_chord() -> Option<Chord> {
        Self::activation_chords().first().copied()
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
