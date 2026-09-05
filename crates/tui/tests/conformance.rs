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

use junie_tui::author::{
    Activated, Binding, BindingState, Bindings, Chord, Cx, Family, FgStep, Focusability, FrameRead,
    GlyphRole, Id, Intent, ItemKey, KeyCode, Part, PartRef, PartStyle, Phase, Position, Rect,
    Response, ScrollState, StateFlags, StylePatch, Ui, Variant,
};
use junie_tui::{
    Action, ActionKey, Anchor, Brand, Button, ButtonCmd, CellDecor, CellPos, CellRef, Checkbox,
    ChipBar, ChipBarAction, ChipBarCmd, ChoiceCmd, CodeAction, CodeCmd, CodeEditor,
    CodeEditorState, Column, ColumnKey, Completion, CompletionAction, CompletionCmd,
    CompletionController, CompletionState, ContextMenu, Diagnostic, Dialog, DialogAction,
    DiffLineKind, DiffRow, DiffSource, DiffView, DiffViewState, EditIntent, Empty, EmptyState,
    Field, FieldError, FieldKind, FieldMut, FieldRef, FieldSpec, FilterList, FilterListAction,
    FilterListCmd, FilterListState, Form, FormAction, FormData, FormState, Grid, GridAction,
    GridCmd, GridEditor, GridModel, GridState, HelpAction, HelpCmd, HelpOverlay, HelpOverlayState,
    HelpSection, Hint, HintBar, HintLayer, Item, KeyHint, KeyPhase, LayerSize, LayerSpec, List,
    ListAction, ListCmd, Menu, MenuAction, MenuBar, MenuCmd, MenuItem, MenuState, Meter, NavList,
    NavListAction, NavListCmd, NavListState, NavUnit, Panel, Picker, PickerAction, PickerChain,
    PickerChainAction, PickerChainCmd, PickerChainState, PickerStage, PickerState, ProgressBar,
    Props, PropsAction, PropsCmd, PropsList, PropsRow, PropsState, RadioGroup, RadioGroupAction,
    RadioGroupState, Role, RowUi, ScreenAlign, ScrollRegion, Secret, SecretPolicy, Select,
    SelectAction, SelectCmd, SelectMode, SelectState, Slot, Span, Spinner, SplitAction, SplitAxis,
    SplitCmd, SplitPane, SplitPaneState, Status, StatusAction, StatusBar, StatusItem, StepState,
    Steps, StepsAction, StepsCmd, StepsState, Tabs, TabsAction, TabsCmd, TextAction, TextArea,
    TextAreaState, TextCmd, TextInput, TextInputState, TextViewport, Theme, Toggle, TooSmall, Tree,
    TreeAction, TreeCmd, TreeNode, TreeState, ViewportAction, ViewportCmd, ViewportLine,
    ViewportState, Wizard, WizardAction, WizardCmd, WizardState, WizardStep, binding_conflicts,
    resolve_anchor,
};
use junie_tui_testing::conformance::{
    Caps, Conformance, Fixture, FixtureRow, PointerGesture, mono_states_required_by,
};
use junie_tui_testing::{Harness, Scene, conformance_suite};

const PROBE: Id = Id::root("conformance.probe");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCmd {
    Activate,
}

const BINDINGS: &[Binding<ProbeCmd>] = &[
    Binding {
        action: ActionKey::custom("probe.activate.enter"),
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: ProbeCmd::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        action: ActionKey::custom("probe.activate.space"),
        chord: Some(Chord::key(KeyCode::Char(' '))),
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
/// while focused; honours `disabled` and `patch`.
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
                Intent::Binding(action) if !f.disabled => {
                    if Binding::command(BINDINGS, action).is_some() {
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
        let mut live = ui.state(PROBE);
        if f.disabled {
            live |= StateFlags::DISABLED;
        }
        ui.publish_bindings(PROBE, live, BINDINGS);
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
/// `patch_part` and caller-owned semantic props.
struct ButtonCase;

impl Conformance for ButtonCase {
    const NAME: &'static str = "button";
    const FAMILY: Family = Family::BUTTON;
    const PARTS: &'static [Part] = Button::PARTS;
    type State = ();
    type Action = Activated;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE | Caps::REPORTS_STATUS | Caps::SELECTS
    }

    fn id() -> Id {
        BTN
    }

    fn update(cx: &mut Cx<'_>, _st: &mut (), f: &Fixture) -> Response<Activated> {
        let button = Button::new(BTN, "Probe")
            .disabled(f.disabled)
            .status(f.status())
            .checked(f.selected);
        button.update(cx)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let b = Button::new(BTN, "Probe")
            .disabled(f.disabled)
            .status(f.status())
            .checked(f.selected)
            .patch_part(patch_of(f));
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
        const STATES: [StateFlags; 7] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "WARNING EDITING ACTIVE: Button has no warning, edit, or active-item state"
    }

    fn mono_fixture(state: StateFlags) -> Fixture {
        let mut fixture = Fixture::default();
        fixture.selected = state.contains(StateFlags::SELECTED);
        fixture
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
        t = t.secret(SecretPolicy::default());
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
            | Caps::REPORTS_STATUS
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

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const EDIT: &[Chord] = &[Chord::key(KeyCode::Char('x'))];
        if state.contains(StateFlags::EDITING) {
            EDIT
        } else {
            &[]
        }
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
        let error = f.status() == Status::Error;
        let field = Field::new(
            "Label",
            text_input(FIELD_INPUT, f).value(shown_value(st, f)),
        )
        .required(true)
        .help("Help text")
        .error(error.then_some("Something is wrong"))
        .patch_part(patch_of(f));
        field.draw(ui, area, &st.st);
    }

    fn bindings(s: BindingState) -> &'static [Binding<TextCmd>] {
        TextInput::new(FIELD_INPUT).bindings(s)
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const EDIT: &[Chord] = &[Chord::key(KeyCode::Char('x'))];
        if state.contains(StateFlags::EDITING) {
            EDIT
        } else {
            &[]
        }
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

fn warning_row_paint(r: &FixtureRow, u: &mut RowUi<'_>) {
    u.marker(GlyphRole::Dirty);
    u.label_spans(&[Span::new(&r.label).role(Role::Warning)]);
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
    let row: fn(&FixtureRow, &mut RowUi<'_>) = if f.decor_flags.contains(StateFlags::WARNING) {
        warning_row_paint
    } else {
        row_paint
    };
    let disabled: &dyn Fn(&FixtureRow) -> bool = &row_disabled;

    List::new(LIST)
        .key(key)
        .row(row)
        .disabled_item(disabled)
        .status(f.status())
        .patch_part(patch_of(f))
}

/// `List`: keyed rows, cursor, choose / activate, wheel and the bar.
struct ListCase;

impl Conformance for ListCase {
    const NAME: &'static str = "list";
    const FAMILY: Family = Family::LIST;
    const PARTS: &'static [Part] = FixtureList::PARTS;
    type State = junie_tui::ListState;
    type Action = ListAction;
    type Cmd = ListCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::FOCUSABLE
            | Caps::COLLECTION
            | Caps::SELECTS
            | Caps::SCROLLS
            | Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        LIST
    }

    fn update(cx: &mut Cx<'_>, st: &mut junie_tui::ListState, f: &Fixture) -> Response<ListAction> {
        list(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &junie_tui::ListState, f: &Fixture) {
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
    /// only `EDITING` and `ACTIVE` are narrowed. Readiness is root-owned in
    /// the conditional left rail.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 8] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::WARNING,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Char(' '))];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
    }
    fn mono_narrowing_reason() -> &'static str {
        "EDITING ACTIVE: List has no edit affordance, and active is represented by the selected row"
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

    Tabs::new(TABS)
        .key(key)
        .row(row)
        .closable(true)
        .status(f.status())
        .patch_part(patch_of(f))
}

/// `Tabs`: stable keys, the active tab and the strip window.
struct TabsCase;

impl Conformance for TabsCase {
    const NAME: &'static str = "tabs";
    const FAMILY: Family = Family::TABS;
    const PARTS: &'static [Part] = FixtureTabs::PARTS;
    type State = junie_tui::TabsState;
    type Action = TabsAction;
    type Cmd = TabsCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::SELECTS | Caps::REPORTS_STATUS
    }

    fn id() -> Id {
        TABS
    }

    fn update(cx: &mut Cx<'_>, st: &mut junie_tui::TabsState, f: &Fixture) -> Response<TabsAction> {
        tabs(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &junie_tui::TabsState, f: &Fixture) {
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

    /// `SELECTED` is established through the real Enter activation path;
    /// forcing it cannot synthesize `TabsState::active`. `ACTIVE` remains
    /// narrowed because it is the internal style state of that same semantic
    /// selection, not an independent public state.
    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 6] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::ERROR,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        // The default update seeds the first tab as active. Move through the
        // real next-tab activation path so SELECTED changes the rendered
        // active rule instead of reproducing the empty state.
        const ACTIVATE: &[Chord] = &[Chord::key(KeyCode::Right)];
        if state.contains(StateFlags::SELECTED) {
            ACTIVATE
        } else {
            &[]
        }
    }
    fn mono_narrowing_reason() -> &'static str {
        "DISABLED WARNING EDITING ACTIVE: Tabs has no disabled prop, warning, edit, or independent active-state affordance"
    }
}

const LAUNCH: Id = Id::root("conformance.dialog.launch");
const DLG: Id = Id::root("conformance.dialog");
const K_OPEN: ActionKey = ActionKey::custom("open");
const DIALOG_ACTIONS: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel").chord(Chord::key(KeyCode::F(6))),
    Action::new(ActionKey::CONFIRM, "OK"),
];

fn dialog(f: &Fixture) -> Dialog<'_> {
    Dialog::confirm(DLG, "Confirm", "Proceed with the operation?")
        .actions(DIALOG_ACTIONS)
        .patch_part(patch_of(f))
}

/// A real dialog action is the tested control; a separate launcher opens its
/// modal layer for the overlay contract.
struct DialogCase;

impl Conformance for DialogCase {
    const NAME: &'static str = "dialog";
    const FAMILY: Family = Family::DIALOG;
    const PARTS: &'static [Part] = Dialog::PARTS;
    type State = junie_tui::DialogState;
    type Action = DialogAction;
    type Cmd = ButtonCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::OVERLAY | Caps::TRAPS_FOCUS
    }

    fn id() -> Id {
        DLG
    }

    fn control_id() -> Id {
        DLG.part(Part::ACTIONS).index(0)
    }

    fn opener_id() -> Id {
        LAUNCH
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut junie_tui::DialogState,
        f: &Fixture,
    ) -> Response<DialogAction> {
        let launch = Button::new(LAUNCH, "Open").update(cx);
        let open = launch.activated()
            || cx.intents(LAUNCH).any(
                |intent| matches!(intent, Intent::Key(key) if Chord::key(KeyCode::F(4)).matches(&key)),
            );
        if open {
            cx.open_layer(DLG, dialog(f).layer(cx));
        }
        let mut r = launch.erase();
        let d = dialog(f).update(cx, st);
        if cx.is_open(DLG) {
            if let Some(DialogAction::Action(_)) = d.action_ref() {
                cx.close_layer(DLG, Some(ActionKey::CONFIRM));
            }
            return if open {
                Response::action(DialogAction::Action(K_OPEN)).for_id(DLG)
            } else {
                d
            };
        }
        if d.action_ref().is_some() {
            return d;
        }
        r |= d.erase();
        if open {
            Response::action(DialogAction::Action(K_OPEN)).for_id(DLG)
        } else {
            r |= Response::ignored();
            r.map_action(|()| DialogAction::Action(K_OPEN))
        }
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &junie_tui::DialogState, f: &Fixture) {
        let b = Button::new(LAUNCH, "Open");
        let used = b.draw(ui, area);
        let below = Rect {
            y: area.y.saturating_add(used.height),
            height: area.height.saturating_sub(used.height),
            ..area
        };
        let layer_drawn = ui
            .layer(DLG, |ui, layer_area| {
                dialog(f).draw(ui, layer_area, st, |_, _| {});
            })
            .is_some();
        if !layer_drawn {
            dialog(f).draw(ui, below, st, |_, _| {});
        }
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ButtonCmd>] {
        Button::new(LAUNCH, "").bindings(s)
    }

    fn dynamic_bindings(_fixture: &Fixture) -> Vec<(ActionKey, Chord)> {
        vec![(ActionKey::CANCEL, Chord::key(KeyCode::F(6)))]
    }

    fn dynamic_binding_id(_action: ActionKey) -> Id {
        DLG.part(Part::ACTIONS).index(0)
    }

    fn open_chord() -> Option<Chord> {
        Some(Chord::key(KeyCode::F(4)))
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
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: the fixture forces the real `Dialog` \
         as well as its launcher. SELECTED and ACTIVE name no modal state; DISABLED belongs to action \
         buttons; EDITING belongs to the prompt. ERROR, WARNING and BUSY remain the §40 readiness \
         narrowing: `Dialog` accepts no readiness prop and paints no MARKER or ICON. If it gains a \
         readiness prop, those states must return with the required symbol affordances."
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
        let sr = ScrollRegion::new(SCROLL).patch_part(patch_of(f));
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

const PROPS_LIST: Id = Id::root("conformance.props_list");

fn props_list_rows(f: &Fixture) -> Vec<PropsRow<'_>> {
    f.rows
        .iter()
        .map(|row| PropsRow::new(row.key, &row.label, &row.meta).copyable())
        .collect()
}

fn props_list(f: &Fixture) -> PropsList<'_> {
    PropsList::new(PROPS_LIST).patch_part(patch_of(f))
}

/// `PropsList`: keyed borrowed rows, navigation, scrolling and copy actions.
struct PropsListCase;

impl Conformance for PropsListCase {
    const NAME: &'static str = "props_list";
    const FAMILY: Family = Family::PROPS;
    const PARTS: &'static [Part] = PropsList::PARTS;
    type State = PropsState;
    type Action = PropsAction;
    type Cmd = PropsCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::SCROLLS
    }

    fn id() -> Id {
        PROPS_LIST
    }

    fn update(cx: &mut Cx<'_>, st: &mut PropsState, f: &Fixture) -> Response<PropsAction> {
        let rows = props_list_rows(f);
        props_list(f).update(cx, st, &rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &PropsState, f: &Fixture) {
        let rows = props_list_rows(f);
        props_list(f).draw(ui, area, st, &rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Char('y'))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<PropsCmd>] {
        PropsList::new(PROPS_LIST).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(action: &PropsAction) -> Option<ItemKey> {
        match action {
            PropsAction::Copy(key) => Some(*key),
        }
    }

    fn prepare_scroll_fixture(f: &mut Fixture) {
        for index in 5..20 {
            f.rows.push(FixtureRow {
                key: ItemKey::num(100_u64.saturating_add(index as u64)),
                label: format!("Property {index}"),
                meta: format!("value-{index}"),
                disabled: false,
            });
        }
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
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: PropsList has no selection, disabled, validation, readiness, editing, or independent active-item affordance"
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
    TextArea::new(TEXT_AREA, 4)
        .placeholder("Type here")
        .disabled(f.disabled)
        .read_only(f.read_only)
        .status(f.status())
        .patch_part(patch_of(f))
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
            | Caps::REPORTS_STATUS
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

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const EDIT: &[Chord] = &[Chord::key(KeyCode::Char('x'))];
        if state.contains(StateFlags::EDITING) {
            EDIT
        } else {
            &[]
        }
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 6] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
            StateFlags::DISABLED,
            StateFlags::ERROR,
            StateFlags::BUSY,
        ];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED WARNING ACTIVE: TextArea has no selection, press, warning, or active-item affordance"
    }
}

const SELECT: Id = Id::root("conformance.select");

type FixtureSelect<'a> =
    Select<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn select(f: &Fixture) -> FixtureSelect<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;

    Select::new(SELECT)
        .key(key)
        .row(row)
        .placeholder("Choose a person")
        .popup_rows(5)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f))
}

/// `Select`: a one-row field that opens a keyed popover.
///
/// `OVERLAY` without `TRAPS_FOCUS` is §29.6 verbatim: the `LayerSpec::popover`
/// is a pointer barrier, the field keeps the one focus stop while the popup is
/// open, so case 14 checks the open/Esc/restore half and skips the trap half.
///
/// `COLLECTION` exercises the keyed popup rows after the overlay driver opens
/// the component through its real activation route.
struct SelectCase;

impl Conformance for SelectCase {
    const NAME: &'static str = "select";
    const FAMILY: Family = Family::SELECT;
    const PARTS: &'static [Part] = Select::<'static, FixtureRow>::PARTS;
    type State = SelectState;
    type Action = SelectAction;
    type Cmd = SelectCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::FOCUSABLE
            | Caps::DISABLEABLE
            | Caps::OVERLAY
            | Caps::SCROLLS
            | Caps::COLLECTION
            | Caps::SELECTS
    }

    fn id() -> Id {
        SELECT
    }

    fn update(cx: &mut Cx<'_>, st: &mut SelectState, f: &Fixture) -> Response<SelectAction> {
        select(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &SelectState, f: &Fixture) {
        select(f).draw(ui, area, st, &f.rows);
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

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn reveal_item_chords(_key: ItemKey, _f: &Fixture) -> Vec<Chord> {
        vec![Chord::key(KeyCode::End)]
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, k)
    }

    fn layer_id() -> Option<Id> {
        Some(SELECT)
    }

    fn prepare_scroll_fixture(fixture: &mut Fixture) {
        for value in 5..12 {
            fixture.rows.push(FixtureRow {
                key: ItemKey::num(100 + value),
                label: format!("row {value}"),
                meta: format!("meta {value}"),
                disabled: false,
            });
        }
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

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const CHOOSE: &[Chord] = &[Chord::key(KeyCode::Enter), Chord::key(KeyCode::Enter)];
        if state.contains(StateFlags::SELECTED) {
            CHOOSE
        } else {
            &[]
        }
    }
    fn mono_narrowing_reason() -> &'static str {
        "ERROR WARNING EDITING BUSY ACTIVE: Select commits a value instead of editing text, \
         takes no validation or readiness status, and reaches ACTIVE only through the same \
         (LABEL, BOLD) mono rule as PRESSED"
    }
}

const RADIO: Id = Id::root("conformance.radio_group");

type FixtureRadio<'a> =
    RadioGroup<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn radio(f: &Fixture, value: Option<ItemKey>) -> FixtureRadio<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;
    let mut r = RadioGroup::new(RADIO)
        .key(key)
        .row(row)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f));
    let fixture_value = if f.selected {
        f.rows.first().map(|row| row.key)
    } else {
        None
    };
    let value = value.or(fixture_value);
    if let Some(value) = value {
        r = r.value(value);
    }
    r
}

/// `RadioGroup`: keyed options with a cursor separate from the value.
struct RadioGroupCase;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RadioGroupCaseState {
    radio: RadioGroupState,
    value: Option<ItemKey>,
}

impl Conformance for RadioGroupCase {
    const NAME: &'static str = "radio_group";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = FixtureRadio::<'static>::PARTS;
    type State = RadioGroupCaseState;
    type Action = RadioGroupAction;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::DISABLEABLE | Caps::SELECTS
    }

    fn id() -> Id {
        RADIO
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut RadioGroupCaseState,
        f: &Fixture,
    ) -> Response<RadioGroupAction> {
        let response = radio(f, st.value).update(cx, &mut st.radio, &f.rows);
        if let Some(RadioGroupAction::Chose(key)) = response.action_ref() {
            st.value = Some(*key);
        }
        response
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &RadioGroupCaseState, f: &Fixture) {
        radio(f, st.value).draw(ui, area, &st.radio, &f.rows);
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

    fn mono_fixture(state: StateFlags) -> Fixture {
        let mut fixture = Fixture::default();
        fixture.selected = state.contains(StateFlags::SELECTED);
        fixture
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

fn checkbox(f: &Fixture, checked: bool) -> Checkbox<'_> {
    Checkbox::new(CHECKBOX, "Accept terms")
        .checked(checked)
        .read_only(f.read_only)
        .disabled(f.disabled)
        .patch_part(patch_of(f))
}

/// `Checkbox`: a controlled boolean choice.
struct CheckboxCase;

impl Conformance for CheckboxCase {
    const NAME: &'static str = "checkbox";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = Checkbox::PARTS;
    type State = bool;
    type Action = Activated;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::DISABLEABLE | Caps::SELECTS
    }

    fn id() -> Id {
        CHECKBOX
    }

    fn update(cx: &mut Cx<'_>, st: &mut bool, f: &Fixture) -> Response<Activated> {
        checkbox(f, *st).update(cx, st)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &bool, f: &Fixture) {
        checkbox(f, *st).draw(ui, area);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        Checkbox::new(CHECKBOX, "").bindings(s)
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Char(' '))];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
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

fn toggle(f: &Fixture, on: bool) -> Toggle<'_> {
    Toggle::new(TOGGLE, "Notifications")
        .on(on)
        .read_only(f.read_only)
        .disabled(f.disabled)
        .patch_part(patch_of(f))
}

/// `Toggle`: a controlled switch.
struct ToggleCase;

impl Conformance for ToggleCase {
    const NAME: &'static str = "toggle";
    const FAMILY: Family = Family::CHOICE;
    const PARTS: &'static [Part] = Toggle::PARTS;
    type State = bool;
    type Action = Activated;
    type Cmd = ChoiceCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::DISABLEABLE | Caps::SELECTS
    }

    fn id() -> Id {
        TOGGLE
    }

    fn update(cx: &mut Cx<'_>, st: &mut bool, f: &Fixture) -> Response<Activated> {
        toggle(f, *st).update(cx, st)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &bool, f: &Fixture) {
        toggle(f, *st).draw(ui, area);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChoiceCmd>] {
        Toggle::new(TOGGLE, "").bindings(s)
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Char(' '))];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
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

    ChipBar::new(CHIP_BAR)
        .key(key)
        .row(row)
        .select_mode(SelectMode::Multi)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f))
}

/// `ChipBar`: a keyed, single-activation chip strip.
struct ChipBarCase;

impl Conformance for ChipBarCase {
    const NAME: &'static str = "chip_bar";
    const FAMILY: Family = Family::CHIP;
    const PARTS: &'static [Part] = FixtureChips::<'static>::PARTS;
    type State = junie_tui::ChipBarState;
    type Action = ChipBarAction;
    type Cmd = ChipBarCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::COLLECTION | Caps::DISABLEABLE | Caps::SELECTS
    }

    fn id() -> Id {
        CHIP_BAR
    }

    fn update(
        cx: &mut Cx<'_>,
        st: &mut junie_tui::ChipBarState,
        f: &Fixture,
    ) -> Response<ChipBarAction> {
        chip_bar(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &junie_tui::ChipBarState, f: &Fixture) {
        chip_bar(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::LABEL, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<ChipBarCmd>] {
        ChipBar::<FixtureRow>::new(CHIP_BAR)
            .select_mode(SelectMode::Multi)
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
            ChipBarAction::AddRequested => None,
        }
    }

    fn row_part(k: ItemKey) -> PartRef {
        PartRef::item(Part::LABEL, k)
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Char(' '))];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
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
const STATUS_LEFT_WARNING: [StatusItem<'static>; 2] = [
    StatusItem::new("Warning").strong().tone(Role::Warning),
    StatusItem::new("main").key(ItemKey::num(1)),
];
const STATUS_CENTER: [StatusItem<'static>; 1] = [StatusItem::new("Ready")];
const STATUS_RIGHT: [StatusItem<'static>; 1] = [StatusItem::new("0 changes").key(ItemKey::num(2))];

fn status_bar(f: &Fixture) -> StatusBar<'_> {
    let left = if f.decor_flags.contains(StateFlags::WARNING) {
        &STATUS_LEFT_WARNING
    } else {
        &STATUS_LEFT
    };

    StatusBar::new(STATUS_BAR)
        .left(left)
        .center(&STATUS_CENTER)
        .right(&STATUS_RIGHT)
        .status(f.status())
        .patch_part(patch_of(f))
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
        let h = HintBar::new(HINT_BAR, &layer)
            .status(f.status())
            .patch_part(patch_of(f));
        h.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 3] = [StateFlags::empty(), StateFlags::ERROR, StateFlags::BUSY];
        &STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED WARNING EDITING ACTIVE: HintBar registers no ring entry and no \
         region and never calls `Ui::state`, so its live flags are `Status::flags()` alone and no \
         runtime state can reach it — nothing focuses, presses or disables a bar that owns no \
         control. A hint is a *label* for a chord another component declared, so SELECTED, EDITING \
         and ACTIVE have no referent here either; its readiness prop has no Warning variant. \
         BUSY and ERROR are kept: `HintBar::status_glyph` leads the status message with a spinner \
         frame or error glyph, and `status_width` counts that glyph, so it also changes how many hints \
         fit the row."
    }
}

const DERIVED_HINT_BAR: Id = Id::root("conformance.derived_hint_bar");

struct DerivedHintBarCase;

impl Conformance for DerivedHintBarCase {
    const NAME: &'static str = "derived_hint_bar";
    const FAMILY: Family = Family::HINTBAR;
    const PARTS: &'static [Part] = HintBar::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ();

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        DERIVED_HINT_BAR
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored()
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), _f: &Fixture) {
        let global = HintLayer::from_bindings(&[Binding {
            action: ActionKey::custom("derived-hint.help"),
            chord: Some(Chord::key(KeyCode::F(1))),
            cmd: (),
            label: "Help",
            priority: 80,
            visible: true,
        }]);
        HintBar::derived(DERIVED_HINT_BAR)
            .global(&global)
            .draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[StateFlags::empty()];
        STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: DerivedHintBar is a stateless composition over hint layers"
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
        let h = KeyHint::new(KEY_HINT, Chord::key(KeyCode::Enter), "Open").patch_part(patch_of(f));
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
        let p = ProgressBar::new(PROGRESS_BAR)
            .label("Uploading")
            .ratio(0.65)
            .status(f.status())
            .patch_part(patch_of(f));
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
        let s = Spinner::new(SPINNER)
            .label("Working")
            .frame(1)
            .patch_part(patch_of(f));
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
        let m = Meter::new(METER)
            .ratio(0.65)
            .value("65%")
            .status(f.status())
            .patch_part(patch_of(f));
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
        let e = Empty::new(EMPTY, empty_state(f)).patch_part(patch_of(f));
        let _ = e.draw(ui, area);
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
        let b = Brand::new(BRAND, "Junie")
            .tagline("Terminal tools")
            .patch_part(patch_of(f));
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

const PANEL: Id = Id::root("conformance.panel");

/// `Panel`: stateless container chrome around caller-painted content.
struct PanelCase;

impl Conformance for PanelCase {
    const NAME: &'static str = "panel";
    const FAMILY: Family = Family::PANEL;
    const PARTS: &'static [Part] = Panel::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ();

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        PANEL
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored().for_id(PANEL)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let panel = Panel::new(PANEL)
            .title("Panel")
            .meta("meta")
            .patch_part(patch_of(f));
        panel.draw(ui, area, |ui, body| {
            let style = ui
                .style(
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::TITLE,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_str(body, "content", style);
        });
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: Panel is stateless container chrome"
    }
}

const SPLIT_PANE: Id = Id::root("conformance.split_pane");

fn split_pane(f: &Fixture) -> SplitPane<'_> {
    SplitPane::new(SPLIT_PANE, SplitAxis::Horizontal)
        .resizable(true)
        .patch_part(patch_of(f))
}

/// `SplitPane`: focusable keyboard resizing and pointer-captured seam dragging.
struct SplitPaneCase;

impl Conformance for SplitPaneCase {
    const NAME: &'static str = "split_pane";
    const FAMILY: Family = Family::SPLIT;
    const PARTS: &'static [Part] = SplitPane::PARTS;
    type State = SplitPaneState;
    type Action = SplitAction;
    type Cmd = SplitCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::CAPTURES
    }

    fn id() -> Id {
        SPLIT_PANE
    }

    fn update(cx: &mut Cx<'_>, st: &mut SplitPaneState, f: &Fixture) -> Response<SplitAction> {
        split_pane(f).update(cx, st)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &SplitPaneState, f: &Fixture) {
        split_pane(f).draw(ui, area, st, |_, _, _| {});
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::SEAM)
    }

    fn bindings(s: BindingState) -> &'static [Binding<SplitCmd>] {
        SplitPane::new(SPLIT_PANE, SplitAxis::Horizontal)
            .resizable(true)
            .bindings(s)
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
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: SplitPane has no selection, disabled, validation, edit, readiness, or active-item state"
    }
}

const TEXT_VIEWPORT: Id = Id::root("conformance.text_viewport");
const DRIVER_POPOVER: Id = Id::root("conformance.popover");

fn viewport_lines(f: &Fixture) -> Vec<ViewportLine<'_>> {
    f.rows
        .iter()
        .flat_map(|row| {
            [
                ViewportLine::Plain(row.label.as_str()),
                ViewportLine::Plain(row.meta.as_str()),
            ]
        })
        .collect()
}

fn text_viewport(f: &Fixture) -> TextViewport<'_> {
    TextViewport::new(TEXT_VIEWPORT)
        .wrap(true)
        .patch_part(patch_of(f))
}

/// `TextViewport`: read-only scrolling, selection capture, and caret output.
struct TextViewportCase;

impl Conformance for TextViewportCase {
    const NAME: &'static str = "text_viewport";
    const FAMILY: Family = Family::VIEWPORT;
    const PARTS: &'static [Part] = TextViewport::PARTS;
    type State = ViewportState;
    type Action = ViewportAction;
    type Cmd = ViewportCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::SCROLLS | Caps::CAPTURES | Caps::CURSOR
    }

    fn id() -> Id {
        TEXT_VIEWPORT
    }

    fn update(cx: &mut Cx<'_>, st: &mut ViewportState, f: &Fixture) -> Response<ViewportAction> {
        // The generic driver owns this layer. Its lifecycle event is not a
        // viewport intent and must not become an unrelated undelivered-event
        // diagnostic in the cursor-layer case.
        let _ = cx.layer_event(DRIVER_POPOVER);
        text_viewport(f).update(cx, st, &viewport_lines(f))
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &ViewportState, f: &Fixture) {
        let mut reference = st.clone();
        reference.set_follow(false);
        let lines = viewport_lines(f);
        let caret_line = reference
            .scroll()
            .offset()
            .min(lines.len().saturating_sub(1));
        // `ScrollState` can transiently hold the unclamped tail offset from
        // the zero-viewport bootstrap frame. The cursor fixture must name a
        // visible logical line, not that pre-layout sentinel.
        reference.set_caret(Some(CellPos::new(caret_line, 0)));
        text_viewport(f).draw(ui, area, &reference, &lines);
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::TEXT)
    }

    fn bindings(s: BindingState) -> &'static [Binding<ViewportCmd>] {
        TextViewport::new(TEXT_VIEWPORT).bindings(s)
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
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: TextViewport has no component selection, disabled, validation, edit, readiness, or active-item state"
    }
}

const CODE_EDITOR: Id = Id::root("conformance.code_editor");

#[derive(Clone, PartialEq, Eq, Debug)]
struct CodeEditorCaseState(CodeEditorState);

impl Default for CodeEditorCaseState {
    fn default() -> Self {
        CodeEditorCaseState(CodeEditorState::new(
            "fn main() {\n    println!(\"hello\");\n}",
        ))
    }
}

fn code_editor(f: &Fixture) -> CodeEditor<'_> {
    CodeEditor::new(CODE_EDITOR, 6)
        .disabled(f.disabled)
        .read_only(f.read_only)
        .patch_part(patch_of(f))
}

struct CodeEditorCase;

impl Conformance for CodeEditorCase {
    const NAME: &'static str = "code_editor";
    const FAMILY: Family = Family::CODE;
    const PARTS: &'static [Part] = CodeEditor::PARTS;
    type State = CodeEditorCaseState;
    type Action = CodeAction;
    type Cmd = CodeCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE
            | Caps::EDITS
            | Caps::CURSOR
            | Caps::TYPES
            | Caps::SCROLLS
            | Caps::DISABLEABLE
    }

    fn id() -> Id {
        CODE_EDITOR
    }

    fn update(cx: &mut Cx<'_>, st: &mut CodeEditorCaseState, f: &Fixture) -> Response<CodeAction> {
        code_editor(f).update(cx, &mut st.0)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &CodeEditorCaseState, f: &Fixture) {
        code_editor(f).draw(ui, area, &st.0);
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::TEXT)
    }

    fn bindings(state: BindingState) -> &'static [Binding<CodeCmd>] {
        CodeEditor::new(CODE_EDITOR, 6).bindings(state)
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const EDIT: &[Chord] = &[Chord::key(KeyCode::Enter)];
        if state.contains(StateFlags::EDITING) {
            EDIT
        } else {
            &[]
        }
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::DISABLED,
            StateFlags::EDITING,
        ];
        STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED ERROR WARNING BUSY ACTIVE: selection is a range, pointer press has no persistent capture, diagnostics are data rather than forced fixture state, and the editor exposes no readiness or active-item state"
    }
}

const DIFF_VIEW: Id = Id::root("conformance.diff_view");
const DIFF_ROWS: &[DiffRow<'static>] = &[
    DiffRow::Hunk {
        old_start: 1,
        new_start: 1,
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "fn main() {",
    },
    DiffRow::Line {
        kind: DiffLineKind::Remove,
        text: "    old();",
    },
    DiffRow::Line {
        kind: DiffLineKind::Add,
        text: "    new();",
    },
    DiffRow::Line {
        kind: DiffLineKind::Context,
        text: "}",
    },
];

struct ConformanceDiff;

impl DiffSource for ConformanceDiff {
    fn revision(&self) -> u64 {
        1
    }
    fn path(&self) -> &'static str {
        "src/main.rs"
    }
    fn status_marker(&self) -> &'static str {
        "M"
    }
    fn status_label(&self) -> &'static str {
        "modified"
    }
    fn row_count(&self) -> usize {
        DIFF_ROWS.len()
    }
    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        DIFF_ROWS.get(index).copied()
    }
}

fn diff_view(f: &Fixture) -> DiffView<'_> {
    DiffView::new(DIFF_VIEW, Some(&ConformanceDiff)).patch_part(patch_of(f))
}

struct DiffViewCase;

impl Conformance for DiffViewCase {
    const NAME: &'static str = "diff_view";
    const FAMILY: Family = Family::DIFF;
    const PARTS: &'static [Part] = DiffView::PARTS;
    type State = DiffViewState;
    type Action = ViewportAction;
    type Cmd = ViewportCmd;

    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::SCROLLS | Caps::CAPTURES
    }

    fn id() -> Id {
        DIFF_VIEW
    }

    fn update(cx: &mut Cx<'_>, st: &mut DiffViewState, f: &Fixture) -> Response<ViewportAction> {
        diff_view(f).update(cx, st)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &DiffViewState, f: &Fixture) {
        diff_view(f).draw(ui, area, st);
    }

    fn activation_part() -> PartRef {
        PartRef::of(Part::TEXT)
    }

    fn bindings(state: BindingState) -> &'static [Binding<ViewportCmd>] {
        TextViewport::new(DIFF_VIEW).bindings(state)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
        ];
        STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: DiffView is a read-only viewport projection with range selection and no validation, edit, readiness, or active-item state"
    }
}

const TREE: Id = Id::root("conformance.tree");

type FixtureTree<'a> =
    Tree<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn fixture_tree_node(row: &FixtureRow) -> TreeNode {
    match row.key {
        key if key == ItemKey::num(101) => TreeNode::parent(0),
        key if key == ItemKey::num(102) => TreeNode::leaf(1),
        _ => TreeNode::leaf(0),
    }
}

fn tree(f: &Fixture) -> FixtureTree<'_> {
    let key: fn(&FixtureRow) -> ItemKey = row_key;
    let row: fn(&FixtureRow, &mut RowUi<'_>) = row_label;
    let disabled: &dyn Fn(&FixtureRow) -> bool = &row_disabled;

    Tree::new(TREE)
        .key(key)
        .row(row)
        .node(&(fixture_tree_node as fn(&FixtureRow) -> TreeNode))
        .disabled_item(disabled)
        .disabled(f.disabled)
        .patch_part(patch_of(f))
}

/// `Tree`: keyed rows, choice/activation, focus, and scrolling.
struct TreeCase;

impl Conformance for TreeCase {
    const NAME: &'static str = "tree";
    const FAMILY: Family = Family::TREE;
    const PARTS: &'static [Part] = FixtureTree::PARTS;
    type State = TreeState;
    type Action = TreeAction;
    type Cmd = TreeCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::DISABLEABLE
            | Caps::FOCUSABLE
            | Caps::COLLECTION
            | Caps::SCROLLS
            | Caps::SELECTS
    }

    fn id() -> Id {
        TREE
    }

    fn update(cx: &mut Cx<'_>, st: &mut TreeState, f: &Fixture) -> Response<TreeAction> {
        tree(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &TreeState, f: &Fixture) {
        tree(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<TreeCmd>] {
        Tree::<FixtureRow>::new(TREE).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(action: &TreeAction) -> Option<ItemKey> {
        match action {
            TreeAction::Expanded(key)
            | TreeAction::Collapsed(key)
            | TreeAction::Chose(key)
            | TreeAction::Activated(key) => Some(*key),
            TreeAction::Moved => None,
        }
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Char(' '))];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
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
        "ERROR WARNING EDITING BUSY ACTIVE: Tree has no component validation, edit, readiness, or active-item state"
    }
}

const NAV_LIST: Id = Id::root("conformance.nav_list");

fn nav_section(_row: &FixtureRow) -> &'static str {
    "Main"
}

fn nav_icon(_row: &FixtureRow) -> &'static str {
    "›"
}

fn nav_badge(row: &FixtureRow) -> Option<&str> {
    (row.key == ItemKey::num(100)).then_some("3")
}

type FixtureNavList<'a> =
    NavList<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn nav_list(f: &Fixture) -> FixtureNavList<'_> {
    let disabled = f.disabled;

    NavList::new(NAV_LIST)
        .key(row_key as fn(&FixtureRow) -> ItemKey)
        .row(row_label as fn(&FixtureRow, &mut RowUi<'_>))
        .section(&(nav_section as fn(&FixtureRow) -> &str))
        .icon(&(nav_icon as fn(&FixtureRow) -> &str))
        .badge(&(nav_badge as fn(&FixtureRow) -> Option<&str>))
        .disabled_item(&(row_disabled as fn(&FixtureRow) -> bool))
        .disabled(disabled)
        .patch_part(patch_of(f))
}

struct NavListCase;

impl Conformance for NavListCase {
    const NAME: &'static str = "nav_list";
    const FAMILY: Family = Family::LIST;
    const PARTS: &'static [Part] = FixtureNavList::PARTS;
    type State = NavListState;
    type Action = NavListAction;
    type Cmd = NavListCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE | Caps::COLLECTION | Caps::SELECTS
    }

    fn id() -> Id {
        NAV_LIST
    }

    fn update(cx: &mut Cx<'_>, st: &mut NavListState, f: &Fixture) -> Response<NavListAction> {
        nav_list(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &NavListState, f: &Fixture) {
        nav_list(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<NavListCmd>] {
        NavList::<FixtureRow>::new(NAV_LIST).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(action: &NavListAction) -> Option<ItemKey> {
        match action {
            NavListAction::Moved(key)
            | NavListAction::Chose(key)
            | NavListAction::EnterContent(key) => Some(*key),
        }
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: &[Chord] = &[Chord::key(KeyCode::Enter)];
        if state.contains(StateFlags::SELECTED) {
            SELECT
        } else {
            &[]
        }
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
        "ERROR WARNING EDITING BUSY ACTIVE: NavList has no validation, edit, readiness, or active-item state"
    }
}

const STEPS: Id = Id::root("conformance.steps");

fn fixture_step(row: &FixtureRow) -> StepState {
    match row.key {
        key if key == ItemKey::num(100) => StepState::Done,
        key if key == ItemKey::num(101) => StepState::Running,
        key if key == ItemKey::num(102) => StepState::Failed,
        key if key == ItemKey::num(103) => StepState::Blocked,
        _ => StepState::Queued,
    }
}

type FixtureSteps<'a> =
    Steps<'a, FixtureRow, fn(&FixtureRow) -> ItemKey, fn(&FixtureRow, &mut RowUi<'_>)>;

fn steps(f: &Fixture) -> FixtureSteps<'_> {
    Steps::navigable(STEPS)
        .key(row_key as fn(&FixtureRow) -> ItemKey)
        .row(row_label as fn(&FixtureRow, &mut RowUi<'_>))
        .step(&(fixture_step as fn(&FixtureRow) -> StepState))
        .disabled(f.disabled)
        .patch_part(patch_of(f))
}

struct StepsCase;

impl Conformance for StepsCase {
    const NAME: &'static str = "steps";
    const FAMILY: Family = Family::STEPS;
    const PARTS: &'static [Part] = FixtureSteps::PARTS;
    type State = StepsState;
    type Action = StepsAction;
    type Cmd = StepsCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::DISABLEABLE | Caps::FOCUSABLE | Caps::COLLECTION | Caps::SCROLLS
    }

    fn id() -> Id {
        STEPS
    }

    fn update(cx: &mut Cx<'_>, st: &mut StepsState, f: &Fixture) -> Response<StepsAction> {
        steps(f).update(cx, st, &f.rows)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &StepsState, f: &Fixture) {
        steps(f).draw(ui, area, st, &f.rows);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Enter)];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }

    fn bindings(s: BindingState) -> &'static [Binding<StepsCmd>] {
        Steps::<FixtureRow>::navigable(STEPS).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(action: &StepsAction) -> Option<ItemKey> {
        match action {
            StepsAction::Moved(key) | StepsAction::Activated(key) => Some(*key),
        }
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
        "SELECTED ERROR WARNING EDITING BUSY ACTIVE: Steps lifecycle states belong to items; the rail has no component selection, validation, edit, readiness, or active-item state"
    }
}

const TOO_SMALL: Id = Id::root("conformance.too_small");

struct TooSmallCase;

impl Conformance for TooSmallCase {
    const NAME: &'static str = "too_small";
    const FAMILY: Family = Family::TOO_SMALL;
    const PARTS: &'static [Part] = TooSmall::PARTS;
    type State = ();
    type Action = ();
    type Cmd = ();

    fn caps() -> Caps {
        Caps::empty()
    }

    fn id() -> Id {
        TOO_SMALL
    }

    fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
        Response::ignored().for_id(TOO_SMALL)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), f: &Fixture) {
        let notice = TooSmall::new(TOO_SMALL, "Junie").patch_part(patch_of(f));
        notice.draw(ui, area);
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 1] = [StateFlags::empty()];
        &STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: TooSmall is stateless terminal-size chrome"
    }
}

const GRID: Id = Id::root("conformance.grid");
const GRID_COLUMNS: [Column<'static>; 2] = [
    Column {
        editable: true,
        sticky: true,
        ..Column::new(ColumnKey::num(1), "Name")
    },
    Column::new(ColumnKey::num(2), "Detail"),
];

struct FixtureGridModel<'a> {
    rows: &'a [FixtureRow],
    editable: bool,
    decor_flags: StateFlags,
}

impl GridModel for FixtureGridModel<'_> {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::index(row), |item| item.key)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let row = self.rows.get(row)?;
        match col {
            0 => Some(CellRef::new(&row.label)),
            1 => Some(CellRef::new(&row.meta)),
            _ => None,
        }
    }

    fn cell_decor(&self, _row: usize, _col: usize) -> CellDecor<'_> {
        CellDecor {
            error: self
                .decor_flags
                .contains(StateFlags::ERROR)
                .then_some("Invalid cell"),
            ..CellDecor::default()
        }
    }
}

impl GridEditor for FixtureGridModel<'_> {
    fn edit_intent(&self, row: usize, _col: usize) -> EditIntent<'_> {
        if self.editable && self.rows.get(row).is_some() {
            EditIntent::Inline { initial: "row 0" }
        } else {
            EditIntent::Refuse {
                reason: self
                    .rows
                    .get(row)
                    .map_or("Missing row", |_| "Read only cell"),
            }
        }
    }

    fn apply_cycle(&mut self, _row: usize, _col: usize) {}

    fn commit_cell(&mut self, _row: usize, _col: usize, _text: &str) -> Result<(), FieldError> {
        Ok(())
    }

    fn is_editable(&self, _row: usize, _col: usize) -> bool {
        self.editable
    }
}

fn grid(f: &Fixture) -> Grid<'_> {
    Grid::new(GRID, &GRID_COLUMNS)
        .nav(NavUnit::Row)
        .patch_part(patch_of(f))
}

struct GridCase;

impl Conformance for GridCase {
    const NAME: &'static str = "grid";
    const FAMILY: Family = Family::GRID;
    const PARTS: &'static [Part] = Grid::PARTS;
    type State = GridState;
    type Action = GridAction;
    type Cmd = GridCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::FOCUSABLE
            | Caps::COLLECTION
            | Caps::SCROLLS
            | Caps::EDITS
            | Caps::SELECTS
    }

    fn id() -> Id {
        GRID
    }

    fn update(cx: &mut Cx<'_>, st: &mut GridState, f: &Fixture) -> Response<GridAction> {
        let mut model = FixtureGridModel {
            rows: &f.rows,
            editable: !f.read_only,
            decor_flags: f.decor_flags,
        };
        if f.read_only {
            grid(f).update(cx, st, &model)
        } else {
            grid(f).update_editable(cx, st, &mut model)
        }
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &GridState, f: &Fixture) {
        grid(f).draw(
            ui,
            area,
            st,
            &FixtureGridModel {
                rows: &f.rows,
                editable: false,
                decor_flags: f.decor_flags,
            },
        );
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 1] = [Chord::key(KeyCode::Enter)];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(Part::CELL, ItemKey::num(100))
    }

    fn activation_gesture() -> PointerGesture {
        PointerGesture::DoubleClick
    }

    fn bindings(s: BindingState) -> &'static [Binding<GridCmd>] {
        Grid::new(GRID, &GRID_COLUMNS).bindings(s)
    }

    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }

    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }

    fn action_key_of(action: &GridAction) -> Option<ItemKey> {
        match action {
            GridAction::Activated(key)
            | GridAction::EditRequested(key, _)
            | GridAction::CellAction(key, _, _) => Some(*key),
            GridAction::Moved
            | GridAction::Sort(_, _)
            | GridAction::FetchMore
            | GridAction::Copy(_)
            | GridAction::LeaveForward
            | GridAction::LeaveBackward => None,
        }
    }

    fn row_part(key: ItemKey) -> PartRef {
        PartRef::item(Part::CELL, key)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 6] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
            StateFlags::EDITING,
            StateFlags::ERROR,
        ];
        &STATES
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const SELECT: [Chord; 1] = [Chord::key(KeyCode::Char(' '))];
        const EDIT: [Chord; 1] = [Chord::key(KeyCode::F(2))];
        if state.contains(StateFlags::SELECTED) {
            &SELECT
        } else if state.contains(StateFlags::EDITING) {
            &EDIT
        } else {
            &[]
        }
    }

    fn mono_fixture(state: StateFlags) -> Fixture {
        let mut fixture = Fixture::default();
        fixture.read_only = false;
        fixture.decor_flags = state & StateFlags::ERROR;
        fixture
    }

    fn mono_narrowing_reason() -> &'static str {
        "DISABLED WARNING BUSY ACTIVE: Grid has no disabled prop, warning or whole-grid readiness, or component active-item state"
    }
}

#[test]
fn grid_case_runs_both_update_entry_points() {
    use junie_tui_testing::conformance::driver::CaseApp;

    for read_only in [true, false] {
        let mut fixture = Fixture::default();
        fixture.read_only = read_only;
        let mut harness = Harness::new(CaseApp::<GridCase>::new(fixture), Theme::junie(), 40, 12);
        assert!(harness.tab_to(GRID));
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(
            harness.app().last,
            Some(GridAction::Activated(ItemKey::num(100))),
            "read_only={read_only} did not execute its Grid update entry point"
        );
    }
}

#[test]
fn grid_mono_setup_chords_reach_real_selection_and_editing() {
    use junie_tui_testing::conformance::driver::CaseApp;

    let setup = |state| {
        let fixture = GridCase::mono_fixture(state);
        assert_eq!(fixture.forced(), None, "setup fixture must stay live");
        let mut harness = Harness::new(CaseApp::<GridCase>::new(fixture), Theme::junie(), 40, 12);
        assert!(harness.tab_to(GRID));
        for chord in GridCase::mono_setup_chords(state) {
            let _ = harness.key_mod(chord.code, chord.mods);
        }
        harness.app().st.clone()
    };

    let selected = setup(StateFlags::SELECTED);
    assert!(selected.selected_rows().contains(ItemKey::num(100)));
    let editing = setup(StateFlags::EDITING);
    assert!(editing.is_editing());
}

#[test]
fn grid_error_is_real_cell_decor_and_mono_distinct() {
    use junie_tui_testing::conformance::driver::CaseApp;

    let semantic = GridCase::mono_fixture(StateFlags::ERROR);
    assert_eq!(semantic.forced(), None, "semantic fixture must stay live");
    let model = FixtureGridModel {
        rows: &semantic.rows,
        editable: false,
        decor_flags: semantic.decor_flags,
    };
    assert!(model.cell_decor(0, 0).flags().contains(StateFlags::ERROR));

    for theme in [Theme::junie(), Theme::paper()] {
        let digest = |state| {
            let mut fixture = GridCase::mono_fixture(state).force(state);
            fixture.color = junie_tui::ColorLevel::Mono;
            fixture.theme = theme.clone();
            let mono = fixture.theme.clone().downgrade(fixture.color);
            Harness::new(CaseApp::<GridCase>::new(fixture), mono, 40, 12)
                .snapshot()
                .digest()
        };
        assert_ne!(
            digest(StateFlags::empty()),
            digest(StateFlags::ERROR),
            "Grid cell ERROR must remain visible in mono"
        );
    }
}

const MENU_BAR: Id = Id::root("conformance.menu_bar");
const CONTEXT_MENU: Id = Id::root("conformance.context_menu");

const FILTER_LIST: Id = Id::root("conformance.filter_list");

fn semantic_items(f: &Fixture) -> Vec<Item<'_>> {
    f.rows
        .iter()
        .map(|row| {
            Item::new(row.key, &row.label)
                .detail(&row.meta)
                .disabled(row.disabled)
        })
        .collect()
}

struct FilterListCase;
impl Conformance for FilterListCase {
    const NAME: &'static str = "filter_list";
    const FAMILY: Family = Family::PICKER;
    const PARTS: &'static [Part] = FilterList::<Item<'static>>::PARTS;
    type State = FilterListState;
    type Action = FilterListAction;
    type Cmd = FilterListCmd;
    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::FOCUSABLE
            | Caps::COLLECTION
            | Caps::SCROLLS
            | Caps::TYPES
            | Caps::REPORTS_STATUS
    }
    fn id() -> Id {
        FILTER_LIST
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let items = semantic_items(f);
        FilterList::<Item<'_>>::new(FILTER_LIST)
            .status(f.status())
            .patch_part(patch_of(f))
            .update(cx, st, &items)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let items = semantic_items(f);
        let list = FilterList::<Item<'_>>::new(FILTER_LIST)
            .status(f.status())
            .patch_part(patch_of(f));
        list.draw(ui, area, st, &items);
    }
    fn activation_chords() -> &'static [Chord] {
        const C: &[Chord] = &[Chord::key(KeyCode::Enter)];
        C
    }
    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        FilterList::<Item<'static>>::new(FILTER_LIST).bindings(state)
    }
    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }
    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }
    fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
        match action {
            FilterListAction::Chose(k)
            | FilterListAction::ChoseAlt(k)
            | FilterListAction::Secondary(k) => Some(*k),
            _ => None,
        }
    }
    fn mono_states() -> &'static [StateFlags] {
        const S: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::ERROR,
            StateFlags::BUSY,
        ];
        S
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED WARNING EDITING ACTIVE: item disabling is per-row; the query consumes text but has no commit lifecycle; the cursor is navigation, not a chosen value"
    }
}

const PICKER: Id = Id::root("conformance.picker");
struct PickerCase;
impl Conformance for PickerCase {
    const NAME: &'static str = "picker";
    const FAMILY: Family = Family::PICKER;
    const PARTS: &'static [Part] = Picker::<Item<'static>>::PARTS;
    type State = PickerState;
    type Action = PickerAction;
    type Cmd = FilterListCmd;
    fn caps() -> Caps {
        Caps::ACTIVATES
            | Caps::FOCUSABLE
            | Caps::COLLECTION
            | Caps::SCROLLS
            | Caps::TYPES
            | Caps::OVERLAY
            | Caps::TRAPS_FOCUS
    }
    fn id() -> Id {
        PICKER
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let items = semantic_items(f);
        let picker = Picker::<Item<'_>>::new(PICKER).patch_part(patch_of(f));
        if cx
            .intents(PICKER)
            .any(|intent| matches!(intent, Intent::Key(k) if Chord::key(KeyCode::F(4)).matches(&k)))
        {
            cx.open_layer(PICKER, picker.layer(cx, &items));
            return Response::changed()
                .for_id(PICKER)
                .map_action(|()| PickerAction::QueryChanged);
        }
        picker.update(cx, st, &items)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let items = semantic_items(f);
        let picker = Picker::<Item<'_>>::new(PICKER).patch_part(patch_of(f));
        let layer_drawn = ui.layer(PICKER, |ui, layer| {
            picker.draw(ui, layer, st, &items);
        });
        if layer_drawn.is_none() {
            picker.draw(ui, area, st, &items);
        }
    }
    fn activation_chords() -> &'static [Chord] {
        const C: &[Chord] = &[Chord::key(KeyCode::Enter)];
        C
    }
    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }
    fn open_chord() -> Option<Chord> {
        Some(Chord::key(KeyCode::F(4)))
    }
    fn layer_id() -> Option<Id> {
        Some(PICKER)
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        FilterList::<Item<'static>>::new(PICKER).bindings(state)
    }
    fn legacy_key_chords() -> &'static [Chord] {
        const CHORDS: &[Chord] = &[Chord::key(KeyCode::F(4))];
        CHORDS
    }
    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|r| r.key).collect()
    }
    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }
    fn reveal_item_chords(_key: ItemKey, _f: &Fixture) -> Vec<Chord> {
        vec![Chord::key(KeyCode::End)]
    }
    fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
        match action {
            PickerAction::Chosen(k) | PickerAction::ChosenAlt(k) | PickerAction::Secondary(k) => {
                Some(*k)
            }
            _ => None,
        }
    }
    fn row_part(key: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, key)
    }
    fn mono_states() -> &'static [StateFlags] {
        const S: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
        ];
        S
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: picker has no whole-control disable/readiness or chosen-value state; typing is query capture rather than commit editing"
    }
}

const COMPLETION: Id = Id::root("conformance.completion");
const COMPLETION_EDITOR: Id = Id::root("conformance.completion.editor");
struct CompletionCase;
impl Conformance for CompletionCase {
    const NAME: &'static str = "completion";
    const FAMILY: Family = Family::COMPLETION;
    const PARTS: &'static [Part] = Completion::<Item<'static>>::PARTS;
    type State = CompletionState;
    type Action = CompletionAction;
    type Cmd = CompletionCmd;
    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::COLLECTION | Caps::SCROLLS | Caps::OVERLAY
    }
    fn id() -> Id {
        COMPLETION
    }
    fn control_id() -> Id {
        COMPLETION_EDITOR
    }
    fn scroll_id() -> Id {
        COMPLETION
    }
    fn activation_id() -> Id {
        COMPLETION
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let items = semantic_items(f);
        Completion::<Item<'_>>::new(COMPLETION).update_for(COMPLETION_EDITOR, cx, st, &items)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let items = semantic_items(f);
        let completion = Completion::<Item<'_>>::new(COMPLETION).patch_part(patch_of(f));
        ui.register_control(COMPLETION_EDITOR, area, Focusability::Focusable);
        let layer_drawn = ui.layer(COMPLETION, |ui, layer_area| {
            completion.draw(ui, layer_area, st, &items);
        });
        if layer_drawn.is_none() {
            completion.draw(ui, area, st, &items);
        }
    }
    fn activation_chords() -> &'static [Chord] {
        const CHORDS: &[Chord] = &[Chord::key(KeyCode::Enter), Chord::key(KeyCode::Tab)];
        CHORDS
    }
    fn activation_part() -> PartRef {
        PartRef::item(Part::ROW, ItemKey::num(100))
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        Completion::<Item<'static>>::new(COMPLETION).bindings(state)
    }
    fn prepare_scroll_fixture(fixture: &mut Fixture) {
        for value in 5..12 {
            fixture.rows.push(FixtureRow {
                key: ItemKey::num(100 + value),
                label: format!("row {value}"),
                meta: format!("meta {value}"),
                disabled: false,
            });
        }
    }
    fn scroll_setup_ticks() -> usize {
        1
    }
    fn item_keys(f: &Fixture) -> Vec<ItemKey> {
        f.rows.iter().map(|row| row.key).collect()
    }
    fn reorder(f: &mut Fixture, perm: &[usize]) {
        let rows = f.rows.clone();
        f.rows = perm.iter().filter_map(|&i| rows.get(i).cloned()).collect();
    }
    fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
        match action {
            CompletionAction::Accepted(key) => Some(*key),
            CompletionAction::Moved | CompletionAction::Dismissed => None,
        }
    }
    fn row_part(key: ItemKey) -> PartRef {
        PartRef::item(Part::ROW, key)
    }
    fn open_chord() -> Option<Chord> {
        None
    }
    fn open_overlay(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> bool {
        let items = semantic_items(f);
        CompletionController::new(COMPLETION_EDITOR, COMPLETION).request(
            cx,
            st,
            Rect::new(0, 0, 1, 1),
            0,
            &items,
        );
        true
    }
    fn layer_id() -> Option<Id> {
        Some(COMPLETION)
    }
    fn mono_states() -> &'static [StateFlags] {
        const S: &[StateFlags] = &[StateFlags::empty(), StateFlags::PRESSED];
        S
    }
    fn mono_narrowing_reason() -> &'static str {
        "FOCUSED SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: completion retains editor focus and readiness belongs to semantic items"
    }
}

const MENU_ITEMS: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::SAVE, "Save").chord(Chord::key(KeyCode::Char('s'))),
    MenuItem::new(ActionKey::CLOSE, "Close"),
];
const MENUS: [Menu<'static>; 1] = [Menu::new("File", &MENU_ITEMS)];
const CONTEXT_LEGACY_CHORDS: &[Chord] = &[Chord::key(KeyCode::F(4))];

macro_rules! menu_case {
    ($case:ident, $name:literal, $id:ident, $ty:ty, $part:expr, $akey:expr, $make:expr, $dynamic:expr, $legacy:expr, $open:expr, $prepare:expr, $layer_draw:expr) => {
        struct $case;
        impl Conformance for $case {
            const NAME: &'static str = $name;
            const FAMILY: Family = Family::MENU;
            const PARTS: &'static [Part] = <$ty>::PARTS;
            type State = MenuState;
            type Action = MenuAction;
            type Cmd = MenuCmd;
            fn caps() -> Caps { Caps::FOCUSABLE | Caps::ACTIVATES | Caps::OVERLAY }
            fn id() -> Id { $id }
            fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
                let component = ($make)().patch_part(patch_of(f));
                ($prepare)(cx, &component);
                component.update(cx, st)
            }
            fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
                let component = ($make)().patch_part(patch_of(f));
                let layer_drawn = ($layer_draw)(ui, st, &component);
                if !layer_drawn { component.draw(ui, area, st); }
            }
            fn activation_chords() -> &'static [Chord] {
                const C: &[Chord] = &[Chord::key(KeyCode::Enter)]; C
            }
            fn activation_part() -> PartRef { $part }
            fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
                ($akey)(action)
            }
            fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
                ($make)().bindings(state)
            }
            fn legacy_key_chords() -> &'static [Chord] { $legacy }
            fn dynamic_bindings(_fixture: &Fixture) -> Vec<(ActionKey, Chord)> { $dynamic }
            fn open_chord() -> Option<Chord> { Some($open) }
            fn layer_id() -> Option<Id> { Some($id) }
            fn mono_states() -> &'static [StateFlags] {
                const S: &[StateFlags] = &[StateFlags::empty(), StateFlags::FOCUSED, StateFlags::PRESSED]; S
            }
            fn mono_narrowing_reason() -> &'static str {
                "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: fixture rows are enabled and menus expose no validation or readiness"
            }
        }
    };
}

menu_case!(
    ContextMenuCase,
    "context_menu",
    CONTEXT_MENU,
    ContextMenu<'static>,
    PartRef::item(Part::ROW, ItemKey::index(0)),
    |action: &MenuAction| matches!(action, MenuAction::Chosen(ActionKey::SAVE))
        .then_some(ItemKey::index(0)),
    || {
        ContextMenu::new(
            CONTEXT_MENU,
            &MENU_ITEMS,
            Anchor::Screen(ScreenAlign::Center),
        )
    },
    vec![(ActionKey::SAVE, Chord::key(KeyCode::Char('s')))],
    CONTEXT_LEGACY_CHORDS,
    Chord::key(KeyCode::F(4)),
    |cx: &mut Cx<'_>, component: &ContextMenu<'_>| {
        if cx.intents(CONTEXT_MENU).any(
            |intent| matches!(intent, Intent::Key(key) if Chord::key(KeyCode::F(4)).matches(&key)),
        ) {
            cx.open_layer(CONTEXT_MENU, component.layer(cx));
        }
    },
    |ui: &mut Ui<'_>, st: &MenuState, component: &ContextMenu<'_>| {
        ui.layer(CONTEXT_MENU, |ui, area| {
            component.draw(ui, area, st);
        })
        .is_some()
    }
);
menu_case!(
    MenuBarCase,
    "menu_bar",
    MENU_BAR,
    MenuBar<'static>,
    PartRef::item(Part::TITLE, ItemKey::index(0)),
    |action: &MenuAction| matches!(action, MenuAction::Opened(0)).then_some(ItemKey::index(0)),
    || { MenuBar::new(MENU_BAR, &MENUS) },
    Vec::new(),
    &[],
    Chord::key(KeyCode::Enter),
    |_cx: &mut Cx<'_>, _component: &MenuBar<'_>| {},
    |_ui: &mut Ui<'_>, _st: &MenuState, _component: &MenuBar<'_>| false
);

#[test]
fn open_menu_bar_dropdown_consumes_dynamic_item_binding() {
    use junie_tui_testing::conformance::driver::CaseApp;

    let mut harness = Harness::new(
        CaseApp::<MenuBarCase>::new(Fixture::default()),
        Theme::junie(),
        40,
        12,
    );
    assert!(harness.tab_to(MENU_BAR));
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app_mut().last.take(), Some(MenuAction::Opened(0)));
    let response = harness.key(KeyCode::Char('s'));
    assert!(response.is_consumed());
    assert_eq!(
        harness.app_mut().last.take(),
        Some(MenuAction::Chosen(ActionKey::SAVE))
    );
}

const HELP_OVERLAY: Id = Id::root("conformance.help_overlay");
struct HelpOverlayCase;
impl Conformance for HelpOverlayCase {
    const NAME: &'static str = "help_overlay";
    const FAMILY: Family = Family::HELP;
    const PARTS: &'static [Part] = HelpOverlay::PARTS;
    type State = HelpOverlayState;
    type Action = HelpAction;
    type Cmd = HelpCmd;
    fn caps() -> Caps {
        Caps::FOCUSABLE | Caps::OVERLAY | Caps::TRAPS_FOCUS | Caps::SCROLLS
    }
    fn id() -> Id {
        HELP_OVERLAY
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let layer = HintLayer::from_bindings(&[Binding {
            action: ActionKey::custom("help-case.close"),
            chord: Some(Chord::key(KeyCode::Enter)),
            cmd: HelpCmd::Close,
            label: "Close",
            priority: 80,
            visible: true,
        }]);
        let sections: Vec<_> = f
            .rows
            .iter()
            .map(|row| HelpSection::new(row.label.as_str(), &layer))
            .collect();
        let help = HelpOverlay::new(HELP_OVERLAY, "Application", &sections);
        if cx.intents(HELP_OVERLAY).any(
            |intent| matches!(intent, Intent::Key(key) if Chord::key(KeyCode::F(4)).matches(&key)),
        ) {
            cx.open_layer(HELP_OVERLAY, help.layer(cx));
            return Response::changed().for_id(HELP_OVERLAY);
        }
        help.update(cx, st)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let layer = HintLayer::from_bindings(&[Binding {
            action: ActionKey::custom("help-case.close"),
            chord: Some(Chord::key(KeyCode::Enter)),
            cmd: HelpCmd::Close,
            label: "Close",
            priority: 80,
            visible: true,
        }]);
        let sections: Vec<_> = f
            .rows
            .iter()
            .map(|row| HelpSection::new(row.label.as_str(), &layer))
            .collect();
        let help = HelpOverlay::new(HELP_OVERLAY, "Application", &sections).patch_part(patch_of(f));
        let layer_drawn = ui.layer(HELP_OVERLAY, |ui, layer_area| {
            help.draw(ui, layer_area, st);
        });
        if layer_drawn.is_none() {
            help.draw(ui, area, st);
        }
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        const EMPTY: &[HelpSection<'static>] = &[];
        HelpOverlay::new(HELP_OVERLAY, "Application", EMPTY).bindings(state)
    }
    fn legacy_key_chords() -> &'static [Chord] {
        const CHORDS: &[Chord] = &[Chord::key(KeyCode::F(4))];
        CHORDS
    }
    fn open_chord() -> Option<Chord> {
        Some(Chord::key(KeyCode::F(4)))
    }
    fn layer_id() -> Option<Id> {
        Some(HELP_OVERLAY)
    }
    fn prepare_scroll_fixture(fixture: &mut Fixture) {
        for value in 5..12 {
            fixture.rows.push(FixtureRow {
                key: ItemKey::num(100 + value),
                label: format!("section {value}"),
                meta: String::new(),
                disabled: false,
            });
        }
    }
    fn mono_states() -> &'static [StateFlags] {
        const S: &[StateFlags] = &[StateFlags::empty(), StateFlags::FOCUSED];
        S
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED DISABLED ERROR WARNING EDITING BUSY ACTIVE: help rows are decorative reference data"
    }
}

const WIZARD: Id = Id::root("conformance.wizard");

struct WizardCase;

impl Conformance for WizardCase {
    const NAME: &'static str = "wizard";
    const FAMILY: Family = Family::WIZARD;
    const PARTS: &'static [Part] = Wizard::PARTS;
    type State = WizardState;
    type Action = WizardAction;
    type Cmd = WizardCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE
    }
    fn id() -> Id {
        WIZARD
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let steps = [
            WizardStep::new(ItemKey::num(1), "Account"),
            WizardStep::new(ItemKey::num(2), "Details").enabled(!f.disabled),
        ];
        Wizard::new(WIZARD, &steps).update(cx, st)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let steps = [
            WizardStep::new(ItemKey::num(1), "Account"),
            WizardStep::new(ItemKey::num(2), "Details").enabled(!f.disabled),
        ];
        let wizard = Wizard::new(WIZARD, &steps).patch_part(patch_of(f));
        wizard.draw(ui, area, st);
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        Wizard::new(WIZARD, &[]).bindings(state)
    }
    fn activation_chords() -> &'static [Chord] {
        const CHORDS: &[Chord] = &[Chord::key(KeyCode::Right)];
        CHORDS
    }
    fn activation_part() -> PartRef {
        PartRef::item(Part::LABEL, ItemKey::num(2))
    }
    fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
        match action {
            WizardAction::Moved(key) | WizardAction::Finish(key) => Some(*key),
        }
    }
    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
        ];
        STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED ERROR WARNING EDITING BUSY ACTIVE: the wizard owns navigation; step-specific flags are declaration state"
    }
}

const PICKER_CHAIN: Id = Id::root("conformance.picker_chain");

#[derive(Clone, Debug, PartialEq, Eq)]
struct PickerChainCaseState(PickerChainState);

impl Default for PickerChainCaseState {
    fn default() -> Self {
        let mut state = PickerChainState::default();
        state.enter(ItemKey::num(1));
        state.enter(ItemKey::num(2));
        Self(state)
    }
}

struct PickerChainCase;

impl Conformance for PickerChainCase {
    const NAME: &'static str = "picker_chain";
    const FAMILY: Family = Family::PICKER;
    const PARTS: &'static [Part] = PickerChain::PARTS;
    type State = PickerChainCaseState;
    type Action = PickerChainAction;
    type Cmd = PickerChainCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE | Caps::REPORTS_STATUS
    }
    fn id() -> Id {
        PICKER_CHAIN
    }
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let stages = [
            PickerStage::new(ItemKey::num(1), "Account"),
            PickerStage::new(ItemKey::num(2), "Details").status(f.status()),
        ];
        PickerChain::new(PICKER_CHAIN, &stages).update(cx, &mut st.0)
    }
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let stages = [
            PickerStage::new(ItemKey::num(1), "Account"),
            PickerStage::new(ItemKey::num(2), "Details").status(f.status()),
        ];
        let chain = PickerChain::new(PICKER_CHAIN, &stages).patch_part(patch_of(f));
        chain.draw(ui, area, &st.0);
    }
    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        PickerChain::new(PICKER_CHAIN, &[]).bindings(state)
    }
    fn activation_chords() -> &'static [Chord] {
        const CHORDS: &[Chord] = &[Chord::key(KeyCode::Backspace)];
        CHORDS
    }
    fn activation_part() -> PartRef {
        PartRef::item(Part::LABEL, ItemKey::num(1))
    }
    fn action_key_of(action: &Self::Action) -> Option<ItemKey> {
        match action {
            PickerChainAction::Back(key) | PickerChainAction::Retry(key) => Some(*key),
        }
    }
    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::PRESSED,
            StateFlags::BUSY,
            StateFlags::ERROR,
        ];
        STATES
    }
    fn mono_narrowing_reason() -> &'static str {
        "SELECTED DISABLED WARNING EDITING ACTIVE: chain readiness is busy/loading/error; selection belongs to nested picker content"
    }
}

const FORM: Id = Id::root("conformance.form");
const FORM_SECRET: Id = Id::root("conformance.form.secret");
const FORM_NOTE: Id = Id::root("conformance.form.note");
const FORM_NOTE_2: Id = Id::root("conformance.form.note-2");
const FORM_NOTES: &[(&str, Role)] = &[
    ("Form guidance", Role::Fg(FgStep::Muted)),
    ("Values remain owner-held", Role::Fg(FgStep::Muted)),
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FormCaseState {
    form: FormState,
    data: FormCaseData,
}

#[derive(Default)]
struct FormCaseData {
    secret: Secret,
    snapshot: Option<SecretSnapshot>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SecretSnapshot {
    len: usize,
    fingerprint: [u8; 8],
}

impl FormCaseData {
    fn secret_snapshot(&self) -> SecretSnapshot {
        self.snapshot.unwrap_or_else(|| SecretSnapshot {
            len: self.secret.len(),
            fingerprint: self.secret.fingerprint(),
        })
    }
}

impl core::fmt::Debug for FormCaseData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FormCaseData")
            .field("secret", &"[redacted]")
            .finish()
    }
}

impl Clone for FormCaseData {
    fn clone(&self) -> Self {
        // `Secret` deliberately has no public plaintext-copy operation. Keep
        // only a non-reversible identity token in snapshots used by the
        // generic driver, never a redacted string that could be committed.
        FormCaseData {
            secret: Secret::default(),
            snapshot: Some(self.secret_snapshot()),
        }
    }
}

impl PartialEq for FormCaseData {
    fn eq(&self, other: &Self) -> bool {
        self.secret_snapshot() == other.secret_snapshot()
    }
}

impl Eq for FormCaseData {}

impl FormData for FormCaseData {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == FORM_SECRET {
            FieldRef::Secret(&self.secret)
        } else {
            FieldRef::Note(FORM_NOTES)
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == FORM_SECRET {
            FieldMut::Secret(&mut self.secret)
        } else {
            FieldMut::ReadOnly
        }
    }
}

const FORM_ACTIONS: &[Action<'static>] =
    &[Action::new(ActionKey::SAVE, "Save").chord(Chord::key(KeyCode::F(6)))];

fn form_fields(_f: &Fixture) -> [FieldSpec<'_>; 3] {
    let secret = TextInput::new(FORM_SECRET).secret(SecretPolicy::default());
    [
        FieldSpec::new(FORM_SECRET, "Secret", FieldKind::Text(secret)),
        FieldSpec::new(FORM_NOTE, "", FieldKind::Note),
        FieldSpec::new(FORM_NOTE_2, "", FieldKind::Note),
    ]
}

struct FormCase;

impl Conformance for FormCase {
    const NAME: &'static str = "form";
    const FAMILY: Family = Family::FORM;
    const PARTS: &'static [Part] = Form::PARTS;
    type State = FormCaseState;
    type Action = FormAction;
    type Cmd = TextCmd;

    fn caps() -> Caps {
        Caps::EDITS | Caps::SECRET | Caps::FOCUSABLE | Caps::SCROLLS | Caps::TYPES
    }

    fn id() -> Id {
        FORM
    }

    fn control_id() -> Id {
        FORM_SECRET
    }

    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action> {
        let fields = form_fields(f);
        Form::new(FORM, &fields)
            .actions(FORM_ACTIONS)
            .update(cx, &mut st.form, &mut st.data)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture) {
        let fields = form_fields(f);
        Form::new(FORM, &fields)
            .actions(FORM_ACTIONS)
            .patch_part(patch_of(f))
            .draw(ui, area, &st.form, &st.data);
    }

    fn bindings(state: BindingState) -> &'static [Binding<Self::Cmd>] {
        TextInput::new(FORM_SECRET)
            .secret(SecretPolicy::default())
            .bindings(state)
    }

    fn prepare_scroll_fixture(fixture: &mut Fixture) {
        fixture.area.height = 4;
    }

    fn secret_bytes() -> &'static str {
        "form-secret"
    }

    fn dynamic_bindings(_fixture: &Fixture) -> Vec<(ActionKey, Chord)> {
        vec![(ActionKey::SAVE, Chord::key(KeyCode::F(6)))]
    }

    fn dynamic_binding_id(_action: ActionKey) -> Id {
        FORM.part(Part::ACTIONS).index(0)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: &[StateFlags] = &[
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::EDITING,
        ];
        STATES
    }

    fn mono_setup_chords(state: StateFlags) -> &'static [Chord] {
        const EDIT: &[Chord] = &[Chord::key(KeyCode::Char('x'))];
        if state.contains(StateFlags::EDITING) {
            EDIT
        } else {
            &[]
        }
    }

    fn mono_narrowing_reason() -> &'static str {
        "SELECTED PRESSED DISABLED ERROR WARNING BUSY ACTIVE: Form's real secret TextInput child owns focus and editing; this fixture has no selection, disabled, validation, readiness, or active-item state"
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
    props_list => PropsListCase,
    text_area => TextAreaCase,
    select => SelectCase,
    radio_group => RadioGroupCase,
    checkbox => CheckboxCase,
    toggle => ToggleCase,
    chip_bar => ChipBarCase,
    status_bar => StatusBarCase,
    hint_bar => HintBarCase,
    derived_hint_bar => DerivedHintBarCase,
    key_hint => KeyHintCase,
    progress_bar => ProgressBarCase,
    spinner => SpinnerCase,
    meter => MeterCase,
    empty => EmptyCase,
    brand => BrandCase,
    panel => PanelCase,
    split_pane => SplitPaneCase,
    text_viewport => TextViewportCase,
    diff_view => DiffViewCase,
    code_editor => CodeEditorCase,
    tree => TreeCase,
    nav_list => NavListCase,
    steps => StepsCase,
    too_small => TooSmallCase,
    grid => GridCase,
    filter_list => FilterListCase,
    picker => PickerCase,
    completion => CompletionCase,
    context_menu => ContextMenuCase,
    help_overlay => HelpOverlayCase,
    menu_bar => MenuBarCase,
    picker_chain => PickerChainCase,
    wizard => WizardCase,
    form => FormCase,
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
        Caps::FOCUSABLE
            | Caps::ACTIVATES
            | Caps::DISABLEABLE
            | Caps::EDITS
            | Caps::COLLECTION
            | Caps::SELECTS,
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

#[test]
fn activation_gesture_defaults_to_click_and_grid_declares_double_click() {
    assert_eq!(ProbeCase::activation_gesture(), PointerGesture::Click);
    assert_eq!(GridCase::activation_gesture(), PointerGesture::DoubleClick);
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
    clean::<PropsListCase>(&states);
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
    clean::<PanelCase>(&states);
    clean::<SplitPaneCase>(&states);
    clean::<TextViewportCase>(&states);
    clean::<DiffViewCase>(&states);
    clean::<CodeEditorCase>(&states);
    clean::<TreeCase>(&states);
    clean::<NavListCase>(&states);
    clean::<StepsCase>(&states);
    clean::<TooSmallCase>(&states);
    clean::<GridCase>(&states);
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
            action: ActionKey::custom("conflict.alias.visible"),
            chord: Some(Chord::key(KeyCode::Char('d'))),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 60,
            visible: true,
        },
        Binding {
            action: ActionKey::custom("conflict.alias.hidden"),
            chord: Some(Chord::key(KeyCode::Char('d'))),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 10,
            visible: false,
        },
    ];
    const CLASH: [Binding<ProbeCmd>; 2] = [
        Binding {
            action: ActionKey::custom("conflict.first"),
            chord: Some(Chord::key(KeyCode::Char('d'))),
            cmd: ProbeCmd::Activate,
            label: "Delete",
            priority: 60,
            visible: true,
        },
        Binding {
            action: ActionKey::custom("conflict.second"),
            chord: Some(Chord::key(KeyCode::Char('d'))),
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

    // Hidden effective duplicates conflict too: visibility controls hints,
    // never dispatch ambiguity.
    assert_eq!(binding_conflicts(OWNER, KeyPhase::Bubble, &ALIAS).len(), 1);

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
    degenerate::<PropsListCase>();
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
    degenerate::<PanelCase>();
    degenerate::<SplitPaneCase>();
    degenerate::<TextViewportCase>();
    degenerate::<DiffViewCase>();
    degenerate::<CodeEditorCase>();
    degenerate::<TreeCase>();
    degenerate::<NavListCase>();
    degenerate::<StepsCase>();
    degenerate::<TooSmallCase>();
    degenerate::<GridCase>();

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

impl junie_tui::App for ZeroLayer {
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

    use std::collections::BTreeSet;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct StyleProvenance {
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
    }

    fn declare(out: &mut BTreeSet<StyleProvenance>, owner: Id, family: Family, parts: &[Part]) {
        for &part in parts {
            out.insert(StyleProvenance {
                owner,
                family,
                variant: Variant::DEFAULT,
                part,
            });
        }
    }

    /// Root-owner style declarations, including composed painters that share
    /// the root id. Family/variant are provenance, not an extra part allowlist.
    fn declared<C: Conformance>() -> BTreeSet<StyleProvenance> {
        let owner = C::id();
        let mut out = BTreeSet::new();
        declare(&mut out, owner, C::FAMILY, C::PARTS);
        match C::NAME {
            // Field intentionally hosts the control under the same id.
            "field" => declare(&mut out, owner, Family::INPUT, TextInput::PARTS),
            // Form owns its chrome and scrollbar under FORM; field chrome is
            // still addressed by FORM while the real control has its own id.
            "form" => {
                declare(&mut out, owner, Family::FIELD, &[Part::LABEL, Part::HELP]);
                declare(&mut out, owner, Family::SCROLLBAR, ScrollRegion::PARTS);
            }
            // These compositions keep the scrollbar's default family. Other
            // compositions explicitly inherit their owner's family.
            "code_editor" | "completion" | "filter_list" | "grid" | "list" | "picker"
            | "select" | "steps" | "text_area" | "tree" => {
                declare(&mut out, owner, Family::SCROLLBAR, ScrollRegion::PARTS)
            }
            // DiffView delegates all painting to TextViewport, including the
            // viewport recipe provenance.
            "diff_view" => {
                out.retain(|entry| entry.family != C::FAMILY);
                declare(&mut out, owner, Family::VIEWPORT, TextViewport::PARTS);
            }
            "hint_bar" | "derived_hint_bar" => {
                declare(&mut out, owner, Family::KEYHINT, &[Part::KEY, Part::ACTION]);
            }
            _ => {}
        }
        out
    }

    fn fixtures<C: Conformance>() -> Vec<Fixture> {
        let mut out = vec![Fixture::default()];
        let mut overflow = Fixture::default();
        C::prepare_scroll_fixture(&mut overflow);
        out.push(overflow);
        for &state in C::mono_states() {
            let mut fixture = C::mono_fixture(state).force(state);
            C::prepare_scroll_fixture(&mut fixture);
            out.push(fixture);
        }
        out
    }

    /// Root-owner style queries across semantic and overflow fixtures.
    /// Nested owner ids have their own registered conformance case.
    fn styled<C: Conformance>() -> BTreeSet<StyleProvenance> {
        let mut out = BTreeSet::new();
        for fixture in fixtures::<C>() {
            let mut scene = Scene::new(C::NAME, fixture.theme.clone(), fixture.color, 40, 12);
            let state = C::State::default();
            scene.draw(|ui, _| {
                C::draw(ui, fixture.area, &state, &fixture);
                for &(owner, family, variant, part, _) in ui.styled_queries() {
                    if owner == C::id() {
                        out.insert(StyleProvenance {
                            owner,
                            family,
                            variant,
                            part,
                        });
                    }
                }
            });
        }
        out
    }

    fn undeclared<C: Conformance>() -> Vec<StyleProvenance> {
        styled::<C>()
            .difference(&declared::<C>())
            .copied()
            .collect()
    }

    fn check<C: Conformance>() {
        let unexpected = undeclared::<C>();
        assert!(
            unexpected.is_empty(),
            "{}: styled undeclared provenance {unexpected:?}; declared {:?}",
            C::NAME,
            declared::<C>()
        );
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
                "props_list",
                "text_area",
                "select",
                "radio_group",
                "checkbox",
                "toggle",
                "chip_bar",
                "status_bar",
                "hint_bar",
                "derived_hint_bar",
                "key_hint",
                "progress_bar",
                "spinner",
                "meter",
                "empty",
                "brand",
                "panel",
                "split_pane",
                "text_viewport",
                "diff_view",
                "code_editor",
                "tree",
                "nav_list",
                "steps",
                "too_small",
                "grid",
                "filter_list",
                "picker",
                "completion",
                "context_menu",
                "help_overlay",
                "menu_bar",
                "picker_chain",
                "wizard",
                "form",
            ]
        );
    }

    #[test]
    fn declared_parts_are_the_parts_actually_styled() {
        check::<ProbeCase>();
        check::<ButtonCase>();
        check::<TextInputCase>();
        check::<FieldCase>();
        check::<ListCase>();
        check::<TabsCase>();
        check::<DialogCase>();
        check::<ScrollRegionCase>();
        check::<PropsCase>();
        check::<PropsListCase>();
        check::<TextAreaCase>();
        check::<SelectCase>();
        check::<RadioGroupCase>();
        check::<CheckboxCase>();
        check::<ToggleCase>();
        check::<ChipBarCase>();
        check::<StatusBarCase>();
        check::<HintBarCase>();
        check::<DerivedHintBarCase>();
        check::<KeyHintCase>();
        check::<ProgressBarCase>();
        check::<SpinnerCase>();
        check::<MeterCase>();
        check::<EmptyCase>();
        check::<BrandCase>();
        check::<PanelCase>();
        check::<SplitPaneCase>();
        check::<TextViewportCase>();
        check::<DiffViewCase>();
        check::<CodeEditorCase>();
        check::<TreeCase>();
        check::<NavListCase>();
        check::<StepsCase>();
        check::<TooSmallCase>();
        check::<GridCase>();
        check::<FilterListCase>();
        check::<PickerCase>();
        check::<CompletionCase>();
        check::<ContextMenuCase>();
        check::<HelpOverlayCase>();
        check::<MenuBarCase>();
        check::<PickerChainCase>();
        check::<WizardCase>();
        check::<FormCase>();
    }

    const UNDECLARED: Id = Id::root("conformance.registry.undeclared");

    struct UndeclaredPartCase;

    impl Conformance for UndeclaredPartCase {
        const NAME: &'static str = "undeclared_part";
        const FAMILY: Family = Family::BUTTON;
        const PARTS: &'static [Part] = &[Part::CONTAINER];
        type State = ();
        type Action = ();
        type Cmd = ();

        fn caps() -> Caps {
            Caps::empty()
        }

        fn id() -> Id {
            UNDECLARED
        }

        fn update(_cx: &mut Cx<'_>, _st: &mut (), _f: &Fixture) -> Response<()> {
            Response::ignored()
        }

        fn draw(ui: &mut Ui<'_>, area: Rect, _st: &(), _f: &Fixture) {
            let _ = PartStyle::new().style(
                ui,
                UNDECLARED,
                Family::BUTTON,
                Variant::DEFAULT,
                Part::DETAIL,
                StateFlags::empty(),
            );
            ui.fill(area, ui.surface_style());
        }
    }

    #[test]
    fn undeclared_style_provenance_is_rejected() {
        assert_eq!(
            undeclared::<UndeclaredPartCase>(),
            vec![StyleProvenance {
                owner: UNDECLARED,
                family: Family::BUTTON,
                variant: Variant::DEFAULT,
                part: Part::DETAIL,
            }]
        );
    }
}
