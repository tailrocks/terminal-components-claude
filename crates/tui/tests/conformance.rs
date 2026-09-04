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
    ActionKey, Anchor, Button, ButtonCmd, Diagnostic, Dialog, DialogAction, Field, KeyPhase,
    LayerSize, LayerSpec, List, ListAction, ListCmd, Props, RowUi, ScreenAlign, ScrollRegion,
    SelectMode, Tabs, TabsAction, TabsCmd, TextCmd, TextInput, TextInputState, Theme,
    binding_conflicts, resolve_anchor,
};
use tui_next_testing::conformance::{Caps, Conformance, Fixture, FixtureRow};
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
        let mut live = if f.state_override.is_empty() {
            ui.state(PROBE)
        } else {
            f.state_override
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
        if let Some(g) = gutter.glyph {
            ui.glyph(gutter_cell, g, gutter.style);
        }
        let label = style_for(ui, Part::LABEL);
        let mut text = Rect {
            x: area.x.saturating_add(1),
            width: area.width.saturating_sub(1),
            ..area
        };
        text.height = 1.min(area.height);
        if label.glyph == Some(GlyphRole::PressLeft) {
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
}

// ───────────────────────────── Slice 4 cases ─────────────────────────────

fn patch_of(f: &Fixture) -> &[(Part, StylePatch)] {
    f.patch.as_ref().map_or(&[], core::slice::from_ref)
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
        Button::new(BTN, "Probe").disabled(f.disabled).update(cx)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let mut b = Button::new(BTN, "Probe")
            .disabled(f.disabled)
            .patch_part(patch_of(f));
        if !f.state_override.is_empty() {
            b = b.state_override(f.state_override);
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

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
        ];
        &STATES
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
        .patch_part(patch_of(f));
    if f.secret.is_some() {
        t = t.secret(tui_next::SecretPolicy::default());
    }
    if !f.state_override.is_empty() {
        t = t.state_override(f.state_override);
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
    type Action = tui_next::TextAction;
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

    fn update(cx: &mut Cx<'_>, st: &mut InputState, f: &Fixture) -> Response<tui_next::TextAction> {
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

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
            StateFlags::ERROR,
        ];
        &STATES
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
    type Action = tui_next::TextAction;
    type Cmd = TextCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::EDITS | Caps::CURSOR | Caps::TYPES | Caps::DISABLEABLE
    }

    fn id() -> Id {
        FIELD_INPUT
    }

    fn update(cx: &mut Cx<'_>, st: &mut InputState, f: &Fixture) -> Response<tui_next::TextAction> {
        text_input(FIELD_INPUT, f).update(cx, &mut st.st, &mut st.value)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &InputState, f: &Fixture) {
        let error = f.state_override.contains(StateFlags::ERROR);
        let mut field = Field::new(
            "Label",
            text_input(FIELD_INPUT, f).value(shown_value(st, f)),
        )
        .required(true)
        .help("Help text")
        .error(error.then_some("Something is wrong"))
        .patch_part(patch_of(f));
        if !f.state_override.is_empty() {
            field = field.state_override(f.state_override);
        }
        field.draw(ui, area, &st.st);
    }

    fn bindings(s: BindingState) -> &'static [Binding<TextCmd>] {
        TextInput::new(FIELD_INPUT).bindings(s)
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
}

const LIST: Id = Id::root("conformance.list");

fn row_key(r: &FixtureRow) -> ItemKey {
    r.key
}

fn row_paint(r: &FixtureRow, u: &mut RowUi<'_>) {
    u.label(&r.label);
    u.meta(&r.meta);
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
        .patch_part(patch_of(f));
    if !f.state_override.is_empty() {
        l = l.state_override(f.state_override);
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

    /// A list never edits and is never the `ACTIVE` element of a strip;
    /// `BUSY`/`LOADING` are the two readiness states §11.4 gives no mono
    /// rule, so they are narrowed out rather than asserted (MA-8).
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
        .patch_part(patch_of(f));
    if !f.state_override.is_empty() {
        t = t.state_override(f.state_override);
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

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::DISABLED,
        ];
        &STATES
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
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::OVERLAY
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
        if cx.is_open(DLG) {
            let d = dialog(f).update(cx, st);
            if let Some(DialogAction::Action(_)) = d.action_ref() {
                cx.close_layer(DLG, Some(ActionKey::CONFIRM));
            }
            return d;
        }
        r |= Response::ignored();
        r.map_action(|()| DialogAction::Action(K_OPEN))
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &tui_next::DialogState, f: &Fixture) {
        let mut b = Button::new(LAUNCH, "Open");
        if !f.state_override.is_empty() {
            b = b.state_override(f.state_override);
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
        if !f.state_override.is_empty() {
            sr = sr.state_override(f.state_override);
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

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
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
);

/// §16.2 suite-level: two **visible** bindings on the same chord in one
/// phase are a `Diagnostic::BindingConflict`. This is the check that makes
/// the historically dead grid `Ctrl+D` detectable, so it is asserted twice:
/// the detector fires on a table built to conflict, and every table every
/// registered component publishes is clean under it.
#[test]
fn conflicting_visible_bindings_are_reported() {
    const OWNER: Id = Id::root("conformance.bindings");
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
    assert!(binding_conflicts(OWNER, KeyPhase::Bubble, &ALIAS).is_empty());

    // and no registered component publishes a conflicting table
    fn clean<C: Conformance>(states: &[StateFlags]) {
        for f in states {
            let st = BindingState { flags: *f };
            let d = binding_conflicts(C::id(), KeyPhase::Bubble, C::bindings(st));
            assert!(d.is_empty(), "{}: {d:?}", C::NAME);
        }
    }
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
    }
}
