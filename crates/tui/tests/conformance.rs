//! The conformance matrix (`COMPONENT_ARCHITECTURE.md` §16.2). Slice 3 has no
//! components yet; `ProbeCase` is a minimal button-like control written on
//! the `author` surface so the driver itself is exercised end to end. Slice 4
//! packages append their `Case`s to the `conformance_suite!` list below.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use tui_next::author::{
    Activated, Binding, BindingState, Bindings, Chord, Cx, Family, Focusability, FrameRead,
    GlyphRole, Id, Intent, ItemKey, KeyCode, Part, PartRef, Phase, Position, Rect, Response,
    ScrollState, StateFlags, StylePatch, Ui, Variant,
};
use tui_next::{
    ActionKey, Anchor, Brand, Button, ButtonCmd, Checkbox, ChipBar, ChipBarAction, ChipBarCmd,
    ChoiceCmd, Diagnostic, Dialog, DialogAction, Empty, EmptyState, Field, Hint, HintBar,
    HintLayer, KeyHint, KeyPhase, LayerSize, LayerSpec, List, ListAction, ListCmd, Meter,
    ProgressBar, Props, RadioGroup, RadioGroupAction, RadioGroupState, RowUi, ScreenAlign,
    ScrollRegion, Select, SelectAction, SelectCmd, SelectMode, SelectState, Slot, Spinner, Status,
    StatusAction, StatusBar, StatusItem, Tabs, TabsAction, TabsCmd, TextAction, TextArea,
    TextAreaState, TextCmd, TextInput, TextInputState, Theme, Toggle, binding_conflicts,
    resolve_anchor,
};
use tui_next_testing::conformance::{
    Caps, Conformance, Fixture, FixtureRow, mono_states_required_by,
};
use tui_next_testing::{Harness, Scene, conformance_suite};

const PROBE: Id = Id::root("conformance.probe");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCmd {
    Activate,
}

const BINDINGS: &[Binding<ProbeCmd>] = &[
    Binding {
        chord: Chord::key(KeyCode::Enter),
        cmd: ProbeCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Char(' ')),
        cmd: ProbeCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: false,
    },
];

#[derive(Clone, Debug, Default, PartialEq)]
struct ProbeState {
    fired: u32,
}

/// A button-like control: Enter / Space / click activate; writes the cursor
/// while focused; honours `disabled`, `state_override` and `patch`.
struct ProbeCase;

impl Conformance for ProbeCase {
    const NAME: &'static str = "probe";
    const FAMILY: Family = Family::BUTTON;
    const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL];
    type State = ProbeState;
    type Action = Activated;
    type Cmd = ProbeCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE | Caps::CURSOR
    }

    fn id() -> Id {
        PROBE
    }

    fn update(cx: &mut Cx<'_>, st: &mut ProbeState, f: &Fixture) -> Response<Activated> {
        let mut r = Response::ignored();
        for it in cx.intents(PROBE) {
            match it {
                Intent::Key(k) if !f.disabled => {
                    if Binding::lookup(BINDINGS, &k).is_some() {
                        st.fired += 1;
                        r = Response::action(Activated);
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click,
                    ..
                } if !f.disabled => {
                    st.fired += 1;
                    r = Response::action(Activated);
                }
                Intent::Pointer { .. } if !f.disabled => r = Response::changed(),
                _ => {}
            }
        }
        r.for_id(PROBE)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &ProbeState, f: &Fixture) {
        if area.is_empty() {
            return;
        }
        let focus = if f.disabled {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        ui.register_control(PROBE, area, focus);
        // `state_override` replaces the runtime state (A11: render a forced state)
        let forced = f.forced();
        let mut live = if forced.is_empty() {
            ui.state(PROBE)
        } else {
            forced
        };
        if f.disabled {
            live |= StateFlags::DISABLED;
        }
        let style_for = |ui: &mut Ui<'_>, part: Part| match f.patch {
            Some((p, patch)) if p == part => {
                ui.style_patched(Family::BUTTON, Variant::DEFAULT, part, live, &patch)
            }
            _ => ui.style(Family::BUTTON, Variant::DEFAULT, part, live),
        };
        let container = style_for(ui, Part::CONTAINER);
        ui.fill(area, container.style);
        let gutter = style_for(ui, Part::GUTTER);
        let gutter_cell = Rect {
            width: 1.min(area.width),
            ..area
        };
        match gutter.glyph {
            Slot::Set(g) => {
                ui.glyph(gutter_cell, g, gutter.style);
            }
            Slot::Inherit | Slot::Clear => {
                ui.fill(gutter_cell, gutter.style);
            }
        }
        let label = style_for(ui, Part::LABEL);
        let mut text = Rect {
            x: area.x.saturating_add(1),
            width: area.width.saturating_sub(1),
            ..area
        };
        text.height = 1.min(area.height);
        if matches!(label.glyph, Slot::Set(GlyphRole::PressLeft)) {
            let used = ui.glyph(text, GlyphRole::PressLeft, label.style);
            text.x = text.x.saturating_add(used);
            text.width = text.width.saturating_sub(used);
            let used = ui.paint_str(text, "Probe", label.style);
            text.x = text.x.saturating_add(used);
            text.width = text.width.saturating_sub(used);
            ui.glyph(text, GlyphRole::PressRight, label.style);
        } else {
            ui.paint_str(text, "Probe", label.style);
        }
        if live.contains(StateFlags::FOCUSED) {
            ui.set_cursor(PROBE, Position::new(area.x.saturating_add(1), area.y));
        }
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::CONTAINER)
    }

    fn bindings(_s: BindingState) -> &'static [Binding<ProbeCmd>] {
        BINDINGS
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED ERROR WARNING EDITING BUSY ACTIVE: probe exposes no corresponding mono affordance"
    }
}

// ───────────────────────────── Slice 4 cases ─────────────────────────────

fn patch_of(f: &Fixture) -> &[(Part, StylePatch)] {
    f.patch.as_slice()
}

const BTN: Id = Id::root("conformance.button");

/// `Button`: Enter / Space / click activate; honours `disabled`,
/// `state_override` and `patch_part`.
struct ButtonCase;

impl Conformance for ButtonCase {
    const NAME: &'static str = "button";
    const FAMILY: Family = Family::BUTTON;
    const PARTS: &'static [Part] = Button::PARTS;
    type State = ();
    type Action = Activated;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE
    }

    fn id() -> Id {
        BTN
    }

    fn update(cx: &mut Cx<'_>, _st: &mut (), f: &Fixture) -> Response<Activated> {
        Button::new(BTN, "Probe")
            .disabled(f.disabled)
            .status(f.status())
            .update(cx)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut b = Button::new(BTN, "Probe")
            .disabled(f.disabled)
            .status(f.status())
            .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            b = b.state_override(f.forced());
        }
        b.draw(ui, area);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ButtonCmd>] {
        Button::new(BTN, "").bindings(s)
    }

    /// `BUSY` is kept: the spinner `Button` paints from
    /// `design.motion.spinner_frames` is a *symbol*, so it is
    /// mono-distinguishable without a theme rule — the driver makes the
    /// forced state real by setting `Status::Busy` on the props too.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED ERROR WARNING EDITING ACTIVE: Button has no collection, validation, edit, or active-item state"
    }
}

const INPUT: Id = Id::root("conformance.text_input");

/// The controlled value beside the input's durable state.
#[derive(Clone, Default, PartialEq)]
struct InputState {
    st: TextInputState,
    value: String,
}

impl core::fmt::Debug for InputState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InputState")
            .field("st", &self.st)
            .field("value", &"[redacted]")
            .finish()
    }
}

fn text_input(id: Id, f: &Fixture) -> TextInput<'_> {
    let mut t = TextInput::new(id)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .placeholder("Type here")
        .status(f.status())
        .patch_part(patch_of(f));
    if f.secret.is_some() {
        t = t.secret(tui_next::SecretPolicy::default());
    }
    if !f.forced().is_empty() {
        t = t.state_override(f.forced());
    }
    t
}

/// The value shown while nothing was typed: a fixture row, so the mono
/// states have text to underline.
fn shown_value<'a>(st: &'a InputState, f: &'a Fixture) -> &'a str {
    if st.value.is_empty() && st.st.error().is_none() {
        f.rows.first().map_or("", |r| r.label.as_str())
    } else {
        &st.value
    }
}

/// `TextInput`: the edit lifecycle, the cursor, typing, secrets.
struct TextInputCase;

impl Conformance for TextInputCase {
    const NAME: &'static str = "text_input";
    const FAMILY: Family = Family::INPUT;
    const PARTS: &'static [Part] = TextInput::PARTS;
    type State = InputState;
    type Action = TextAction;
    type Cmd = TextCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE
            | Caps::EDITS
            | Caps::CURSOR
            | Caps::TYPES
            | Caps::SECRET
            | Caps::DISABLEABLE
    }

    fn id() -> Id {
        INPUT
    }

    fn update(cx: &mut Cx<'_>, st: &mut InputState, f: &Fixture) -> Response<TextAction> {
        text_input(INPUT, f).update(cx, &mut st.st, &mut st.value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &InputState, f: &Fixture) {
        text_input(INPUT, f)
            .value(shown_value(st, f))
            .draw(ui, area, &st.st);
    }

    fn bindings(s: BindingState) -> &'static [Binding<TextCmd>] {
        TextInput::new(INPUT).bindings(s)
    }

    fn secret_bytes() -> &'static str {
        "hunter2"
    }

    /// The full set the caps imply. `DISABLED` is reachable because §11.4's
    /// mono table now carries `(FIELD, DISABLED)` and `(TEXT, DISABLED)` —
    /// the two parts a text control paints for its own content — and `BUSY`
    /// because `draw` paints the readiness spinner in the trailing cell.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 6] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
            StateFlags::ERROR,
            StateFlags::DISABLED,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED WARNING ACTIVE: TextInput has no selection, press, warning, or active-item affordance"
    }
}

const FIELD_INPUT: Id = Id::root("conformance.field");

/// `Field` chrome over a `TextInput`.
struct FieldCase;

impl Conformance for FieldCase {
    const NAME: &'static str = "field";
    const FAMILY: Family = Family::FIELD;
    const PARTS: &'static [Part] = Field::<TextInput<'static>>::PARTS;
    type State = InputState;
    type Action = TextAction;
    type Cmd = TextCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::EDITS | Caps::CURSOR | Caps::TYPES | Caps::DISABLEABLE
    }

    fn id() -> Id {
        FIELD_INPUT
    }

    fn update(cx: &mut Cx<'_>, st: &mut InputState, f: &Fixture) -> Response<TextAction> {
        text_input(FIELD_INPUT, f).update(cx, &mut st.st, &mut st.value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &InputState, f: &Fixture) {
        let error = f.forced().contains(StateFlags::ERROR);
        let mut field = Field::new(
            "Label",
            text_input(FIELD_INPUT, f).value(shown_value(st, f)),
        )
        .required(true)
        .help("Help text")
        .error(error.then_some("Something is wrong"))
        .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            field = field.state_override(f.forced());
        }
        field.draw(ui, area, &st.st);
    }

    fn bindings(s: BindingState) -> &'static [Binding<TextCmd>] {
        TextInput::new(FIELD_INPUT).bindings(s)
    }

    /// `DISABLED` no longer depends on the chrome's `LABEL` being the one
    /// part a mono rule reaches: the control's own `FIELD`/`TEXT` carry the
    /// `DIM` too, so this asserts what §29 requires rather than what the
    /// chrome happens to paint.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
            StateFlags::DISABLED,
            StateFlags::ERROR,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED WARNING BUSY ACTIVE: Field has no collection, press, warning, readiness, or active-item affordance"
    }
}

const LIST: Id = Id::root("conformance.list");

fn row_key(r: &FixtureRow) -> ItemKey {
    r.key
}

fn row_paint(r: &FixtureRow, u: &mut RowUi<'_>) {
    u.label(&r.label);
    u.meta(&r.meta);
}

fn row_label(r: &FixtureRow, u: &mut RowUi<'_>) {
    u.label(&r.label);
}

fn row_disabled(r: &FixtureRow) -> bool {
    r.disabled
}

type FixtureList<'a> =
    List<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn list(f: &Fixture) -> FixtureList<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_paint;
    let disabled: &dyn Fn(&FixtureRow) -> bool = &row_disabled;
    let mut l = List::new(LIST)
        .key(key)
        .row(row)
        .disabled_item(disabled)
        .status(f.status())
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        l = l.state_override(f.forced());
    }
    l
}

/// `List`: keyed rows, cursor, choose / activate, wheel and the bar.
struct ListCase;

impl Conformance for ListCase {
    const NAME: &'static str = "list";
    const FAMILY: Family = Family::LIST;
    const PARTS: &'static [Part] = FixtureList::PARTS;
    type State = tui_next::ListState;
    type Action = ListAction;
    type Cmd = ListCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::SCROLLS
    }

    fn id() -> Id {
        LIST
    }

    fn update(cx: &mut Cx<'_>, st: &mut tui_next::ListState, f: &Fixture) -> Response<ListAction> {
        list(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &tui_next::ListState, f: &Fixture) {
        list(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<ListCmd>] {
        List::<FixtureRow>::new(LIST).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(a: &ListAction) -> Option<ItemKey> {
        match a {
            ListAction::Chose(k) | ListAction::Toggled(k) | ListAction::Activated(k) => Some(*k),
            ListAction::Moved | ListAction::ToggledAll => None,
        }
    }

    /// A list never edits and is never the `ACTIVE` element of a strip, so
    /// `EDITING` and `ACTIVE` are narrowed permanently. `BUSY`/`LOADING` are
    /// narrowed **temporarily**: §11.4 makes readiness a component
    /// obligation (paint `Part::ICON` from `design.motion.spinner_frames`),
    /// and `List` paints no such affordance yet. That is a named obligation
    /// on `List` (slice 4E/4F), not a permanent exemption — when the
    /// affordance lands, `BUSY` comes back here.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 7] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::WARNING,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "EDITING BUSY ACTIVE: List has no edit or readiness affordance, and active is represented by the selected row"
    }
}

const TABS: Id = Id::root("conformance.tabs");

fn tab_paint(r: &FixtureRow, u: &mut RowUi<'_>) {
    // the digit only, so every fixture tab fits the window
    u.label(r.label.get(4..).unwrap_or(""));
}

type FixtureTabs<'a> =
    Tabs<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn tabs(f: &Fixture) -> FixtureTabs<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = tab_paint;
    let mut t = Tabs::new(TABS)
        .key(key)
        .row(row)
        .closable(true)
        .status(f.status())
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        t = t.state_override(f.forced());
    }
    t
}

/// `Tabs`: stable keys, the active tab and the strip window.
struct TabsCase;

impl Conformance for TabsCase {
    const NAME: &'static str = "tabs";
    const FAMILY: Family = Family::TABS;
    const PARTS: &'static [Part] = FixtureTabs::PARTS;
    type State = tui_next::TabsState;
    type Action = TabsAction;
    type Cmd = TabsCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION
    }

    fn id() -> Id {
        TABS
    }

    fn update(cx: &mut Cx<'_>, st: &mut tui_next::TabsState, f: &Fixture) -> Response<TabsAction> {
        tabs(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &tui_next::TabsState, f: &Fixture) {
        tabs(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::TAB, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<TabsCmd>] {
        Tabs::<FixtureRow>::new(TABS).closable(true).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(a: &TabsAction) -> Option<ItemKey> {
        match a {
            TabsAction::Activated(k) | TabsAction::Close(k) => Some(*k),
            TabsAction::New => None,
        }
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::TAB, k)
    }

    /// `ACTIVE` is narrowed because a tab strip expresses it through forced
    /// `SELECTED`: the first windowed tab becomes `ACTIVE` only when the
    /// forced state contains `SELECTED`, so forcing `ACTIVE` directly paints
    /// nothing — and making it paint would make `SELECTED` and `ACTIVE`
    /// produce identical output and fail this very case's pairwise
    /// distinctness. An undocumented narrowing is indistinguishable from an
    /// oversight, so the reason is written here.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: Tabs has no validation, edit, readiness, or independent active-state affordance"
    }
}

const LAUNCH: Id = Id::root("conformance.dialog.launch");
const DLG: Id = Id::root("conformance.dialog");
const K_OPEN: ActionKey = ActionKey::custom("open");

fn dialog(f: &Fixture) -> Dialog<'_> {
    Dialog::confirm(DLG, "Confirm", "Proceed with the operation?").patch_part(patch_of(f))
}

/// `Dialog` as layer content behind a launcher button: the launcher is the
/// focusable, activatable id; activation opens the modal.
struct DialogCase;

impl Conformance for DialogCase {
    const NAME: &'static str = "dialog";
    const FAMILY: Family = Family::DIALOG;
    const PARTS: &'static [Part] = Dialog::PARTS;
    type State = tui_next::DialogState;
    type Action = DialogAction;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::OVERLAY | Caps::TRAPS_FOCUS
    }

    fn id() -> Id {
        LAUNCH
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut tui_next::DialogState,
        f: &Fixture,
    ) -> Response<DialogAction> {
        let launch = Button::new(LAUNCH, "Open").update(cx);
        if launch.activated() {
            cx.open_layer(DLG, LayerSpec::modal(DLG));
            return Response::action(DialogAction::Action(K_OPEN)).for_id(LAUNCH);
        }
        let mut r = launch.erase();
        let d = dialog(f).update(cx, st);
        if cx.is_open(DLG) {
            if let Some(DialogAction::Action(_)) = d.action_ref() {
                cx.close_layer(DLG, Some(ActionKey::CONFIRM));
            }
            return d;
        }
        if d.action_ref().is_some() {
            return d;
        }
        r |= d.erase();
        r |= Response::ignored();
        r.map_action(|()| DialogAction::Action(K_OPEN))
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &tui_next::DialogState, f: &Fixture) {
        let mut b = Button::new(LAUNCH, "Open");
        if !f.forced().is_empty() {
            b = b.state_override(f.forced());
        }
        let used = b.draw(ui, area);
        let below = Rect {
            y: area.y.saturating_add(used.height),
            height: area.height.saturating_sub(used.height),
            ..area
        };
        // the dialog chrome as plain content on the page (the digest and the
        // clipping cases; no action buttons, so the launcher stays the only
        // control between the sentinels), and inside its layer once opened
        dialog(f).actions(&[]).draw(ui, below, st, |_, _| {});
        ui.layer(DLG, |ui, a| {
            dialog(f).draw(ui, a, st, |_, _| {});
        });
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ButtonCmd>] {
        Button::new(LAUNCH, "").bindings(s)
    }

    fn layer_id() -> Option<Id> {
        Some(DLG)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 3] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: every symbol this case compares belongs \
         to the launcher `Button` — `id()` is LAUNCH and `draw` forces state on the button alone, \
         while the `Dialog` is drawn unforced — so the states below are narrowed off the *launcher*, \
         not off `Dialog`. SELECTED, EDITING and ACTIVE name nothing a modal can be, and DISABLED \
         belongs to its action buttons, which are their own `ButtonCase`. ERROR, WARNING and BUSY \
         are narrowed **temporarily**: §11.4 makes readiness a component obligation and `Dialog` \
         paints neither MARKER nor ICON, so widening this fixture today would only compare identical \
         frames. That is a named obligation on `Dialog` (slice 4 package 4F), not a permanent \
         exemption — when `Dialog` paints a readiness affordance, this case forces the dialog rather \
         than the launcher and ERROR, WARNING and BUSY come back here."
    }
}

const SCROLL: Id = Id::root("conformance.scroll_region");
const SCROLL_ROWS: usize = 40;

/// `ScrollRegion` over forty painted rows.
struct ScrollRegionCase;

impl Conformance for ScrollRegionCase {
    const NAME: &'static str = "scroll_region";
    const FAMILY: Family = Family::SCROLLBAR;
    const PARTS: &'static [Part] = ScrollRegion::PARTS;
    type State = ScrollState;
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::SCROLLS | Caps::CAPTURES
    }

    fn id() -> Id {
        SCROLL
    }

    fn update(cx: &mut Cx<'_>, st: &mut ScrollState, _f: &Fixture) -> Response<()> {
        ScrollRegion::new(SCROLL).update(cx, st, SCROLL_ROWS)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &ScrollState, f: &Fixture) {
        let mut sr = ScrollRegion::new(SCROLL).patch_part(patch_of(f));
        if !f.forced().is_empty() {
            sr = sr.state_override(f.forced());
        }
        let content = sr.draw(ui, area, st, SCROLL_ROWS);
        let view = ScrollRegion::view(st, content, SCROLL_ROWS);
        // the rows are the *container's* content, not a part of the scroll
        // region, so they are painted with a plain style query (which records
        // nothing) rather than through `RowUi`
        let style = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        for (row, i) in content.rows().zip(view.visible_range()) {
            ui.paint_str(row, &format!("row {i}"), style);
        }
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::THUMB)
    }

    /// `PRESSED` is **kept**, and that is the whole point of the case: a
    /// `Caps::CAPTURES` component holds `PRESSED` for the life of a thumb
    /// drag, and §11.4's `(THUMB, PRESSED, add(BOLD))` mono rule exists to
    /// keep that drag visible without hue. Narrowing it away left the rule
    /// unexecuted.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 2] = [StateFlags::empty(), StateFlags::PRESSED];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: a scroll region is never a focus \
         stop and registers no ring entry; it paints only TRACK and THUMB, and no §11.4 rule binds either \
         for those states. PRESSED is kept: Caps::CAPTURES means a live thumb capture holds PRESSED, and \
         the §11.4 THUMB/PRESSED mono rule gives it BOLD."
    }
}

const PROPS: Id = Id::root("conformance.props");

/// `Props`: label / value rows.
struct PropsCase;

impl Conformance for PropsCase {
    const NAME: &'static str = "props";
    const FAMILY: Family = Family::PROPS;
    const PARTS: &'static [Part] = Props::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        PROPS
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let rows: Vec<(&str, &str)> = f
            .rows
            .iter()
            .map(|r| (r.label.as_str(), r.meta.as_str()))
            .collect();
        Props::new(&rows).patch_part(patch_of(f)).draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: Props is a stateless label/value surface"
    }
}

const TEXT_AREA: Id = Id::root("conformance.text_area");
const TEXT_AREA_SETTLE: Id = Id::root("conformance.text_area.settle");
const TEXT_AREA_VALUE: &str = "The quick brown fox";

#[derive(Clone, PartialEq, Eq, Debug)]
struct TextAreaCaseState {
    st: TextAreaState,
    value: String,
}

impl Default for TextAreaCaseState {
    fn default() -> Self {
        TextAreaCaseState {
            st: TextAreaState::default(),
            value: TEXT_AREA_VALUE.to_owned(),
        }
    }
}

fn text_area(f: &Fixture) -> TextArea<'_> {
    let mut t = TextArea::new(TEXT_AREA, 4)
        .placeholder("Type here")
        .disabled(f.disabled)
        .read_only(f.read_only)
        .status(f.status())
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        t = t.state_override(f.forced());
    }
    t
}

/// `TextArea`: multiline editing, scrolling and the hardware cursor.
struct TextAreaCase;

impl Conformance for TextAreaCase {
    const NAME: &'static str = "text_area";
    const FAMILY: Family = Family::TEXTAREA;
    const PARTS: &'static [Part] = TextArea::PARTS;
    type State = TextAreaCaseState;
    type Action = TextAction;
    type Cmd = TextCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE
            | Caps::EDITS
            | Caps::CURSOR
            | Caps::TYPES
            | Caps::SCROLLS
            | Caps::DISABLEABLE
    }

    fn id() -> Id {
        TEXT_AREA
    }

    fn update(cx: &mut Cx<'_>, st: &mut TextAreaCaseState, f: &Fixture) -> Response<TextAction> {
        text_area(f).update(cx, &mut st.st, &mut st.value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &TextAreaCaseState, f: &Fixture) {
        if !area.is_empty() {
            // Keep the first harness focus stop outside the editor so `tab_to`
            // delivers FocusIn before the cursor assertion runs.
            let settle = Rect {
                width: 1.min(area.width),
                height: 1.min(area.height),
                ..area
            };
            ui.register_control(TEXT_AREA_SETTLE, settle, Focusability::Focusable);
        }
        text_area(f).value(&st.value).draw(ui, area, &st.st);
    }

    fn bindings(s: BindingState) -> &'static [Binding<TextCmd>] {
        TextArea::new(TEXT_AREA, 4).bindings(s)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
            StateFlags::DISABLED,
            StateFlags::ERROR,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED WARNING BUSY ACTIVE: TextArea has no selection, press, warning, readiness, or active-item affordance"
    }
}

const SELECT: Id = Id::root("conformance.select");

type FixtureSelect<'a> =
    Select<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn select(f: &Fixture) -> FixtureSelect<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;
    let mut s = Select::new(SELECT)
        .key(key)
        .row(row)
        .placeholder("Choose a person")
        .popup_rows(5)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        s = s.state_override(f.forced());
    }
    s
}

/// A select that already carries a committed value, so the closed field
/// paints its label through `RowUi` and the mono `(LABEL, …)` rules have a
/// text run to reach. `SelectState::default()` has no value and shows the
/// placeholder, which no mono state rule touches — the same reason
/// `TextInputCase` shows a fixture row instead of an empty value.
#[derive(Clone, Debug, PartialEq)]
struct ChosenSelect(SelectState);

impl Default for ChosenSelect {
    fn default() -> Self {
        let mut st = SelectState::default();
        st.set_value(Some(ItemKey::num(100)));
        ChosenSelect(st)
    }
}

/// `Select`: a one-row field that opens a keyed popover.
///
/// `OVERLAY` without `TRAPS_FOCUS` is §29.6 verbatim: the `LayerSpec::popover`
/// is a pointer barrier, the field keeps the one focus stop while the popup is
/// open, so case 14 checks the open/Esc/restore half and skips the trap half.
///
/// `COLLECTION` is **not** declared, and that is not a narrowing of a state
/// the component wears: case 12 addresses rows through
/// `PartRef::item(Part::ROW, k)`, and `Select` registers those regions only
/// inside the open popover layer. Nothing in the driver or in `SelectState`'s
/// public API can open that layer before the case's first click —
/// `SelectState::open` is private and `Select::open` runs only from a
/// delivered intent. `SELECTED` is kept in [`SelectCase::mono_states`]
/// regardless, so no mono state is lost by the omission.
struct SelectCase;

impl Conformance for SelectCase {
    const NAME: &'static str = "select";
    const FAMILY: Family = Family::SELECT;
    const PARTS: &'static [Part] = Select::<'static, FixtureRow>::PARTS;
    type State = ChosenSelect;
    type Action = SelectAction;
    type Cmd = SelectCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::DISABLEABLE | Caps::OVERLAY
    }

    fn id() -> Id {
        SELECT
    }

    fn update(cx: &mut Cx<'_>, st: &mut ChosenSelect, f: &Fixture) -> Response<SelectAction> {
        select(f).update(cx, &mut st.0, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &ChosenSelect, f: &Fixture) {
        select(f).draw(ui, area, &st.0, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::FIELD)
    }

    fn bindings(s: BindingState) -> &'static [Binding<SelectCmd>] {
        Select::<FixtureRow>::new(SELECT).bindings(s)
    }

    fn action_key_of(a: &SelectAction) -> Option<ItemKey> {
        match a {
            SelectAction::Chose(k) => Some(*k),
            SelectAction::Opened | SelectAction::Closed => None,
        }
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, k)
    }

    fn layer_id() -> Option<Id> {
        Some(SELECT)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 7] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::WARNING,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "EDITING BUSY ACTIVE: Select commits a value instead of editing text, \
         takes no readiness status, and reaches ACTIVE only through the same \
         (LABEL, BOLD) mono rule as PRESSED"
    }
}

const RADIO: Id = Id::root("conformance.radio_group");

type FixtureRadio<'a> =
    RadioGroup<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn radio(f: &Fixture) -> FixtureRadio<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;
    let mut r = RadioGroup::new(RADIO)
        .key(key)
        .row(row)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f));
    if f.forced().contains(StateFlags::SELECTED) {
        r = r.value(ItemKey::num(100));
    }
    if !f.forced().is_empty() {
        r = r.state_override(f.forced());
    }
    r
}

/// `RadioGroup`: keyed options with a cursor separate from the value.
struct RadioGroupCase;

impl Conformance for RadioGroupCase {
    const NAME: &'static str = "radio_group";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = FixtureRadio::<'static>::PARTS;
    type State = RadioGroupState;
    type Action = RadioGroupAction;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::DISABLEABLE
    }

    fn id() -> Id {
        RADIO
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut RadioGroupState,
        f: &Fixture,
    ) -> Response<RadioGroupAction> {
        radio(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState, f: &Fixture) {
        radio(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        RadioGroup::<FixtureRow>::new(RADIO).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(a: &RadioGroupAction) -> Option<ItemKey> {
        match a {
            RadioGroupAction::Chose(k) => Some(*k),
        }
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, k)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: RadioGroup has no validation, warning, edit, readiness, or independent active state"
    }
}

const CHECKBOX: Id = Id::root("conformance.checkbox");

fn checkbox(f: &Fixture) -> Checkbox<'_> {
    let mut c = Checkbox::new(CHECKBOX, "Accept terms")
        .checked(f.forced().contains(StateFlags::SELECTED))
        .read_only(f.read_only)
        .disabled(f.disabled)
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        c = c.state_override(f.forced());
    }
    c
}

/// `Checkbox`: a controlled boolean choice.
struct CheckboxCase;

impl Conformance for CheckboxCase {
    const NAME: &'static str = "checkbox";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = Checkbox::PARTS;
    type State = ();
    type Action = Activated;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::DISABLEABLE
    }

    fn id() -> Id {
        CHECKBOX
    }

    fn update(cx: &mut Cx<'_>, _st: &mut (), f: &Fixture) -> Response<Activated> {
        let mut value = false;
        checkbox(f).update(cx, &mut value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        checkbox(f).draw(ui, area);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        Checkbox::new(CHECKBOX, "").bindings(s)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::SELECTED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: Checkbox has no validation, warning, edit, readiness, or active-item state"
    }
}

const TOGGLE: Id = Id::root("conformance.toggle");

fn toggle(f: &Fixture) -> Toggle<'_> {
    let mut t = Toggle::new(TOGGLE, "Notifications")
        .on(f.forced().contains(StateFlags::SELECTED))
        .read_only(f.read_only)
        .disabled(f.disabled)
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        t = t.state_override(f.forced());
    }
    t
}

/// `Toggle`: a controlled switch.
struct ToggleCase;

impl Conformance for ToggleCase {
    const NAME: &'static str = "toggle";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = Toggle::PARTS;
    type State = ();
    type Action = Activated;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::DISABLEABLE
    }

    fn id() -> Id {
        TOGGLE
    }

    fn update(cx: &mut Cx<'_>, _st: &mut (), f: &Fixture) -> Response<Activated> {
        let mut value = false;
        toggle(f).update(cx, &mut value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        toggle(f).draw(ui, area);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        Toggle::new(TOGGLE, "").bindings(s)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::SELECTED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: Toggle has no validation, warning, edit, readiness, or active-item state"
    }
}

const CHIP_BAR: Id = Id::root("conformance.chip_bar");

type FixtureChips<'a> =
    ChipBar<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn chip_bar(f: &Fixture) -> FixtureChips<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;
    let mut c = ChipBar::new(CHIP_BAR)
        .key(key)
        .row(row)
        .select_mode(SelectMode::Single)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f));
    if !f.forced().is_empty() {
        c = c.state_override(f.forced());
    }
    c
}

/// `ChipBar`: a keyed, single-activation chip strip.
struct ChipBarCase;

impl Conformance for ChipBarCase {
    const NAME: &'static str = "chip_bar";
    const FAMILY: Family = Family::CHIP;
    const PARTS: &'static [Part] = FixtureChips::<'static>::PARTS;
    type State = tui_next::ChipBarState;
    type Action = ChipBarAction;
    type Cmd = ChipBarCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::DISABLEABLE
    }

    fn id() -> Id {
        CHIP_BAR
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut tui_next::ChipBarState,
        f: &Fixture,
    ) -> Response<ChipBarAction> {
        chip_bar(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &tui_next::ChipBarState, f: &Fixture) {
        chip_bar(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Enter)];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::LABEL, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChipBarCmd>] {
        ChipBar::<FixtureRow>::new(CHIP_BAR)
            .select_mode(SelectMode::Single)
            .bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(a: &ChipBarAction) -> Option<ItemKey> {
        match a {
            ChipBarAction::Toggled(k) | ChipBarAction::Closed(k) | ChipBarAction::Activated(k) => {
                Some(*k)
            }
        }
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::LABEL, k)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: ChipBar has no validation, warning, edit, readiness, or independent active state"
    }
}

const STATUS_BAR: Id = Id::root("conformance.status_bar");
const STATUS_LEFT: [StatusItem<'static>; 2] = [
    StatusItem::new("Workspace").strong(),
    StatusItem::new("main").key(ItemKey::num(1)),
];
const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Ready")];
const STATUS_RIGHT: [StatusItem<'static>; 1] = [StatusItem::new("0 changes").key(ItemKey::num(2))];

fn status_bar(f: &Fixture) -> StatusBar<'_> {
    let mut s = StatusBar::new(STATUS_BAR)
        .left(&STATUS_LEFT)
        .center(&STATUS_CENTER)
        .right(&STATUS_RIGHT)
        .status(f.status())
        .patch_part(patch_of(f));
    // `Overrides::flags` is `self.state.unwrap_or(live)`, so an unconditional
    // `.state_override(StateFlags::empty())` does not mean "force nothing" —
    // it means "force the empty state", erasing the flags the component
    // derived for itself. Every case guards the call for that reason.
    if !f.forced().is_empty() {
        s = s.state_override(f.forced());
    }
    s
}

/// `StatusBar`: stateless priority-ordered status items.
struct StatusBarCase;

impl Conformance for StatusBarCase {
    const NAME: &'static str = "status_bar";
    const FAMILY: Family = Family::STATUSBAR;
    const PARTS: &'static [Part] = StatusBar::PARTS;
    type State = ();
    type Action = StatusAction;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        STATUS_BAR
    }

    fn update(cx: &mut Cx<'_>, _st: &mut (), f: &Fixture) -> Response<StatusAction> {
        status_bar(f).update(cx)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        status_bar(f).draw(ui, area);
    }

    /// `PRESSED` is kept beyond what `Caps::REPORTS_STATUS` requires because
    /// the strip really can wear it: `LastFrame::state` sets `PRESSED` when
    /// `snapshot.pressed == Some(id)`, and `StatusBar` registers its
    /// clickable item regions under its **own** id.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 5] = [
            StateFlags::empty(),
            StateFlags::PRESSED,
            StateFlags::ERROR,
            StateFlags::WARNING,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED DISABLED EDITING ACTIVE: StatusBar registers item regions but no focus-ring \
         entry, and both FOCUSED and DISABLED are read off the ring, so neither can ever be set on it; \
         it declares no state of its own, and an item's weight is its `.emphasis` and `.tone(Role)` \
         rather than a StateFlags bit, so SELECTED, EDITING and ACTIVE name nothing a strip can be. \
         BUSY, ERROR and WARNING are kept: `StatusBar::readiness` paints a spinner frame into \
         Part::ICON, the error glyph into Part::MARKER and GlyphRole::Dirty into Part::MARKER, and \
         the leading glyph also shifts every item right, so the four renderings differ in more than \
         one cell."
    }
}

const HINT_BAR: Id = Id::root("conformance.hint_bar");

/// `HintBar`: derived key hints and a status message.
struct HintBarCase;

impl Conformance for HintBarCase {
    const NAME: &'static str = "hint_bar";
    const FAMILY: Family = Family::HINTBAR;
    const PARTS: &'static [Part] = HintBar::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        HINT_BAR
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let layer = HintLayer {
            hints: vec![
                Hint {
                    chord: Chord::key(KeyCode::Enter),
                    label: "Open",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::Esc),
                    label: "Close",
                    priority: 70,
                },
            ],
            badge: Some("F1"),
            status: Some(std::borrow::Cow::Borrowed("Ready")),
            centered: false,
        };
        let mut h = HintBar::new(HINT_BAR, &layer)
            .status(f.status())
            .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            h = h.state_override(f.forced());
        }
        h.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::ERROR,
            StateFlags::WARNING,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED EDITING ACTIVE: HintBar registers no ring entry and no \
         region and never calls `Ui::state`, so its live flags are `Status::flags()` alone and no \
         runtime state can reach it — nothing focuses, presses or disables a bar that owns no \
         control. A hint is a *label* for a chord another component declared, so SELECTED, EDITING \
         and ACTIVE have no referent here either. BUSY, ERROR and WARNING are kept: \
         `HintBar::status_glyph` leads the status message with a spinner frame, the error glyph or \
         GlyphRole::Dirty, and `status_width` counts that glyph, so it also changes how many hints \
         fit the row."
    }
}

const KEY_HINT: Id = Id::root("conformance.key_hint");

/// `KeyHint`: one fixed-width chord/action pair.
struct KeyHintCase;

impl Conformance for KeyHintCase {
    const NAME: &'static str = "key_hint";
    const FAMILY: Family = Family::KEYHINT;
    const PARTS: &'static [Part] = KeyHint::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        KEY_HINT
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut h =
            KeyHint::new(KEY_HINT, Chord::key(KeyCode::Enter), "Open").patch_part(patch_of(f));
        if !f.forced().is_empty() {
            h = h.state_override(f.forced());
        }
        h.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: KeyHint is a stateless key hint"
    }
}

const PROGRESS_BAR: Id = Id::root("conformance.progress_bar");

/// `ProgressBar`: determinate progress with readiness status.
struct ProgressBarCase;

impl Conformance for ProgressBarCase {
    const NAME: &'static str = "progress_bar";
    const FAMILY: Family = Family::PROGRESS;
    const PARTS: &'static [Part] = ProgressBar::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        PROGRESS_BAR
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut p = ProgressBar::new(PROGRESS_BAR)
            .label("Uploading")
            .ratio(0.65)
            .status(f.status())
            .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            p = p.state_override(f.forced());
        }
        p.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 3] = [StateFlags::empty(), StateFlags::ERROR, StateFlags::BUSY];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED WARNING EDITING ACTIVE: ProgressBar registers no ring \
         entry and no region, so it never wears a runtime state; it declares no MARKER, so §11.4's \
         WARNING rule reaches nothing. BUSY and ERROR are kept: derived from .status(Status) and \
         painted into Part::ICON."
    }
}

const SPINNER: Id = Id::root("conformance.spinner");

/// `Spinner`: deterministic frame-driven progress indication.
struct SpinnerCase;

impl Conformance for SpinnerCase {
    const NAME: &'static str = "spinner";
    const FAMILY: Family = Family::PROGRESS;
    const PARTS: &'static [Part] = Spinner::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        SPINNER
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut s = Spinner::new(SPINNER)
            .label("Working")
            .frame(1)
            .patch_part(patch_of(f));
        // Unguarded, this overwrote `Spinner::draw`'s own
        // `ov.flags(StateFlags::BUSY)` with the empty state, so the case
        // never once rendered a spinner in the state a spinner is.
        if !f.forced().is_empty() {
            s = s.state_override(f.forced());
        }
        s.draw(ui, area);
    }

    /// Deliberately **not** `Caps::REPORTS_STATUS`: that capability is the
    /// obligation to paint §11.4's readiness affordance, and a `Spinner` does
    /// not paint one — it *is* one.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: Spinner *is* §11.4's \
         BUSY affordance rather than a component that enters BUSY. `draw` paints a \
         `design.motion.spinner_frames` frame unconditionally under `ov.flags(StateFlags::BUSY)`, \
         and which frame it shows is the `.frame(usize)` prop, not a state — so there is no \
         not-spinning rendering for a second state to be distinguishable from. Nothing can give it \
         one either: `Spinner` takes no `.status`, registers no ring entry and no region, and its \
         only parts are ICON and LABEL."
    }
}

const METER: Id = Id::root("conformance.meter");

/// `Meter`: a ratio-driven compact meter.
struct MeterCase;

impl Conformance for MeterCase {
    const NAME: &'static str = "meter";
    const FAMILY: Family = Family::METER;
    const PARTS: &'static [Part] = Meter::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        METER
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut m = Meter::new(METER)
            .ratio(0.65)
            .value("65%")
            .status(f.status())
            .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            m = m.state_override(f.forced());
        }
        m.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 3] = [StateFlags::empty(), StateFlags::ERROR, StateFlags::BUSY];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED WARNING EDITING ACTIVE: Meter's live flags are \
         `Status::flags()` alone — it never calls `Ui::state` — and it registers no ring entry and \
         no region, so no runtime state reaches it; its parts are TRACK, THUMB, LABEL and ICON, with \
         no MARKER for §11.4's WARNING rule to reach. BUSY and ERROR are kept: `Meter::icon` returns \
         a `design.motion.spinner_frames` frame while busy and GlyphRole::Error while errored, and \
         nothing at all at baseline."
    }
}

const EMPTY: Id = Id::root("conformance.empty");

fn empty_state(f: &Fixture) -> EmptyState<'static> {
    match f.status() {
        Status::Ready => EmptyState::Empty {
            title: "No results",
            hint: Some("Try a different filter"),
        },
        Status::Busy | Status::Loading => EmptyState::Loading { label: "Loading" },
        Status::Error => EmptyState::Error {
            message: "Unable to load",
            detail: Some("Try again"),
        },
        _ => EmptyState::Empty {
            title: "No results",
            hint: None,
        },
    }
}

/// `Empty`: the shared empty/loading/error surface.
struct EmptyCase;

impl Conformance for EmptyCase {
    const NAME: &'static str = "empty";
    const FAMILY: Family = Family::EMPTY;
    const PARTS: &'static [Part] = Empty::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        EMPTY
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut e = Empty::new(EMPTY, empty_state(f)).patch_part(patch_of(f));
        if !f.forced().is_empty() {
            e = e.state_override(f.forced());
        }
        e.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 3] = [StateFlags::empty(), StateFlags::ERROR, StateFlags::BUSY];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED WARNING EDITING ACTIVE: Empty registers no ring entry and \
         no region and reads no runtime state, so nothing can focus, press, disable or edit it, and \
         it has no rows to select and no strip to be ACTIVE in. WARNING has no referent either: the \
         whole input is the `EmptyState` variant, whose arms are Empty, Loading, Partial and Error \
         and never a warning. BUSY and ERROR are kept, and they are the largest difference in the \
         suite: `EmptyState::draw` leads Loading/Partial with a spinner frame and Error with \
         GlyphRole::Error, the title text changes with the arm (`No results` / `Loading` / `Unable \
         to load`), and Loading has no detail line, so the block goes from three rows to one."
    }
}

const BRAND: Id = Id::root("conformance.brand");

/// `Brand`: a non-interactive application lockup.
struct BrandCase;

impl Conformance for BrandCase {
    const NAME: &'static str = "brand";
    const FAMILY: Family = Family::BRAND;
    const PARTS: &'static [Part] = Brand::PARTS;
    type State = ();
    type Action = Activated;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        BRAND
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<Activated> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut b = Brand::new(BRAND, "Junie")
            .tagline("Terminal tools")
            .patch_part(patch_of(f));
        if !f.forced().is_empty() {
            b = b.state_override(f.forced());
        }
        b.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: Brand is a stateless brand surface"
    }
}

conformance_suite!(
    probe => ProbeCase,
    button => ButtonCase,
    text_input => TextInputCase,
    field => FieldCase,
    list => ListCase,
    tabs => TabsCase,
    dialog => DialogCase,
    scroll_region => ScrollRegionCase,
    props => PropsCase,
    text_area => TextAreaCase,
    select => SelectCase,
    radio_group => RadioGroupCase,
    checkbox => CheckboxCase,
    toggle => ToggleCase,
    chip_bar => ChipBarCase,
    status_bar => StatusBarCase,
    hint_bar => HintBarCase,
    key_hint => KeyHintCase,
    progress_bar => ProgressBarCase,
    spinner => SpinnerCase,
    meter => MeterCase,
    empty => EmptyCase,
    brand => BrandCase,
);

/// §16.2 suite-level (MA-8): the states a component's capabilities imply are
/// a **union**, not a first match. The `if / else if` chain this replaced is
/// what let a case declaring `EDITS | DISABLEABLE` keep only `EDITING` and
/// narrow `DISABLED` away while the guard stayed green.
#[test]
fn mono_states_required_by_is_a_union() {
    let both = mono_states_required_by(Caps::EDITS | Caps::DISABLEABLE);
    assert!(
        both.contains(&StateFlags::EDITING) && both.contains(&StateFlags::DISABLED),
        "EDITS | DISABLEABLE must require both: {both:?}"
    );
    // every capability contributes, and the default state is always required
    let all = mono_states_required_by(
        Caps::FOCUSABLE | Caps::ACTIVATES | Caps::DISABLEABLE | Caps::EDITS | Caps::COLLECTION,
    );
    for s in [
        StateFlags::empty(),
        StateFlags::FOCUSED,
        StateFlags::PRESSED,
        StateFlags::DISABLED,
        StateFlags::EDITING,
        StateFlags::SELECTED,
    ] {
        assert!(all.contains(&s), "{s:?} missing from {all:?}");
    }
    // a component with no capabilities owes only the default state
    assert_eq!(
        mono_states_required_by(Caps::empty()),
        vec![StateFlags::empty()]
    );
}

/// Every binding table `C` publishes over `states` is conflict-free.
fn clean<C: Conformance>(states: &[StateFlags]) {
    for f in states {
        let st = BindingState { flags: *f };
        let d = binding_conflicts(C::id(), KeyPhase::Bubble, C::bindings(st));
        assert!(d.is_empty(), "{}: {d:?}", C::NAME);
    }
}

/// One [`clean`] call per registered case: `clean` is generic, so the roster
/// cannot be a slice. It lives beside the test rather than inside it so that
/// registering the next component does not push the test over
/// `clippy::too_many_lines`.
fn every_registered_table_is_clean() {
    let states = [
        StateFlags::empty(),
        StateFlags::FOCUSED,
        StateFlags::EDITING,
        StateFlags::DISABLED,
    ];
    clean::<ProbeCase>(&states);
    clean::<ButtonCase>(&states);
    clean::<TextInputCase>(&states);
    clean::<FieldCase>(&states);
    clean::<ListCase>(&states);
    clean::<TabsCase>(&states);
    clean::<DialogCase>(&states);
    clean::<ScrollRegionCase>(&states);
    clean::<PropsCase>(&states);
    clean::<TextAreaCase>(&states);
    clean::<SelectCase>(&states);
    clean::<RadioGroupCase>(&states);
    clean::<CheckboxCase>(&states);
    clean::<ToggleCase>(&states);
    clean::<ChipBarCase>(&states);
    clean::<StatusBarCase>(&states);
    clean::<HintBarCase>(&states);
    clean::<KeyHintCase>(&states);
    clean::<ProgressBarCase>(&states);
    clean::<SpinnerCase>(&states);
    clean::<MeterCase>(&states);
    clean::<EmptyCase>(&states);
    clean::<BrandCase>(&states);
}

/// §16.2 suite-level: two **visible** bindings on the same chord in one
/// phase are a `Diagnostic::BindingConflict`. This is the check that makes
/// the historically dead grid `Ctrl+D` detectable, so it is asserted twice:
/// the detector fires on a table built to conflict, and every table every
/// registered component publishes is clean under it.
#[test]
fn conflicting_visible_bindings_are_reported() {
    const OWNER: Id = Id::root("conformance.bindings");
    const ALIAS: [Binding<ProbeCmd>; 2] = [
        Binding {
            chord: Chord::key(KeyCode::Char('d')),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 60,
            visible: true,
        },
        Binding {
            chord: Chord::key(KeyCode::Char('d')),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 10,
            visible: false,
        },
    ];
    const CLASH: [Binding<ProbeCmd>; 2] = [
        Binding {
            chord: Chord::key(KeyCode::Char('d')),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 60,
            visible: true,
        },
        Binding {
            chord: Chord::key(KeyCode::Char('d')),
            cmd: ProbeCmd::Activate,
            label: "Duplicate",
            priority: 60,
            visible: true,
        },
    ];
    let found = binding_conflicts(OWNER, KeyPhase::Bubble, &CLASH);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        matches!(
            found.first(),
            Some(Diagnostic::BindingConflict {
                chord,
                phase: KeyPhase::Bubble,
                a,
                b,
            }) if *chord == Chord::key(KeyCode::Char('d')) && *a == OWNER && *b == OWNER
        ),
        "{found:?}"
    );

    // a hidden alias on the same chord is not a conflict: every component
    // ships one (Space beside Enter, `j` beside Down)
    assert!(binding_conflicts(OWNER, KeyPhase::Bubble, &ALIAS).is_empty());

    // and no registered component publishes a conflicting table
    every_registered_table_is_clean();
    // every selection mode and every strip configuration, not just the
    // fixture's own
    for m in [
        SelectMode::Single,
        SelectMode::Multi,
        SelectMode::Range,
        SelectMode::None,
    ] {
        let t = List::<FixtureRow>::new(LIST).select_mode(m);
        assert!(
            binding_conflicts(LIST, KeyPhase::Bubble, t.bindings(BindingState::default()))
                .is_empty()
        );
    }
    for (closable, allow_new) in [(false, false), (true, false), (false, true), (true, true)] {
        let t = Tabs::<FixtureRow>::new(TABS)
            .closable(closable)
            .allow_new(allow_new);
        assert!(
            binding_conflicts(TABS, KeyPhase::Bubble, t.bindings(BindingState::default()))
                .is_empty()
        );
    }
}

/// §16.2 suite-level: a component that cannot draw registers nothing — the
/// `0×0` case across the whole registry, extended by §26 N1 to a
/// `LayerSize::Fixed(0, h)` request, which is an **empty layer** and never
/// the screen.
#[test]
fn draw_registers_nothing_when_it_cannot_draw() {
    fn degenerate<C: Conformance>() {
        for area in [
            Rect::new(4, 4, 0, 0),
            Rect::new(4, 4, 0, 6),
            Rect::new(4, 4, 20, 0),
        ] {
            let mut f = Fixture::default();
            f.area = area;
            let mut scene = Scene::new(C::NAME, f.theme.clone(), f.color, 30, 12);
            let st = C::State::default();
            scene.draw(|ui, _| C::draw(ui, area, &st, &f));
            let regions = scene.registry().map_or(0, |r| r.regions().len());
            assert_eq!(regions, 0, "{}: {area:?} registered {regions}", C::NAME);
            let ring = scene.ring().map_or(0, |r| r.reachable().count());
            assert_eq!(ring, 0, "{}: {area:?} left {ring} ring entries", C::NAME);
        }
    }
    degenerate::<ProbeCase>();
    degenerate::<ButtonCase>();
    degenerate::<TextInputCase>();
    degenerate::<FieldCase>();
    degenerate::<ListCase>();
    degenerate::<TabsCase>();
    degenerate::<DialogCase>();
    degenerate::<ScrollRegionCase>();
    degenerate::<PropsCase>();
    degenerate::<TextAreaCase>();
    degenerate::<SelectCase>();
    degenerate::<RadioGroupCase>();
    degenerate::<CheckboxCase>();
    degenerate::<ToggleCase>();
    degenerate::<ChipBarCase>();
    degenerate::<StatusBarCase>();
    degenerate::<HintBarCase>();
    degenerate::<KeyHintCase>();
    degenerate::<ProgressBarCase>();
    degenerate::<SpinnerCase>();
    degenerate::<MeterCase>();
    degenerate::<EmptyCase>();
    degenerate::<BrandCase>();

    // §26 N1: a zero-size request resolves to `Rect::ZERO`, so the layer's
    // content is clipped away and registers nothing either.
    let screen = Rect::new(0, 0, 40, 12);
    assert_eq!(
        resolve_anchor(
            screen,
            Anchor::Screen(ScreenAlign::Center),
            LayerSize::Fixed(0, 8)
        ),
        Rect::ZERO
    );
    assert_eq!(
        resolve_anchor(screen, Anchor::Screen(ScreenAlign::Center), LayerSize::Fill),
        screen
    );
    let mut h = Harness::new(ZeroLayer, Theme::junie(), 40, 12).with_auto_draw(true);
    let _ = h.tick();
    assert!(h.is_open(ZERO), "the layer is open, it is merely empty");
    assert_eq!(
        h.runtime().region_count(),
        0,
        "a zero-size layer's content registers nothing"
    );
    assert_eq!(h.focus(), None);
}

const ZERO: Id = Id::root("conformance.zero_layer");

/// An app whose only layer asks for `LayerSize::Fixed(0, h)`.
struct ZeroLayer;

impl tui_next::App for ZeroLayer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !cx.is_open(ZERO) {
            cx.open_layer(ZERO, LayerSpec::modal(ZERO).size(LayerSize::Fixed(0, 8)));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ZERO, |ui, a| {
            assert!(a.is_empty(), "a zero-size layer resolves to an empty rect");
            Button::new(BTN, "Never").draw(ui, a);
        });
    }
}

mod registry {
    use super::*;

    /// The parts a case resolves in one draw.
    fn styled<C: Conformance>() -> Vec<Part> {
        let f = Fixture::default();
        let mut scene = Scene::new(C::NAME, f.theme.clone(), f.color, 40, 12);
        let st = C::State::default();
        let mut out = Vec::new();
        scene.draw(|ui, _| {
            C::draw(ui, f.area, &st, &f);
            out = ui
                .styled_parts()
                .iter()
                .filter(|(o, _)| *o == C::id())
                .map(|(_, p)| *p)
                .collect();
        });
        out.sort();
        out.dedup();
        out
    }

    fn check<C: Conformance>(extra: &[Part]) {
        for p in styled::<C>() {
            assert!(
                C::PARTS.contains(&p) || extra.contains(&p),
                "{}: styled {p:?} which is not in PARTS {:?}",
                C::NAME,
                C::PARTS
            );
        }
    }

    #[test]
    fn every_public_component_is_registered() {
        assert_eq!(
            registered_cases(),
            vec![
                "probe",
                "button",
                "text_input",
                "field",
                "list",
                "tabs",
                "dialog",
                "scroll_region",
                "props",
                "text_area",
                "select",
                "radio_group",
                "checkbox",
                "toggle",
                "chip_bar",
                "status_bar",
                "hint_bar",
                "key_hint",
                "progress_bar",
                "spinner",
                "meter",
                "empty",
                "brand",
            ]
        );
    }

    #[test]
    fn declared_parts_are_the_parts_actually_styled() {
        check::<ButtonCase>(&[]);
        check::<TextInputCase>(&[]);
        // the chrome and its control register under one id
        check::<FieldCase>(TextInput::PARTS);
        check::<ListCase>(&[]);
        check::<TabsCase>(&[]);
        check::<ScrollRegionCase>(&[]);
        check::<TextAreaCase>(&[]);
        check::<SelectCase>(&[]);
        check::<RadioGroupCase>(&[]);
        check::<CheckboxCase>(&[]);
        check::<ToggleCase>(&[]);
        check::<ChipBarCase>(&[]);
        check::<StatusBarCase>(&[]);
        check::<HintBarCase>(&[]);
        check::<KeyHintCase>(&[]);
        check::<ProgressBarCase>(&[]);
        check::<SpinnerCase>(&[]);
        check::<MeterCase>(&[]);
        check::<EmptyCase>(&[]);
        check::<BrandCase>(&[]);
    }
}
